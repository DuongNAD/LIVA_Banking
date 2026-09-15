//! Banking command implementations for LIVA Banking Reconciliation Engine.
//!
//! Exposes IPC commands:
//! - `banking_get_overview`
//! - `statement_ingest_file`
//! - `banking_run_reconciliation`
//! - `banking_get_reconciliation_matrix`
//! - `reconciliation_resolve_hitl`
//! - `banking_get_compliance_status`

use rusqlite::{OptionalExtension, params};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::banking::compliance::audit_ledger::AuditLedger;
use crate::banking::compliance::security::{get_audit_key, get_compliance_status};
use crate::banking::models::*;
use crate::banking::parser::sniff_and_parse;
use crate::banking::reconciliation::ReconciliationEngine;

pub const OWNED: &[&str] = &[
    "banking_get_overview",
    "banking:get_overview",
    "statement_ingest_file",
    "statement:ingest_file",
    "banking:import_statement",
    "banking_run_reconciliation",
    "banking:run_reconciliation",
    "banking_get_reconciliation_matrix",
    "banking:get_reconciliation_matrix",
    "reconciliation_resolve_hitl",
    "banking:resolve_hitl",
    "banking_get_compliance_status",
    "banking:get_compliance_status",
    "banking:seed_demo_data",
];

pub fn owns(command: &str) -> bool {
    OWNED.contains(&command)
}

pub async fn handle(state: Arc<AppState>, command: &str, payload: Value) -> Result<Value, String> {
    match command {
        "banking_get_overview" | "banking:get_overview" => get_overview(state).await,
        "statement_ingest_file" | "statement:ingest_file" | "banking:import_statement" => {
            ingest_statement_file(state, payload).await
        }
        "banking_run_reconciliation" | "banking:run_reconciliation" => {
            run_reconciliation(state).await
        }
        "banking_get_reconciliation_matrix" | "banking:get_reconciliation_matrix" => {
            get_reconciliation_matrix(state, payload).await
        }
        "reconciliation_resolve_hitl" | "banking:resolve_hitl" => {
            resolve_hitl(state, payload).await
        }
        "banking_get_compliance_status" | "banking:get_compliance_status" => {
            get_compliance(state).await
        }
        "banking:seed_demo_data" => seed_demo_data(state).await,
        _ => Err(format!("Unknown banking command: {command}")),
    }
}

// ---------------------------------------------------------------------------
// 1. Overview
// ---------------------------------------------------------------------------

async fn get_overview(state: Arc<AppState>) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let conn = state
            .db
            .readers
            .get()
            .map_err(|e| format!("Failed to acquire read connection: {e}"))?;

        // Ensure accounts exist (seed defaults if empty)
        let count_acc: i64 = conn
            .query_row("SELECT COUNT(*) FROM bank_accounts", [], |r| r.get(0))
            .unwrap_or(0);

        if count_acc == 0 {
            drop(conn);
            let mut writer = state
                .db
                .writer
                .get()
                .map_err(|e| format!("Failed to acquire write connection: {e}"))?;
            seed_default_accounts_and_data(&mut writer, &state)?;
            return fetch_overview_dto(&state);
        }

        fetch_overview_dto(&state)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

