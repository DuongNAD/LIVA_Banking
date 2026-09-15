---
name: liva-teamwork-harness
description: Coordinate high-assurance multi-agent banking swarms through structured 9-step banking task elicitation, dual-swarm role allocation (Dev Swarm and R01-R13 Persona Swarm), mathematical invariant enforcement (i128 checked_add), zero data egress validation, and Cordis-inspired DAG lifecycle management. Use when launching multi-agent banking initiatives, orchestrating multi-sprint banking features, or conducting adversarial Maker-Checker simulations.
---

# LIVA Teamwork Harness

## Overview
LIVA Teamwork Harness is the high-assurance multi-agent coordination system tailored for regulated banking automation. It replaces generic agent team workflows with a rigorous, two-phase protocol:
1. **Phase 1 (Banking Task Elicitation)**: Synthesize an unambiguous, mathematically bounded, regulatory-compliant task specification artifact (`prompt_draft_banking.md`) through 9 structured steps.
2. **Phase 2 (Dual-Swarm Delegation)**: Decompose the approved task specification into a Directed Acyclic Graph (DAG) and dispatch to specialized subagent pools under strict Segregation of Duties (SoD) and fail-closed verification gates.

---

## Core Invariants & Guardrails
All agents and workflows coordinated by this harness must strictly enforce:
- **Zero Floating-Point Arithmetic**: Financial amounts are stored in base currency units (`i64` for VND). Aggregations use `i128` with `checked_add` / `checked_sub`. Any floating-point representation (`f32`, `f64`) on currency execution paths causes immediate pipeline termination.
- **Zero Data Egress**: Intranet allowlist only (ERP, SFTP, LDAP, HSM, NTP). No unauthenticated or internet-facing network sockets. Every outbound frame is logged; unlisted destination attempts trigger an instant halt.
- **Strict Segregation of Duties (SoD)**:
  - `maker_user_id != checker_user_id` enforced at backend API and database constraint levels.
  - Rule configurators cannot approve batches utilizing those rules within the same accounting period.
  - IT Administrators cannot self-grant permissions.
  - Security/Key Administrators possess zero operational financial permissions.
- **Immutable Tamper-Evident Audit Trail**: Continuous append-only hash-chain for runtime events + batch/period Merkle Tree proofs. Audit logs cannot be modified, deleted, or suppressed by any administrative role.
- **Local SLM Boundary**: Small Language Models (Q4_K_M GGUF via local `llama.cpp`) are strictly limited to text classification and JSON Schema extraction. SLMs never perform arithmetic, balance validation, or automated reconciliation without deterministic Rust validation.

---

## Phase 1: 9-Step Banking Task Elicitation Protocol

Maintain an interactive `prompt_draft_banking.md` artifact during elicitation. Walk through the 9 steps with the user:

### Step 1: Banking Domain & Regulatory Scope
- Identify target page cluster:
  - Data Intake & Pre-flight: P30–P36
  - Reconciliation & Matching: P40–P46
  - Journal Posting & Period Close: P50–P56
  - Treasury & Liquidity: P60–P66
  - AML & Surveillance: P70–P74
  - Audit & Compliance: P80–P83
  - System & Model Admin: P90–P106
- Bind relevant statutory frameworks: Decree 13/2023/NĐ-CP (PII Protection), Circular 09/2020/TT-NHNN (Information Security & Dual Control), Decision 11/2023/QĐ-TTg (AML Thresholds).

### Step 2: Mathematical Invariant & Precision Contract
- Reconfirm zero-drift rules: Currency integer types, arithmetic overflow boundaries, and double-entry invariants:
  $$\text{Closing Balance} \equiv \text{Opening Balance} + \sum \text{Credits} - \sum \text{Debits}$$
- Define rounding policies and fee deduction tolerance bands (e.g., standard wire fee 1,100–22,000 VND).

### Step 3: Security & Network Sandbox Boundaries
- Establish isolation mode: Complete Air-Gap vs. Intranet Allowlist.
- List exact internal IP endpoints required (e.g., MISA/FAST ERP port, SFTP server, local HSM).
- Enforce PII masking rules: Natural person identities redacted; enterprise account titles preserved.

### Step 4: Role Allocation & SoD Matrix
- Designate participating roles from the 13 canonical positions (R01–R13).
- Define Maker and Checker actors for all mutating steps.
- Configure escalation paths for unresolved items exceeding SLA.

### Step 5: Forcing Function & Verification Rigor
- Design independent verification mechanisms that prevent agent self-certification:
  - **Programmatic Prover**: Dedicated Rust test binary executing property-based fuzzing (`proptest`).
  - **Golden Dataset**: Minimum 500 pre-validated test cases covering 1:1 exact, fuzzy fee discrepancy, 1:N / N:1 subset-sum splits, partial matches, reversed transactions, and duplicate references.
  - **Audit Verification**: Autonomous hash-chain re-computation and Merkle inclusion proof verification.

