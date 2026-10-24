//! Miền xác thực người dùng & RBAC cục bộ.
//!
//! Exposes IPC commands:
//! - `auth:login` / `auth_login`
//! - `auth:get_quick_accounts` / `auth_get_quick_accounts`
//! - `auth:create_user`
//! - `auth:get_user`

use crate::AppState;
use crate::db::users::{self, NewUser};
use serde_json::{json, Value};
use std::sync::Arc;

pub const OWNED: &[&str] = &[
    "auth:login",
    "auth:get_quick_accounts",
    "auth:create_user",
    "auth:get_user",
    "auth:check_permission",
    "auth_login",
    "auth_get_quick_accounts",
    "auth_check_permission",
];

pub fn owns(command: &str) -> bool {
    OWNED.contains(&command) || command.starts_with("auth:")
}

pub async fn handle(state: Arc<AppState>, command: &str, payload: Value) -> Result<Value, String> {
    let verb = command.strip_prefix("auth:").unwrap_or(command);
    match verb {
        "login" | "auth_login" => login(state, payload).await,
        "get_quick_accounts" | "auth_get_quick_accounts" => get_quick_accounts(state).await,
        "create_user" => create_user(state, payload).await,
        "get_user" => get_user(state, payload).await,
        "check_permission" | "auth_check_permission" => check_permission(state, payload).await,
        _ => Err(format!("Unknown auth verb: {verb}")),
    }
}

async fn login(state: Arc<AppState>, payload: Value) -> Result<Value, String> {
    let username = payload
        .get("username")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing username parameter".to_string())?
        .to_string();
    let password = payload
        .get("password")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let quick_login = payload
        .get("quick_login")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
        || payload
            .get("quickLogin")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

    let res = state
        .db
        .spawn_reader(move |conn| {
            users::verify_user_login(conn, &username, password.as_deref(), quick_login)
        })
        .await?;

    match res {
        Ok(profile) => Ok(json!({
            "success": true,
            "user": profile
        })),
        Err(err) => Ok(json!({
            "success": false,
            "error": err
        })),
    }
}

async fn get_quick_accounts(state: Arc<AppState>) -> Result<Value, String> {
    let accounts = state
        .db
        .spawn_reader(users::list_quick_accounts)
        .await?;

    Ok(json!({
        "success": true,
        "accounts": accounts
    }))
}

async fn create_user(state: Arc<AppState>, payload: Value) -> Result<Value, String> {
    let username = payload
        .get("username")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing username parameter".to_string())?
        .to_string();
    let password = payload
        .get("password")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing password parameter".to_string())?
        .to_string();
    let full_name = payload
        .get("fullName")
        .or_else(|| payload.get("full_name"))
        .and_then(|v| v.as_str())
        .unwrap_or(&username)
        .to_string();
    let role = payload
        .get("role")
        .and_then(|v| v.as_str())
        .unwrap_or("MAKER")
        .to_string();
    let department = payload
        .get("department")
        .and_then(|v| v.as_str())
        .map(str::to_string);

    let new_user = NewUser {
        username,
        password,
        full_name,
        role,
        department,
    };

    let res = state
        .db
        .spawn_writer(move |conn| users::create_user(conn, &new_user))
        .await?;

    match res {
        Ok(profile) => Ok(json!({
            "success": true,
            "user": profile
        })),
        Err(err) => Ok(json!({
            "success": false,
            "error": err
        })),
    }
}

async fn get_user(state: Arc<AppState>, payload: Value) -> Result<Value, String> {
    let username_opt = payload
        .get("username")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let id_opt = payload
        .get("id")
        .and_then(|v| v.as_str())
        .map(str::to_string);

    let user_record = state
        .db
        .spawn_reader(move |conn| {
            if let Some(ref u) = username_opt {
                users::get_user_by_username(conn, u)
            } else if let Some(ref id) = id_opt {
                users::get_user_by_id(conn, id)
            } else {
                Ok(None)
            }
        })
        .await?;

    if let Some(user) = user_record {
        let profile = users::UserPublicProfile {
            id: user.id,
            username: user.username,
            full_name: user.full_name.clone(),
            role_title: users::role_title(&user.role).to_string(),
            department: user.department.unwrap_or_else(|| "Phòng Tài chính".to_string()),
            description: users::role_description(&user.role).to_string(),
            avatar_initials: users::avatar_initials(&user.full_name, &user.role),
            avatar_color: users::avatar_color(&user.role).to_string(),
            role: user.role,
            status: user.status,
        };
        Ok(json!({ "success": true, "user": profile }))
    } else {
        Ok(json!({ "success": false, "error": "User not found" }))
    }
}

