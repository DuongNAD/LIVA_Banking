//! Cryptographic Merkle Audit Trail and Configuration Sealing Tests.

use liva_compliance::audit::{
    compute_config_seal, compute_merkle_root, create_config_seal, hash_leaf, hash_node,
    verify_config_integrity, BinaryMerkleTree, ConfigSealError, RFC6962_LEAF_PREFIX,
    RFC6962_NODE_PREFIX,
};

#[test]
fn test_rfc6962_domain_separation_invariants() {
    let data = b"sample_transaction_data";

    let leaf_hash = hash_leaf(data);
    let node_hash = hash_node(&leaf_hash, &leaf_hash);

    // Leaf hash cannot equal node hash due to domain prefixes 0x00 != 0x01
    assert_ne!(leaf_hash, node_hash);
    assert_eq!(RFC6962_LEAF_PREFIX, 0x00);
    assert_eq!(RFC6962_NODE_PREFIX, 0x01);
}

#[test]
fn test_rfc6962_inclusion_proof_and_sub_millisecond_verification() {
    let leaves: Vec<Vec<u8>> = (0..16)
        .map(|i| format!("TX-RECORD-LEAF-{}", i).into_bytes())
        .collect();

    let tree = BinaryMerkleTree::from_raw_leaves(&leaves);
    assert_eq!(tree.total_leaves(), 16);
    assert_eq!(tree.root_hex().len(), 64);

    for (idx, leaf) in leaves.iter().enumerate() {
        let proof = tree
            .generate_inclusion_proof(idx)
            .expect("Must generate inclusion proof");
        assert_eq!(proof.leaf_index, idx);
        assert_eq!(proof.total_leaves, 16);

        // Measure verification duration
        let start = std::time::Instant::now();
        let valid = proof.verify(leaf);
        let elapsed = start.elapsed();

        assert!(valid, "Inclusion proof must verify against true leaf data");
        assert!(
            elapsed.as_millis() < 1,
            "Verification must execute in < 1 millisecond"
        );

        // Forged leaf data must fail verification
        let forged_leaf = format!("FORGED-TX-{}", idx).into_bytes();
        assert!(!proof.verify(&forged_leaf));
    }
}

#[test]
fn test_configuration_sealing_and_tamper_detection_fail_closed() {
    let original_yaml = include_str!("../config/aml_policy_default.yaml");
    let hmac_key = b"SUPER_SECRET_AUDIT_SECURITY_KEY_256BIT";
    let now = 1726300000;

    // 1. Create valid dual-authorized seal
    let seal = create_config_seal(
        original_yaml,
        "AML_POLICY_VN",
        "1.0.0",
        "COMPLIANCE_MAKER",
        "CHIEF_COMPLIANCE_OFFICER",
        hmac_key,
        now,
    )
    .expect("Must create valid config seal");

    assert_eq!(seal.policy_id, "AML_POLICY_VN");
    assert_eq!(seal.merkle_root_hex.len(), 64);
    assert!(!seal.seal_signature_hmac.is_empty());

    let root_bytes = compute_config_seal(original_yaml).expect("Must compute config seal");
    assert_eq!(hex::encode(root_bytes), seal.merkle_root_hex);

    // 2. Verify unmodified active config -> Must succeed
    let is_valid = verify_config_integrity(original_yaml, &seal, hmac_key)
        .expect("Verification must succeed on intact config");
    assert!(is_valid);

    // 3. TAMPER SCENARIO: Adversary modifies threshold from 400,000,000 to 900,000,000
    let tampered_yaml = original_yaml.replace("400000000", "900000000");
    assert_ne!(original_yaml, tampered_yaml);

    let tamper_result = verify_config_integrity(&tampered_yaml, &seal, hmac_key);
    assert!(
        matches!(tamper_result, Err(ConfigSealError::SealMismatch { .. })),
        "Tampered YAML must trigger SealMismatch and fail closed"
    );

    // 4. SIGNATURE FORGERY SCENARIO: Adversary alters signature in seal record
    let mut forged_seal = seal.clone();
    forged_seal.seal_signature_hmac = "0000111122223333444455556666777788889999aaaabbbbccccddddeeeeffff".to_string();

    let forgery_result = verify_config_integrity(original_yaml, &forged_seal, hmac_key);
    assert!(
        matches!(forgery_result, Err(ConfigSealError::InvalidSignature)),
        "Forged signature must trigger InvalidSignature"
    );

    // 5. MAKER == CHECKER SOD VIOLATION AT SEALING
    let sod_err = create_config_seal(
        original_yaml,
        "AML_POLICY_VN",
        "1.0.0",
        "SAME_OFFICER",
        "SAME_OFFICER", // Maker == Checker
        hmac_key,
        now,
    );
    assert!(matches!(
        sod_err,
        Err(ConfigSealError::MakerCheckerSodViolation { .. })
    ));
}

#[test]
fn test_backwards_compatible_compute_merkle_root() {
    let hashes = vec![
        "0x1111111111111111".to_string(),
        "0x2222222222222222".to_string(),
        "0x3333333333333333".to_string(),
    ];
    let root1 = compute_merkle_root(&hashes);
    assert!(root1.starts_with("0x"));

    let hashes_tampered = vec![
        "0x1111111111111111".to_string(),
        "0x2222222222222223".to_string(), // 1 bit flip
        "0x3333333333333333".to_string(),
    ];
    let root2 = compute_merkle_root(&hashes_tampered);
    assert_ne!(root1, root2);
}
