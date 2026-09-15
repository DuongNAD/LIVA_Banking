//! Adversarial Stress & Empirical Challenge Test Suite (Challenger 2)
//!
//! Milestone 3: Native Core Reconciliation Engine & On-Premise Security.
//! Covers:
//! 1. Zero Network Egress: Loopback permission & strict rejection of public IPs, cloud URLs, wildcard bindings, and proxy vars.
//! 2. PII Redaction Boundary: Masking 8-digit & 16-digit accounts with prefixes, masking 12-digit CCCD,
//!    while strictly preserving standalone numeric amounts and narrations without account keywords.
//! 3. Audit Chain Tamper Detection: Detection of SQLite alterations (payload, record_hash, signature, timestamp, prev_hash) with exact seq_id pinpointing.

use liva_native_core::banking::compliance::{
    AuditLedger, genesis_hash, is_egress_permitted, sanitize_pii, verify_zero_egress,
    verify_zero_egress_from,
};
use rusqlite::Connection;

// ===========================================================================
// 1. Zero Network Egress Adversarial Stress Tests
// ===========================================================================

#[test]
fn test_challenger2_is_egress_permitted_strict_boundaries() {
    // A. Permitted Loopback Destinations (IPv4 127.0.0.0/8, IPv6 ::1, localhost)
    let valid_loopback_cases = [
        "127.0.0.1",
        "127.0.0.1:8002",
        "127.0.0.2",
        "127.255.255.254",
        "localhost",
        "LOCALHOST",
        "LocalHost",
        "localhost:3000",
        "::1",
        "[::1]",
        "[::1]:8080",
        "http://127.0.0.1:8002/v1/health",
        "https://localhost:8443/reconcile",
        "ws://[::1]:8002/ws",
        "tcp://127.0.0.1:9000",
        "http://admin:secret@localhost:8080/metrics",
        "http://user:pass@127.0.0.1:3000/api",
        "localhost:8080/path?query=1#frag",
        "127.0.0.1/status",
    ];
    for dest in &valid_loopback_cases {
        assert!(
            is_egress_permitted(dest),
            "Expected loopback destination '{dest}' to be PERMITTED"
        );
    }

    // B. Prohibited Public IPs, Cloud URLs, Wildcard Bindings, and Spoofing Attacks
    let prohibited_cases = [
        // Public IPs & DNS
        "8.8.8.8",
        "8.8.8.8:53",
        "8.8.4.4",
        "1.1.1.1:443",
        "142.250.190.46",
        // Wildcard / Unspecified Bindings (must never be permitted as isolated egress)
        "0.0.0.0",
        "0.0.0.0:8002",
        "0.0.0.0:80",
        // Private LAN (non-loopback)
        "192.168.1.1",
        "192.168.1.100:8080",
        "10.0.0.1",
        "172.16.0.1",
        // Cloud & External URLs
        "api.openai.com",
        "https://api.openai.com/v1",
        "https://api.openai.com/v1/chat/completions",
        "aws.com",
        "http://aws.com:80",
        "https://s3.amazonaws.com",
        "https://azure.microsoft.com",
        "https://cloud.google.com",
        // Domain Spoofing / Subdomain Evasion Attacks
        "localhost.evil.com",
        "http://localhost.attacker.io:8080",
        "127.0.0.1.attacker.com",
        "http://evil.com?localhost",
        "http://evil.com#127.0.0.1",
        "http://127.0.0.1:80@attacker.com",
        "http://user:pass@evil.com:8000/leak",
        // Empty or whitespace
        "",
        "   ",
        "\t\n",
    ];
    for dest in &prohibited_cases {
        assert!(
            !is_egress_permitted(dest),
            "Expected non-loopback destination '{dest}' to be STRICTLY REJECTED"
        );
    }
}

