use liva_banking::compliance::{
    ApprovalPayload, BinaryMerkleTree, MakerCheckerEngine, ProposalLifecycle, UserIdentity,
    UserRole, ZeroEgressNetfilter,
};
use liva_banking::service::{PeriodCloseGate, PeriodStatus};

#[test]
fn test_maker_checker_separation_of_duties() {
    let mut engine = MakerCheckerEngine::new(b"secret-salt-xyz");

    let maker = UserIdentity {
        user_id: "user_maker_01".to_string(),
        employee_id: "EMP1001".to_string(),
        citizen_id_hash: "cccd_hash_111".to_string(),
        role: UserRole::Maker,
        name: "Nguyen Van A".to_string(),
    };

    let self_as_checker = UserIdentity {
        user_id: "user_maker_01".to_string(), // Same user!
        employee_id: "EMP1001".to_string(),
        citizen_id_hash: "cccd_hash_111".to_string(),
        role: UserRole::Checker,
        name: "Nguyen Van A".to_string(),
    };

    let real_checker = UserIdentity {
        user_id: "user_checker_02".to_string(),
        employee_id: "EMP2002".to_string(),
        citizen_id_hash: "cccd_hash_222".to_string(),
        role: UserRole::Checker,
        name: "Tran Thi B".to_string(),
    };

    let payload = ApprovalPayload {
        proposal_id: "PROP-001".to_string(),
        bank_tx_id: 1,
        erp_doc_ids: vec!["DOC1".to_string()],
        matched_amount: 50_000_000,
        fee_amount: 0,
        version: 1,
    };

    let now = 1755216000;

    // Self-approval must FAIL CLOSED!
    let self_challenge = engine.issue_challenge(&maker, &self_as_checker, &payload, now);
    assert!(self_challenge.is_err(), "Must block Maker from acting as Checker on same proposal");

    // Independent Checker succeeds
    let challenge = engine
        .issue_challenge(&maker, &real_checker, &payload, now)
        .expect("Should issue challenge token to independent checker");

    assert_eq!(challenge.expires_at, now + 900); // 15-minute TTL

    // Approve with valid token
    let approve_res = engine
        .verify_and_approve(&maker, &real_checker, &payload, &challenge.token, now + 300)
        .expect("Should approve within TTL");
    assert_eq!(approve_res, ProposalLifecycle::Approved);

    // Replay attack with same token must FAIL
    let replay_res = engine.verify_and_approve(&maker, &real_checker, &payload, &challenge.token, now + 400);
    assert!(replay_res.is_err(), "Must reject replayed / already used token");
}

#[test]
fn test_challenge_token_ttl_and_stale_invalidation() {
    let mut engine = MakerCheckerEngine::new(b"secret-salt-xyz");

    let maker = UserIdentity {
        user_id: "user_maker_01".to_string(),
        employee_id: "EMP1001".to_string(),
        citizen_id_hash: "cccd_hash_111".to_string(),
        role: UserRole::Maker,
        name: "Nguyen Van A".to_string(),
    };

    let checker = UserIdentity {
        user_id: "user_checker_02".to_string(),
        employee_id: "EMP2002".to_string(),
        citizen_id_hash: "cccd_hash_222".to_string(),
        role: UserRole::Checker,
        name: "Tran Thi B".to_string(),
    };

    let payload = ApprovalPayload {
        proposal_id: "PROP-002".to_string(),
        bank_tx_id: 2,
        erp_doc_ids: vec!["DOC2".to_string()],
        matched_amount: 30_000_000,
        fee_amount: 0,
        version: 1,
    };

    let now = 1755216000;
    let challenge = engine.issue_challenge(&maker, &checker, &payload, now).unwrap();

    // 1. Expiration test (> 900 seconds)
    let expired_res = engine.verify_and_approve(&maker, &checker, &payload, &challenge.token, now + 901);
    assert!(expired_res.is_err(), "Must reject challenge token after 15-minute TTL");

    // 2. Data modification test -> STALE invalidation
    let challenge2 = engine.issue_challenge(&maker, &checker, &payload, now).unwrap();
    let mut modified_payload = payload.clone();
    modified_payload.matched_amount = 35_000_000; // Payload changed after token issuance!
    modified_payload.version = 2;

    let stale_res = engine
        .verify_and_approve(&maker, &checker, &modified_payload, &challenge2.token, now + 100)
        .expect("Should execute verification");
    assert_eq!(stale_res, ProposalLifecycle::Stale, "Must invalidate to STALE if payload mutated");
}