fn fetch_overview_dto(state: &AppState) -> Result<Value, String> {
    let conn = state
        .db
        .readers
        .get()
        .map_err(|e| format!("Read connection error: {e}"))?;

    let mut vcb_bal = 1_450_230_000u64;
    let mut tcb_bal = 785_600_000u64;
    let mut bidv_bal = 350_000_000u64;

    let mut stmt = conn
        .prepare("SELECT bank_code, current_balance FROM bank_accounts")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? as u64))
        })
        .map_err(|e| e.to_string())?;

    for (code, bal) in rows.flatten() {
        match code.as_str() {
            "VCB" => vcb_bal = bal,
            "TCB" => tcb_bal = bal,
            "BIDV" => bidv_bal = bal,
            _ => {}
        }
    }

    let total_bal = vcb_bal + tcb_bal + bidv_bal;

    // Reconciliation stats
    let total_tx: usize = conn
        .query_row("SELECT COUNT(*) FROM bank_transactions", [], |r| r.get(0))
        .unwrap_or(0);

    let matched_count: usize = conn
        .query_row(
            "SELECT COUNT(*) FROM bank_transactions WHERE reconciled_status = 'MATCHED'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let hitl_count: usize = conn
        .query_row(
            "SELECT COUNT(*) FROM bank_transactions WHERE reconciled_status = 'PENDING_HITL'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let discrepancy_count: usize = conn
        .query_row(
            "SELECT COUNT(*) FROM bank_transactions WHERE reconciled_status = 'DISCREPANCY'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let matched_ratio = if total_tx > 0 {
        (matched_count as f64 / total_tx as f64) * 100.0
    } else {
        99.8 // Mockup default
    };

    // Recent transactions
    let mut tx_stmt = conn
        .prepare(
            "SELECT bt.id, bt.statement_id, bt.account_id, ba.bank_code, bt.tx_date, bt.value_date, \
                    bt.doc_ref, bt.tx_type, bt.amount, bt.balance_after, bt.counterparty_name, \
                    bt.counterparty_bank, bt.narration_enc, bt.reconciled_status, bt.created_at \
             FROM bank_transactions bt \
             LEFT JOIN bank_accounts ba ON bt.account_id = ba.id \
             ORDER BY bt.tx_date DESC LIMIT 20",
        )
        .map_err(|e| e.to_string())?;

    let tx_rows = tx_stmt
        .query_map([], |r| {
            let tx_type_str: String = r.get(7)?;
            let tx_type = tx_type_str.parse().unwrap_or(TransactionType::Credit);
            let status_str: String = r.get(13)?;
            let status = status_str
                .parse()
                .unwrap_or(ReconciliationStatus::Unmatched);
            let enc_narration: String = r.get(12)?;
            let dec_narration = state.crypto.decrypt_read(&enc_narration);

            Ok(BankTransactionRow {
                id: r.get(0)?,
                statement_id: r.get(1)?,
                account_id: r.get(2)?,
                bank_code: r.get(3).unwrap_or_else(|_| "VCB".to_string()),
                tx_date: r.get(4)?,
                value_date: r.get(5)?,
                doc_ref: r.get(6)?,
                tx_type,
                amount: r.get::<_, i64>(8)? as u64,
                balance_after: r.get::<_, Option<i64>>(9)?.map(|b| b as u64),
                counterparty_account: None,
                counterparty_name: r.get(10)?,
                counterparty_bank: r.get(11)?,
                narration: dec_narration,
                reconciled_status: status,
                reconciled_match_id: None,
                created_at: r.get(14)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut recent_transactions = Vec::new();
    for r in tx_rows.flatten() {
        recent_transactions.push(r);
    }

    // 30-day forecast projection
    let mut rolling_forecast = Vec::new();
    let mut running_proj = total_bal as i64;
    for day in 1..=30 {
        let inflow = if day % 5 == 0 {
            250_000_000
        } else {
            45_000_000
        };
        let outflow = if day == 15 { 450_000_000 } else { 35_000_000 };
        running_proj += (inflow as i64) - (outflow as i64);
        rolling_forecast.push(DailyCashflowForecast {
            date: format!("2026-09-{:02}", (day % 30) + 1),
            expected_inflow: inflow,
            expected_outflow: outflow,
            projected_balance: running_proj,
            is_deficit_risk: running_proj < 500_000_000,
        });
    }

    let dto = BankingOverviewDto {
        total_balance: total_bal,
        vcb_balance: vcb_bal,
        tcb_balance: tcb_bal,
        bidv_balance: bidv_bal,
        matched_ratio,
        matched_count,
        total_count: total_tx,
        automatic_count: matched_count.saturating_sub(hitl_count),
        hitl_count,
        discrepancy_count,
        last_sync_time: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64,
        recent_transactions,
        rolling_forecast,
    };

    Ok(json!(dto))
}

// ---------------------------------------------------------------------------
// 2. Statement Ingestion
// ---------------------------------------------------------------------------

async fn ingest_statement_file(state: Arc<AppState>, payload: Value) -> Result<Value, String> {
    let file_path = payload
        .get("file_path")
        .or_else(|| payload.get("filePath"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'file_path' in payload".to_string())?
        .to_string();

    tokio::task::spawn_blocking(move || {
        let bytes = std::fs::read(&file_path)
            .map_err(|e| format!("Cannot read file '{file_path}': {e}"))?;

        let filename = std::path::Path::new(&file_path)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("statement.dat");

        // Parse statement
        let parsed = sniff_and_parse(&bytes, filename)
            .map_err(|e| format!("Statement parse error: {e}"))?;

        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let file_hash = hex::encode(hasher.finalize());

        let mut conn = state
            .db
            .writer
            .get()
            .map_err(|e| format!("Database write lock error: {e}"))?;

        let statement_id = format!("stmt_{}_{}", parsed.bank_code.to_lowercase(), Uuid::new_v4().simple());
        let account_id = format!("acc_{}", parsed.bank_code.to_lowercase());

        let now_ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        // Ensure bank account exists
        let acc_num = parsed.account_number.as_deref().unwrap_or("0011001234567");
        let enc_acc_num = state.crypto.encrypt(acc_num)?;
        let acc_name = parsed.account_name.as_deref().unwrap_or("DOANH NGHIEP DEMO");

        conn.execute(
            "INSERT OR IGNORE INTO bank_accounts (id, bank_code, account_number_enc, account_name, opening_balance, current_balance, last_synced_at, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                account_id,
                parsed.bank_code,
                enc_acc_num,
                acc_name,
                parsed.opening_balance.unwrap_or(1_000_000_000) as i64,
                parsed.closing_balance.unwrap_or(1_450_000_000) as i64,
                now_ts,
                now_ts
            ],
        )
        .map_err(|e| e.to_string())?;

        // Insert bank_statements
        conn.execute(
            "INSERT OR REPLACE INTO bank_statements (id, account_id, filename, file_hash, file_format, statement_from, statement_to, total_transactions, parsed_duration_ms, parsed_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                statement_id,
                account_id,
                filename,
                file_hash,
                filename.rsplit('.').next().unwrap_or("dat"),
                parsed.statement_from.unwrap_or(now_ts),
                parsed.statement_to.unwrap_or(now_ts),
                parsed.transactions.len() as i64,
                parsed.parse_duration_ms as i64,
                now_ts
            ],
        )
        .map_err(|e| e.to_string())?;

        // Insert bank_transactions (encrypting sensitive fields)
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        for item in &parsed.transactions {
            let tx_id = format!("tx_{}_{}", parsed.bank_code.to_lowercase(), Uuid::new_v4().simple());
            let enc_narration = state.crypto.encrypt(&item.narration)?;
            let enc_cp_acc = item
                .counterparty_account
                .as_ref()
                .map(|a| state.crypto.encrypt(a))
                .transpose()?;

            tx.execute(
                "INSERT INTO bank_transactions (id, statement_id, account_id, tx_date, value_date, doc_ref, tx_type, amount, balance_after, counterparty_account_enc, counterparty_name, counterparty_bank, narration_enc, reconciled_status, created_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, 'UNMATCHED', ?14)",
                params![
                    tx_id,
                    statement_id,
                    account_id,
                    item.tx_date,
                    item.value_date,
                    item.doc_ref,
                    item.tx_type.to_string(),
                    item.amount as i64,
                    item.balance_after.map(|b| b as i64),
                    enc_cp_acc,
                    item.counterparty_name,
                    item.counterparty_bank,
                    enc_narration,
                    now_ts
                ],
            )
            .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;

        // Audit log
        let audit_key = get_audit_key(&state);
        let audit_payload = format!(
            "Statement imported: filename={}, bank={}, tx_count={}",
            filename,
            parsed.bank_code,
            parsed.transactions.len()
        );
        let _ = AuditLedger::append(
            &conn,
            &audit_key,
            "STATEMENT_IMPORTED",
            "TauriDashboard",
            &audit_payload,
        );

        let res = StatementIngestResultDto {
            statement_id,
            bank_code: parsed.bank_code,
            filename: filename.to_string(),
            total_transactions: parsed.transactions.len(),
            parse_duration_ms: parsed.parse_duration_ms,
            opening_balance: parsed.opening_balance,
            closing_balance: parsed.closing_balance,
        };

        Ok(json!(res))
    })
    .await
    .map_err(|e| format!("Task error: {e}"))?
}

// ---------------------------------------------------------------------------
// 3. Run Reconciliation
// ---------------------------------------------------------------------------

async fn run_reconciliation(state: Arc<AppState>) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let mut conn = state
            .db
            .writer
            .get()
            .map_err(|e| format!("Write lock error: {e}"))?;

        // 1. Fetch unallocated bank transactions
        let mut bank_txs: Vec<BankTransactionRow> = Vec::new();
        {
            let mut stmt = conn
                .prepare(
                    "SELECT bt.id, bt.statement_id, bt.account_id, ba.bank_code, bt.tx_date, bt.value_date, \
                            bt.doc_ref, bt.tx_type, bt.amount, bt.balance_after, bt.counterparty_name, \
                            bt.counterparty_bank, bt.narration_enc, bt.reconciled_status, bt.created_at \
                     FROM bank_transactions bt \
                     LEFT JOIN bank_accounts ba ON bt.account_id = ba.id \
                     WHERE bt.reconciled_status = 'UNMATCHED'",
                )
                .map_err(|e| e.to_string())?;

            let rows = stmt
                .query_map([], |r| {
                    let tx_type_str: String = r.get(7)?;
                    let tx_type = tx_type_str.parse().unwrap_or(TransactionType::Credit);
                    let status_str: String = r.get(13)?;
                    let status = status_str.parse().unwrap_or(ReconciliationStatus::Unmatched);
                    let enc_narration: String = r.get(12)?;
                    let dec_narration = state.crypto.decrypt_read(&enc_narration);

                    Ok(BankTransactionRow {
                        id: r.get(0)?,
                        statement_id: r.get(1)?,
                        account_id: r.get(2)?,
                        bank_code: r.get(3).unwrap_or_else(|_| "VCB".to_string()),
                        tx_date: r.get(4)?,
                        value_date: r.get(5)?,
                        doc_ref: r.get(6)?,
                        tx_type,
                        amount: r.get::<_, i64>(8)? as u64,
                        balance_after: r.get::<_, Option<i64>>(9)?.map(|b| b as u64),
                        counterparty_account: None,
                        counterparty_name: r.get(10)?,
                        counterparty_bank: r.get(11)?,
                        narration: dec_narration,
                        reconciled_status: status,
                        reconciled_match_id: None,
                        created_at: r.get(14)?,
                    })
                })
                .map_err(|e| e.to_string())?;

            for r in rows.flatten() {
                bank_txs.push(r);
            }
        }

        // If no bank transactions, seed demo ones
        if bank_txs.is_empty() {
            seed_default_accounts_and_data(&mut conn, &state)?;
            return fetch_and_reconcile(&mut conn, &state);
        }

        // 2. Fetch unallocated internal ledger entries
        let mut ledger_entries: Vec<InternalLedgerEntry> = Vec::new();
        {
            let mut stmt = conn
                .prepare(
                    "SELECT id, account_id, doc_no, entry_date, entry_type, amount, partner_code, partner_name, description, reconciled_status, created_at \
                     FROM internal_ledger_entries WHERE reconciled_status = 'UNMATCHED'",
                )
                .map_err(|e| e.to_string())?;

            let rows = stmt
                .query_map([], |r| {
                    let entry_type_str: String = r.get(4)?;
                    let entry_type = entry_type_str.parse().unwrap_or(TransactionType::Credit);
                    let status_str: String = r.get(9)?;
                    let status = status_str.parse().unwrap_or(ReconciliationStatus::Unmatched);

                    Ok(InternalLedgerEntry {
                        id: r.get(0)?,
                        account_id: r.get(1)?,
                        doc_no: r.get(2)?,
                        entry_date: r.get(3)?,
                        entry_type,
                        amount: r.get::<_, i64>(5)? as u64,
                        partner_code: r.get(6)?,
                        partner_name: r.get(7)?,
                        description: r.get(8)?,
                        reconciled_status: status,
                        created_at: r.get(10)?,
                    })
                })
                .map_err(|e| e.to_string())?;

            for r in rows.flatten() {
                ledger_entries.push(r);
            }
        }

        // 3. Execute Deterministic 3-Tier Reconciliation
        let (matches, summary) = ReconciliationEngine::reconcile(&bank_txs, &ledger_entries);

        // 4. Persist matches and update statuses
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        for m in &matches {
            let ledger_ids_json = serde_json::to_string(&m.ledger_entry_ids).unwrap_or_default();
            tx.execute(
                "INSERT OR REPLACE INTO reconciliation_matches (id, bank_tx_id, ledger_entry_ids_json, match_type, confidence_score, matched_amount, discrepancy_amount, status, matched_by, matched_at, notes, hitl_token) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    m.id,
                    m.bank_tx_id,
                    ledger_ids_json,
                    m.match_type.to_string(),
                    m.confidence_score,
                    m.matched_amount as i64,
                    m.discrepancy_amount,
                    m.status,
                    m.matched_by,
                    m.matched_at,
                    m.notes,
                    m.hitl_token
                ],
            )
            .map_err(|e| e.to_string())?;

            // Update bank transaction status
            let new_tx_status = if m.status == "PENDING_HITL" {
                "PENDING_HITL"
            } else if m.discrepancy_amount != 0 {
                "DISCREPANCY"
            } else {
                "MATCHED"
            };

            tx.execute(
                "UPDATE bank_transactions SET reconciled_status = ?1, reconciled_match_id = ?2 WHERE id = ?3",
                params![new_tx_status, m.id, m.bank_tx_id],
            )
            .map_err(|e| e.to_string())?;

            // Update ledger entries status
            if m.status == "APPROVED" {
                for l_id in &m.ledger_entry_ids {
                    tx.execute(
                        "UPDATE internal_ledger_entries SET reconciled_status = 'MATCHED' WHERE id = ?1",
                        params![l_id],
                    )
                    .map_err(|e| e.to_string())?;
                }
            }
        }
        tx.commit().map_err(|e| e.to_string())?;

        // 5. Audit Log
        let audit_key = get_audit_key(&state);
        let log_msg = format!(
            "Reconciliation executed: total={}, matched={}, hitl={}, disc={}",
            summary.total_bank_transactions,
            summary.total_matched_count,
            summary.pending_hitl_count,
            summary.discrepancy_count
        );
        let _ = AuditLedger::append(&conn, &audit_key, "MATCH_AUTO", "ReconciliationEngine", &log_msg);

        Ok(json!(summary))
    })
    .await
    .map_err(|e| format!("Task error: {e}"))?
}

fn fetch_and_reconcile(conn: &mut rusqlite::Connection, state: &AppState) -> Result<Value, String> {
    let mut bank_txs: Vec<BankTransactionRow> = Vec::new();
    let mut stmt = conn
        .prepare(
            "SELECT bt.id, bt.statement_id, bt.account_id, ba.bank_code, bt.tx_date, bt.value_date, \
                    bt.doc_ref, bt.tx_type, bt.amount, bt.balance_after, bt.counterparty_name, \
                    bt.counterparty_bank, bt.narration_enc, bt.reconciled_status, bt.created_at \
             FROM bank_transactions bt \
             LEFT JOIN bank_accounts ba ON bt.account_id = ba.id",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |r| {
            let tx_type_str: String = r.get(7)?;
            let tx_type = tx_type_str.parse().unwrap_or(TransactionType::Credit);
            let status_str: String = r.get(13)?;
            let status = status_str
                .parse()
                .unwrap_or(ReconciliationStatus::Unmatched);
            let enc_narration: String = r.get(12)?;
            let dec_narration = state.crypto.decrypt_read(&enc_narration);

            Ok(BankTransactionRow {
                id: r.get(0)?,
                statement_id: r.get(1)?,
                account_id: r.get(2)?,
                bank_code: r.get(3).unwrap_or_else(|_| "VCB".to_string()),
                tx_date: r.get(4)?,
                value_date: r.get(5)?,
                doc_ref: r.get(6)?,
                tx_type,
                amount: r.get::<_, i64>(8)? as u64,
                balance_after: r.get::<_, Option<i64>>(9)?.map(|b| b as u64),
                counterparty_account: None,
                counterparty_name: r.get(10)?,
                counterparty_bank: r.get(11)?,
                narration: dec_narration,
                reconciled_status: status,
                reconciled_match_id: None,
                created_at: r.get(14)?,
            })
        })
        .map_err(|e| e.to_string())?;

    for r in rows.flatten() {
        bank_txs.push(r);
    }

    let mut ledger_entries: Vec<InternalLedgerEntry> = Vec::new();
    let mut l_stmt = conn
        .prepare(
            "SELECT id, account_id, doc_no, entry_date, entry_type, amount, partner_code, partner_name, description, reconciled_status, created_at \
             FROM internal_ledger_entries",
        )
        .map_err(|e| e.to_string())?;

    let l_rows = l_stmt
        .query_map([], |r| {
            let entry_type_str: String = r.get(4)?;
            let entry_type = entry_type_str.parse().unwrap_or(TransactionType::Credit);
            let status_str: String = r.get(9)?;
            let status = status_str
                .parse()
                .unwrap_or(ReconciliationStatus::Unmatched);

            Ok(InternalLedgerEntry {
                id: r.get(0)?,
                account_id: r.get(1)?,
                doc_no: r.get(2)?,
                entry_date: r.get(3)?,
                entry_type,
                amount: r.get::<_, i64>(5)? as u64,
                partner_code: r.get(6)?,
                partner_name: r.get(7)?,
                description: r.get(8)?,
                reconciled_status: status,
                created_at: r.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?;

    for r in l_rows.flatten() {
        ledger_entries.push(r);
    }

    let (_matches, summary) = ReconciliationEngine::reconcile(&bank_txs, &ledger_entries);
    Ok(json!(summary))
}

// ---------------------------------------------------------------------------
// 4. Reconciliation Matrix
// ---------------------------------------------------------------------------

async fn get_reconciliation_matrix(state: Arc<AppState>, payload: Value) -> Result<Value, String> {
    let filter = payload
        .get("filter")
        .and_then(|v| v.as_str())
        .unwrap_or("ALL")
        .to_uppercase();

    tokio::task::spawn_blocking(move || {
        let conn = state
            .db
            .readers
            .get()
            .map_err(|e| format!("Read lock error: {e}"))?;

        let sql = match filter.as_str() {
            "MATCHED" => {
                "SELECT bt.id, bt.statement_id, bt.account_id, ba.bank_code, bt.tx_date, bt.value_date, \
                        bt.doc_ref, bt.tx_type, bt.amount, bt.balance_after, bt.counterparty_name, \
                        bt.counterparty_bank, bt.narration_enc, bt.reconciled_status, bt.created_at, \
                        rm.match_type, rm.confidence_score, rm.discrepancy_amount, rm.hitl_token, rm.notes, rm.ledger_entry_ids_json \
                 FROM bank_transactions bt \
                 LEFT JOIN bank_accounts ba ON bt.account_id = ba.id \
                 LEFT JOIN reconciliation_matches rm ON bt.id = rm.bank_tx_id \
                 WHERE bt.reconciled_status = 'MATCHED' \
                 ORDER BY bt.tx_date DESC LIMIT 100"
            }
            "PENDING_HITL" => {
                "SELECT bt.id, bt.statement_id, bt.account_id, ba.bank_code, bt.tx_date, bt.value_date, \
                        bt.doc_ref, bt.tx_type, bt.amount, bt.balance_after, bt.counterparty_name, \
                        bt.counterparty_bank, bt.narration_enc, bt.reconciled_status, bt.created_at, \
                        rm.match_type, rm.confidence_score, rm.discrepancy_amount, rm.hitl_token, rm.notes, rm.ledger_entry_ids_json \
                 FROM bank_transactions bt \
                 LEFT JOIN bank_accounts ba ON bt.account_id = ba.id \
                 LEFT JOIN reconciliation_matches rm ON bt.id = rm.bank_tx_id \
                 WHERE bt.reconciled_status = 'PENDING_HITL' \
                 ORDER BY bt.tx_date DESC LIMIT 100"
            }
            "DISCREPANCY" => {
                "SELECT bt.id, bt.statement_id, bt.account_id, ba.bank_code, bt.tx_date, bt.value_date, \
                        bt.doc_ref, bt.tx_type, bt.amount, bt.balance_after, bt.counterparty_name, \
                        bt.counterparty_bank, bt.narration_enc, bt.reconciled_status, bt.created_at, \
                        rm.match_type, rm.confidence_score, rm.discrepancy_amount, rm.hitl_token, rm.notes, rm.ledger_entry_ids_json \
                 FROM bank_transactions bt \
                 LEFT JOIN bank_accounts ba ON bt.account_id = ba.id \
                 LEFT JOIN reconciliation_matches rm ON bt.id = rm.bank_tx_id \
                 WHERE bt.reconciled_status = 'DISCREPANCY' \
                 ORDER BY bt.tx_date DESC LIMIT 100"
            }
            _ => {
                "SELECT bt.id, bt.statement_id, bt.account_id, ba.bank_code, bt.tx_date, bt.value_date, \
                        bt.doc_ref, bt.tx_type, bt.amount, bt.balance_after, bt.counterparty_name, \
                        bt.counterparty_bank, bt.narration_enc, bt.reconciled_status, bt.created_at, \
                        rm.match_type, rm.confidence_score, rm.discrepancy_amount, rm.hitl_token, rm.notes, rm.ledger_entry_ids_json \
                 FROM bank_transactions bt \
                 LEFT JOIN bank_accounts ba ON bt.account_id = ba.id \
                 LEFT JOIN reconciliation_matches rm ON bt.id = rm.bank_tx_id \
                 ORDER BY bt.tx_date DESC LIMIT 100"
            }
        };

        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |r| {
                let tx_type_str: String = r.get(7)?;
                let tx_type = tx_type_str.parse().unwrap_or(TransactionType::Credit);
                let status_str: String = r.get(13)?;
                let status = status_str.parse().unwrap_or(ReconciliationStatus::Unmatched);
                let enc_narration: String = r.get(12)?;
                let dec_narration = state.crypto.decrypt_read(&enc_narration);

                let match_type: Option<MatchType> = r
                    .get::<_, Option<String>>(15)?
                    .and_then(|s| s.parse().ok());
                let confidence_score: Option<f64> = r.get(16)?;
                let discrepancy_amount: Option<i64> = r.get(17)?;
                let hitl_token: Option<String> = r.get(18)?;
                let notes: Option<String> = r.get(19)?;
                let ledger_json: Option<String> = r.get(20)?;

                let bank_tx = BankTransactionRow {
                    id: r.get(0)?,
                    statement_id: r.get(1)?,
                    account_id: r.get(2)?,
                    bank_code: r.get(3).unwrap_or_else(|_| "VCB".to_string()),
                    tx_date: r.get(4)?,
                    value_date: r.get(5)?,
                    doc_ref: r.get(6)?,
                    tx_type,
                    amount: r.get::<_, i64>(8)? as u64,
                    balance_after: r.get::<_, Option<i64>>(9)?.map(|b| b as u64),
                    counterparty_account: None,
                    counterparty_name: r.get(10)?,
                    counterparty_bank: r.get(11)?,
                    narration: dec_narration,
                    reconciled_status: status,
                    reconciled_match_id: None,
                    created_at: r.get(14)?,
                };

                Ok((
                    bank_tx,
                    match_type,
                    confidence_score,
                    discrepancy_amount,
                    hitl_token,
                    notes,
                    ledger_json,
                ))
            })
            .map_err(|e| e.to_string())?;

        let mut items = Vec::new();
        let mut matched_cnt = 0;
        let mut hitl_cnt = 0;
        let mut disc_cnt = 0;

        for r in rows.flatten() {
            let (bank_tx, match_type, conf, disc, hitl_tok, notes, led_json) = r;

            match bank_tx.reconciled_status {
                ReconciliationStatus::Matched => matched_cnt += 1,
                ReconciliationStatus::PendingHitl => hitl_cnt += 1,
                ReconciliationStatus::Discrepancy => disc_cnt += 1,
                _ => {}
            }

            let mut matched_entries = Vec::new();
            if let Some(json_str) = led_json
                && let Ok(entry_ids) = serde_json::from_str::<Vec<String>>(&json_str)
            {
                for entry_id in entry_ids {
                        if let Ok(entry) = conn.query_row(
                            "SELECT id, account_id, doc_no, entry_date, entry_type, amount, partner_code, partner_name, description, reconciled_status, created_at \
                             FROM internal_ledger_entries WHERE id = ?1",
                            params![entry_id],
                            |er| {
                                let etype_str: String = er.get(4)?;
                                let etype = etype_str.parse().unwrap_or(TransactionType::Credit);
                                let st_str: String = er.get(9)?;
                                let st = st_str.parse().unwrap_or(ReconciliationStatus::Unmatched);
                                Ok(InternalLedgerEntry {
                                    id: er.get(0)?,
                                    account_id: er.get(1)?,
                                    doc_no: er.get(2)?,
                                    entry_date: er.get(3)?,
                                    entry_type: etype,
                                    amount: er.get::<_, i64>(5)? as u64,
                                    partner_code: er.get(6)?,
                                    partner_name: er.get(7)?,
                                    description: er.get(8)?,
                                    reconciled_status: st,
                                    created_at: er.get(10)?,
                                })
                            },
                        ) {
                            matched_entries.push(entry);
                        }
                    }
                }

            items.push(ReconciliationMatrixItemDto {
                bank_tx,
                matched_entries,
                match_type,
                confidence_score: conf,
                discrepancy_amount: disc,
                hitl_token: hitl_tok,
                notes,
            });
        }

        let matrix_dto = ReconciliationMatrixDto {
            total_count: items.len(),
            items,
            matched_count: matched_cnt,
            pending_hitl_count: hitl_cnt,
            discrepancy_count: disc_cnt,
        };

        Ok(json!(matrix_dto))
    })
    .await
    .map_err(|e| format!("Task error: {e}"))?
}

// ---------------------------------------------------------------------------
// 5. Resolve HITL (Two-Phase Confirmation)
// ---------------------------------------------------------------------------

async fn resolve_hitl(state: Arc<AppState>, payload: Value) -> Result<Value, String> {
    let match_id = payload
        .get("match_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'match_id'".to_string())?
        .to_string();

    let hitl_token = payload
        .get("hitl_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'hitl_token' for Two-Phase Confirmation".to_string())?
        .to_string();

    let decision = payload
        .get("decision")
        .and_then(|v| v.as_str())
        .unwrap_or("APPROVE")
        .to_uppercase();

    let notes = payload
        .get("notes")
        .and_then(|v| v.as_str())
        .map(str::to_string);

    let maker_id = payload
        .get("maker_id")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("accountant_maker")
        .to_string();

    let checker_id = payload
        .get("checker_id")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("chief_checker")
        .to_string();

    if maker_id.eq_ignore_ascii_case(&checker_id) {
        return Err(format!(
            "Circular 09/2020/TT-NHNN violation: Maker cannot self-approve as Checker ('{}' == '{}'). Dual control required.",
            maker_id, checker_id
        ));
    }

    tokio::task::spawn_blocking(move || {
        let conn = state
            .db
            .writer
            .get()
            .map_err(|e| format!("Write lock error: {e}"))?;

        // 1. Verify match_id and hitl_token
        let stored_token: Option<String> = conn
            .query_row(
                "SELECT hitl_token FROM reconciliation_matches WHERE id = ?1",
                params![match_id],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .flatten();

        let token_matches = stored_token
            .as_ref()
            .map(|t| t == &hitl_token)
            .unwrap_or(false);

        if !token_matches {
            return Err("Invalid or expired Two-Phase Confirmation HITL token".to_string());
        }

        // 2. Apply decision
        let (new_status, new_tx_status) = match decision.as_str() {
            "APPROVE" | "CONFIRM" => ("APPROVED", "MATCHED"),
            "REJECT" => ("REJECTED", "UNMATCHED"),
            _ => ("APPROVED", "MATCHED"),
        };

        let now_ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        // Invalidate single-use HITL token upon resolution (Circular 09/2020/TT-NHNN)
        conn.execute(
            "UPDATE reconciliation_matches SET status = ?1, matched_by = 'USER_HITL', matched_at = ?2, notes = ?3, hitl_token = NULL WHERE id = ?4",
            params![new_status, now_ts, notes, match_id],
        )
        .map_err(|e| e.to_string())?;

        // Update corresponding bank transaction
        let bank_tx_id: String = conn
            .query_row(
                "SELECT bank_tx_id FROM reconciliation_matches WHERE id = ?1",
                params![match_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;

        conn.execute(
            "UPDATE bank_transactions SET reconciled_status = ?1 WHERE id = ?2",
            params![new_tx_status, bank_tx_id],
        )
        .map_err(|e| e.to_string())?;

        // Synchronize linked ledger entries status
        let ledger_ids_opt: Option<String> = conn
            .query_row(
                "SELECT ledger_entry_ids_json FROM reconciliation_matches WHERE id = ?1",
                params![match_id],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .flatten();

        if let Some(json_str) = ledger_ids_opt {
            if let Ok(entry_ids) = serde_json::from_str::<Vec<String>>(&json_str) {
                let ledger_target_status = if new_status == "APPROVED" {
                    "MATCHED"
                } else {
                    "UNMATCHED"
                };
                for l_id in entry_ids {
                    let _ = conn.execute(
                        "UPDATE internal_ledger_entries SET reconciled_status = ?1 WHERE id = ?2",
                        params![ledger_target_status, l_id],
                    );
                }
            }
        }

        // 3. Audit Log
        let audit_key = get_audit_key(&state);
        let audit_msg = format!(
            "HITL Confirmed (Circular 09/2020/TT-NHNN Dual Control): match_id={}, decision={}, new_status={}, maker_id={}, checker_id={}",
            match_id, decision, new_status, maker_id, checker_id
        );
        let _ = AuditLedger::append(&conn, &audit_key, "HITL_CONFIRMED", &checker_id, &audit_msg);

        let res = HitlConfirmResultDto {
            success: true,
            match_id,
            new_status: new_status.to_string(),
            message: format!("Successfully resolved HITL review with decision: {decision}"),
        };

        Ok(json!(res))
    })
    .await
    .map_err(|e| format!("Task error: {e}"))?
}

// ---------------------------------------------------------------------------
// 6. Compliance Status
// ---------------------------------------------------------------------------

async fn get_compliance(state: Arc<AppState>) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let conn = state
            .db
            .readers
            .get()
            .map_err(|e| format!("Read lock error: {e}"))?;

        let status_dto = get_compliance_status(&conn, &state);
        Ok(json!(status_dto))
    })
    .await
    .map_err(|e| format!("Task error: {e}"))?
}

// ---------------------------------------------------------------------------
// Seed Demo Data helper
// ---------------------------------------------------------------------------

async fn seed_demo_data(state: Arc<AppState>) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let mut conn = state
            .db
            .writer
            .get()
            .map_err(|e| format!("Write lock error: {e}"))?;
        seed_default_accounts_and_data(&mut conn, &state)?;
        Ok(json!({ "status": "ok", "message": "Demo banking data successfully seeded" }))
    })
    .await
    .map_err(|e| format!("Task error: {e}"))?
}

fn seed_default_accounts_and_data(
    conn: &mut rusqlite::Connection,
    state: &AppState,
) -> Result<(), String> {
    let now_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    // 1. Seed Accounts
    let vcb_acc_enc = state.crypto.encrypt("0011001234567")?;
    let tcb_acc_enc = state.crypto.encrypt("19034567890123")?;
    let bidv_acc_enc = state.crypto.encrypt("12010001234567")?;

    conn.execute(
        "INSERT OR REPLACE INTO bank_accounts (id, bank_code, account_number_enc, account_name, opening_balance, current_balance, last_synced_at, created_at) \
         VALUES ('acc_vcb', 'VCB', ?1, 'CONG TY CP LIVA HARNESS', 1400000000, 1450230000, ?2, ?3)",
        params![vcb_acc_enc, now_ts, now_ts],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR REPLACE INTO bank_accounts (id, bank_code, account_number_enc, account_name, opening_balance, current_balance, last_synced_at, created_at) \
         VALUES ('acc_tcb', 'TCB', ?1, 'CONG TY CP LIVA HARNESS', 750000000, 785600000, ?2, ?3)",
        params![tcb_acc_enc, now_ts, now_ts],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR REPLACE INTO bank_accounts (id, bank_code, account_number_enc, account_name, opening_balance, current_balance, last_synced_at, created_at) \
         VALUES ('acc_bidv', 'BIDV', ?1, 'CONG TY CP LIVA HARNESS', 300000000, 350000000, ?2, ?3)",
        params![bidv_acc_enc, now_ts, now_ts],
    ).map_err(|e| e.to_string())?;

    // 2. Seed Mock Ledger Entries (ERP Open Items)
    let erp_items = [
        (
            "led_1001",
            "acc_vcb",
            "HD1001",
            25_000_000u64,
            "CUST01",
            "Cong ty Co phan Thuong mai ABC",
            "Ban hang thang 8",
        ),
        (
            "led_1002",
            "acc_vcb",
            "HD1002",
            45_000_000u64,
            "CUST02",
            "Cong ty TNHH Thep Viet Nhat",
            "Cung cap nguyen vat lieu",
        ),
        (
            "led_1003",
            "acc_vcb",
            "HD1003",
            55_000_000u64,
            "CUST02",
            "Cong ty TNHH Thep Viet Nhat",
            "Phu kien xay dung",
        ),
        (
            "led_1004",
            "acc_tcb",
            "HD2001",
            50_000_000u64,
            "CUST03",
            "Cong ty CP Dau tu Ha tang",
            "Hop dong thi cong so 02",
        ),
        (
            "led_1005",
            "acc_bidv",
            "PC091",
            15_000_000u64,
            "VEND01",
            "Cong ty Dich vu Cong nghe",
            "Chi phi server hosting",
        ),
    ];

    for (id, acc, doc, amt, p_code, p_name, desc) in erp_items {
        conn.execute(
            "INSERT OR REPLACE INTO internal_ledger_entries (id, account_id, doc_no, entry_date, entry_type, amount, partner_code, partner_name, description, reconciled_status, created_at) \
             VALUES (?1, ?2, ?3, ?4, 'CREDIT', ?5, ?6, ?7, ?8, 'UNMATCHED', ?9)",
            params![id, acc, doc, now_ts - 3600, amt as i64, p_code, p_name, desc, now_ts],
        ).map_err(|e| e.to_string())?;
    }

    // 3. Seed Mock Bank Transactions
    let bank_items = [
        // Exact match with HD1001
        (
            "tx_vcb_1",
            "stmt_vcb",
            "acc_vcb",
            "HD1001",
            25_000_000u64,
            "Cong ty Co phan Thuong mai ABC",
            "CTY ABC CK THANH TOAN HD1001",
        ),
        // Composite split match: 45M + 55M = 100M
        (
            "tx_vcb_2",
            "stmt_vcb",
            "acc_vcb",
            "FT262568912345",
            100_000_000u64,
            "Cong ty TNHH Thep Viet Nhat",
            "CK HD 1002 VA 1003 THEP VIET NHAT",
        ),
        // Fuzzy match with fee discrepancy (50M - 11k = 49,989,000)
        (
            "tx_tcb_1",
            "stmt_tcb",
            "acc_tcb",
            "FT998877",
            49_989_000u64,
            "Cong ty CP Dau tu Ha tang",
            "CONG TY DAU TU HA TANG CK TIEN HANG",
        ),
    ];

    for (id, stmt, acc, doc, amt, cp_name, narr) in bank_items {
        let enc_narr = state.crypto.encrypt(narr)?;
        conn.execute(
            "INSERT OR REPLACE INTO bank_transactions (id, statement_id, account_id, tx_date, value_date, doc_ref, tx_type, amount, balance_after, counterparty_name, narration_enc, reconciled_status, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'CREDIT', ?7, 1450230000, ?8, ?9, 'UNMATCHED', ?10)",
            params![id, stmt, acc, now_ts - 1800, now_ts - 1800, doc, amt as i64, cp_name, enc_narr, now_ts],
        ).map_err(|e| e.to_string())?;
    }

    // 4. Initial Audit Ledger Genesis
    let audit_key = get_audit_key(state);
    let _ = AuditLedger::append(
        conn,
        &audit_key,
        "SYSTEM_INITIALIZED",
        "TauriSetup",
        "LIVA Banking Harness v1.0.0 initialized with AES-256-GCM and DPAPI security",
    );

    Ok(())
}
