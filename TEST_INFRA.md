# E2E Test Infra: Universal Corporate Banking & Treasury Harness

## 1. Test Philosophy & Principles
- **Opaque-Box & Requirement-Driven**: Tests are derived strictly from authoritative specifications in `ORIGINAL_REQUEST.md` (Follow-up 2026-09-14T17:59:25Z) and `PROJECT.md § Feature Inventory`.
- **Zero Module Coupling & Contract-Strict**: Tests validate observable behaviors through public interfaces, standard data fixtures (`data/demo_ready/`), and statutory compliance invariants.
- **Zero Floating-Point Drift Invariant**: All monetary computations operate on signed 64-bit integer values in VND cents. Floating-point rounding discrepancies are strictly forbidden ($0.00$ tolerance).
- **Statutory Regulatory Adherence**:
  - *Circular 09/2020/TT-NHNN*: Maker-Checker dual control gate (`maker != checker`), 15-minute token TTL, replay prevention.
  - *Circular 09/2023/TT-NHNN & Decision 11/2023/QĐ-TTg*: AML screening for high-value ($\ge 400$M VND), smurfing/structuring ($\ge 3$ transfers within 24h summing $\ge 400$M), rapid pass-through ($\ge 100$M drained $\ge 90\%$ in $< 30$ mins), and night velocity (23:00–05:00, $\ge 50$M).
  - *Decree 13/2023/NĐ-CP*: Personal Data Protection & Zero Data Egress (client-side execution, no cloud leakage).

---

## 2. Feature Inventory & Test Coverage Matrix

| # | Feature Code & Title | Primary Source | Tier 1 (Feature) | Tier 2 (Boundary) | Tier 3 (Pairwise) |
|---|----------------------|----------------|:----------------:|:-----------------:|:-----------------:|
| F01 | Universal Statement Dropzone | R1 §Ingestion | [x] 5 | [x] 5 | [x] |
| F02 | Multi-Bank Schema Normalization | R1 §Ingestion | [x] 5 | [x] 5 | [x] |
| F03 | Amount & Currency Integer Parser | R1 §Ingestion | [x] 5 | [x] 5 | [x] |
| F04 | Balance Invariant Validator | R1 §Ingestion | [x] 5 | [x] 5 | [x] |
| F05 | Tier 1 Exact Matcher | R1 §Reconciliation | [x] 5 | [x] 5 | [x] |
| F06 | Tier 2 Fuzzy Heuristic Matcher | R1 §Reconciliation | [x] 5 | [x] 5 | [x] |
| F07 | Tier 3 Subset-Sum Split Solver | R1 §Reconciliation | [x] 5 | [x] 5 | [x] |
| F08 | HITL Quarantine Management | R1 §Reconciliation | [x] 5 | [x] 5 | [x] |
| F09 | Vietnamese Intent Normalizer | R2 §Intelligence | [x] 5 | [x] 5 | [x] |
| F10 | Wire Fee Disentanglement | R2 §Intelligence | [x] 5 | [x] 5 | [x] |
| F11 | AML Structuring Detector | R2 §AML | [x] 5 | [x] 5 | [x] |
| F12 | AML High-Value Detector | R2 §AML | [x] 5 | [x] 5 | [x] |
| F13 | AML Rapid Pass-Through Detector | R2 §AML | [x] 5 | [x] 5 | [x] |
| F14 | AML Night-Time Velocity Detector | R2 §AML | [x] 5 | [x] 5 | [x] |
| F15 | Statutory Form STR Generator | R2 §AML | [x] 5 | [x] 5 | [x] |
| F16 | System Prompt Inspector UI | R2 §Intelligence | [x] 5 | [x] 5 | [x] |
| F17 | Maker-Checker Dual Control Gate | R1 §Treasury | [x] 5 | [x] 5 | [x] |
| F18 | Cryptographic Merkle Audit Proof | R1 §Treasury | [x] 5 | [x] 5 | [x] |
| F19 | 2D Conversational Copilot Drawer | R3 §Demo | [x] 5 | [x] 5 | [x] |
| F20 | 1-Click Guided Presentation Tour | R3 §Demo | [x] 5 | [x] 5 | [x] |
| F21 | Standalone Web Compilation & Hosting | R3 §Deployment | [x] 5 | [x] 5 | [x] |

---

## 3. Test Architecture & Runner Invocation

### 3.1 Invocation Commands
The test suite can be run directly via Node.js:

```powershell
# Run from repository root
node teamwork_projects/liva_banking_universal/tests/e2e/runner.mjs

# Or from project directory
cd teamwork_projects/liva_banking_universal
node tests/e2e/runner.mjs
# or
npm run test:e2e
```

### 3.2 Pass/Fail Semantics
- **Exit Code 0**: All test assertions evaluate truthy with zero failures and zero uncaught exceptions.
- **Exit Code 1**: Any test assertion fails, or a critical security/mathematical invariant is violated.
- **Fail-Closed Dual Control**: Any approval transaction where `makerId === checkerId` must throw an authorization error or be rejected.
- **Deterministic Math Invariant**: For all statements, $\text{Opening} + \sum \text{Credit} - \sum \text{Debit} == \text{Closing}$ must hold exactly.
- **Zero Data Leakage**: No network calls, telemetry egress, or external API dependencies.

---

## 4. Real-World Application Scenarios (Tier 4)

| Scenario ID | Scenario Name | Features Exercised | Description |
|---|---|---|---|
| **S01** | Multi-Bank Month-End Reconciliation Cycle | F01, F02, F03, F04, F05, F06, F07, F08, F10 | Ingest real VCB, TCB, BIDV statements and ledger CSV from `data/demo_ready/`. Reconcile records across 1:1, fuzzy (+-72h, fee deduction), and composite split matches with >= 99.8% match rate. |
| **S02** | AML Fraud Surveillance & STR Reporting Lifecycle | F11, F12, F13, F14, F15, F16 | Detect structuring/smurfing, high-value transfers, night-time anomalies, and rapid pass-through drain from suspicious BIDV/VCB data. Inspect prompt and generate Form STR report. |
| **S03** | High-Value Payment Maker-Checker & Merkle Chain | F17, F18, F12 | Create 550M VND payment order, enforce dual-control authorization gate, block self-approval, verify OTP/biometric approval, and record HMAC-SHA256 chained Merkle audit leaf. |
| **S04** | Unstructured Memo & Wire Fee Disentanglement Pipeline | F01, F03, F09, F10, F06 | Parse malformed and abbreviated Vietnamese memos ("ck tien may bom hd 88"), disentangle embedded wire fees (TK 6425), and link to ledger entries without LLM math hallucination. |
| **S05** | Executive Guided Demo Tour & Financial Copilot | F19, F20, F21 | Execute 5-minute automated walkthrough sequence across all views, test conversational copilot drawer with Vietnamese NLP queries, and verify standalone zero-backend web build readiness. |

---

## 5. Coverage Thresholds
- **Tier 1 (Feature Coverage)**: $21 \times 5 = 105$ tests (Happy path, standard specifications)
- **Tier 2 (Boundary & Corner Cases)**: $21 \times 5 = 105$ tests (Extreme values, zero/negative amounts, malformed data, security attacks)
- **Tier 3 (Cross-Feature Combinations)**: 21 tests (Pairwise feature integration)
- **Tier 4 (Real-World Scenarios)**: 5 comprehensive multi-bank workflows
- **Total Suite Minimum**: $\ge 236$ tests
