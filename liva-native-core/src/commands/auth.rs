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
    "auth_login",
    "auth_get_quick_accounts",
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
        .spawn_reader(|conn| users::list_quick_accounts(conn))
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
    }
}
