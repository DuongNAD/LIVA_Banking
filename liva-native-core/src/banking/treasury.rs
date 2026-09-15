//! Treasury & Payment Order Lifecycle Management adhering to SBV Circular 09/2020/TT-NHNN.
//!
//! Dual Control (Maker-Checker / 4-Eyes principle):
//! 1. Maker initiates payment order proposal -> Single-use UUIDv4 HITL token issued (15-min TTL).
//! 2. Checker reviews and approves or rejects with single-use token.
//! 3. Fail-closed self-approval prevention: maker_id != checker_id.
//! 4. Cryptographic HMAC-SHA256 audit signature generated upon approval and logged to audit ledger.
//! 5. SQLite persistence in table `payment_orders` with zero floating-point rounding errors (integer VND).

use crate::banking::compliance::audit_ledger::AuditLedger;
use crate::banking::compliance::maker_checker::{
    CIRCULAR_09_REF, HITL_TOKEN_TTL_SECONDS, MakerCheckerError, compute_audit_signature,
    CheckerDecision,
};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

/// Resolves a 32-byte HMAC audit key, using the supplied key or a deterministic fallback.
pub fn resolve_audit_key(provided_key: Option<&[u8; 32]>) -> [u8; 32] {
    if let Some(k) = provided_key {
        *k
    } else {
        use sha2::Digest;
        let mut hasher = sha2::Sha256::new();
        hasher.update(b"liva-banking-treasury-audit-master-key-2026");
        let hash = hasher.finalize();
        let mut key = [0u8; 32];
        key.copy_from_slice(&hash);
        key
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentOrderStatus {
    Draft,
    PendingApproval,
    Approved,
    Rejected,
    Expired,
}

impl PaymentOrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentOrderStatus::Draft => "DRAFT",
            PaymentOrderStatus::PendingApproval => "PENDING_APPROVAL",
            PaymentOrderStatus::Approved => "APPROVED",
            PaymentOrderStatus::Rejected => "REJECTED",
            PaymentOrderStatus::Expired => "EXPIRED",
        }
    }
}

impl std::str::FromStr for PaymentOrderStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "DRAFT" => Ok(PaymentOrderStatus::Draft),
            "PENDING_APPROVAL" => Ok(PaymentOrderStatus::PendingApproval),
            "APPROVED" => Ok(PaymentOrderStatus::Approved),
            "REJECTED" => Ok(PaymentOrderStatus::Rejected),
            "EXPIRED" => Ok(PaymentOrderStatus::Expired),
            _ => Err(format!("Unknown payment order status: '{s}'")),
        }
    }
}

/// Representation of a Treasury Payment Order (Ủy nhiệm chi).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentOrder {
    pub id: String,
    pub order_ref: String,
    pub debit_account: String,
    pub beneficiary_account: String,
    pub beneficiary_name: String,
    pub beneficiary_bank: String,
    /// Scaled integer amount in VND (0 floating point error).
    pub amount_vnd: u64,
    pub purpose: String,
    pub maker_id: String,
    #[serde(default)]
    pub checker_id: Option<String>,
    pub status: PaymentOrderStatus,
    #[serde(default)]
    pub hitl_token: Option<String>,
    #[serde(default)]
    pub signature_hmac: Option<String>,
    pub created_at: i64,
    #[serde(default)]
    pub approved_at: Option<i64>,
    #[serde(default)]
    pub rejection_reason: Option<String>,
}

/// Treasury Engine providing Maker-Checker workflow and persistence for payment orders.
#[derive(Debug, Default)]
pub struct TreasuryEngine;

