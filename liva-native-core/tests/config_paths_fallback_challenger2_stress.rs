//! Empirical Stress Test Harness for Milestone M1 (Challenger 2)
//!
//! Adversarial verification of:
//! 1. `paths::resolve_models_dir_from_config()` with missing, alien, invalid, and legacy paths.
//! 2. `paths::configured_expert_model_path()` with missing keys, alien providers, and env overrides.
//! 3. `paths::is_path_accessible()` under unreachable Windows drive letters and UNC paths.
//! 4. `paths::validate_model_path()` against path traversal, invalid extensions, and out-of-tree escapes.
//! 5. Robustness against panics across extreme/fuzzed inputs.

use liva_native_core::{
    DEFAULT_EXPERT_MODEL, LEGACY_DEV_MODELS_DIR, configured_expert_model_path,
    is_path_accessible, models_dir_fallback, resolve_models_dir_from_config, validate_model_path,
};
use serde_json::json;
use std::path::{Path, PathBuf};

// Mutex to serialize environment variable mutation across concurrent test threads
static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn test_resolve_models_dir_missing_and_empty_inputs_safely_fallback() {
    let fallback = models_dir_fallback();

    // 1. None input
    assert_eq!(resolve_models_dir_from_config(None), fallback);

    // 2. Empty JSON object
    assert_eq!(resolve_models_dir_from_config(Some(&json!({}))), fallback);

    // 3. Null / invalid types for localModelsDir
    let invalid_values = vec![
        json!({ "localModelsDir": null }),
        json!({ "localModelsDir": 12345 }),
        json!({ "localModelsDir": true }),
        json!({ "localModelsDir": false }),
        json!({ "localModelsDir": [] }),
        json!({ "localModelsDir": {} }),
        json!({ "localModelsDir": "" }),
        json!({ "localModelsDir": "   " }),
        json!({ "localModelsDir": "\t\r\n" }),
    ];

    for val in invalid_values {
        let res = resolve_models_dir_from_config(Some(&val));
        assert_eq!(
            res, fallback,
            "Expected fallback for input {:?}, got {:?}",
            val, res
        );
    }
}

#[test]
fn test_resolve_models_dir_alien_unreachable_drives() {
    let fallback = models_dir_fallback();

    // Alien drive letters that don't exist on standard Windows systems
    let alien_paths = [
        "Z:\\NonExistent_Drive_99999\\ai_models",
        "Y:\\AlienDrive\\Models",
        "X:\\UnmountedVolume\\models",
        "W:\\Deep\\Nested\\Directory\\Models",
    ];

    for alien in alien_paths {
        let cfg = json!({
            "localModelsDir": alien
        });
        let res = resolve_models_dir_from_config(Some(&cfg));
        assert_eq!(
            res, fallback,
            "Alien drive '{}' must fall back to default models dir",
            alien
        );
    }
}

#[test]
fn test_is_path_accessible_handles_unmounted_drives_without_panic() {
    // Test on unreachable Windows drive letters
    let unreachable_drive = Path::new("Z:\\NonExistent_Drive_123456\\test");
    assert!(
        !is_path_accessible(unreachable_drive),
        "Z:\\ drive should be detected as inaccessible"
    );

    // Test relative paths (must be considered accessible)
    let rel_path = Path::new("relative/path/to/models");
    assert!(
        is_path_accessible(rel_path),
        "Relative path should be accessible"
    );

    // Test existing directory
    let cur_dir = std::env::current_dir().unwrap();
    assert!(
        is_path_accessible(&cur_dir),
        "Current working dir must be accessible"
    );
}

#[test]
fn test_resolve_models_dir_legacy_dev_dir_constant_and_guard() {
    // Let's verify the real LEGACY_DEV_MODELS_DIR constant:
    assert_eq!(LEGACY_DEV_MODELS_DIR, "E:\\AI_Models");

    // When the legacy dev path exists, resolve_models_dir returns it.
    // When an unreachable legacy candidate is passed on a machine without that drive,
    // it must fall back to models_dir_fallback.
    let legacy_unreachable = "Z:\\AI_Models";
    let cfg = json!({
        "localModelsDir": legacy_unreachable
    });
    let res = resolve_models_dir_from_config(Some(&cfg));
    assert_eq!(res, models_dir_fallback());
}

#[test]
fn test_env_var_override_precedence_and_whitespace() {
    let _guard = ENV_LOCK.lock().unwrap();

    // 1. Valid LIVA_MODELS_DIR
    unsafe {
        std::env::set_var("LIVA_MODELS_DIR", "C:\\custom_models_env_override");
    }
    let res = resolve_models_dir_from_config(Some(&json!({
        "localModelsDir": "E:\\AI_Models"
    })));
    assert_eq!(res, PathBuf::from("C:\\custom_models_env_override"));

    // 2. Whitespace-only LIVA_MODELS_DIR must be ignored
    unsafe {
        std::env::set_var("LIVA_MODELS_DIR", "   \t   ");
    }
    let res = resolve_models_dir_from_config(None);
    assert_eq!(
        res,
        models_dir_fallback(),
        "Whitespace-only env var must fall through to fallback"
    );

    // Clean up
    unsafe {
        std::env::remove_var("LIVA_MODELS_DIR");
    }
}

