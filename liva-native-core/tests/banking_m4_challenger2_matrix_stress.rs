//! Milestone 4 Challenger 2: Adversarial 75-Cell Matrix Boundary & Audit Ledger Tamper Detection Stress Suite
//!
//! Independently authored and executed by Milestone 4 Challenger 2 (challenger_m4_2).
//! Adversarially challenges:
//! 1. Exhaustive 75-Cell Role x Permission Matrix Boundary Verification:
//!    - Strictly enforces fail-closed Segregation of Duties (SoD) for all 5 roles across 15 permissions.
//!    - Verifies exact cell counts: 17 granted cells, 58 denied cells.
//! 2. Hostile Payloads and Malformed Input Injection:
//!    - SQL injection payloads in role, username, and user_id.
//!    - Role spoofing ("ROOT", "SUPERADMIN", "DBA", "OPERATOR", "SYSTEM", "GUEST", "null").
//!    - Unicode casing, accents, homoglyphs, and combining diacritics.
//!    - Empty strings, whitespace padding, tabs, and newlines.
//!    - Null tokens, embedded NUL bytes ("\0"), and non-string JSON data types.
//! 3. HMAC-SHA256 Audit Ledger Tamper Detection with Exact Seq ID Pinpointing:
//!    - Systematically tests deliberate tampering on:
//!      `payload_digest`, `record_hash`, `timestamp`, `prev_hash`, `signature`, `actor_principal`, `event_type`.
//!    - Verifies across Head (seq_id = 1), Middle (seq_id = 8), and Tail (seq_id = 15) positions.
//!    - Verifies row deletion attacks and wrong HMAC key rejection.
//! 4. End-to-End SoD Self-Approval Violation Automatic Logging & Forward Hash Chain Continuity.

use liva_native_core::banking::compliance::AuditLedger;
use liva_native_core::crypto::EncryptionEngine;
use liva_native_core::{
    AppState, CommandPrincipal, Permission, UserRole,
    check_command_role_permission, check_user_permission, get_role_permissions, db,
    handle_command_as, llm, stt, tts,
};
use rusqlite::{Connection, params};
use serde_json::json;
use std::sync::Arc;

const ADVERSARIAL_AUDIT_KEY: &[u8; 32] = b"m4_challenger2_adversarial_key32";

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

// ===========================================================================
// Test 1: Exhaustive 75-Cell Matrix Boundary Verification
// ===========================================================================

