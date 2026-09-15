---
title: "P50_P53_Ledger_ERP_Posting_PRD"
tags:
  - liva/knowledge
  - liva/ledger
  - liva/erp-posting
  - liva/p50
  - liva/p51
  - liva/p52
  - liva/p53
author: "architect"
last_update: "2026-09-15T20:05:00Z"
---

# Product Requirements Document (PRD): P50–P53 Accounting Ledgers & Automated ERP Posting

## 1. Executive Summary & Regulatory Mandate
- **Target Persona**: **R01 — Kế toán viên** (Journal Proposer), **R04 — Kế toán trưởng** (Posting Authorizer), **R05 — Quản trị hệ thống ERP** (Sync Administrator), **R09 — Kiểm toán nội bộ** (Ledger Auditor).
- **Regulatory Framework**:
  - Vietnam Accounting System (Circular 200/2014/TT-BTC & Circular 133/2016/TT-BTC).
  - Decree 123/2020/NĐ-CP on Electronic Invoices & Source Vouchers.
  - Zero Data Egress: Enterprise ERP credentials, token payloads, and customer financial ledgers reside exclusively on-premise.
- **Mission**: Automate the synthesis of balanced double-entry accounting entries ($\sum \text{Debit} \equiv \sum \text{Credit}$) from confirmed reconciliation matches and quarantine releases, enabling one-click or automated posting to enterprise ERPs (MISA AMIS, FAST Business Online, BRAVO) with strict idempotency keys (`voucher_guid`).

---

## 2. Core Modules Architecture (P50–P53)

```
   [Confirmed Matches P42 / Released Quarantine P44]
                          │
                          ▼
            ┌───────────────────────────┐
            │  P50: Journal Generator   │ ── Balanced Double-Entry Rule Engine
            └─────────────┬─────────────┘    (TK 1121, 131, 331, 6425, 1388, 3388)
                          │
                          ▼
            ┌───────────────────────────┐
            │  P51: ERP Payload Bridge  │ ── MISA AMIS / FAST Payload Serializer
            └─────────────┬─────────────┘    (Idempotency Key: UUIDv4 + Hash)
                          │
                          ▼
            ┌───────────────────────────┐
            │  P52: Sync Monitor & DLQ  │ ── Status: DRAFT -> POSTED / FAILED
            └─────────────┬─────────────┘    (Exponential backoff retry)
                          │
                          ▼
            ┌───────────────────────────┐
            │  P53: Balance Certificate │ ── Bank vs GL Audit Invariant
            └───────────────────────────┘    (Closing Bank - In-Transit == Closing GL)
```

---

## 3. Account Allocation Matrix (Circular 200/2014/TT-BTC)

| Transaction Type | Debit Account (Nợ) | Credit Account (Có) | Rationale |
|---|---|---|---|
| Customer Wire Receipt (1:1 Exact) | `1121` (Tiền gửi NH) | `131` (Phải thu khách hàng) | Clean settlement of sales invoice. |
| Vendor Wire Payment (1:1 Exact) | `331` (Phải trả người bán) | `1121` (Tiền gửi NH) | Supplier invoice disbursement. |
| Customer Receipt with Wire Fee | `1121` (Net bank amount)<br>`6425` (Wire fee amount) | `131` (Gross invoice amount) | Intermediary bank fee absorbed by business. |
| Composite Split (1 Bank : $N$ GL) | `1121` (Single lump sum) | $N \times$ `131` (Individual invoices) | Bounded subset-sum invoice liquidation ($k \le 8$). |
| Quarantined Suspense Receipt | `1121` (Unidentified funds) | `3388` (Phải trả khác - Tạm treo) | Funds received with ambiguous or missing memo. |
| Quarantined Wire Chargeback | `1388` (Phải thu khác) | `1121` (Disbursed bank funds) | Chargeback under dispute investigation. |

---

## 4. Idempotency & Zero Double-Posting Guardrail

To prevent duplicate voucher registration in MISA AMIS and FAST Business Online under network retries:
1. **Deterministic Idempotency Key**:
   $$\text{IdempotencyKey} = \text{SHA-256}(\text{BatchID} \parallel \text{MatchID} \parallel \text{BankTxCode} \parallel \text{GrossAmount})$$
2. **ERP Header Tagging**:
   - MISA AMIS: `VoucherGUID: string (UUIDv4 derived)`
   - FAST: `<VoucherID>...</VoucherID>`
3. **Database Uniqueness**:
   - `sqlite_stat` and SQLite WAL enforce `UNIQUE(idempotency_key)` in the ledger journal event store.

---

## 5. UI/UX Specifications (P50–P53 Integrated View)

1. **Journal Entry Proposals Table**:
   - Filter by status (`CHỜ HẠCH TOÁN`, `ĐÃ ĐỒNG BỘ ERP`, `LỖI ĐỒNG BỘ`).
   - Voucher number, accounting date, debit accounts, credit accounts, total amount, partner name.
   - Difference validator indicator (always $0\text{ VND}$).
2. **ERP Integration Actions**:
   - Target ERP selector: `MISA AMIS`, `FAST Business Online`, `BRAVO ERP`.
   - Action buttons: `Đồng Bộ Hàng Loạt (Batch Post)`, `Xuất XML/JSON MISA`, `Xem Chi Tiết Định Khoản`.
3. **P53 Balance Reconciliation Certificate Tab**:
   - Compares:
     - Bank Statement Closing Balance
     - Ledger Closing Balance (TK 1121)
     - Outstanding In-Transit Items
     - Unreconciled Variance (Must be $0\text{ VND}$)
   - Cryptographic Audit Attestation Stamp with SHA-256 Merkle root.