impl TreasuryEngine {
    /// Maker proposes a new payment order. Generates a single-use UUIDv4 HITL token with 15-minute TTL.
    pub fn propose(
        debit_account: &str,
        beneficiary_account: &str,
        beneficiary_name: &str,
        beneficiary_bank: &str,
        amount_vnd: u64,
        purpose: &str,
        maker_id: &str,
    ) -> Result<PaymentOrder, String> {
        if amount_vnd == 0 {
            return Err("Payment order amount must be greater than 0 VND".to_string());
        }
        if debit_account.trim().is_empty() {
            return Err("Debit account cannot be empty".to_string());
        }
        if beneficiary_account.trim().is_empty() {
            return Err("Beneficiary account cannot be empty".to_string());
        }
        if maker_id.trim().is_empty() {
            return Err("Maker ID cannot be empty".to_string());
        }

        let now_ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let id = Uuid::new_v4().to_string();
        let short_id = &id[..8];
        let order_ref = format!("PO-{now_ts}-{short_id}");
        let hitl_token = Uuid::new_v4().to_string();

        Ok(PaymentOrder {
            id,
            order_ref,
            debit_account: debit_account.trim().to_string(),
            beneficiary_account: beneficiary_account.trim().to_string(),
            beneficiary_name: beneficiary_name.trim().to_string(),
            beneficiary_bank: beneficiary_bank.trim().to_string(),
            amount_vnd,
            purpose: purpose.trim().to_string(),
            maker_id: maker_id.trim().to_string(),
            checker_id: None,
            status: PaymentOrderStatus::PendingApproval,
            hitl_token: Some(hitl_token),
            signature_hmac: None,
            created_at: now_ts,
            approved_at: None,
            rejection_reason: None,
        })
    }

    /// Checker inspects an existing payment order. Checks expiration status.
    pub fn review(order: &mut PaymentOrder) {
        let now_ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        if order.status == PaymentOrderStatus::PendingApproval
            && now_ts > order.created_at + HITL_TOKEN_TTL_SECONDS
        {
            order.status = PaymentOrderStatus::Expired;
            order.hitl_token = None;
        }
    }

    /// Checker approves the payment order with Dual Control authorization.
    /// Strictly rejects self-approval (fail-closed).
    pub fn approve(
        order: &mut PaymentOrder,
        checker_id: &str,
        hitl_token: &str,
        audit_key: Option<&[u8; 32]>,
        conn: Option<&Connection>,
    ) -> Result<(), MakerCheckerError> {
        let trimmed_checker = checker_id.trim();
        if trimmed_checker.is_empty() {
            return Err(MakerCheckerError::EmptyCheckerId);
        }

        // 1. Strict Self-Approval Prevention (Circular 09/2020/TT-NHNN)
        if order.maker_id.eq_ignore_ascii_case(trimmed_checker) {
            return Err(MakerCheckerError::SelfApprovalProhibited {
                maker_id: order.maker_id.clone(),
            });
        }

        // 2. State verification
        if order.status != PaymentOrderStatus::PendingApproval {
            return Err(MakerCheckerError::InvalidProposalState {
                current: match order.status {
                    PaymentOrderStatus::Draft => crate::banking::compliance::maker_checker::ProposalStatus::Pending,
                    PaymentOrderStatus::PendingApproval => crate::banking::compliance::maker_checker::ProposalStatus::Pending,
                    PaymentOrderStatus::Approved => crate::banking::compliance::maker_checker::ProposalStatus::Approved,
                    PaymentOrderStatus::Rejected => crate::banking::compliance::maker_checker::ProposalStatus::Rejected,
                    PaymentOrderStatus::Expired => crate::banking::compliance::maker_checker::ProposalStatus::Expired,
                },
                expected: crate::banking::compliance::maker_checker::ProposalStatus::Pending,
            });
        }

        let now_ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        // 3. TTL Check (15 minutes)
        if now_ts > order.created_at + HITL_TOKEN_TTL_SECONDS {
            order.status = PaymentOrderStatus::Expired;
            order.hitl_token = None;
            return Err(MakerCheckerError::TokenExpired {
                expired_at: order.created_at + HITL_TOKEN_TTL_SECONDS,
                current_time: now_ts,
            });
        }

        // 4. Token match check
        match &order.hitl_token {
            Some(expected_token) if expected_token == hitl_token => {}
            Some(_) => return Err(MakerCheckerError::InvalidToken),
            None => {
                return Err(MakerCheckerError::TokenAlreadyUsed {
                    token: hitl_token.to_string(),
                });
            }
        }

        // 5. Compute cryptographic HMAC-SHA256 digital signature
        let resolved_key = resolve_audit_key(audit_key);
        let payload_hash = format!("{}:{}:{}", order.order_ref, order.amount_vnd, order.purpose);
        let signature = compute_audit_signature(
            &resolved_key,
            &order.id,
            &order.maker_id,
            trimmed_checker,
            CheckerDecision::Approve,
            now_ts,
            &payload_hash,
        );

        // 6. Transition state and consume token
        order.status = PaymentOrderStatus::Approved;
        order.checker_id = Some(trimmed_checker.to_string());
        order.approved_at = Some(now_ts);
        order.signature_hmac = Some(signature.clone());
        order.hitl_token = None; // Token consumed immediately (single-use)

        // 7. If DB connection provided, record to forward hash-chained audit ledger
        if let Some(c) = conn {
            let audit_payload = format!(
                "PAYMENT_ORDER_APPROVED|order_id={}|ref={}|amount={}|maker={}|checker={}|sig={}|ref={}",
                order.id, order.order_ref, order.amount_vnd, order.maker_id, trimmed_checker, signature, CIRCULAR_09_REF
            );
            let _ = AuditLedger::append(
                c,
                &resolved_key,
                "PAYMENT_ORDER_APPROVAL",
                trimmed_checker,
                &audit_payload,
            );
            let _ = update_payment_order(c, order);
        }

        Ok(())
    }

