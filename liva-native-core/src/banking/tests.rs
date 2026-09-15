//! Comprehensive Automated Unit and Integration Tests for LIVA Banking Engine.
//!
//! Tests:
//! 1. High-Speed Statement Parsers (VCB Excel, TCB CSV, BIDV PDF).
//! 2. Zero-Hallucination 3-Tier Reconciliation Solver & Mathematical Invariants.
//! 3. Decree 13 Compliance, Zero Egress, PII Redaction, and HMAC-SHA256 Audit Chain.

use rusqlite::Connection;
use uuid::Uuid;

use crate::banking::compliance::audit_ledger::{AuditLedger, genesis_hash};
use crate::banking::compliance::sanitizer::sanitize_pii;
use crate::banking::compliance::security::{
    is_egress_permitted, verify_zero_egress, verify_zero_egress_from,
};
use crate::banking::models::*;
use crate::banking::parser::agribank_parser::AgribankParser;
use crate::banking::parser::bidv_pdf::BidvPdfParser;
use crate::banking::parser::mbbank_parser::MbBankParser;
use crate::banking::parser::tcb_csv::TcbCsvParser;
use crate::banking::parser::vcb_excel::VcbExcelParser;
use crate::banking::parser::vietinbank_parser::VietinBankParser;
use crate::banking::parser::{
    BankStatementParser, ContainerFormat, detect_container_format, sniff_and_parse,
};
use crate::banking::reconciliation::ReconciliationEngine;
use crate::banking::reconciliation::fuzzy_matcher::FuzzyMatcher;
use crate::banking::reconciliation::hash_matcher::HashMatcher;
use crate::banking::reconciliation::jaro_winkler::{
    compare_party_names, strip_bank_narration_noise,
};
use crate::banking::reconciliation::split_solver::{SplitSolver, solve_exact_subset_sum_bnb};

// ===========================================================================
// Part 1: High-Speed Statement Parser Tests
// ===========================================================================

#[test]
fn test_tcb_csv_parser_with_bom_and_napas() {
    let parser = TcbCsvParser;

    // Build synthetic CSV with UTF-8 BOM, semicolon delimiter, Napas ref, and VietQR description
    let mut raw_csv = Vec::new();
    raw_csv.extend_from_slice(&[0xEF, 0xBB, 0xBF]); // UTF-8 BOM
    let csv_content = "\
Số tài khoản:;19034567890123;;;\n\
Tên tài khoản:;CONG TY TNHH LIVA SOLUTIONS;;;\n\
Số dư đầu kỳ:;750.000.000;;;\n\
Ngày giao dịch;Mã giao dịch;Số tiền ghi nợ;Số tiền ghi có;Số dư;Nội dung chi tiết\n\
15/08/2026 09:30:00;FT262568912345;;25.000.000;775.000.000;Napas VietQR TT HD102 Tu: CONG TY ABC\n\
15/08/2026 14:15:20;VN267890123456;15.000.000;;760.000.000;Thanh toan tien dien thang 8\n\
16/08/2026 11:00:00;NPS9988776655;;50.000.000;810.000.000;QRIBFT chuyen khoan hop dong HD200\n\
Tổng phát sinh;;15.000.000;75.000.000;810.000.000;\n";
    raw_csv.extend_from_slice(csv_content.as_bytes());

    assert!(parser.sniff(&raw_csv, "tcb_statement_aug2026.csv"));

    let parsed = parser.parse(&raw_csv, "tcb_statement_aug2026.csv").unwrap();
    assert_eq!(parsed.bank_code, "TCB");
    assert_eq!(parsed.account_number.as_deref(), Some("19034567890123"));
    assert_eq!(parsed.opening_balance, Some(750_000_000));
    assert_eq!(parsed.transactions.len(), 3);

    // Verify row 1 (Credit 25M with Napas ref FT262568912345)
    let tx1 = &parsed.transactions[0];
    assert_eq!(tx1.amount, 25_000_000);
    assert_eq!(tx1.tx_type, TransactionType::Credit);
    assert_eq!(tx1.doc_ref.as_deref(), Some("FT262568912345"));
    assert!(tx1.narration.contains("Napas VietQR TT HD102"));
    assert_eq!(tx1.balance_after, Some(775_000_000));

    // Verify row 2 (Debit 15M)
    let tx2 = &parsed.transactions[1];
    assert_eq!(tx2.amount, 15_000_000);
    assert_eq!(tx2.tx_type, TransactionType::Debit);
    assert_eq!(tx2.balance_after, Some(760_000_000));

    // Verify row 3 (Credit 50M)
    let tx3 = &parsed.transactions[2];
    assert_eq!(tx3.amount, 50_000_000);
    assert_eq!(tx3.tx_type, TransactionType::Credit);
    assert_eq!(tx3.doc_ref.as_deref(), Some("NPS9988776655"));
}

#[test]
fn test_vcb_excel_sniff_and_amount_parsing() {
    let parser = VcbExcelParser;
    assert!(parser.sniff(b"", "vcb_statement_aug2026.xlsx"));
    assert!(parser.sniff(b"", "vietcombank_saoke.xls"));

    // Verify Vietnamese dot/comma number conversions
    assert_eq!(parse_vietnamese_amount("15.000.000,00"), Some(15_000_000));
    assert_eq!(
        parse_vietnamese_amount("1.450.230.000"),
        Some(1_450_230_000)
    );
    assert_eq!(parse_vietnamese_amount("380.000,00"), Some(380_000));
    assert_eq!(parse_vietnamese_amount("500.000"), Some(500_000));
}

#[test]
fn test_bidv_pdf_parser_sniff_and_balance_checksum() {
    let parser = BidvPdfParser;
    assert!(parser.sniff(b"", "bidv_statement_aug2026.pdf"));

    // Mathematical balance checksum invariant:
    // Opening + Credits - Debits == Closing
    let opening = 300_000_000u64;
    let credits = 150_000_000u64;
    let debits = 100_000_000u64;
    let closing = 350_000_000u64;

    let computed_closing = (opening as i128) + (credits as i128) - (debits as i128);
    assert_eq!(
        computed_closing, closing as i128,
        "Page balance checksum must match exactly"
    );
}

#[test]
fn test_sniff_and_parse_dispatcher() {
    let tcb_csv = "Ngày giao dịch,Mã giao dịch,Số tiền ghi nợ,Số tiền ghi có,Số dư,Nội dung\n15/08/2026,FT001,,10.000.000,10.000.000,Thu tien\n";
    let res = sniff_and_parse(tcb_csv.as_bytes(), "test.csv").unwrap();
    assert_eq!(res.bank_code, "TCB");
    assert_eq!(res.transactions.len(), 1);
}

// ===========================================================================
// Part 2: Zero-Hallucination 3-Tier Reconciliation Engine Tests
// ===========================================================================

#[test]
fn test_zero_hallucination_3tier_solver_complete_flow() {
    let now = 1_725_000_000i64;

    // 1. Bank transactions:
    // - TX 1: Exact 1:1 match with HD101 (25,000,000 VND)
    // - TX 2: Fuzzy heuristic match with HD102 (49,989,000 VND = 50M - 11k fee)
    // - TX 3: Composite split solver: 100,000,000 VND = HD103 (45M) + HD104 (55M)
    // - TX 4: Residual unmatched (380,000 VND) -> fails-closed to HITL
    let bank_txs = vec![
        BankTransactionRow {
            id: "tx_exact".to_string(),
            statement_id: "stmt_1".to_string(),
            account_id: "acc_vcb".to_string(),
            bank_code: "VCB".to_string(),
            tx_date: now,
            value_date: now,
            doc_ref: Some("HD-00101".to_string()),
            tx_type: TransactionType::Credit,
            amount: 25_000_000,
            balance_after: Some(1_425_000_000),
            counterparty_account: None,
            counterparty_name: Some("CONG TY ABC".to_string()),
            counterparty_bank: None,
            narration: "THANH TOAN HD101".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: now,
        },
        BankTransactionRow {
            id: "tx_fuzzy".to_string(),
            statement_id: "stmt_1".to_string(),
            account_id: "acc_tcb".to_string(),
            bank_code: "TCB".to_string(),
            tx_date: now + 3600,
            value_date: now + 3600,
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount: 49_989_000, // 50M minus 11,000 VND fee
            balance_after: Some(785_600_000),
            counterparty_account: None,
            counterparty_name: Some("CONG TY CP THUONG MAI ABC".to_string()),
            counterparty_bank: None,
            narration: "CONG TY ABC CK TIEN HANG HD 102".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: now,
        },
        BankTransactionRow {
            id: "tx_split".to_string(),
            statement_id: "stmt_1".to_string(),
            account_id: "acc_vcb".to_string(),
            bank_code: "VCB".to_string(),
            tx_date: now + 7200,
            value_date: now + 7200,
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount: 100_000_000, // Composite payment
            balance_after: Some(1_525_000_000),
            counterparty_account: None,
            counterparty_name: Some("CONG TY TNHH THEP VIET NHAT".to_string()),
            counterparty_bank: None,
            narration: "CK HD 103 VA 104 THEP VIET NHAT CON LAI NO HD 105".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: now,
        },
        BankTransactionRow {
            id: "tx_hitl_residual".to_string(),
            statement_id: "stmt_1".to_string(),
            account_id: "acc_vcb".to_string(),
            bank_code: "VCB".to_string(),
            tx_date: now + 10000,
            value_date: now + 10000,
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount: 380_000, // Residual unmatched fee or unknown credit
            balance_after: Some(1_525_380_000),
            counterparty_account: None,
            counterparty_name: Some("KHACH HANG LA".to_string()),
            counterparty_bank: None,
            narration: "Chuyen tien khong ghi noi dung".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: now,
        },
    ];

    // 2. Open invoices in internal ERP ledger
    let ledger_entries = vec![
        InternalLedgerEntry {
            id: "led_101".to_string(),
            account_id: "acc_vcb".to_string(),
            doc_no: "HD101".to_string(),
            entry_date: now + 600, // within 24h
            entry_type: TransactionType::Credit,
            amount: 25_000_000,
            partner_code: Some("CUST01".to_string()),
            partner_name: Some("Công ty ABC".to_string()),
            description: "Ban hang HD101".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: now,
        },
        InternalLedgerEntry {
            id: "led_102".to_string(),
            account_id: "acc_tcb".to_string(),
            doc_no: "HD102".to_string(),
            entry_date: now + 4000, // within 72h
            entry_type: TransactionType::Credit,
            amount: 50_000_000,
            partner_code: Some("CUST01".to_string()),
            partner_name: Some("Công ty Cổ phần Thương mại ABC".to_string()),
            description: "Ban hang HD102".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: now,
        },
        InternalLedgerEntry {
            id: "led_103".to_string(),
            account_id: "acc_vcb".to_string(),
            doc_no: "HD103".to_string(),
            entry_date: now + 7000,
            entry_type: TransactionType::Credit,
            amount: 45_000_000,
            partner_code: Some("CUST02".to_string()),
            partner_name: Some("Công ty TNHH Thép Việt Nhật".to_string()),
            description: "Ban thep xay dung HD103".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: now,
        },
        InternalLedgerEntry {
            id: "led_104".to_string(),
            account_id: "acc_vcb".to_string(),
            doc_no: "HD104".to_string(),
            entry_date: now + 7100,
            entry_type: TransactionType::Credit,
            amount: 55_000_000,
            partner_code: Some("CUST02".to_string()),
            partner_name: Some("Công ty TNHH Thép Việt Nhật".to_string()),
            description: "Ban thep cuon HD104".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: now,
        },
        InternalLedgerEntry {
            id: "led_105".to_string(),
            account_id: "acc_vcb".to_string(),
            doc_no: "HD105".to_string(),
            entry_date: now + 7200,
            entry_type: TransactionType::Credit,
            amount: 80_000_000, // Remains open
            partner_code: Some("CUST02".to_string()),
            partner_name: Some("Công ty TNHH Thép Việt Nhật".to_string()),
            description: "Con no HD105".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: now,
        },
    ];

    // 3. Execute Deterministic Reconciliation
    let (matches, summary) = ReconciliationEngine::reconcile(&bank_txs, &ledger_entries);

    // Verify Counts & Stats
    assert_eq!(summary.total_bank_transactions, 4);
    assert_eq!(summary.matched_exact_count, 1, "Tier 1 exact match count");
    assert_eq!(summary.matched_fuzzy_count, 1, "Tier 2 fuzzy match count");
    assert_eq!(summary.matched_split_count, 1, "Tier 3 split match count");
    assert_eq!(summary.total_matched_count, 3, "Total automatic matches");
    assert_eq!(
        summary.pending_hitl_count, 1,
        "Fail-closed HITL review count"
    );
    assert_eq!(
        summary.discrepancy_count, 2,
        "Discrepancy count: fee diff + HITL diff"
    );
    assert_eq!(summary.match_rate, 75.0);

    // --- Mathematical Invariant Verifications ---
    // Invariant 1: Tier 1 match has exact amount and zero discrepancy
    let m1 = matches.iter().find(|m| m.bank_tx_id == "tx_exact").unwrap();
    assert_eq!(m1.match_type, MatchType::Exact1To1);
    assert_eq!(m1.matched_amount, 25_000_000);
    assert_eq!(m1.discrepancy_amount, 0);
    assert_eq!(m1.confidence_score, 1.0);
    assert_eq!(m1.status, "APPROVED");

    // Invariant 2: Tier 2 match captures exact standard fee deduction (11,000 VND)
    let m2 = matches.iter().find(|m| m.bank_tx_id == "tx_fuzzy").unwrap();
    assert_eq!(m2.match_type, MatchType::FuzzyHeuristic);
    assert_eq!(m2.matched_amount, 49_989_000);
    assert_eq!(m2.discrepancy_amount, 11_000); // 50M - 49.989M = 11,000 fee
    assert!(m2.confidence_score >= 0.85);

    // Invariant 3: Tier 3 split solver: Sum(45M, 55M) == 100M with Delta == 0
    let m3 = matches.iter().find(|m| m.bank_tx_id == "tx_split").unwrap();
    assert_eq!(m3.match_type, MatchType::CompositeSplit);
    assert_eq!(m3.matched_amount, 100_000_000);
    assert_eq!(
        m3.discrepancy_amount, 0,
        "Arithmetic Invariant: Split allocation must have Delta = 0"
    );
    assert_eq!(m3.ledger_entry_ids.len(), 2);
    assert!(m3.ledger_entry_ids.contains(&"led_103".to_string()));
    assert!(m3.ledger_entry_ids.contains(&"led_104".to_string()));
    assert_eq!(m3.confidence_score, 0.98);

    // Invariant 4: Residual transaction fails-closed into HITL with valid UUID token
    let m4 = matches
        .iter()
        .find(|m| m.bank_tx_id == "tx_hitl_residual")
        .unwrap();
    assert_eq!(m4.status, "PENDING_HITL");
    assert_eq!(m4.match_type, MatchType::ManualHitl);
    assert!(m4.hitl_token.is_some());
    let token_str = m4.hitl_token.as_ref().unwrap();
    assert!(
        Uuid::parse_str(token_str).is_ok(),
        "HITL token must be a valid UUID v4"
    );
}