#[test]
fn test_merkle_tree_proof_and_tamper_detection() {
    let tx1 = b"TX001: 50,000,000 VND to ABC";
    let tx2 = b"TX002: 15,000,000 VND to XYZ";
    let tx3 = b"TX003: 30,000,000 VND to DEF";
    let tx4 = b"TX004: 10,000,000 VND to GHI";

    let tree = BinaryMerkleTree::from_leaves_data(&[tx1, tx2, tx3, tx4]);
    let root = tree.root_hash();
    assert_ne!(root, [0u8; 32]);

    // Verify valid inclusion proof for TX2 (index 1)
    let proof = tree.generate_proof(1).expect("Should generate proof for leaf 1");
    assert!(proof.verify(tx2), "Proof must verify against original transaction data");

    // Tampered transaction data must FAIL verification
    let tampered_tx2 = b"TX002: 15,000,000 VND to ATTACKER";
    assert!(!proof.verify(tampered_tx2), "Tampered data must FAIL proof verification");
}

#[test]
fn test_zero_egress_netfilter() {
    let filter = ZeroEgressNetfilter::new();

    // Permitted local endpoints
    assert!(filter.assert_zero_egress("127.0.0.1:8080").is_ok());
    assert!(filter.assert_zero_egress("localhost:3000").is_ok());
    assert!(filter.assert_zero_egress("192.168.1.50:5432").is_ok()); // Internal LAN

    // Prohibited external internet egress
    assert!(filter.assert_zero_egress("api.openai.com:443").is_err());
    assert!(filter.assert_zero_egress("telemetry.vendor.com:443").is_err());
}

#[test]
fn test_period_close_gate_and_reopen() {
    let checker = UserIdentity {
        user_id: "user_checker_02".to_string(),
        employee_id: "EMP2002".to_string(),
        citizen_id_hash: "cccd_hash_222".to_string(),
        role: UserRole::Checker,
        name: "Tran Thi B".to_string(),
    };

    let txs: Vec<&[u8]> = vec![b"TX1", b"TX2"];

    // 1. Cannot close with open exceptions
    let fail_res = PeriodCloseGate::close_period(
        "PERIOD-2026-08",
        "LEGAL_ENTITY_VN",
        "0011000123456",
        1754000000,
        1756000000,
        1, // 1 open exception remaining!
        0,
        true,
        &txs,
        50_000_000,
        15_000_000,
        35_000_000,
        &checker,
        1756000100,
    );
    assert!(fail_res.is_err(), "Must block period close if exceptions are open");

    // 2. Successful close when all conditions satisfied
    let mut report = PeriodCloseGate::close_period(
        "PERIOD-2026-08",
        "LEGAL_ENTITY_VN",
        "0011000123456",
        1754000000,
        1756000000,
        0, // 0 exceptions
        0, // 0 pending
        true, // balanced
        &txs,
        50_000_000,
        15_000_000,
        35_000_000,
        &checker,
        1756000100,
    )
    .expect("Should close period successfully");

    assert_eq!(report.status, PeriodStatus::Closed);
    assert!(!report.merkle_root.is_empty());

    // 3. Reopen with justification
    PeriodCloseGate::reopen_period(&mut report, &checker, "Audit correction per Tax Office request", 1756000500)
        .expect("Should reopen period");
    assert_eq!(report.status, PeriodStatus::ReopenedWithApproval);
}
