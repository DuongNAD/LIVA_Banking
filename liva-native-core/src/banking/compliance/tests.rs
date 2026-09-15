//! Comprehensive Integration and Unit Tests for Compliance, Cryptographic Auditing & Security.
//!
//! Validates:
//! 1. Zero-Egress Netfilter Hardening (0 bytes egress to external networks).
//! 2. Maker-Checker 4-Eyes Authorization Engine (Circular 09/2020/TT-NHNN).
//! 3. RFC 6962 Binary Merkle Tree & Sub-millisecond Inclusion Proofs.

use crate::banking::compliance::*;
use crate::banking::models::{TransactionRecord, TransactionType};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

const TEST_SECRET_KEY: &[u8; 32] = b"compliance-test-secret-key-32b!!";

#[test]
fn test_zero_egress_strict_interception_and_zero_byte_assertion() {
    let tracker = EgressTrafficTracker::new();

    // 1. Permitted loopback transmissions (Local LLM inference & internal IPC)
    assert!(tracker.record_egress("127.0.0.1:8002", 4096).is_ok());
    assert!(
        tracker
            .record_egress("http://localhost:11434/api/generate", 8192)
            .is_ok()
    );
    assert!(tracker.record_egress("[::1]:8080", 2048).is_ok());

    // 2. Prohibited external egress operations
    let cloud_endpoints = [
        "https://api.openai.com/v1/chat/completions",
        "https://api.anthropic.com/v1/messages",
        "https://telemetry.bank.vn/collector",
        "http://203.113.131.1:443",
    ];

    for endpoint in cloud_endpoints {
        let err = tracker.record_egress(endpoint, 1024);
        assert!(
            err.is_err(),
            "Outbound operation to '{endpoint}' must be blocked"
        );
    }

    let report = tracker.report();
    assert_eq!(
        report.external_bytes_transmitted,
        (cloud_endpoints.len() * 1024) as u64
    );
    assert_eq!(report.loopback_bytes_transmitted, 4096 + 8192 + 2048);
    assert_eq!(report.blocked_attempts_count, cloud_endpoints.len());
    assert!(!report.is_zero_egress);
    assert!(tracker.assert_zero_external_egress().is_err());
}

#[test]
fn test_bank_statement_processing_zero_egress_verified() {
    // Verify 0 bytes egress during statement parsing and local processing
    let dummy_txs: Vec<TransactionRecord> = (0..1000)
        .map(|i| TransactionRecord {
            row_id: i,
            tx_date: 1726000000 + (i as i64 * 60),
            value_date: 1726000000 + (i as i64 * 60),
            doc_ref: Some(format!("FT24090{i}")),
            tx_type: if i % 2 == 0 {
                TransactionType::Credit
            } else {
                TransactionType::Debit
            },
            amount: 500_000 * (i as u64 + 1),
            balance_after: Some(100_000_000),
            counterparty_account: Some("0123456789".to_string()),
            counterparty_name: Some("CONG TY TNHH TEST".to_string()),
            counterparty_bank: Some("VCB".to_string()),
            narration: format!("Thanh toan tien hang hoa don {i}"),
            ft_number: None,
            trace_id: None,
            raw_ref: None,
        })
        .collect();

    let (processed_count, report) = verify_statement_processing_zero_egress(|| {
        // Execute in-memory processing
        let count = dummy_txs.len();
        let total_amount: u64 = dummy_txs.iter().map(|t| t.amount).sum();
        assert!(total_amount > 0);
        count
    })
    .expect("Zero egress check must pass");

    assert_eq!(processed_count, 1000);
    assert_eq!(report.external_bytes_transmitted, 0);
    assert!(report.is_zero_egress);
    assert_eq!(report.blocked_attempts_count, 0);
}

#[test]
fn test_socket_netfilter_prohibits_non_loopback() {
    let local = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8002);
    let public_ip = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(54, 251, 12, 88)), 443);

    assert!(ZeroEgressNetfilter::validate_socket_addr(&local).is_ok());
    let err = ZeroEgressNetfilter::validate_socket_addr(&public_ip);
    assert!(err.is_err());
}

