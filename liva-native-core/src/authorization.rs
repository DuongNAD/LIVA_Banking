//! Principal-aware command authorization for every external command-plane entry.
//!
//! Trusted local diagnostics keep the legacy unrestricted dispatcher. Every
//! WebView/WebSocket/Telegram principal is fail-closed and must appear in an
//! explicit allow-list here before it can reach [`crate::handle_command`].

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CommandPrincipal {
    LocalCli,
    Test,
    TauriWidget,
    TauriDashboard,
    TauriSetup,
    WebSocketWidget,
    WebSocketDashboard,
    WebSocketRemote,
    Telegram,
}

const SETUP_COMMANDS: &[&str] = &["setup:status", "setup:paths", "setup:fetch"];

const WIDGET_COMMANDS: &[&str] = &[
    "ping",
    "status",
    "audio_play_started",
    "audio_play_finished",
    "get_config",
    "get_ai_config",
    "get_voice_status",
    "get_voice_profiles",
    "get_system_status",
    "get_memory_status",
    "get_user_profile",
    "get_avatar_models",
    "llm:health_check",
    "chat:completion",
    "vision:capture",
    "vision:ask",
    "voice:stt_start",
    "voice:stt_chunk",
    "voice:stt_stop",
    "voice:tts_speak",
    "voice:tts_stop",
    "message:draft",
    "message:confirm",
    "message:cancel",
    "message:pending",
    "messenger:status",
    "telemetry:summary",
    "auth:login",
    "auth:get_quick_accounts",
    "auth:get_user",
    "auth:check_permission",
    "auth_login",
    "auth_get_quick_accounts",
    "auth_check_permission",
    "config:get",
    "config:update",
];

const DASHBOARD_COMMANDS: &[&str] = &[
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
    "ping",
    "echo",
    "status",
    "get_config",
    "update_config",
    "config:get",
    "config:update",
    "get_ai_config",
    "ai:get_config",
    "ai:update_config",
    "auth:login",
    "auth:get_quick_accounts",
    "auth:create_user",
    "auth:get_user",
    "auth:check_permission",
    "auth_login",
    "auth_get_quick_accounts",
    "auth_create_user",
    "auth_check_permission",
    "get_voice_status",
    "get_voice_profiles",
    "select_voice_profile",
    "get_system_status",
    "get_memory_status",
    "get_preflight_status",
    "get_skills_list",
    "toggle_skill",
    "toggle_all_skills",
    "get_user_profile",
    "update_user_profile",
    "get_avatar_models",
    "import_avatar_folder",
    "delete_avatar_model",
    "consent:get",
    "consent:grant",
    "consent:revoke",
    "integrations:list",
    "llm:embed",
    "llm:health_check",
    "chat:completion",
    "task_plan_chat",
    "get_memory_data",
    "memory:set_fact",
    "memory:get_fact",
    "delete_memory_fact",
    "memory:delete_conversation",
    "memory:delete_subject",
    "memory:sweep_retention",
    "consolidate_memory",
    "reset_memory",
    "memory:search_hybrid",
    "memory:upsert_vector",
    "contacts:list",
    "contacts:upsert",
    "contacts:delete",
    "message:draft",
    "message:confirm",
    "message:cancel",
    "message:pending",
    "messenger:status",
    "get_tasks",
    "add_task",
    "delete_task",
    "update_task",
    "vision:capture",
    "vision:add_region",
    "vision:remove_region",
    "vision:get_changed_regions",
    "vision:set_config",
    "vision:ask",
    "voice:stt_start",
    "voice:stt_chunk",
    "voice:stt_stop",
    "voice:set_language",
    "voice:list_vieneu_voices",
    "voice:set_vieneu_voice",
    "voice:tts_speak",
    "voice:tts_stop",
    "mcp:list_tools",
    "mcp_client:list_servers",
    "mcp_client:list_tools",
    "skills:list",
    "skills:search",
    "skills:signals",
    "skills:history",
    "telemetry:summary",
];

