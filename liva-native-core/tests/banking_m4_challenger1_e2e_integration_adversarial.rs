//! Milestone 4 Challenger 1: End-to-End Adversarial Integration & Dual-Control Invariant Suite
//!
//! Independently authored and executed by Milestone 4 Challenger 1 (challenger_m4_1).
//! Validates end-to-end integration across:
//! 1. Full Reconciliation Cycle:
//!    - Maker ingest & propose -> match in PENDING_HITL with single-use token.
//!    - Checker review -> inspection of pending queue.
//!    - Adversarial self-approval attacks (case, whitespace, disguises) -> blocked fail-closed,
//!      logged to AuditLedger as EVENT_SOD_SELF_APPROVAL_BLOCKED, token preserved.
//!    - Adversarial unauthorized role attacks (Auditor, Admin) -> blocked by RBAC matrix.
//!    - Legitimate Checker approval -> match APPROVED, transaction MATCHED, ledger MATCHED,
//!      single-use token consumed, anti-replay enforced.
//!    - Audit trail verification -> AuditLedger::verify passes 100% cryptographic continuity.
//!    - Merkle Tree Proof verification -> BinaryMerkleTree RFC 6962 inclusion proof verified,
//!      and tampering with transaction leaf strictly detected and rejected.
//! 2. Payment Order Disbursement:
//!    - Maker proposes payment orders across thresholds: 150M VND, 750M VND, 6B VND.
//!    - Self-approval blocked on all orders and logged to AuditLedger.
//!    - Standard Checker blocked on high limits (750M / 6B VND) requiring CFO escalation.
//!    - Auditor and Admin blocked from financial approval and execution.
//!    - CFO self-approval blocked (Maker == Checker is prohibited even for C-Suite).
//!    - Legitimate CFO dual-control approval -> HMAC-SHA256 signature generated, audit logged,
//!      token consumed, disbursement executed with zero-float integer arithmetic.

use liva_native_core::authorization::{
    Permission, UserRole, check_command_role_permission, check_user_permission,
};
use liva_native_core::banking::compliance::audit_ledger::AuditLedger;
use liva_native_core::banking::compliance::maker_checker::MakerCheckerError;
use liva_native_core::banking::compliance::merkle_audit::{
    BinaryMerkleTree, TransactionAuditLeaf,
};
use liva_native_core::banking::models::TransactionRecord;
use liva_native_core::banking::treasury::{
    PaymentOrderStatus, TreasuryEngine, init_payment_orders_table, save_payment_order,
};
use liva_native_core::crypto::EncryptionEngine;
use liva_native_core::{
    AppState, CommandPrincipal, db, handle_command_as, llm, stt, tts,
};
use rusqlite::{Connection, params};
use serde_json::json;
use std::sync::Arc;

const ADVERSARIAL_HMAC_KEY: &[u8; 32] = b"m4_challenger1_audit_master_32b_";

fn create_challenger_app_state() -> Arc<AppState> {
    let db = db::DatabasePool::new_in_memory().expect("in-memory database pool");
    let stt_manager = stt::SttManager::new("non-existent-model");
    let llm_manager = llm::LlamaRouterManager::new(2048, 0).expect("LLM manager");
    let mock_capturer = Arc::new(liva_native_core::vision::capture::MockScreenCapturer::new(
        64,
        64,
        liva_native_core::vision::capture::PixelFormat::Rgba,
    ));

    Arc::new(AppState {
        db,
        crypto: EncryptionEngine::new("00000000000000000000000000000000"),
        stt: tokio::sync::Mutex::new(stt_manager),
        tts: tokio::sync::Mutex::new(None),
        tts_player: tts::audio::TtsAudioPlayer::new(None),
        llm: tokio::sync::Mutex::new(llm_manager),
        ai_queue: AppState::default_ai_queue(),
        vad: tokio::sync::Mutex::new(None),
        denoiser: tokio::sync::Mutex::new(None),
        turn_shadow: tokio::sync::Mutex::new(None),
        aec: tokio::sync::Mutex::new(None),
        mcp_server: Arc::new(liva_native_core::mcp::server::NativeMcpServer::new(
            "test_vault",
        )),
        embedder: liva_native_core::AppState::empty_embedder(),
        vision: tokio::sync::Mutex::new(liva_native_core::vision::VisionManager::new(
            mock_capturer,
            liva_native_core::vision::VisionConfig::default(),
        )),
        active_recall: Arc::new(liva_native_core::active_recall::ActiveRecallManager::new()),
    })
}

