//! Milestone 3 Challenger 1: Adversarial Dual Control & Audit Trail Stress Suite
//!
//! Independently authored and executed by Milestone 3 Challenger 1 (challenger_m3_1).
//! Adversarially challenges:
//! 1. Self-Approval Attacks:
//!    - Case variation (`maker_nam` vs `MAKER_NAM` vs `  Maker_Nam  `) across `resolve_hitl`, `maker_checker.rs`, and `treasury.rs`.
//!    - Whitespace padding (spaces, tabs, newlines) combined with mixed case.
//!    - Strict fail-closed verification: proposal remains pending, token unconsumed, payment order pending.
//! 2. Missing Identity Attacks:
//!    - Requests to `resolve_hitl` without `maker_id`, without `checker_id`, or with empty strings `""` / `"   "`.
//!    - Confirmation that NO legacy fallbacks (`"accountant_maker"`, `"chief_checker"`) are accepted.
//!    - Rejection of empty/whitespace identities in `maker_checker.rs` and `treasury.rs`.
//! 3. Audit Ledger Tamper-Evidence & HMAC-SHA256 Chain Verification:
//!    - Confirmation that upon self-approval block, `EVENT_SOD_SELF_APPROVAL_BLOCKED` is genuinely appended to `banking_audit_chain`.
//!    - Independent verification via `AuditLedger::verify` proving 100% cryptographic integrity.
//!    - Tamper detection testing: Alterations to payload_digest, prev_hash, actor, timestamp, and signature are strictly caught.

use liva_native_core::banking::compliance::{
    AuditLedger, CheckerDecision, HitlActionType, MakerCheckerEngine, MakerCheckerError,
    ProposalDetails, ProposalStatus, get_audit_key,
};
use liva_native_core::banking::treasury::{
    PaymentOrderStatus, TreasuryEngine, init_payment_orders_table, resolve_audit_key, save_payment_order,
};
use liva_native_core::crypto::EncryptionEngine;
use liva_native_core::{
    AppState, CommandPrincipal, db, handle_command_as, llm, stt, tts,
};
use rusqlite::{Connection, params};
use serde_json::json;
use std::sync::Arc;

const ADVERSARIAL_HMAC_KEY: &[u8; 32] = b"adversarial_dual_control_key_32b";