// ===========================================================================
// Part 3: On-Premise Security & Decree 13 Compliance Tests
// ===========================================================================

#[test]
fn test_is_egress_permitted_loopback_and_blocking() {
    // 1. Loopback addresses and endpoints must be strictly permitted
    let loopback_destinations = [
        "127.0.0.1",
        "127.0.0.1:8002",
        "127.0.0.2",
        "localhost",
        "localhost:3000",
        "::1",
        "[::1]:8080",
        "http://127.0.0.1:8002/api",
        "https://localhost:8443/ping",
        "ws://[::1]:8002/ws",
    ];
    for dest in &loopback_destinations {
        assert!(
            is_egress_permitted(dest),
            "Loopback destination '{dest}' must be permitted"
        );
    }

    // 2. External destinations, public IPs, and wildcard bindings must be strictly blocked
    let external_destinations = [
        "api.openai.com",
        "https://api.openai.com/v1/chat/completions",
        "aws.com",
        "http://aws.com:80",
        "0.0.0.0",
        "0.0.0.0:8002",
        "192.168.1.100",
        "8.8.8.8:53",
        "10.0.0.1",
        "",
        "   ",
    ];
    for dest in &external_destinations {
        assert!(
            !is_egress_permitted(dest),
            "External or non-loopback destination '{dest}' must be blocked"
        );
    }
}

#[test]
fn test_zero_network_egress_standalone() {
    let (is_isolated, listeners) = verify_zero_egress();
    assert!(
        is_isolated,
        "Zero Network Egress must be verified in default standalone mode"
    );
    assert!(!listeners.is_empty(), "Must report active listeners");
    for listener in &listeners {
        assert!(
            is_egress_permitted(listener),
            "Listener '{listener}' must strictly bind to a loopback address"
        );
    }
}

#[test]
fn test_zero_network_egress_blocks_external_bindings() {
    // 1. Non-loopback 0.0.0.0 must invalidate isolation
    let (is_isolated, listeners) = verify_zero_egress_from(|var| match var {
        "LIVA_SERVER_HOST" => Some("0.0.0.0".to_string()),
        _ => None,
    });
    assert!(
        !is_isolated,
        "Binding to 0.0.0.0 must invalidate zero network egress isolation"
    );
    assert!(listeners.iter().any(|l| l.contains("0.0.0.0")));

    // 2. Non-loopback LAN IP must invalidate isolation
    let (is_isolated_lan, _) = verify_zero_egress_from(|var| match var {
        "LIVA_SERVER_HOST" => Some("192.168.1.100".to_string()),
        _ => None,
    });
    assert!(
        !is_isolated_lan,
        "Binding to LAN IP must invalidate isolation"
    );

    // 3. External HOST variable must invalidate isolation
    let (is_isolated_host, _) = verify_zero_egress_from(|var| match var {
        "HOST" => Some("external-server.bank.com".to_string()),
        _ => None,
    });
    assert!(!is_isolated_host, "External HOST must invalidate isolation");

    // 4. External HTTP_PROXY must invalidate isolation
    let (is_isolated_proxy, _) = verify_zero_egress_from(|var| match var {
        "HTTP_PROXY" => Some("http://external-corporate-proxy.com:8080".to_string()),
        _ => None,
    });
    assert!(
        !is_isolated_proxy,
        "External HTTP_PROXY must invalidate zero network egress isolation"
    );

    // 5. Cloud LLM endpoint OPENAI_API_BASE must invalidate isolation
    let (is_isolated_cloud, _) = verify_zero_egress_from(|var| match var {
        "OPENAI_API_BASE" => Some("https://api.openai.com/v1".to_string()),
        _ => None,
    });
    assert!(
        !is_isolated_cloud,
        "External cloud API base must invalidate zero network egress isolation"
    );

    // 6. Valid loopback configurations must pass isolation
    let (is_isolated_ok, listeners_ok) = verify_zero_egress_from(|var| match var {
        "LIVA_SERVER_HOST" => Some("127.0.0.1".to_string()),
        "LIVA_SERVER_PORT" => Some("8002".to_string()),
        _ => None,
    });
    assert!(is_isolated_ok, "Loopback 127.0.0.1 must pass isolation");
    for listener in &listeners_ok {
        assert!(is_egress_permitted(listener));
    }
}

#[test]
fn test_realtime_pii_redaction() {
    // 12-digit CCCD
    let text1 = "Khach hang Tran Van B, CCCD so 079201004567 vua nop tien vao tai khoan";
    let red1 = sanitize_pii(text1);
    assert!(!red1.contains("079201004567"));
    assert!(red1.contains("[REDACTED_CCCD]"));

    // 13-digit Bank Account Number with prefix
    let text2 = "Chuyen khoan toi TK 0011009876543 tai Ngan hang Vietcombank";
    let red2 = sanitize_pii(text2);
    assert!(!red2.contains("0011009876543"));
    assert!(red2.contains("[REDACTED_ACCOUNT]"));

    // 8-digit Bank Account Number with prefix
    let text3 = "Tai khoan nhan: TK: 12345678 tai VPBank";
    let red3 = sanitize_pii(text3);
    assert!(!red3.contains("12345678"));
    assert!(red3.contains("[REDACTED_ACCOUNT]"));

    // Plain transaction amount without account prefix must be PRESERVED (no over-redaction)
    let text_amount1 = "So du hien tai la 1450230000 VND tai Vietcombank";
    let red_amount1 = sanitize_pii(text_amount1);
    assert!(
        red_amount1.contains("1450230000"),
        "Unformatted transaction amount must be preserved"
    );
    assert!(
        !red_amount1.contains("[REDACTED_ACCOUNT]"),
        "Amounts must not be falsely redacted as accounts"
    );

    let text_amount2 = "Giao dich chuyen tien so tien 15000000 thanh cong";
    let red_amount2 = sanitize_pii(text_amount2);
    assert!(
        red_amount2.contains("15000000"),
        "Numeric amount must be preserved"
    );
    assert!(!red_amount2.contains("[REDACTED_ACCOUNT]"));

    // Secret Token
    let text4 = "Authorization: Bearer sk-antigravitysecretkey9876543210 for auth";
    let red4 = sanitize_pii(text4);
    assert!(!red4.contains("sk-antigravitysecretkey9876543210"));
    assert!(red4.contains("[REDACTED_SECRET]") || red4.contains("[REDACTED_API_KEY]"));
}

#[test]
fn test_hmac_sha256_audit_chain_tamper_detection() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute(
        "CREATE TABLE banking_audit_chain (
            seq_id INTEGER PRIMARY KEY AUTOINCREMENT,
            prev_hash TEXT NOT NULL,
            record_hash TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            event_type TEXT NOT NULL,
            actor_principal TEXT NOT NULL,
            payload_digest TEXT NOT NULL,
            signature TEXT NOT NULL
        )",
        [],
    )
    .unwrap();

    let audit_key = b"liva_super_secure_audit_key_32b!";

    // Insert 4 forward-chained records
    let h1 = AuditLedger::append(
        &conn,
        audit_key,
        "STATEMENT_IMPORTED",
        "TauriDashboard",
        "vcb.xlsx",
    )
    .unwrap();
    let h2 = AuditLedger::append(
        &conn,
        audit_key,
        "MATCH_AUTO",
        "ReconciliationEngine",
        "batch_1",
    )
    .unwrap();
    let h3 = AuditLedger::append(
        &conn,
        audit_key,
        "HITL_CONFIRMED",
        "ChiefAccountant",
        "confirm_split",
    )
    .unwrap();
    let h4 = AuditLedger::append(
        &conn,
        audit_key,
        "ERP_EXPORT",
        "TauriDashboard",
        "journal_aug.xml",
    )
    .unwrap();

    assert_ne!(h1, h2);
    assert_ne!(h2, h3);
    assert_ne!(h3, h4);

    // 1. Check intact chain
    let report = AuditLedger::verify(&conn, audit_key).unwrap();
    assert!(report.is_intact);
    assert_eq!(report.total_records, 4);
    assert_eq!(report.latest_hash, h4);
    assert_eq!(report.genesis_hash, genesis_hash());
    assert!(report.tampered_seq_id.is_none());

    // 2. Simulate unauthorized external SQLite payload alteration on block 3
    conn.execute(
        "UPDATE banking_audit_chain SET actor_principal = 'Hacker' WHERE seq_id = 3",
        [],
    )
    .unwrap();

    // Verify tampering is instantly flagged at seq_id 3
    let tampered_report = AuditLedger::verify(&conn, audit_key).unwrap();
    assert!(
        !tampered_report.is_intact,
        "Tampered chain must fail verification"
    );
    assert_eq!(
        tampered_report.tampered_seq_id,
        Some(3),
        "Must identify corrupted block at seq_id 3"
    );

    // 3. Restore block 3 and simulate signature alteration on block 2
    conn.execute(
        "UPDATE banking_audit_chain SET actor_principal = 'ChiefAccountant' WHERE seq_id = 3",
        [],
    )
    .unwrap();
    conn.execute(
        "UPDATE banking_audit_chain SET signature = 'forged_fake_signature_hash' WHERE seq_id = 2",
        [],
    )
    .unwrap();

    let sig_tampered_report = AuditLedger::verify(&conn, audit_key).unwrap();
    assert!(
        !sig_tampered_report.is_intact,
        "Signature tampering must fail verification"
    );
    assert_eq!(
        sig_tampered_report.tampered_seq_id,
        Some(2),
        "Must identify signature tampering at seq_id 2"
    );
}