async fn check_permission(state: Arc<AppState>, payload: Value) -> Result<Value, String> {
    // 1. Resolve role from payload["role"] or query SQLite users table by username or user_id
    let role_res: Result<crate::authorization::UserRole, String> = if let Some(r_str) = payload.get("role").and_then(|v| v.as_str()) {
        r_str.parse()
    } else if let Some(username) = payload.get("username").and_then(|v| v.as_str()) {
        let username_str = username.to_string();
        let q_name = username_str.clone();
        let u = state
            .db
            .spawn_reader(move |conn| users::get_user_by_username(conn, &q_name))
            .await?;
        match u {
            Some(profile) => profile.role.parse(),
            None => {
                return Ok(json!({
                    "success": false,
                    "allowed": false,
                    "error": format!("User '{username_str}' not found")
                }));
            }
        }
    } else if let Some(user_id) = payload
        .get("user_id")
        .or_else(|| payload.get("userId"))
        .and_then(|v| v.as_str())
    {
        let user_id_str = user_id.to_string();
        let q_id = user_id_str.clone();
        let u = state
            .db
            .spawn_reader(move |conn| users::get_user_by_id(conn, &q_id))
            .await?;
        match u {
            Some(profile) => profile.role.parse(),
            None => {
                return Ok(json!({
                    "success": false,
                    "allowed": false,
                    "error": format!("User ID '{user_id_str}' not found")
                }));
            }
        }
    } else {
        return Ok(json!({
            "success": false,
            "allowed": false,
            "error": "Missing 'role', 'username', or 'user_id' in payload"
        }));
    };

    let role = match role_res {
        Ok(r) => r,
        Err(e) => {
            return Ok(json!({
                "success": false,
                "allowed": false,
                "error": e
            }));
        }
    };

    // 2. Evaluate discrete permission if provided
    if let Some(perm_str) = payload.get("permission").and_then(|v| v.as_str()) {
        let perm = match crate::authorization::Permission::from_str_loose(perm_str) {
            Ok(p) => p,
            Err(e) => {
                return Ok(json!({
                    "success": false,
                    "allowed": false,
                    "role": role.as_str(),
                    "error": e
                }));
            }
        };
        let allowed = crate::authorization::check_user_permission(role, perm);
        let reason = if allowed {
            None
        } else {
            Some(format!("Role {:?} lacks permission {:?}", role, perm))
        };
        return Ok(json!({
            "success": true,
            "allowed": allowed,
            "role": role.as_str(),
            "permission": perm.as_str(),
            "reason": reason
        }));
    }

    // 3. Evaluate discrete command if provided
    if let Some(cmd_str) = payload.get("command").and_then(|v| v.as_str()) {
        let check_res = crate::authorization::check_command_role_permission(role, cmd_str);
        let allowed = check_res.is_ok();
        return Ok(json!({
            "success": true,
            "allowed": allowed,
            "role": role.as_str(),
            "command": cmd_str,
            "reason": check_res.err()
        }));
    }

    // 4. Discovery mode: return full capability list for the role
    let granted_perms: Vec<String> = crate::authorization::get_role_permissions(role)
        .into_iter()
        .map(|p| p.as_str().to_string())
        .collect();

    Ok(json!({
        "success": true,
        "allowed": true,
        "role": role.as_str(),
        "permissions": granted_perms
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DatabasePool;

    #[tokio::test]
    async fn test_auth_commands_dispatch() {
        let db = DatabasePool::new_in_memory().unwrap();
        let stt_manager = crate::stt::SttManager::new(std::path::PathBuf::from("non_existent_dir"));
        let llm_manager = crate::llm::LlamaRouterManager::new(2048, 0).unwrap();
        let mock_capturer = Arc::new(crate::vision::capture::MockScreenCapturer::new(
            1920,
            1080,
            crate::vision::capture::PixelFormat::Rgba,
        ));
        let vision_manager = crate::vision::VisionManager::new(
            mock_capturer,
            crate::vision::VisionConfig::default(),
        );

        let state = Arc::new(AppState {
            db,
            crypto: crate::crypto::EncryptionEngine::new("00000000000000000000000000000000"),
            stt: tokio::sync::Mutex::new(stt_manager),
            tts: tokio::sync::Mutex::new(None),
            tts_player: crate::tts::audio::TtsAudioPlayer::new(None),
            llm: tokio::sync::Mutex::new(llm_manager),
            ai_queue: AppState::default_ai_queue(),
            vad: tokio::sync::Mutex::new(None),
            denoiser: tokio::sync::Mutex::new(None),
            turn_shadow: tokio::sync::Mutex::new(None),
            aec: tokio::sync::Mutex::new(None),
            mcp_server: Arc::new(crate::mcp::server::NativeMcpServer::new("test_vault")),
            embedder: AppState::empty_embedder(),
            vision: tokio::sync::Mutex::new(vision_manager),
            active_recall: Arc::new(crate::active_recall::ActiveRecallManager::new()),
        });

        // 1. Get quick accounts
        let res = handle(state.clone(), "auth:get_quick_accounts", json!({}))
            .await
            .unwrap();
        assert_eq!(res["success"], true);
        let accounts = res["accounts"].as_array().unwrap();
        assert_eq!(accounts.len(), 7);

        // 2. Login with valid credentials
        let login_res = handle(
            state.clone(),
            "auth:login",
            json!({
                "username": "maker_nam",
                "password": "LivaMaker@2026"
            }),
        )
        .await
        .unwrap();
        assert_eq!(login_res["success"], true);
        assert_eq!(login_res["user"]["username"], "maker_nam");
        assert_eq!(login_res["user"]["role"], "MAKER");

        // 3. Login with wrong password
        let wrong_pwd = handle(
            state.clone(),
            "auth:login",
            json!({
                "username": "maker_nam",
                "password": "WrongPassword"
            }),
        )
        .await
        .unwrap();
        assert_eq!(wrong_pwd["success"], false);
        assert!(
            wrong_pwd["error"]
                .as_str()
                .unwrap()
                .contains("không chính xác")
        );

        // 4. Quick login (one-click demo)
        let quick_login = handle(
            state.clone(),
            "auth:login",
            json!({
                "username": "cfo_hoang",
                "quick_login": true
            }),
        )
        .await
        .unwrap();
        assert_eq!(quick_login["success"], true);
        assert_eq!(quick_login["user"]["role"], "CFO");

        // 5. Create new user
        let create_res = handle(
            state.clone(),
            "auth:create_user",
            json!({
                "username": "test_checker",
                "password": "Password@123",
                "fullName": "Kiểm toán viên Mới",
                "role": "CHECKER",
                "department": "Ban Kiểm soát"
            }),
        )
        .await
        .unwrap();
        assert_eq!(create_res["success"], true);
        assert_eq!(create_res["user"]["username"], "test_checker");

        // 6. Login as new user
        let login_new = handle(
            state.clone(),
            "auth:login",
            json!({
                "username": "test_checker",
                "password": "Password@123"
            }),
        )
        .await
        .unwrap();
        assert_eq!(login_new["success"], true);
        assert_eq!(login_new["user"]["role"], "CHECKER");

        // 7. Check permission: Role + Permission (Maker -> IngestStatement: allowed; ApproveHitlMatch: denied)
        let perm_allowed = handle(
            state.clone(),
            "auth:check_permission",
            json!({
                "role": "MAKER",
                "permission": "IngestStatement"
            }),
        )
        .await
        .unwrap();
        assert_eq!(perm_allowed["success"], true);
        assert_eq!(perm_allowed["allowed"], true);

        let perm_denied = handle(
            state.clone(),
            "auth:check_permission",
            json!({
                "role": "MAKER",
                "permission": "ApproveHitlMatch"
            }),
        )
        .await
        .unwrap();
        assert_eq!(perm_denied["success"], true);
        assert_eq!(perm_denied["allowed"], false);

        // 8. Check permission: Role + Command (Checker -> reconciliation_resolve_hitl: allowed; statement_ingest_file: denied)
        let cmd_allowed = handle(
            state.clone(),
            "auth_check_permission",
            json!({
                "role": "CHECKER",
                "command": "reconciliation_resolve_hitl"
            }),
        )
        .await
        .unwrap();
        assert_eq!(cmd_allowed["success"], true);
        assert_eq!(cmd_allowed["allowed"], true);

        let cmd_denied = handle(
            state.clone(),
            "auth:check_permission",
            json!({
                "role": "CHECKER",
                "command": "statement_ingest_file"
            }),
        )
        .await
        .unwrap();
        assert_eq!(cmd_denied["success"], true);
        assert_eq!(cmd_denied["allowed"], false);

        // 9. Check permission via Username database resolution
        let user_check = handle(
            state.clone(),
            "auth:check_permission",
            json!({
                "username": "maker_nam",
                "permission": "RunReconciliation"
            }),
        )
        .await
        .unwrap();
        assert_eq!(user_check["success"], true);
        assert_eq!(user_check["allowed"], true);
        assert_eq!(user_check["role"], "MAKER");

        // 10. Capability discovery mode (neither permission nor command provided)
        let discovery = handle(
            state.clone(),
            "auth:check_permission",
            json!({
                "role": "CFO"
            }),
        )
        .await
        .unwrap();
        assert_eq!(discovery["success"], true);
        assert_eq!(discovery["allowed"], true);
        let perms = discovery["permissions"].as_array().unwrap();
        let perm_names: Vec<&str> = perms.iter().filter_map(|v| v.as_str()).collect();
        assert!(perm_names.contains(&"ViewCashflow30_90"));
        assert!(perm_names.contains(&"ApproveHighLimit"));
        assert!(perm_names.contains(&"ApprovePaymentOrder"));
        assert!(perm_names.contains(&"RejectPaymentOrder"));
        assert!(!perm_names.contains(&"ApproveHitlMatch"));
    }
}
