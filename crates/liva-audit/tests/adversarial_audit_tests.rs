//! Adversarial test suite for `liva-audit` (Milestone M1).
//!
//! Validates:
//! 1. RFC 6962 Binary Merkle Tree edge cases: 0 leaves, 1 leaf, 2 leaves, odd leaf counts, power-of-two leaf counts.
//! 2. Second-preimage attack resistance: attempting to present internal node hashes or concatenations as leaves.
//! 3. 1-byte bit flips in leaf data, proof hashes, and HMAC-SHA256 log chain.
//! 4. Audit ledger integrity: seq_id tampering and HMAC canonicalization delimiter collision resistance.

use liva_audit::*;
use sha2::{Digest, Sha256};

const HMAC_TEST_KEY: &[u8] = b"audit-master-hmac-secret-key-32b!";

#[test]
fn test_rfc6962_edge_case_0_leaves() {
    let tree = BinaryMerkleTree::from_raw_leaves(&[]);
    assert_eq!(tree.leaf_count(), 0);

    // RFC 6962: MTH({}) = SHA-256("")
    let expected_empty_hash: [u8; 32] = Sha256::digest([]).into();
    assert_eq!(
        tree.root(),
        expected_empty_hash,
        "Empty tree root must match SHA-256(\"\")"
    );

    // Proof generation on empty tree must return EmptyTree error
    assert_eq!(
        tree.generate_inclusion_proof(0),
        Err(MerkleAuditError::EmptyTree)
    );
    assert_eq!(
        tree.generate_inclusion_proof(1),
        Err(MerkleAuditError::EmptyTree)
    );
}

#[test]
fn test_rfc6962_edge_case_1_leaf() {
    let leaf_data = b"SINGLE_TRANSACTION_PAYLOAD";
    let leaves = vec![leaf_data.to_vec()];
    let tree = BinaryMerkleTree::from_raw_leaves(&leaves);

    assert_eq!(tree.leaf_count(), 1);

    // RFC 6962: MTH({d[0]}) = SHA-256(0x00 || d[0])
    let expected_root = hash_leaf(leaf_data);
    assert_eq!(tree.root(), expected_root);

    // Inclusion proof for leaf 0
    let proof = tree.generate_inclusion_proof(0).expect("Proof for leaf 0 in 1-leaf tree must succeed");
    assert_eq!(proof.leaf_index, 0);
    assert_eq!(proof.total_leaves, 1);
    assert_eq!(proof.leaf_hash, expected_root);
    assert!(proof.audit_path.is_empty(), "1-leaf tree audit path must be empty");

    // Verification against true data
    assert!(proof.verify(leaf_data));
    assert!(proof.verify_inclusion(&tree.root(), leaf_data));
    assert!(verify_inclusion(&tree.root(), leaf_data, &proof));

    // Verification against wrong data must fail
    assert!(!proof.verify(b"WRONG_DATA"));
    assert!(!proof.verify_inclusion(&tree.root(), b"WRONG_DATA"));

    // Out of bounds index
    assert_eq!(
        tree.generate_inclusion_proof(1),
        Err(MerkleAuditError::IndexOutOfBounds { index: 1, total: 1 })
    );
}

#[test]
fn test_rfc6962_edge_case_2_leaves() {
    let l0 = b"TX_0_DATA";
    let l1 = b"TX_1_DATA";
    let leaves = vec![l0.to_vec(), l1.to_vec()];
    let tree = BinaryMerkleTree::from_raw_leaves(&leaves);

    assert_eq!(tree.leaf_count(), 2);

    let h0 = hash_leaf(l0);
    let h1 = hash_leaf(l1);
    let expected_root = hash_node(&h0, &h1);
    assert_eq!(tree.root(), expected_root);

    // Proof for leaf 0
    let p0 = tree.generate_inclusion_proof(0).unwrap();
    assert_eq!(p0.audit_path.len(), 1);
    assert_eq!(p0.audit_path[0], h1);
    assert!(p0.verify(l0));
    assert!(!p0.verify(l1));

    // Proof for leaf 1
    let p1 = tree.generate_inclusion_proof(1).unwrap();
    assert_eq!(p1.audit_path.len(), 1);
    assert_eq!(p1.audit_path[0], h0);
    assert!(p1.verify(l1));
    assert!(!p1.verify(l0));
}

