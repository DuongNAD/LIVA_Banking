//! Milestone 3 Challenger 2: Adversarial RBAC & Command Authorization Challenge Suite
//!
//! Independently authored and executed by Milestone 3 Challenger 2 (challenger_m3_2).
//! Adversarially challenges:
//! 1. Exhaustive 5-Role x 15-Permission Matrix (75 discrete cells):
//!    - Strictly enforces fail-closed Segregation of Duties (SoD).
//! 2. Adversarial Role Permutations:
//!    - Maker trying to approve HITL or payment order -> MUST return `allowed: false` / `Err`.
//!    - Checker trying to ingest statements or propose payment -> MUST return `allowed: false` / `Err`.
//!    - Auditor trying to execute any financial mutation -> MUST return `allowed: false` / `Err`.
//!    - Admin trying to execute financial transactions -> MUST return `allowed: false` / `Err`.
//!    - Standard Checker trying to approve high limit (> 5B VND) -> MUST return `allowed: false` / `Err`.
//! 3. IPC Dispatch Verification:
//!    - Verify `auth:check_permission` and `auth_check_permission` through `handle_command_as`.
//!    - Verify structured JSON responses (success, allowed, role, permission/command, reason).
//!    - Verify capability discovery mode, username DB lookup, and user_id DB lookup.
//! 4. Hostile Payloads & Boundary Defense:
//!    - Role spoofing ("ROOT", "SUPERADMIN", "HACKER", null, empty).
//!    - SQL injection in username / user_id lookup.
//!    - Loose permission string parsing (snake_case, camelCase, kebab-case, whitespace).
//! 5. Principal Isolation:
//!    - Unauthorized principals (TauriSetup, WebSocketRemote, Telegram) strictly fail-closed.

use liva_native_core::crypto::EncryptionEngine;
use liva_native_core::{
    AppState, CommandPrincipal, Permission, UserRole, authorize_command,
    check_command_role_permission, check_user_permission, get_role_permissions, db,
    handle_command_as, llm, stt, tts,
};
use serde_json::json;
use std::sync::Arc;

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
// Test 1: Exhaustive 75-Cell Role x Permission Matrix Challenge
// ===========================================================================

#[test]
fn test_adversarial_exhaustive_75_role_permission_permutations() {
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

    let mut true_count = 0;
    let mut false_count = 0;

    for &role in &all_roles {
        let granted = get_role_permissions(role);
        for &perm in &all_permissions {
            let allowed = check_user_permission(role, perm);
            let in_granted = granted.contains(&perm);
            assert_eq!(
                allowed, in_granted,
                "Inconsistency for role {:?} on permission {:?}",
                role, perm
            );

            if allowed {
                true_count += 1;
            } else {
                false_count += 1;
            }

            // Invariant assertions per role:
            match role {
                UserRole::Maker => {
                    let expected = matches!(
                        perm,
                        Permission::IngestStatement
                            | Permission::RunReconciliation
                            | Permission::ProposeHitlMatch
                            | Permission::ProposePaymentOrder
                    );
                    assert_eq!(allowed, expected, "Maker permission mismatch for {:?}", perm);
                }
                UserRole::Checker => {
                    let expected = matches!(
                        perm,
                        Permission::ApproveHitlMatch
                            | Permission::ApprovePaymentOrder
                            | Permission::RejectPaymentOrder
                    );
                    assert_eq!(allowed, expected, "Checker permission mismatch for {:?}", perm);
                }
                UserRole::Cfo => {
                    let expected = matches!(
                        perm,
                        Permission::ViewCashflow30_90
                            | Permission::ApproveHighLimit
                            | Permission::ApprovePaymentOrder
                            | Permission::RejectPaymentOrder
                    );
                    assert_eq!(allowed, expected, "CFO permission mismatch for {:?}", perm);
                }
                UserRole::Auditor => {
                    let expected = matches!(
                        perm,
                        Permission::ViewAuditChain
                            | Permission::VerifyMerkleProof
                            | Permission::ExportDecree13Report
                    );
                    assert_eq!(allowed, expected, "Auditor permission mismatch for {:?}", perm);
                }
                UserRole::Admin => {
                    let expected = matches!(
                        perm,
                        Permission::ManageSystemConfig
                            | Permission::ManageUsers
                            | Permission::ViewSystemDiagnostics
                    );
                    assert_eq!(allowed, expected, "Admin permission mismatch for {:?}", perm);
                }
            }
        }
    }

    // Expected matrix breakdown:
    // Maker: 4 True, 11 False
    // Checker: 3 True, 12 False
    // CFO: 4 True, 11 False
    // Auditor: 3 True, 12 False
    // Admin: 3 True, 12 False
    // Total True: 17, Total False: 58 (Sum = 75)
    assert_eq!(true_count, 17, "Exactly 17 granted permission cells");
    assert_eq!(false_count, 58, "Exactly 58 denied permission cells");
    assert_eq!(true_count + false_count, 75);
}

