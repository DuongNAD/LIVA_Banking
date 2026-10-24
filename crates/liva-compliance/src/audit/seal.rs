//! Cryptographic Configuration Sealing and Integrity Verification.
//!
//! Enforces tamper resistance for declarative policy files (AML, Maker-Checker).
//! Uses RFC 6962 Binary Merkle Tree and HMAC-SHA256 digital signatures to seal
//! canonical YAML configurations. Fails closed on any tampering.

use super::merkle::BinaryMerkleTree;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

/// Errors occurring during configuration sealing or verification.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ConfigSealError {
    #[error("Failed to parse YAML configuration: {0}")]
    YamlParseError(String),

    #[error("Failed to serialize canonical JSON: {0}")]
    JsonCanonicalError(String),

    #[error("Configuration Merkle seal mismatch: active config root {active} != sealed root {sealed}")]
    SealMismatch { active: String, sealed: String },

    #[error("Cryptographic HMAC signature verification failed for policy seal")]
    InvalidSignature,

    #[error("Segregation of Duties violation during sealing: maker ({maker}) == checker ({checker})")]
    MakerCheckerSodViolation { maker: String, checker: String },
}

/// Cryptographic seal record certifying an authorized policy configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigSealRecord {
    pub policy_id: String,
    pub policy_version: String,
    pub merkle_root_hex: String,
    pub leaf_count: usize,
    pub sealed_at: i64,
    pub maker_id: String,
    pub checker_id: String,
    pub seal_signature_hmac: String,
}

/// Computes the deterministic RFC 6962 Merkle tree and root for a YAML policy configuration string.
pub fn compute_config_merkle_tree(yaml_str: &str) -> Result<BinaryMerkleTree, ConfigSealError> {
    let parsed: serde_yaml::Value = serde_yaml::from_str(yaml_str)
        .map_err(|e| ConfigSealError::YamlParseError(e.to_string()))?;

    let json_val: serde_json::Value = serde_json::to_value(&parsed)
        .map_err(|e| ConfigSealError::JsonCanonicalError(e.to_string()))?;

    let leaves = extract_canonical_leaves(&json_val)?;
    Ok(BinaryMerkleTree::from_raw_leaves(&leaves))
}

/// Computes 32-byte Merkle root for a YAML configuration string.
pub fn compute_config_seal(yaml_str: &str) -> Result<[u8; 32], ConfigSealError> {
    let tree = compute_config_merkle_tree(yaml_str)?;
    Ok(tree.root())
}

/// Creates a certified `ConfigSealRecord` with dual-authorization (Maker-Checker) and HMAC signature.
pub fn create_config_seal(
    yaml_str: &str,
    policy_id: &str,
    policy_version: &str,
    maker_id: &str,
    checker_id: &str,
    hmac_key: &[u8],
    sealed_at: i64,
) -> Result<ConfigSealRecord, ConfigSealError> {
    // Segregation of duties for configuration sealing: maker != checker
    if maker_id == checker_id {
        return Err(ConfigSealError::MakerCheckerSodViolation {
            maker: maker_id.to_string(),
            checker: checker_id.to_string(),
        });
    }

    let tree = compute_config_merkle_tree(yaml_str)?;
    let root_hex = tree.root_hex();
    let leaf_count = tree.total_leaves();

    let sig = compute_seal_hmac(
        hmac_key,
        &root_hex,
        policy_id,
        policy_version,
        maker_id,
        checker_id,
        sealed_at,
    );

    Ok(ConfigSealRecord {
        policy_id: policy_id.to_string(),
        policy_version: policy_version.to_string(),
        merkle_root_hex: root_hex,
        leaf_count,
        sealed_at,
        maker_id: maker_id.to_string(),
        checker_id: checker_id.to_string(),
        seal_signature_hmac: hex::encode(sig),
    })
}

/// Verifies that an active YAML configuration string matches the certified seal record.
/// Fails closed if content has been modified or signature is forged.
pub fn verify_config_integrity(
    active_yaml: &str,
    seal: &ConfigSealRecord,
    hmac_key: &[u8],
) -> Result<bool, ConfigSealError> {
    // 1. Verify HMAC signature of seal record
    let expected_sig = compute_seal_hmac(
        hmac_key,
        &seal.merkle_root_hex,
        &seal.policy_id,
        &seal.policy_version,
        &seal.maker_id,
        &seal.checker_id,
        seal.sealed_at,
    );
    let expected_sig_hex = hex::encode(expected_sig);

    if !bool::from(
        expected_sig_hex
            .as_bytes()
            .ct_eq(seal.seal_signature_hmac.as_bytes()),
    ) {
        return Err(ConfigSealError::InvalidSignature);
    }

    // 2. Compute active Merkle root
    let active_tree = compute_config_merkle_tree(active_yaml)?;
    let active_root_hex = active_tree.root_hex();

    // 3. Constant-time comparison of active root vs sealed root
    if !bool::from(
        active_root_hex
            .as_bytes()
            .ct_eq(seal.merkle_root_hex.as_bytes()),
    ) {
        return Err(ConfigSealError::SealMismatch {
            active: active_root_hex,
            sealed: seal.merkle_root_hex.clone(),
        });
    }

    Ok(true)
}

fn compute_seal_hmac(
    key: &[u8],
    root_hex: &str,
    policy_id: &str,
    version: &str,
    maker: &str,
    checker: &str,
    sealed_at: i64,
) -> [u8; 32] {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
    mac.update(root_hex.as_bytes());
    mac.update(b"|");
    mac.update(policy_id.as_bytes());
    mac.update(b"|");
    mac.update(version.as_bytes());
    mac.update(b"|");
    mac.update(maker.as_bytes());
    mac.update(b"|");
    mac.update(checker.as_bytes());
    mac.update(b"|");
    mac.update(&sealed_at.to_be_bytes());
    mac.finalize().into_bytes().into()
}

/// Canonicalizes JSON value and produces sectional leaf payloads.
fn extract_canonical_leaves(val: &serde_json::Value) -> Result<Vec<Vec<u8>>, ConfigSealError> {
    let mut leaves = Vec::new();

    match val {
        serde_json::Value::Object(map) => {
            // Sort keys lexicographically to guarantee canonical ordering
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();

            for k in keys {
                let v = &map[k];
                let canonical_val = canonical_json_string(v);
                let leaf_payload = format!("{}:{}", k, canonical_val);
                leaves.push(leaf_payload.into_bytes());
            }
        }
        serde_json::Value::Array(arr) => {
            for (idx, item) in arr.iter().enumerate() {
                let canonical_item = canonical_json_string(item);
                let leaf_payload = format!("{}:{}", idx, canonical_item);
                leaves.push(leaf_payload.into_bytes());
            }
        }
        other => {
            leaves.push(canonical_json_string(other).into_bytes());
        }
    }

    if leaves.is_empty() {
        leaves.push(b"empty_config".to_vec());
    }

    Ok(leaves)
}

/// Recursively formats a JSON value into RFC 8785 Canonical JSON representation.
pub fn canonical_json_string(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => serde_json::to_string(s).unwrap_or_else(|_| s.clone()),
        serde_json::Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(canonical_json_string).collect();
            format!("[{}]", items.join(","))
        }
        serde_json::Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let entries: Vec<String> = keys
                .into_iter()
                .map(|k| {
                    format!(
                        "{}:{}",
                        serde_json::to_string(k).unwrap_or_else(|_| k.clone()),
                        canonical_json_string(&map[k])
                    )
                })
                .collect();
            format!("{{{}}}", entries.join(","))
        }
    }
}