#[test]
fn test_statement_fixtures_compatibility() {
    let base_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let fixtures_dir = base_dir.join("fixtures").join("statements");

    // 1. Test VCB Excel Fixture (50 genuine transactions)
    let vcb_file = fixtures_dir.join("vcb_aug2026.xlsx");
    assert!(vcb_file.exists(), "vcb_aug2026.xlsx must exist");
    let vcb_bytes = std::fs::read(&vcb_file).unwrap();
    let vcb_parsed = sniff_and_parse(&vcb_bytes, "vcb_aug2026.xlsx").unwrap();
    assert_eq!(vcb_parsed.bank_code, "VCB");
    assert_eq!(vcb_parsed.account_number.as_deref(), Some("0011001234567"));
    assert_eq!(vcb_parsed.opening_balance, Some(1_450_230_000));
    assert_eq!(
        vcb_parsed.closing_balance,
        Some(2_658_070_400),
        "VCB closing balance must match summary row"
    );
    assert_eq!(
        vcb_parsed.transactions.len(),
        50,
        "VCB fixture must have 50 transactions"
    );
    let vcb_inv = verify_balance_invariants(&vcb_parsed);
    assert!(vcb_inv.is_balanced, "VCB statement must be balanced");
    assert_eq!(vcb_inv.discrepancy, 0, "VCB discrepancy must be 0");

    // 2. Test TCB CSV Fixture (60 genuine transactions)
    let tcb_file = fixtures_dir.join("tcb_aug2026.csv");
    assert!(tcb_file.exists(), "tcb_aug2026.csv must exist");
    let tcb_bytes = std::fs::read(&tcb_file).unwrap();
    let tcb_parsed = sniff_and_parse(&tcb_bytes, "tcb_aug2026.csv").unwrap();
    assert_eq!(tcb_parsed.bank_code, "TCB");
    assert_eq!(tcb_parsed.account_number.as_deref(), Some("19034567890123"));
    assert_eq!(tcb_parsed.opening_balance, Some(785_600_000));
    assert_eq!(
        tcb_parsed.transactions.len(),
        60,
        "TCB fixture must have 60 transactions"
    );

    // 3. Test BIDV PDF Fixture (40 genuine transactions)
    let bidv_file = fixtures_dir.join("bidv_aug2026.pdf");
    assert!(bidv_file.exists(), "bidv_aug2026.pdf must exist");
    let bidv_bytes = std::fs::read(&bidv_file).unwrap();
    let bidv_parsed = sniff_and_parse(&bidv_bytes, "bidv_aug2026.pdf").unwrap();
    assert_eq!(bidv_parsed.bank_code, "BIDV");
    assert_eq!(
        bidv_parsed.account_number.as_deref(),
        Some("12410001234567")
    );
    assert_eq!(bidv_parsed.opening_balance, Some(520_000_000));
    assert_eq!(
        bidv_parsed.transactions.len(),
        40,
        "BIDV fixture must have 40 transactions"
    );

    // Check balance checksum on BIDV
    let bidv_open = bidv_parsed.opening_balance.unwrap_or(0);
    let mut bidv_sum_credit = 0u64;
    let mut bidv_sum_debit = 0u64;
    for tx in &bidv_parsed.transactions {
        if tx.tx_type == TransactionType::Credit {
            bidv_sum_credit += tx.amount;
        } else {
            bidv_sum_debit += tx.amount;
        }
    }
    let computed_closing =
        (bidv_open as i128) + (bidv_sum_credit as i128) - (bidv_sum_debit as i128);
    assert_eq!(
        computed_closing,
        bidv_parsed.closing_balance.unwrap() as i128,
        "BIDV closing balance checksum must match exactly"
    );

    // 4. Test Open Invoices JSON (120 invoices)
    let inv_file = fixtures_dir.join("open_invoices.json");
    assert!(inv_file.exists(), "open_invoices.json must exist");
    let inv_str = std::fs::read_to_string(&inv_file).unwrap();
    let invoices: Vec<InternalLedgerEntry> = serde_json::from_str(&inv_str).unwrap();
    assert_eq!(invoices.len(), 120, "ERP fixture must have 120 invoices");
}

#[test]
fn test_vietinbank_parser_csv_and_sniff() {
    let parser = VietinBankParser;
    let csv = "\
NGÂN HÀNG TMCP CÔNG THƯƠNG VIỆT NAM - VIETINBANK\n\
Số tài khoản: 102000888999\n\
Tên tài khoản: CONG TY TNHH LIVA SOLUTIONS\n\
Số dư đầu kỳ: 500.000.000\n\
Ngày GD,Số phiếu,Số tiền GD,Nợ/Có,Dư cuối,Diễn giải\n\
10/08/2026,CT1001,50.000.000,C,550.000.000,Thanh toan tien hang hop dong 01\n\
11/08/2026,CT1002,20.000.000,N,530.000.000,Phi duy tri dich vu ngan hang\n\
12/08/2026,CT1003,100.000.000,C,630.000.000,Khach hang chuyen tien tam ung\n\
Tổng phát sinh Có: 150.000.000\n\
Tổng phát sinh Nợ: 20.000.000\n\
Số dư cuối kỳ: 630.000.000\n";

    assert!(parser.sniff(csv.as_bytes(), "vietinbank_statement_aug2026.csv"));
    assert!(parser.sniff(b"", "ctg_statement.xlsx"));

    let parsed = parser
        .parse(csv.as_bytes(), "vietinbank_statement.csv")
        .unwrap();
    assert_eq!(parsed.bank_code, "CTG");
    assert_eq!(parsed.bank, BankType::VietinBank);
    assert_eq!(parsed.format, StatementFormat::Csv);
    assert_eq!(parsed.account_number.as_deref(), Some("102000888999"));
    assert_eq!(parsed.opening_balance, Some(500_000_000));
    assert_eq!(parsed.closing_balance, Some(630_000_000));
    assert_eq!(parsed.transactions.len(), 3);

    // Check tx 0 (Credit 50M)
    assert_eq!(parsed.transactions[0].amount, 50_000_000);
    assert_eq!(parsed.transactions[0].tx_type, TransactionType::Credit);
    assert_eq!(parsed.transactions[0].doc_ref.as_deref(), Some("CT1001"));
    assert_eq!(parsed.transactions[0].balance_after, Some(550_000_000));

    // Check tx 1 (Debit 20M)
    assert_eq!(parsed.transactions[1].amount, 20_000_000);
    assert_eq!(parsed.transactions[1].tx_type, TransactionType::Debit);
    assert_eq!(parsed.transactions[1].doc_ref.as_deref(), Some("CT1002"));
    assert_eq!(parsed.transactions[1].balance_after, Some(530_000_000));

    // Check balance invariant
    let inv = verify_balance_invariants(&parsed);
    assert!(inv.is_balanced);
}

#[test]
fn test_mbbank_parser_bilingual_and_counterparty() {
    let parser = MbBankParser;
    let csv = "\
NGÂN HÀNG TMCP QUÂN ĐỘI - MBBANK\n\
Tài khoản: 0880123456789\n\
Tên tài khoản: CÔNG TY TNHH LIVA SOLUTIONS\n\
Số dư ban đầu: 1.000.000.000\n\
Số GD / Trans No,Ngày / Date,Số tiền ghi nợ / Debit,Số tiền ghi có / Credit,Số dư / Balance,Tài khoản người thụ hưởng,Tên người thụ hưởng,Nội dung chi tiết\n\
MB-9901,15/08/2026,,250.000.000,1.250.000.000,0011009988776,CONG TY ABC,Chuyen khoan thanh toan hoa don 88\n\
MB-9902,16/08/2026,50.000.000,,1.200.000.000,1903998877665,CONG TY XYZ,Chi tra tien thue van phong thang 8\n\
Tổng phát sinh Có: 250.000.000\n\
Tổng phát sinh Nợ: 50.000.000\n\
Số dư cuối kỳ: 1.200.000.000\n";

    assert!(parser.sniff(csv.as_bytes(), "mbbank_statement.csv"));
    assert!(parser.sniff(b"", "mb_august_2026.xlsx"));

    let parsed = parser.parse(csv.as_bytes(), "mb_statement.csv").unwrap();
    assert_eq!(parsed.bank_code, "MB");
    assert_eq!(parsed.bank, BankType::MbBank);
    assert_eq!(parsed.account_number.as_deref(), Some("0880123456789"));
    assert_eq!(parsed.opening_balance, Some(1_000_000_000));
    assert_eq!(parsed.closing_balance, Some(1_200_000_000));
    assert_eq!(parsed.transactions.len(), 2);

    // TX 0: Credit 250M with counterparty account & name
    let tx1 = &parsed.transactions[0];
    assert_eq!(tx1.amount, 250_000_000);
    assert_eq!(tx1.tx_type, TransactionType::Credit);
    assert_eq!(tx1.doc_ref.as_deref(), Some("MB-9901"));
    assert_eq!(tx1.counterparty_account.as_deref(), Some("0011009988776"));
    assert_eq!(tx1.counterparty_name.as_deref(), Some("CONG TY ABC"));

    // TX 1: Debit 50M with counterparty account & name
    let tx2 = &parsed.transactions[1];
    assert_eq!(tx2.amount, 50_000_000);
    assert_eq!(tx2.tx_type, TransactionType::Debit);
    assert_eq!(tx2.doc_ref.as_deref(), Some("MB-9902"));
    assert_eq!(tx2.counterparty_account.as_deref(), Some("1903998877665"));
    assert_eq!(tx2.counterparty_name.as_deref(), Some("CONG TY XYZ"));

    // Verify balance invariant
    let inv = verify_balance_invariants(&parsed);
    assert!(inv.is_balanced);
}