// ===========================================================================
// Test 2: Adversarial Role Permutations (Specific Hostile Invariant Scenarios)
// ===========================================================================

#[test]
fn test_adversarial_maker_hostile_violations() {
    // Scenario 1A: Maker trying to approve HITL match -> MUST BE DENIED
    assert!(
        !check_user_permission(UserRole::Maker, Permission::ApproveHitlMatch),
        "Maker must NOT have ApproveHitlMatch permission"
    );
    assert!(
        check_command_role_permission(UserRole::Maker, "reconciliation_resolve_hitl").is_err(),
        "Maker must be blocked from reconciliation_resolve_hitl"
    );
    assert!(
        check_command_role_permission(UserRole::Maker, "banking:resolve_hitl").is_err(),
        "Maker must be blocked from banking:resolve_hitl"
    );

    // Scenario 1B: Maker trying to approve payment order -> MUST BE DENIED
    assert!(
        !check_user_permission(UserRole::Maker, Permission::ApprovePaymentOrder),
        "Maker must NOT have ApprovePaymentOrder permission"
    );
    assert!(
        check_command_role_permission(UserRole::Maker, "treasury:approve_payment").is_err(),
        "Maker must be blocked from treasury:approve_payment"
    );
    assert!(
        check_command_role_permission(UserRole::Maker, "payment:approve").is_err(),
        "Maker must be blocked from payment:approve"
    );
    assert!(
        check_command_role_permission(UserRole::Maker, "treasury_approve_payment").is_err(),
        "Maker must be blocked from treasury_approve_payment"
    );

    // Scenario 1C: Maker trying to reject payment order -> MUST BE DENIED
    assert!(
        !check_user_permission(UserRole::Maker, Permission::RejectPaymentOrder),
        "Maker must NOT have RejectPaymentOrder permission"
    );
    assert!(
        check_command_role_permission(UserRole::Maker, "treasury:reject_payment").is_err(),
        "Maker must be blocked from treasury:reject_payment"
    );
}

