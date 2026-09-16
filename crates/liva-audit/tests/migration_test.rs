//! Integration tests for Milestone M2:
//! - SQLx Migrations (10 Core Banking Harness Entities)
//! - DB Triggers enforcing append-only invariants on `audit_logs`
//! - Segregation of Duties enforcing `CHECK (maker_id != checker_id)` on `quarantine_items`
//! - Continuous SHA-256 Hash Chain verification and 1-byte tamper detection
//! - Dual-dialect compatibility verification (SQLite and PostgreSQL)

use liva_audit::{
    compute_audit_row_hash, digest_payload, genesis_hash, verify_audit_db_chain, AuditDbRecord,
};
use rusqlite::{params, Connection};

fn setup_test_db() -> Connection {
    let conn = Connection::open_in_memory().expect("Failed to open in-memory SQLite db");
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .expect("Failed to enable foreign keys");

    let schema_sql = include_str!("../../../migrations/20260916000001_init_banking_schema.sql");
    conn.execute_batch(schema_sql)
        .expect("Failed to apply initial banking schema migration");

    let triggers_sql = include_str!("../../../migrations/20260916000002_audit_log_triggers.sql");
    conn.execute_batch(triggers_sql)
        .expect("Failed to apply audit log triggers migration");

    conn
}

#[test]
fn test_all_10_tables_and_indexes_created() {
    let conn = setup_test_db();

    let expected_tables = vec![
        "legal_entities",
        "fiscal_periods",
        "counterparties",
        "counterparty_aliases",
        "holidays",
        "bank_accounts",
        "bank_profiles",
        "bank_transactions",
        "quarantine_items",
        "audit_logs",
    ];

    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .unwrap();
    let table_rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .collect::<Result<Vec<String>, _>>()
        .unwrap();

    for expected in &expected_tables {
        assert!(
            table_rows.contains(&expected.to_string()),
            "Table {expected} was not created by migration"
        );
    }

    // Verify critical indexes
    let expected_indexes = vec![
        "idx_legal_entities_tax_id",
        "idx_fiscal_periods_dates",
        "idx_counterparties_normalized",
        "idx_counterparties_tax",
        "idx_aliases_normalized",
        "idx_aliases_counterparty",
        "idx_holidays_date",
        "idx_bank_accounts_entity",
        "idx_bank_accounts_lookup",
        "idx_bank_profiles_lookup",
        "idx_bank_tx_fingerprint",
        "idx_bank_tx_lookup",
        "idx_bank_tx_reconcile",
        "idx_quarantine_tx",
        "idx_quarantine_status",
        "idx_quarantine_token",
        "idx_audit_logs_lookup",
    ];

    let mut idx_stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='index' AND name NOT LIKE 'sqlite_%'")
        .unwrap();
    let index_rows = idx_stmt
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .collect::<Result<Vec<String>, _>>()
        .unwrap();

    for expected_idx in &expected_indexes {
        assert!(
            index_rows.contains(&expected_idx.to_string()),
            "Index {expected_idx} was not created by migration"
        );
    }
}