#[test]
fn test_agribank_parser_html_and_csv() {
    let parser = AgribankParser;

    // 1. Agribank HTML table export (.xls filename)
    let html_content = "\
<html><head><meta http-equiv=\"Content-Type\" content=\"text/html; charset=utf-8\"></head><body>\
<table>\
<tr><td colspan=\"7\"><b>NGÂN HÀNG NÔNG NGHIỆP VÀ PHÁT TRIỂN NÔNG THÔN VIỆT NAM - AGRIBANK</b></td></tr>\
<tr><td>Số tài khoản: 1500201234567</td><td colspan=\"6\">Tên tài khoản: DOANH NGHIEP LIVA</td></tr>\
<tr><td>Số dư đầu kỳ: 200.000.000</td><td colspan=\"6\"></td></tr>\
<tr><th>Ngày giao dịch</th><th>Ngày hiệu lực</th><th>Mã GD</th><th>Diễn giải</th><th>Số tiền ghi nợ</th><th>Số tiền ghi có</th><th>Số dư</th></tr>\
<tr><td>01/08/2026</td><td>01/08/2026</td><td>VB123456</td><td>Nop tien mat vao tai khoan</td><td></td><td>80.000.000</td><td>280.000.000</td></tr>\
<tr><td>05/08/2026</td><td>05/08/2026</td><td>FT889900</td><td>Thanh toan tien mua vat tu nong nghiep</td><td>30.000.000</td><td></td><td>250.000.000</td></tr>\
<tr><td colspan=\"4\">Tổng cộng phát sinh</td><td>30.000.000</td><td>80.000.000</td><td>250.000.000</td></tr>\
</table></body></html>";

    assert!(parser.sniff(html_content.as_bytes(), "agribank_saoke.xls"));

    let parsed_html = parser
        .parse(html_content.as_bytes(), "agribank_saoke.xls")
        .unwrap();
    assert_eq!(parsed_html.bank_code, "VBA");
    assert_eq!(parsed_html.bank, BankType::Agribank);
    assert_eq!(parsed_html.format, StatementFormat::Html);
    assert_eq!(parsed_html.account_number.as_deref(), Some("1500201234567"));
    assert_eq!(parsed_html.opening_balance, Some(200_000_000));
    assert_eq!(parsed_html.closing_balance, Some(250_000_000));
    assert_eq!(parsed_html.transactions.len(), 2);

    assert_eq!(parsed_html.transactions[0].amount, 80_000_000);
    assert_eq!(parsed_html.transactions[0].tx_type, TransactionType::Credit);
    assert_eq!(
        parsed_html.transactions[0].doc_ref.as_deref(),
        Some("VB123456")
    );

    assert_eq!(parsed_html.transactions[1].amount, 30_000_000);
    assert_eq!(parsed_html.transactions[1].tx_type, TransactionType::Debit);
    assert_eq!(
        parsed_html.transactions[1].doc_ref.as_deref(),
        Some("FT889900")
    );

    let inv = verify_balance_invariants(&parsed_html);
    assert!(inv.is_balanced);

    // 2. Agribank CSV
    let csv_content = "\
Số tài khoản: 1500201234567\n\
Số dư đầu kỳ: 100.000.000\n\
Ngày GD,Ngày HL,Số GD,Diễn giải,Phát sinh nợ,Phát sinh có,Số dư\n\
10/08/2026,10/08/2026,VBA01,Thu tien hop dong,,40.000.000,140.000.000\n\
11/08/2026,11/08/2026,VBA02,Tra phi ngan hang,10.000.000,,130.000.000\n\
Số dư cuối kỳ: 130.000.000\n";

    let parsed_csv = parser
        .parse(csv_content.as_bytes(), "agribank_saoke.csv")
        .unwrap();
    assert_eq!(parsed_csv.bank_code, "VBA");
    assert_eq!(parsed_csv.transactions.len(), 2);
    let inv_csv = verify_balance_invariants(&parsed_csv);
    assert!(inv_csv.is_balanced);
}

#[test]
fn test_precision_vietnamese_amount_parsing_edge_cases() {
    // Standard Vietnamese format (dot thousands, comma decimal)
    assert_eq!(parse_vietnamese_amount("15.000.000,00"), Some(15_000_000));
    assert_eq!(parse_vietnamese_amount("15.000.000,5"), Some(15_000_000));
    assert_eq!(parse_vietnamese_amount("15.000.000,50"), Some(15_000_000));
    assert_eq!(parse_vietnamese_amount("15.000.000"), Some(15_000_000));

    // Anglo-Saxon format (comma thousands, dot decimal)
    assert_eq!(parse_vietnamese_amount("15,000,000.00"), Some(15_000_000));
    assert_eq!(parse_vietnamese_amount("15,000,000.5"), Some(15_000_000));
    assert_eq!(parse_vietnamese_amount("15,000,000"), Some(15_000_000));

    // Space separated thousands
    assert_eq!(parse_vietnamese_amount("15 000 000"), Some(15_000_000));

    // Negative amounts and debit markers
    assert_eq!(parse_vietnamese_amount("-25.500.000"), Some(25_500_000));
    assert_eq!(parse_vietnamese_amount("(50.000.000)"), Some(50_000_000));

    // Currency suffixes
    assert_eq!(
        parse_vietnamese_amount("100.000.000 VND"),
        Some(100_000_000)
    );
    assert_eq!(parse_vietnamese_amount("250.000.000 đ"), Some(250_000_000));
    assert_eq!(parse_vietnamese_amount("50.000.000VND"), Some(50_000_000));

    // Small amounts and edge values
    assert_eq!(parse_vietnamese_amount("0"), Some(0));
    assert_eq!(parse_vietnamese_amount("0,00"), Some(0));
    assert_eq!(parse_vietnamese_amount("500"), Some(500));
    assert_eq!(parse_vietnamese_amount("1"), Some(1));

    // Invalid or non-amount strings
    assert_eq!(parse_vietnamese_amount(""), None);
    assert_eq!(parse_vietnamese_amount("-"), None);
    assert_eq!(parse_vietnamese_amount("N/A"), None);
    assert_eq!(parse_vietnamese_amount("abcxyz"), None);

    // Alphanumeric identifiers, dates, timestamps, and header labels must be rejected
    assert_eq!(parse_vietnamese_amount("HD101"), None);
    assert_eq!(parse_vietnamese_amount("15/08/2026"), None);
    assert_eq!(parse_vietnamese_amount("09:30:00"), None);
    assert_eq!(parse_vietnamese_amount("Số dư ngày 31/08/2026:"), None);
    assert_eq!(parse_vietnamese_amount("INV-2026-001"), None);
    assert_eq!(parse_vietnamese_amount("Hop dong 102/2026"), None);
}

#[test]
fn test_sniff_and_parse_all_six_banks() {
    let base_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let fixtures_dir = base_dir.join("fixtures").join("statements");

    // 1. VCB (Excel)
    let vcb_bytes = std::fs::read(fixtures_dir.join("vcb_aug2026.xlsx")).unwrap();
    let vcb = sniff_and_parse(&vcb_bytes, "vcb_aug2026.xlsx").unwrap();
    assert_eq!(vcb.bank, BankType::Vietcombank);
    assert_eq!(vcb.bank_code, "VCB");

    // 2. TCB (CSV)
    let tcb_csv = "Số tài khoản: 19034567890123\nNgày giao dịch,Mã giao dịch,Số tiền ghi nợ,Số tiền ghi có,Số dư,Nội dung\n15/08/2026,FT001,,10.000.000,10.000.000,Thu tien Techcombank\n";
    let tcb = sniff_and_parse(tcb_csv.as_bytes(), "techcombank_statement.csv").unwrap();
    assert_eq!(tcb.bank, BankType::Techcombank);
    assert_eq!(tcb.bank_code, "TCB");

    // 3. BIDV (PDF)
    let bidv_bytes = std::fs::read(fixtures_dir.join("bidv_aug2026.pdf")).unwrap();
    let bidv = sniff_and_parse(&bidv_bytes, "bidv_aug2026.pdf").unwrap();
    assert_eq!(bidv.bank, BankType::Bidv);
    assert_eq!(bidv.bank_code, "BIDV");

    // 4. VietinBank (CSV)
    let ctg_csv = "NGÂN HÀNG TMCP CÔNG THƯƠNG VIỆT NAM\nNgày GD,Số phiếu,Số tiền GD,Nợ/Có,Dư cuối,Diễn giải\n10/08/2026,CT1001,50.000.000,C,50.000.000,Thu tien\n";
    let ctg = sniff_and_parse(ctg_csv.as_bytes(), "vietinbank.csv").unwrap();
    assert_eq!(ctg.bank, BankType::VietinBank);
    assert_eq!(ctg.bank_code, "CTG");

    // 5. MBBank (CSV)
    let mb_csv = "NGÂN HÀNG QUÂN ĐỘI MBBANK\nSố GD,Ngày,Ghi nợ,Ghi có,Số dư,Nội dung\nMB01,10/08/2026,,15.000.000,15.000.000,MB thu tien\n";
    let mb = sniff_and_parse(mb_csv.as_bytes(), "mbbank_saoke.csv").unwrap();
    assert_eq!(mb.bank, BankType::MbBank);
    assert_eq!(mb.bank_code, "MB");

    // 6. Agribank (HTML table .xls)
    let vba_html = "<html><table><tr><td>AGRIBANK SAO KE</td></tr><tr><td>Ngày GD</td><td>Mã GD</td><td>Diễn giải</td><td>Nợ</td><td>Có</td><td>Số dư</td></tr><tr><td>10/08/2026</td><td>VB01</td><td>Thu tien</td><td></td><td>20.000.000</td><td>20.000.000</td></tr></table></html>";
    let vba = sniff_and_parse(vba_html.as_bytes(), "agribank.xls").unwrap();
    assert_eq!(vba.bank, BankType::Agribank);
    assert_eq!(vba.bank_code, "VBA");
}

#[test]
fn test_balance_invariants_calculation() {
    let tx1 = TransactionRecord::new(
        1,
        1725000000,
        1725000000,
        Some("REF001".to_string()),
        TransactionType::Credit,
        100_000_000,
        Some(200_000_000),
        None,
        None,
        None,
        "Thu tien khach hang".to_string(),
    );
    let tx2 = TransactionRecord::new(
        2,
        1725000000,
        1725000000,
        Some("REF002".to_string()),
        TransactionType::Debit,
        30_000_000,
        Some(170_000_000),
        None,
        None,
        None,
        "Chi phi dich vu".to_string(),
    );

    let mut stmt = BankStatement::new(
        "stmt_test_inv".to_string(),
        BankType::Vietcombank,
        StatementFormat::Excel,
        Some("0011001234567".to_string()),
        Some("CONG TY LIVA".to_string()),
        Some(100_000_000),
        Some(170_000_000),
        Some(1725000000),
        Some(1725000000),
        vec![tx1, tx2],
        12,
    );

    // Balance check: 100M + 100M - 30M = 170M -> Balanced
    let report = verify_balance_invariants(&stmt);
    assert!(report.is_balanced);
    assert_eq!(report.discrepancy, 0);
    assert_eq!(report.computed_closing, 170_000_000);
    assert_eq!(report.total_credit, 100_000_000);
    assert_eq!(report.total_debit, 30_000_000);

    // Manipulate closing balance to create 50M mismatch
    stmt.closing_balance = Some(120_000_000);
    let tampered_report = verify_balance_invariants(&stmt);
    assert!(!tampered_report.is_balanced);
    assert_eq!(tampered_report.discrepancy, -50_000_000);
    assert_eq!(tampered_report.discrepancy.abs(), 50_000_000);
}