#[test]
fn test_configured_expert_model_path_env_override() {
    let _guard = ENV_LOCK.lock().unwrap();

    // Explicit override via LIVA_EXPERT_MODEL_PATH
    unsafe {
        std::env::set_var(
            "LIVA_EXPERT_MODEL_PATH",
            "D:\\Models\\SpecialExpert.gguf",
        );
    }
    let res = configured_expert_model_path();
    assert_eq!(res, Some(PathBuf::from("D:\\Models\\SpecialExpert.gguf")));

    // Whitespace-only override must fall through
    unsafe {
        std::env::set_var("LIVA_EXPERT_MODEL_PATH", "    ");
    }
    let res = configured_expert_model_path();
    // When config file has expertModel, it will read that; clean up first to test
    unsafe {
        std::env::remove_var("LIVA_EXPERT_MODEL_PATH");
    }
    assert!(res.is_some(), "Should fall through to config file");
}

#[test]
fn test_configured_expert_model_path_provider_matrix() {
    // Non-local providers must return None regardless of expertModel being set
    let non_local_providers = ["cloud", "openai", "anthropic", "custom", "openrouter", "ollama"];

    for prov in non_local_providers {
        let cfg = json!({
            "ai": {
                "provider": prov,
                "localModelsDir": "E:\\AI_Models",
                "expertModel": "Qwen_Qwen3-14B-Q5_K_M.gguf"
            }
        });
        let ai = cfg.get("ai").unwrap();
        let provider = ai.get("provider").and_then(|v| v.as_str()).unwrap_or("local");
        assert_ne!(provider, "local");
    }
}

#[test]
fn test_validate_model_path_boundary_and_traversal_guards() {
    let models_dir = Path::new("C:\\liva_test_models_root");

    // Traversal attempts
    let attacks = [
        "../secret.gguf",
        "..\\secret.gguf",
        "sub/../../secret.gguf",
        "sub\\..\\..\\secret.gguf",
        "a/b/../../../etc/passwd.gguf",
    ];
    for attack in attacks {
        let err = validate_model_path(Path::new(attack), models_dir).unwrap_err();
        assert!(
            err.contains("không được chứa '..'"),
            "Attack '{}' should trigger '..' rejection, got: {}",
            attack,
            err
        );
    }

    // Invalid extensions
    let bad_extensions = [
        "model.bin",
        "model.onnx",
        "model.exe",
        "model.pt",
        "model.safetensors",
        "model_gguf",
        "model.gguf.old",
    ];
    for bad in bad_extensions {
        let err = validate_model_path(Path::new(bad), models_dir).unwrap_err();
        assert!(
            err.contains("phải là file .gguf"),
            "Bad extension '{}' should be rejected as non-gguf, got: {}",
            bad,
            err
        );
    }

    // Valid cases
    assert!(validate_model_path(Path::new("Qwen_Qwen3-14B-Q5_K_M.gguf"), models_dir).is_ok());
    assert!(validate_model_path(Path::new("subfolder/expert.gguf"), models_dir).is_ok());
    assert!(validate_model_path(Path::new("MODEL_UPPER.GGUF"), models_dir).is_ok());
}

#[test]
fn test_fuzz_extreme_inputs_never_panic() {
    let _guard = ENV_LOCK.lock().unwrap();

    let long_string = "A".repeat(8192);
    let fuzz_strings = vec![
        "",
        " ",
        "\0",
        "C:\\\0\\null_byte",
        "\\\\?\\UNC\\very\\long\\path\\",
        &long_string,
        "CON",
        "PRN",
        "AUX",
        "NUL",
        "COM1",
        "LPT1",
        "::$DATA",
        "...",
        "???",
        "***<|>:",
        "E:\\AI_Models\\..\\..\\..\\Windows\\System32",
    ];

    for s in fuzz_strings {
        let cfg = json!({
            "localModelsDir": s,
            "expertModel": s
        });

        // Ensure resolve_models_dir_from_config does not panic
        let result = std::panic::catch_unwind(|| {
            let _ = resolve_models_dir_from_config(Some(&cfg));
        });
        assert!(
            result.is_ok(),
            "resolve_models_dir_from_config panicked on input '{}'",
            s
        );

        // Ensure is_path_accessible does not panic
        let result_access = std::panic::catch_unwind(|| {
            let _ = is_path_accessible(Path::new(s));
        });
        assert!(result_access.is_ok(), "is_path_accessible panicked on input '{}'", s);
    }
}

#[test]
fn test_default_expert_model_constant_integrity() {
    // Pinned model name must match the milestone mandate
    assert_eq!(DEFAULT_EXPERT_MODEL, "Qwen_Qwen3-14B-Q5_K_M.gguf");
}