#[test]
fn test_quarantine_items_maker_checker_segregation_of_duties() {
    let conn = setup_test_db();

    // 1. Seed prerequisite legal entity, bank account, and transaction
    conn.execute(
        "INSERT INTO legal_entities (id, tax_id, legal_name, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params!["ent_01", "0101234567", "CONG TY TNHH LIVA", 1773000000i64, 1773000000i64],
    ).unwrap();

    conn.execute(
        "INSERT INTO bank_accounts (id, entity_id, bank_code, account_number_enc, account_number_hash, account_name, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params!["acc_01", "ent_01", "VCB", "enc_num_01", "hash_num_01", "VCB CORP ACC", 1773000000i64, 1773000000i64],
    ).unwrap();

    conn.execute(
        "INSERT INTO bank_transactions (id, account_id, statement_fingerprint, txn_hash, tx_date, tx_type, amount, is_credit, balance_after, narration, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            "tx_01", "acc_01", "stmt_fp_01", "hash_tx_001", 1773000100i64,
            "PAYMENT", 15000000i64, false, 485000000i64, "Thanh toan tien hang", 1773000100i64
        ],
    ).unwrap();

    // 2. Test Case A: maker_id != checker_id (Valid Four-Eyes segregation) -> MUST SUCCEED
    let res_valid = conn.execute(
        "INSERT INTO quarantine_items (
            id, bank_tx_id, account_id, amount, quarantine_reason,
            confidence_score, hitl_token, maker_id, checker_id,
            status, created_at, expires_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            "q_item_01", "tx_01", "acc_01", 15000000i64, "LOW_CONFIDENCE_MATCH",
            0.55f64, "hitl_tok_001", "usr_maker_alice", "usr_checker_bob",
            "PENDING_REVIEW", 1773000100i64, 1773086500i64
        ],
    );
    assert!(res_valid.is_ok(), "Valid maker-checker insertion failed: {:?}", res_valid.err());

    // 3. Test Case B: maker_id set, checker_id is NULL (Initial submission before review) -> MUST SUCCEED
    let res_null_checker = conn.execute(
        "INSERT INTO quarantine_items (
            id, bank_tx_id, account_id, amount, quarantine_reason,
            confidence_score, hitl_token, maker_id, checker_id,
            status, created_at, expires_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            "q_item_02", "tx_01", "acc_01", 15000000i64, "AMBIGUOUS_COUNTERPARTY",
            0.45f64, "hitl_tok_002", "usr_maker_alice", Option::<String>::None,
            "PENDING_REVIEW", 1773000100i64, 1773086500i64
        ],
    );
    assert!(res_null_checker.is_ok(), "NULL checker_id insertion failed: {:?}", res_null_checker.err());

    // 4. Test Case C: maker_id == checker_id (Self-approval violation on INSERT) -> MUST FAIL CHECK CONSTRAINT
    let res_same_insert = conn.execute(
        "INSERT INTO quarantine_items (
            id, bank_tx_id, account_id, amount, quarantine_reason,
            confidence_score, hitl_token, maker_id, checker_id,
            status, created_at, expires_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            "q_item_03", "tx_01", "acc_01", 15000000i64, "FRAUD_SUSPECT",
            0.30f64, "hitl_tok_003", "usr_maker_alice", "usr_maker_alice",
            "PENDING_REVIEW", 1773000100i64, 1773086500i64
        ],
    );
    assert!(res_same_insert.is_err(), "INSERT with maker_id == checker_id must fail check constraint!");
    let err_msg = res_same_insert.unwrap_err().to_string();
    assert!(
        err_msg.contains("CHECK constraint failed") || err_msg.contains("chk_maker_checker"),
        "Unexpected error for maker_id == checker_id: {err_msg}"
    );

    // 5. Test Case D: Updating q_item_02 to set checker_id == maker_id -> MUST FAIL CHECK CONSTRAINT
    let res_same_update = conn.execute(
        "UPDATE quarantine_items SET checker_id = 'usr_maker_alice' WHERE id = 'q_item_02'",
        [],
    );
    assert!(res_same_update.is_err(), "UPDATE with checker_id == maker_id must fail check constraint!");
    let err_msg = res_same_update.unwrap_err().to_string();
    assert!(
        err_msg.contains("CHECK constraint failed") || err_msg.contains("chk_maker_checker"),
        "Unexpected error for UPDATE checker_id == maker_id: {err_msg}"
    );

    // 6. Test Case E: Updating q_item_02 to a distinct checker_id -> MUST SUCCEED
    let res_distinct_update = conn.execute(
        "UPDATE quarantine_items SET checker_id = 'usr_checker_carol', status = 'RESOLVED' WHERE id = 'q_item_02'",
        [],
    );
    assert!(res_distinct_update.is_ok(), "UPDATE with distinct checker failed: {:?}", res_distinct_update.err());
}