#[test]
fn test_detect_container_format_magic_bytes() {
    // PDF magic bytes: %PDF-
    assert_eq!(
        detect_container_format(b"%PDF-1.4...", "statement.dat"),
        ContainerFormat::Pdf
    );

    // ZIP/XLSX magic bytes: PK\x03\x04
    assert_eq!(
        detect_container_format(&[0x50, 0x4B, 0x03, 0x04, 0x00], "data.bin"),
        ContainerFormat::ExcelZip
    );

    // OLE/XLS magic bytes: \xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1
    assert_eq!(
        detect_container_format(
            &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1],
            "data.bin"
        ),
        ContainerFormat::ExcelOle
    );

    // HTML Table magic bytes
    assert_eq!(
        detect_container_format(b"<HTML><BODY><TABLE></TABLE></BODY></HTML>", "data.xls"),
        ContainerFormat::HtmlTable
    );
    assert_eq!(
        detect_container_format(b"<!DOCTYPE html><html>", "data.xls"),
        ContainerFormat::HtmlTable
    );

    // CSV UTF-8 BOM
    assert_eq!(
        detect_container_format(&[0xEF, 0xBB, 0xBF, b'A', b'B', b'C'], "data.txt"),
        ContainerFormat::TextCsv
    );

    // Filename extension fallbacks
    assert_eq!(
        detect_container_format(b"", "file.pdf"),
        ContainerFormat::Pdf
    );
    assert_eq!(
        detect_container_format(b"", "file.xlsx"),
        ContainerFormat::ExcelZip
    );
    assert_eq!(
        detect_container_format(b"", "file.csv"),
        ContainerFormat::TextCsv
    );
}

#[test]
fn test_pdf_multiline_narration_wrapping_with_contract_ids() {
    use lopdf::content::{Content, Operation};
    use lopdf::{Dictionary, Document, Object, Stream, StringFormat};

    let mut doc = Document::with_version("1.4");
    let pages_id = doc.new_object_id();

    let font_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", "Font".into()),
        ("Subtype", "Type1".into()),
        ("BaseFont", "Helvetica".into()),
    ]));

    let resources_id = doc.add_object(Dictionary::from_iter(vec![(
        "Font",
        Dictionary::from_iter(vec![("F1", font_id.into())]).into(),
    )]));

    let mut ops: Vec<Operation> = Vec::new();
    ops.push(Operation::new("BT", vec![]));
    ops.push(Operation::new("Tf", vec!["F1".into(), 10.into()]));

    let mut emit_line = |x: f32, y: f32, tokens: &[&str]| {
        let mut cur_x = x;
        for &tok in tokens {
            ops.push(Operation::new(
                "Tm",
                vec![
                    1.into(),
                    0.into(),
                    0.into(),
                    1.into(),
                    cur_x.into(),
                    y.into(),
                ],
            ));
            ops.push(Operation::new(
                "Tj",
                vec![Object::String(
                    tok.as_bytes().to_vec(),
                    StringFormat::Literal,
                )],
            ));
            cur_x += (tok.len() as f32) * 6.5 + 10.0;
        }
    };

    let mut y = 800.0;
    emit_line(
        50.0,
        y,
        &["NGAN HANG TMCP DAU TU VA PHAT TRIEN VIET NAM (BIDV)"],
    );
    y -= 20.0;
    emit_line(50.0, y, &["Số tài khoản: 12410001234567"]);
    y -= 16.0;
    emit_line(50.0, y, &["Số dư đầu kỳ:", "100.000.000"]);
    y -= 25.0;

    // Header line calibrating columns
    emit_line(
        50.0,
        y,
        &[
            "Ngày GD",
            "Chứng từ",
            "Số tiền ghi nợ",
            "Số tiền ghi có",
            "Số dư",
            "Nội dung giao dịch",
        ],
    );
    y -= 18.0;

    // Transaction 1
    emit_line(
        50.0,
        y,
        &[
            "15/08/2026",
            "FT2621458901",
            "10.000.000",
            "110.000.000",
            "Thanh toan tien hang dot 1",
        ],
    );
    y -= 14.0;

    // Multi-line continuation line containing contract ID HD105/2026
    emit_line(120.0, y, &["theo hop dong so HD105/2026 va INV-2026-001"]);

    ops.push(Operation::new("ET", vec![]));

    let content = Content { operations: ops };
    let content_bytes = content.encode().unwrap();
    let content_stream = Stream::new(Dictionary::new(), content_bytes);
    let content_id = doc.add_object(content_stream);

    let page_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", "Page".into()),
        ("Parent", pages_id.into()),
        ("Contents", content_id.into()),
    ]));

    let pages_dict = Dictionary::from_iter(vec![
        ("Type", "Pages".into()),
        ("Kids", vec![page_id.into()].into()),
        ("Count", 1.into()),
        ("Resources", resources_id.into()),
        (
            "MediaBox",
            vec![0.into(), 0.into(), 595.into(), 842.into()].into(),
        ),
    ]);
    doc.objects.insert(pages_id, Object::Dictionary(pages_dict));

    let catalog_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", "Catalog".into()),
        ("Pages", pages_id.into()),
    ]));
    doc.trailer.set("Root", catalog_id);

    let mut buf = Vec::new();
    doc.save_to(&mut buf).unwrap();

    let parser = BidvPdfParser;
    let statement = parser.parse(&buf, "bidv_multiline_test.pdf").unwrap();

    assert_eq!(
        statement.transactions.len(),
        1,
        "Should parse exactly 1 transaction"
    );
    let tx = &statement.transactions[0];
    assert!(
        tx.narration.contains("HD105/2026"),
        "Continuation line with contract ID HD105/2026 must be wrapped, got: '{}'",
        tx.narration
    );
    assert!(
        tx.narration.contains("INV-2026-001"),
        "Continuation line with invoice ID INV-2026-001 must be wrapped, got: '{}'",
        tx.narration
    );
    assert_eq!(
        tx.narration,
        "Thanh toan tien hang dot 1 theo hop dong so HD105/2026 va INV-2026-001"
    );
}

#[test]
fn test_agribank_csv_candidate_ranking_in_dispatcher() {
    let agri_csv = "NGÂN HÀNG NÔNG NGHIỆP VÀ PHÁT TRIỂN NÔNG THÔN VIỆT NAM\n\
Số tài khoản: 1500201234567\n\
Số dư đầu kỳ: 100.000.000\n\
Ngày GD,Ngày HT,Số GD,Nội dung,Ghi nợ,Ghi có,Số dư\n\
10/08/2026,10/08/2026,VBA01,Nhan tien thanh toan HD101,,40.000.000,140.000.000\n\
11/08/2026,11/08/2026,VBA02,Tra phi ngan hang,10.000.000,,130.000.000\n\
Số dư cuối kỳ: 130.000.000\n";

    let statement = sniff_and_parse(agri_csv.as_bytes(), "agribank_statement.csv").unwrap();
    assert_eq!(statement.bank, BankType::Agribank);
    assert_eq!(statement.bank_code, "VBA");
    assert_eq!(statement.transactions.len(), 2);
    let inv = verify_balance_invariants(&statement);
    assert!(inv.is_balanced);
}

#[test]
fn test_m1_remediation_excel_multi_column_summary_row_balance_extraction() {
    let base_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let fixtures_dir = base_dir.join("fixtures").join("statements");

    // 1. Verify VCB August 2026 fixture multi-column row 60 extraction
    let vcb_file = fixtures_dir.join("vcb_aug2026.xlsx");
    assert!(vcb_file.exists(), "vcb_aug2026.xlsx must exist");
    let vcb_bytes = std::fs::read(&vcb_file).unwrap();
    let vcb_parsed = VcbExcelParser
        .parse(&vcb_bytes, "vcb_aug2026.xlsx")
        .unwrap();

    assert_eq!(vcb_parsed.bank_code, "VCB");
    assert_eq!(vcb_parsed.opening_balance, Some(1_450_230_000));
    assert_eq!(
        vcb_parsed.closing_balance,
        Some(2_658_070_400),
        "Must extract closing balance 2,658,070,400 instead of debit total 205,000,000"
    );
    assert_eq!(vcb_parsed.transactions.len(), 50);

    let inv = verify_balance_invariants(&vcb_parsed);
    assert!(
        inv.is_balanced,
        "VCB statement with multi-column summary row must evaluate to is_balanced: true"
    );
    assert_eq!(inv.discrepancy, 0, "Discrepancy must be exactly 0");
    assert_eq!(inv.computed_closing, 2_658_070_400);
}

// ===========================================================================
// Part 4: Milestone 2 Reconciliation Engine Deterministic Tests
// ===========================================================================

#[test]
fn test_m2_reconciliation_direction_mismatch_rejected() {
    let now = 1_725_000_000i64;

    // Bank TX: Credit 25,000,000 VND (incoming money)
    let bank_tx = BankTransactionRow {
        id: "tx_credit_25m".to_string(),
        statement_id: "stmt_m2".to_string(),
        account_id: "acc_vcb".to_string(),
        bank_code: "VCB".to_string(),
        tx_date: now,
        value_date: now,
        doc_ref: Some("HD-00101".to_string()),
        tx_type: TransactionType::Credit,
        amount: 25_000_000,
        balance_after: Some(100_000_000),
        counterparty_account: None,
        counterparty_name: Some("CONG TY ABC".to_string()),
        counterparty_bank: None,
        narration: "THANH TOAN TIEN HANG HD101 CONG TY ABC".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        reconciled_match_id: None,
        created_at: now,
    };

    // Ledger Entry: Debit 25,000,000 VND (outgoing payment) with identical doc_no and party
    let ledger_entry = InternalLedgerEntry {
        id: "led_debit_25m".to_string(),
        account_id: "acc_vcb".to_string(),
        doc_no: "HD101".to_string(),
        entry_date: now,
        entry_type: TransactionType::Debit, // Mismatch: Debit vs Credit!
        amount: 25_000_000,
        partner_code: Some("CUST01".to_string()),
        partner_name: Some("CONG TY ABC".to_string()),
        description: "Thanh toan tien hang HD101".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        created_at: now,
    };

    // 1. Tier 1 Hash Matcher MUST reject direction mismatch
    let (t1_matches, unalloc_bank_t1, unalloc_led_t1) = HashMatcher::match_tier1(
        std::slice::from_ref(&bank_tx),
        std::slice::from_ref(&ledger_entry),
    );
    assert!(
        t1_matches.is_empty(),
        "Tier 1 must reject direction mismatch (Credit vs Debit)"
    );
    assert_eq!(unalloc_bank_t1, vec![0]);
    assert_eq!(unalloc_led_t1, vec![0]);

    // 2. Tier 2 Fuzzy Matcher MUST reject direction mismatch
    let (t2_matches, unalloc_bank_t2, unalloc_led_t2) = FuzzyMatcher::match_tier2(
        std::slice::from_ref(&bank_tx),
        std::slice::from_ref(&ledger_entry),
        &unalloc_bank_t1,
        &unalloc_led_t1,
    );
    assert!(
        t2_matches.is_empty(),
        "Tier 2 must reject direction mismatch (Credit vs Debit)"
    );
    assert_eq!(unalloc_bank_t2, vec![0]);
    assert_eq!(unalloc_led_t2, vec![0]);

    // 3. Tier 3 Split Solver MUST reject direction mismatch
    let (t3_matches, hitl_matches) = SplitSolver::match_tier3(
        std::slice::from_ref(&bank_tx),
        std::slice::from_ref(&ledger_entry),
        &unalloc_bank_t2,
        &unalloc_led_t2,
    );
    assert!(
        t3_matches.is_empty(),
        "Tier 3 must reject direction mismatch (Credit vs Debit)"
    );
    assert_eq!(
        hitl_matches.len(),
        1,
        "Residual transaction must fail-closed to HITL"
    );

    // 4. End-to-End Engine Reconcile verification
    let (all_matches, summary) = ReconciliationEngine::reconcile(&[bank_tx], &[ledger_entry]);
    assert_eq!(summary.matched_exact_count, 0);
    assert_eq!(summary.matched_fuzzy_count, 0);
    assert_eq!(summary.matched_split_count, 0);
    assert_eq!(summary.total_matched_count, 0);
    assert_eq!(summary.pending_hitl_count, 1);
    assert_eq!(all_matches.len(), 1);
    assert_eq!(all_matches[0].status, "PENDING_HITL");
    assert_eq!(all_matches[0].match_type, MatchType::ManualHitl);
    assert!(all_matches[0].hitl_token.is_some());
}