#[test]
fn test_challenger2_verify_zero_egress_environment_matrix() {
    // 1. Clean Loopback Environment -> Must be isolated
    let (isolated_clean, listeners_clean) = verify_zero_egress_from(|var| match var {
        "LIVA_SERVER_HOST" => Some("127.0.0.1".to_string()),
        "LIVA_SERVER_PORT" => Some("8002".to_string()),
        _ => None,
    });
    assert!(
        isolated_clean,
        "Clean 127.0.0.1 environment must pass zero egress"
    );
    assert!(listeners_clean.iter().all(|l| is_egress_permitted(l)));

    // 2. Wildcard 0.0.0.0 Binding -> Must fail isolation
    let (isolated_wildcard, _) = verify_zero_egress_from(|var| match var {
        "LIVA_SERVER_HOST" => Some("0.0.0.0".to_string()),
        _ => None,
    });
    assert!(
        !isolated_wildcard,
        "0.0.0.0 server host must invalidate isolation"
    );

    // 3. Corporate HTTP_PROXY set -> Must fail isolation
    let (isolated_proxy, _) = verify_zero_egress_from(|var| match var {
        "HTTP_PROXY" => Some("http://proxy.bank.corp:8080".to_string()),
        _ => None,
    });
    assert!(!isolated_proxy, "HTTP_PROXY must invalidate isolation");

    // 4. HTTPS_PROXY set -> Must fail isolation
    let (isolated_https_proxy, _) = verify_zero_egress_from(|var| match var {
        "HTTPS_PROXY" => Some("https://10.0.0.254:3128".to_string()),
        _ => None,
    });
    assert!(
        !isolated_https_proxy,
        "HTTPS_PROXY must invalidate isolation"
    );

    // 5. Cloud LLM Endpoint OPENAI_API_BASE set -> Must fail isolation
    let (isolated_openai, _) = verify_zero_egress_from(|var| match var {
        "OPENAI_API_BASE" => Some("https://api.openai.com/v1".to_string()),
        _ => None,
    });
    assert!(
        !isolated_openai,
        "OPENAI_API_BASE must invalidate isolation"
    );

    // 6. Cloud Storage AWS_ENDPOINT_URL set -> Must fail isolation
    let (isolated_aws, _) = verify_zero_egress_from(|var| match var {
        "AWS_ENDPOINT_URL" => Some("https://s3.ap-southeast-1.amazonaws.com".to_string()),
        _ => None,
    });
    assert!(!isolated_aws, "AWS_ENDPOINT_URL must invalidate isolation");

    // 7. Generic external HOST set -> Must fail isolation
    let (isolated_host, _) = verify_zero_egress_from(|var| match var {
        "HOST" => Some("8.8.8.8".to_string()),
        _ => None,
    });
    assert!(
        !isolated_host,
        "External generic HOST must invalidate isolation"
    );

    // 8. In-process standalone call
    let (isolated_default, listeners_default) = verify_zero_egress();
    assert!(isolated_default, "Default in-process must be isolated");
    assert!(!listeners_default.is_empty());
}

// ===========================================================================
// 2. PII Redaction Boundary Adversarial Stress Tests
// ===========================================================================

#[test]
fn test_challenger2_pii_account_boundaries_and_prefixes() {
    // 8-digit accounts with all supported prefix variations
    let prefixes = [
        "TK: ",
        "tk ",
        "TK ",
        "STK: ",
        "stk ",
        "STK ",
        "so tk: ",
        "số tk: ",
        "SO TK: ",
        "SỐ TK: ",
        "so tai khoan: ",
        "số tài khoản: ",
        "SO TAI KHOAN: ",
        "account: ",
        "Account: ",
        "acc: ",
        "ACC: ",
        "acc ",
    ];
    for pfx in &prefixes {
        let text_8digit = format!("Chuyen tien toi {pfx}12345678 tai VPBank");
        let sanitized_8digit = sanitize_pii(&text_8digit);
        assert!(
            !sanitized_8digit.contains("12345678"),
            "Failed to redact 8-digit account with prefix '{pfx}': got '{sanitized_8digit}'"
        );
        assert!(
            sanitized_8digit.contains("[REDACTED_ACCOUNT]"),
            "Expected [REDACTED_ACCOUNT] for prefix '{pfx}': got '{sanitized_8digit}'"
        );
    }

    // 16-digit accounts with prefixes
    for pfx in &["TK: ", "STK: ", "account: ", "so tk: "] {
        let text_16digit = format!("Thanh toan den {pfx}1234567890123456 ngan hang Vietcombank");
        let sanitized_16digit = sanitize_pii(&text_16digit);
        assert!(
            !sanitized_16digit.contains("1234567890123456"),
            "Failed to redact 16-digit account with prefix '{pfx}'"
        );
        assert!(sanitized_16digit.contains("[REDACTED_ACCOUNT]"));
    }

    // Intermediate account lengths (10-digit, 13-digit, 14-digit)
    let text_13 = "Tai khoan huong thu: TK: 0011009876543 tai VCB";
    assert!(!sanitize_pii(text_13).contains("0011009876543"));
    assert!(sanitize_pii(text_13).contains("[REDACTED_ACCOUNT]"));

    let text_14 = "Chuyen den STK 19034567890123 tai Techcombank";
    assert!(!sanitize_pii(text_14).contains("19034567890123"));
    assert!(sanitize_pii(text_14).contains("[REDACTED_ACCOUNT]"));
}