#[test]
fn test_audit_logs_append_only_triggers_reject_update_and_delete() {
    let conn = setup_test_db();

    let gen_hash = genesis_hash();
    let payload = r#"{"action":"SYSTEM_BOOT","module":"liva-server"}"#;
    let payload_dig = digest_payload(payload.as_bytes());
    let ts = 1773000000i64;

    let client_ip = "127.0.0.1";
    let sig = "sig_boot_01";
    let row_h = compute_audit_row_hash(
        1,
        &gen_hash,
        ts,
        "sys_admin",
        "BOOT",
        "system",
        "boot_01",
        &payload_dig,
        client_ip,
        sig,
    );

    // 1. Insert initial audit log entry -> MUST SUCCEED
    let insert_res = conn.execute(
        "INSERT INTO audit_logs (
            prev_hash, row_hash, timestamp, actor_id,
            event_type, entity_type, entity_id, payload_digest,
            signature, client_ip
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            gen_hash, row_h, ts, "sys_admin",
            "BOOT", "system", "boot_01", payload_dig,
            sig, client_ip
        ],
    );
    assert!(insert_res.is_ok(), "Initial audit log insert failed: {:?}", insert_res.err());

    // 2. Attempt direct UPDATE on audit_logs -> MUST FAIL VIA DB TRIGGER
    let update_res = conn.execute(
        "UPDATE audit_logs SET actor_id = 'malicious_intruder' WHERE seq_id = 1",
        [],
    );
    assert!(update_res.is_err(), "UPDATE on audit_logs must be aborted by trigger!");
    let update_err = update_res.unwrap_err().to_string();
    assert!(
        update_err.contains("INVARIANT_VIOLATION") && update_err.contains("UPDATE operation is strictly prohibited"),
        "Trigger error mismatch on UPDATE: {update_err}"
    );

    // 3. Attempt direct DELETE on audit_logs -> MUST FAIL VIA DB TRIGGER
    let delete_res = conn.execute(
        "DELETE FROM audit_logs WHERE seq_id = 1",
        [],
    );
    assert!(delete_res.is_err(), "DELETE on audit_logs must be aborted by trigger!");
    let delete_err = delete_res.unwrap_err().to_string();
    assert!(
        delete_err.contains("INVARIANT_VIOLATION") && delete_err.contains("DELETE operation is strictly prohibited"),
        "Trigger error mismatch on DELETE: {delete_err}"
    );

    // 4. Attempt direct INSERT OR REPLACE on audit_logs -> MUST FAIL VIA DB TRIGGER
    let replace_res = conn.execute(
        "INSERT OR REPLACE INTO audit_logs (
            seq_id, prev_hash, row_hash, timestamp, actor_id,
            event_type, entity_type, entity_id, payload_digest,
            signature, client_ip
         ) VALUES (1, 'forged_prev', 'forged_row', 9999, 'malicious_replacer', 'BOOT', 'system', 'boot_01', 'pl', 'sig', '127.0.0.1')",
        [],
    );
    assert!(replace_res.is_err(), "INSERT OR REPLACE on audit_logs must be aborted by trigger!");
    let replace_err = replace_res.unwrap_err().to_string();
    assert!(
        replace_err.contains("INVARIANT_VIOLATION") && replace_err.contains("INSERT OR REPLACE / collision mutation is strictly prohibited"),
        "Trigger error mismatch on INSERT OR REPLACE: {replace_err}"
    );

    // 5. Verify data in audit_logs remained completely unmodified
    let actor_in_db: String = conn.query_row(
        "SELECT actor_id FROM audit_logs WHERE seq_id = 1",
        [],
        |row| row.get(0),
    ).unwrap();
    assert_eq!(actor_in_db, "sys_admin", "Audit log data was altered despite trigger!");
}