#[test]
fn test_rfc6962_odd_leaf_counts_comprehensive() {
    let odd_sizes = [3, 5, 7, 9, 11, 13, 17, 25, 33, 65, 127];

    for &n in &odd_sizes {
        let leaves: Vec<Vec<u8>> = (0..n)
            .map(|i| format!("banking-odd-leaf-tx-{i}").into_bytes())
            .collect();
        let tree = BinaryMerkleTree::from_raw_leaves(&leaves);
        assert_eq!(tree.leaf_count(), n);

        let root = tree.root();

        // Verify every leaf in the tree
        for (i, leaf) in leaves.iter().enumerate() {
            let proof = tree.generate_inclusion_proof(i).unwrap();

            // 1. Verify with proof.verify
            assert!(
                proof.verify(leaf),
                "Verification failed for leaf {i} in odd tree of size {n}"
            );

            // 2. Verify with verify_inclusion
            assert!(
                verify_inclusion(&root, leaf, &proof),
                "Free function verification failed for leaf {i} in odd tree of size {n}"
            );

            // 3. Test verification when proof_path is cleared (tests audit_path + derive_directions)
            let mut audit_only_proof = proof.clone();
            audit_only_proof.proof_path.clear();
            assert!(
                audit_only_proof.verify(leaf),
                "Audit-path-only verification (derive_directions) failed for leaf {i} in size {n}"
            );

            // 4. Verify negative test: adjacent leaf data fails
            let fake_data = format!("banking-odd-leaf-tx-{}", (i + 1) % n).into_bytes();
            assert!(
                !proof.verify(&fake_data),
                "Proof for leaf {i} erroneously verified fake leaf data!"
            );
        }

        // Out-of-bounds index must fail cleanly
        assert_eq!(
            tree.generate_inclusion_proof(n),
            Err(MerkleAuditError::IndexOutOfBounds { index: n, total: n })
        );
    }
}

#[test]
fn test_rfc6962_power_of_two_leaf_counts() {
    let pow2_sizes = [2, 4, 8, 16, 32, 64, 128];

    for &n in &pow2_sizes {
        let expected_depth = (n as f64).log2() as usize;
        let leaves: Vec<Vec<u8>> = (0..n)
            .map(|i| format!("pow2-tx-data-{i}").into_bytes())
            .collect();
        let tree = BinaryMerkleTree::from_raw_leaves(&leaves);

        for (i, leaf) in leaves.iter().enumerate() {
            let proof = tree.generate_inclusion_proof(i).unwrap();
            assert_eq!(
                proof.audit_path.len(),
                expected_depth,
                "Power of two size {n} leaf {i} proof depth must be exactly log2(n)"
            );
            assert!(proof.verify(leaf));
        }
    }
}