#[test]
fn test_m4_challenger2_exhaustive_75_cell_matrix_boundaries() {
    let all_roles = [
        UserRole::Maker,
        UserRole::Checker,
        UserRole::Cfo,
        UserRole::Auditor,
        UserRole::Admin,
    ];

    let all_permissions = [
        Permission::IngestStatement,
        Permission::RunReconciliation,
        Permission::ProposeHitlMatch,
        Permission::ApproveHitlMatch,
        Permission::ProposePaymentOrder,
        Permission::ApprovePaymentOrder,
        Permission::RejectPaymentOrder,
        Permission::ViewCashflow30_90,
        Permission::ApproveHighLimit,
        Permission::ViewAuditChain,
        Permission::VerifyMerkleProof,
        Permission::ExportDecree13Report,
        Permission::ManageSystemConfig,
        Permission::ManageUsers,
        Permission::ViewSystemDiagnostics,
    ];

    assert_eq!(all_roles.len(), 5);
    assert_eq!(all_permissions.len(), 15);

    let mut granted_count = 0;
    let mut denied_count = 0;

    for &role in &all_roles {
        let role_perms = get_role_permissions(role);
        for &perm in &all_permissions {
            let is_allowed = check_user_permission(role, perm);
            let in_list = role_perms.contains(&perm);
            assert_eq!(
                is_allowed, in_list,
                "Inconsistency between check_user_permission and get_role_permissions for {:?} / {:?}",
                role, perm
            );

            if is_allowed {
                granted_count += 1;
            } else {
                denied_count += 1;
            }

            // Explicit role-boundary invariant check
            match role {
                UserRole::Maker => {
                    let expected = matches!(
                        perm,
                        Permission::IngestStatement
                            | Permission::RunReconciliation
                            | Permission::ProposeHitlMatch
                            | Permission::ProposePaymentOrder
                    );
                    assert_eq!(is_allowed, expected, "Maker mismatch on {:?}", perm);
                }
                UserRole::Checker => {
                    let expected = matches!(
                        perm,
                        Permission::ApproveHitlMatch
                            | Permission::ApprovePaymentOrder
                            | Permission::RejectPaymentOrder
                    );
                    assert_eq!(is_allowed, expected, "Checker mismatch on {:?}", perm);
                }
                UserRole::Cfo => {
                    let expected = matches!(
                        perm,
                        Permission::ViewCashflow30_90
                            | Permission::ApproveHighLimit
                            | Permission::ApprovePaymentOrder
                            | Permission::RejectPaymentOrder
                    );
                    assert_eq!(is_allowed, expected, "CFO mismatch on {:?}", perm);
                }
                UserRole::Auditor => {
                    let expected = matches!(
                        perm,
                        Permission::ViewAuditChain
                            | Permission::VerifyMerkleProof
                            | Permission::ExportDecree13Report
                    );
                    assert_eq!(is_allowed, expected, "Auditor mismatch on {:?}", perm);
                }
                UserRole::Admin => {
                    let expected = matches!(
                        perm,
                        Permission::ManageSystemConfig
                            | Permission::ManageUsers
                            | Permission::ViewSystemDiagnostics
                    );
                    assert_eq!(is_allowed, expected, "Admin mismatch on {:?}", perm);
                }
            }
        }
    }

    assert_eq!(granted_count, 17, "Must have exactly 17 granted cells in 75-cell matrix");
    assert_eq!(denied_count, 58, "Must have exactly 58 denied cells in 75-cell matrix");
    assert_eq!(granted_count + denied_count, 75);

    // Also verify check_command_role_permission for key role-bound commands
    assert!(check_command_role_permission(UserRole::Maker, "statement_ingest_file").is_ok());
    assert!(check_command_role_permission(UserRole::Maker, "reconciliation_resolve_hitl").is_err());
    assert!(check_command_role_permission(UserRole::Checker, "reconciliation_resolve_hitl").is_ok());
    assert!(check_command_role_permission(UserRole::Checker, "statement_ingest_file").is_err());
    assert!(check_command_role_permission(UserRole::Cfo, "treasury:approve_high_limit").is_ok());
    assert!(check_command_role_permission(UserRole::Auditor, "compliance:get_audit_chain").is_ok());
    assert!(check_command_role_permission(UserRole::Auditor, "treasury:approve_payment").is_err());
    assert!(check_command_role_permission(UserRole::Admin, "update_config").is_ok());
    assert!(check_command_role_permission(UserRole::Admin, "banking_run_reconciliation").is_err());
}

// ===========================================================================
// Test 2: Hostile Payloads and Malformed Input Injection
// ===========================================================================

