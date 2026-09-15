---
title: "P42_Reconciliation_Workbench_PRD"
tags:
  - liva/knowledge
  - liva/reconciliation
  - liva/workbench
  - liva/p42
author: "architect"
last_update: "2026-09-15T18:50:00Z"
---

# Product Requirements Document (PRD): P42 Reconciliation Workbench

## 1. Executive Summary & Business Objectives
- **Target Persona**: **R01 — Kế toán viên (Maker)** (Primary Operator), **R04 — Kế toán trưởng (Checker)** (Reviewer), **R09 — Kiểm toán nội bộ** (Read-Only Auditor).
- **Core Problem**: Traditional banking reconciliation requires manual cross-referencing between disjointed bank PDFs/Excel files and ERP general ledger screens, consuming 3–4 hours daily and introducing human rounding/transcription errors.
- **Mission**: Provide an ultra-responsive, synchronized **Three-Column Interactive Workbench** reducing daily reconciliation cycle time to **< 30 minutes/day**, achieving $\ge 90\%$ automated resolution, and guaranteeing 100% auditability for manual interventions.

---

## 2. Regulatory & Mathematical Invariants
- **VND Integer Arithmetic**: All monetary quantities must be handled as integer dong (`i64`), aggregated with `i128` and `checked_add`. **Zero floating-point arithmetic allowed on any financial execution path**.
- **Segregation of Duties (SoD)**: The Maker (R01) who prepares, manually pairs, or overrides matches within this workbench cannot approve the batch for ERP journal posting. The approval barrier must be verified at the database constraint level (`maker_user_id != checker_user_id`).
- **Decree 13/2023/NĐ-CP Compliance**: Natural person customer PII (Citizen IDs, personal bank account numbers, phone numbers) must be masked by default in transaction narrations, while enterprise vendor/customer identifiers are preserved.
- **Circular 09/2020/TT-NHNN Dual Control**: All manual match creations, overrides, fee splits, and unmatch events produce cryptographically sealed entries in an append-only hash-chain.

---

## 3. Screen Topology & 3-Column Layout

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ P42 RECONCILIATION WORKBENCH — BATCH: BATCH-20260915-VCB-01 (Status: IN_PROGRESS)                │
├───────────────────────────────┬────────────────────────────────┬─────────────────────────────────┤
│ COLUMN 1: BANK STATEMENT      │ COLUMN 2: ERP GENERAL LEDGER   │ COLUMN 3: MATCH RESULTS & DOCK  │
│ [Search Ref / VietQR] [Bank ▼]│ [Search Invoice / Partner]     │ [Filter: All | Diff != 0 | Fee] │
├───────────────────────────────┼────────────────────────────────┼─────────────────────────────────┤
│ • 15/09 09:15 | VCB           │ • 15/09 | INV-8821             │ ┌─────────────────────────────┐ │
│   +50,000,000 VND             │   +50,000,000 VND              │ │ [Tier 1: 100%] EXACT MATCH  │ │
│   Ref: FT26258912 | VietQR    │   Công ty TNHH An Phát         │ │ Bank: FT26258912 (+50M)     │ │
│   [Drag Handle] [Checkbox]    │   [Dropzone Target] [Checkbox] │ │ GL: INV-8821 (+50M)         │ │
│                               │                                │ │ Diff: 0 VND [Accept] [Break]│ │
│ • 15/09 10:30 | VCB           │ • 14/09 | INV-8829             │ └─────────────────────────────┘ │
│   -9,988,000 VND              │   -10,000,000 VND              │ ┌─────────────────────────────┐ │
│   Ref: FT26259001 | Wire      │   Công ty CP Cơ Khí Minh Tâm   │ │ [Tier 2: 95%] FEE CANDIDATE │ │
│   [Drag Handle] [Checkbox]    │   [Dropzone Target] [Checkbox] │ │ Bank: FT26259001 (-9,988k)  │ │
│                               │                                │ │ GL: INV-8829 (-10,000k)     │ │
│ • 15/09 11:00 | VCB           │ • 15/09 | INV-8835 (+12M)      │ │ Diff: -12,000 VND           │ │
│   +30,000,000 VND             │ • 15/09 | INV-8836 (+18M)      │ │ [Auto-Split Fee TK 6425]    │ │
│   Ref: FT26259110 | Batch In  │   [Dropzone Target]            │ └─────────────────────────────┘ │
├───────────────────────────────┴────────────────────────────────┴─────────────────────────────────┤
│ FOOTER BAR: Total Bank: 70,012,000 VND | Total GL: 70,000,000 VND | Unreconciled: 0 | [Submit]   │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Key Functional Requirements