#[test]
fn test_challenger2_pii_preserves_standalone_amounts_and_narrations() {
    // 1. Transaction amounts without account keywords must NOT be redacted
    let amount_cases = [
        "So du hien tai: 1450230000 VND tai Vietcombank",
        "Giao dich chuyen tien so tien 15000000 thanh cong",
        "Thu chi ngay 15/08: 785600000 VND",
        "So tien chenh lech: 380000 VND",
        "Tong tai san luu dong: 100000000000 VND",
        "Thanh toan 50000000 dong",
        "So tien: 25000000",
    ];
    for case in &amount_cases {
        let result = sanitize_pii(case);
        assert!(
            !result.contains("[REDACTED_ACCOUNT]"),
            "Amount falsely redacted as account in '{case}': got '{result}'"
        );
    }

    // 2. Realistic banking narrations without account keywords
    let narration_cases = [
        "CONG TY TNHH THEP HOA PHAT THANH TOAN TIEN HANG HD 100200300",
        "Thanh toan tien dien thang 08 hop dong so 987654321",
        "Chuyen tien luong thang 8 nam 2026 cho 150 nhan vien",
        "Phi dich vu Napas 5500 cho giao dich FT262568912345",
    ];
    for narr in &narration_cases {
        let result = sanitize_pii(narr);
        assert!(
            !result.contains("[REDACTED_ACCOUNT]"),
            "Narration corrupted by false account redaction in '{narr}': got '{result}'"
        );
    }
}

#[test]
fn test_challenger2_pii_masks_12digit_cccd_and_secrets() {
    // 1. Vietnamese 12-digit CCCD starting with 0
    let cccd_cases = [
        "Khach hang CCCD: 001095012345 yeu cau sao ke",
        "So can cuoc cong dan 079201004567 da duoc xac minh",
        "Dinh danh: 038096001234",
    ];
    for cccd in &cccd_cases {
        let result = sanitize_pii(cccd);
        assert!(
            result.contains("[REDACTED_CCCD]"),
            "Expected [REDACTED_CCCD] in '{cccd}': got '{result}'"
        );
    }

    // 2. Secret tokens and OpenAI API keys
    let secret1 = "Authorization: Bearer sk-antigravitylivekey9876543210abcdef";
    let res_sec1 = sanitize_pii(secret1);
    assert!(!res_sec1.contains("sk-antigravitylivekey9876543210abcdef"));
    assert!(res_sec1.contains("[REDACTED_API_KEY]") || res_sec1.contains("[REDACTED_SECRET]"));

    let secret2 = "password: super_secret_bank_db_password_12345";
    let res_sec2 = sanitize_pii(secret2);
    assert!(!res_sec2.contains("super_secret_bank_db_password_12345"));
    assert!(res_sec2.contains("[REDACTED_SECRET]"));
}

// ===========================================================================
// 3. Audit Chain Tamper Detection Adversarial Tests
// ===========================================================================

fn setup_challenger_audit_db() -> Connection {
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
    conn
}

