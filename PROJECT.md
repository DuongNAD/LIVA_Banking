# Project: LIVA Banking Milestone M1 & M2 Harness Architecture

## Architecture
Monorepo workspace modularization separating financial calculations, cryptographic audit logging, statement parsing, normalization, local SLM NER memo extraction, matching, and system hardening into isolated Rust crates under `crates/`.

```
crates/
├── liva-money/        # Strict minor-unit (cents) integer arithmetic, Banker's rounding, zero float drift
├── liva-ledger/       # Double-entry ledger state machine, Circular 200/2014/TT-BTC, balance invariants
├── liva-audit/        # RFC 6962 Binary Merkle Tree, 0x00 leaf / 0x01 node prefix, inclusion proof, HMAC-SHA256
├── liva-normalize/    # ISO 8601 dates, VND u64 minor units, uppercase unaccented names, reference extraction
├── liva-ingest/       # Multi-bank sniffing, XLSX/CSV/CAMT.053/MT940/PDF parsers for VCB, TCB, BIDV, CTG, MBB, VBA
├── liva-match/        # Tier 1 O(1) hash matcher (±24h), Tier 2 fuzzy Jaro-Winkler + Fee Splitter TK 6425 (1.1k-22k VND)
└── liva-nlp/          # Local SLM NER memo extractor (llama.cpp GGUF Q4_K_M), GBNF JSON grammar, HITL queue (<0.6)
```

## Code Layout
- `Cargo.toml`: Workspace manifest declaring all crates in `crates/` and `liva-native-core`, `liva-desktop/src-tauri`.
- `crates/liva-money`:
  - `src/lib.rs`: `Money`, `Currency`, `checked_mul_ratio`, `allocate`, `to_vietnamese_display`.
  - `tests/proptest_invariants.rs`: Proptest suite (>= 100,000 cases).
- `crates/liva-ledger`:
  - `src/lib.rs`: `JournalEntry`, `JournalLine`, `PostingType`, `verify_statement_balance`.
  - `tests/proptest_ledger.rs`: Double-entry and statement balance invariants.
- `crates/liva-audit`:
  - `src/lib.rs`: Re-exports `merkle` and `chain`.
  - `src/merkle.rs`: RFC 6962 Merkle Tree, `MerkleInclusionProof`, second-preimage attack resistance.
  - `src/chain.rs`: HMAC-SHA256 append-only tamper-evident audit ledger.
  - `tests/audit_tamper_tests.rs`: 1-byte tamper detection tests.
- `crates/liva-normalize`:
  - `src/lib.rs`: `NormalizedTransaction`, `NormalizedDate`, `normalize_vietnamese_text`.
  - `src/cleaner.rs`: Unaccented uppercase folding, corporate legal noise removal.
- `crates/liva-ingest`:
  - `src/lib.rs`: `StatementParser`, `StatementSniffer`, container format detection.
  - `src/parsers/`: VCB (Excel merged cells), TCB (CSV semicolon), BIDV (PDF vector text), VietinBank, MBBank, Agribank, CAMT.053 XML (`quick-xml`), SWIFT MT940 FSM.
- `crates/liva-match`:
  - `src/lib.rs`: `ReconciliationEngine`, `ReconciliationMatch`.
  - `src/tier1_exact.rs`: Exact match O(1) HashMap in ±24h.
  - `src/tier2_fuzzy.rs`: Jaro-Winkler fuzzy match (>= 0.85).
  - `src/fee_splitter.rs`: Fee Splitter (1,100 to 22,000 VND + VAT) booking to TK 6425.
- `crates/liva-nlp`:
  - `src/lib.rs`: `LocalNlpExtractor`, `ExtractedMemo`, `MemoGrammar`.
  - `src/grammar.rs`: GBNF grammar for JSON schema (`invoice_no`, `order_no`, `partner_name`, `fee_flag`, `confidence`).
  - `src/hitl.rs`: HITL queue dispatcher for confidence < 0.6.