### 4.1. Three-Column Interactive Display
- **Column 1 (Bank Statement Lines)**:
  - Displays unmatched and in-review bank statement lines.
  - Fields: Timestamp, Bank Code, Integer Amount (Debit / Credit color-coded), Transaction Reference, Description / VietQR memo, Running Balance.
  - Filtering: Text search (case-insensitive across Reference and Description), Bank account selector, Date range, Amount min/max.
  - Controls: Draggable card handle, multi-select checkboxes for composite split.
- **Column 2 (ERP General Ledger Open Invoices)**:
  - Displays open AP/AR invoices and cashbook records from ERP connectors (MISA, FAST, SAP).
  - Fields: Posting Date, Voucher/Invoice Number, Integer Amount (Debit / Credit), Partner / Counterparty Title, Tax ID, Cost Center.
  - Controls: Drop target zone, multi-select checkboxes for composite split.
- **Column 3 (Match Results & Staging Dock)**:
  - Shows match proposals grouped by confidence tiers:
    - **Tier 1 (Exact Match, 100%)**: Identical amount, identical normalized reference, date difference $\le 1$ day.
    - **Tier 2 (Fuzzy Heuristic, 80%–99%)**: Signal matrix matching (Date proximity $\pm 3$ days, partner name token Jaccard similarity $\ge 0.8$, wire fee tolerance).
    - **Tier 3 (Composite Split, 1:N / N:1)**: Results from the Split Solver.
    - **Manual Matches**: User drag-and-drop pairings.
  - Displays difference indicator:
    - Exactly $0\text{ VND}$: Highlighted in emerald green.
    - Fee candidate (difference between $1,100\text{ VND}$ and $22,000\text{ VND}$): Highlighted in amber with a 1-click `Tách phí ngân hàng (TK 6425)` action.
    - Material discrepancy ($\ne 0\text{ VND}$ and outside fee band): Highlighted in rose red with override justification prompt.

### 4.2. Drag-and-Drop & Pairing Interaction
- Dragging a Bank Card from Column 1 and dropping it onto a GL Card in Column 2 creates an immediate staged pair in Column 3.
- Dragging multiple items into the Column 3 dock activates the composite matching validator.

### 4.3. P45 Split Solver (1:N and N:1 Composite Matches)
- For consolidated payments (e.g., 1 bank credit of $30,000,000\text{ VND}$ settling multiple open invoices), the operator selects the parent transaction and clicks `Mở Split Solver`.
- The solver executes a deterministic subset-sum algorithm over open GL records bounded by $k \le 8$ items.
- Presents candidate combinations ranked by date proximity and counterparty match. The operator selects the verified combination with one click.

### 4.4. Reasoned Unmatch Protocol
- Unmatching an auto-paired or manually paired record is never silent.
- Clicking `Hủy đối soát (Unmatch)` triggers `UnmatchReasonModal`:
  - Category selection: `SAI_DOI_TAC`, `LECH_THOI_GIAN`, `LECH_PHI`, `HOA_DON_HUY`, `KHAC`.
  - Mandatory justification memo (minimum 10 characters).
  - Generates an immutable audit event in the SQLite WAL hash-chain:
    $$\text{Hash}_t = \text{SHA-256}(\text{Hash}_{t-1} \parallel \text{Timestamp} \parallel \text{OperatorID} \parallel \text{MatchID} \parallel \text{ReasonCode} \parallel \text{Memo})$$

### 4.5. Batch Submission & Checker Hand-off
- Once all discrepancies are resolved or placed into Quarantine (P44), the Maker clicks `Trình Kế toán trưởng duyệt`.
- The system verifies the double-entry invariant:
  $$\text{Closing Balance} - \text{Opening Balance} \equiv \sum \text{Credits} - \sum \text{Debits}$$
- The batch state transitions from `IN_PROGRESS` to `SUBMITTED_TO_CHECKER` and generates a single-use 15-minute HITL approval token.