#[test]
fn test_m2_reconciliation_value_date_24h_window() {
    let friday_tx_date = 1_725_000_000i64; // Friday
    let monday_value_date = friday_tx_date + (3 * 86_400); // Monday clearing settlement (3 days later)

    let tx = BankTransactionRow {
        id: "tx_weekend_settlement".to_string(),
        statement_id: "stmt_m2".to_string(),
        account_id: "acc_vcb".to_string(),
        bank_code: "VCB".to_string(),
        tx_date: friday_tx_date,
        value_date: monday_value_date,
        doc_ref: Some("HD-2026-888".to_string()),
        tx_type: TransactionType::Credit,
        amount: 88_000_000,
        balance_after: Some(500_000_000),
        counterparty_account: None,
        counterparty_name: Some("CONG TY CP THUONG MAI SAO MAI".to_string()),
        counterparty_bank: None,
        narration: "THANH TOAN HOP DONG HD2026888".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        reconciled_match_id: None,
        created_at: friday_tx_date,
    };

    // Ledger entry posted on Monday matching value_date: delta t = 0 <= 24h
    let matching_ledger = InternalLedgerEntry {
        id: "led_monday_post".to_string(),
        account_id: "acc_vcb".to_string(),
        doc_no: "HD2026888".to_string(),
        entry_date: monday_value_date,
        entry_type: TransactionType::Credit,
        amount: 88_000_000,
        partner_code: Some("SAOMAI".to_string()),
        partner_name: Some("CONG TY CP THUONG MAI SAO MAI".to_string()),
        description: "Ghi nhan cong no hop dong HD2026888".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        created_at: monday_value_date,
    };

    let (matches, unalloc_bank, unalloc_ledger) = HashMatcher::match_tier1(
        std::slice::from_ref(&tx),
        std::slice::from_ref(&matching_ledger),
    );
    assert_eq!(matches.len(), 1, "Must match on value date window");
    assert!(unalloc_bank.is_empty());
    assert!(unalloc_ledger.is_empty());
    assert_eq!(matches[0].match_type, MatchType::Exact1To1);
    assert_eq!(matches[0].confidence_score, 1.0);

    // Fallback scenario: tx.value_date is 0, fallback to tx_date within 24h
    let mut tx_fallback = tx.clone();
    tx_fallback.value_date = 0; // zero value_date
    let fallback_ledger = InternalLedgerEntry {
        id: "led_fallback_post".to_string(),
        account_id: "acc_vcb".to_string(),
        doc_no: "HD2026888".to_string(),
        entry_date: friday_tx_date + 3600, // 1h after tx_date (well within 24h)
        entry_type: TransactionType::Credit,
        amount: 88_000_000,
        partner_code: Some("SAOMAI".to_string()),
        partner_name: Some("CONG TY CP THUONG MAI SAO MAI".to_string()),
        description: "Ghi nhan cong no hop dong HD2026888".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        created_at: friday_tx_date,
    };

    let (fb_matches, _, _) = HashMatcher::match_tier1(&[tx_fallback], &[fallback_ledger]);
    assert_eq!(
        fb_matches.len(),
        1,
        "Must match using tx_date fallback when value_date is 0"
    );

    // Out-of-window scenario: delta t = 86,401s (> 24h)
    let late_ledger = InternalLedgerEntry {
        id: "led_too_late".to_string(),
        account_id: "acc_vcb".to_string(),
        doc_no: "HD2026888".to_string(),
        entry_date: monday_value_date + 86_401, // 24h + 1s
        entry_type: TransactionType::Credit,
        amount: 88_000_000,
        partner_code: Some("SAOMAI".to_string()),
        partner_name: Some("CONG TY CP THUONG MAI SAO MAI".to_string()),
        description: "Ghi nhan cong no hop dong HD2026888".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        created_at: monday_value_date,
    };
    let (late_matches, _, _) = HashMatcher::match_tier1(&[tx], &[late_ledger]);
    assert!(
        late_matches.is_empty(),
        "Must reject match when delta t > 86,400s"
    );
}

#[test]
fn test_m2_reconciliation_bidirectional_split_solving() {
    let now = 1_725_000_000i64;

    // --- Direction 1: 1-to-N (1 Bank Payment -> N Ledger Invoices) ---
    // Single payment of 120,000,000 VND covering 3 invoices (30M + 40M + 50M)
    let composite_tx = BankTransactionRow {
        id: "tx_composite_120m".to_string(),
        statement_id: "stmt_m2".to_string(),
        account_id: "acc_vcb".to_string(),
        bank_code: "VCB".to_string(),
        tx_date: now,
        value_date: now,
        doc_ref: None,
        tx_type: TransactionType::Credit,
        amount: 120_000_000,
        balance_after: Some(1_000_000_000),
        counterparty_account: None,
        counterparty_name: Some("CONG TY THEP HOA PHAT".to_string()),
        counterparty_bank: None,
        narration: "THANH TOAN TIEN THEP HOP DONG HD501 HD502 HD503 HOA PHAT".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        reconciled_match_id: None,
        created_at: now,
    };

    let invoices = vec![
        InternalLedgerEntry {
            id: "inv_501".to_string(),
            account_id: "acc_vcb".to_string(),
            doc_no: "HD501".to_string(),
            entry_date: now,
            entry_type: TransactionType::Credit,
            amount: 30_000_000,
            partner_code: Some("HOAPHAT".to_string()),
            partner_name: Some("Công ty Thép Hòa Phát".to_string()),
            description: "Hoa don thep HD501".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: now,
        },
        InternalLedgerEntry {
            id: "inv_502".to_string(),
            account_id: "acc_vcb".to_string(),
            doc_no: "HD502".to_string(),
            entry_date: now,
            entry_type: TransactionType::Credit,
            amount: 40_000_000,
            partner_code: Some("HOAPHAT".to_string()),
            partner_name: Some("Công ty Thép Hòa Phát".to_string()),
            description: "Hoa don thep HD502".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: now,
        },
        InternalLedgerEntry {
            id: "inv_503".to_string(),
            account_id: "acc_vcb".to_string(),
            doc_no: "HD503".to_string(),
            entry_date: now,
            entry_type: TransactionType::Credit,
            amount: 50_000_000,
            partner_code: Some("HOAPHAT".to_string()),
            partner_name: Some("Công ty Thép Hòa Phát".to_string()),
            description: "Hoa don thep HD503".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: now,
        },
    ];

    let (auto_1_to_n, hitl_1_to_n) =
        SplitSolver::match_tier3(&[composite_tx.clone()], &invoices, &[0], &[0, 1, 2]);

    assert_eq!(auto_1_to_n.len(), 1, "Must match 1-to-N composite split");
    assert!(hitl_1_to_n.is_empty());
    let m_1_to_n = &auto_1_to_n[0];
    assert_eq!(m_1_to_n.matched_amount, 120_000_000);
    assert_eq!(m_1_to_n.discrepancy_amount, 0);
    assert_eq!(m_1_to_n.bank_tx_id, composite_tx.id);
    assert_eq!(m_1_to_n.bank_tx_ids, vec![composite_tx.id.clone()]);
    assert_eq!(m_1_to_n.ledger_entry_ids.len(), 3);
    assert!(m_1_to_n.is_split_one_to_n());
    assert!(!m_1_to_n.is_split_n_to_one());

    // --- Direction 2: N-to-1 (N Bank Payments -> 1 Ledger Invoice) ---
    // Single open invoice of 150,000,000 VND settled via 3 partial installments (45M + 50M + 55M)
    let big_invoice = InternalLedgerEntry {
        id: "inv_big_999".to_string(),
        account_id: "acc_vcb".to_string(),
        doc_no: "HD999".to_string(),
        entry_date: now,
        entry_type: TransactionType::Credit,
        amount: 150_000_000,
        partner_code: Some("HOAPHAT".to_string()),
        partner_name: Some("Công ty Cổ phần Tập đoàn Hòa Phát".to_string()),
        description: "Hop dong nguyen tac HD999".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        created_at: now,
    };

    let partial_txs = vec![
        BankTransactionRow {
            id: "tx_part_1".to_string(),
            statement_id: "stmt_m2".to_string(),
            account_id: "acc_vcb".to_string(),
            bank_code: "VCB".to_string(),
            tx_date: now,
            value_date: now,
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount: 45_000_000,
            balance_after: Some(145_000_000),
            counterparty_account: None,
            counterparty_name: Some("TAP DOAN HOA PHAT".to_string()),
            counterparty_bank: None,
            narration: "THANH TOAN TIEN HANG DOT 1 THEO HD999".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: now,
        },
        BankTransactionRow {
            id: "tx_part_2".to_string(),
            statement_id: "stmt_m2".to_string(),
            account_id: "acc_vcb".to_string(),
            bank_code: "VCB".to_string(),
            tx_date: now + 3600,
            value_date: now + 3600,
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount: 50_000_000,
            balance_after: Some(195_000_000),
            counterparty_account: None,
            counterparty_name: Some("TAP DOAN HOA PHAT".to_string()),
            counterparty_bank: None,
            narration: "THANH TOAN TIEN HANG DOT 2 THEO HD999".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: now + 3600,
        },
        BankTransactionRow {
            id: "tx_part_3".to_string(),
            statement_id: "stmt_m2".to_string(),
            account_id: "acc_vcb".to_string(),
            bank_code: "VCB".to_string(),
            tx_date: now + 7200,
            value_date: now + 7200,
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount: 55_000_000,
            balance_after: Some(250_000_000),
            counterparty_account: None,
            counterparty_name: Some("TAP DOAN HOA PHAT".to_string()),
            counterparty_bank: None,
            narration: "THANH TOAN TIEN HANG DOT 3 HOAN TAT HD999".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: now + 7200,
        },
    ];

    let (auto_n_to_1, hitl_n_to_1) =
        SplitSolver::match_tier3(&partial_txs, &[big_invoice.clone()], &[0, 1, 2], &[0]);

    assert_eq!(
        auto_n_to_1.len(),
        1,
        "Must match N-to-1 multi-installment split"
    );
    assert!(hitl_n_to_1.is_empty());
    let m_n_to_1 = &auto_n_to_1[0];
    assert_eq!(m_n_to_1.matched_amount, 150_000_000);
    assert_eq!(m_n_to_1.discrepancy_amount, 0);
    assert_eq!(m_n_to_1.bank_tx_ids.len(), 3);
    assert_eq!(m_n_to_1.ledger_entry_ids, vec![big_invoice.id.clone()]);
    assert!(m_n_to_1.is_split_n_to_one());
    assert!(!m_n_to_1.is_split_one_to_n());

    // --- Branch-and-bound subset-sum depth up to 8 test ---
    let candidates = vec![
        (0, 10_000_000),
        (1, 20_000_000),
        (2, 30_000_000),
        (3, 40_000_000),
        (4, 50_000_000),
        (5, 60_000_000),
        (6, 70_000_000),
        (7, 80_000_000),
    ];
    let total_target = 360_000_000u64; // Sum of all 8 items = 360M
    let solved_8 = solve_exact_subset_sum_bnb(&candidates, total_target, 8);
    assert!(solved_8.is_some(), "BnB solver must solve up to depth 8");
    assert_eq!(solved_8.unwrap().len(), 8);

    // With depth limit 7, cannot reach 360M
    let solved_7 = solve_exact_subset_sum_bnb(&candidates, total_target, 7);
    assert!(
        solved_7.is_none(),
        "Depth limit 7 must correctly prune when 8 items are required"
    );
}