- `fixtures/statements/`:
  - `vcb_aug2026.xlsx`, `tcb_aug2026.csv`, `bidv_aug2026.pdf`, `vcb_adversarial_merged.xlsx`.
  - `camt053_aug2026.xml` (ISO 20022 CAMT.053 fixture).
  - `mt940_aug2026.txt` (SWIFT MT940 fixture).
- `fixtures/erp_ledger/open_invoices.json`.

## Feature Inventory
| # | Feature | Description | Milestone | Source |
|---|---------|-------------|-----------|--------|
| 1 | Proptest >= 100k Cases | Expand `liva-money` proptest cases to >= 100,000 samples to verify zero overflow & zero float drift | M1 | ORIGINAL_REQUEST §R1, Criteria 39 |
| 2 | Crates Modularization: liva-audit | Extract RFC 6962 Binary Merkle Tree & HMAC-SHA256 into `crates/liva-audit` | M1 | ORIGINAL_REQUEST §R1 |
| 3 | Crates Modularization: liva-match | Extract Tier 1 Exact O(1) in ±24h & Tier 2 Fuzzy Jaro-Winkler into `crates/liva-match` | M1 | ORIGINAL_REQUEST §R1 |
| 4 | Fee Splitter TK 6425 | Fee Splitter recognizing 1,100 - 22,000 VND bank fees (+VAT) and booking to TK 6425 | M1 | ORIGINAL_REQUEST §R1, Criteria 46 |
| 5 | Statement Sniffer & 6 Bank Parsers | Ingestion for VCB (XLSX), TCB (CSV), BIDV (PDF), VietinBank, MBBank, Agribank | M2 | ORIGINAL_REQUEST §R2 |
| 6 | CAMT.053 XML Parser & Fixture | ISO 20022 XML statement ingestion and physical fixture `camt053_aug2026.xml` | M2 | ORIGINAL_REQUEST §R2 |
| 7 | SWIFT MT940 Parser & Fixture | SWIFT MT940 FSM statement parser and physical fixture `mt940_aug2026.txt` | M2 | ORIGINAL_REQUEST §R2 |
| 8 | Transaction Normalization Engine | Normalize to ISO 8601, VND u64 minor units, uppercase unaccented names in `crates/liva-normalize` | M2 | ORIGINAL_REQUEST §R2 |
| 9 | Local SLM GBNF Grammar NER | Local SLM inference with GBNF constrained JSON (`invoice_no`, `order_no`, partner, `fee_flag`, `confidence`) | M3 | ORIGINAL_REQUEST §R3, Criteria 49 |
| 10| SLM HITL Confidence Routing | Auto-flag transactions with confidence < 0.6 to HITL review queue | M3 | ORIGINAL_REQUEST §R3 |
| 11| Statement Balance Invariant Verifier | Validate `closing_cents == opening_cents + SUM(credit) - SUM(debit)` across all statements | M4 | ORIGINAL_REQUEST §R4, Criteria 40 |
| 12| Merkle 1-Byte Tamper Test | Verify 1-byte tamper in transaction data or inclusion proof fails verification 100% | M4 | ORIGINAL_REQUEST §R4, Criteria 41 |
| 13| Golden Fixture Statement Tests | Parse 100% fixtures (VCB, TCB, BIDV, CAMT.053, MT940) with 0 dropped rows & 0 amount errors | M4 | ORIGINAL_REQUEST §R4, Criteria 44 |
| 14| Reconciler Precision >= 97% | Match engine Tier 1 + Tier 2 benchmark on `open_invoices.json` achieving >= 97% precision | M4 | ORIGINAL_REQUEST §R4, Criteria 45 |
| 15| Zero-Egress Guard 127.0.0.1 | Network allowlist strictly blocking external connections, allowing only 127.0.0.1 / ::1 | M4 | ORIGINAL_REQUEST §R4, Criteria 51 |
| 16| SQLCipher AES-256 Storage Config | Database storage configuration for SQLCipher AES-256 full database encryption | M4 | ORIGINAL_REQUEST §R4 |

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| M1 | `liva-core::Money` Checked Arithmetic & Period Balance Continuity Invariant | Strict minor-unit integer arithmetic, Banker's rounding, zero float drift, `#![deny(clippy::float_arithmetic)]`, `PeriodBalance` continuity | none | **DONE (PASS)** (45 tests, 130k proptests, 0 warnings, clean audit) |
| M2 | Database Schema, Migrations, Triggers & Audit Trail | 10 banking entities, `quarantine_items` maker-checker check, append-only triggers blocking UPDATE/DELETE/REPLACE, `client_ip` & `signature` hash chain binding | M1 | **DONE (PASS)** (36 tests pass, 0 warnings, clean audit) |
| M3 | `apps/liva-server` Axum Service & Centralized DB Pool | Axum 0.7, Tokio, centralized SQLx pool in `AppState`, `/health` endpoint with DB ping, modular router, Zero-Egress loopback binding | M2 | **DONE (PASS)** (11 tests pass, 0 warnings, clean audit) |
| M4 | Multi-Format Ingest & Deduplication Engine | `crates/liva-ingest` dynamic `BankProfile` (calamine XLSX + CSV), `statement_fingerprint` & `txn_hash` deduplication | M3 | **IN_PROGRESS** |
| M5 | Full Workspace Integration Gate & Final Victory Audit | Full integration verification, RAM guardrails (-j 2, --test-threads 2, sequential), Zero-Egress, final victory audit | M1, M2, M3, M4 | **PLANNED** |

