//! Integration tests for Compile-Time & Release Guard for Default Encryption Key (Debt C3).
//! Verifies:
//! 1. `check_release_key_guard` contract (rejects DEFAULT_ENCRYPTION_KEY in release mode, allows in debug, allows valid keys).
//! 2. `EncryptionEngine::new_rescue` contract (preserves legacy decryption/migration on DEFAULT_ENCRYPTION_KEY in all modes).
//! 3. `EncryptionEngine::new` debug vs release mode behavior.
//! 4. `resolve_and_rekey` in-memory ephemeral random key generation in release mode vs default key retention in debug.

use liva_native_core::crypto::{DEFAULT_ENCRYPTION_KEY, EncryptionEngine};
use liva_native_core::db::DatabasePool;
use liva_native_core::resolve_and_rekey;

#[test]
fn test_check_release_key_guard_contract() {
    // Contract 1: Release mode (is_release = true) + DEFAULT_ENCRYPTION_KEY -> Err
    let release_default = EncryptionEngine::check_release_key_guard(DEFAULT_ENCRYPTION_KEY, true);
    assert_eq!(
        release_default,
        Err("DEFAULT_ENCRYPTION_KEY is strictly forbidden in release builds")
    );

    // Contract 2: Debug mode (is_release = false) + DEFAULT_ENCRYPTION_KEY -> Ok
    let debug_default = EncryptionEngine::check_release_key_guard(DEFAULT_ENCRYPTION_KEY, false);
    assert!(debug_default.is_ok());

    // Contract 3: Custom secure key in release mode -> Ok
    let custom_key = "secure-random-passphrase-32-byte";
    assert!(EncryptionEngine::check_release_key_guard(custom_key, true).is_ok());

    // Contract 4: Custom secure key in debug mode -> Ok
    assert!(EncryptionEngine::check_release_key_guard(custom_key, false).is_ok());
}

#[test]
fn test_new_rescue_contract_preserves_default_key_across_modes() {
    // Contract: new_rescue MUST NOT panic on DEFAULT_ENCRYPTION_KEY in any build profile,
    // ensuring legacy migrations and rekeying operations succeed in production environments.
    let rescue_engine = EncryptionEngine::new_rescue(DEFAULT_ENCRYPTION_KEY);
    let secret = "legacy-bank-statement-record-from-dev";
    let encrypted = rescue_engine
        .encrypt(secret)
        .expect("new_rescue must be capable of encrypting");
    let decrypted = rescue_engine.decrypt_read(&encrypted);
    assert_eq!(decrypted, secret);
}

#[test]
#[cfg(debug_assertions)]
fn test_debug_mode_allows_default_key_construction_and_roundtrip() {
    // In debug mode, EncryptionEngine::new allows DEFAULT_ENCRYPTION_KEY with a warning
    // for developer convenience without crashing.
    let engine = EncryptionEngine::new(DEFAULT_ENCRYPTION_KEY);
    let sample = "developer-local-testing-secret";
    let encrypted = engine
        .encrypt(sample)
        .expect("debug mode allows default encryption key");
    assert!(encrypted.starts_with("v2:"));
    let decrypted = engine.decrypt_read(&encrypted);
    assert_eq!(decrypted, sample);
}

#[test]
#[cfg(not(debug_assertions))]
#[should_panic(expected = "CRITICAL SECURITY GUARD [Debt C3]")]
fn test_release_mode_panics_on_default_key() {
    // In release mode, EncryptionEngine::new strictly panics with security guard message
    let _engine = EncryptionEngine::new(DEFAULT_ENCRYPTION_KEY);
}

#[test]
fn test_in_memory_resolve_and_rekey_succeeds_without_crashing() {
    // In-memory resolve_and_rekey must succeed without crashing:
    // In debug mode: falls back to DEFAULT_ENCRYPTION_KEY cleanly
    // In release mode: generates ephemeral random 32-byte key without hitting release panic guard
    let db = DatabasePool::new_in_memory().expect("in-memory DatabasePool");
    let dummy_path = std::path::Path::new("in_memory_test.sqlite");
    let boot_key = resolve_and_rekey(&db, dummy_path, true).expect("resolve_and_rekey in_memory");

    assert_eq!(boot_key.source, "in-memory");
    assert!(boot_key.escrow_hex.is_none());

    // Verify engine operates correctly
    let sample = "transient-in-memory-session-state";
    let encrypted = boot_key
        .engine
        .encrypt(sample)
        .expect("in-memory engine encrypt");
    assert_eq!(boot_key.engine.decrypt_read(&encrypted), sample);
}
