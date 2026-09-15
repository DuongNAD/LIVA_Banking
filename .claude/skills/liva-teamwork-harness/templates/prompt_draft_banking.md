# LIVA Banking Teamwork Task — [Task Title]

> Status: Step 1 — Eliciting Domain & Regulatory Scope
> Target Phase: [G0 Kickoff | G1 Design | G2 Foundation | G3 Engine MVP | G4 Ledger | G5 Shadow | G6 Go-live]
> Sprint Target: [S01–S17]
> SoD Rule: maker_user_id != checker_user_id [STRICTLY ENFORCED]

## 1. Domain & Statutory Framework
- **Target Page Clusters**: [e.g., P40–P46 Reconciliation / P50–P56 Journal & Close]
- **Statutory Mandates**:
  - [ ] Decree 13/2023/NĐ-CP (Personal Data Protection / Natural Person PII Redacted)
  - [ ] Circular 09/2020/TT-NHNN (Maker-Checker Dual Control & Information Security)
  - [ ] Decision 11/2023/QĐ-TTg (AML Statutory Reporting Thresholds)
- **Currency Invariant**: VND strictly stored as `i64` base units; aggregations use `i128` with `checked_add`. **Zero floating-point arithmetic allowed**.

## 2. Participating Subagent Swarms
- **Dev Swarm**: [lead_architect, rust_math_specialist, backend_sod_engineer, frontend_workbench_dev, integration_engineer, secops_egress_guard, qa_golden_challenger]
- **Persona Validation Swarm**: [sim_maker_r01, sim_bank_acc_r02, sim_gl_r03, sim_checker_r04, sim_treasury_r05, sim_aml_r08, sim_auditor_r09]

## 3. Strict Invariant & Guardrail Requirements
- [R1] **Arithmetic Invariance**: Double-entry balance $\text{Closing} \equiv \text{Opening} + \sum \text{Credits} - \sum \text{Debits}$ verified without float.
- [R2] **Zero Network Egress**: Air-gap / Intranet allowlist enforced; all outbound attempts logged; unauthorized connections aborted.
- [R3] **Cryptographic Audit Trail**: Append-only hash-chain for streaming events; Merkle Tree root generated upon period close.
- [R4] **Maker-Checker Segregation**: Mutating operations require distinct Maker proposal and Checker approval tokens.

## 4. Forcing Function & Verification Plan
- **Programmatic Prover**: Property-based invariant tests (`cargo test -j 2 -- --test-threads 2`).
- **Golden Dataset**: Test against 500+ standard banking transactions (exact match, fee tolerance, 1:N / N:1 subset splits, reversals).
- **Security Audit**: Program Dependence Graph (PDG) taint tracking confirming zero PII leak to non-whitelisted sinks.

## 5. Acceptance Criteria (DoD)
- [ ] 100% mathematical balance invariant pass on golden dataset.
- [ ] Auto-match precision $\ge 99.5\%$ with false auto-match $< 0.5\%$.
- [ ] 0 egress violations observed in sandboxed runtime logs.
- [ ] Tamper-evident hash-chain verification passes independently.
- [ ] Execution report and architectural decision records synchronized to Obsidian Vault.
