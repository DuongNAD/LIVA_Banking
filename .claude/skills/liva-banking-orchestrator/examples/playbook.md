# Orchestration Playbook: End-to-End Banking Operations DAG

This demonstration playbook guides the coordinated multi-stage execution of a complete banking operations cycle: Statement Ingestion -> Compliance Pre-Screening -> 3-Tier Reconciliation -> Solvency Risk Check -> Maker Payment Proposal -> Checker Cryptographic Approval.

## Scenario
At 08:30 AM, the treasury system initiates the morning reconciliation run:
1. Techcombank statement (`fixtures/statements/tcb_aug2026.csv`) is ingested.
2. Compliance screen ensures 0 data egress, screens for AML red flags, and redacts customer PII.
3. 3-Tier engine matches statement against ERP open invoices (`fixtures/erp_ledger/open_invoices.json`).
4. Risk engine confirms solvency buffer (DSCR $\ge 1.30$).
5. Maker initiates a payment order to settle an approved supplier invoice.
6. Checker validates the dual-control token and cryptographically approves the transfer.

---

## Stage 1: Compliance Pre-Screening
Tool: `compliance_aml_screen`

```json
{
  "tool": "compliance_aml_screen",
  "arguments": {
    "transactions": [
      {
        "tx_id": "TCB-TX-001",
        "account_number": "19034567890123",
        "counterparty_name": "CONG TY TNHH THEP VIET NHAT",
        "counterparty_account": "0011001234567",
        "amount_vnd": 250000000,
        "timestamp": 1726300000,
        "narration": "Thanh toan tien hang dot 1 HD-2026-088"
      }
    ],
    "redact_pii": true
  }
}
```
*Result*: `zero_egress_verified: true`, no statutory AML red flags, corporate entity preserved.

---

## Stage 2: 3-Tier Deterministic Reconciliation
Tool: `banking_reconcile`

```json
{
  "tool": "banking_reconcile",
  "arguments": {
    "statement_file_path": "fixtures/statements/tcb_aug2026.csv",
    "statement_format": "tcb_csv",
    "erp_ledger_path": "fixtures/erp_ledger/open_invoices.json",
    "fee_tolerance_vnd": 11000
  }
}
```
*Result*: Macro balance invariant valid. Matched: 42 exact, 14 fuzzy (Napas wire fees), 4 split payments. Zero mathematical drift.

---

## Stage 3: Solvency & Liquidity Verification
Tool: `credit_risk_scoring`

```json
{
  "tool": "credit_risk_scoring",
  "arguments": {
    "ebitda_vnd": 1800000000,
    "capex_vnd": 300000000,
    "debt_service_principal_vnd": 500000000,
    "debt_service_interest_vnd": 150000000,
    "cash_and_equivalents_vnd": 850000000,
    "marketable_securities_vnd": 0,
    "accounts_receivable_vnd": 600000000,
    "current_liabilities_vnd": 900000000,
    "forecast_days": 30
  }
}
```
*Result*: DSCR = $2.31\times$ (`HEALTHY`), Quick Ratio = $1.61\times$ (`STRONG`). Safe liquidity margin confirmed for vendor disbursement.

---

## Stage 4: Treasury Maker Payment Proposal
Tool: `treasury_payment_order` (`action: "propose"`)

```json
{
  "tool": "treasury_payment_order",
  "arguments": {
    "action": "propose",
    "debit_account": "19034567890123",
    "beneficiary_account": "0011001234567",
    "beneficiary_name": "CONG TY TNHH THEP VIET NHAT",
    "beneficiary_bank": "VIETCOMBANK",
    "amount_vnd": 250000000,
    "purpose": "Thanh toan tien hang dot 1 HD-2026-088",
    "maker_id": "accountant_maker_01"
  }
}
```
*Result*: `order_id: "ORD-20260914-DAG01"`, `status: "PENDING_APPROVAL"`, issued single-use HITL token `c4d3e2f1-a0b9-48c7-d6e5-f4a3b2c1d0e9` (TTL: 900s).

---

## Stage 5: Circular 09 Checker Authorization Gate
Tool: `treasury_payment_order` (`action: "approve"`)

```json
{
  "tool": "treasury_payment_order",
  "arguments": {
    "action": "approve",
    "order_id": "ORD-20260914-DAG01",
    "checker_id": "chief_accountant_checker_02",
    "hitl_token": "c4d3e2f1-a0b9-48c7-d6e5-f4a3b2c1d0e9"
  }
}
```
*Result*:
- Status transitioned to `APPROVED`.
- `maker_id` (`accountant_maker_01`) $\ne$ `checker_id` (`chief_accountant_checker_02`).
- Generated HMAC-SHA256 digital signature: `7a9f8b...`.
- Single-use token consumed and recorded in SQLite audit ledger.

---

## Stage 6: Orchestrator Consolidated Summary
```markdown
### Báo Cáo Tổng Hợp Điều Phối Tác Tử Ngân Hàng (Morning Operations DAG)

- **Giai đoạn 1 (Tuân thủ AML & PII)**: Đạt 100% Zero Data Egress; không phát hiện giao dịch đáng ngờ; PII được ẩn danh hóa.
- **Giai đoạn 2 (Đối soát sao kê)**: Khớp 60/60 giao dịch Techcombank với sổ cái ERP; chênh lệch số học = 0 VND.
- **Giai đoạn 3 (Thẩm định thanh khoản)**: DSCR đạt 2.31x (An toàn); Quick Ratio đạt 1.61x; bảo đảm thanh khoản trước giải ngân.
- **Giai đoạn 4 & 5 (Ủy nhiệm chi Maker-Checker)**:
  - Lệnh: `ORD-20260914-DAG01` (250.000.000 VND).
  - Maker: `accountant_maker_01` -> Checker: `chief_accountant_checker_02`.
  - Phê duyệt 2 vòng tuân thủ Thông tư 09/2020/TT-NHNN; chữ ký số HMAC-SHA256 đã lưu vào sổ cái kiểm toán.
- **Trạng thái chu trình**: HOÀN TẤT THÀNH CÔNG (CLOSED).
```