## Interface Contracts

### `liva-money` ↔ `liva-ledger` & `liva-match`
- Types: `Money { amount: i64, currency: Currency }`, `Currency { VND, USD, EUR }`.
- Invariant: `#![deny(clippy::float_arithmetic)]`, checked arithmetic returning `Result<Money, MoneyError>`.
- Remainder distribution: `allocate(ratios: &[u32]) -> Result<Vec<Money>, MoneyError>` with zero penny drift.

### `liva-audit` ↔ System Compliance
- Prefix standards: RFC 6962 leaf prefix `0x00`, internal node prefix `0x01`.
- Structs: `MerkleTree`, `MerkleInclusionProof { leaf_index: usize, leaf_hash: [u8; 32], audit_path: Vec<[u8; 32]> }`.
- Method: `verify_inclusion(root_hash: &[u8; 32], leaf_data: &[u8]) -> bool`.
- Log Chain: `AuditLedger` using `HMAC-SHA256(secret_key, prev_hash || timestamp || actor || event || payload_digest)`.

### `liva-ingest` ↔ `liva-normalize`
- Types: `RawStatementRecord`, `StatementContainerType`, `BankIdentifier`.
- Output: `NormalizedStatement { bank: BankIdentifier, account_no: String, opening_cents: u64, closing_cents: u64, transactions: Vec<NormalizedTransaction> }`.
- `NormalizedTransaction { id: String, date: String (ISO 8601), booking_date: Option<String>, voucher_no: Option<String>, amount_cents: u64, is_credit: bool, balance_cents: u64, narration: String, counterparty_name: Option<String>, reference_codes: Vec<String> }`.

### `liva-match` ↔ `liva-ledger`
- Input: `Vec<NormalizedTransaction>`, `Vec<InternalLedgerEntry>`.
- Tier 1: Exact amount, matching reference voucher/invoice token, $|t_{bank} - t_{ledger}| \le 24\text{h}$.
- Tier 2: Amount bucket search, unaccented uppercase Jaro-Winkler score $\ge 0.85$, $|t_{bank} - t_{ledger}| \le 72\text{h}$, Fee Splitter tolerance 1,100 - 22,000 VND generating debit entry to Account 6425.

### `liva-nlp` ↔ `liva-match`
- Input: `narration: &str`.
- Output: `ExtractedMemo { invoice_no: Option<String>, order_no: Option<String>, partner_name: Option<String>, fee_flag: bool, confidence: f32, hitl_required: bool }`.
- Threshold: If `confidence < 0.6`, `hitl_required = true`.