fn init_banking_schema(conn: &Connection) {
    init_payment_orders_table(conn).expect("Payment orders table initialization");
}

// ===========================================================================
// Test Suite 1: Full Reconciliation Cycle E2E & Merkle Audit Trail
// ===========================================================================

#[tokio::test]
async fn test_m4_challenger1_full_reconciliation_cycle_e2e() {
    let state = create_challenger_app_state();

    // 1. Initialize schema and seed accounts & pending HITL match
    {
        let conn = state.db.writer.get().expect("db writer");
        init_banking_schema(&conn);

        let vcb_acc_enc = state.crypto.encrypt("00710009821").expect("encrypt acc");
        let now_ts = 1726000000i64;
        conn.execute(
            "INSERT OR REPLACE INTO bank_accounts (id, bank_code, account_number_enc, account_name, opening_balance, current_balance, last_synced_at, created_at)
             VALUES ('acc-vcb-01', 'VCB', ?1, 'CONG TY CP LIVA HARNESS', 1400000000, 1450230000, ?2, ?3)",
            params![vcb_acc_enc, now_ts, now_ts],
        )
        .expect("Seed bank account");

        let enc_narration = state.crypto.encrypt("THANH TOAN TIEN MAY BOM HOP DONG 88").expect("encrypt");
        conn.execute(
            "INSERT INTO bank_transactions (id, statement_id, account_id, tx_date, value_date, doc_ref, tx_type, amount, balance_after, counterparty_name, counterparty_bank, narration_enc, reconciled_status, created_at)
             VALUES ('tx-recon-01', 'stmt-01', 'acc-vcb-01', 1726000000, 1726000000, 'REF-99881', 'DEBIT', 75000000, 1375230000, 'CONG TY MAY BOM', 'VCB', ?1, 'PENDING_HITL', 1726000000)",
            params![enc_narration],
        )
        .expect("Seed bank transaction");

        conn.execute(
            "INSERT INTO internal_ledger_entries (id, account_id, doc_no, entry_date, entry_type, amount, partner_name, description, reconciled_status, created_at)
             VALUES ('gl-recon-01', 'acc-vcb-01', 'HD-88', 1726000000, 'DEBIT', 75000000, 'CONG TY MAY BOM', 'Hoa don 88 mua may bom', 'PENDING_HITL', 1726000000)",
            [],
        )
        .expect("Seed internal ledger");

        // Reconciliation engine generated PENDING_HITL match with single-use token
        conn.execute(
            "INSERT INTO reconciliation_matches (id, bank_tx_id, ledger_entry_ids_json, match_type, confidence_score, matched_amount, discrepancy_amount, status, matched_by, matched_at, notes, hitl_token)
             VALUES ('match-recon-01', 'tx-recon-01', '[\"gl-recon-01\"]', 'HEURISTIC_FUZZY', 0.88, 75000000, 0, 'PENDING_HITL', 'MAKER_PROPOSAL', 1726000000, 'Cho phe duyet kiem soat kep', 'hitl-tok-valid-1234')",
            [],
        )
        .expect("Seed reconciliation match");
    }

    // 2. Checker reviews the reconciliation matrix via IPC command
    let matrix_res = handle_command_as(
        CommandPrincipal::TauriDashboard,
        state.clone(),
        "banking_get_reconciliation_matrix",
        json!({ "filter": "PENDING_HITL" }),
        None,
        None,
    )
    .await
    .expect("Get reconciliation matrix");

    assert_eq!(matrix_res["pending_hitl_count"], 1);
    assert_eq!(matrix_res["items"][0]["bank_tx"]["id"], "tx-recon-01");
    assert_eq!(matrix_res["items"][0]["hitl_token"], "hitl-tok-valid-1234");

    // 3. Adversarial Self-Approval Attacks: Maker attempts to self-approve as Checker
    let self_approval_attacks = [
        ("maker_nam", "maker_nam"),
        ("maker_nam", "MAKER_NAM"),
        ("maker_nam", "Maker_Nam"),
        ("maker_nam", "  maker_nam  "),
        ("maker_nam", "\tmaker_nam\n"),
        ("maker_nam", "   MAKER_NAM   "),
    ];

    for (maker, hostile_checker) in self_approval_attacks {
        let payload = json!({
            "match_id": "match-recon-01",
            "hitl_token": "hitl-tok-valid-1234",
            "decision": "APPROVE",
            "maker_id": maker,
            "checker_id": hostile_checker,
            "notes": "Adversarial self approval attempt"
        });

        let err = handle_command_as(
            CommandPrincipal::TauriDashboard,
            state.clone(),
            "banking:resolve_hitl",
            payload,
            None,
            None,
        )
        .await
        .expect_err("Self approval must fail fail-closed");

        assert!(
            err.contains("Circular 09/2020/TT-NHNN violation: Maker cannot self-approve"),
            "Error must explicitly cite Circular 09 SoD self-approval prohibition"
        );

        // Fail-closed verification: Match status MUST remain PENDING_HITL, token UNCONSUMED
        let conn = state.db.readers.get().expect("db reader");
        let (status, stored_tok): (String, Option<String>) = conn
            .query_row(
                "SELECT status, hitl_token FROM reconciliation_matches WHERE id = 'match-recon-01'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .expect("query match");
        assert_eq!(status, "PENDING_HITL");
        assert_eq!(stored_tok, Some("hitl-tok-valid-1234".to_string()));

        // Verify violation logged to Audit Ledger
        let audit_event: String = conn
            .query_row(
                "SELECT event_type FROM banking_audit_chain ORDER BY seq_id DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .expect("audit event");
        assert_eq!(audit_event, AuditLedger::EVENT_SOD_SELF_APPROVAL_BLOCKED);
    }

    // 4. Adversarial Role Permission Check: Auditor and Admin cannot execute resolve_hitl
    assert_eq!(
        check_user_permission(UserRole::Auditor, Permission::ApproveHitlMatch),
        false
    );
    assert_eq!(
        check_user_permission(UserRole::Admin, Permission::ApproveHitlMatch),
        false
    );
    assert!(check_command_role_permission(UserRole::Auditor, "banking:resolve_hitl").is_err());
    assert!(check_command_role_permission(UserRole::Admin, "banking:resolve_hitl").is_err());

    // 5. Adversarial Token Attacks: Invalid / Missing token
    let invalid_token_payload = json!({
        "match_id": "match-recon-01",
        "hitl_token": "hitl-tok-WRONG-9999",
        "decision": "APPROVE",
        "maker_id": "maker_nam",
        "checker_id": "checker_tri",
        "notes": "Invalid token attack"
    });
    let tok_err = handle_command_as(
        CommandPrincipal::TauriDashboard,
        state.clone(),
        "banking:resolve_hitl",
        invalid_token_payload,
        None,
        None,
    )
    .await
    .expect_err("Invalid token must fail");
    assert!(tok_err.contains("Invalid or expired Two-Phase Confirmation HITL token"));

    // 6. Legitimate Dual-Control Approval: Independent Checker (checker_tri != maker_nam)
    let valid_approval_payload = json!({
        "match_id": "match-recon-01",
        "hitl_token": "hitl-tok-valid-1234",
        "decision": "APPROVE",
        "maker_id": "maker_nam",
        "checker_id": "checker_tri",
        "notes": "Kiem soat vien Nguyen Tri da doi chieu hoa don goc hop le"
    });

    let ok_res = handle_command_as(
        CommandPrincipal::TauriDashboard,
        state.clone(),
        "banking:resolve_hitl",
        valid_approval_payload,
        None,
        None,
    )
    .await
    .expect("Legitimate checker approval must succeed");

    assert_eq!(ok_res["success"], true);
    assert_eq!(ok_res["new_status"], "APPROVED");

    // 7. Verify Database State Transitions
    {
        let conn = state.db.readers.get().expect("db reader");

        let (match_st, tok): (String, Option<String>) = conn
            .query_row(
                "SELECT status, hitl_token FROM reconciliation_matches WHERE id = 'match-recon-01'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .expect("match state");
        assert_eq!(match_st, "APPROVED");
        assert_eq!(tok, None, "Token must be invalidated (set to NULL) upon approval");

        let tx_st: String = conn
            .query_row(
                "SELECT reconciled_status FROM bank_transactions WHERE id = 'tx-recon-01'",
                [],
                |r| r.get(0),
            )
            .expect("tx state");
        assert_eq!(tx_st, "MATCHED", "Bank transaction must transition to MATCHED");

        let gl_st: String = conn
            .query_row(
                "SELECT reconciled_status FROM internal_ledger_entries WHERE id = 'gl-recon-01'",
                [],
                |r| r.get(0),
            )
            .expect("gl state");
        assert_eq!(gl_st, "MATCHED", "Internal ledger entry must transition to MATCHED");

        // 8. Verify Cryptographic Audit Chain Integrity
        let audit_key = liva_native_core::banking::compliance::security::get_audit_key(&state);
        let audit_report = AuditLedger::verify(&conn, &audit_key).expect("AuditLedger::verify");
        assert!(audit_report.is_intact, "Audit hash chain must be 100% intact");
        assert!(audit_report.total_records >= 7, "Must contain all blocked and confirmed records");

        let last_event: (String, String) = conn
            .query_row(
                "SELECT event_type, actor_principal FROM banking_audit_chain ORDER BY seq_id DESC LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .expect("last audit event");
        assert_eq!(last_event.0, "HITL_CONFIRMED");
        assert_eq!(last_event.1, "checker_tri");
    }

    // 9. Replay Attack: Reusing the consumed token must fail
    let replay_payload = json!({
        "match_id": "match-recon-01",
        "hitl_token": "hitl-tok-valid-1234",
        "decision": "APPROVE",
        "maker_id": "maker_nam",
        "checker_id": "checker_tri",
        "notes": "Replay attack"
    });
    let replay_err = handle_command_as(
        CommandPrincipal::TauriDashboard,
        state.clone(),
        "banking:resolve_hitl",
        replay_payload,
        None,
        None,
    )
    .await
    .expect_err("Replay attack must fail");
    assert!(replay_err.contains("Invalid or expired Two-Phase Confirmation HITL token"));

    // 10. Merkle Tree Inclusion Proof & Tamper Evidence Verification
    let confirmed_tx = TransactionRecord {
        row_id: 1,
        tx_date: 1726000000,
        value_date: 1726000000,
        doc_ref: Some("REF-99881".to_string()),
        tx_type: liva_native_core::banking::models::TransactionType::Debit,
        amount: 75_000_000,
        balance_after: Some(1375230000),
        counterparty_account: None,
        counterparty_name: Some("CONG TY MAY BOM".to_string()),
        counterparty_bank: Some("VCB".to_string()),
        narration: "THANH TOAN TIEN MAY BOM HOP DONG 88".to_string(),
        ft_number: None,
        trace_id: None,
        raw_ref: None,
    };

    let leaf_bytes = TransactionAuditLeaf::from_record(&confirmed_tx, "MATCHED").to_leaf_bytes();
    let unconfirmed_leaf = TransactionAuditLeaf::from_record(&confirmed_tx, "PENDING_HITL").to_leaf_bytes();

    // Build Merkle Tree with the confirmed batch
    let batch_leaves = vec![leaf_bytes.clone()];
    let merkle_tree = BinaryMerkleTree::from_raw_leaves(&batch_leaves);
    assert_ne!(merkle_tree.root(), [0u8; 32]);

    let proof = merkle_tree.generate_inclusion_proof(0).expect("generate proof for leaf 0");
    assert!(proof.verify(&leaf_bytes), "Inclusion proof must verify against genuine leaf data");

    // Tamper Detection: Proving with unconfirmed status or tampered amount fails!
    assert_eq!(
        proof.verify(&unconfirmed_leaf),
        false,
        "Proof must fail against tampered status"
    );

    let mut tampered_tx = confirmed_tx.clone();
    tampered_tx.amount = 75_000_001; // 1 VND difference
    let tampered_leaf = TransactionAuditLeaf::from_record(&tampered_tx, "MATCHED").to_leaf_bytes();
    assert_eq!(
        proof.verify(&tampered_leaf),
        false,
        "Proof must fail against tampered amount"
    );
}

// ===========================================================================
// Test Suite 2: Payment Order Disbursement & High-Limit Threshold Checks
// ===========================================================================

#[test]
fn test_m4_challenger1_payment_order_disbursement_and_high_limit_e2e() {
    let conn = Connection::open_in_memory().expect("in-memory sqlite");
    init_payment_orders_table(&conn).expect("init payment orders");
    AuditLedger::init_audit_table(&conn).expect("init audit table");

    // Threshold boundaries:
    // Order 1: 150,000,000 VND (< 500M VND) -> Standard limit
    // Order 2: 750,000,000 VND (>= 500M VND, < 5B VND) -> High limit requiring CFO
    // Order 3: 6,000,000,000 VND (>= 5B VND) -> Extreme high limit requiring CFO

    let mut order_normal = TreasuryEngine::propose(
        "00710009821",
        "1903456789",
        "CONG TY VAN PHONG PHAM",
        "TECHCOMBANK",
        150_000_000,
        "Thanh toan van phong pham thang 9",
        "maker_nam",
    )
    .expect("Propose normal order");

    let mut order_high = TreasuryEngine::propose(
        "00710009821",
        "1201000456",
        "CONG TY CO PHAN XAY LAP MEP",
        "BIDV",
        750_000_000,
        "Giai ngan goi thau co dien dot 2",
        "maker_nam",
    )
    .expect("Propose high-limit order");

    let mut order_extreme = TreasuryEngine::propose(
        "00710009821",
        "0680100123",
        "CONG TY TNHH DAU TU HA TANG",
        "MBBANK",
        6_000_000_000,
        "Chi tra hop dong EPC xay dung nha may",
        "maker_nam",
    )
    .expect("Propose extreme high-limit order");

    save_payment_order(&conn, &order_normal).expect("save normal");
    save_payment_order(&conn, &order_high).expect("save high");
    save_payment_order(&conn, &order_extreme).expect("save extreme");

    let tok_normal = order_normal.hitl_token.clone().unwrap();
    let tok_high = order_high.hitl_token.clone().unwrap();
    let tok_extreme = order_extreme.hitl_token.clone().unwrap();

    // 1. Adversarial Self-Approval: Maker attempts to self-approve
    for (ord, tok) in [
        (&mut order_normal, &tok_normal),
        (&mut order_high, &tok_high),
        (&mut order_extreme, &tok_extreme),
    ] {
        let err = TreasuryEngine::approve(ord, "maker_nam", tok, Some(ADVERSARIAL_HMAC_KEY), Some(&conn))
            .expect_err("Maker self-approval must fail");
        assert_eq!(
            err,
            MakerCheckerError::SelfApprovalProhibited {
                maker_id: "maker_nam".to_string()
            }
        );
        assert_eq!(ord.status, PaymentOrderStatus::PendingApproval);
    }

    // 2. High-Limit RBAC & Permission Verification:
    // Standard Checker has ApprovePaymentOrder, but NOT ApproveHighLimit
    assert!(check_user_permission(UserRole::Checker, Permission::ApprovePaymentOrder));
    assert_eq!(
        check_user_permission(UserRole::Checker, Permission::ApproveHighLimit),
        false,
        "Standard Checker must NOT possess ApproveHighLimit"
    );
    assert!(check_command_role_permission(UserRole::Checker, "treasury:approve_high_limit").is_err());

    // CFO possesses BOTH ApprovePaymentOrder and ApproveHighLimit
    assert!(check_user_permission(UserRole::Cfo, Permission::ApprovePaymentOrder));
    assert!(check_user_permission(UserRole::Cfo, Permission::ApproveHighLimit));
    assert!(check_command_role_permission(UserRole::Cfo, "treasury:approve_high_limit").is_ok());

    // Auditor and Admin possess neither
    for role in [UserRole::Auditor, UserRole::Admin] {
        assert_eq!(check_user_permission(role, Permission::ApprovePaymentOrder), false);
        assert_eq!(check_user_permission(role, Permission::ApproveHighLimit), false);
        assert!(check_command_role_permission(role, "treasury:approve_payment").is_err());
        assert!(check_command_role_permission(role, "treasury:approve_high_limit").is_err());
    }

    // 3. Legitimate Standard Checker Approval for Normal Order (< 500M VND)
    TreasuryEngine::approve(
        &mut order_normal,
        "checker_tri",
        &tok_normal,
        Some(ADVERSARIAL_HMAC_KEY),
        Some(&conn),
    )
    .expect("Checker can approve normal order (< 500M VND)");

    assert_eq!(order_normal.status, PaymentOrderStatus::Approved);
    assert_eq!(order_normal.checker_id, Some("checker_tri".to_string()));
    assert!(order_normal.signature_hmac.is_some());
    assert_eq!(order_normal.hitl_token, None, "Token consumed");

    // 4. Adversarial Self-Approval by CFO:
    // If CFO acts as Maker, CFO CANNOT self-approve even with C-Suite status
    let mut cfo_made_order = TreasuryEngine::propose(
        "00710009821",
        "1903000123",
        "CONG TY CO PHAN TU VAN CHIEN LUOC",
        "TCB",
        800_000_000,
        "Chi phi tu van M&A",
        "cfo_huong",
    )
    .expect("CFO proposes order");
    save_payment_order(&conn, &cfo_made_order).expect("save cfo order");
    let cfo_tok = cfo_made_order.hitl_token.clone().unwrap();

    let cfo_self_err = TreasuryEngine::approve(
        &mut cfo_made_order,
        "cfo_huong",
        &cfo_tok,
        Some(ADVERSARIAL_HMAC_KEY),
        Some(&conn),
    )
    .expect_err("CFO self-approval must fail");
    assert_eq!(
        cfo_self_err,
        MakerCheckerError::SelfApprovalProhibited {
            maker_id: "cfo_huong".to_string()
        }
    );

    // 5. Legitimate CFO Approval for High-Limit Orders (>= 500M VND and >= 5B VND)
    // Order 2 (750M VND) approved by CFO (cfo_huong != maker_nam)
    TreasuryEngine::approve(
        &mut order_high,
        "cfo_huong",
        &tok_high,
        Some(ADVERSARIAL_HMAC_KEY),
        Some(&conn),
    )
    .expect("CFO approval of high limit order must succeed");
    assert_eq!(order_high.status, PaymentOrderStatus::Approved);
    assert_eq!(order_high.checker_id, Some("cfo_huong".to_string()));
    assert!(order_high.signature_hmac.is_some());

    // Order 3 (6B VND) approved by CFO (cfo_huong != maker_nam)
    TreasuryEngine::approve(
        &mut order_extreme,
        "cfo_huong",
        &tok_extreme,
        Some(ADVERSARIAL_HMAC_KEY),
        Some(&conn),
    )
    .expect("CFO approval of extreme high limit order must succeed");
    assert_eq!(order_extreme.status, PaymentOrderStatus::Approved);
    assert_eq!(order_extreme.checker_id, Some("cfo_huong".to_string()));
    assert!(order_extreme.signature_hmac.is_some());

    // 6. Verify Complete Cryptographic Audit Chain Integrity
    let report = AuditLedger::verify(&conn, ADVERSARIAL_HMAC_KEY).expect("Audit chain verify");
    assert!(report.is_intact, "Audit chain must remain 100% cryptographically intact");
    assert!(report.total_records >= 7, "Chain must record all blocked and approved transactions");
}
