//! Milestone: Security Sanitization & Machine Paths Generalization
//! Empirical Adversarial Stress Test Suite — Challenger 2
//!
//! Adversarial verification of:
//! 1. Machine Paths & Unmounted Drives:
//!    - Behavior of `is_path_accessible` on unmounted drives, invalid prefixes, rootless paths, UNC, and device paths.
//!    - Graceful fallback in `resolve_models_dir_from_config` without Windows OS Error 3 panic.
//!    - Precedence and edge cases of `LIVA_MODELS_DIR`.
//!    - Security boundaries in `validate_model_path`.
//! 2. WebSocket Origin Security:
//!    - Strict default allowlist (rejects arbitrary external origins and old hardcoded personal ngrok domains).
//!    - Origin spoofing resistance: rejection of subdomain and suffix attacks.
//!    - Scheme spoofing resistance: rejection of non-web schemes (`ftp://`, `javascript://`, `file://`, etc.).
//!    - JSON array format parsing and malformed recovery for `LIVA_WS_ALLOWED_ORIGINS`.
//!    - CSV edge cases (redundant commas, whitespace, empty entries).

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use serde_json::json;

use liva_native_core::{
    DEFAULT_WS_ALLOWED_ORIGINS,
    is_path_accessible,
    models_dir_fallback,
    origin_allowed,
    resolve_models_dir_from_config,
    validate_model_path,
};

// Global lock to serialize tests mutating environment variables
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn with_env_vars<F: FnOnce()>(vars: &[(&str, Option<&str>)], f: F) {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut saved = Vec::new();
    for &(name, new_val) in vars {
        let old_val = std::env::var(name).ok();
        saved.push((name, old_val));
        match new_val {
            Some(v) => unsafe { std::env::set_var(name, v) },
            None => unsafe { std::env::remove_var(name) },
        }
    }

    f();

    for (name, old_val) in saved {
        match old_val {
            Some(v) => unsafe { std::env::set_var(name, v) },
            None => unsafe { std::env::remove_var(name) },
        }
    }
}

// ===========================================================================
// Section 1: Machine Paths & Unmounted Drives Stress Tests
// ===========================================================================

#[test]
fn test_is_path_accessible_unmounted_drives_never_panics() {
    // Test on all potentially unmounted Windows drive letters
    let letters = ["Z", "Y", "X", "W", "V", "U", "T", "S", "R", "Q"];
    for letter in letters {
        let root = format!("{}:\\", letter);
        let is_mounted = Path::new(&root).exists();
        let test_path_str = format!("{}:\\NonExistent_Dir_987654\\models\\llm", letter);
        let test_path = Path::new(&test_path_str);

        // Crucial invariant: MUST NOT PANIC with OS Error 3
        let accessible = is_path_accessible(test_path);

        if !is_mounted {
            assert!(
                !accessible,
                "Drive {} is unmounted; is_path_accessible must return false",
                letter
            );
        }
    }
}

#[test]
fn test_is_path_accessible_boundary_inputs_no_panic() {
    let test_cases: &[(&str, bool)] = &[
        ("", true), // empty path -> true (path.exists() is false, no prefix, returns true)
        ("relative/path/to/models", true),
        (".", true),
        ("..", true),
        ("../relative/path", true),
    ];

    for &(raw, _) in test_cases {
        let p = Path::new(raw);
        // Assert it executes without panic
        let _ = is_path_accessible(p);
    }
}

#[test]
fn test_resolve_models_dir_from_config_unmounted_drive_fallback() {
    with_env_vars(&[("LIVA_MODELS_DIR", None)], || {
        let unmounted_cfg = json!({
            "localModelsDir": "Z:\\Unmounted_Drive_NonExistent_12345\\models"
        });

        let resolved = resolve_models_dir_from_config(Some(&unmounted_cfg));
        let expected_fallback = models_dir_fallback();

        assert_eq!(
            resolved, expected_fallback,
            "Unmounted drive in localModelsDir must gracefully fallback to models_dir_fallback()"
        );
    });
}

#[test]
fn test_resolve_models_dir_from_config_null_and_malformed_structures() {
    with_env_vars(&[("LIVA_MODELS_DIR", None)], || {
        let fallback = models_dir_fallback();

        assert_eq!(resolve_models_dir_from_config(None), fallback);
        assert_eq!(resolve_models_dir_from_config(Some(&json!({}))), fallback);
        assert_eq!(
            resolve_models_dir_from_config(Some(&json!({"localModelsDir": null}))),
            fallback
        );
        assert_eq!(
            resolve_models_dir_from_config(Some(&json!({"localModelsDir": 12345}))),
            fallback
        );
        assert_eq!(
            resolve_models_dir_from_config(Some(&json!({"localModelsDir": ""}))),
            fallback
        );
        assert_eq!(
            resolve_models_dir_from_config(Some(&json!({"localModelsDir": "   "}))),
            fallback
        );
    });
}