#[test]
fn test_adversarial_checker_hostile_violations() {
    // Scenario 2A: Checker trying to ingest statements -> MUST BE DENIED
    assert!(
        !check_user_permission(UserRole::Checker, Permission::IngestStatement),
        "Checker must NOT have IngestStatement permission"
    );
    assert!(
        check_command_role_permission(UserRole::Checker, "statement_ingest_file").is_err(),
        "Checker must be blocked from statement_ingest_file"
    );
    assert!(
        check_command_role_permission(UserRole::Checker, "statement:ingest_file").is_err(),
        "Checker must be blocked from statement:ingest_file"
    );
    assert!(
        check_command_role_permission(UserRole::Checker, "banking:import_statement").is_err(),
        "Checker must be blocked from banking:import_statement"
    );

    // Scenario 2B: Checker trying to run reconciliation -> MUST BE DENIED
    assert!(
        !check_user_permission(UserRole::Checker, Permission::RunReconciliation),
        "Checker must NOT have RunReconciliation permission"
    );
    assert!(
        check_command_role_permission(UserRole::Checker, "banking_run_reconciliation").is_err(),
        "Checker must be blocked from banking_run_reconciliation"
    );
    assert!(
        check_command_role_permission(UserRole::Checker, "banking:run_reconciliation").is_err(),
        "Checker must be blocked from banking:run_reconciliation"
    );

    // Scenario 2C: Checker trying to propose payment -> MUST BE DENIED
    assert!(
        !check_user_permission(UserRole::Checker, Permission::ProposePaymentOrder),
        "Checker must NOT have ProposePaymentOrder permission"
    );
    assert!(
        check_command_role_permission(UserRole::Checker, "treasury:propose_payment").is_err(),
        "Checker must be blocked from treasury:propose_payment"
    );
    assert!(
        check_command_role_permission(UserRole::Checker, "payment:propose").is_err(),
        "Checker must be blocked from payment:propose"
    );
    assert!(
        check_command_role_permission(UserRole::Checker, "treasury_propose_payment").is_err(),
        "Checker must be blocked from treasury_propose_payment"
    );

    // Scenario 2D: Standard Checker trying to approve high limit (> 5B VND) -> MUST BE DENIED
    assert!(
        !check_user_permission(UserRole::Checker, Permission::ApproveHighLimit),
        "Standard Checker must NOT have ApproveHighLimit permission"
    );
    assert!(
        check_command_role_permission(UserRole::Checker, "treasury:approve_high_limit").is_err(),
        "Checker must be blocked from treasury:approve_high_limit"
    );
    assert!(
        check_command_role_permission(UserRole::Checker, "treasury_approve_high_limit").is_err(),
        "Checker must be blocked from treasury_approve_high_limit"
    );
}

#[test]
fn test_adversarial_auditor_hostile_mutations() {
    // Scenario 3: Auditor trying to execute ANY financial mutation -> MUST BE DENIED
    let mutation_commands = [
        "statement_ingest_file",
        "statement:ingest_file",
        "banking:import_statement",
        "banking_run_reconciliation",
        "banking:run_reconciliation",
        "reconciliation_resolve_hitl",
        "banking:resolve_hitl",
        "treasury:propose_payment",
        "payment:propose",
        "treasury_propose_payment",
        "treasury:approve_payment",
        "payment:approve",
        "treasury_approve_payment",
        "treasury:reject_payment",
        "payment:reject",
        "treasury_reject_payment",
        "treasury:approve_high_limit",
        "treasury_approve_high_limit",
        "update_config",
        "config:update",
        "auth:create_user",
    ];

    for cmd in mutation_commands {
        assert!(
            check_command_role_permission(UserRole::Auditor, cmd).is_err(),
            "Auditor must be strictly blocked from mutation command: {}",
            cmd
        );
    }

    // Auditor is strictly read-only for compliance/audit
    assert!(check_command_role_permission(UserRole::Auditor, "compliance:get_audit_chain").is_ok());
    assert!(check_command_role_permission(UserRole::Auditor, "compliance:verify_merkle").is_ok());
    assert!(check_command_role_permission(UserRole::Auditor, "compliance:export_decree13").is_ok());
    assert!(check_command_role_permission(UserRole::Auditor, "banking_get_compliance_status").is_ok());
}

#[test]
fn test_adversarial_admin_financial_isolation() {
    // Scenario 4: Admin trying to execute ANY financial transactions -> MUST BE DENIED
    let financial_commands = [
        "statement_ingest_file",
        "statement:ingest_file",
        "banking:import_statement",
        "banking_run_reconciliation",
        "banking:run_reconciliation",
        "reconciliation_resolve_hitl",
        "banking:resolve_hitl",
        "treasury:propose_payment",
        "payment:propose",
        "treasury_propose_payment",
        "treasury:approve_payment",
        "payment:approve",
        "treasury_approve_payment",
        "treasury:reject_payment",
        "payment:reject",
        "treasury_reject_payment",
        "treasury:approve_high_limit",
        "treasury_approve_high_limit",
    ];

    for cmd in financial_commands {
        assert!(
            check_command_role_permission(UserRole::Admin, cmd).is_err(),
            "Admin must be strictly prohibited from financial command: {}",
            cmd
        );
    }

    // Admin CAN manage system configuration and users
    assert!(check_command_role_permission(UserRole::Admin, "update_config").is_ok());
    assert!(check_command_role_permission(UserRole::Admin, "auth:create_user").is_ok());
    assert!(check_command_role_permission(UserRole::Admin, "get_preflight_status").is_ok());
    assert!(check_command_role_permission(UserRole::Admin, "get_system_status").is_ok());
}