#[tokio::test]
async fn test_m4_challenger2_hostile_payloads_sqli_spoofing_unicode() {
    let state = challenger_test_state();

    // 1. Role spoofing with privileged / system titles
    let hostile_roles = [
        "ROOT",
        "SUPERADMIN",
        "SUPERUSER",
        "DBA",
        "OPERATOR",
        "SYSADMIN",
        "SYSTEM",
        "GUEST",
        "ANONYMOUS",
        "MAKER_ADMIN",
        "CHECKER_MAKER",
        "maker;checker",
        "maker,checker",
        "null",
        "undefined",
        "None",
    ];

    for fake_role in hostile_roles {
        let res = handle_command_as(
            CommandPrincipal::TauriDashboard,
            state.clone(),
            "auth:check_permission",
            json!({
                "role": fake_role,
                "permission": "IngestStatement"
            }),
            None,
            None,
        )
        .await
        .expect("graceful error returned");

        assert_eq!(res["success"], false, "Spoofed role '{}' must fail", fake_role);
        assert_eq!(res["allowed"], false);
        assert!(res["error"].is_string(), "Must contain error message");
    }

    // 2. Empty strings and whitespace variations
    let empty_roles = ["", " ", "   ", "\t", "\r\n", " \t \r\n "];
    for empty_str in empty_roles {
        let res = handle_command_as(
            CommandPrincipal::TauriDashboard,
            state.clone(),
            "auth:check_permission",
            json!({
                "role": empty_str,
                "permission": "IngestStatement"
            }),
            None,
            None,
        )
        .await
        .expect("graceful response");

        assert_eq!(res["success"], false);
        assert_eq!(res["allowed"], false);
    }

    // 3. Null byte injection attacks ("\0")
    let null_byte_payloads = [
        ("MAKER\0CHECKER", "IngestStatement"),
        ("MAKER\0", "IngestStatement"),
        ("\0ADMIN", "ManageSystemConfig"),
        ("MAKER", "IngestStatement\0"),
    ];

    for (role_str, perm_str) in null_byte_payloads {
        let res = handle_command_as(
            CommandPrincipal::TauriDashboard,
            state.clone(),
            "auth:check_permission",
            json!({
                "role": role_str,
                "permission": perm_str
            }),
            None,
            None,
        )
        .await
        .expect("handled safely");

        assert_eq!(res["success"], false, "Null byte payload must be rejected");
        assert_eq!(res["allowed"], false);
    }

    // 4. SQL Injection in role, username, and userId
    let sqli_payloads = [
        json!({"role": "MAKER' OR '1'='1", "permission": "IngestStatement"}),
        json!({"role": "ADMIN'; DROP TABLE banking_audit_chain; --", "permission": "ManageSystemConfig"}),
        json!({"username": "maker_nam' OR 1=1 --", "permission": "IngestStatement"}),
        json!({"username": "admin' UNION SELECT * FROM users --", "permission": "ManageUsers"}),
        json!({"userId": "'; DELETE FROM reconciliation_matches; --", "permission": "ApproveHitlMatch"}),
    ];

    for sqli in sqli_payloads {
        let res = handle_command_as(
            CommandPrincipal::TauriDashboard,
            state.clone(),
            "auth:check_permission",
            sqli.clone(),
            None,
            None,
        )
        .await
        .expect("handled safely");

        assert_eq!(res["success"], false, "SQLi attack must fail: {:?}", sqli);
        assert_eq!(res["allowed"], false);
    }

    // 5. Unicode accents, homoglyphs, and combining characters
    let unicode_roles = [
        "mákèr",
        "chèckèr",
        "cfò",
        "àudìtòr",
        "àdmìn",
        "ＭＡＫＥＲ", // Fullwidth
        "ＣＨＥＣＫＥＲ",
        "M\u{0300}aker", // Combining grave accent
    ];

    for u_role in unicode_roles {
        let res = handle_command_as(
            CommandPrincipal::TauriDashboard,
            state.clone(),
            "auth:check_permission",
            json!({
                "role": u_role,
                "permission": "IngestStatement"
            }),
            None,
            None,
        )
        .await
        .expect("handled safely");

        // Unicode distorted roles must not accidentally resolve to valid roles
        assert_eq!(res["success"], false, "Unicode distorted role '{}' must not resolve", u_role);
        assert_eq!(res["allowed"], false);
    }

    // 6. Non-string JSON data types
    let bad_type_payloads = [
        json!({"role": 12345, "permission": "IngestStatement"}),
        json!({"role": true, "permission": "IngestStatement"}),
        json!({"role": ["MAKER"], "permission": "IngestStatement"}),
        json!({"role": {"name": "MAKER"}, "permission": "IngestStatement"}),
        json!({"role": null, "permission": "IngestStatement"}),
    ];

    for bad_type in bad_type_payloads {
        let res = handle_command_as(
            CommandPrincipal::TauriDashboard,
            state.clone(),
            "auth:check_permission",
            bad_type.clone(),
            None,
            None,
        )
        .await
        .expect("handled safely");

        assert_eq!(res["success"], false, "Malformed type must fail: {:?}", bad_type);
        assert_eq!(res["allowed"], false);
    }
}

// ===========================================================================
// Test 3: Audit Ledger Tamper Detection Stress with Exact Seq ID Pinpointing
// ===========================================================================