#[test]
fn test_maker_checker_circular_09_enforcement() {
    let mut engine = MakerCheckerEngine::new();
    let t0 = 1726000000;

    let details = ProposalDetails {
        bank_tx_id: "tx-bank-999".to_string(),
        selected_ledger_entry_ids: vec!["gl-voucher-888".to_string()],
        matched_amount: 50_000_000,
        discrepancy_amount: 15_000,
        notes: Some("Bank charge deduction variance approved by CFO".to_string()),
    };

    // 1. Maker creates proposal
    let (proposal, token) = engine
        .create_proposal(
            "maker_accountant_01",
            "Nguyen Van An",
            "match_uuid_123",
            HitlActionType::OverrideDiscrepancy,
            details,
            t0,
        )
        .expect("Proposal creation should succeed");

    assert_eq!(proposal.status, ProposalStatus::Pending);
    assert_eq!(proposal.expires_at, t0 + HITL_TOKEN_TTL_SECONDS);

    // 2. Self-approval attempt MUST fail-closed (maker_id == checker_id)
    let self_approval_err = engine.submit_decision(
        "maker_accountant_01",
        "Nguyen Van An",
        &proposal.proposal_id,
        &token,
        CheckerDecision::Approve,
        None,
        TEST_SECRET_KEY,
        t0 + 60,
    );
    match self_approval_err {
        Err(MakerCheckerError::SelfApprovalProhibited { maker_id }) => {
            assert_eq!(maker_id, "maker_accountant_01");
        }
        other => panic!("Expected SelfApprovalProhibited, got {other:?}"),
    }

    // 3. Different Checker approves successfully
    let audit_record = engine
        .submit_decision(
            "checker_chief_02",
            "Tran Thi Bich",
            &proposal.proposal_id,
            &token,
            CheckerDecision::Approve,
            Some("Reviewed and approved per Circular 09".to_string()),
            TEST_SECRET_KEY,
            t0 + 180,
        )
        .expect("Checker approval should succeed");

    assert_eq!(audit_record.decision, CheckerDecision::Approve);
    assert_eq!(audit_record.maker_id, "maker_accountant_01");
    assert_eq!(audit_record.checker_id, "checker_chief_02");
    assert_eq!(audit_record.circular_reference, CIRCULAR_09_REF);
    assert!(engine.verify_audit_record(&audit_record, TEST_SECRET_KEY));

    // 4. Token replay attempt MUST be rejected
    let replay_err = engine.submit_decision(
        "checker_chief_03",
        "Le Van Cuong",
        &proposal.proposal_id,
        &token,
        CheckerDecision::Approve,
        None,
        TEST_SECRET_KEY,
        t0 + 200,
    );
    match replay_err {
        Err(MakerCheckerError::TokenAlreadyUsed { .. }) => {}
        other => panic!("Expected TokenAlreadyUsed, got {other:?}"),
    }
}

#[test]
fn test_maker_checker_token_ttl_15_minutes_strict_expiration() {
    let mut engine = MakerCheckerEngine::new();
    let t0 = 1726000000;

    let details = ProposalDetails {
        bank_tx_id: "tx-bank-ttl".to_string(),
        selected_ledger_entry_ids: vec!["gl-voucher-ttl".to_string()],
        matched_amount: 10_000_000,
        discrepancy_amount: 0,
        notes: None,
    };

    let (proposal, token) = engine
        .create_proposal(
            "maker_01",
            "Maker Name",
            "match_ttl",
            HitlActionType::ApproveMatch,
            details,
            t0,
        )
        .unwrap();

    // Exactly at t0 + 15 mins (900s) -> Valid
    // But at t0 + 901s -> Expired
    let expired_err = engine.submit_decision(
        "checker_02",
        "Checker Name",
        &proposal.proposal_id,
        &token,
        CheckerDecision::Approve,
        None,
        TEST_SECRET_KEY,
        t0 + 901,
    );

    match expired_err {
        Err(MakerCheckerError::TokenExpired {
            expired_at,
            current_time,
        }) => {
            assert_eq!(expired_at, t0 + 900);
            assert_eq!(current_time, t0 + 901);
        }
        other => panic!("Expected TokenExpired, got {other:?}"),
    }

    let p = engine.get_proposal(&proposal.proposal_id).unwrap();
    assert_eq!(p.status, ProposalStatus::Expired);
}