#[test]
fn test_adversarial_cfo_high_limit_privilege() {
    // Scenario 5: High Limit (> 5B VND) is CFO privilege ONLY
    assert!(check_user_permission(UserRole::Cfo, Permission::ApproveHighLimit));
    assert!(check_command_role_permission(UserRole::Cfo, "treasury:approve_high_limit").is_ok());
    assert!(check_command_role_permission(UserRole::Cfo, "treasury_approve_high_limit").is_ok());

    // Denied for all other 4 roles
    for role in [UserRole::Maker, UserRole::Checker, UserRole::Auditor, UserRole::Admin] {
        assert!(
            !check_user_permission(role, Permission::ApproveHighLimit),
            "Role {:?} must NOT have ApproveHighLimit",
            role
        );
        assert!(
            check_command_role_permission(role, "treasury:approve_high_limit").is_err(),
            "Role {:?} must be blocked from treasury:approve_high_limit",
            role
        );
    }
}

// ===========================================================================
// Test 3: IPC Dispatch Verification (`auth:check_permission` & `auth_check_permission`)
// ===========================================================================

#[tokio::test]
async fn test_ipc_dispatch_verbs_parity_and_structured_json() {
    let state = challenger_test_state();

    // Verify both verbs "auth:check_permission" and "auth_check_permission" return identical structured JSON
    for cmd_verb in ["auth:check_permission", "auth_check_permission"] {
        // 1. Valid permission check
        let res_ok = handle_command_as(
            CommandPrincipal::TauriDashboard,
            state.clone(),
            cmd_verb,
            json!({
                "role": "MAKER",
                "permission": "ProposePaymentOrder"
            }),
            None,
            None,
        )
        .await
        .expect("IPC command invocation succeeded");

        assert_eq!(res_ok["success"], true);
        assert_eq!(res_ok["allowed"], true);
        assert_eq!(res_ok["role"], "MAKER");
        assert_eq!(res_ok["permission"], "ProposePaymentOrder");
        assert!(res_ok["reason"].is_null(), "reason must be null when allowed");

        // 2. Denied permission check (SoD violation)
        let res_denied = handle_command_as(
            CommandPrincipal::TauriDashboard,
            state.clone(),
            cmd_verb,
            json!({
                "role": "MAKER",
                "permission": "ApprovePaymentOrder"
            }),
            None,
            None,
        )
        .await
        .expect("IPC command invocation succeeded");

        assert_eq!(res_denied["success"], true);
        assert_eq!(res_denied["allowed"], false);
        assert_eq!(res_denied["role"], "MAKER");
        assert_eq!(res_denied["permission"], "ApprovePaymentOrder");
        assert!(
            res_denied["reason"]
                .as_str()
                .expect("reason string exists")
                .contains("lacks permission"),
            "Reason must explain permission lack"
        );

        // 3. Command authorization check
        let res_cmd_denied = handle_command_as(
            CommandPrincipal::TauriDashboard,
            state.clone(),
            cmd_verb,
            json!({
                "role": "CHECKER",
                "command": "statement_ingest_file"
            }),
            None,
            None,
        )
        .await
        .expect("IPC command invocation succeeded");

        assert_eq!(res_cmd_denied["success"], true);
        assert_eq!(res_cmd_denied["allowed"], false);
        assert_eq!(res_cmd_cmd_denied_role(&res_cmd_denied), "CHECKER");
        assert_eq!(res_cmd_denied["command"], "statement_ingest_file");
        assert!(res_cmd_denied["reason"].is_string());
    }
}