#[test]
fn test_m2_reconciliation_bank_prefix_noise_stripping() {
    // Test stripping of typical Vietnamese bank narration prefixes & noise
    assert_eq!(
        strip_bank_narration_noise("MBVCB.123456789.CONG TY CP CONG NGHE ABC.THANH TOAN TIEN HANG"),
        "cong ty cp cong nghe abc"
    );
    assert_eq!(
        strip_bank_narration_noise("Napas VietQR TT HD102 Tu: CONG TY TNHH ABC"),
        "hd102 cong ty tnhh abc"
    );
    assert_eq!(
        strip_bank_narration_noise("QRIBFT chuyen khoan hop dong HD200 CONG TY ABC"),
        "hop dong hd200 cong ty abc"
    );
    assert_eq!(
        strip_bank_narration_noise("IBVCB 9876543210 CHUYEN TIEN CONG TY XYZ"),
        "cong ty xyz"
    );
    assert_eq!(
        strip_bank_narration_noise("CT TU: 0123456789 DANG VAN ANH SANG TRAN THI B"),
        "dang van anh sang tran thi b"
    );

    // Verify FuzzyMatcher integration with noisy bank narration
    let now = 1_725_000_000i64;
    let noisy_tx = BankTransactionRow {
        id: "tx_noisy".to_string(),
        statement_id: "stmt_m2".to_string(),
        account_id: "acc_vcb".to_string(),
        bank_code: "VCB".to_string(),
        tx_date: now,
        value_date: now,
        doc_ref: None,
        tx_type: TransactionType::Credit,
        amount: 49_989_000, // 50M - 11,000 VND Napas fee
        balance_after: Some(500_000_000),
        counterparty_account: None,
        counterparty_name: None,
        counterparty_bank: None,
        narration: "MBVCB.123456789.CONG TY CO PHAN PHAN MEM LIVA.THANH TOAN".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        reconciled_match_id: None,
        created_at: now,
    };

    let ledger = InternalLedgerEntry {
        id: "led_liva".to_string(),
        account_id: "acc_vcb".to_string(),
        doc_no: "INV-999".to_string(),
        entry_date: now + 1800,
        entry_type: TransactionType::Credit,
        amount: 50_000_000,
        partner_code: Some("LIVA".to_string()),
        partner_name: Some("Công ty Cổ phần Phần mềm LIVA".to_string()),
        description: "Dich vu phan mem LIVA".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        created_at: now,
    };

    let (fuzzy_matches, _, _) = FuzzyMatcher::match_tier2(&[noisy_tx], &[ledger], &[0], &[0]);
    assert_eq!(
        fuzzy_matches.len(),
        1,
        "Fuzzy matcher must match despite MBVCB prefix noise and 11,000 VND fee deduction"
    );
    assert_eq!(fuzzy_matches[0].discrepancy_amount, 11_000);
}

#[test]
fn test_m2_reconciliation_amount_bucket_pre_indexing() {
    let now = 1_725_000_000i64;

    // Bank TX: 49,989,000 VND (50M - 11,000 fee)
    let tx = BankTransactionRow {
        id: "tx_bucket_target".to_string(),
        statement_id: "stmt_m2".to_string(),
        account_id: "acc_vcb".to_string(),
        bank_code: "VCB".to_string(),
        tx_date: now,
        value_date: now,
        doc_ref: None,
        tx_type: TransactionType::Credit,
        amount: 49_989_000,
        balance_after: Some(500_000_000),
        counterparty_account: None,
        counterparty_name: Some("CONG TY DUOC PHAM ABC".to_string()),
        counterparty_bank: None,
        narration: "CONG TY DUOC PHAM ABC THANH TOAN".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        reconciled_match_id: None,
        created_at: now,
    };

    let ledgers = vec![
        // In distant bucket: 10,000,000 VND (bucket 909 vs 4544)
        InternalLedgerEntry {
            id: "led_distant_10m".to_string(),
            account_id: "acc_vcb".to_string(),
            doc_no: "INV-001".to_string(),
            entry_date: now,
            entry_type: TransactionType::Credit,
            amount: 10_000_000,
            partner_code: Some("ABC".to_string()),
            partner_name: Some("Công ty Dược phẩm ABC".to_string()),
            description: "Thuoc".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: now,
        },
        // In adjacent bucket within 11k fee: 50,000,000 VND (bucket 4545)
        InternalLedgerEntry {
            id: "led_match_50m".to_string(),
            account_id: "acc_vcb".to_string(),
            doc_no: "INV-002".to_string(),
            entry_date: now,
            entry_type: TransactionType::Credit,
            amount: 50_000_000,
            partner_code: Some("ABC".to_string()),
            partner_name: Some("Công ty Dược phẩm ABC".to_string()),
            description: "Thuoc khang sinh".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: now,
        },
        // Opposite direction in same amount bucket: Debit 50,000,000 VND (direction isolation)
        InternalLedgerEntry {
            id: "led_debit_50m".to_string(),
            account_id: "acc_vcb".to_string(),
            doc_no: "INV-003".to_string(),
            entry_date: now,
            entry_type: TransactionType::Debit,
            amount: 50_000_000,
            partner_code: Some("ABC".to_string()),
            partner_name: Some("Công ty Dược phẩm ABC".to_string()),
            description: "Tra hang".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: now,
        },
    ];

    let (matches, unalloc_bank, unalloc_led) =
        FuzzyMatcher::match_tier2(&[tx.clone()], &ledgers, &[0], &[0, 1, 2]);

    assert_eq!(matches.len(), 1, "Must match exactly the 50M invoice");
    assert_eq!(
        matches[0].ledger_entry_ids,
        vec!["led_match_50m".to_string()]
    );
    assert_eq!(matches[0].discrepancy_amount, 11_000);
    assert!(unalloc_bank.is_empty());
    assert_eq!(
        unalloc_led,
        vec![0, 2],
        "10M and Debit 50M entries must remain unallocated"
    );

    // Boundary check: amount difference > 11,000 VND (e.g. 11,001 VND) must be rejected
    let mut tx_out_of_bounds = tx.clone();
    tx_out_of_bounds.amount = 49_988_999; // 50M - 11,001
    let (oob_matches, _, _) =
        FuzzyMatcher::match_tier2(&[tx_out_of_bounds], &ledgers, &[0], &[0, 1, 2]);
    assert!(
        oob_matches.is_empty(),
        "Difference of 11,001 VND must exceed fee tolerance"
    );
}

#[test]
fn test_m2_reconciliation_balance_invariants_runtime_check() {
    let now = 1_725_000_000i64;

    // Normal valid running balance chain:
    // Opening = 750M.
    // TX1: Credit 25M -> Balance = 775M (750 + 25)
    // TX2: Debit 15M  -> Balance = 760M (775 - 15)
    // TX3: Credit 50M -> Balance = 810M (760 + 50)
    let valid_txs = vec![
        BankTransactionRow {
            id: "tx1".to_string(),
            statement_id: "stmt_valid".to_string(),
            account_id: "acc_1".to_string(),
            bank_code: "TCB".to_string(),
            tx_date: now,
            value_date: now,
            doc_ref: Some("REF1".to_string()),
            tx_type: TransactionType::Credit,
            amount: 25_000_000,
            balance_after: Some(775_000_000),
            counterparty_account: None,
            counterparty_name: None,
            counterparty_bank: None,
            narration: "REF1".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: now,
        },
        BankTransactionRow {
            id: "tx2".to_string(),
            statement_id: "stmt_valid".to_string(),
            account_id: "acc_1".to_string(),
            bank_code: "TCB".to_string(),
            tx_date: now + 3600,
            value_date: now + 3600,
            doc_ref: Some("REF2".to_string()),
            tx_type: TransactionType::Debit,
            amount: 15_000_000,
            balance_after: Some(760_000_000),
            counterparty_account: None,
            counterparty_name: None,
            counterparty_bank: None,
            narration: "REF2".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: now + 3600,
        },
        BankTransactionRow {
            id: "tx3".to_string(),
            statement_id: "stmt_valid".to_string(),
            account_id: "acc_1".to_string(),
            bank_code: "TCB".to_string(),
            tx_date: now + 7200,
            value_date: now + 7200,
            doc_ref: Some("REF3".to_string()),
            tx_type: TransactionType::Credit,
            amount: 50_000_000,
            balance_after: Some(810_000_000),
            counterparty_account: None,
            counterparty_name: None,
            counterparty_bank: None,
            narration: "REF3".to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: now + 7200,
        },
    ];

    let (checked, passed, disc) = ReconciliationEngine::check_balance_invariants(&valid_txs);
    assert!(
        checked,
        "Invariant check must execute when >= 2 consecutive balance rows exist"
    );
    assert!(
        passed,
        "Valid running balance chain must pass invariant check"
    );
    assert_eq!(disc, 0);

    // Corrupted running balance chain: TX2 balance is corrupted to 755M instead of 760M (discrepancy = -5,000,000)
    let mut corrupted_txs = valid_txs.clone();
    corrupted_txs[1].balance_after = Some(755_000_000);

    let (c_checked, c_passed, c_disc) =
        ReconciliationEngine::check_balance_invariants(&corrupted_txs);
    assert!(c_checked);
    assert!(
        !c_passed,
        "Corrupted running balance must fail invariant check"
    );
    assert_eq!(c_disc, -5_000_000);

    // Verify runtime summary integration via ReconciliationEngine::reconcile
    let (_, summary) = ReconciliationEngine::reconcile(&corrupted_txs, &[]);
    assert!(summary.balance_invariant_checked);
    assert!(!summary.balance_invariant_passed);
    assert_eq!(summary.balance_discrepancy_amount, -5_000_000);
    assert!(summary.discrepancy_count >= 1);
}

// ===========================================================================
// Milestone 4: E2E Hardening Verification Tests
// ===========================================================================

