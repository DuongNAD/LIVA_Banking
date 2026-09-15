# Treasury Playbook: Maker-Checker Payment Order Authorization Flow

This demonstration playbook guides the step-by-step creation, dual-control validation, self-approval defense, and cryptographic signoff of a corporate payment order according to SBV Circular 09/2020/TT-NHNN.

## Scenario
An accountant (Maker: `accountant_duong`) prepares an electronic payment order to disburse 150,000,000 VND to a steel supplier. The Chief Accountant (Checker: `chief_accountant_hieu`) must independently verify and authorize the payment with a single-use HITL token.

---

## Step 1: Maker Proposes Payment Order
The accountant submits the transfer details:

```json
{
  "tool": "treasury_payment_order",
  "arguments": {
    "action": "propose",
    "debit_account": "19034567890123",
    "beneficiary_account": "0011001234567",
    "beneficiary_name": "CONG TY TNHH THEP VIET NHAT",
    "beneficiary_bank": "VIETCOMBANK",
    "amount_vnd": 150000000,
    "purpose": "Thanh toan tien thep theo hoa don HD-2026-088",
    "maker_id": "accountant_duong"
  }
}
```

### Tool Response:
```json
{
  "order_id": "ORD-20260914-7F8A",
  "status": "PENDING_APPROVAL",
  "maker_id": "accountant_duong",
  "checker_id": null,
  "hitl_token": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
  "signature_hmac": null,
  "created_at": 1726300000,
  "approved_at": null,
  "error": null
}
```

---

## Step 2: Adversarial Simulation — Self-Approval Rejection
The accountant attempts to approve their own voucher using their credentials:

```json
{
  "tool": "treasury_payment_order",
  "arguments": {
    "action": "approve",
    "order_id": "ORD-20260914-7F8A",
    "checker_id": "accountant_duong",
    "hitl_token": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d"
  }
}
```

### Tool Response (Fail-Closed Rejection):
```json
{
  "order_id": null,
  "status": "ERROR",
  "maker_id": null,
  "checker_id": null,
  "hitl_token": null,
  "signature_hmac": null,
  "created_at": null,
  "approved_at": null,
  "error": "Payment order approval rejected: Circular 09 Violation: Self-approval strictly prohibited for Maker 'accountant_duong' (Fail-Closed Dual Control)"
}
```
*Verification*: The order remains `PENDING_APPROVAL`, and the token remains unconsumed.

---

## Step 3: Independent Checker Authorization
The Chief Accountant reviews the order and signs approval within the 15-minute window:

```json
{
  "tool": "treasury_payment_order",
  "arguments": {
    "action": "approve",
    "order_id": "ORD-20260914-7F8A",
    "checker_id": "chief_accountant_hieu",
    "hitl_token": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d"
  }
}
```

### Tool Response (Approved & Cryptographically Signed):
```json
{
  "order_id": "ORD-20260914-7F8A",
  "status": "APPROVED",
  "maker_id": "accountant_duong",
  "checker_id": "chief_accountant_hieu",
  "hitl_token": null,
  "signature_hmac": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "created_at": 1726300000,
  "approved_at": 1726300120,
  "error": null
}
```

---

## Step 4: Token Replay Prevention Verification
Attempting to resubmit the approval with the consumed token:

```json
{
  "tool": "treasury_payment_order",
  "arguments": {
    "action": "approve",
    "order_id": "ORD-20260914-7F8A",
    "checker_id": "cfo_minh",
    "hitl_token": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d"
  }
}
```

### Tool Response:
```json
{
  "order_id": null,
  "status": "ERROR",
  "maker_id": null,
  "checker_id": null,
  "hitl_token": null,
  "signature_hmac": null,
  "created_at": null,
  "approved_at": null,
  "error": "Payment order approval rejected: Invalid Proposal State: current Approved, expected Pending"
}
```
*Verification*: Single-use token cannot be re-executed. Audit trail is permanent and immutable.
