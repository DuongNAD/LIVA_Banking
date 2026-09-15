//! Database persistence and operations for multi-role banking users.
//!
//! Stores credentials securely using random salts and HKDF-SHA256 password hashes.

use hkdf::Hkdf;
use rand::RngCore;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRecord {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub salt: String,
    pub full_name: String,
    pub role: String,
    pub department: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPublicProfile {
    pub id: String,
    pub username: String,
    pub full_name: String,
    pub role: String,
    pub role_title: String,
    pub department: String,
    pub description: String,
    pub avatar_initials: String,
    pub avatar_color: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub full_name: String,
    pub role: String,
    pub department: Option<String>,
}

/// Generates a 16-byte cryptographically secure random salt in hex string format.
pub fn generate_salt() -> String {
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);
    hex::encode(salt)
}

/// Computes an HKDF-SHA256 password hash using the provided salt.
pub fn hash_password(password: &str, salt: &str) -> String {
    let hk = Hkdf::<Sha256>::new(Some(salt.as_bytes()), password.as_bytes());
    let mut okm = [0u8; 32];
    hk.expand(b"liva-auth-v1", &mut okm)
        .expect("32 bytes is valid length for HKDF-Sha256");
    hex::encode(okm)
}

/// Constant-time byte slice comparison to mitigate side-channel timing attacks on credential hashes.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (&x, &y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Verifies a password against the stored salt and hash using constant-time comparison.
pub fn verify_password(password: &str, salt: &str, expected_hash: &str) -> bool {
    let actual_hash = hash_password(password, salt);
    constant_time_eq(actual_hash.as_bytes(), expected_hash.as_bytes())
}

/// Format current UTC time into RFC3339 / ISO-8601 string without external chrono crate.
pub fn current_iso_timestamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format_timestamp_iso(secs)
}

fn format_timestamp_iso(secs: u64) -> String {
    let days = secs / 86400;
    let rem_secs = secs % 86400;
    let hours = rem_secs / 3600;
    let rem_secs2 = rem_secs % 3600;
    let minutes = rem_secs2 / 60;
    let seconds = rem_secs2 % 60;

    let z = days as i64 + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1024 + doe / 1461 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}T{hours:02}:{minutes:02}:{seconds:02}Z")
}

pub fn role_title(role: &str) -> &'static str {
    match role.trim().to_uppercase().as_str() {
        "MAKER" => "Kế toán viên (Maker)",
        "CHECKER" => "Kế toán trưởng (Checker)",
        "CFO" => "Giám đốc Tài chính (CFO)",
        "AUDITOR" => "Kiểm toán viên (Auditor)",
        "ADMIN" => "Quản trị viên Hệ thống (Admin)",
        _ => "Cán bộ Vận hành",
    }
}

pub fn role_description(role: &str) -> &'static str {
    match role.trim().to_uppercase().as_str() {
        "MAKER" => "Tải sao kê, lập đề xuất xử lý lệch, tạo lệnh chi tiền",
        "CHECKER" => "Phê duyệt ngoại lệ đối soát (4-Eyes HITL), duyệt chi",
        "CFO" => "Giám sát ngân quỹ 30/90 ngày, quản trị hạn mức thanh khoản",
        "AUDITOR" => "Quyền chỉ đọc, kiểm tra Merkle Audit Chain & Nghị định 13",
        "ADMIN" => "Cấu hình bảo mật Zero-Egress, AI offline, nhật ký vận hành",
        _ => "Thành viên hệ thống đối soát và quản lý ngân quỹ",
    }
}

pub fn avatar_color(role: &str) -> &'static str {
    match role.trim().to_uppercase().as_str() {
        "MAKER" => "#2563eb",
        "CHECKER" => "#059669",
        "CFO" => "#7c3aed",
        "AUDITOR" => "#d97706",
        "ADMIN" => "#dc2626",
        _ => "#0284c7",
    }
}

pub fn avatar_initials(full_name: &str, role: &str) -> String {
    if role.eq_ignore_ascii_case("ADMIN") {
        return "AD".to_string();
    }
    let parts: Vec<&str> = full_name.split_whitespace().collect();
    match parts.len() {
        0 => "LV".to_string(),
        1 => parts[0].chars().take(2).collect::<String>().to_uppercase(),
        _ => {
            let len = parts.len();
            let p1 = parts[len - 2].chars().next().unwrap_or('A');
            let p2 = parts[len - 1].chars().next().unwrap_or('B');
            format!("{p1}{p2}").to_uppercase()
        }
    }
}