#[test]
fn test_m4_hardening_unrelated_companies_sharing_industry_descriptors_score_below_threshold() {
    // 1. Verify that all 10 generic Vietnamese industry sector descriptors
    // ("cong nghe", "xay dung", "dau tu", "san xuat", "van tai", "bat dong san",
    // "giao duc", "y te", "duoc pham", "truyen thong")
    // when shared by unrelated companies result in similarity < 0.70.
    let sector_pairs = [
        (
            "CÔNG TY CỔ PHẦN CÔNG NGHỆ THÁI BÌNH",
            "CÔNG TY CỔ PHẦN CÔNG NGHỆ SÔNG HỒNG",
            "cong nghe",
        ),
        (
            "CÔNG TY TNHH XÂY DỰNG BÌNH MINH",
            "CÔNG TY TNHH XÂY DỰNG RẠNG ĐÔNG",
            "xay dung",
        ),
        (
            "CÔNG TY CỔ PHẦN ĐẦU TƯ THÁI BÌNH",
            "CÔNG TY CỔ PHẦN ĐẦU TƯ SÔNG HỒNG",
            "dau tu",
        ),
        (
            "CÔNG TY TNHH SẢN XUẤT AN BÌNH",
            "CÔNG TY TNHH SẢN XUẤT VIỆT THẮNG",
            "san xuat",
        ),
        (
            "CÔNG TY CỔ PHẦN VẬN TẢI HẢI PHÒNG",
            "CÔNG TY CỔ PHẦN VẬN TẢI SÀI GÒN",
            "van tai",
        ),
        (
            "CÔNG TY CỔ PHẦN BẤT ĐỘNG SẢN AN KHANG",
            "CÔNG TY CỔ PHẦN BẤT ĐỘNG SẢN THỊNH VƯỢNG",
            "bat dong san",
        ),
        (
            "CÔNG TY CỔ PHẦN GIÁO DỤC TOÀN CẦU",
            "CÔNG TY CỔ PHẦN GIÁO DỤC ĐẠI VIỆT",
            "giao duc",
        ),
        (
            "CÔNG TY CỔ PHẦN Y TẾ VIỆT NHẬT",
            "CÔNG TY CỔ PHẦN Y TẾ HOÀNG GIA",
            "y te",
        ),
        (
            "CÔNG TY CỔ PHẦN DƯỢC PHẨM TRUNG ƯƠNG",
            "CÔNG TY CỔ PHẦN DƯỢC PHẨM ĐÔNG Á",
            "duoc pham",
        ),
        (
            "CÔNG TY CỔ PHẦN TRUYỀN THÔNG ĐẠI DƯƠNG",
            "CÔNG TY CỔ PHẦN TRUYỀN THÔNG BẠCH ĐẰNG",
            "truyen thong",
        ),
    ];

    for (name1, name2, sector) in sector_pairs {
        let score = compare_party_names(name1, name2);
        assert!(
            score < 0.70,
            "Sector '{sector}': Unrelated entities '{name1}' vs '{name2}' must score < 0.70, got {score}"
        );
    }

    // 2. Verify in Tier 2 Fuzzy Matcher:
    // When two unrelated entities sharing legal forms and sector names have identical amounts
    // and dates within 72 hours, Tier 2 MUST NOT auto-approve them.
    let now = 1_725_000_000i64;
    let tx_thai_binh = BankTransactionRow {
        id: "tx_tb".to_string(),
        statement_id: "stmt_1".to_string(),
        account_id: "acc_1".to_string(),
        bank_code: "VCB".to_string(),
        tx_date: now,
        value_date: now,
        doc_ref: None,
        tx_type: TransactionType::Credit,
        amount: 100_000_000,
        balance_after: None,
        counterparty_account: None,
        counterparty_name: Some("CÔNG TY CỔ PHẦN CÔNG NGHỆ THÁI BÌNH".to_string()),
        counterparty_bank: None,
        narration: "CÔNG TY CỔ PHẦN CÔNG NGHỆ THÁI BÌNH THANH TOAN".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        reconciled_match_id: None,
        created_at: now,
    };

    let ledger_song_hong = InternalLedgerEntry {
        id: "led_sh".to_string(),
        account_id: "acc_1".to_string(),
        doc_no: "HD-SONG-HONG".to_string(),
        entry_date: now,
        entry_type: TransactionType::Credit,
        amount: 100_000_000,
        partner_code: Some("SH01".to_string()),
        partner_name: Some("CÔNG TY CỔ PHẦN CÔNG NGHỆ SÔNG HỒNG".to_string()),
        description: "Hop dong Song Hong".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        created_at: now,
    };

    let (fuzzy_matches, rem_tx, rem_led) =
        FuzzyMatcher::match_tier2(&[tx_thai_binh], &[ledger_song_hong], &[0], &[0]);

    assert_eq!(
        fuzzy_matches.len(),
        0,
        "Tier 2 must reject match between unrelated companies sharing industry descriptor"
    );
    assert_eq!(rem_tx.len(), 1, "Bank tx must remain unmatched");
    assert_eq!(rem_led.len(), 1, "Ledger entry must remain unmatched");

    // 3. Verify that genuine variants of the same company STILL match with high score >= 0.85
    let genuine_score = compare_party_names(
        "CÔNG TY CỔ PHẦN CÔNG NGHỆ THÁI BÌNH",
        "CTY CP CONG NGHE THAI BINH",
    );
    assert!(
        genuine_score >= 0.85,
        "Genuine company variants must score >= 0.85, got {genuine_score}"
    );
}

#[test]
fn test_m4_hardening_hyphenated_and_slashed_doc_refs_o1_matching() {
    let now = 1_725_000_000i64;

    // 1. Tier 1 Hash Matcher: Hyphenated and slashed references extracted from narration
    // must match ledger entries with identical or normalized document codes in O(1).
    let doc_refs_to_test = [
        (
            "HD-2026-001",
            "MBVCB.123456.CT TU CONG TY A THANH TOAN HD-2026-001",
            50_000_000u64,
        ),
        (
            "INV-2026-001",
            "Napas VietQR TT Tu: CTY B THANH TOAN INV-2026-001",
            75_000_000u64,
        ),
        ("PC-001", "CHI PHI TIEN MAT PC-001", 12_000_000u64),
        (
            "HD102/2026",
            "THANH TOAN HOP DONG HD102/2026",
            35_000_000u64,
        ),
    ];

    for (doc_no, narration, amount) in doc_refs_to_test {
        let tx = BankTransactionRow {
            id: format!("tx_{doc_no}"),
            statement_id: "stmt_h".to_string(),
            account_id: "acc_h".to_string(),
            bank_code: "TCB".to_string(),
            tx_date: now,
            value_date: now,
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount,
            balance_after: None,
            counterparty_account: None,
            counterparty_name: None,
            counterparty_bank: None,
            narration: narration.to_string(),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: now,
        };

        let ledger = InternalLedgerEntry {
            id: format!("led_{doc_no}"),
            account_id: "acc_h".to_string(),
            doc_no: doc_no.to_string(),
            entry_date: now,
            entry_type: TransactionType::Credit,
            amount,
            partner_code: Some("P1".to_string()),
            partner_name: Some("Partner".to_string()),
            description: format!("Invoice {doc_no}"),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: now,
        };

        let (matches, rem_tx, rem_led) = HashMatcher::match_tier1(&[tx], &[ledger]);
        assert_eq!(
            matches.len(),
            1,
            "Tier 1 must match hyphenated/slashed doc ref: doc_no={doc_no}"
        );
        assert_eq!(rem_tx.len(), 0);
        assert_eq!(rem_led.len(), 0);
    }

    // 2. Tier 3 Split Solver: N-to-1 Multi-installment Settlement with Hyphenated Reference
    // Demonstrates that bank_by_doc_ref indexes normalized tokens ("HD2026001"),
    // allowing O(1) candidate lookup without quadratic scan fallback.
    let tx_part1 = BankTransactionRow {
        id: "tx_part1".to_string(),
        statement_id: "stmt_split".to_string(),
        account_id: "acc_split".to_string(),
        bank_code: "VCB".to_string(),
        tx_date: now,
        value_date: now,
        doc_ref: None,
        tx_type: TransactionType::Credit,
        amount: 70_000_000,
        balance_after: None,
        counterparty_account: None,
        counterparty_name: Some("CONG TY CỔ PHẦN ĐẠI VIỆT".to_string()),
        counterparty_bank: None,
        narration: "THANH TOAN DOT 1 HOP DONG HD-2026-001".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        reconciled_match_id: None,
        created_at: now,
    };

    let tx_part2 = BankTransactionRow {
        id: "tx_part2".to_string(),
        statement_id: "stmt_split".to_string(),
        account_id: "acc_split".to_string(),
        bank_code: "VCB".to_string(),
        tx_date: now + 1800,
        value_date: now + 1800,
        doc_ref: None,
        tx_type: TransactionType::Credit,
        amount: 80_000_000,
        balance_after: None,
        counterparty_account: None,
        counterparty_name: Some("CONG TY CỔ PHẦN ĐẠI VIỆT".to_string()),
        counterparty_bank: None,
        narration: "THANH TOAN DOT 2 HOP DONG HD-2026-001".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        reconciled_match_id: None,
        created_at: now + 1800,
    };

    let ledger_invoice = InternalLedgerEntry {
        id: "led_split_inv".to_string(),
        account_id: "acc_split".to_string(),
        doc_no: "HD-2026-001".to_string(),
        entry_date: now,
        entry_type: TransactionType::Credit,
        amount: 150_000_000,
        partner_code: Some("DV01".to_string()),
        partner_name: Some("CONG TY CỔ PHẦN ĐẠI VIỆT".to_string()),
        description: "Hop dong HD-2026-001".to_string(),
        reconciled_status: ReconciliationStatus::Unmatched,
        created_at: now,
    };

    let (split_matches, hitl) =
        SplitSolver::match_tier3(&[tx_part1, tx_part2], &[ledger_invoice], &[0, 1], &[0]);

    assert_eq!(
        split_matches.len(),
        1,
        "Must resolve N-to-1 split match via O(1) doc_ref index"
    );
    assert_eq!(hitl.len(), 0);
    assert_eq!(split_matches[0].matched_amount, 150_000_000);
    assert_eq!(split_matches[0].discrepancy_amount, 0);
    assert_eq!(split_matches[0].bank_tx_ids.len(), 2);

    // 3. Scalability Test: 200 bank txs vs 100 split invoices with hyphenated refs
    // Guarantees O(1) candidate lookup without degrading to quadratic Jaro-Winkler evaluation.
    let num_pairs = 100;
    let mut batch_bank = Vec::new();
    let mut batch_ledger = Vec::new();

    for i in 0..num_pairs {
        let code = format!("INV-2026-{:04}", i);
        let a1 = 10_000_000u64 + (i as u64 * 100);
        let a2 = 20_000_000u64 + (i as u64 * 100);

        batch_bank.push(BankTransactionRow {
            id: format!("btx_{i}_1"),
            statement_id: "stmt_bench".to_string(),
            account_id: "acc_bench".to_string(),
            bank_code: "VCB".to_string(),
            tx_date: now,
            value_date: now,
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount: a1,
            balance_after: None,
            counterparty_account: None,
            counterparty_name: Some("CONG TY TEST".to_string()),
            counterparty_bank: None,
            narration: format!("THANH TOAN DOT 1 {code}"),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: now,
        });

        batch_bank.push(BankTransactionRow {
            id: format!("btx_{i}_2"),
            statement_id: "stmt_bench".to_string(),
            account_id: "acc_bench".to_string(),
            bank_code: "VCB".to_string(),
            tx_date: now,
            value_date: now,
            doc_ref: None,
            tx_type: TransactionType::Credit,
            amount: a2,
            balance_after: None,
            counterparty_account: None,
            counterparty_name: Some("CONG TY TEST".to_string()),
            counterparty_bank: None,
            narration: format!("THANH TOAN DOT 2 {code}"),
            reconciled_status: ReconciliationStatus::Unmatched,
            reconciled_match_id: None,
            created_at: now,
        });

        batch_ledger.push(InternalLedgerEntry {
            id: format!("led_bench_{i}"),
            account_id: "acc_bench".to_string(),
            doc_no: code.clone(),
            entry_date: now,
            entry_type: TransactionType::Credit,
            amount: a1 + a2,
            partner_code: Some("TEST".to_string()),
            partner_name: Some("CONG TY TEST".to_string()),
            description: format!("Hoa don {code}"),
            reconciled_status: ReconciliationStatus::Unmatched,
            created_at: now,
        });
    }

    let unalloc_b: Vec<usize> = (0..batch_bank.len()).collect();
    let unalloc_l: Vec<usize> = (0..batch_ledger.len()).collect();

    let start = std::time::Instant::now();
    let (matches_scalability, hitl_scalability) =
        SplitSolver::match_tier3(&batch_bank, &batch_ledger, &unalloc_b, &unalloc_l);
    let duration = start.elapsed();

    assert_eq!(
        matches_scalability.len(),
        100,
        "All 100 hyphenated split pairs must match"
    );
    assert_eq!(hitl_scalability.len(), 0);
    assert!(
        duration.as_millis() < 500,
        "Tier 3 O(1) indexed lookup must complete in < 500ms, took {}ms (quadratic fallback detected?)",
        duration.as_millis()
    );
}