    /// Checker rejects the payment order.
    /// Strictly rejects self-rejection (maker cannot act as checker).
    pub fn reject(
        order: &mut PaymentOrder,
        checker_id: &str,
        hitl_token: &str,
        rejection_reason: &str,
        audit_key: Option<&[u8; 32]>,
        conn: Option<&Connection>,
    ) -> Result<(), MakerCheckerError> {
        let trimmed_checker = checker_id.trim();
        if trimmed_checker.is_empty() {
            return Err(MakerCheckerError::EmptyCheckerId);
        }

        if order.maker_id.eq_ignore_ascii_case(trimmed_checker) {
            return Err(MakerCheckerError::SelfApprovalProhibited {
                maker_id: order.maker_id.clone(),
            });
        }

        if order.status != PaymentOrderStatus::PendingApproval {
            return Err(MakerCheckerError::InvalidProposalState {
                current: crate::banking::compliance::maker_checker::ProposalStatus::Pending,
                expected: crate::banking::compliance::maker_checker::ProposalStatus::Pending,
            });
        }

        let now_ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        if now_ts > order.created_at + HITL_TOKEN_TTL_SECONDS {
            order.status = PaymentOrderStatus::Expired;
            order.hitl_token = None;
            return Err(MakerCheckerError::TokenExpired {
                expired_at: order.created_at + HITL_TOKEN_TTL_SECONDS,
                current_time: now_ts,
            });
        }

        match &order.hitl_token {
            Some(expected_token) if expected_token == hitl_token => {}
            Some(_) => return Err(MakerCheckerError::InvalidToken),
            None => {
                return Err(MakerCheckerError::TokenAlreadyUsed {
                    token: hitl_token.to_string(),
                });
            }
        }

        order.status = PaymentOrderStatus::Rejected;
        order.checker_id = Some(trimmed_checker.to_string());
        order.rejection_reason = Some(rejection_reason.trim().to_string());
        order.hitl_token = None;

        if let Some(c) = conn {
            let resolved_key = resolve_audit_key(audit_key);
            let audit_payload = format!(
                "PAYMENT_ORDER_REJECTED|order_id={}|ref={}|amount={}|maker={}|checker={}|reason={}",
                order.id, order.order_ref, order.amount_vnd, order.maker_id, trimmed_checker, rejection_reason
            );
            let _ = AuditLedger::append(
                c,
                &resolved_key,
                "PAYMENT_ORDER_REJECTION",
                trimmed_checker,
                &audit_payload,
            );
            let _ = update_payment_order(c, order);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// SQLite Persistence for Payment Orders
// ---------------------------------------------------------------------------

pub fn init_payment_orders_table(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS payment_orders (
            id TEXT PRIMARY KEY,
            order_ref TEXT NOT NULL,
            debit_account TEXT NOT NULL,
            beneficiary_account TEXT NOT NULL,
            beneficiary_name TEXT NOT NULL,
            beneficiary_bank TEXT NOT NULL,
            amount_vnd INTEGER NOT NULL,
            purpose TEXT NOT NULL,
            maker_id TEXT NOT NULL,
            checker_id TEXT,
            status TEXT NOT NULL,
            hitl_token TEXT,
            signature_hmac TEXT,
            created_at INTEGER NOT NULL,
            approved_at INTEGER,
            rejection_reason TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_payment_orders_status ON payment_orders(status);
        CREATE INDEX IF NOT EXISTS idx_payment_orders_maker ON payment_orders(maker_id);",
    )
}

pub fn save_payment_order(conn: &Connection, order: &PaymentOrder) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO payment_orders (
            id, order_ref, debit_account, beneficiary_account, beneficiary_name,
            beneficiary_bank, amount_vnd, purpose, maker_id, checker_id,
            status, hitl_token, signature_hmac, created_at, approved_at, rejection_reason
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
        params![
            order.id,
            order.order_ref,
            order.debit_account,
            order.beneficiary_account,
            order.beneficiary_name,
            order.beneficiary_bank,
            order.amount_vnd as i64,
            order.purpose,
            order.maker_id,
            order.checker_id,
            order.status.as_str(),
            order.hitl_token,
            order.signature_hmac,
            order.created_at,
            order.approved_at,
            order.rejection_reason,
        ],
    )?;
    Ok(())
}

pub fn update_payment_order(conn: &Connection, order: &PaymentOrder) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE payment_orders SET
            checker_id = ?1,
            status = ?2,
            hitl_token = ?3,
            signature_hmac = ?4,
            approved_at = ?5,
            rejection_reason = ?6
        WHERE id = ?7",
        params![
            order.checker_id,
            order.status.as_str(),
            order.hitl_token,
            order.signature_hmac,
            order.approved_at,
            order.rejection_reason,
            order.id,
        ],
    )?;
    Ok(())
}