#[test]
fn test_m4_challenger2_audit_ledger_multi_field_tamper_pinpointing() {
    let conn = Connection::open_in_memory().expect("in-memory test db");
    AuditLedger::init_audit_table(&conn).expect("init audit table");
    let key = ADVERSARIAL_AUDIT_KEY;

    // Build a forward hash-chained audit ledger with 15 records
    let audit_events = [
        (AuditLedger::EVENT_SYSTEM_INITIALIZED, "kernel", "Bootstrapping LIVA banking kernel"),
        (AuditLedger::EVENT_STATEMENT_IMPORTED, "maker_nam", "VCB_2026_09_16_001.xlsx"),
        (AuditLedger::EVENT_MATCH_AUTO, "reconciliation_engine", "1:1 exact match for batch 001"),
        (AuditLedger::EVENT_ROLE_PERMISSION_DENIED, "maker_nam", "Blocked attempt to approve payment"),
        (AuditLedger::EVENT_SOD_SELF_APPROVAL_BLOCKED, "maker_nam", "Self-approval blocked on HITL-01"),
        (AuditLedger::EVENT_HITL_CONFIRMED, "checker_tri", "Approved HITL-01 composite split"),
        (AuditLedger::EVENT_STATEMENT_IMPORTED, "maker_nam", "TCB_2026_09_16_002.csv"),
        (AuditLedger::EVENT_MATCH_AUTO, "reconciliation_engine", "Fuzzy match for batch 002"),
        (AuditLedger::EVENT_UNAUTHORIZED_COMMAND_ATTEMPT, "auditor_lan", "Blocked mutation command attempt"),
        (AuditLedger::EVENT_PAYMENT_ORDER_APPROVAL, "checker_tri", "Approved payment order PO-101"),
        (AuditLedger::EVENT_SOD_SELF_APPROVAL_BLOCKED, "maker_nam", "Self-approval blocked on PO-102"),
        (AuditLedger::EVENT_PAYMENT_ORDER_REJECTION, "cfo_huong", "Rejected payment order PO-103 exceeding budget"),
        (AuditLedger::EVENT_STATEMENT_IMPORTED, "maker_nam", "BIDV_2026_09_16_003.xlsx"),
        (AuditLedger::EVENT_MATCH_AUTO, "reconciliation_engine", "1:1 exact match for batch 003"),
        (AuditLedger::EVENT_PAYMENT_ORDER_APPROVAL, "cfo_huong", "Approved high limit payment order PO-104"),
    ];

    for (ev_type, actor, payload) in &audit_events {
        AuditLedger::append(&conn, key, ev_type, actor, payload).expect("append audit record");
    }

    // 1. Untampered baseline check
    let baseline = AuditLedger::verify(&conn, key).expect("verify baseline");
    assert!(baseline.is_intact, "Baseline audit chain must be 100% cryptographically intact");
    assert_eq!(baseline.total_records, 15);
    assert_eq!(baseline.tampered_seq_id, None);
    assert_eq!(baseline.error_message, None);

    // Helper macro to run tamper-check-restore cycle
    let test_tamper = |target_seq: i64, field: &str, tampered_val: &str| {
        let is_timestamp = field == "timestamp";

        // Save original value as TEXT
        let orig_text: String = conn
            .query_row(
                &format!("SELECT CAST({field} AS TEXT) FROM banking_audit_chain WHERE seq_id = ?1"),
                params![target_seq],
                |r| r.get(0),
            )
            .expect("read original value");

        // Apply tamper
        if is_timestamp {
            let ts_int: i64 = tampered_val.parse().unwrap();
            conn.execute(
                &format!("UPDATE banking_audit_chain SET {field} = ?1 WHERE seq_id = ?2"),
                params![ts_int, target_seq],
            )
            .expect("apply timestamp tamper");
        } else {
            conn.execute(
                &format!("UPDATE banking_audit_chain SET {field} = ?1 WHERE seq_id = ?2"),
                params![tampered_val, target_seq],
            )
            .expect("apply text tamper");
        }

        // Verify tampering is detected and pinpointed
        let report = AuditLedger::verify(&conn, key).expect("verify tampered chain");
        assert!(
            !report.is_intact,
            "Chain must NOT be intact when '{field}' is altered at seq_id {target_seq}"
        );
        assert_eq!(
            report.tampered_seq_id,
            Some(target_seq),
            "Must pinpoint exact tampered seq_id {target_seq} for field '{field}'"
        );

        // Restore original value
        if is_timestamp {
            let orig_int: i64 = orig_text.parse().unwrap();
            conn.execute(
                &format!("UPDATE banking_audit_chain SET {field} = ?1 WHERE seq_id = ?2"),
                params![orig_int, target_seq],
            )
            .expect("restore timestamp");
        } else {
            conn.execute(
                &format!("UPDATE banking_audit_chain SET {field} = ?1 WHERE seq_id = ?2"),
                params![orig_text, target_seq],
            )
            .expect("restore text");
        }

        // Confirm chain is intact again
        let restored = AuditLedger::verify(&conn, key).expect("verify restored chain");
        assert!(
            restored.is_intact,
            "Chain must be restored to intact status after restoring '{field}' at seq_id {target_seq}"
        );
    };

    // 2. Systematic Tampering at Head (seq_id = 1)
    test_tamper(1, "payload_digest", "tampered_genesis_digest_1111");
    test_tamper(1, "record_hash", "tampered_genesis_hash_2222");
    test_tamper(1, "timestamp", "1234567890");
    test_tamper(1, "signature", "tampered_genesis_sig_3333");
    test_tamper(1, "actor_principal", "malicious_injected_actor");
    test_tamper(1, "event_type", "FORGED_EVENT_TYPE");
    test_tamper(1, "prev_hash", "tampered_genesis_prev_hash");

    // 3. Systematic Tampering at Middle (seq_id = 8)
    test_tamper(8, "payload_digest", "tampered_middle_digest_4444");
    test_tamper(8, "record_hash", "tampered_middle_hash_5555");
    test_tamper(8, "timestamp", "1726999999");
    test_tamper(8, "signature", "tampered_middle_sig_6666");
    test_tamper(8, "actor_principal", "hacker_middle_actor");
    test_tamper(8, "event_type", "FORGED_MIDDLE_EVENT");
    test_tamper(8, "prev_hash", "tampered_middle_prev_hash");

    // 4. Systematic Tampering at Tail (seq_id = 15)
    test_tamper(15, "payload_digest", "tampered_tail_digest_7777");
    test_tamper(15, "record_hash", "tampered_tail_hash_8888");
    test_tamper(15, "timestamp", "1726888888");
    test_tamper(15, "signature", "tampered_tail_sig_9999");
    test_tamper(15, "actor_principal", "hacker_tail_actor");
    test_tamper(15, "event_type", "FORGED_TAIL_EVENT");
    test_tamper(15, "prev_hash", "tampered_tail_prev_hash");

    // 5. Row Deletion Attack: Delete record at seq_id = 7
    // Record 8's prev_hash points to record 7's record_hash.
    // Deleting record 7 causes record 8 to read record 6 as predecessor -> broken chain at seq_id = 8!
    let r7_prev_hash: String = conn.query_row("SELECT prev_hash FROM banking_audit_chain WHERE seq_id = 7", [], |r| r.get(0)).unwrap();
    let r7_rec_hash: String = conn.query_row("SELECT record_hash FROM banking_audit_chain WHERE seq_id = 7", [], |r| r.get(0)).unwrap();
    let r7_ts: i64 = conn.query_row("SELECT timestamp FROM banking_audit_chain WHERE seq_id = 7", [], |r| r.get(0)).unwrap();
    let r7_ev: String = conn.query_row("SELECT event_type FROM banking_audit_chain WHERE seq_id = 7", [], |r| r.get(0)).unwrap();
    let r7_actor: String = conn.query_row("SELECT actor_principal FROM banking_audit_chain WHERE seq_id = 7", [], |r| r.get(0)).unwrap();
    let r7_pdigest: String = conn.query_row("SELECT payload_digest FROM banking_audit_chain WHERE seq_id = 7", [], |r| r.get(0)).unwrap();
    let r7_sig: String = conn.query_row("SELECT signature FROM banking_audit_chain WHERE seq_id = 7", [], |r| r.get(0)).unwrap();

    conn.execute("DELETE FROM banking_audit_chain WHERE seq_id = 7", []).expect("delete row 7");
    let del_report = AuditLedger::verify(&conn, key).expect("verify after deletion");
    assert!(!del_report.is_intact, "Row deletion must break forward hash chain");
    assert_eq!(
        del_report.tampered_seq_id,
        Some(8),
        "Row 8 must report broken prev_hash link when Row 7 is deleted"
    );

    // Restore row 7
    conn.execute(
        "INSERT INTO banking_audit_chain (seq_id, prev_hash, record_hash, timestamp, event_type, actor_principal, payload_digest, signature) \
         VALUES (7, ?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![r7_prev_hash, r7_rec_hash, r7_ts, r7_ev, r7_actor, r7_pdigest, r7_sig],
    ).expect("restore row 7");

    let final_check = AuditLedger::verify(&conn, key).expect("verify after row restore");
    assert!(final_check.is_intact, "Chain must be completely intact after restoring row 7");
    assert_eq!(final_check.total_records, 15);
}