fn res_cmd_cmd_denied_role(val: &serde_json::Value) -> &str {
    val["role"].as_str().unwrap()
}

#[tokio::test]
async fn test_ipc_dispatch_database_identity_resolution() {
    let state = challenger_test_state();

    // 1. By Username: "maker_nam" (seeded in in-memory DB as MAKER)
    let res_user = handle_command_as(
        CommandPrincipal::TauriDashboard,
        state.clone(),
        "auth:check_permission",
        json!({
            "username": "maker_nam",
            "permission": "IngestStatement"
        }),
        None,
        None,
    )
    .await
    .expect("command ok");

    assert_eq!(res_user["success"], true);
    assert_eq!(res_user["allowed"], true);
    assert_eq!(res_user["role"], "MAKER");

    // 2. By Username: "checker_tri" (seeded as CHECKER)
    let res_checker = handle_command_as(
        CommandPrincipal::TauriDashboard,
        state.clone(),
        "auth:check_permission",
        json!({
            "username": "checker_tri",
            "permission": "ApproveHitlMatch"
        }),
        None,
        None,
    )
    .await
    .expect("command ok");

    assert_eq!(res_checker["success"], true);
    assert_eq!(res_checker["allowed"], true);
    assert_eq!(res_checker["role"], "CHECKER");

    // 3. By User ID: fetch checker_tri's UUID and query via userId
    let quick_res = handle_command_as(
        CommandPrincipal::TauriDashboard,
        state.clone(),
        "auth:get_quick_accounts",
        json!({}),
        None,
        None,
    )
    .await
    .expect("quick accounts ok");

    let accounts = quick_res["accounts"].as_array().expect("accounts array");
    let checker_acct = accounts
        .iter()
        .find(|a| a["username"] == "checker_tri")
        .expect("found checker_tri");
    let checker_id = checker_acct["id"].as_str().expect("checker user id");

    let res_by_id = handle_command_as(
        CommandPrincipal::TauriDashboard,
        state.clone(),
        "auth:check_permission",
        json!({
            "userId": checker_id,
            "permission": "IngestStatement" // Checker should NOT be allowed to ingest
        }),
        None,
        None,
    )
    .await
    .expect("command ok");

    assert_eq!(res_by_id["success"], true);
    assert_eq!(res_by_id["allowed"], false);
    assert_eq!(res_by_id["role"], "CHECKER");
}

#[tokio::test]
async fn test_ipc_dispatch_capability_discovery_mode() {
    let state = challenger_test_state();

    for &(role_str, expected_count) in &[
        ("MAKER", 4),
        ("CHECKER", 3),
        ("CFO", 4),
        ("AUDITOR", 3),
        ("ADMIN", 3),
    ] {
        let res = handle_command_as(
            CommandPrincipal::TauriDashboard,
            state.clone(),
            "auth:check_permission",
            json!({
                "role": role_str
            }),
            None,
            None,
        )
        .await
        .expect("discovery mode succeeded");

        assert_eq!(res["success"], true);
        assert_eq!(res["allowed"], true);
        assert_eq!(res["role"], role_str);

        let perms = res["permissions"].as_array().expect("permissions array");
        assert_eq!(
            perms.len(),
            expected_count,
            "Role {} must have exactly {} permissions in discovery",
            role_str,
            expected_count
        );
    }
}

// ===========================================================================
// Test 4: Hostile Payloads, Injection & Boundary Defense
// ===========================================================================