#[test]
fn test_resolve_models_dir_env_precedence_and_trimming() {
    // 1. Env variable takes precedence over config
    with_env_vars(
        &[("LIVA_MODELS_DIR", Some("C:\\Custom_Models_Override_Dir"))],
        || {
            let cfg = json!({ "localModelsDir": "C:\\Ignored_Config_Dir" });
            let resolved = resolve_models_dir_from_config(Some(&cfg));
            assert_eq!(
                resolved,
                PathBuf::from("C:\\Custom_Models_Override_Dir"),
                "LIVA_MODELS_DIR env must override localModelsDir in config"
            );
        },
    );

    // 2. Whitespace-only env variable must fall through to config or fallback
    with_env_vars(&[("LIVA_MODELS_DIR", Some("   "))], || {
        let cfg = json!({ "localModelsDir": "Z:\\Unmounted_Drive_XYZ\\models" });
        let resolved = resolve_models_dir_from_config(Some(&cfg));
        assert_eq!(
            resolved,
            models_dir_fallback(),
            "Whitespace-only LIVA_MODELS_DIR must be ignored and fall back"
        );
    });
}

#[test]
fn test_validate_model_path_security_boundary_enforcement() {
    let models_dir = Path::new("C:\\Liva\\models\\llm");

    // Path traversal attacks must be rejected
    assert!(validate_model_path(Path::new("../secret.gguf"), models_dir).is_err());
    assert!(validate_model_path(Path::new("sub/../../secret.gguf"), models_dir).is_err());
    assert!(validate_model_path(Path::new("..\\secret.gguf"), models_dir).is_err());

    // Non-gguf file extensions must be rejected
    assert!(validate_model_path(Path::new("model.bin"), models_dir).is_err());
    assert!(validate_model_path(Path::new("model.safetensors"), models_dir).is_err());
    assert!(validate_model_path(Path::new("model.exe"), models_dir).is_err());
    assert!(validate_model_path(Path::new("model.GGUF.bak"), models_dir).is_err());
    assert!(validate_model_path(Path::new("model"), models_dir).is_err());

    // Valid gguf files under directory must pass
    assert!(validate_model_path(Path::new("qwen-14b.gguf"), models_dir).is_ok());
    assert!(validate_model_path(Path::new("qwen-14b.GGUF"), models_dir).is_ok());
    assert!(validate_model_path(Path::new("deep/nested/qwen.gguf"), models_dir).is_ok());
}

// ===========================================================================
// Section 2: WebSocket Origin Security Stress Tests
// ===========================================================================

#[test]
fn test_origin_allowed_default_state_rejects_personal_ngrok() {
    with_env_vars(&[("LIVA_WS_ALLOWED_ORIGINS", None)], || {
        // Legitimate default loopback / native origins
        for o in DEFAULT_WS_ALLOWED_ORIGINS {
            assert!(origin_allowed(Some(o)), "Default origin {} must be allowed", o);
        }
        assert!(origin_allowed(None), "None origin (native desktop / test client) must be allowed");

        // Old personal ngrok domain MUST be rejected across all schemes
        let old_ngrok = "rd_3JHcwHJK1iUTdwx1D22XRMeUA5K.ngrok-free.app";
        assert!(!origin_allowed(Some(&format!("https://{}", old_ngrok))));
        assert!(!origin_allowed(Some(&format!("http://{}", old_ngrok))));
        assert!(!origin_allowed(Some(&format!("wss://{}", old_ngrok))));
        assert!(!origin_allowed(Some(&format!("ws://{}", old_ngrok))));

        // Empty and whitespace origins must be rejected
        assert!(!origin_allowed(Some("")));
        assert!(!origin_allowed(Some("   ")));
        assert!(!origin_allowed(Some("\t")));
    });
}

