# Project: Universal Corporate Banking & Treasury Harness (liva_banking_universal)

## 1. Executive Summary
The Universal Corporate Banking & Treasury Harness (`teamwork_projects/liva_banking_universal`) is a production-ready, zero-backend-dependency static web application designed for Vietnamese corporate treasury and multi-bank operations. It complies strictly with Circular 09/2020/TT-NHNN (Maker-Checker Dual Control), Circular 09/2023/TT-NHNN & Decision 11/2023/QĐ-TTg (AML/CTF Fraud Surveillance), and Decree 13/2023/NĐ-CP (Personal Data Protection / Zero Data Egress).

The system features:
1. Client-side Universal Ingestion for Excel (`.xlsx`), CSV, and pasted tabular text from major banks (Vietcombank, Techcombank, BIDV, etc.).
2. 3-Tier Deterministic Reconciliation Engine achieving $\ge 99.8\%$ automated match rate with zero floating-point drift.
3. Local AI Intelligence Layer: Vietnamese colloquial transaction interpretation, embedded fee extraction (TK 6425), 4 AML anomaly detectors, and an Explainable System Prompt Inspector UI with 1-click Form STR filing.
4. Circular 09/2020 Maker-Checker payment authorization with biometric/OTP simulation and tamper-evident Merkle tree audit ledger.
5. Interactive 5-minute Demo Day presentation tour and 2D Financial Copilot drawer.

---

## 2. Architecture & Code Layout

### 2.1 Technology Stack
- **Framework**: Vue 3.5 (Composition API `<script setup lang="ts">`) + Vite 8.1 + TypeScript 5.8+.
- **State Management**: Pinia 4.0.
- **Styling**: UnoCSS / Tailwind-compatible styling.
- **Spreadsheet Parsing**: SheetJS (`xlsx` 0.18.5) client-side parsing.
- **Platform Architecture**: Polymorphic `IPlatformAdapter` with active in-memory `MockWebAdapter` state machine.

### 2.2 Code Layout (`teamwork_projects/liva_banking_universal/`)
```
teamwork_projects/liva_banking_universal/
├── index.html
├── package.json
├── vite.config.ts
├── tsconfig.json
├── vercel.json
├── public/
│   └── data/
│       ├── 01_VCB_SaoKe_Thang8_Chuan.xlsx
│       ├── 02_TCB_SaoKe_Thang8_Chuan.csv
│       ├── 03_BIDV_SaoKe_NgoaiGio_BatThuong.csv
│       ├── 04_SoNhatKy_Chung_Thang8.csv
│       └── 05_DuLieu_Dan_GoogleSheets.txt
└── src/
    ├── main.ts
    ├── App.vue
    ├── types/
    │   ├── banking.ts
    │   ├── reconciliation.ts
    │   ├── aml.ts
    │   └── adapter.ts
    ├── adapters/
    │   ├── IPlatformAdapter.ts
    │   └── MockWebAdapter.ts
    ├── engine/
    │   ├── ingestion/
    │   │   ├── universalParser.ts
    │   │   ├── delimiterSniffer.ts
    │   │   └── columnScorer.ts
    │   ├── reconciliation/
    │   │   ├── tier1ExactMatcher.ts
    │   │   ├── tier2FuzzyMatcher.ts
    │   │   ├── tier3SplitSolver.ts
    │   │   └── balanceValidator.ts
    │   ├── intelligence/
    │   │   ├── vietnameseNlp.ts
    │   │   ├── feeExtractor.ts
    │   │   └── amlSurveillance.ts
    │   └── treasury/
    │       ├── makerChecker.ts
    │       └── merkleAudit.ts
    ├── stores/
    │   ├── bankingStore.ts
    │   ├── reconciliationStore.ts
    │   ├── amlStore.ts
    │   └── treasuryStore.ts
    ├── components/
    │   ├── common/
    │   ├── ingestion/
    │   │   └── StatementDropzone.vue
    │   ├── reconciliation/
    │   │   ├── ReconciliationGauge.vue
    │   │   ├── MatchMatrix.vue
    │   │   └── HitlModal.vue
    │   ├── aml/
    │   │   ├── AmlAlertList.vue
    │   │   ├── SystemPromptInspector.vue
    │   │   └── StrReportModal.vue
    │   ├── treasury/
    │   │   ├── MakerCheckerModal.vue
    │   │   └── MerkleProofCard.vue
    │   └── copilot/
    │       ├── FinancialCopilotDrawer.vue
    │       └── GuidedTourOverlay.vue
    └── views/
        ├── BankingDashboardView.vue
        ├── ReconciliationWorkbenchView.vue
        ├── ComplianceAmlView.vue
        └── TreasuryPaymentView.vue
```