#[tokio::test]
async fn test_adversarial_malformed_and_hostile_payloads() {
    let state = challenger_test_state();

    // A. Role spoofing attacks -> Must be rejected with success: false, allowed: false
    let spoof_roles = ["ROOT", "SUPERADMIN", "SYSTEM", "HACKER", "maker_admin", "", "  ", "null"];
    for fake_role in spoof_roles {
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

        assert_eq!(res["success"], false, "Fake role '{}' must fail", fake_role);
        assert_eq!(res["allowed"], false);
        assert!(res["error"].is_string());
    }

    // B. Missing identity parameter (neither role, username, nor user_id)
    let res_empty = handle_command_as(
        CommandPrincipal::TauriDashboard,
        state.clone(),
        "auth:check_permission",
        json!({
            "permission": "IngestStatement"
        }),
        None,
        None,
    )
    .await
    .expect("graceful response");

    assert_eq!(res_empty["success"], false);
    assert_eq!(res_empty["allowed"], false);
    assert!(res_empty["error"].as_str().unwrap().contains("Missing"));

    // C. Non-existent username
    let res_no_user = handle_command_as(
        CommandPrincipal::TauriDashboard,
        state.clone(),
        "auth:check_permission",
        json!({
            "username": "ghost_attacker_99",
            "permission": "IngestStatement"
        }),
        None,
        None,
    )
    .await
    .expect("graceful response");

    assert_eq!(res_no_user["success"], false);
    assert_eq!(res_no_user["allowed"], false);
    assert!(res_no_user["error"].as_str().unwrap().contains("not found"));

    // D. SQL Injection payload in username
    let res_sqli = handle_command_as(
        CommandPrincipal::TauriDashboard,
        state.clone(),
        "auth:check_permission",
        json!({
            "username": "admin' OR '1'='1",
            "permission": "IngestStatement"
        }),
        None,
        None,
    )
    .await
    .expect("graceful response");

    assert_eq!(res_sqli["success"], false);
    assert_eq!(res_sqli["allowed"], false);
    assert!(res_sqli["error"].as_str().unwrap().contains("not found"));

    // E. Invalid permission string
    let res_bad_perm = handle_command_as(
        CommandPrincipal::TauriDashboard,
        state.clone(),
        "auth:check_permission",
        json!({
            "role": "MAKER",
            "permission": "ArbitraryNonExistentPermission"
        }),
        None,
        None,
    )
    .await
    .expect("graceful response");

    assert_eq!(res_bad_perm["success"], false);
    assert_eq!(res_bad_perm["allowed"], false);
    assert!(res_bad_perm["error"].as_str().unwrap().contains("Unknown permission"));
}

#[test]
fn test_adversarial_loose_permission_parsing() {
    // Verify Permission::from_str_loose normalizes diverse format styles:
    let variations = [
        ("IngestStatement", Permission::IngestStatement),
        ("ingest_statement", Permission::IngestStatement),
        ("ingestStatement", Permission::IngestStatement),
        ("INGESTSTATEMENT", Permission::IngestStatement),
        ("  ingest_statement  ", Permission::IngestStatement),
        ("ingest-statement", Permission::IngestStatement),
        ("approve_hitl_match", Permission::ApproveHitlMatch),
        ("approveHitlMatch", Permission::ApproveHitlMatch),
        ("propose_payment_order", Permission::ProposePaymentOrder),
        ("propose_payment", Permission::ProposePaymentOrder),
        ("approve_payment_order", Permission::ApprovePaymentOrder),
        ("approve_payment", Permission::ApprovePaymentOrder),
        ("reject_payment_order", Permission::RejectPaymentOrder),
        ("reject_payment", Permission::RejectPaymentOrder),
        ("view_cashflow_30_90", Permission::ViewCashflow30_90),
        ("view_cashflow", Permission::ViewCashflow30_90),
        ("approve_high_limit", Permission::ApproveHighLimit),
        ("view_audit_chain", Permission::ViewAuditChain),
        ("verify_merkle_proof", Permission::VerifyMerkleProof),
        ("export_decree13_report", Permission::ExportDecree13Report),
        ("export_decree13", Permission::ExportDecree13Report),
        ("manage_system_config", Permission::ManageSystemConfig),
        ("manage_users", Permission::ManageUsers),
        ("view_system_diagnostics", Permission::ViewSystemDiagnostics),
    ];

    for (input, expected) in variations {
        let parsed = Permission::from_str_loose(input).unwrap_or_else(|_| panic!("Failed to parse: '{input}'"));
        assert_eq!(parsed, expected, "Mismatch for input: '{input}'");
    }
}

// ===========================================================================
// Test 5: Principal Isolation for auth:check_permission
// ===========================================================================