### Step 6: Acceptance Criteria with Hard Gates (DoD)
- Define non-negotiable success metrics:
  - Reconciliation auto-match rate $\ge 90\%$.
  - False auto-match rate $< 0.5\%$.
  - Daily batch processing time $< 30$ minutes.
  - Zero arithmetic discrepancies.
  - Zero egress violations.
  - 100% audit trail verifiability.

### Step 7: Infrastructure & Offline Execution Bounds
- Specify local model parameters: GGUF model path, context window budget, token limits, and strict JSON Schema grammar constraints.
- Assert bounded hardware resources: RAM usage $\le 4\text{GB}$, compilation `-j 2`, test execution `-- --test-threads 2`.

### Step 8: Persistence & Knowledge Synchronization
- Configure state tracking in local SQLite WAL (`orchestrator_workflows`, `orchestrator_tasks`).
- Designate Obsidian Vault documentation paths in `teamwork_projects/obsidian_llm_wiki/vault/Knowledge/`.

### Step 9: Assemble, Validate & Launch
- Present the assembled `prompt_draft_banking.md` to the user for formal approval.
- Upon explicit confirmation, initiate Phase 2 delegation.

---

## Phase 2: Dual-Swarm Execution Topology

The harness coordinates two complementary agent swarms:

```
┌────────────────────────────────────────────────────────────────────────┐
│                          DUAL-SWARM TOPOLOGY                           │
├───────────────────────────────────┬────────────────────────────────────┤
│         1. DEV SWARM              │      2. PERSONA SIMULATION SWARM   │
│     (Engineering & Build)         │      (Adversarial UAT & Shadow)    │
├───────────────────────────────────┼────────────────────────────────────┤
│ • lead_architect                  │ • sim_maker_r01 (Accountant)       │
│ • rust_math_specialist            │ • sim_bank_acc_r02 (Bank Admin)    │
│ • backend_sod_engineer            │ • sim_gl_r03 (General Ledger)      │
│ • frontend_workbench_dev          │ • sim_checker_r04 (Chief Acc)      │
│ • integration_engineer            │ • sim_treasury_r05 (Treasury Ops)  │
│ • secops_egress_guard             │ • sim_aml_r08 (Compliance/AML)     │
│ • qa_golden_challenger            │ • sim_auditor_r09 (Auditor)        │
└───────────────────────────────────┴────────────────────────────────────┘
```

### 1. Dev Swarm Responsibilities
- **lead_architect**: Enforce architectural boundaries, ADR compliance, and canonical schemas.
- **rust_math_specialist**: Author math engine, 3-tier matching algorithms, and cryptographic hash-chains in `liva-native-core`.
- **backend_sod_engineer**: Implement state machines, API endpoints, and database-level Maker-Checker constraints.
- **frontend_workbench_dev**: Build high-performance UI (P42 3-column Workbench, drag-and-drop manual matching).
- **integration_engineer**: Build resilient bank statement parsers (CAMT.053, MT940, VCB Excel) and ERP connectors.
- **secops_egress_guard**: Audit source code for taint flows (PDG analysis) and enforce sandbox firewalls.
- **qa_golden_challenger**: Execute adversarial edge cases, stress tests, and boundary fuzzing against the golden dataset.

### 2. Persona Simulation Swarm Responsibilities
- Simulates realistic enterprise users to validate end-to-end user journeys before human pilot:
  - `sim_maker_r01` ingests messy statement files and proposes split matches.
  - `sim_checker_r04` attempts to bypass Maker-Checker gates (asserting system rejection) and verifies period-close checklists.
  - `sim_auditor_r09` executes read-only cryptographic audits and inclusion proofs.

---

## Cordis-Inspired Lifecycle & State Engine
The harness execution engine implements:
1. **DAG Topological Scheduling**: Dispatches independent tasks concurrently while respecting synchronization barriers (fan-in / fan-out).
2. **Fiber & LIFO Rollback**: Every subagent runs within a tracked fiber. Invariant failures trigger Last-In, First-Out cleanup of temporary artifacts and state rollbacks.
3. **Dead Letter Queue (DLQ)**: Tasks failing $\ge 3$ consecutive attempts are isolated into the DLQ with an automated diagnostic report, allowing unrelated branches to proceed safely.
4. **Human-in-the-Loop (HITL) Barriers**: High-consequence operations (period closure, key rotation, production migrations) halt execution and require explicit operator authorization.

---

## Stop Conditions & Failure Protocols
Halt execution and report immediately if:
- Any arithmetic operation encounters floating-point drift or unchecked overflow.
- A network egress attempt outside the approved intranet allowlist is detected.
- An SoD bypass occurs (`maker_user_id == checker_user_id` on any approval).
- Hash-chain verification fails or Merkle root mismatch is identified.
- Golden dataset auto-match precision drops below $99.5\%$ on exact matches.
