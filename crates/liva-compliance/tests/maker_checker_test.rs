//! Comprehensive unit & integration tests for 4-Tier Maker-Checker Policy Engine.

use liva_compliance::maker_checker::{
    ApprovalAction, ApprovalDecision, ApprovalMode, ApprovalRequest, MakerCheckerEngine,
    MakerCheckerError, ProposalStatus,
};

#[test]
fn test_tier1_micro_variance_auto_approval() {
    let mut engine = MakerCheckerEngine::with_default_policy();
    let now = 1726300000;

    // Amount: 5,000,000 VND (< 10M VND) -> Tier 1
    let req = ApprovalRequest {
        request_id: "REQ-T1-01".to_string(),
        amount_vnd: 5_000_000,
        description: "Chenh lech phi vi mo".to_string(),
        maker_id: "OPERATOR_01".to_string(),
        maker_role: "OPERATOR_MAKER".to_string(),
    };

    let proposal = engine.create_proposal(req, now).expect("Must create Tier 1 proposal");
    assert_eq!(proposal.tier_level, 1);
    assert_eq!(proposal.approval_mode, ApprovalMode::Auto);
    assert_eq!(proposal.status, ProposalStatus::Approved);
    assert!(proposal.current_token.is_none(), "Auto-approved items do not need checker tokens");
    assert!(proposal.audit_records.len() >= 2);
}

#[test]
fn test_tier2_standard_single_checker_workflow() {
    let mut engine = MakerCheckerEngine::with_default_policy();
    let now = 1726300000;

    // Amount: 50,000,000 VND (10M - 100M VND) -> Tier 2
    let req = ApprovalRequest {
        request_id: "REQ-T2-01".to_string(),
        amount_vnd: 50_000_000,
        description: "Khop giao dich hoa don 50M".to_string(),
        maker_id: "MAKER_01".to_string(),
        maker_role: "OPERATOR_MAKER".to_string(),
    };

    let proposal = engine.create_proposal(req, now).expect("Must create Tier 2 proposal");
    assert_eq!(proposal.tier_level, 2);
    assert_eq!(proposal.approval_mode, ApprovalMode::SingleChecker);
    assert_eq!(proposal.status, ProposalStatus::PendingChecker1);
    let token = proposal.current_token.expect("Must issue single-use token");

    // Attempt 1: Self-approval violation (Maker attempts to approve)
    let self_approve_action = ApprovalAction {
        decision: ApprovalDecision::Approve,
        checker_id: "MAKER_01".to_string(), // SAME as maker!
        checker_role: "OPERATOR_CHECKER".to_string(),
        token: token.clone(),
        notes: "Tu phe duyet".to_string(),
    };

    let err = engine.process_action("REQ-T2-01", self_approve_action, now + 10);
    assert!(matches!(err, Err(MakerCheckerError::SelfApprovalProhibited { .. })));

    // Attempt 2: Valid checker approval
    let valid_action = ApprovalAction {
        decision: ApprovalDecision::Approve,
        checker_id: "CHECKER_01".to_string(),
        checker_role: "OPERATOR_CHECKER".to_string(),
        token: token.clone(),
        notes: "Phe duyet hop le".to_string(),
    };

    let approved = engine
        .process_action("REQ-T2-01", valid_action, now + 20)
        .expect("Must approve");
    assert_eq!(approved.status, ProposalStatus::Approved);
    assert_eq!(approved.checker1_id.as_deref(), Some("CHECKER_01"));

    // Attempt 3: Anti-Replay: Reuse consumed token
    let replay_action = ApprovalAction {
        decision: ApprovalDecision::Approve,
        checker_id: "CHECKER_02".to_string(),
        checker_role: "OPERATOR_CHECKER".to_string(),
        token: token.clone(),
        notes: "Thu lai token da dung".to_string(),
    };
    let replay_err = engine.process_action("REQ-T2-01", replay_action, now + 30);
    assert!(matches!(replay_err, Err(MakerCheckerError::InvalidStatus { .. } | MakerCheckerError::TokenAlreadyUsed { .. })));
}

#[test]
fn test_tier3_senior_checker_role_enforcement() {
    let mut engine = MakerCheckerEngine::with_default_policy();
    let now = 1726300000;

    // Amount: 300,000,000 VND (100M - 500M VND) -> Tier 3 Senior Checker
    let req = ApprovalRequest {
        request_id: "REQ-T3-01".to_string(),
        amount_vnd: 300_000_000,
        description: "Giao dich 300M".to_string(),
        maker_id: "MAKER_01".to_string(),
        maker_role: "OPERATOR_MAKER".to_string(),
    };

    let proposal = engine.create_proposal(req, now).expect("Must create Tier 3 proposal");
    assert_eq!(proposal.tier_level, 3);
    let token = proposal.current_token.unwrap();

    // Junior operator checker role not allowed for Tier 3!
    let junior_action = ApprovalAction {
        decision: ApprovalDecision::Approve,
        checker_id: "CHECKER_JUNIOR".to_string(),
        checker_role: "OPERATOR_CHECKER".to_string(), // Not in Tier 3 allowed roles!
        token: token.clone(),
        notes: "Duyet vuot cap".to_string(),
    };

    let err = engine.process_action("REQ-T3-01", junior_action, now + 10);
    assert!(matches!(err, Err(MakerCheckerError::UnauthorizedRole { .. })));

    // Senior Checker role approved
    let senior_action = ApprovalAction {
        decision: ApprovalDecision::Approve,
        checker_id: "CHECKER_SENIOR".to_string(),
        checker_role: "SENIOR_CHECKER".to_string(),
        token,
        notes: "Senior checker phe duyet".to_string(),
    };

    let approved = engine
        .process_action("REQ-T3-01", senior_action, now + 20)
        .expect("Must succeed");
    assert_eq!(approved.status, ProposalStatus::Approved);
}