#[test]
fn test_adversarial_principal_isolation_auth_check_permission() {
    for cmd in ["auth:check_permission", "auth_check_permission"] {
        // Permitted callers:
        assert!(authorize_command(CommandPrincipal::TauriDashboard, cmd).is_ok());
        assert!(authorize_command(CommandPrincipal::WebSocketDashboard, cmd).is_ok());
        assert!(authorize_command(CommandPrincipal::TauriWidget, cmd).is_ok());
        assert!(authorize_command(CommandPrincipal::WebSocketWidget, cmd).is_ok());
        assert!(authorize_command(CommandPrincipal::LocalCli, cmd).is_ok());
        assert!(authorize_command(CommandPrincipal::Test, cmd).is_ok());

        // Prohibited callers: Fail-Closed!
        assert!(
            authorize_command(CommandPrincipal::TauriSetup, cmd).is_err(),
            "TauriSetup must NOT access {}",
            cmd
        );
        assert!(
            authorize_command(CommandPrincipal::WebSocketRemote, cmd).is_err(),
            "WebSocketRemote must NOT access {}",
            cmd
        );
        assert!(
            authorize_command(CommandPrincipal::Telegram, cmd).is_err(),
            "Telegram must NOT access {}",
            cmd
        );
    }
}

// ===========================================================================
// Test 6: Whitespace and Mixed-Case Role Resolution
// ===========================================================================

#[tokio::test]
async fn test_adversarial_whitespace_and_mixed_case_roles() {
    let state = challenger_test_state();

    // 1. Whitespace and lowercase roles in payload
    let test_cases = [
        ("  MAKER  ", "IngestStatement", true, false),
        ("  maker  ", "ApproveHitlMatch", false, false),
        ("  CHECKER  ", "ApproveHitlMatch", true, false),
        ("checker", "IngestStatement", false, false),
        ("Cfo", "ApproveHighLimit", true, false),
        ("  cfo  ", "IngestStatement", false, false),
        ("Auditor", "ViewAuditChain", true, false),
        ("auditor", "ApprovePaymentOrder", false, false),
        ("  Admin  ", "ManageSystemConfig", true, false),
        ("admin", "statement_ingest_file", false, true),
    ];

    for (role_input, perm_or_cmd, expected, is_command) in test_cases {
        let payload = if is_command {
            json!({
                "role": role_input,
                "command": perm_or_cmd
            })
        } else {
            json!({
                "role": role_input,
                "permission": perm_or_cmd
            })
        };

        let res = handle_command_as(
            CommandPrincipal::TauriDashboard,
            state.clone(),
            "auth:check_permission",
            payload,
            None,
            None,
        )
        .await
        .expect("check_permission completed");

        assert_eq!(
            res["success"], true,
            "Role input '{}' must be successfully recognized",
            role_input
        );
        assert_eq!(
            res["allowed"], expected,
            "Role '{}' on '{}' expected allowed: {}",
            role_input, perm_or_cmd, expected
        );
    }

    // 2. Create user with lowercase role and verify DB resolution
    let create_res = handle_command_as(
        CommandPrincipal::TauriDashboard,
        state.clone(),
        "auth:create_user",
        json!({
            "username": "custom_checker_user",
            "password": "Password@2026",
            "role": "checker"
        }),
        None,
        None,
    )
    .await
    .expect("user created");

    assert_eq!(create_res["success"], true);

    // Query permission via username
    let check_custom = handle_command_as(
        CommandPrincipal::TauriDashboard,
        state.clone(),
        "auth:check_permission",
        json!({
            "username": "custom_checker_user",
            "permission": "ApproveHitlMatch"
        }),
        None,
        None,
    )
    .await
    .expect("permission check succeeded");

    assert_eq!(check_custom["success"], true);
    assert_eq!(check_custom["allowed"], true);
    assert_eq!(check_custom["role"], "CHECKER");
}

// ===========================================================================
// Test 7: SoD Self-Approval Rejection & Audit Trail Integrity (HITL)
// ===========================================================================

