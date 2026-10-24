use liva_native_core::crypto::EncryptionEngine;
use liva_native_core::{
    AppState, CommandPrincipal, Permission, UserRole, authorize_command,
    check_command_role_permission, check_user_permission, get_role_permissions, db,
    handle_command_as, llm, stt, tts,
};
use std::sync::Arc;

fn test_state() -> Arc<AppState> {
    let db = db::DatabasePool::new_in_memory().expect("in-memory database");
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

fn duoc(principal: CommandPrincipal, command: &str) {
    authorize_command(principal, command)
        .unwrap_or_else(|error| panic!("{principal:?} phải được gọi {command}: {error}"));
}

fn bi_chan(principal: CommandPrincipal, command: &str) {
    assert!(
        authorize_command(principal, command).is_err(),
        "{principal:?} không được phép gọi {command}"
    );
}

#[test]
fn setup_chi_duoc_dung_mien_setup() {
    for command in ["setup:status", "setup:paths", "setup:fetch"] {
        duoc(CommandPrincipal::TauriSetup, command);
    }

    for command in [
        "get_config",
        "update_config",
        "get_memory_data",
        "llm:swap_model",
        "mcp:call_tool",
    ] {
        bi_chan(CommandPrincipal::TauriSetup, command);
    }
}

#[test]
fn widget_duoc_hoi_thoai_nhung_khong_duoc_quan_tri() {
    for command in [
        "ping",
        "get_config",
        "get_system_status",
        "get_user_profile",
        "get_avatar_models",
        "chat:completion",
        "vision:ask",
        "voice:tts_speak",
        "message:pending",
        "message:confirm",
        "message:cancel",
        "telemetry:summary",
    ] {
        duoc(CommandPrincipal::TauriWidget, command);
        duoc(CommandPrincipal::WebSocketWidget, command);
    }

    for command in [
        "update_config",
        "get_memory_data",
        "delete_memory_fact",
        "memory:delete_subject",
        "memory:sweep_retention",
        "add_task",
        "skills:pin_ids",
        "llm:swap_model",
        "setup:fetch",
        "mcp:call_tool",
        "telegram:send_text",
    ] {
        bi_chan(CommandPrincipal::TauriWidget, command);
        bi_chan(CommandPrincipal::WebSocketWidget, command);
    }
}

#[test]
fn dashboard_duoc_quan_tri_ui_nhung_khong_co_cua_thoat_native_tho() {
    for command in [
        "get_config",
        "update_config",
        "get_memory_data",
        "delete_memory_fact",
        "memory:delete_conversation",
        "memory:delete_subject",
        "memory:sweep_retention",
        "consolidate_memory",
        "get_tasks",
        "add_task",
        "skills:list",
        "skills:search",
        "consent:grant",
        "vision:set_config",
        "voice:set_vieneu_voice",
        "telemetry:summary",
    ] {
        duoc(CommandPrincipal::TauriDashboard, command);
        duoc(CommandPrincipal::WebSocketDashboard, command);
    }

    for command in [
        "setup:fetch",
        "llm:swap_model",
        "mcp:call_tool",
        "mcp_client:call_tool",
        "skills:pin_ids",
        "telegram:send_text",
        "integration:smart_home_control",
    ] {
        bi_chan(CommandPrincipal::TauriDashboard, command);
        bi_chan(CommandPrincipal::WebSocketDashboard, command);
    }
}

#[test]
fn websocket_remote_chi_duoc_hoi_thoai_khong_doc_du_lieu_may() {
    for command in [
        "ping",
        "status",
        "llm:health_check",
        "chat:completion",
        "voice:stt_start",
        "voice:stt_chunk",
        "voice:stt_stop",
        "voice:tts_speak",
        "voice:tts_stop",
    ] {
        duoc(CommandPrincipal::WebSocketRemote, command);
    }

    for command in [
        "get_config",
        "get_user_profile",
        "get_memory_data",
        "vision:capture",
        "vision:ask",
        "update_config",
        "message:confirm",
        "setup:fetch",
        "mcp:list_tools",
    ] {
        bi_chan(CommandPrincipal::WebSocketRemote, command);
    }
}

#[test]
fn principal_khong_tin_cay_mac_dinh_tu_choi_lenh_moi() {
    for principal in [
        CommandPrincipal::TauriWidget,
        CommandPrincipal::TauriDashboard,
        CommandPrincipal::TauriSetup,
        CommandPrincipal::WebSocketWidget,
        CommandPrincipal::WebSocketDashboard,
        CommandPrincipal::WebSocketRemote,
        CommandPrincipal::Telegram,
    ] {
        bi_chan(principal, "future:dangerous_command");
    }
}

#[test]
fn cli_noi_bo_va_test_la_hai_principal_tin_cay_ro_rang() {
    for principal in [CommandPrincipal::LocalCli, CommandPrincipal::Test] {
        duoc(principal, "mcp:call_tool");
        duoc(principal, "llm:swap_model");
        duoc(principal, "future:diagnostic_command");
    }
}

#[tokio::test]
async fn dispatcher_chan_principal_truoc_khi_chay_handler() {
    let error = handle_command_as(
        CommandPrincipal::TauriWidget,
        test_state(),
        "update_config",
        serde_json::json!({"system": {"language": "en"}}),
        None,
        None,
    )
    .await
    .expect_err("widget không được ghi config");

    assert!(error.contains("TauriWidget"), "{error}");
    assert!(error.contains("update_config"), "{error}");
}

#[tokio::test]
async fn dispatcher_van_chay_lenh_duoc_cap_quyen() {
    let response = handle_command_as(
        CommandPrincipal::TauriWidget,
        test_state(),
        "ping",
        serde_json::json!({}),
        None,
        None,
    )
    .await
    .expect("widget được ping");

    assert_eq!(response, serde_json::json!({"pong": true}));
}

#[tokio::test]
async fn dashboard_co_the_chay_projection_batch_thu_cong() {
    let response = handle_command_as(
        CommandPrincipal::TauriDashboard,
        test_state(),
        "consolidate_memory",
        serde_json::json!({"batchSize": 25}),
        None,
        None,
    )
    .await
    .expect("dashboard được chạy projection batch");

    assert_eq!(response["processed"], 0);
    assert_eq!(response["consolidated"], 0);
}

#[test]
fn dashboard_va_widget_duoc_dung_auth_check_permission() {
    for cmd in ["auth:check_permission", "auth_check_permission"] {
        duoc(CommandPrincipal::TauriDashboard, cmd);
        duoc(CommandPrincipal::WebSocketDashboard, cmd);
        duoc(CommandPrincipal::TauriWidget, cmd);
        duoc(CommandPrincipal::WebSocketWidget, cmd);
        duoc(CommandPrincipal::LocalCli, cmd);
        duoc(CommandPrincipal::Test, cmd);

        bi_chan(CommandPrincipal::TauriSetup, cmd);
        bi_chan(CommandPrincipal::WebSocketRemote, cmd);
        bi_chan(CommandPrincipal::Telegram, cmd);
    }
}

#[test]
fn rbac_user_role_exhaustive_matrix() {
    let roles = [
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

    for role in roles {
        let granted = get_role_permissions(role);
        for perm in all_permissions {
            let allowed = check_user_permission(role, perm);
            assert_eq!(
                allowed,
                granted.contains(&perm),
                "Mismatch for role {role:?} and permission {perm:?}"
            );
        }
    }

    // Explicit invariant assertions:
    // 1. Maker: Propose-only, strictly zero approval permissions
    assert!(check_user_permission(UserRole::Maker, Permission::IngestStatement));
    assert!(check_user_permission(UserRole::Maker, Permission::RunReconciliation));
    assert!(check_user_permission(UserRole::Maker, Permission::ProposeHitlMatch));
    assert!(check_user_permission(UserRole::Maker, Permission::ProposePaymentOrder));
    assert!(!check_user_permission(UserRole::Maker, Permission::ApproveHitlMatch));
    assert!(!check_user_permission(UserRole::Maker, Permission::ApprovePaymentOrder));
    assert!(!check_user_permission(UserRole::Maker, Permission::ApproveHighLimit));

    // 2. Checker: Approve-only, strictly zero propose/ingest permissions
    assert!(check_user_permission(UserRole::Checker, Permission::ApproveHitlMatch));
    assert!(check_user_permission(UserRole::Checker, Permission::ApprovePaymentOrder));
    assert!(check_user_permission(UserRole::Checker, Permission::RejectPaymentOrder));
    assert!(!check_user_permission(UserRole::Checker, Permission::IngestStatement));
    assert!(!check_user_permission(UserRole::Checker, Permission::RunReconciliation));
    assert!(!check_user_permission(UserRole::Checker, Permission::ProposeHitlMatch));
    assert!(!check_user_permission(UserRole::Checker, Permission::ProposePaymentOrder));

    // 3. CFO: High-limit, payment approve/reject, cashflow 30-90, zero HITL match approval
    assert!(check_user_permission(UserRole::Cfo, Permission::ViewCashflow30_90));
    assert!(check_user_permission(UserRole::Cfo, Permission::ApproveHighLimit));
    assert!(check_user_permission(UserRole::Cfo, Permission::ApprovePaymentOrder));
    assert!(check_user_permission(UserRole::Cfo, Permission::RejectPaymentOrder));
    assert!(!check_user_permission(UserRole::Cfo, Permission::ApproveHitlMatch));
    assert!(!check_user_permission(UserRole::Cfo, Permission::IngestStatement));

    // 4. Auditor: Strict read-only audit & Merkle verification
    assert!(check_user_permission(UserRole::Auditor, Permission::ViewAuditChain));
    assert!(check_user_permission(UserRole::Auditor, Permission::VerifyMerkleProof));
    assert!(check_user_permission(UserRole::Auditor, Permission::ExportDecree13Report));
    assert!(!check_user_permission(UserRole::Auditor, Permission::IngestStatement));
    assert!(!check_user_permission(UserRole::Auditor, Permission::ApproveHitlMatch));

    // 5. Admin: Technical system operations only, strictly zero banking transactions
    assert!(check_user_permission(UserRole::Admin, Permission::ManageSystemConfig));
    assert!(check_user_permission(UserRole::Admin, Permission::ManageUsers));
    assert!(check_user_permission(UserRole::Admin, Permission::ViewSystemDiagnostics));
    assert!(!check_user_permission(UserRole::Admin, Permission::IngestStatement));
    assert!(!check_user_permission(UserRole::Admin, Permission::ApprovePaymentOrder));
}

#[test]
fn rbac_command_role_enforcement_and_violations() {
    // 1. Common commands permitted to all roles
    for role in [
        UserRole::Maker,
        UserRole::Checker,
        UserRole::Cfo,
        UserRole::Auditor,
        UserRole::Admin,
    ] {
        assert!(check_command_role_permission(role, "ping").is_ok());
        assert!(check_command_role_permission(role, "banking:get_overview").is_ok());
        assert!(check_command_role_permission(role, "auth:check_permission").is_ok());
        assert!(check_command_role_permission(role, "auth_check_permission").is_ok());
        assert!(check_command_role_permission(role, "banking_get_reconciliation_matrix").is_ok());
        assert!(check_command_role_permission(role, "banking_get_compliance_status").is_ok());
    }

    // 2. Maker boundaries
    assert!(check_command_role_permission(UserRole::Maker, "statement_ingest_file").is_ok());
    assert!(check_command_role_permission(UserRole::Maker, "statement:ingest_file").is_ok());
    assert!(check_command_role_permission(UserRole::Maker, "banking_run_reconciliation").is_ok());
    assert!(check_command_role_permission(UserRole::Maker, "reconciliation_resolve_hitl").is_err());
    assert!(check_command_role_permission(UserRole::Maker, "banking:resolve_hitl").is_err());
    assert!(check_command_role_permission(UserRole::Maker, "treasury:approve_payment").is_err());
    assert!(check_command_role_permission(UserRole::Maker, "update_config").is_err());

    // 3. Checker boundaries
    assert!(check_command_role_permission(UserRole::Checker, "reconciliation_resolve_hitl").is_ok());
    assert!(check_command_role_permission(UserRole::Checker, "banking:resolve_hitl").is_ok());
    assert!(check_command_role_permission(UserRole::Checker, "treasury:approve_payment").is_ok());
    assert!(check_command_role_permission(UserRole::Checker, "statement_ingest_file").is_err());
    assert!(check_command_role_permission(UserRole::Checker, "banking_run_reconciliation").is_err());

    // 4. Auditor boundaries
    assert!(check_command_role_permission(UserRole::Auditor, "compliance:get_audit_chain").is_ok());
    assert!(check_command_role_permission(UserRole::Auditor, "compliance:verify_merkle").is_ok());
    assert!(check_command_role_permission(UserRole::Auditor, "compliance:export_decree13").is_ok());
    assert!(check_command_role_permission(UserRole::Auditor, "statement_ingest_file").is_err());
    assert!(check_command_role_permission(UserRole::Auditor, "reconciliation_resolve_hitl").is_err());

    // 5. Admin boundaries
    assert!(check_command_role_permission(UserRole::Admin, "update_config").is_ok());
    assert!(check_command_role_permission(UserRole::Admin, "auth:create_user").is_ok());
    assert!(check_command_role_permission(UserRole::Admin, "get_preflight_status").is_ok());
    assert!(check_command_role_permission(UserRole::Admin, "statement_ingest_file").is_err());
    assert!(check_command_role_permission(UserRole::Admin, "reconciliation_resolve_hitl").is_err());
    assert!(check_command_role_permission(UserRole::Admin, "treasury:approve_payment").is_err());
}

#[tokio::test]
async fn dispatcher_executes_auth_check_permission_e2e() {
    let state = test_state();

    // Mode A: Role + Permission check (valid)
    let res = handle_command_as(
        CommandPrincipal::TauriDashboard,
        state.clone(),
        "auth:check_permission",
        serde_json::json!({
            "role": "MAKER",
            "permission": "IngestStatement"
        }),
        None,
        None,
    )
    .await
    .expect("auth:check_permission allowed");
    assert_eq!(res["success"], true);
    assert_eq!(res["allowed"], true);
    assert_eq!(res["role"], "MAKER");
    assert_eq!(res["permission"], "IngestStatement");

    // Mode A: Role + Permission check (SoD violation)
    let res_denied = handle_command_as(
        CommandPrincipal::TauriDashboard,
        state.clone(),
        "auth_check_permission",
        serde_json::json!({
            "role": "MAKER",
            "permission": "ApproveHitlMatch"
        }),
        None,
        None,
    )
    .await
    .expect("auth_check_permission executed");
    assert_eq!(res_denied["success"], true);
    assert_eq!(res_denied["allowed"], false);
    assert!(res_denied["reason"].as_str().unwrap().contains("lacks permission"));

    // Mode B: Role + Command check (valid)
    let res_cmd_ok = handle_command_as(
        CommandPrincipal::TauriDashboard,
        state.clone(),
        "auth:check_permission",
        serde_json::json!({
            "role": "CHECKER",
            "command": "reconciliation_resolve_hitl"
        }),
        None,
        None,
    )
    .await
    .expect("command check ok");
    assert_eq!(res_cmd_ok["success"], true);
    assert_eq!(res_cmd_ok["allowed"], true);

    // Mode B: Role + Command check (SoD violation)
    let res_cmd_denied = handle_command_as(
        CommandPrincipal::TauriDashboard,
        state.clone(),
        "auth:check_permission",
        serde_json::json!({
            "role": "CHECKER",
            "command": "statement_ingest_file"
        }),
        None,
        None,
    )
    .await
    .expect("command check executed");
    assert_eq!(res_cmd_denied["success"], true);
    assert_eq!(res_cmd_denied["allowed"], false);

    // Mode C: Username database lookup
    let res_user = handle_command_as(
        CommandPrincipal::TauriDashboard,
        state.clone(),
        "auth:check_permission",
        serde_json::json!({
            "username": "maker_nam",
            "permission": "RunReconciliation"
        }),
        None,
        None,
    )
    .await
    .expect("username lookup ok");
    assert_eq!(res_user["success"], true);
    assert_eq!(res_user["allowed"], true);
    assert_eq!(res_user["role"], "MAKER");

    // Mode D: Capability discovery mode
    let res_discovery = handle_command_as(
        CommandPrincipal::TauriDashboard,
        state.clone(),
        "auth:check_permission",
        serde_json::json!({
            "role": "AUDITOR"
        }),
        None,
        None,
    )
    .await
    .expect("discovery ok");
    assert_eq!(res_discovery["success"], true);
    assert_eq!(res_discovery["allowed"], true);
    let perms = res_discovery["permissions"].as_array().unwrap();
    let perm_strings: Vec<&str> = perms.iter().filter_map(|v| v.as_str()).collect();
    assert_eq!(perm_strings.len(), 3);
    assert!(perm_strings.contains(&"ViewAuditChain"));
    assert!(perm_strings.contains(&"VerifyMerkleProof"));
    assert!(perm_strings.contains(&"ExportDecree13Report"));
}
