use liva_audit::*;

const TEST_SECRET: &[u8] = b"audit-master-hmac-secret-key-32b!";

#[test]
fn test_rfc6962_second_preimage_attack_resistance() {
    let left_hash = [0x11u8; 32];
    let right_hash = [0x22u8; 32];

    let internal_node = hash_node(&left_hash, &right_hash);

    let mut fake_leaf_data = Vec::with_capacity(64);
    fake_leaf_data.extend_from_slice(&left_hash);
    fake_leaf_data.extend_from_slice(&right_hash);

    let fake_leaf_hash = hash_leaf(&fake_leaf_data);

    assert_ne!(
        internal_node, fake_leaf_hash,
        "Second-preimage collision MUST be prevented by RFC 6962 domain prefixes"
    );
}

#[test]
fn test_exhaustive_1_byte_tamper_detection_in_leaf_data() {
    let payload = b"VIETCOMBANK_TX_20260915_AMOUNT_50000000_VND_RECONCILED";
    let leaves = vec![
        payload.to_vec(),
        b"TX_2".to_vec(),
        b"TX_3".to_vec(),
        b"TX_4".to_vec(),
    ];
    let tree = BinaryMerkleTree::from_raw_leaves(&leaves);

    let proof = tree.generate_inclusion_proof(0).unwrap();
    assert!(proof.verify_inclusion(&tree.root(), payload));

    // Exhaustively mutate EVERY SINGLE BYTE in the payload (every byte, every bit)
    for byte_idx in 0..payload.len() {
        for bit in 0..8 {
            let mut tampered_payload = payload.to_vec();
            tampered_payload[byte_idx] ^= 1 << bit;

            let is_valid = proof.verify_inclusion(&tree.root(), &tampered_payload);
            assert!(
                !is_valid,
                "Verification MUST FAIL when byte {byte_idx} bit {bit} is flipped!"
            );
        }
    }
}

#[test]
fn test_exhaustive_1_byte_tamper_detection_in_proof_hashes() {
    let leaves: Vec<Vec<u8>> = (0..16)
        .map(|i| format!("banking-audit-record-id-{i}").into_bytes())
        .collect();
    let tree = BinaryMerkleTree::from_raw_leaves(&leaves);
    let original_root = tree.root();

    let target_idx = 7;
    let target_leaf = &leaves[target_idx];
    let proof = tree.generate_inclusion_proof(target_idx).unwrap();
    assert!(proof.verify_inclusion(&original_root, target_leaf));

    // 1. Mutate every byte of proof.leaf_hash
    for byte_idx in 0..32 {
        let mut tampered_proof = proof.clone();
        tampered_proof.leaf_hash[byte_idx] ^= 0x01;
        assert!(
            !tampered_proof.verify_inclusion(&original_root, target_leaf),
            "Flipping byte {byte_idx} of leaf_hash MUST fail verification!"
        );
    }

    // 2. Mutate every byte of every sibling hash in audit_path
    for step_idx in 0..proof.audit_path.len() {
        for byte_idx in 0..32 {
            let mut tampered_proof = proof.clone();
            tampered_proof.audit_path[step_idx][byte_idx] ^= 0x55;
            if !tampered_proof.proof_path.is_empty() {
                tampered_proof.proof_path[step_idx].sibling_hash[byte_idx] ^= 0x55;
            }
            assert!(
                !tampered_proof.verify_inclusion(&original_root, target_leaf),
                "Flipping byte {byte_idx} at step {step_idx} of audit_path MUST fail verification!"
            );
        }
    }

    // 3. Mutate every byte of root_hash
    for byte_idx in 0..32 {
        let mut tampered_root = original_root;
        tampered_root[byte_idx] ^= 0xAA;
        assert!(
            !proof.verify_inclusion(&tampered_root, target_leaf),
            "Flipping byte {byte_idx} of root_hash MUST fail verification!"
        );
    }
}

#[test]
fn test_audit_ledger_chain_and_1_byte_tamper_detection() {
    let mut ledger = AuditLedger::new(TEST_SECRET);

    ledger.append("MakerAccountant", "IMPORT_STATEMENT", "vcb_aug2026.xlsx");
    ledger.append("ReconciliationEngine", "MATCH_TIER1", "tx_102_matched");
    ledger.append("ChiefAccountant", "APPROVE_DISCREPANCY", "fee_split_approved_11000");

    let initial_report = ledger.verify();
    assert!(initial_report.is_intact);
    assert_eq!(initial_report.total_records, 3);
    assert!(initial_report.tampered_seq_id.is_none());

    // 1. Tamper 1 byte in payload_digest of record 2
    let mut tampered_records = ledger.records().to_vec();
    let mut bytes = tampered_records[1].payload_digest.clone().into_bytes();
    bytes[0] = if bytes[0] == b'a' { b'b' } else { b'a' };
    tampered_records[1].payload_digest = String::from_utf8(bytes).unwrap();
    let report = AuditLedger::verify_records(TEST_SECRET, &tampered_records);
    assert!(!report.is_intact);
    assert_eq!(report.tampered_seq_id, Some(2));

    // 2. Tamper 1 byte in record_hash of record 1
    let mut tampered_records2 = ledger.records().to_vec();
    let mut bytes2 = tampered_records2[0].record_hash.clone().into_bytes();
    bytes2[0] = if bytes2[0] == b'0' { b'1' } else { b'0' };
    tampered_records2[0].record_hash = String::from_utf8(bytes2).unwrap();
    let report2 = AuditLedger::verify_records(TEST_SECRET, &tampered_records2);
    assert!(!report2.is_intact);
    assert_eq!(report2.tampered_seq_id, Some(1));

    // 3. Tamper timestamp by 1 second on record 3
    let mut tampered_records3 = ledger.records().to_vec();
    tampered_records3[2].timestamp += 1;
    let report3 = AuditLedger::verify_records(TEST_SECRET, &tampered_records3);
    assert!(!report3.is_intact);
    assert_eq!(report3.tampered_seq_id, Some(3));

    // 4. Tamper actor principal on record 2
    let mut tampered_records4 = ledger.records().to_vec();
    tampered_records4[1].actor_principal = "MaliciousAttacker".to_string();
    let report4 = AuditLedger::verify_records(TEST_SECRET, &tampered_records4);
    assert!(!report4.is_intact);
    assert_eq!(report4.tampered_seq_id, Some(2));
}

#[test]
fn test_odd_leaves_and_proof_verification() {
    for n in [1, 2, 3, 5, 7, 9, 13, 27, 64] {
        let leaves: Vec<Vec<u8>> = (0..n)
            .map(|i| format!("test-payload-transaction-{i}").into_bytes())
            .collect();
        let tree = BinaryMerkleTree::from_raw_leaves(&leaves);
        assert_eq!(tree.leaf_count(), n);

        for (i, leaf) in leaves.iter().enumerate() {
            let proof = tree.generate_inclusion_proof(i).unwrap();
            assert!(
                proof.verify(leaf),
                "Verification failed for leaf {i} out of {n} leaves"
            );
            assert!(
                verify_inclusion(&tree.root(), leaf, &proof),
                "Free function verification failed for leaf {i}"
            );

            // Sub-millisecond verification duration
            let (valid, duration) = proof.verify_with_duration(leaf);
            assert!(valid);
            assert!(
                duration.as_millis() < 1,
                "Proof verification must take < 1 ms, took {:?}",
                duration
            );
        }
    }
}