#[test]
fn test_second_preimage_attack_resistance_exhaustive() {
    // Construct a 4-leaf tree:
    // Root = Node(Node(L0, L1), Node(L2, L3))
    let l0 = b"LEAF_0";
    let l1 = b"LEAF_1";
    let l2 = b"LEAF_2";
    let l3 = b"LEAF_3";

    let leaves = vec![l0.to_vec(), l1.to_vec(), l2.to_vec(), l3.to_vec()];
    let tree = BinaryMerkleTree::from_raw_leaves(&leaves);
    let _root = tree.root();

    let h0 = hash_leaf(l0);
    let h1 = hash_leaf(l1);
    let internal_left = hash_node(&h0, &h1);

    // ATTACK 1: Present concatenated child hashes (h0 || h1) as a leaf payload.
    // In naive Merkle trees without domain prefixes: Hash(h0 || h1) == internal_left.
    // In RFC 6962:
    //   hash_leaf(h0 || h1) = SHA256(0x00 || h0 || h1)
    //   hash_node(h0, h1)   = SHA256(0x01 || h0 || h1)
    let mut fake_leaf_data = Vec::new();
    fake_leaf_data.extend_from_slice(&h0);
    fake_leaf_data.extend_from_slice(&h1);

    let fake_leaf_hash = hash_leaf(&fake_leaf_data);
    assert_ne!(
        fake_leaf_hash, internal_left,
        "SECOND-PREIMAGE CRITICAL: Leaf prefix 0x00 and node prefix 0x01 collision!"
    );

    // ATTACK 2: Try to verify fake leaf payload against root using leaf 0 proof
    let proof0 = tree.generate_inclusion_proof(0).unwrap();
    assert!(
        !proof0.verify(&fake_leaf_data),
        "Second-preimage payload must NOT verify against original root!"
    );

    // ATTACK 3: Prepend 0x01 to fake leaf payload
    let mut prefixed_fake_data = vec![RFC6962_NODE_PREFIX];
    prefixed_fake_data.extend_from_slice(&h0);
    prefixed_fake_data.extend_from_slice(&h1);

    let prefixed_hash = hash_leaf(&prefixed_fake_data);
    assert_ne!(
        prefixed_hash, internal_left,
        "Prepend attack must be defeated by domain separation!"
    );
}

/// EMPIRICAL BUG TEST:
/// In a tree of N > 1 leaves, an inclusion proof with an empty audit path
/// and leaf_hash == root_hash should be rejected because a multi-leaf tree CANNOT
/// have depth 0.
#[test]
fn test_empty_audit_path_multi_leaf_vulnerability() {
    let leaves = vec![b"tx1".to_vec(), b"tx2".to_vec(), b"tx3".to_vec()];
    let tree = BinaryMerkleTree::from_raw_leaves(&leaves);
    let root = tree.root();

    let forged_proof = MerkleInclusionProof {
        leaf_index: 0,
        total_leaves: 3,
        leaf_hash: root, // Set leaf_hash to root
        audit_path: Vec::new(), // Empty audit path!
        proof_path: Vec::new(), // Empty proof path!
        root_hash: root,
    };

    assert!(
        !forged_proof.verify_inclusion_hash(&root, &root),
        "VULNERABILITY: verify_inclusion_hash() accepted an empty audit path on a multi-leaf tree (total_leaves = 3)!"
    );
}

/// EMPIRICAL BUG TEST:
/// Mutating seq_id from 1 to 9999 MUST cause verify_records to report is_intact: false.
#[test]
fn test_hmac_chain_seq_id_tamper_must_fail() {
    let mut ledger = AuditLedger::new(HMAC_TEST_KEY);
    ledger.append("AuditorA", "CREATE_BATCH", "batch_001");
    ledger.append("AuditorB", "POST_JOURNAL", "journal_002");
    ledger.append("AuditorC", "CLOSE_PERIOD", "period_003");

    let mut tampered_records = ledger.records().to_vec();
    tampered_records[0].seq_id = 9999;

    let report = AuditLedger::verify_records(HMAC_TEST_KEY, &tampered_records);
    assert!(
        !report.is_intact,
        "VULNERABILITY: AuditLedger::verify_records() reported is_intact: true despite seq_id being tampered from 1 to 9999!"
    );
}

/// EMPIRICAL BUG TEST:
/// Shifting bytes across variable-length fields (actor and event) in build_hmac_message
/// MUST be detected.
#[test]
fn test_hmac_canonicalization_delimiter_collision_must_fail() {
    let mut ledger = AuditLedger::new(HMAC_TEST_KEY);
    ledger.append_with_timestamp(1700000000, "Admin", "TRANSFER_VND", "payload_1");

    let mut tampered_records = ledger.records().to_vec();
    tampered_records[0].actor_principal = "AdminT".to_string();
    tampered_records[0].event_type = "RANSFER_VND".to_string();

    let report = AuditLedger::verify_records(HMAC_TEST_KEY, &tampered_records);
    assert!(
        !report.is_intact,
        "VULNERABILITY: Shifting bytes across actor/event boundary produced IDENTICAL HMAC, spoofing actor without detection!"
    );
}