#[test]
fn test_continuous_sha256_hash_chain_and_tamper_detection() {
    let conn = setup_test_db();

    let count = 50usize;
    let mut prev = genesis_hash();
    let base_ts = 1773000000i64;

    // 1. Insert 50 chained records into audit_logs
    for i in 1..=count {
        let seq = i as u64;
        let ts = base_ts + (i as i64 * 60);
        let actor = format!("usr_worker_{}", (i % 5) + 1);
        let event = if i % 2 == 0 { "TRANSACTION_INGEST" } else { "RECONCILIATION_MATCH" };
        let entity_type = "bank_transactions";
        let entity_id = format!("tx_uuid_{i:04}");
        let payload = format!(r#"{{"tx_id":{},"amount":{}}}"#, i, i * 1000);
        let payload_digest = digest_payload(payload.as_bytes());

        let client_ip = "127.0.0.1";
        let sig = format!("sig_worker_{i}");
        let row_h = compute_audit_row_hash(
            seq,
            &prev,
            ts,
            &actor,
            event,
            entity_type,
            &entity_id,
            &payload_digest,
            client_ip,
            &sig,
        );

        conn.execute(
            "INSERT INTO audit_logs (
                prev_hash, row_hash, timestamp, actor_id,
                event_type, entity_type, entity_id, payload_digest,
                signature, client_ip
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                prev, row_h, ts, actor,
                event, entity_type, entity_id, payload_digest,
                sig, client_ip
            ],
        ).unwrap();

        prev = row_h;
    }

    // 2. Query back all 50 records from SQLite
    let mut stmt = conn.prepare(
        "SELECT seq_id, prev_hash, row_hash, timestamp, actor_id, event_type, entity_type, entity_id, payload_digest, signature, client_ip
         FROM audit_logs ORDER BY seq_id ASC"
    ).unwrap();

    let records: Vec<AuditDbRecord> = stmt.query_map([], |row| {
        Ok(AuditDbRecord {
            seq_id: row.get::<_, i64>(0)? as u64,
            prev_hash: row.get(1)?,
            row_hash: row.get(2)?,
            timestamp: row.get(3)?,
            actor_id: row.get(4)?,
            event_type: row.get(5)?,
            entity_type: row.get(6)?,
            entity_id: row.get(7)?,
            payload_digest: row.get(8)?,
            signature: row.get(9)?,
            client_ip: row.get(10)?,
        })
    }).unwrap().collect::<Result<Vec<_>, _>>().unwrap();

    assert_eq!(records.len(), count);

    // 3. Verify clean chain integrity
    let report = verify_audit_db_chain(&records);
    assert!(report.is_intact, "Audit chain verification failed on clean database!");
    assert_eq!(report.total_records, count);
    assert_eq!(report.tampered_seq_id, None);
    assert_eq!(report.genesis_hash, genesis_hash());
    assert_eq!(report.latest_hash, records.last().unwrap().row_hash);

    // 4. Exhaustive Tamper Testing on Invariants:

    // Case 4A: 1-byte tamper in payload_digest at seq_id = 25
    {
        let mut tampered = records.clone();
        let orig_digest = &tampered[24].payload_digest;
        let mut mutated_bytes = orig_digest.as_bytes().to_vec();
        mutated_bytes[0] ^= 0x01; // flip 1 bit
        tampered[24].payload_digest = String::from_utf8(mutated_bytes).unwrap();

        let tamper_report = verify_audit_db_chain(&tampered);
        assert!(!tamper_report.is_intact, "1-byte payload tamper must be detected!");
        assert_eq!(tamper_report.tampered_seq_id, Some(25));
        assert!(tamper_report.error_message.unwrap().contains("Hash verification mismatch at seq_id 25"));
    }

    // Case 4B: 1-byte tamper in actor_id at seq_id = 10
    {
        let mut tampered = records.clone();
        tampered[9].actor_id = "usr_evil_impostor".to_string();

        let tamper_report = verify_audit_db_chain(&tampered);
        assert!(!tamper_report.is_intact, "Actor alteration must be detected!");
        assert_eq!(tamper_report.tampered_seq_id, Some(10));
    }

    // Case 4C: Monotonic sequence violation (missing row 15)
    {
        let mut tampered = records.clone();
        tampered.remove(14); // remove index 14 (seq_id = 15)

        let tamper_report = verify_audit_db_chain(&tampered);
        assert!(!tamper_report.is_intact, "Monotonic sequence violation must be detected!");
        assert_eq!(tamper_report.tampered_seq_id, Some(16));
        assert!(tamper_report.error_message.unwrap().contains("Monotonic sequence violation"));
    }

    // Case 4D: Chain break (corrupted prev_hash at seq_id = 30)
    {
        let mut tampered = records.clone();
        tampered[29].prev_hash = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef".to_string();

        let tamper_report = verify_audit_db_chain(&tampered);
        assert!(!tamper_report.is_intact, "Chain break must be detected!");
        assert_eq!(tamper_report.tampered_seq_id, Some(30));
        assert!(tamper_report.error_message.unwrap().contains("Chain broken at seq_id 30"));
    }

    // Case 4E: Genesis anchoring violation on first record
    {
        let mut tampered = records.clone();
        tampered[0].prev_hash = "0000000000000000000000000000000000000000000000000000000000000000".to_string();

        let tamper_report = verify_audit_db_chain(&tampered);
        assert!(!tamper_report.is_intact, "Genesis violation must be detected!");
        assert_eq!(tamper_report.tampered_seq_id, Some(1));
    }

    // Case 4F: 1-byte tamper in client_ip at seq_id = 20
    {
        let mut tampered = records.clone();
        tampered[19].client_ip = "192.168.1.50".to_string();

        let tamper_report = verify_audit_db_chain(&tampered);
        assert!(!tamper_report.is_intact, "Client IP alteration must be detected!");
        assert_eq!(tamper_report.tampered_seq_id, Some(20));
        assert!(tamper_report.error_message.unwrap().contains("Hash verification mismatch at seq_id 20"));
    }

    // Case 4G: 1-byte tamper in signature at seq_id = 35
    {
        let mut tampered = records.clone();
        tampered[34].signature = "forged_signature_35".to_string();

        let tamper_report = verify_audit_db_chain(&tampered);
        assert!(!tamper_report.is_intact, "Signature alteration must be detected!");
        assert_eq!(tamper_report.tampered_seq_id, Some(35));
        assert!(tamper_report.error_message.unwrap().contains("Hash verification mismatch at seq_id 35"));
    }
}