pub fn find_payment_order_by_id(
    conn: &Connection,
    order_id: &str,
) -> Result<Option<PaymentOrder>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, order_ref, debit_account, beneficiary_account, beneficiary_name,
                beneficiary_bank, amount_vnd, purpose, maker_id, checker_id,
                status, hitl_token, signature_hmac, created_at, approved_at, rejection_reason
         FROM payment_orders WHERE id = ?1",
    )?;

    let mut rows = stmt.query(params![order_id])?;
    if let Some(row) = rows.next()? {
        let status_str: String = row.get(10)?;
        let status = PaymentOrderStatus::from_str(&status_str)
            .unwrap_or(PaymentOrderStatus::PendingApproval);
        let amount_i64: i64 = row.get(6)?;

        Ok(Some(PaymentOrder {
            id: row.get(0)?,
            order_ref: row.get(1)?,
            debit_account: row.get(2)?,
            beneficiary_account: row.get(3)?,
            beneficiary_name: row.get(4)?,
            beneficiary_bank: row.get(5)?,
            amount_vnd: amount_i64.max(0) as u64,
            purpose: row.get(7)?,
            maker_id: row.get(8)?,
            checker_id: row.get(9)?,
            status,
            hitl_token: row.get(11)?,
            signature_hmac: row.get(12)?,
            created_at: row.get(13)?,
            approved_at: row.get(14)?,
            rejection_reason: row.get(15)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn list_payment_orders(
    conn: &Connection,
    status_filter: Option<PaymentOrderStatus>,
) -> Result<Vec<PaymentOrder>, rusqlite::Error> {
    let query = if let Some(st) = status_filter {
        format!(
            "SELECT id, order_ref, debit_account, beneficiary_account, beneficiary_name,
                    beneficiary_bank, amount_vnd, purpose, maker_id, checker_id,
                    status, hitl_token, signature_hmac, created_at, approved_at, rejection_reason
             FROM payment_orders WHERE status = '{}' ORDER BY created_at DESC",
            st.as_str()
        )
    } else {
        "SELECT id, order_ref, debit_account, beneficiary_account, beneficiary_name,
                beneficiary_bank, amount_vnd, purpose, maker_id, checker_id,
                status, hitl_token, signature_hmac, created_at, approved_at, rejection_reason
         FROM payment_orders ORDER BY created_at DESC"
            .to_string()
    };

    let mut stmt = conn.prepare(&query)?;
    let rows = stmt.query_map([], |row| {
        let status_str: String = row.get(10)?;
        let status = PaymentOrderStatus::from_str(&status_str)
            .unwrap_or(PaymentOrderStatus::PendingApproval);
        let amount_i64: i64 = row.get(6)?;

        Ok(PaymentOrder {
            id: row.get(0)?,
            order_ref: row.get(1)?,
            debit_account: row.get(2)?,
            beneficiary_account: row.get(3)?,
            beneficiary_name: row.get(4)?,
            beneficiary_bank: row.get(5)?,
            amount_vnd: amount_i64.max(0) as u64,
            purpose: row.get(7)?,
            maker_id: row.get(8)?,
            checker_id: row.get(9)?,
            status,
            hitl_token: row.get(11)?,
            signature_hmac: row.get(12)?,
            created_at: row.get(13)?,
            approved_at: row.get(14)?,
            rejection_reason: row.get(15)?,
        })
    })?;

    let mut result = Vec::new();
    for r in rows {
        result.push(r?);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_treasury_maker_propose_order() {
        let order = TreasuryEngine::propose(
            "19034567890123",
            "0011001234567",
            "CONG TY TNHH THEP VIET NHAT",
            "VIETCOMBANK",
            500_000_000,
            "Thanh toan tien thep xay dung",
            "accountant_01",
        )
        .expect("Propose payment order");

        assert_eq!(order.amount_vnd, 500_000_000);
        assert_eq!(order.status, PaymentOrderStatus::PendingApproval);
        assert_eq!(order.maker_id, "accountant_01");
        assert!(order.checker_id.is_none());
        assert!(order.hitl_token.is_some());
        assert!(order.signature_hmac.is_none());
    }

    #[test]
    fn test_treasury_fail_closed_self_approval() {
        let mut order = TreasuryEngine::propose(
            "19034567890123",
            "0011001234567",
            "CONG TY THEP",
            "VIETCOMBANK",
            100_000_000,
            "Thanh toan hop dong",
            "accountant_01",
        )
        .expect("Propose");

        let token = order.hitl_token.clone().unwrap();

        // Accountant attempts self-approval
        let err = TreasuryEngine::approve(&mut order, "accountant_01", &token, None, None)
            .expect_err("Self-approval must fail");

        match err {
            MakerCheckerError::SelfApprovalProhibited { maker_id } => {
                assert_eq!(maker_id, "accountant_01");
            }
            _ => panic!("Expected SelfApprovalProhibited"),
        }
        assert_eq!(order.status, PaymentOrderStatus::PendingApproval);
    }

    #[test]
    fn test_treasury_successful_checker_approval_and_persistence() {
        let conn = Connection::open_in_memory().expect("In-memory sqlite");
        init_payment_orders_table(&conn).expect("Init table");
        AuditLedger::init_audit_table(&conn).expect("Init audit table");

        let mut order = TreasuryEngine::propose(
            "19034567890123",
            "0011001234567",
            "CONG TY THEP",
            "VIETCOMBANK",
            250_000_000,
            "Thanh toan hoa don HD-2026",
            "maker_accountant",
        )
        .expect("Propose");

        save_payment_order(&conn, &order).expect("Save order");
        let token = order.hitl_token.clone().unwrap();

        // Independent Checker approves
        TreasuryEngine::approve(&mut order, "chief_accountant", &token, None, Some(&conn))
            .expect("Approval succeeds");

        assert_eq!(order.status, PaymentOrderStatus::Approved);
        assert_eq!(order.checker_id.as_deref(), Some("chief_accountant"));
        assert!(order.approved_at.is_some());
        assert!(order.signature_hmac.is_some());
        assert!(order.hitl_token.is_none()); // Single-use token consumed

        // Verify loaded from DB
        let loaded = find_payment_order_by_id(&conn, &order.id)
            .expect("Query DB")
            .expect("Order exists");

        assert_eq!(loaded.status, PaymentOrderStatus::Approved);
        assert_eq!(loaded.checker_id.as_deref(), Some("chief_accountant"));
        assert_eq!(loaded.amount_vnd, 250_000_000);
        assert!(loaded.signature_hmac.is_some());
    }

    #[test]
    fn test_treasury_token_expiration() {
        let mut order = TreasuryEngine::propose(
            "19034567890123",
            "0011001234567",
            "CONG TY THEP",
            "VIETCOMBANK",
            100_000_000,
            "Thanh toan",
            "maker_accountant",
        )
        .expect("Propose");

        // Simulate created 901 seconds ago
        order.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            - 901;

        let token = order.hitl_token.clone().unwrap();
        let err = TreasuryEngine::approve(&mut order, "chief_accountant", &token, None, None)
            .expect_err("Expired token must fail");

        match err {
            MakerCheckerError::TokenExpired { .. } => {}
            _ => panic!("Expected TokenExpired"),
        }
        assert_eq!(order.status, PaymentOrderStatus::Expired);
    }

    #[test]
    fn test_treasury_empty_checker_id_rejected() {
        let mut order = TreasuryEngine::propose(
            "19034567890123",
            "0011001234567",
            "CONG TY THEP",
            "VIETCOMBANK",
            100_000_000,
            "Thanh toan",
            "maker_accountant",
        )
        .expect("Propose");

        let token = order.hitl_token.clone().unwrap();

        // Empty checker_id in approve
        let err_approve = TreasuryEngine::approve(&mut order, "", &token, None, None)
            .expect_err("Empty checker_id in approve must fail");
        assert_eq!(err_approve, MakerCheckerError::EmptyCheckerId);

        let err_approve_spaces = TreasuryEngine::approve(&mut order, "   ", &token, None, None)
            .expect_err("Whitespace checker_id in approve must fail");
        assert_eq!(err_approve_spaces, MakerCheckerError::EmptyCheckerId);

        // Empty checker_id in reject
        let err_reject = TreasuryEngine::reject(&mut order, "", &token, "reason", None, None)
            .expect_err("Empty checker_id in reject must fail");
        assert_eq!(err_reject, MakerCheckerError::EmptyCheckerId);

        let err_reject_spaces = TreasuryEngine::reject(&mut order, "   ", &token, "reason", None, None)
            .expect_err("Whitespace checker_id in reject must fail");
        assert_eq!(err_reject_spaces, MakerCheckerError::EmptyCheckerId);

        assert_eq!(order.status, PaymentOrderStatus::PendingApproval);
    }
}