pub fn seed_default_users_if_empty(conn: &Connection) -> Result<(), rusqlite::Error> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |r| r.get(0))?;
    if count > 0 {
        return Ok(());
    }

    let default_users = [
        (
            "KT_TRINH_VAN_NAM",
            "maker_nam",
            "LivaMaker@2026",
            "Trịnh Văn Nam",
            "MAKER",
            "Phòng Kế toán Vốn",
        ),
        (
            "KT_LE_PHUONG_MAI",
            "maker_mai",
            "LivaMaker@2026",
            "Lê Phương Mai",
            "MAKER",
            "Phòng Kế toán Thanh toán",
        ),
        (
            "KTT_NGUYEN_MINH_TRI",
            "checker_tri",
            "LivaChecker@2026",
            "Nguyễn Minh Trí",
            "CHECKER",
            "Ban Giám đốc Tài chính - Kế toán",
        ),
        (
            "KTT_DO_LAN_HUONG",
            "checker_huong",
            "LivaChecker@2026",
            "Đỗ Lan Hương",
            "CHECKER",
            "Ban Kiểm soát Kế toán",
        ),
        (
            "CFO_TRAN_VIET_HOANG",
            "cfo_hoang",
            "LivaCfo@2026",
            "Trần Việt Hoàng",
            "CFO",
            "Ban Điều hành C-Suite",
        ),
        (
            "AUDIT_PHAM_HUONG_LAN",
            "auditor_lan",
            "LivaAudit@2026",
            "Phạm Hương Lan",
            "AUDITOR",
            "Ban Kiểm toán & Tuân thủ",
        ),
        (
            "ADMIN_HE_THONG",
            "admin_sys",
            "LivaAdmin@2026",
            "Quản trị viên An ninh",
            "ADMIN",
            "Trung tâm An toàn Thông tin (SOC)",
        ),
    ];

    let now = current_iso_timestamp();
    for (id, username, password, full_name, role, department) in default_users {
        let salt = generate_salt();
        let hash = hash_password(password, &salt);
        conn.execute(
            "INSERT INTO users (id, username, password_hash, salt, full_name, role, department, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'active', ?8, ?8)",
            rusqlite::params![id, username, hash, salt, full_name, role, department, now],
        )?;
    }
    tracing::info!("Seeded default banking users into SQLite database");
    Ok(())
}