#[test]
fn test_database_idempotency_and_foreign_keys() {
    let conn = setup_test_db();

    // 1. Legal Entity
    conn.execute(
        "INSERT INTO legal_entities (id, tax_id, legal_name, created_at, updated_at)
         VALUES ('ent_01', '0101234567', 'LIVA BANKING CORP', 1773000000, 1773000000)",
        [],
    ).unwrap();

    // Duplicate tax_id must fail UNIQUE constraint
    let dup_tax = conn.execute(
        "INSERT INTO legal_entities (id, tax_id, legal_name, created_at, updated_at)
         VALUES ('ent_02', '0101234567', 'DUPLICATE TAX CORP', 1773000000, 1773000000)",
        [],
    );
    assert!(dup_tax.is_err(), "Duplicate tax_id must fail unique constraint");

    // 2. Bank Account & Transaction
    conn.execute(
        "INSERT INTO bank_accounts (id, entity_id, bank_code, account_number_enc, account_number_hash, account_name, created_at, updated_at)
         VALUES ('acc_01', 'ent_01', 'TCB', 'enc_01', 'hash_01', 'TECHCOMBANK ACC', 1773000000, 1773000000)",
        [],
    ).unwrap();

    conn.execute(
        "INSERT INTO bank_transactions (id, account_id, statement_fingerprint, txn_hash, tx_date, tx_type, amount, is_credit, balance_after, narration, created_at)
         VALUES ('tx_01', 'acc_01', 'fp_aug_2026', 'tx_unique_hash_999', 1773000000, 'CREDIT', 50000000, 1, 50000000, 'Nap tien', 1773000000)",
        [],
    ).unwrap();

    // Duplicate txn_hash must fail UNIQUE constraint (idempotent statement ingestion)
    let dup_tx = conn.execute(
        "INSERT INTO bank_transactions (id, account_id, statement_fingerprint, txn_hash, tx_date, tx_type, amount, is_credit, balance_after, narration, created_at)
         VALUES ('tx_02', 'acc_01', 'fp_aug_2026', 'tx_unique_hash_999', 1773000000, 'CREDIT', 50000000, 1, 50000000, 'Nap tien duplicate', 1773000000)",
        [],
    );
    assert!(dup_tx.is_err(), "Duplicate txn_hash must fail unique constraint for ingest idempotency");

    // 3. Foreign key violation: Bank Account referencing non-existent entity_id
    let fk_fail = conn.execute(
        "INSERT INTO bank_accounts (id, entity_id, bank_code, account_number_enc, account_number_hash, account_name, created_at, updated_at)
         VALUES ('acc_99', 'ent_non_existent', 'BIDV', 'enc_99', 'hash_99', 'GHOST ACC', 1773000000, 1773000000)",
        [],
    );
    assert!(fk_fail.is_err(), "Foreign key violation must reject orphaned bank account");
}