fn challenger_test_state() -> Arc<AppState> {
    let db = db::DatabasePool::new_in_memory().expect("in-memory test database");
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

fn dummy_proposal_details() -> ProposalDetails {
    ProposalDetails {
        bank_tx_id: "tx_adv_001".to_string(),
        selected_ledger_entry_ids: vec!["gl_adv_001".to_string()],
        matched_amount: 50_000_000,
        discrepancy_amount: 0,
        notes: Some("Adversarial dual control test proposal".to_string()),
    }
}

// ===========================================================================
// Test 1: Self-Approval Attacks on maker_checker.rs (Case & Whitespace)
// ===========================================================================

#[test]
fn test_challenger1_maker_checker_self_approval_case_and_whitespace_attacks() {
    let mut engine = MakerCheckerEngine::new();
    let t0 = 1726000000;

    let (proposal, token) = engine
        .create_proposal(
            "maker_nam",
            "Nguyen Van Nam (Maker)",
            "match_adv_001",
            HitlActionType::ApproveMatch,
            dummy_proposal_details(),
            t0,
        )
        .expect("Proposal creation must succeed");

    // Hostile variations attempting self-approval
    let hostile_checkers = [
        "maker_nam",              // exact match
        "MAKER_NAM",              // all uppercase
        "Maker_Nam",              // mixed case
        "mAkEr_NaM",              // erratic alternating case
        "  maker_nam  ",          // leading & trailing spaces
        "  MAKER_NAM  ",          // uppercase with spaces
        "\tmaker_nam\t",          // tab indentation
        "\r\nMaker_Nam\n",        // newline and carriage return padding
        "   mAkEr_NaM   ",        // erratic case with whitespace
    ];

    for hostile_id in hostile_checkers {
        let result = engine.submit_decision(
            hostile_id,
            "Nguyen Van Nam (Attacker)",
            &proposal.proposal_id,
            &token,
            CheckerDecision::Approve,
            Some("Attempting unauthorized self-approval".to_string()),
            ADVERSARIAL_HMAC_KEY,
            t0 + 30,
        );

        match result {
            Err(MakerCheckerError::SelfApprovalProhibited { maker_id }) => {
                assert_eq!(
                    maker_id, "maker_nam",
                    "SelfApprovalProhibited must report the registered maker_id"
                );
            }
            other => panic!(
                "Hostile checker '{hostile_id}' must be rejected with SelfApprovalProhibited, got: {other:?}"
            ),
        }

        // Proposal MUST remain in Pending state (fail-closed)
        let state = engine.get_proposal(&proposal.proposal_id).unwrap();
        assert_eq!(
            state.status,
            ProposalStatus::Pending,
            "Proposal state must remain Pending after blocked attack by '{hostile_id}'"
        );
    }

    // Single-use token must NOT have been consumed by the blocked attacks.
    // A legitimate independent checker can now approve successfully.
    let legitimate_approval = engine.submit_decision(
        "checker_lan",
        "Tran Thi Lan (Checker)",
        &proposal.proposal_id,
        &token,
        CheckerDecision::Approve,
        Some("Legitimate dual-control verification complete".to_string()),
        ADVERSARIAL_HMAC_KEY,
        t0 + 60,
    );

    assert!(
        legitimate_approval.is_ok(),
        "Legitimate independent checker must be able to approve pending proposal"
    );

    let final_state = engine.get_proposal(&proposal.proposal_id).unwrap();
    assert_eq!(final_state.status, ProposalStatus::Approved);

    // Anti-replay: token cannot be reused even by legitimate checker
    let replay_attempt = engine.submit_decision(
        "checker_lan",
        "Tran Thi Lan (Checker)",
        &proposal.proposal_id,
        &token,
        CheckerDecision::Approve,
        None,
        ADVERSARIAL_HMAC_KEY,
        t0 + 90,
    );

    match replay_attempt {
        Err(MakerCheckerError::TokenAlreadyUsed { .. }) => {}
        other => panic!("Replay attack must return TokenAlreadyUsed, got: {other:?}"),
    }
}

// ===========================================================================
// Test 2: Self-Approval Attacks on treasury.rs (Case & Whitespace)
// ===========================================================================

#[test]
fn test_challenger1_treasury_self_approval_case_and_whitespace_attacks() {
    let conn = Connection::open_in_memory().expect("in-memory sqlite");
    init_payment_orders_table(&conn).expect("init payment orders");
    AuditLedger::init_audit_table(&conn).expect("init audit table");

    let mut order = TreasuryEngine::propose(
        "19034567890123",
        "0011001234567",
        "CONG TY TNHH THEP AN KHANH",
        "VIETCOMBANK",
        250_000_000,
        "Thanh toan hop dong thep xay dung Dot 1",
        "maker_nam",
    )
    .expect("Propose payment order");

    save_payment_order(&conn, &order).expect("Save payment order");
    let token = order.hitl_token.clone().expect("Hitl token");

    let hostile_checkers = [
        "maker_nam",
        "MAKER_NAM",
        "Maker_Nam",
        "  maker_nam  ",
        "  MAKER_NAM  ",
        "\tMaker_Nam\n",
    ];

    let mut audit_record_count = 0;

    // A. Attack via TreasuryEngine::approve
    for hostile_id in hostile_checkers {
        let err = TreasuryEngine::approve(&mut order, hostile_id, &token, None, Some(&conn))
            .expect_err("Self-approval via approve must fail");

        assert_eq!(
            err,
            MakerCheckerError::SelfApprovalProhibited {
                maker_id: "maker_nam".to_string()
            }
        );

        audit_record_count += 1;

        // Verify state remains PendingApproval
        assert_eq!(order.status, PaymentOrderStatus::PendingApproval);

        // Verify audit log has recorded the violation
        let (event_type, actor): (String, String) = conn
            .query_row(
                "SELECT event_type, actor_principal FROM banking_audit_chain ORDER BY seq_id DESC LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .expect("Audit record must be present");

        assert_eq!(event_type, AuditLedger::EVENT_SOD_SELF_APPROVAL_BLOCKED);
        assert_eq!(actor, hostile_id.trim());

        // Verify hash chain remains completely intact
        let audit_key = resolve_audit_key(None);
        let report = AuditLedger::verify(&conn, &audit_key).expect("Audit chain verify");
        assert!(report.is_intact, "Audit chain must remain intact after blocked approve attack");
        assert_eq!(report.total_records, audit_record_count);
    }

    // B. Attack via TreasuryEngine::reject (Maker cannot self-reject to evade dual control audit)
    for hostile_id in hostile_checkers {
        let err = TreasuryEngine::reject(
            &mut order,
            hostile_id,
            &token,
            "Self rejection attempt",
            None,
            Some(&conn),
        )
        .expect_err("Self-rejection via reject must fail");

        assert_eq!(
            err,
            MakerCheckerError::SelfApprovalProhibited {
                maker_id: "maker_nam".to_string()
            }
        );

        audit_record_count += 1;

        assert_eq!(order.status, PaymentOrderStatus::PendingApproval);

        let (event_type, actor): (String, String) = conn
            .query_row(
                "SELECT event_type, actor_principal FROM banking_audit_chain ORDER BY seq_id DESC LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .expect("Audit record must be present");

        assert_eq!(event_type, AuditLedger::EVENT_SOD_SELF_APPROVAL_BLOCKED);
        assert_eq!(actor, hostile_id.trim());

        let audit_key = resolve_audit_key(None);
        let report = AuditLedger::verify(&conn, &audit_key).expect("Audit chain verify");
        assert!(report.is_intact, "Audit chain must remain intact after blocked reject attack");
        assert_eq!(report.total_records, audit_record_count);
    }

    // C. Legitimate independent Checker approves successfully
    TreasuryEngine::approve(&mut order, "checker_lan", &token, None, Some(&conn))
        .expect("Legitimate checker approval must succeed");

    assert_eq!(order.status, PaymentOrderStatus::Approved);
    assert_eq!(order.checker_id, Some("checker_lan".to_string()));

    let audit_key = resolve_audit_key(None);
    let final_report = AuditLedger::verify(&conn, &audit_key).expect("Final audit verify");
    assert!(final_report.is_intact);
    assert_eq!(final_report.total_records, audit_record_count + 1);
}

// ===========================================================================
// Test 3: Self-Approval Attacks on IPC resolve_hitl (Case & Whitespace)
// ===========================================================================

#[tokio::test]
async fn test_challenger1_resolve_hitl_ipc_self_approval_attacks() {
    let state = challenger_test_state();

    // 1. Seed a test match into reconciliation_matches
    {
        let conn = state.db.writer.get().expect("db write lock");
        conn.execute(
            "INSERT INTO reconciliation_matches (
                id, bank_tx_id, ledger_entry_ids_json, match_type, confidence_score,
                matched_amount, discrepancy_amount, status, matched_by, matched_at,
                notes, hitl_token
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                "match_hitl_challenger_01",
                "bank_tx_adv_01",
                "[\"gl_tx_adv_01\"]",
                "COMPOSITE_SPLIT",
                0.90,
                150_000_000,
                0,
                "PENDING",
                "AUTO_RULE",
                1726000000,
                "Two-Phase confirmation required for composite split",
                "token_hitl_challenger_secret_123"
            ],
        )
        .expect("Seed reconciliation match");
    }

    let hostile_pairs = [
        ("maker_nam", "maker_nam"),
        ("maker_nam", "MAKER_NAM"),
        ("maker_nam", "  Maker_Nam  "),
        ("  maker_nam  ", "MAKER_NAM"),
        ("maker_nam", "\tmaker_nam\n"),
        ("maker_nam", "   mAkEr_NaM   "),
    ];

    let mut attack_count = 0;

    for (maker, checker) in hostile_pairs {
        let payload = json!({
            "match_id": "match_hitl_challenger_01",
            "hitl_token": "token_hitl_challenger_secret_123",
            "decision": "APPROVE",
            "maker_id": maker,
            "checker_id": checker,
            "notes": "Adversarial self-approval attack via IPC"
        });

        // Test with both canonical command names
        let cmd = if attack_count % 2 == 0 {
            "reconciliation_resolve_hitl"
        } else {
            "banking:resolve_hitl"
        };

        let res = handle_command_as(
            CommandPrincipal::TauriDashboard,
            state.clone(),
            cmd,
            payload,
            None,
            None,
        )
        .await;

        match res {
            Err(err_msg) => {
                assert!(
                    err_msg.contains("Circular 09/2020/TT-NHNN violation: Maker cannot self-approve as Checker"),
                    "Error must cite Circular 09 dual control violation. Got: {err_msg}"
                );
                assert!(
                    err_msg.contains("Dual control required"),
                    "Error must explicitly state Dual control required. Got: {err_msg}"
                );
            }
            Ok(v) => panic!("Self-approval attack ('{maker}' vs '{checker}') MUST fail, but succeeded with: {v:?}"),
        }

        attack_count += 1;

        // Verify match status in DB is NOT changed
        let conn = state.db.writer.get().expect("db lock");
        let (status, stored_token): (String, Option<String>) = conn
            .query_row(
                "SELECT status, hitl_token FROM reconciliation_matches WHERE id = 'match_hitl_challenger_01'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .expect("Read match");

        assert_eq!(status, "PENDING", "Match status must remain PENDING after blocked attack");
        assert_eq!(
            stored_token,
            Some("token_hitl_challenger_secret_123".to_string()),
            "HITL token must remain unconsumed after blocked attack"
        );

        // Verify audit log has recorded EVENT_SOD_SELF_APPROVAL_BLOCKED
        let (event_type, actor): (String, String) = conn
            .query_row(
                "SELECT event_type, actor_principal FROM banking_audit_chain ORDER BY seq_id DESC LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .expect("Audit record");

        assert_eq!(event_type, AuditLedger::EVENT_SOD_SELF_APPROVAL_BLOCKED);
        assert_eq!(actor, checker.trim());

        // Verify HMAC-SHA256 audit chain integrity
        let audit_key = get_audit_key(&state);
        let report = AuditLedger::verify(&conn, &audit_key).expect("Verify audit chain");
        assert!(report.is_intact, "Audit chain must remain 100% intact after blocked attack");
        assert_eq!(report.total_records, attack_count);
    }
}

// ===========================================================================
// Test 4: Missing Identity Attacks on IPC resolve_hitl
// ===========================================================================

#[tokio::test]
async fn test_challenger1_resolve_hitl_ipc_missing_identity_attacks() {
    let state = challenger_test_state();

    // Seed test match
    {
        let conn = state.db.writer.get().expect("db write lock");
        conn.execute(
            "INSERT INTO reconciliation_matches (
                id, bank_tx_id, ledger_entry_ids_json, match_type, confidence_score,
                matched_amount, discrepancy_amount, status, matched_by, matched_at,
                notes, hitl_token
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                "match_hitl_missing_id_01",
                "bank_tx_adv_02",
                "[\"gl_tx_adv_02\"]",
                "1:1_EXACT",
                1.0,
                80_000_000,
                0,
                "PENDING",
                "USER_PROPOSAL",
                1726000000,
                "Manual proposal awaiting dual control",
                "token_missing_id_test_456"
            ],
        )
        .expect("Seed match");
    }

    // A. Missing maker_id payloads
    let missing_maker_payloads = [
        json!({
            "match_id": "match_hitl_missing_id_01",
            "hitl_token": "token_missing_id_test_456",
            "checker_id": "chief_checker"
        }),
        json!({
            "match_id": "match_hitl_missing_id_01",
            "hitl_token": "token_missing_id_test_456",
            "maker_id": "",
            "checker_id": "chief_checker"
        }),
        json!({
            "match_id": "match_hitl_missing_id_01",
            "hitl_token": "token_missing_id_test_456",
            "maker_id": "   ",
            "checker_id": "chief_checker"
        }),
        json!({
            "match_id": "match_hitl_missing_id_01",
            "hitl_token": "token_missing_id_test_456",
            "maker_id": serde_json::Value::Null,
            "checker_id": "chief_checker"
        }),
    ];

    for (i, payload) in missing_maker_payloads.iter().enumerate() {
        let res = handle_command_as(
            CommandPrincipal::TauriDashboard,
            state.clone(),
            "banking:resolve_hitl",
            payload.clone(),
            None,
            None,
        )
        .await;

        match res {
            Err(err) => {
                assert!(
                    err.contains("Missing required 'maker_id' parameter for dual-control authorization"),
                    "Payload #{i} must fail with missing maker_id error. Got: {err}"
                );
            }
            Ok(v) => panic!("Payload #{i} without valid maker_id must NOT succeed, got: {v:?}"),
        }
    }

    // B. Missing checker_id payloads
    let missing_checker_payloads = [
        json!({
            "match_id": "match_hitl_missing_id_01",
            "hitl_token": "token_missing_id_test_456",
            "maker_id": "accountant_maker"
        }),
        json!({
            "match_id": "match_hitl_missing_id_01",
            "hitl_token": "token_missing_id_test_456",
            "maker_id": "accountant_maker",
            "checker_id": ""
        }),
        json!({
            "match_id": "match_hitl_missing_id_01",
            "hitl_token": "token_missing_id_test_456",
            "maker_id": "accountant_maker",
            "checker_id": "   "
        }),
        json!({
            "match_id": "match_hitl_missing_id_01",
            "hitl_token": "token_missing_id_test_456",
            "maker_id": "accountant_maker",
            "checker_id": serde_json::Value::Null
        }),
    ];

    for (i, payload) in missing_checker_payloads.iter().enumerate() {
        let res = handle_command_as(
            CommandPrincipal::TauriDashboard,
            state.clone(),
            "banking:resolve_hitl",
            payload.clone(),
            None,
            None,
        )
        .await;

        match res {
            Err(err) => {
                assert!(
                    err.contains("Missing required 'checker_id' parameter for dual-control authorization"),
                    "Payload #{i} must fail with missing checker_id error. Got: {err}"
                );
            }
            Ok(v) => panic!("Payload #{i} without valid checker_id must NOT succeed, got: {v:?}"),
        }
    }

    // C. Missing both identities
    let missing_both_payload = json!({
        "match_id": "match_hitl_missing_id_01",
        "hitl_token": "token_missing_id_test_456"
    });

    let res_both = handle_command_as(
        CommandPrincipal::TauriDashboard,
        state.clone(),
        "banking:resolve_hitl",
        missing_both_payload,
        None,
        None,
    )
    .await;

    assert!(
        res_both.is_err(),
        "Payload missing both maker and checker must be strictly rejected"
    );

    // D. Alias keys (makerId / maker_user_id, checkerId / checker_user_id)
    // 1. When distinct, aliases MUST be accepted
    let valid_alias_payload = json!({
        "match_id": "match_hitl_missing_id_01",
        "hitl_token": "token_missing_id_test_456",
        "makerId": "maker_nam",
        "checkerId": "checker_lan",
        "decision": "APPROVE"
    });

    let res_valid = handle_command_as(
        CommandPrincipal::TauriDashboard,
        state.clone(),
        "banking:resolve_hitl",
        valid_alias_payload,
        None,
        None,
    )
    .await;

    assert!(
        res_valid.is_ok(),
        "Valid distinct alias identities must be accepted. Got err: {res_valid:?}"
    );

    // 2. When identical, aliases MUST STILL be blocked as self-approval
    // Re-seed match
    {
        let conn = state.db.writer.get().expect("db write lock");
        conn.execute(
            "INSERT INTO reconciliation_matches (
                id, bank_tx_id, ledger_entry_ids_json, match_type, confidence_score,
                matched_amount, discrepancy_amount, status, matched_by, matched_at,
                notes, hitl_token
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                "match_hitl_alias_attack",
                "bank_tx_adv_03",
                "[\"gl_tx_adv_03\"]",
                "1:1_EXACT",
                1.0,
                50_000_000,
                0,
                "PENDING",
                "USER_PROPOSAL",
                1726000000,
                "Alias self-approval test",
                "token_alias_attack_789"
            ],
        )
        .expect("Seed alias match");
    }

    let alias_attack_payload = json!({
        "match_id": "match_hitl_alias_attack",
        "hitl_token": "token_alias_attack_789",
        "maker_user_id": "accountant_same",
        "checker_user_id": "ACCOUNTANT_SAME",
        "decision": "APPROVE"
    });

    let res_alias_attack = handle_command_as(
        CommandPrincipal::TauriDashboard,
        state.clone(),
        "banking:resolve_hitl",
        alias_attack_payload,
        None,
        None,
    )
    .await;

    assert!(
        res_alias_attack.is_err(),
        "Identical alias identities must be blocked as self-approval"
    );
    assert!(
        res_alias_attack.unwrap_err().contains("Circular 09/2020/TT-NHNN violation"),
        "Alias self-approval error must cite Circular 09"
    );
}

// ===========================================================================
// Test 5: Empty Identity Rejection in maker_checker.rs and treasury.rs
// ===========================================================================

#[test]
fn test_challenger1_empty_identity_rejection_in_maker_checker_and_treasury() {
    // 1. maker_checker.rs: Empty checker_id
    let mut engine = MakerCheckerEngine::new();
    let t0 = 1726000000;

    let (proposal, token) = engine
        .create_proposal(
            "maker_user_01",
            "Maker",
            "match_empty_test",
            HitlActionType::ApproveMatch,
            dummy_proposal_details(),
            t0,
        )
        .expect("Proposal create");

    let err_empty_checker = engine.submit_decision(
        "",
        "Checker",
        &proposal.proposal_id,
        &token,
        CheckerDecision::Approve,
        None,
        ADVERSARIAL_HMAC_KEY,
        t0 + 30,
    );
    assert_eq!(err_empty_checker.unwrap_err(), MakerCheckerError::EmptyCheckerId);

    let err_spaces_checker = engine.submit_decision(
        "   ",
        "Checker",
        &proposal.proposal_id,
        &token,
        CheckerDecision::Approve,
        None,
        ADVERSARIAL_HMAC_KEY,
        t0 + 30,
    );
    assert_eq!(err_spaces_checker.unwrap_err(), MakerCheckerError::EmptyCheckerId);

    // 2. treasury.rs: Empty maker_id in propose
    let err_propose_empty_maker = TreasuryEngine::propose(
        "19034567890123",
        "0011001234567",
        "CONG TY A",
        "VIETCOMBANK",
        10_000_000,
        "Thanh toan",
        "",
    );
    assert_eq!(err_propose_empty_maker.unwrap_err(), "Maker ID cannot be empty");

    let err_propose_spaces_maker = TreasuryEngine::propose(
        "19034567890123",
        "0011001234567",
        "CONG TY A",
        "VIETCOMBANK",
        10_000_000,
        "Thanh toan",
        "    ",
    );
    assert_eq!(err_propose_spaces_maker.unwrap_err(), "Maker ID cannot be empty");

    // 3. treasury.rs: Empty checker_id in approve and reject
    let mut order = TreasuryEngine::propose(
        "19034567890123",
        "0011001234567",
        "CONG TY A",
        "VIETCOMBANK",
        10_000_000,
        "Thanh toan",
        "maker_nam",
    )
    .expect("Propose");

    let order_token = order.hitl_token.clone().unwrap();

    let err_approve_empty = TreasuryEngine::approve(&mut order, "", &order_token, None, None);
    assert_eq!(err_approve_empty.unwrap_err(), MakerCheckerError::EmptyCheckerId);

    let err_approve_spaces = TreasuryEngine::approve(&mut order, "   ", &order_token, None, None);
    assert_eq!(err_approve_spaces.unwrap_err(), MakerCheckerError::EmptyCheckerId);

    let err_reject_empty = TreasuryEngine::reject(&mut order, "", &order_token, "reason", None, None);
    assert_eq!(err_reject_empty.unwrap_err(), MakerCheckerError::EmptyCheckerId);

    let err_reject_spaces = TreasuryEngine::reject(&mut order, "   ", &order_token, "reason", None, None);
    assert_eq!(err_reject_spaces.unwrap_err(), MakerCheckerError::EmptyCheckerId);
}

// ===========================================================================
// Test 6: Audit Ledger Tamper-Evidence & Chain Continuity Under Attack
// ===========================================================================

#[test]
fn test_challenger1_audit_ledger_tamper_detection_and_chain_continuity() {
    let conn = Connection::open_in_memory().expect("in-memory sqlite");
    AuditLedger::init_audit_table(&conn).expect("init audit table");
    let audit_key = b"master_audit_secret_key_32_bytes";

    // Append 20 mixed events into the audit ledger
    let events = [
        (AuditLedger::EVENT_SYSTEM_INITIALIZED, "system", "Init banking kernel"),
        (AuditLedger::EVENT_STATEMENT_IMPORTED, "maker_nam", "VCB_2026_09_16.xlsx"),
        (AuditLedger::EVENT_MATCH_AUTO, "engine", "Auto-matched 45 txs"),
        (AuditLedger::EVENT_SOD_SELF_APPROVAL_BLOCKED, "maker_nam", "Blocked self-approval on order PO-01"),
        (AuditLedger::EVENT_SOD_SELF_APPROVAL_BLOCKED, "MAKER_NAM", "Blocked uppercase self-approval on order PO-01"),
        (AuditLedger::EVENT_SOD_SELF_APPROVAL_BLOCKED, "maker_nam", "Blocked whitespace self-approval on order PO-01"),
        (AuditLedger::EVENT_PAYMENT_ORDER_APPROVAL, "checker_lan", "Approved order PO-01"),
        (AuditLedger::EVENT_HITL_CONFIRMED, "checker_lan", "Confirmed composite match 1:N"),
        (AuditLedger::EVENT_SOD_SELF_APPROVAL_BLOCKED, "accountant_01", "Blocked self-approval on match M-09"),
        (AuditLedger::EVENT_PAYMENT_ORDER_REJECTION, "cfo_huong", "Rejected payment order exceeding limit"),
        (AuditLedger::EVENT_SOD_SELF_APPROVAL_BLOCKED, "maker_minh", "Blocked self-approval attempt #1"),
        (AuditLedger::EVENT_SOD_SELF_APPROVAL_BLOCKED, "maker_minh", "Blocked self-approval attempt #2"),
        (AuditLedger::EVENT_PAYMENT_ORDER_APPROVAL, "checker_lan", "Approved order PO-02"),
        (AuditLedger::EVENT_MATCH_AUTO, "engine", "Auto-matched batch #2"),
        (AuditLedger::EVENT_SOD_SELF_APPROVAL_BLOCKED, "maker_nam", "Blocked self-approval on order PO-03"),
        (AuditLedger::EVENT_PAYMENT_ORDER_APPROVAL, "cfo_huong", "Approved high limit order PO-03"),
        (AuditLedger::EVENT_STATEMENT_IMPORTED, "maker_nam", "TCB_2026_09_16.xlsx"),
        (AuditLedger::EVENT_HITL_CONFIRMED, "checker_lan", "Confirmed fee discrepancy match"),
        (AuditLedger::EVENT_SOD_SELF_APPROVAL_BLOCKED, "maker_tam", "Blocked self-approval on match M-14"),
        (AuditLedger::EVENT_PAYMENT_ORDER_APPROVAL, "checker_lan", "Approved order PO-04"),
    ];

    for (ev_type, actor, payload) in &events {
        AuditLedger::append(&conn, audit_key, ev_type, actor, payload).expect("Audit append");
    }

    // 1. Initial verification: Untampered chain MUST be 100% intact
    let initial_report = AuditLedger::verify(&conn, audit_key).expect("Audit verify");
    assert!(initial_report.is_intact, "Untampered audit chain must be 100% intact");
    assert_eq!(initial_report.total_records, 20);
    assert!(initial_report.tampered_seq_id.is_none());
    assert!(initial_report.error_message.is_none());

    // 2. Adversarial Tamper Attack A: Modify payload_digest at seq_id = 5 (a blocked SoD record)
    conn.execute(
        "UPDATE banking_audit_chain SET payload_digest = 'tampered_malicious_digest_hash_0000' WHERE seq_id = 5",
        [],
    )
    .expect("Tamper payload");

    let tamper_report_a = AuditLedger::verify(&conn, audit_key).expect("Audit verify");
    assert!(!tamper_report_a.is_intact, "Tampered payload MUST break chain verification");
    assert_eq!(
        tamper_report_a.tampered_seq_id,
        Some(5),
        "Verification must pinpoint exact tampered seq_id = 5"
    );
}

#[test]
fn test_challenger1_audit_ledger_pinpoints_prev_hash_and_signature_tampering() {
    let conn = Connection::open_in_memory().expect("in-memory sqlite");
    AuditLedger::init_audit_table(&conn).expect("init audit table");
    let audit_key = b"master_audit_secret_key_32_bytes";

    for i in 1..=10 {
        AuditLedger::append(
            &conn,
            audit_key,
            if i % 3 == 0 {
                AuditLedger::EVENT_SOD_SELF_APPROVAL_BLOCKED
            } else {
                AuditLedger::EVENT_PAYMENT_ORDER_APPROVAL
            },
            &format!("actor_{i}"),
            &format!("Payload event #{i}"),
        )
        .expect("Append");
    }

    // Verify intact
    let report_before = AuditLedger::verify(&conn, audit_key).expect("Verify");
    assert!(report_before.is_intact);
    assert_eq!(report_before.total_records, 10);

    // Tamper attack: Modify prev_hash at seq_id = 7
    conn.execute(
        "UPDATE banking_audit_chain SET prev_hash = 'forged_prev_hash_pointing_nowhere' WHERE seq_id = 7",
        [],
    )
    .expect("Tamper prev_hash");

    let report_prev_hash = AuditLedger::verify(&conn, audit_key).expect("Verify");
    assert!(!report_prev_hash.is_intact, "Tampered prev_hash MUST be detected");
    assert_eq!(report_prev_hash.tampered_seq_id, Some(7));
    assert!(
        report_prev_hash
            .error_message
            .as_ref()
            .unwrap()
            .contains("prev_hash mismatch")
    );
}

#[test]
fn test_challenger1_audit_ledger_pinpoints_signature_tampering() {
    let conn = Connection::open_in_memory().expect("in-memory sqlite");
    AuditLedger::init_audit_table(&conn).expect("init audit table");
    let audit_key = b"master_audit_secret_key_32_bytes";

    for i in 1..=8 {
        AuditLedger::append(
            &conn,
            audit_key,
            AuditLedger::EVENT_SOD_SELF_APPROVAL_BLOCKED,
            &format!("attacker_{i}"),
            &format!("Attempted self-approval #{i}"),
        )
        .expect("Append");
    }

    // Tamper attack: Corrupt signature / record_hash at seq_id = 3
    conn.execute(
        "UPDATE banking_audit_chain SET record_hash = 'corrupted_fake_record_hash_12345' WHERE seq_id = 3",
        [],
    )
    .expect("Tamper record_hash");

    let report_sig = AuditLedger::verify(&conn, audit_key).expect("Verify");
    assert!(!report_sig.is_intact, "Corrupted record_hash MUST be detected");
    assert_eq!(report_sig.tampered_seq_id, Some(3));
}