const REMOTE_COMMANDS: &[&str] = &[
    "ping",
    "status",
    "llm:health_check",
    "chat:completion",
    "voice:stt_start",
    "voice:stt_chunk",
    "voice:stt_stop",
    "voice:tts_speak",
    "voice:tts_stop",
];

const TELEGRAM_COMMANDS: &[&str] = &["ping", "status", "chat:completion"];

pub fn is_known_command(command: &str) -> bool {
    SETUP_COMMANDS.contains(&command)
        || WIDGET_COMMANDS.contains(&command)
        || DASHBOARD_COMMANDS.contains(&command)
        || REMOTE_COMMANDS.contains(&command)
        || TELEGRAM_COMMANDS.contains(&command)
}

pub fn authorize_command(principal: CommandPrincipal, command: &str) -> Result<(), String> {
    let allowed = match principal {
        CommandPrincipal::LocalCli | CommandPrincipal::Test => true,
        CommandPrincipal::TauriSetup => SETUP_COMMANDS.contains(&command),
        CommandPrincipal::TauriWidget | CommandPrincipal::WebSocketWidget => {
            WIDGET_COMMANDS.contains(&command)
        }
        CommandPrincipal::TauriDashboard | CommandPrincipal::WebSocketDashboard => {
            DASHBOARD_COMMANDS.contains(&command)
        }
        CommandPrincipal::WebSocketRemote => REMOTE_COMMANDS.contains(&command),
        CommandPrincipal::Telegram => TELEGRAM_COMMANDS.contains(&command),
    };

    if allowed {
        Ok(())
    } else {
        Err(format!(
            "principal {principal:?} is not authorized for command '{command}'"
        ))
    }
}

/// Authenticated operator roles under SBV Circular 09/2020/TT-NHNN and Decree 13/2023/NĐ-CP.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserRole {
    #[serde(alias = "Maker", alias = "maker")]
    Maker,
    #[serde(alias = "Checker", alias = "checker")]
    Checker,
    #[serde(alias = "Cfo", alias = "cfo", alias = "CFO")]
    Cfo,
    #[serde(alias = "Auditor", alias = "auditor")]
    Auditor,
    #[serde(alias = "Admin", alias = "admin")]
    Admin,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Maker => "MAKER",
            UserRole::Checker => "CHECKER",
            UserRole::Cfo => "CFO",
            UserRole::Auditor => "AUDITOR",
            UserRole::Admin => "ADMIN",
        }
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "MAKER" => Ok(UserRole::Maker),
            "CHECKER" => Ok(UserRole::Checker),
            "CFO" => Ok(UserRole::Cfo),
            "AUDITOR" => Ok(UserRole::Auditor),
            "ADMIN" => Ok(UserRole::Admin),
            _ => Err(format!("Unknown user role: '{s}'")),
        }
    }
}

/// Discrete operational and governance permissions across LIVA Banking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Permission {
    IngestStatement,
    RunReconciliation,
    ProposeHitlMatch,
    ApproveHitlMatch,
    ProposePaymentOrder,
    ApprovePaymentOrder,
    RejectPaymentOrder,
    ViewCashflow30_90,
    ApproveHighLimit,
    ViewAuditChain,
    VerifyMerkleProof,
    ExportDecree13Report,
    ManageSystemConfig,
    ManageUsers,
    ViewSystemDiagnostics,
}