#[tokio::test]
async fn test_adversarial_sod_self_approval_hitl_and_audit_logging() {
    let state = challenger_test_state();

    // Attempt resolve_hitl with maker_id == checker_id (case-insensitive)
    let res = handle_command_as(
        CommandPrincipal::TauriDashboard,
        state.clone(),
        "reconciliation_resolve_hitl",
        json!({
            "match_id": "MATCH_ADVERSARIAL_01",
            "hitl_token": "TOKEN_ADVERSARIAL_01",
            "decision": "APPROVE",
            "maker_id": "maker_nam",
            "checker_id": "  MAKER_NAM  " // Hostile case + whitespace bypass attempt
        }),
        None,
        None,
    )
    .await;

    // Must return Err
    assert!(res.is_err(), "Self-approval must be strictly rejected with Err");
    let err_msg = res.unwrap_err();
    assert!(
        err_msg.contains("Circular 09/2020/TT-NHNN violation") || err_msg.contains("cannot self-approve"),
        "Error message must specify dual control violation: {err_msg}"
    );

    // Verify violation was appended to AuditLedger in DB
    let conn = state.db.writer.get().expect("DB connection");
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM banking_audit_chain WHERE event_type = 'SOD_SELF_APPROVAL_BLOCKED'",
            [],
            |r| r.get(0),
        )
        .expect("query audit chain");
    assert!(count >= 1, "Must have at least 1 SOD_SELF_APPROVAL_BLOCKED record in audit ledger");

    // Verify audit chain cryptographic integrity using state's derived audit key
    let audit_key = liva_native_core::banking::compliance::get_audit_key(&state);
    let report = liva_native_core::banking::compliance::AuditLedger::verify(&conn, &audit_key)
        .expect("verify audit chain");
    assert!(report.is_intact, "Audit ledger must remain 100% cryptographically intact after SoD violation logging");
}

// ===========================================================================
// Test 8: SoD Self-Approval Rejection & Audit Trail Integrity (Treasury)
// ===========================================================================

#[test]
fn test_adversarial_sod_self_approval_treasury_and_audit_logging() {
    use liva_native_core::banking::treasury::{TreasuryEngine, init_payment_orders_table};
    use liva_native_core::banking::compliance::maker_checker::MakerCheckerError;
    use rusqlite::Connection;

    let conn = Connection::open_in_memory().expect("in-memory db");
    liva_native_core::banking::compliance::AuditLedger::init_audit_table(&conn).expect("init audit ledger");
    init_payment_orders_table(&conn).expect("init payment orders");

    let mut order = TreasuryEngine::propose(
        "19034567890123",
        "0011009876543",
        "CONG TY TNHH ABC",
        "VCB",
        150_000_000,
        "Thanh toan hop dong cung cap dich vu",
        "maker_nam",
    )
    .expect("propose payment order");

    let token = order.hitl_token.clone().expect("hitl token present");
    let audit_key = [0x42u8; 32];

    // Attempt self-approval with case variation: "MAKER_NAM" vs "maker_nam"
    let err = TreasuryEngine::approve(
        &mut order,
        "  MAKER_NAM  ",
        &token,
        Some(&audit_key),
        Some(&conn),
    )
    .expect_err("Maker must not be permitted to self-approve as Checker");

    assert!(matches!(err, MakerCheckerError::SelfApprovalProhibited { .. }));

    // Verify that the SoD violation was logged to the tamper-evident audit ledger
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM banking_audit_chain WHERE event_type = 'SOD_SELF_APPROVAL_BLOCKED'",
            [],
            |r| r.get(0),
        )
        .expect("query audit ledger");
    assert_eq!(count, 1, "Exactly 1 SoD violation record must be logged");

    // Verify that the forward HMAC-SHA256 hash chain remains 100% intact
    let report = liva_native_core::banking::compliance::AuditLedger::verify(&conn, &audit_key)
        .expect("verify audit ledger");
    assert!(report.is_intact, "Audit ledger must remain 100% cryptographically intact");
    assert_eq!(report.total_records, 1);
}