pub fn get_user_by_username(
    conn: &Connection,
    username: &str,
) -> Result<Option<UserRecord>, rusqlite::Error> {
    let trimmed = username.trim().to_lowercase();
    let mut stmt = conn.prepare(
        "SELECT id, username, password_hash, salt, full_name, role, department, status, created_at, updated_at
         FROM users
         WHERE LOWER(username) = ?1
         LIMIT 1",
    )?;
    let mut rows = stmt.query(rusqlite::params![trimmed])?;
    if let Some(row) = rows.next()? {
        Ok(Some(UserRecord {
            id: row.get(0)?,
            username: row.get(1)?,
            password_hash: row.get(2)?,
            salt: row.get(3)?,
            full_name: row.get(4)?,
            role: row.get(5)?,
            department: row.get(6)?,
            status: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn get_user_by_id(
    conn: &Connection,
    id: &str,
) -> Result<Option<UserRecord>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, username, password_hash, salt, full_name, role, department, status, created_at, updated_at
         FROM users
         WHERE id = ?1
         LIMIT 1",
    )?;
    let mut rows = stmt.query(rusqlite::params![id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(UserRecord {
            id: row.get(0)?,
            username: row.get(1)?,
            password_hash: row.get(2)?,
            salt: row.get(3)?,
            full_name: row.get(4)?,
            role: row.get(5)?,
            department: row.get(6)?,
            status: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn list_quick_accounts(conn: &Connection) -> Result<Vec<UserPublicProfile>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, username, full_name, role, department, status, created_at, updated_at
         FROM users
         WHERE status = 'active'
         ORDER BY rowid ASC",
    )?;
    let mut rows = stmt.query([])?;
    let mut accounts = Vec::new();
    while let Some(row) = rows.next()? {
        let id: String = row.get(0)?;
        let username: String = row.get(1)?;
        let full_name: String = row.get(2)?;
        let role: String = row.get(3)?;
        let department: Option<String> = row.get(4)?;
        let status: String = row.get(5)?;

        let profile = UserPublicProfile {
            id,
            username,
            full_name: full_name.clone(),
            role_title: role_title(&role).to_string(),
            department: department.unwrap_or_else(|| "Phòng Tài chính".to_string()),
            description: role_description(&role).to_string(),
            avatar_initials: avatar_initials(&full_name, &role),
            avatar_color: avatar_color(&role).to_string(),
            role,
            status,
        };
        accounts.push(profile);
    }
    Ok(accounts)
}

pub fn verify_user_login(
    conn: &Connection,
    username: &str,
    password: Option<&str>,
    quick_login: bool,
) -> Result<Result<UserPublicProfile, String>, rusqlite::Error> {
    let Some(user) = get_user_by_username(conn, username)? else {
        return Ok(Err("Tên đăng nhập không tồn tại trong hệ thống.".to_string()));
    };

    if user.status != "active" {
        return Ok(Err("Tài khoản đã bị khóa hoặc ngừng hoạt động.".to_string()));
    }

    if !quick_login {
        let Some(pwd) = password else {
            return Ok(Err("Vui lòng nhập mật khẩu.".to_string()));
        };
        if !verify_password(pwd, &user.salt, &user.password_hash) {
            return Ok(Err("Mật khẩu không chính xác.".to_string()));
        }
    }

    let profile = UserPublicProfile {
        id: user.id,
        username: user.username,
        full_name: user.full_name.clone(),
        role_title: role_title(&user.role).to_string(),
        department: user.department.unwrap_or_else(|| "Phòng Tài chính".to_string()),
        description: role_description(&user.role).to_string(),
        avatar_initials: avatar_initials(&user.full_name, &user.role),
        avatar_color: avatar_color(&user.role).to_string(),
        role: user.role,
        status: user.status,
    };

    Ok(Ok(profile))
}

pub fn create_user(
    conn: &Connection,
    new_user: &NewUser,
) -> Result<Result<UserPublicProfile, String>, rusqlite::Error> {
    let trimmed_user = new_user.username.trim().to_lowercase();
    if trimmed_user.is_empty() {
        return Ok(Err("Tên đăng nhập không được để trống.".to_string()));
    }
    if new_user.password.is_empty() {
        return Ok(Err("Mật khẩu không được để trống.".to_string()));
    }

    if let Some(_) = get_user_by_username(conn, &trimmed_user)? {
        return Ok(Err("Tên đăng nhập đã tồn tại.".to_string()));
    }

    let normalized_role = new_user.role.trim().to_uppercase();
    let role = if normalized_role.is_empty() {
        "MAKER".to_string()
    } else {
        normalized_role
    };

    let salt = generate_salt();
    let hash = hash_password(&new_user.password, &salt);
    let id = format!("USER_{}", uuid::Uuid::new_v4().simple()).to_uppercase();
    let now = current_iso_timestamp();

    conn.execute(
        "INSERT INTO users (id, username, password_hash, salt, full_name, role, department, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'active', ?8, ?8)",
        rusqlite::params![
            id,
            trimmed_user,
            hash,
            salt,
            new_user.full_name,
            role,
            new_user.department,
            now
        ],
    )?;

    let profile = UserPublicProfile {
        id,
        username: trimmed_user,
        full_name: new_user.full_name.clone(),
        role_title: role_title(&role).to_string(),
        department: new_user
            .department
            .clone()
            .unwrap_or_else(|| "Phòng Tài chính".to_string()),
        description: role_description(&role).to_string(),
        avatar_initials: avatar_initials(&new_user.full_name, &role),
        avatar_color: avatar_color(&role).to_string(),
        role,
        status: "active".to_string(),
    };

    Ok(Ok(profile))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash_and_verify() {
        let salt = generate_salt();
        let hash = hash_password("LivaMaker@2026", &salt);
        assert!(verify_password("LivaMaker@2026", &salt, &hash));
        assert!(!verify_password("WrongPassword", &salt, &hash));
    }

    #[test]
    fn test_user_seeding_and_auth() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::init_schemas(&conn).unwrap();

        // Check seeded users
        let accounts = list_quick_accounts(&conn).unwrap();
        assert_eq!(accounts.len(), 7);

        // Test login valid
        let login_res = verify_user_login(&conn, "maker_nam", Some("LivaMaker@2026"), false).unwrap();
        assert!(login_res.is_ok());
        let profile = login_res.unwrap();
        assert_eq!(profile.username, "maker_nam");
        assert_eq!(profile.role, "MAKER");

        // Test case-insensitive username login
        let ci_res = verify_user_login(&conn, "  MAKER_NAM  ", Some("LivaMaker@2026"), false).unwrap();
        assert!(ci_res.is_ok());

        // Test wrong password
        let wrong_pwd = verify_user_login(&conn, "maker_nam", Some("WrongPass"), false).unwrap();
        assert!(wrong_pwd.is_err());

        // Test unknown user
        let unknown = verify_user_login(&conn, "unknown_user", Some("AnyPass"), false).unwrap();
        assert!(unknown.is_err());

        // Test quick login (demo bypass)
        let quick = verify_user_login(&conn, "checker_tri", None, true).unwrap();
        assert!(quick.is_ok());
        assert_eq!(quick.unwrap().role, "CHECKER");

        // Test create new user
        let created = create_user(
            &conn,
            &NewUser {
                username: "new_auditor".to_string(),
                password: "SecurePass@2026".to_string(),
                full_name: "Hoàng Văn Kiểm".to_string(),
                role: "AUDITOR".to_string(),
                department: Some("Kiểm toán nội bộ".to_string()),
            },
        )
        .unwrap();
        assert!(created.is_ok());
        let new_profile = created.unwrap();
        assert_eq!(new_profile.username, "new_auditor");

        // Verify login with new user
        let login_new = verify_user_login(&conn, "new_auditor", Some("SecurePass@2026"), false).unwrap();
        assert!(login_new.is_ok());
    }
}