impl Permission {
    pub fn as_str(&self) -> &'static str {
        match self {
            Permission::IngestStatement => "IngestStatement",
            Permission::RunReconciliation => "RunReconciliation",
            Permission::ProposeHitlMatch => "ProposeHitlMatch",
            Permission::ApproveHitlMatch => "ApproveHitlMatch",
            Permission::ProposePaymentOrder => "ProposePaymentOrder",
            Permission::ApprovePaymentOrder => "ApprovePaymentOrder",
            Permission::RejectPaymentOrder => "RejectPaymentOrder",
            Permission::ViewCashflow30_90 => "ViewCashflow30_90",
            Permission::ApproveHighLimit => "ApproveHighLimit",
            Permission::ViewAuditChain => "ViewAuditChain",
            Permission::VerifyMerkleProof => "VerifyMerkleProof",
            Permission::ExportDecree13Report => "ExportDecree13Report",
            Permission::ManageSystemConfig => "ManageSystemConfig",
            Permission::ManageUsers => "ManageUsers",
            Permission::ViewSystemDiagnostics => "ViewSystemDiagnostics",
        }
    }

    pub fn from_str_loose(s: &str) -> Result<Self, String> {
        let normalized = s.trim().replace(['_', '-', ' '], "").to_lowercase();
        match normalized.as_str() {
            "ingeststatement" => Ok(Permission::IngestStatement),
            "runreconciliation" => Ok(Permission::RunReconciliation),
            "proposehitlmatch" => Ok(Permission::ProposeHitlMatch),
            "approvehitlmatch" => Ok(Permission::ApproveHitlMatch),
            "proposepaymentorder" | "proposepayment" => Ok(Permission::ProposePaymentOrder),
            "approvepaymentorder" | "approvepayment" => Ok(Permission::ApprovePaymentOrder),
            "rejectpaymentorder" | "rejectpayment" => Ok(Permission::RejectPaymentOrder),
            "viewcashflow3090" | "viewcashflow" => Ok(Permission::ViewCashflow30_90),
            "approvehighlimit" => Ok(Permission::ApproveHighLimit),
            "viewauditchain" | "viewaudit" => Ok(Permission::ViewAuditChain),
            "verifymerkleproof" | "verifymerkle" => Ok(Permission::VerifyMerkleProof),
            "exportdecree13report" | "exportdecree13" => Ok(Permission::ExportDecree13Report),
            "managesystemconfig" | "manageconfig" => Ok(Permission::ManageSystemConfig),
            "manageusers" => Ok(Permission::ManageUsers),
            "viewsystemdiagnostics" | "viewdiagnostics" => Ok(Permission::ViewSystemDiagnostics),
            _ => Err(format!("Unknown permission: '{s}'")),
        }
    }
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for Permission {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_loose(s)
    }
}

/// Evaluates if a given role is authorized for a specific permission under strict fail-closed SoD rules.
pub fn check_user_permission(role: UserRole, permission: Permission) -> bool {
    match role {
        UserRole::Maker => matches!(
            permission,
            Permission::IngestStatement
                | Permission::RunReconciliation
                | Permission::ProposeHitlMatch
                | Permission::ProposePaymentOrder
        ),
        UserRole::Checker => matches!(
            permission,
            Permission::ApproveHitlMatch
                | Permission::ApprovePaymentOrder
                | Permission::RejectPaymentOrder
        ),
        UserRole::Cfo => matches!(
            permission,
            Permission::ViewCashflow30_90
                | Permission::ApproveHighLimit
                | Permission::ApprovePaymentOrder
                | Permission::RejectPaymentOrder
        ),
        UserRole::Auditor => matches!(
            permission,
            Permission::ViewAuditChain
                | Permission::VerifyMerkleProof
                | Permission::ExportDecree13Report
        ),
        UserRole::Admin => matches!(
            permission,
            Permission::ManageSystemConfig
                | Permission::ManageUsers
                | Permission::ViewSystemDiagnostics
        ),
    }
}