---

## 3. Feature Inventory

| # | Feature | Description | Milestone | Source |
|---|---------|-------------|-----------|--------|
| F01 | Universal Statement Dropzone | Client-side ingestion supporting `.xlsx`, `.csv`, and raw pasted text via drag-and-drop or file upload | M1 | Survey / R1 |
| F02 | Multi-Bank Schema Normalization | Heuristic candidate header scoring and canonical column mapping for VCB, TCB, BIDV, CTG, MBB, VBA, ISO 20022 | M1 | Survey / R1 |
| F03 | Amount & Currency Integer Parser | Robust parsing of European dot, English comma notations into signed 64-bit integer (`u64` VND cents) | M1 | Survey / R1 |
| F04 | Balance Invariant Validator | Strict mathematical check: $\text{Closing} = \text{Opening} + \sum \text{Credit} - \sum \text{Debit}$ and running step continuity | M1 | Survey / R1 |
| F05 | Tier 1 Exact Matcher | $O(1)$ reference and exact amount hash matcher ($\pm 24$h window, zero discrepancy) | M1 | Survey / R1 |
| F06 | Tier 2 Fuzzy Heuristic Matcher | Amount-bucket indexed matcher ($\pm 72$h effective date, Jaro-Winkler $\ge 0.85$, wire fee deduction) | M1 | Survey / R1 |
| F07 | Tier 3 Subset-Sum Split Solver | Bidirectional branch-and-bound subset-sum solver for 1:N and N:1 composite transactions ($k \le 8$, zero residual) | M1 | Survey / R1 |
| F08 | HITL Quarantine Management | Fail-closed quarantine for ambiguous records with single-use resolution tokens (15-min TTL) | M1 | Survey / R1 |
| F09 | Vietnamese Intent Normalizer | Tokenizer and dictionary normalizing colloquial bank memo abbreviations (`ck`, `tt`, `ung`, `hd`, `mst`, etc.) | M2 | Survey / R2 |
| F10 | Wire Fee Disentanglement | Automated extraction of interbank wire fees ($1,100 - 22,000$ VND) and allocation to TK 6425 without LLM math hallucination | M2 | Survey / R2 |
| F11 | AML Structuring Detector | Detection of $\ge 3$ sub-400M transfers within 24 hours summing $\ge 400$M VND (Circular 09/2023) | M2 | Survey / R2 |
| F12 | AML High-Value Detector | Mandatory detection and reporting of single transactions $\ge 400$M VND (Decision 11/2023/QĐ-TTg) | M2 | Survey / R2 |
| F13 | AML Rapid Pass-Through Detector | Inflow $\ge 100$M VND drained $\ge 90-95\%$ to counterparty accounts within $< 15-30$ minutes | M2 | Survey / R2 |
| F14 | AML Night-Time Velocity Detector | Anomaly detection for transactions between 23:00 and 05:00 with amount $\ge 50$M VND | M2 | Survey / R2 |
| F15 | Statutory Form STR Generator | 1-click export of Suspicious Transaction Reports adhering to Phụ lục II Thông tư 09/2023/TT-NHNN | M2 | Survey / R2 |
| F16 | System Prompt Inspector UI | Dual-pane compliance UI allowing live inspection, parameter tuning (thresholds, prompt), and test sandbox | M2 | Survey / R2 |
| F17 | Maker-Checker Dual Control Gate | Circular 09/2020 workflow (`DRAFT` -> `PENDING` -> `APPROVED`/`REJECTED` -> `SETTLED`), `maker != checker`, OTP/biometrics | M3 | Survey / R1 |
| F18 | Cryptographic Merkle Audit Proof | HMAC-SHA256 hash chaining and binary Merkle tree root display guaranteeing tamper-evident audit trail | M3 | Survey / R1 |
| F19 | 2D Conversational Copilot Drawer | Interactive natural language treasury assistant with domain intent classifier and contextual responses | M3 | Survey / R3 |
| F20 | 1-Click Guided Presentation Tour | Automated 5-minute Demo Day walkthrough sequence demonstrating all key features seamlessly | M3 | Survey / R3 |
| F21 | Standalone Web Compilation & Hosting | Zero-backend `MockWebAdapter` compilation, zero-error `npm run build`, and deployment configuration (`vercel.json`) | M4 | Survey / R3 |

---

## 4. Milestones

| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| M1 | Universal Statement Ingestion & 3-Tier Reconciliation Engine | F01, F02, F03, F04, F05, F06, F07, F08 | None | DONE |
| M2 | Local AI Intelligence, AML/STR Surveillance & System Prompt Inspector | F09, F10, F11, F12, F13, F14, F15, F16 | M1 Interface Contracts | DONE |
| M3 | Treasury Maker-Checker Dual Control & Interactive Demo Presentation | F17, F18, F19, F20 | M1, M2 Interface Contracts | DONE |
| M4 | Standalone Web Harness Scaffolding & Zero-Backend Integration | F21, Integration of M1-M3 into `liva_banking_universal/` | M1, M2, M3 | DONE |
| M5 | Final Verification, Adversarial Hardening & Acceptance | 100% E2E test suite pass, adversarial stress tests, Forensic Audit Gate | M4, E2E Testing Track | DONE |

---

## 5. Interface Contracts

### 5.1 Ingestion & Normalization (`src/engine/ingestion/`)
```typescript
export interface RawStatementRow {
  date: string;
  txCode: string;
  debit: number;
  credit: number;
  netAmount: number; // Signed integer (VND)
  balance: number;
  narration: string;
  counterparty?: string;
  bankCode: 'VCB' | 'TCB' | 'BIDV' | 'CTG' | 'MBB' | 'VBA' | 'GENERIC';
}

export interface StatementParseResult {
  bankCode: string;
  accountNumber?: string;
  openingBalance: number;
  closingBalance: number;
  transactions: RawStatementRow[];
  balanceInvariantPassed: boolean;
  rawRowCount: number;
}
```

### 5.2 Reconciliation Engine (`src/engine/reconciliation/`)
```typescript
export type MatchTier = 'TIER1_EXACT' | 'TIER2_FUZZY' | 'TIER3_SPLIT' | 'HITL_QUARANTINE';

export interface ReconciliationMatch {
  matchId: string;
  tier: MatchTier;
  bankTransactionIds: string[];
  ledgerEntryIds: string[];
  matchedAmount: number;
  feeAmount: number;
  confidence: number;
  explanation: string;
  timestamp: string;
}

export interface ReconciliationSummary {
  totalBankTransactions: number;
  totalLedgerEntries: number;
  matchedCount: number;
  matchRate: number; // Target >= 99.8%
  tier1Count: number;
  tier2Count: number;
  tier3Count: number;
  hitlQuarantineCount: number;
  totalMatchedAmount: number;
  totalFeeDisentangled: number;
}
```

### 5.3 AML & Surveillance Engine (`src/engine/intelligence/`)
```typescript
export type AmlAnomalyType = 
  | 'STRUCTURING_SMURFING'
  | 'HIGH_VALUE'
  | 'RAPID_PASS_THROUGH'
  | 'NIGHT_VELOCITY'
  | 'WATCHLIST_HIT';

export interface AmlAlert {
  alertId: string;
  anomalyType: AmlAnomalyType;
  severity: 'MEDIUM' | 'HIGH' | 'CRITICAL';
  involvedTransactionIds: string[];
  totalAmount: number;
  detectedAt: string;
  reasoning: string;
  statutoryRuleRef: string; // e.g., "Thông tư 09/2023/TT-NHNN Điều 3"
  suggestedStrReport: boolean;
}

export interface FormStrData {
  reportingEntity: string;
  reportDate: string;
  alertType: AmlAnomalyType;
  suspectAccount: string;
  suspectName: string;
  transactionCount: number;
  totalVndAmount: number;
  narrativeSummary: string;
  complianceOfficerNotes: string;
}
```

### 5.4 Maker-Checker Dual Control (`src/engine/treasury/`)
```typescript
export type VoucherStatus = 'DRAFT' | 'PENDING_APPROVAL' | 'APPROVED' | 'REJECTED' | 'SETTLED' | 'EXPIRED';

export interface PaymentVoucher {
  voucherId: string;
  makerId: string;
  checkerId?: string;
  beneficiaryAccount: string;
  beneficiaryBank: string;
  amountVnd: number;
  purpose: string;
  status: VoucherStatus;
  createdAt: string;
  approvedAt?: string;
  authMethod?: 'BIOMETRIC_SIM' | 'OTP_SMS';
  merkleLeafHash?: string;
}
```

---

## 6. Verification & Quality Gates
- **Sequential Execution Only**: Run build, check, and test commands sequentially.
- **Strict Lint & Typecheck**: `vue-tsc --noEmit` and `npm run build` must complete with 0 errors.
- **E2E Acceptance Gate**: Full E2E test suite (Tiers 1-4) must achieve 100% pass rate.
- **Forensic Auditor Gate**: Forensic Auditor verification is a mandatory binary veto.