#[test]
fn test_postgresql_migration_scripts_syntax_and_parity() {
    let pg_schema = include_str!("../../../migrations/postgres/20260916000001_init_banking_schema.sql");
    let pg_triggers = include_str!("../../../migrations/postgres/20260916000002_audit_log_triggers.sql");

    // Verify all 10 tables are present in PostgreSQL migration
    let required_tables = [
        "legal_entities",
        "fiscal_periods",
        "counterparties",
        "counterparty_aliases",
        "holidays",
        "bank_accounts",
        "bank_profiles",
        "bank_transactions",
        "quarantine_items",
        "audit_logs",
    ];

    for tbl in &required_tables {
        assert!(
            pg_schema.contains(&format!("CREATE TABLE IF NOT EXISTS {tbl}")),
            "PostgreSQL schema missing table: {tbl}"
        );
    }

    // Verify PostgreSQL specific elements: BIGSERIAL, plpgsql function, TRUNCATE trigger
    assert!(pg_schema.contains("seq_id BIGSERIAL PRIMARY KEY"));
    assert!(pg_schema.contains("CONSTRAINT chk_maker_checker CHECK (maker_id != checker_id)"));
    assert!(pg_triggers.contains("LANGUAGE plpgsql"));
    assert!(pg_triggers.contains("RAISE EXCEPTION"));
    assert!(pg_triggers.contains("BEFORE TRUNCATE ON audit_logs"));
    assert!(pg_triggers.contains("BEFORE UPDATE ON audit_logs"));
    assert!(pg_triggers.contains("BEFORE DELETE ON audit_logs"));
}

#[test]
fn test_sqlite_dedicated_migrations_parity() {
    let sqlite_schema = include_str!("../../../migrations/sqlite/20260916000001_init_banking_schema.sql");
    let sqlite_triggers = include_str!("../../../migrations/sqlite/20260916000002_audit_log_triggers.sql");

    assert!(sqlite_schema.contains("seq_id INTEGER PRIMARY KEY AUTOINCREMENT"));
    assert!(sqlite_schema.contains("CONSTRAINT chk_maker_checker CHECK (maker_id != checker_id)"));
    assert!(sqlite_triggers.contains("SELECT RAISE(ABORT"));
    assert!(sqlite_triggers.contains("BEFORE UPDATE ON audit_logs"));
    assert!(sqlite_triggers.contains("BEFORE DELETE ON audit_logs"));
    assert!(sqlite_triggers.contains("BEFORE INSERT ON audit_logs"));
    assert!(sqlite_triggers.contains("trg_audit_logs_no_replace"));
}