/// Returns all permissions granted to a given role.
pub fn get_role_permissions(role: UserRole) -> Vec<Permission> {
    const ALL_PERMISSIONS: &[Permission] = &[
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
    ALL_PERMISSIONS
        .iter()
        .copied()
        .filter(|&p| check_user_permission(role, p))
        .collect()
}

/// Verifies whether an authenticated human role is permitted to execute a specific command.
/// Fail-closed: unauthorized command invocations return Err.
pub fn check_command_role_permission(role: UserRole, command: &str) -> Result<(), String> {
    let cmd = command.trim();

    // 1. Common read-only & diagnostic operations permitted to all authenticated roles
    const COMMON_READ_COMMANDS: &[&str] = &[
        "ping",
        "echo",
        "status",
        "get_config",
        "config:get",
        "get_ai_config",
        "ai:get_config",
        "get_user_profile",
        "auth:login",
        "auth_login",
        "auth:get_quick_accounts",
        "auth_get_quick_accounts",
        "auth:get_user",
        "auth:check_permission",
        "auth_check_permission",
        "banking_get_overview",
        "banking:get_overview",
        "banking_get_reconciliation_matrix",
        "banking:get_reconciliation_matrix",
        "banking_get_compliance_status",
        "banking:get_compliance_status",
        "banking:seed_demo_data",
    ];

    if COMMON_READ_COMMANDS.contains(&cmd) {
        return Ok(());
    }

    // 2. Map discrete financial/governance commands to specific permissions
    let required_permission = match cmd {
        "statement_ingest_file" | "statement:ingest_file" | "banking:import_statement" => {
            Some(Permission::IngestStatement)
        }
        "banking_run_reconciliation" | "banking:run_reconciliation" => {
            Some(Permission::RunReconciliation)
        }
        "reconciliation_resolve_hitl" | "banking:resolve_hitl" => {
            Some(Permission::ApproveHitlMatch)
        }
        "treasury:propose_payment" | "payment:propose" | "treasury_propose_payment" => {
            Some(Permission::ProposePaymentOrder)
        }
        "treasury:approve_payment" | "payment:approve" | "treasury_approve_payment" => {
            Some(Permission::ApprovePaymentOrder)
        }
        "treasury:reject_payment" | "payment:reject" | "treasury_reject_payment" => {
            Some(Permission::RejectPaymentOrder)
        }
        "treasury:approve_high_limit" | "treasury_approve_high_limit" => {
            Some(Permission::ApproveHighLimit)
        }
        "cashflow:view_30_90" | "banking:get_cashflow_30_90" | "banking_get_cashflow_30_90" => {
            Some(Permission::ViewCashflow30_90)
        }
        "compliance:get_audit_chain" | "audit:view_chain" | "compliance_get_audit_chain" => {
            Some(Permission::ViewAuditChain)
        }
        "compliance:verify_merkle" | "audit:verify_merkle" | "compliance_verify_merkle" => {
            Some(Permission::VerifyMerkleProof)
        }
        "compliance:export_decree13" | "export_decree13_report" | "compliance_export_decree13" => {
            Some(Permission::ExportDecree13Report)
        }
        "update_config" | "config:update" | "ai:update_config" => {
            Some(Permission::ManageSystemConfig)
        }
        "auth:create_user" | "auth_create_user" => Some(Permission::ManageUsers),
        "get_system_status" | "get_preflight_status" | "get_skills_list" | "get_memory_status" => {
            Some(Permission::ViewSystemDiagnostics)
        }
        _ => None,
    };

    match required_permission {
        Some(perm) => {
            if check_user_permission(role, perm) {
                Ok(())
            } else {
                Err(format!(
                    "Role {:?} is not authorized for command '{}' (requires permission {:?})",
                    role, command, perm
                ))
            }
        }
        None => {
            // If the command is not mapped to any specific permission or common read,
            // check if Admin role can access system commands
            if role == UserRole::Admin && DASHBOARD_COMMANDS.contains(&cmd) {
                // Ensure Admin cannot execute financial transactions or approvals
                const FINANCIAL_COMMANDS: &[&str] = &[
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
                if FINANCIAL_COMMANDS.contains(&cmd) {
                    return Err(format!(
                        "Role Admin is strictly prohibited from executing financial transaction command '{}'",
                        command
                    ));
                }
                Ok(())
            } else {
                Err(format!(
                    "Role {:?} is not authorized for command '{}'",
                    role, command
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preflight_chi_mo_cho_dashboard_cuc_bo() {
        assert!(
            authorize_command(CommandPrincipal::TauriDashboard, "get_preflight_status").is_ok()
        );
        assert!(
            authorize_command(CommandPrincipal::WebSocketDashboard, "get_preflight_status").is_ok()
        );
        assert!(authorize_command(CommandPrincipal::TauriWidget, "get_preflight_status").is_err());
        assert!(
            authorize_command(CommandPrincipal::WebSocketRemote, "get_preflight_status").is_err()
        );
    }

    #[test]
    fn import_avatar_folder_chi_mo_cho_dashboard() {
        assert!(
            authorize_command(CommandPrincipal::TauriDashboard, "import_avatar_folder").is_ok()
        );
        assert!(
            authorize_command(CommandPrincipal::WebSocketDashboard, "import_avatar_folder").is_ok()
        );
        assert!(authorize_command(CommandPrincipal::TauriWidget, "import_avatar_folder").is_err());
        assert!(
            authorize_command(CommandPrincipal::WebSocketWidget, "import_avatar_folder").is_err()
        );
        assert!(
            authorize_command(CommandPrincipal::WebSocketRemote, "import_avatar_folder").is_err()
        );
        assert!(authorize_command(CommandPrincipal::Telegram, "import_avatar_folder").is_err());
    }

    #[test]
    fn toggle_skills_chi_mo_cho_dashboard() {
        for cmd in ["toggle_skill", "toggle_all_skills"] {
            assert!(authorize_command(CommandPrincipal::TauriDashboard, cmd).is_ok());
            assert!(authorize_command(CommandPrincipal::WebSocketDashboard, cmd).is_ok());
            assert!(authorize_command(CommandPrincipal::TauriWidget, cmd).is_err());
            assert!(authorize_command(CommandPrincipal::WebSocketWidget, cmd).is_err());
            assert!(authorize_command(CommandPrincipal::WebSocketRemote, cmd).is_err());
            assert!(authorize_command(CommandPrincipal::Telegram, cmd).is_err());
        }
    }

    #[test]
    fn disk_modifying_config_commands_chi_mo_cho_dashboard() {
        for cmd in [
            "update_user_profile",
            "select_voice_profile",
            "delete_avatar_model",
        ] {
            assert!(authorize_command(CommandPrincipal::TauriDashboard, cmd).is_ok());
            assert!(authorize_command(CommandPrincipal::WebSocketDashboard, cmd).is_ok());
            assert!(authorize_command(CommandPrincipal::TauriWidget, cmd).is_err());
            assert!(authorize_command(CommandPrincipal::WebSocketWidget, cmd).is_err());
            assert!(authorize_command(CommandPrincipal::WebSocketRemote, cmd).is_err());
            assert!(authorize_command(CommandPrincipal::Telegram, cmd).is_err());
        }
    }

    #[test]
    fn test_user_role_serde_and_parsing() {
        assert_eq!(serde_json::from_str::<UserRole>("\"MAKER\"").unwrap(), UserRole::Maker);
        assert_eq!(serde_json::from_str::<UserRole>("\"maker\"").unwrap(), UserRole::Maker);
        assert_eq!(serde_json::from_str::<UserRole>("\"Maker\"").unwrap(), UserRole::Maker);

        assert_eq!(serde_json::from_str::<UserRole>("\"CHECKER\"").unwrap(), UserRole::Checker);
        assert_eq!(serde_json::from_str::<UserRole>("\"checker\"").unwrap(), UserRole::Checker);
        assert_eq!(serde_json::from_str::<UserRole>("\"Checker\"").unwrap(), UserRole::Checker);

        assert_eq!(serde_json::from_str::<UserRole>("\"CFO\"").unwrap(), UserRole::Cfo);
        assert_eq!(serde_json::from_str::<UserRole>("\"cfo\"").unwrap(), UserRole::Cfo);
        assert_eq!(serde_json::from_str::<UserRole>("\"Cfo\"").unwrap(), UserRole::Cfo);

        assert_eq!(serde_json::from_str::<UserRole>("\"AUDITOR\"").unwrap(), UserRole::Auditor);
        assert_eq!(serde_json::from_str::<UserRole>("\"auditor\"").unwrap(), UserRole::Auditor);
        assert_eq!(serde_json::from_str::<UserRole>("\"Auditor\"").unwrap(), UserRole::Auditor);

        assert_eq!(serde_json::from_str::<UserRole>("\"ADMIN\"").unwrap(), UserRole::Admin);
        assert_eq!(serde_json::from_str::<UserRole>("\"admin\"").unwrap(), UserRole::Admin);
        assert_eq!(serde_json::from_str::<UserRole>("\"Admin\"").unwrap(), UserRole::Admin);

        assert_eq!(serde_json::to_string(&UserRole::Maker).unwrap(), "\"MAKER\"");
        assert_eq!(serde_json::to_string(&UserRole::Checker).unwrap(), "\"CHECKER\"");
        assert_eq!(serde_json::to_string(&UserRole::Cfo).unwrap(), "\"CFO\"");
        assert_eq!(serde_json::to_string(&UserRole::Auditor).unwrap(), "\"AUDITOR\"");
        assert_eq!(serde_json::to_string(&UserRole::Admin).unwrap(), "\"ADMIN\"");
    }

    #[test]
    fn test_permission_from_str_loose() {
        assert_eq!(Permission::from_str_loose("IngestStatement").unwrap(), Permission::IngestStatement);
        assert_eq!(Permission::from_str_loose("ingest_statement").unwrap(), Permission::IngestStatement);
        assert_eq!(Permission::from_str_loose("ingestStatement").unwrap(), Permission::IngestStatement);
        assert_eq!(Permission::from_str_loose("run_reconciliation").unwrap(), Permission::RunReconciliation);
        assert_eq!(Permission::from_str_loose("approve_hitl_match").unwrap(), Permission::ApproveHitlMatch);
        assert_eq!(Permission::from_str_loose("export_decree13_report").unwrap(), Permission::ExportDecree13Report);
        assert_eq!(Permission::from_str_loose("view_cashflow_30_90").unwrap(), Permission::ViewCashflow30_90);
    }

    #[test]
    fn test_rbac_fail_closed_sod_matrix() {
        // Maker: propose only, NO approval
        assert!(check_user_permission(UserRole::Maker, Permission::IngestStatement));
        assert!(check_user_permission(UserRole::Maker, Permission::RunReconciliation));
        assert!(check_user_permission(UserRole::Maker, Permission::ProposeHitlMatch));
        assert!(check_user_permission(UserRole::Maker, Permission::ProposePaymentOrder));
        assert!(!check_user_permission(UserRole::Maker, Permission::ApproveHitlMatch));
        assert!(!check_user_permission(UserRole::Maker, Permission::ApprovePaymentOrder));
        assert!(!check_user_permission(UserRole::Maker, Permission::ApproveHighLimit));
        assert!(!check_user_permission(UserRole::Maker, Permission::ManageSystemConfig));

        // Checker: approve only, NO propose, NO ingest
        assert!(check_user_permission(UserRole::Checker, Permission::ApproveHitlMatch));
        assert!(check_user_permission(UserRole::Checker, Permission::ApprovePaymentOrder));
        assert!(check_user_permission(UserRole::Checker, Permission::RejectPaymentOrder));
        assert!(!check_user_permission(UserRole::Checker, Permission::IngestStatement));
        assert!(!check_user_permission(UserRole::Checker, Permission::RunReconciliation));
        assert!(!check_user_permission(UserRole::Checker, Permission::ProposeHitlMatch));
        assert!(!check_user_permission(UserRole::Checker, Permission::ProposePaymentOrder));
        assert!(!check_user_permission(UserRole::Checker, Permission::ManageSystemConfig));

        // CFO: High limit, cashflow, payment approve/reject, NO HITL match approval
        assert!(check_user_permission(UserRole::Cfo, Permission::ViewCashflow30_90));
        assert!(check_user_permission(UserRole::Cfo, Permission::ApproveHighLimit));
        assert!(check_user_permission(UserRole::Cfo, Permission::ApprovePaymentOrder));
        assert!(check_user_permission(UserRole::Cfo, Permission::RejectPaymentOrder));
        assert!(!check_user_permission(UserRole::Cfo, Permission::ApproveHitlMatch));
        assert!(!check_user_permission(UserRole::Cfo, Permission::IngestStatement));
        assert!(!check_user_permission(UserRole::Cfo, Permission::RunReconciliation));

        // Auditor: Strict [READ-ONLY], audit chain, Merkle proof, Decree 13 export
        assert!(check_user_permission(UserRole::Auditor, Permission::ViewAuditChain));
        assert!(check_user_permission(UserRole::Auditor, Permission::VerifyMerkleProof));
        assert!(check_user_permission(UserRole::Auditor, Permission::ExportDecree13Report));
        assert!(!check_user_permission(UserRole::Auditor, Permission::IngestStatement));
        assert!(!check_user_permission(UserRole::Auditor, Permission::RunReconciliation));
        assert!(!check_user_permission(UserRole::Auditor, Permission::ApproveHitlMatch));
        assert!(!check_user_permission(UserRole::Auditor, Permission::ApprovePaymentOrder));
        assert!(!check_user_permission(UserRole::Auditor, Permission::ManageSystemConfig));

        // Admin: Technical SOC, system diagnostics, user management, NO financial transactions
        assert!(check_user_permission(UserRole::Admin, Permission::ManageSystemConfig));
        assert!(check_user_permission(UserRole::Admin, Permission::ManageUsers));
        assert!(check_user_permission(UserRole::Admin, Permission::ViewSystemDiagnostics));
        assert!(!check_user_permission(UserRole::Admin, Permission::IngestStatement));
        assert!(!check_user_permission(UserRole::Admin, Permission::RunReconciliation));
        assert!(!check_user_permission(UserRole::Admin, Permission::ApproveHitlMatch));
        assert!(!check_user_permission(UserRole::Admin, Permission::ApprovePaymentOrder));
        assert!(!check_user_permission(UserRole::Admin, Permission::ApproveHighLimit));
    }

    #[test]
    fn test_check_command_role_permission() {
        // Common commands allowed for all
        for role in [UserRole::Maker, UserRole::Checker, UserRole::Cfo, UserRole::Auditor, UserRole::Admin] {
            assert!(check_command_role_permission(role, "ping").is_ok());
            assert!(check_command_role_permission(role, "banking:get_overview").is_ok());
            assert!(check_command_role_permission(role, "auth:check_permission").is_ok());
            assert!(check_command_role_permission(role, "auth_check_permission").is_ok());
        }

        // Maker can ingest and reconcile, but CANNOT resolve HITL
        assert!(check_command_role_permission(UserRole::Maker, "statement_ingest_file").is_ok());
        assert!(check_command_role_permission(UserRole::Maker, "banking_run_reconciliation").is_ok());
        assert!(check_command_role_permission(UserRole::Maker, "reconciliation_resolve_hitl").is_err());
        assert!(check_command_role_permission(UserRole::Maker, "treasury:approve_payment").is_err());

        // Checker can resolve HITL, but CANNOT ingest statements or run recon
        assert!(check_command_role_permission(UserRole::Checker, "reconciliation_resolve_hitl").is_ok());
        assert!(check_command_role_permission(UserRole::Checker, "treasury:approve_payment").is_ok());
        assert!(check_command_role_permission(UserRole::Checker, "statement_ingest_file").is_err());
        assert!(check_command_role_permission(UserRole::Checker, "banking_run_reconciliation").is_err());

        // Auditor cannot execute financial mutations
        assert!(check_command_role_permission(UserRole::Auditor, "compliance:get_audit_chain").is_ok());
        assert!(check_command_role_permission(UserRole::Auditor, "compliance:verify_merkle").is_ok());
        assert!(check_command_role_permission(UserRole::Auditor, "reconciliation_resolve_hitl").is_err());
        assert!(check_command_role_permission(UserRole::Auditor, "statement_ingest_file").is_err());

        // Admin cannot execute financial transactions
        assert!(check_command_role_permission(UserRole::Admin, "update_config").is_ok());
        assert!(check_command_role_permission(UserRole::Admin, "auth:create_user").is_ok());
        assert!(check_command_role_permission(UserRole::Admin, "get_preflight_status").is_ok());
        assert!(check_command_role_permission(UserRole::Admin, "reconciliation_resolve_hitl").is_err());
        assert!(check_command_role_permission(UserRole::Admin, "statement_ingest_file").is_err());
    }
}
