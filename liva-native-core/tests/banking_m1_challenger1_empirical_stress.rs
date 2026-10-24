use liva_native_core::{
    DEFAULT_EXPERT_MODEL, embedded_file_hash, embedded_model_hash, verify_model_artifact,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

fn temp_models_root() -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "liva_m1_challenger_{}_{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::SeqCst)
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn stress_test_path_traversal_rejection() {
    let root = temp_models_root();

    let traversal_payloads = [
        "../evil.gguf",
        "..\\evil.gguf",
        "nested/../../evil.gguf",
        "nested\\..\\..\\evil.gguf",
        "./../evil.gguf",
        ".\\..\\evil.gguf",
        "a/b/../../../evil.gguf",
    ];

    for payload in traversal_payloads {
        let candidate = Path::new(payload);
        let result = verify_model_artifact(&root, candidate);
        assert!(
            result.is_err(),
            "Path traversal candidate '{}' must be rejected",
            payload
        );
        let err = result.unwrap_err();
        assert!(
            err.contains("không được chứa '..'"),
            "Error for '{}' should mention '..', got: {}",
            payload,
            err
        );
    }

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn stress_test_invalid_extension_rejection() {
    let root = temp_models_root();

    let invalid_extensions = [
        "model.bin",
        "model.onnx",
        "model.exe",
        "model.txt",
        "model.dll",
        "model",
        "model.gguf.exe",
        "model.gguf.bat",
        "model.GGUF.old",
    ];

    for name in invalid_extensions {
        let file_path = root.join(name);
        fs::write(&file_path, b"fake payload").unwrap();
        let result = verify_model_artifact(&root, Path::new(name));
        assert!(
            result.is_err(),
            "Candidate with extension '{}' must be rejected",
            name
        );
        let err = result.unwrap_err();
        assert!(
            err.contains("phải là file .gguf"),
            "Error for '{}' should mention '.gguf', got: {}",
            name,
            err
        );
    }

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn stress_test_absolute_path_escape_rejection() {
    let root = temp_models_root();
    let outside = temp_models_root();

    let outside_file = outside.join("Qwen_Qwen3-14B-Q5_K_M.gguf");
    fs::write(&outside_file, b"unauthorized location").unwrap();

    let result = verify_model_artifact(&root, &outside_file);
    assert!(
        result.is_err(),
        "Absolute path outside models root must be rejected"
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("thoát khỏi canonical models root") || err.contains("thoát khỏi trust root"),
        "Error should indicate root escape, got: {}",
        err
    );

    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&outside);
}

#[test]
fn stress_test_unmanifested_model_rejection() {
    let root = temp_models_root();
    let unmanifested = "untrusted_arbitrary_neural_network.gguf";
    fs::write(root.join(unmanifested), b"malicious gguf weights").unwrap();

    let result = verify_model_artifact(&root, Path::new(unmanifested));
    assert!(
        result.is_err(),
        "Model not in manifest must be rejected unconditionally"
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("không có SHA-256 hợp lệ trong trust manifest"),
        "Error should indicate missing manifest hash, got: {}",
        err
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn stress_test_hash_mismatch_and_tampered_model_rejection() {
    let root = temp_models_root();
    let qwen_name = "Qwen_Qwen3-14B-Q5_K_M.gguf";

    // Tampered payload with wrong content
    fs::write(root.join(qwen_name), b"tampered qwen fake weights").unwrap();

    let result = verify_model_artifact(&root, Path::new(qwen_name));
    assert!(
        result.is_err(),
        "Tampered model with hash mismatch must fail validation"
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("SHA-256 artifact không khớp"),
        "Error should report hash mismatch, got: {}",
        err
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn stress_test_directory_named_gguf_rejection() {
    let root = temp_models_root();
    let fake_dir_model = "Qwen_Qwen3-14B-Q5_K_M.gguf";
    fs::create_dir_all(root.join(fake_dir_model)).unwrap();

    let result = verify_model_artifact(&root, Path::new(fake_dir_model));
    assert!(
        result.is_err(),
        "Directory masquerading as .gguf must be rejected"
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("không phải file"),
        "Error should indicate artifact is not a file, got: {}",
        err
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn stress_test_qwen3_14b_manifest_trust_anchor() {
    let expected_hash = "bf19bf5c77c530762012f71215b62aa8f0bae08111ee0d537b8496479e17cfa3";
    let model_name = "Qwen_Qwen3-14B-Q5_K_M.gguf";

    let hash_model = embedded_model_hash(model_name).expect("embedded_model_hash must resolve");
    assert_eq!(hash_model, expected_hash);

    let hash_file = embedded_file_hash(model_name).expect("embedded_file_hash must resolve");
    assert_eq!(hash_file, expected_hash);

    assert_eq!(DEFAULT_EXPERT_MODEL, model_name);
}

#[test]
fn stress_test_real_qwen3_14b_disk_artifact_if_present() {
    let expected_hash = "bf19bf5c77c530762012f71215b62aa8f0bae08111ee0d537b8496479e17cfa3";
    let model_name = "Qwen_Qwen3-14B-Q5_K_M.gguf";
    let real_dir = Path::new("E:\\AI_Models");
    let real_file = real_dir.join(model_name);

    if real_file.exists() {
        let meta = fs::metadata(&real_file).expect("Failed to read metadata for Qwen3 14B");
        assert_eq!(
            meta.len(),
            10514570272,
            "Real disk file size must match exactly 10,514,570,272 bytes"
        );

        let verified_path = verify_model_artifact(real_dir, Path::new(model_name))
            .expect("Real disk Qwen3 14B file must pass verify_model_artifact");
        assert_eq!(
            verified_path,
            real_file.canonicalize().unwrap(),
            "Verified path must match canonical path on disk"
        );

        let manifest_hash = embedded_model_hash(model_name).unwrap();
        assert_eq!(manifest_hash, expected_hash);
    } else {
        println!("Note: Real Qwen3 14B file not found on E:\\AI_Models (CI/headless mode).");
    }
}