#[test]
fn test_rfc6962_merkle_tree_second_preimage_attack_defense() {
    // RFC 6962 domain separation:
    // Leaf prefix = 0x00, Internal node prefix = 0x01.
    let left = [0xAAu8; 32];
    let right = [0xBBu8; 32];

    let internal_node = hash_node(&left, &right);

    // Attacker crafts a leaf matching the internal node children
    let mut attacker_leaf = Vec::new();
    attacker_leaf.extend_from_slice(&left);
    attacker_leaf.extend_from_slice(&right);

    let leaf_h = hash_leaf(&attacker_leaf);

    // Assert that domain separation guarantees distinct hashes
    assert_ne!(
        internal_node, leaf_h,
        "Second-Preimage Attack MUST be impossible under RFC 6962 domain separation"
    );
}

#[test]
fn test_merkle_tree_sub_millisecond_inclusion_proofs_on_large_dataset() {
    let count = 10_000;
    let leaves: Vec<Vec<u8>> = (0..count)
        .map(|i| {
            let tx = TransactionRecord {
                row_id: i,
                tx_date: 1726000000 + i as i64,
                value_date: 1726000000 + i as i64,
                doc_ref: Some(format!("DOC-{i}")),
                tx_type: TransactionType::Credit,
                amount: 1_000_000 + (i as u64 * 100),
                balance_after: None,
                counterparty_account: Some("987654321".to_string()),
                counterparty_name: Some("PARTNER CORP".to_string()),
                counterparty_bank: Some("TCB".to_string()),
                narration: format!("Reconciled audit trail record seq {i}"),
                ft_number: None,
                trace_id: None,
                raw_ref: None,
            };
            TransactionAuditLeaf::from_record(&tx, "MATCHED").to_leaf_bytes()
        })
        .collect();

    let tree = BinaryMerkleTree::from_raw_leaves(&leaves);
    assert_eq!(tree.leaf_count(), count);

    // Sample arbitrary indices across the tree
    let sample_indices = [0, 1, 500, 2500, 4999, 5000, 7500, 9998, 9999];

    for &idx in &sample_indices {
        let proof = tree
            .generate_inclusion_proof(idx)
            .expect("Inclusion proof generation must succeed");

        // O(log N) depth check: ceil(log2(10000)) = 14
        assert!(
            proof.proof_path.len() <= 15,
            "Proof path must be O(log N), got {}",
            proof.proof_path.len()
        );

        // Verify inclusion proof and time it
        let (is_valid, duration) = proof.verify_with_duration(&leaves[idx]);
        assert!(
            is_valid,
            "Merkle inclusion proof must be valid for index {idx}"
        );

        // Must verify in < 1 ms
        assert!(
            duration < Duration::from_millis(1),
            "Proof verification duration ({:?}) must be < 1 ms",
            duration
        );
    }
}

#[test]
fn test_merkle_inclusion_proof_zero_exposure_of_unrelated_transactions() {
    let leaves = vec![
        b"CONFIDENTIAL_TX_CUSTOMER_A_100_BILLION".to_vec(),
        b"TARGET_TX_RECONCILED_50_MILLION".to_vec(),
        b"CONFIDENTIAL_TX_CUSTOMER_C_500_BILLION".to_vec(),
        b"CONFIDENTIAL_TX_CUSTOMER_D_20_BILLION".to_vec(),
    ];

    let tree = BinaryMerkleTree::from_raw_leaves(&leaves);
    let proof = tree.generate_inclusion_proof(1).unwrap();

    // Verify proof
    assert!(proof.verify(&leaves[1]));

    // Check proof contents: proof only contains hashes, never leaf plaintext
    let proof_json = serde_json::to_string(&proof).unwrap();
    assert!(!proof_json.contains("CUSTOMER_A"));
    assert!(!proof_json.contains("CUSTOMER_C"));
    assert!(!proof_json.contains("CUSTOMER_D"));
    assert!(!proof_json.contains("100_BILLION"));
}