#[test]
fn test_tier4_dual_checker_4_eyes_workflow() {
    let mut engine = MakerCheckerEngine::with_default_policy();
    let now = 1726300000;

    // Amount: 800,000,000 VND (>= 500M VND) -> Tier 4 Dual Checker
    let req = ApprovalRequest {
        request_id: "REQ-T4-01".to_string(),
        amount_vnd: 800_000_000,
        description: "Chuyen tien tai tro 800M".to_string(),
        maker_id: "MAKER_ALICE".to_string(),
        maker_role: "OPERATOR_MAKER".to_string(),
    };

    let proposal = engine.create_proposal(req, now).expect("Must create Tier 4 proposal");
    assert_eq!(proposal.tier_level, 4);
    assert_eq!(proposal.approval_mode, ApprovalMode::DualChecker);
    assert_eq!(proposal.status, ProposalStatus::PendingChecker1);
    let token1 = proposal.current_token.expect("Must issue Token 1");

    // Stage 1 Approval by Senior Checker Bob
    let stage1_action = ApprovalAction {
        decision: ApprovalDecision::Approve,
        checker_id: "CHECKER_BOB".to_string(),
        checker_role: "SENIOR_CHECKER".to_string(),
        token: token1,
        notes: "Kiem soat vien 1 dong y".to_string(),
    };

    let p_stage1 = engine
        .process_action("REQ-T4-01", stage1_action, now + 100)
        .expect("Stage 1 must pass");
    assert_eq!(p_stage1.status, ProposalStatus::PendingChecker2);
    let token2 = p_stage1.current_token.expect("Must issue fresh Token 2 for Stage 2");

    // Violation Check A: Maker Alice attempts to act as Checker 2
    let maker_stage2 = ApprovalAction {
        decision: ApprovalDecision::Approve,
        checker_id: "MAKER_ALICE".to_string(),
        checker_role: "CHIEF_ACCOUNTANT".to_string(),
        token: token2.clone(),
        notes: "Maker tu duyet vong 2".to_string(),
    };
    let err_maker = engine.process_action("REQ-T4-01", maker_stage2, now + 150);
    assert!(matches!(err_maker, Err(MakerCheckerError::SelfApprovalProhibited { .. })));

    // Violation Check B: Checker Bob attempts to act as Checker 2 (Duplicate Checker)
    let duplicate_checker = ApprovalAction {
        decision: ApprovalDecision::Approve,
        checker_id: "CHECKER_BOB".to_string(), // SAME as Checker 1!
        checker_role: "CHIEF_ACCOUNTANT".to_string(),
        token: token2.clone(),
        notes: "Bob tu duyet ca 2 vong".to_string(),
    };
    let err_dup = engine.process_action("REQ-T4-01", duplicate_checker, now + 200);
    assert!(matches!(err_dup, Err(MakerCheckerError::DuplicateCheckerProhibited { .. })));

    // Valid Stage 2 Approval by Chief Accountant Charlie
    let stage2_action = ApprovalAction {
        decision: ApprovalDecision::Approve,
        checker_id: "CHECKER_CHARLIE".to_string(),
        checker_role: "CHIEF_ACCOUNTANT".to_string(),
        token: token2,
        notes: "Ke toan truong phe duyet vong 2".to_string(),
    };

    let p_final = engine
        .process_action("REQ-T4-01", stage2_action, now + 250)
        .expect("Stage 2 must pass");
    assert_eq!(p_final.status, ProposalStatus::Approved);
    assert_eq!(p_final.checker1_id.as_deref(), Some("CHECKER_BOB"));
    assert_eq!(p_final.checker2_id.as_deref(), Some("CHECKER_CHARLIE"));
    assert!(p_final.current_token.is_none());
    assert!(p_final.audit_records.len() >= 3);
}

#[test]
fn test_token_ttl_expiration() {
    let mut engine = MakerCheckerEngine::with_default_policy();
    let now = 1726300000;

    let req = ApprovalRequest {
        request_id: "REQ-EXP-01".to_string(),
        amount_vnd: 50_000_000,
        description: "Giao dich het han token".to_string(),
        maker_id: "MAKER_01".to_string(),
        maker_role: "OPERATOR_MAKER".to_string(),
    };

    let proposal = engine.create_proposal(req, now).unwrap();
    let token = proposal.current_token.unwrap();

    // Action submitted after 901 seconds (TTL is 900 seconds)
    let expired_action = ApprovalAction {
        decision: ApprovalDecision::Approve,
        checker_id: "CHECKER_01".to_string(),
        checker_role: "OPERATOR_CHECKER".to_string(),
        token,
        notes: "Duyet sau khi het han".to_string(),
    };

    let err = engine.process_action("REQ-EXP-01", expired_action, now + 901);
    assert!(matches!(err, Err(MakerCheckerError::TokenExpired { .. })));
}