// ===========================================================================
// Test 4: Key Invalidation Stress
// ===========================================================================

#[test]
fn test_m4_challenger2_wrong_hmac_key_rejection() {
    let conn = Connection::open_in_memory().expect("in-memory db");
    AuditLedger::init_audit_table(&conn).expect("init audit table");
    let valid_key = ADVERSARIAL_AUDIT_KEY;
    let wrong_key = b"attacker_wrong_hmac_secret_key32";

    AuditLedger::append(&conn, valid_key, AuditLedger::EVENT_SYSTEM_INITIALIZED, "system", "Boot").unwrap();
    AuditLedger::append(&conn, valid_key, AuditLedger::EVENT_STATEMENT_IMPORTED, "maker_nam", "vcb.xlsx").unwrap();

    // Verification with wrong key must fail at seq_id = 1
    let report = AuditLedger::verify(&conn, wrong_key).expect("verify with wrong key");
    assert!(!report.is_intact, "Verification with incorrect key must fail");
    assert_eq!(report.tampered_seq_id, Some(1));
    assert!(
        report.error_message.as_ref().unwrap().contains("Hash recalculation mismatch"),
        "Must cite HMAC recalculation mismatch"
    );
}

// ===========================================================================
// Test 5: End-to-End SoD Self-Approval Automatic Logging & Chain Continuity
// ===========================================================================