#[test]
fn test_origin_allowed_spoofing_subdomain_and_suffix_rejection() {
    with_env_vars(
        &[("LIVA_WS_ALLOWED_ORIGINS", Some("my-tunnel.ngrok-free.app"))],
        || {
            // Authorized exact domain matching with standard web schemes
            assert!(origin_allowed(Some("https://my-tunnel.ngrok-free.app")));
            assert!(origin_allowed(Some("http://my-tunnel.ngrok-free.app")));
            assert!(origin_allowed(Some("wss://my-tunnel.ngrok-free.app")));
            assert!(origin_allowed(Some("ws://my-tunnel.ngrok-free.app")));

            // Attack Scenario 1: Subdomain prepending / prefix spoofing
            assert!(
                !origin_allowed(Some("https://evil-my-tunnel.ngrok-free.app")),
                "Prefix spoofing must be rejected"
            );
            assert!(
                !origin_allowed(Some("https://notmy-tunnel.ngrok-free.app")),
                "Prefix spoofing must be rejected"
            );
            assert!(
                !origin_allowed(Some("https://sub.my-tunnel.ngrok-free.app")),
                "Subdomain addition must be rejected"
            );

            // Attack Scenario 2: Suffix appending / attacker domain spoofing
            assert!(
                !origin_allowed(Some("https://my-tunnel.ngrok-free.app.attacker.com")),
                "Suffix attacker domain must be rejected"
            );
            assert!(
                !origin_allowed(Some("https://my-tunnel.ngrok-free.app:8080")),
                "Unregistered port suffix must be rejected"
            );
            assert!(
                !origin_allowed(Some("https://my-tunnel.ngrok-free.app.evil.org")),
                "Suffix attacker TLD must be rejected"
            );

            // Attack Scenario 3: Path embedding
            assert!(
                !origin_allowed(Some("https://attacker.com/my-tunnel.ngrok-free.app")),
                "Path embedded target must be rejected"
            );
        },
    );
}

#[test]
fn test_origin_allowed_scheme_spoofing_rejection() {
    with_env_vars(
        &[("LIVA_WS_ALLOWED_ORIGINS", Some("my-tunnel.ngrok-free.app"))],
        || {
            // Non-web schemes must be rejected even with matching host
            let forbidden_schemes = [
                "ftp://my-tunnel.ngrok-free.app",
                "javascript://my-tunnel.ngrok-free.app",
                "file://my-tunnel.ngrok-free.app",
                "data://my-tunnel.ngrok-free.app",
                "gopher://my-tunnel.ngrok-free.app",
                "blob://my-tunnel.ngrok-free.app",
                "mailto:my-tunnel.ngrok-free.app",
                "ssh://my-tunnel.ngrok-free.app",
            ];

            for bad in forbidden_schemes {
                assert!(
                    !origin_allowed(Some(bad)),
                    "Non-web scheme {} must be strictly rejected",
                    bad
                );
            }
        },
    );
}

#[test]
fn test_origin_allowed_json_array_parsing_and_recovery() {
    // 1. Valid JSON Array
    with_env_vars(
        &[(
            "LIVA_WS_ALLOWED_ORIGINS",
            Some(r#"["https://bank1.vn", "https://bank2.vn", "http://preview.bank.vn:4173"]"#),
        )],
        || {
            assert!(origin_allowed(Some("https://bank1.vn")));
            assert!(origin_allowed(Some("https://bank2.vn")));
            assert!(origin_allowed(Some("http://preview.bank.vn:4173")));

            // Unlisted origins
            assert!(!origin_allowed(Some("https://bank3.vn")));
            assert!(!origin_allowed(Some("https://bank1.vn.attacker.com")));
            assert!(!origin_allowed(Some("https://evil.bank1.vn")));

            // Scheme mismatch when configured with explicit scheme
            assert!(
                !origin_allowed(Some("http://bank1.vn")),
                "http://bank1.vn must be rejected when allowlist specifies https://bank1.vn"
            );
        },
    );

    // 2. Empty JSON Array
    with_env_vars(&[("LIVA_WS_ALLOWED_ORIGINS", Some("[]"))], || {
        assert!(!origin_allowed(Some("https://bank1.vn")));
        assert!(origin_allowed(Some("http://localhost:5173"))); // default still works
    });

    // 3. Malformed JSON Array (gracefully falls back to CSV without crash)
    with_env_vars(
        &[(
            "LIVA_WS_ALLOWED_ORIGINS",
            Some(r#"["https://bank1.vn", malformed json"#),
        )],
        || {
            // Serde parse fails, falls back to CSV split.
            // In CSV, entries are split by comma.
            // Neither causes panic.
            let _ = origin_allowed(Some("https://bank1.vn"));
            let _ = origin_allowed(Some("https://attacker.com"));
        },
    );
}

#[test]
fn test_origin_allowed_csv_delimiter_edge_cases() {
    // Redundant commas and excess whitespace
    with_env_vars(
        &[(
            "LIVA_WS_ALLOWED_ORIGINS",
            Some("  ,,  https://treasury.vcb.vn  , ,  https://bidv.com.vn  ,  "),
        )],
        || {
            assert!(origin_allowed(Some("https://treasury.vcb.vn")));
            assert!(origin_allowed(Some("https://bidv.com.vn")));
            assert!(!origin_allowed(Some("https://tcb.com.vn")));

            // Trailing / empty entries must never match empty or arbitrary origins
            assert!(!origin_allowed(Some("")));
            assert!(!origin_allowed(Some(" ")));
            assert!(!origin_allowed(Some("https://")));
        },
    );
}
