---
title: "P44_Quarantine_Queue_PRD"
tags:
  - liva/knowledge
  - liva/reconciliation
  - liva/quarantine
  - liva/p44
  - liva/sod
author: "architect"
last_update: "2026-09-15T19:30:00Z"
---

# Product Requirements Document (PRD): P44 Quarantine Queue & Dual Control Hub

## 1. Executive Summary & Regulatory Mandate
- **Target Persona**: **R01 — Kế toán viên (Maker)** (Investigator/Proposer), **R04 — Kế toán trưởng (Checker)** (Authorizer), **R08 — AML/Compliance** (Monitor), **R09 — Kiểm toán nội bộ** (Attestor).
- **Core Problem**: Financial transactions that fail automated matching (Tier 1–3) or trigger regulatory red flags cannot be silently ignored or arbitrarily forced into the ledger.
- **Mission**: Provide an isolated, tamper-evident **Quarantine Queue (P44)** governed by the **Circular 09/2020/TT-NHNN Dual Control (4-Eyes Principle)** and **Decree 13/2023/NĐ-CP Audit Accountability**. Every exception must follow a verifiable two-phase proposal and approval lifecycle with single-use cryptographic tokens.

---

## 2. Segregation of Duties (SoD) & Fail-Closed Guardrails

1. **Non-Negotiable Maker-Checker Invariant**:
   $$\text{maker\_user\_id} \ne \text{checker\_user\_id}$$
   - Any attempt by the same user to approve their own proposed resolution is blocked at both the UI layer and the backend state machine.
2. **Single-Use UUIDv4 Token with 15-Minute TTL**:
   - When a transaction enters the Quarantine queue or a resolution is proposed, the system issues a UUIDv4 token with a strict 900-second time-to-live ($t \le 15\text{ mins}$).
   - If the Checker does not act within the TTL window, the token state transitions to `EXPIRED_ESCALATED`, preventing stale or hijacked authorizations.
3. **Mandatory Reason & Audit Hash-Chain**:
   - Neither Maker proposals nor Checker decisions can be submitted with empty notes (minimum 10 characters required).
   - All actions produce an immutable event block in the SQLite WAL hash-chain:
     $$\text{Hash}_t = \text{SHA-256}(\text{Hash}_{t-1} \parallel \text{Timestamp} \parallel \text{OperatorID} \parallel \text{TokenUUID} \parallel \text{Decision} \parallel \text{ReasonNote})$$

---

## 3. Exception Classification Taxonomy

| Exception Code | Severity | Trigger Criteria | Action Protocol |
|---|---|---|---|
| `AML_HIGH_VALUE` | **CRITICAL** | Transaction amount $\ge 400,000,000\text{ VND}$ (Decision 11/2023/QĐ-TTg) | Mandatory compliance screening; requires explicit Checker signoff before ledger entry. |
| `FEE_OVER_LIMIT` | **MEDIUM** | Wire fee discrepancy $> 22,000\text{ VND}$ | Operator must verify whether sender or receiver bears abnormal intermediary bank charges. |
| `UNKNOWN_PARTNER` | **HIGH** | Narration contains no recognized customer or vendor keywords | Operator must manually identify partner Tax ID or bank account number. |
| `DUPLICATE_REF` | **HIGH** | Transaction reference code has appeared in a previously reconciled batch | Potential duplicate bank debit/credit; requires verification against bank statement archives. |
| `REVERSAL_SUSPECT` | **CRITICAL** | Exact reverse amount of a transaction within 48 hours | Suspected transaction reversal; must be paired with original charge before release. |

---

## 4. State Transition Life Cycle

```
       [Transaction Fails Tier 1-3 or Triggers Flag]
                            │
                            ▼
                    ┌───────────────┐
                    │ PENDING_MAKER │
                    └───────┬───────┘
                            │ Maker investigates & proposes GL allocation
                            ▼
                ┌───────────────────────┐
                │ SUBMITTED_TO_CHECKER  │ <─── Token UUIDv4 (TTL: 15 mins)
                └───────┬───────┬───────┘
                        │       │
       Checker Approves │       │ Checker Rejects
      (checker != maker)│       │ (checker != maker)
                        ▼       ▼
                  ┌──────────┐ ┌──────────┐
                  │ APPROVED │ │ REJECTED │
                  └────┬─────┘ └──────────┘
                       │
                       ▼
        [Release to P50 Journal Proposed]
```

---

## 5. UI/UX Specifications

- **Split Master-Detail View**:
  - **Left Pane (Quarantine List)**: Filterable list with severity badges, transaction codes, bank amounts, and status tags (`Chờ Maker đề xuất`, `Chờ KTT duyệt`, `Đã duyệt`, `Đã từ chối`).
  - **Right Pane (Investigation & Dual Control)**:
    - Transaction Forensics (Bank code, amount, raw narration, timestamp, account number).
    - Exception Diagnostics (Why it was quarantined, rule triggered).
    - **Maker Workspace (Active when user is Maker)**: Allocation dropdown (TK 6425 Chi phí NH, TK 1388 Phải thu khác, TK 3388 Phải trả khác), justification memo field, and `Trình Duyệt KTT` button.
    - **Checker Workspace (Active when user is Checker)**: Proposed allocation summary, Maker's note, TTL countdown timer ($900$s), SoD validation indicator, and 2 action buttons (`Phê duyệt`, `Bác bỏ`).
- **Interactive Role Switcher**: Top toolbar includes a toggle between **R01 Kế toán viên (Maker)** and **R04 Kế toán trưởng (Checker)** for testing and demonstrating the 4-Eyes principle.