#[tokio::test]
async fn test_m4_challenger2_e2e_sod_self_approval_audit_trail_continuity() {
    let state = challenger_test_state();

    // 1. Seed match into reconciliation_matches
    {
        let conn = state.db.writer.get().expect("db writer lock");
        conn.execute(
            "INSERT INTO reconciliation_matches (
                id, bank_tx_id, ledger_entry_ids_json, match_type, confidence_score,
                matched_amount, discrepancy_amount, status, matched_by, matched_at,
                notes, hitl_token
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                "match_m4_stress_01",
                "bank_tx_m4_01",
                "[\"gl_tx_m4_01\"]",
                "1:1_EXACT",
                1.0,
                200_000_000,
                0,
                "PENDING",
                "USER_PROPOSAL",
                1726000000,
                "M4 Challenger 2 E2E Stress Match",
                "token_m4_challenger2_secret_key"
            ],
        )
        .expect("seed match");
    }

    // 2. Perform 5 consecutive blocked self-approval attacks with diverse casing & whitespace
    let attack_pairs = [
        ("maker_nam", "maker_nam"),
        ("maker_nam", "MAKER_NAM"),
        ("maker_nam", "  Maker_Nam  "),
        ("maker_nam", "\tmaker_nam\n"),
        ("maker_nam", "   mAkEr_NaM   "),
    ];

    for (maker, checker) in attack_pairs {
        let res = handle_command_as(
            CommandPrincipal::TauriDashboard,
            state.clone(),
            "banking:resolve_hitl",
            json!({
                "match_id": "match_m4_stress_01",
                "hitl_token": "token_m4_challenger2_secret_key",
                "decision": "APPROVE",
                "maker_id": maker,
                "checker_id": checker,
                "notes": "Adversarial self-approval stress test"
            }),
            None,
            None,
        )
        .await;

        assert!(res.is_err(), "Self-approval must strictly fail");
        let err_msg = res.unwrap_err();
        assert!(
            err_msg.contains("Circular 09/2020/TT-NHNN violation: Maker cannot self-approve as Checker"),
            "Must cite Circular 09 violation"
        );
    }

    // 3. Verify that all 5 violations are logged to banking_audit_chain
    let conn = state.db.writer.get().expect("db lock");
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM banking_audit_chain WHERE event_type = 'SOD_SELF_APPROVAL_BLOCKED'",
            [],
            |r| r.get(0),
        )
        .expect("query audit count");
    assert!(count >= 5, "At least 5 SOD_SELF_APPROVAL_BLOCKED events must be logged");

    // 4. Verify forward hash-chain continuity using the derived audit key
    let audit_key = liva_native_core::banking::compliance::get_audit_key(&state);
    let report = AuditLedger::verify(&conn, &audit_key).expect("verify audit chain");
    assert!(report.is_intact, "Audit ledger must remain 100% cryptographically intact after 5 blocked attacks");
    assert!(report.tampered_seq_id.is_none());
}