#[test]
fn test_challenger2_audit_chain_tamper_detection_matrix() {
    let conn = setup_challenger_audit_db();
    let key = b"adversarial_audit_key_32_bytes!";

    // Insert 5 forward-chained records
    let h1 = AuditLedger::append(
        &conn,
        key,
        "STATEMENT_IMPORTED",
        "TauriDashboard",
        "vcb_stmt.xlsx",
    )
    .unwrap();
    let h2 = AuditLedger::append(
        &conn,
        key,
        "MATCH_AUTO_TIER1",
        "ReconciliationEngine",
        "batch_1",
    )
    .unwrap();
    let h3 = AuditLedger::append(
        &conn,
        key,
        "HITL_CONFIRMED",
        "ChiefAccountant",
        "uuid_token_confirm",
    )
    .unwrap();
    let h4 = AuditLedger::append(
        &conn,
        key,
        "ERP_EXPORT",
        "TauriDashboard",
        "sap_journal.xml",
    )
    .unwrap();
    let h5 = AuditLedger::append(
        &conn,
        key,
        "COMPLIANCE_CHECK",
        "AuditDaemon",
        "decree_13_pass",
    )
    .unwrap();

    assert_ne!(h1, h2);
    assert_ne!(h2, h3);
    assert_ne!(h3, h4);
    assert_ne!(h4, h5);

    // Initial clean verification
    let initial_report = AuditLedger::verify(&conn, key).unwrap();
    assert!(
        initial_report.is_intact,
        "Initial 5-block chain must be intact"
    );
    assert_eq!(initial_report.total_records, 5);
    assert_eq!(initial_report.latest_hash, h5);
    assert_eq!(initial_report.genesis_hash, genesis_hash());
    assert!(initial_report.tampered_seq_id.is_none());

    // --- Tamper Scenario A: Modify payload_digest on seq_id 3 ---
    conn.execute(
        "UPDATE banking_audit_chain SET payload_digest = 'forged_malicious_digest' WHERE seq_id = 3",
        [],
    )
    .unwrap();
    let rep_a = AuditLedger::verify(&conn, key).unwrap();
    assert!(!rep_a.is_intact, "Payload tampering must fail verification");
    assert_eq!(
        rep_a.tampered_seq_id,
        Some(3),
        "Must pinpoint tampering at seq_id 3"
    );

    // Restore seq_id 3
    let mut p3_hasher = sha2::Sha256::default();
    use sha2::Digest;
    p3_hasher.update(b"uuid_token_confirm");
    let orig_p3_digest = hex::encode(p3_hasher.finalize());
    conn.execute(
        "UPDATE banking_audit_chain SET payload_digest = ?1 WHERE seq_id = 3",
        [&orig_p3_digest],
    )
    .unwrap();
    assert!(
        AuditLedger::verify(&conn, key).unwrap().is_intact,
        "Must be intact after restoration"
    );

    // --- Tamper Scenario B: Modify actor_principal on seq_id 2 ---
    conn.execute(
        "UPDATE banking_audit_chain SET actor_principal = 'HackerPrincipal' WHERE seq_id = 2",
        [],
    )
    .unwrap();
    let rep_b = AuditLedger::verify(&conn, key).unwrap();
    assert!(!rep_b.is_intact, "Actor tampering must fail verification");
    assert_eq!(
        rep_b.tampered_seq_id,
        Some(2),
        "Must pinpoint actor tampering at seq_id 2"
    );

    // Restore seq_id 2
    conn.execute(
        "UPDATE banking_audit_chain SET actor_principal = 'ReconciliationEngine' WHERE seq_id = 2",
        [],
    )
    .unwrap();
    assert!(AuditLedger::verify(&conn, key).unwrap().is_intact);

    // --- Tamper Scenario C: Modify signature on seq_id 4 ---
    conn.execute(
        "UPDATE banking_audit_chain SET signature = 'forged_fake_signature_hash_hex' WHERE seq_id = 4",
        [],
    )
    .unwrap();
    let rep_c = AuditLedger::verify(&conn, key).unwrap();
    assert!(
        !rep_c.is_intact,
        "Signature tampering must fail verification"
    );
    assert_eq!(
        rep_c.tampered_seq_id,
        Some(4),
        "Must pinpoint signature tampering at seq_id 4"
    );

    // Restore seq_id 4
    conn.execute(
        "UPDATE banking_audit_chain SET signature = record_hash WHERE seq_id = 4",
        [],
    )
    .unwrap();
    assert!(AuditLedger::verify(&conn, key).unwrap().is_intact);

    // --- Tamper Scenario D: Break prev_hash linkage on seq_id 5 ---
    conn.execute(
        "UPDATE banking_audit_chain SET prev_hash = 'broken_prev_hash_link' WHERE seq_id = 5",
        [],
    )
    .unwrap();
    let rep_d = AuditLedger::verify(&conn, key).unwrap();
    assert!(!rep_d.is_intact, "Linkage break must fail verification");
    assert_eq!(
        rep_d.tampered_seq_id,
        Some(5),
        "Must pinpoint broken chain link at seq_id 5"
    );

    // Restore seq_id 5
    conn.execute(
        "UPDATE banking_audit_chain SET prev_hash = ?1 WHERE seq_id = 5",
        [&h4],
    )
    .unwrap();
    assert!(AuditLedger::verify(&conn, key).unwrap().is_intact);

    // --- Tamper Scenario E: Modify timestamp on seq_id 1 ---
    conn.execute(
        "UPDATE banking_audit_chain SET timestamp = timestamp + 100 WHERE seq_id = 1",
        [],
    )
    .unwrap();
    let rep_e = AuditLedger::verify(&conn, key).unwrap();
    assert!(
        !rep_e.is_intact,
        "Timestamp modification must fail verification"
    );
    assert_eq!(
        rep_e.tampered_seq_id,
        Some(1),
        "Must pinpoint timestamp modification at seq_id 1"
    );
}
