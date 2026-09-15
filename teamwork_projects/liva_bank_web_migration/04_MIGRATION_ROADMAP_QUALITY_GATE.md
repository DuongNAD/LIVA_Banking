# Migration Roadmap, API Parity Matrix & Quality Gate Specification
**Document ID:** `SPEC-MIGRATION-LIVA-WEB-2026-09-14`  
**Target System:** LIVA Banking Hybrid Web & Desktop Platform  
**Author:** Worker 2 (Banking Security, Migration Roadmap & Quality Gate Specialist)  
**Scope:** 4-Phase Transition Roadmap, 14-Command API Parity Matrix, OpenAPI 3.1.0 & WebSocket Schemas, 5-Tier Verification Strategy, and Disaster Recovery / Rollback Protocol  

---

## 1. Executive Summary & Migration Principles

The objective of the LIVA Banking Web Migration is to transform LIVA from a single-workstation desktop application (Tauri v2) into a dual-platform **Hybrid Web & Desktop** system. 

To ensure complete business continuity and zero regulatory exposure during this architectural transition, the migration is governed by three non-negotiable engineering principles:

1. **Dual-Platform Parity Principle:**  
   Desktop mode must retain **100% feature parity, native execution speed, and local OS capabilities** (Windows DPAPI key escrow, local hot-folder file monitoring, and desktop window dragging). The introduction of web capabilities must never degrade or alter native desktop performance.
2. **Zero Regression & Zero Data Egress:**  
   The underlying banking domain core (`liva-native-core`), deterministic 3-tier reconciliation engine, Circular 09 Maker-Checker state machine, and SQLite WAL database actor remain 100% shared. Web capabilities are enabled by an abstraction boundary (`IPlatformAdapter`) on the frontend and an Axum/Hyper Web Gateway on the backend. No personal data (PII) is ever persisted to browser disk storage or transmitted across unencrypted/external networks.
3. **Quantitative Quality Gates:**  
   Each migration phase is governed by objective, quantitative test milestones. Advancement to subsequent phases requires 100% pass rates across unit, integration, and adversarial stress suites under bounded hardware constraints (`-j 2`, RAM peak $\le 680\text{ MB}$).

---

## 2. 4-Phase Migration Roadmap

The migration is partitioned into four distinct, sequentially verifiable phases. Each phase defines concrete deliverables, effort estimations in Story Points (SP) and Person-Days (PD), quantitative exit criteria, and risk mitigations.

```
                             4-PHASE MIGRATION TIMELINE
┌─────────────────────────────────────────────────────────────────────────────┐
│ PHASE 1: Platform Abstraction Layer (Sprints 1–2 / 10 Person-Days / 13 SP)  │
│  - Define IPlatformAdapter TypeScript contracts                             │
│  - Implement TauriAdapter (preserve 100% desktop fidelity)                  │
│  - Scaffold WebAdapter (REST + SSE/WebSocket)                               │
│  - Decouple 12 coupled frontend files from window.__TAURI_INTERNALS__       │
│  - Milestone 1 Gate: 100% Vitest pass, zero type errors, desktop unbroken   │
└──────────────────────────────────────┬──────────────────────────────────────┘
                                       │
┌──────────────────────────────────────▼──────────────────────────────────────┐
│ PHASE 2: Rust Web Gateway & Session Auth (Sprints 3–5 / 15 Person-Days / 21 SP)
│  - Embed Axum/Hyper Web Gateway in liva-native-core alongside Tauri IPC     │
│  - Implement TLS 1.3 listener with PFS ciphers and HSTS                     │
│  - Build __Host-LIVA-Session HttpOnly session store                         │
│  - Implement X-CSRF-Token synchronizer middleware                           │
│  - Milestone 2 Gate: Zero unauthorized access, 100% session isolation       │
└──────────────────────────────────────┬──────────────────────────────────────┘
                                       │
┌──────────────────────────────────────▼──────────────────────────────────────┐
│ PHASE 3: Ingestion & Security Hardening (Sprints 6–8 / 15 Person-Days / 21 SP)
│  - Streaming in-memory multipart upload (< 64MB, zero disk spooling)        │
│  - Server-asserted Maker-Checker dual control (eliminate client IDs)        │
│  - Headless two-tier envelope encryption (Vault/KMS/Argon2id KEK + DEK)     │
│  - Implement Vue useMemoryScrubber composable and strict CSP Level 3        │
│  - Milestone 3 Gate: 50k-line parse in < 3s, zero temp files, C3 guard pass │
└──────────────────────────────────────┬──────────────────────────────────────┘
                                       │
┌──────────────────────────────────────▼──────────────────────────────────────┐
│ PHASE 4: E2E Integration & Quality Gate (Sprints 9–10 / 10 Person-Days / 13 SP)
│  - Playwright cross-browser test suite (Chrome, Edge, Firefox, Safari)      │
│  - Adversarial Challenge: Malformed statements, CSRF bypass, replay attacks │
│  - 50,000-line bank statement load benchmark (< 680MB RAM peak)             │
│  - Fail-closed rollback drills and forensic audit signoff                   │
│  - Milestone 4 Gate: Full Verification Matrix pass, 0 security vulnerabilities│
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.1. Phase Breakdown & Quantitative Milestones

| Phase | Core Objectives & Deliverables | Affected Codebases | Effort (SP / PD) | Priority | Quantitative Milestone & Exit Gate |
|---|---|---|---|---|---|
| **Phase 1: Platform Abstraction Layer** | 1. Create `liva-ui/src/platform/IPlatformAdapter.ts`.<br>2. Implement `TauriAdapter.ts` and `WebAdapter.ts`.<br>3. Refactor 12 coupled frontend files (`statementStore.ts`, `reconciliationStore.ts`, `bankingStore.ts`, `useGateway.ts`, `setup-main.ts`, etc.) to consume `IPlatformAdapter`.<br>4. Unit test suite for adapter polymorphic routing. | `liva-ui` | 13 SP / 10 PD | P0 (Blocker) | **Exit Gate 1:**<br>- `npx vue-tsc` returns 0 type errors.<br>- `npm run test -w liva-ui` 100% pass.<br>- Desktop Tauri app launches with zero regressions.<br>- Browser mode loads without uncaught IPC exceptions. |
| **Phase 2: Rust Web Gateway & Session Auth** | 1. Initialize Axum/Hyper HTTP/WS server in `liva-native-core/src/gateway/`.<br>2. TLS 1.3 listener with PFS ciphers (`TLS_AES_256_GCM_SHA384`, `TLS_CHACHA20_POLY1305_SHA256`) and HSTS.<br>3. Server-side session engine issuing `__Host-LIVA-Session; Secure; HttpOnly; SameSite=Strict`.<br>4. Anti-CSRF token middleware (`X-CSRF-Token`).<br>5. Concurrency backpressure semaphores (Ingest: 2, Queries: 32). | `liva-native-core`, `liva-desktop` | 21 SP / 15 PD | P0 (Blocker) | **Exit Gate 2:**<br>- Axum integration tests pass 100% (`cargo test gateway::tests`).<br>- Unauthenticated requests to `/api/v1/banking/*` return `401 Unauthorized`.<br>- Mutating POST requests without `X-CSRF-Token` return `403 Forbidden`.<br>- Concurrent session isolation verified across 50 simulated users. |
| **Phase 3: Ingestion & Security Hardening** | 1. Implement in-memory streaming multipart upload handler in Rust (`multer` / `bytes::BytesMut`, 64MB limit).<br>2. Integrate streaming event-driven SAX parser (`quick-xml::Reader`) for ISO 20022 XML and CSV/XLSX streaming to prevent memory amplification.<br>3. Add SHA-256 `file_hash` idempotency check to reject duplicate statement uploads.<br>4. Eliminate client-supplied `maker_id`/`checker_id`; implement server-asserted `resolve_hitl_web` with immutable `employee_id`/`citizen_id_hash` human operator assertion and `subtle::ConstantTimeEq` token comparison.<br>5. Replace DPAPI with Two-Tier Envelope Encryption (Vault/KMS/Argon2id KEK + DEK with >= 128-bit entropy and container `cap_add: [IPC_LOCK]`).<br>6. Implement `useMemoryScrubber` composable (with `Uint8Array` cryptographic wiping) and strict CSP Level 3 headers (`connect-src 'self' wss://banking.corp.vn; navigate-to 'self';`). | `liva-native-core`, `liva-ui` | 21 SP / 15 PD | P0 (Blocker) | **Exit Gate 3:**<br>- Zero bytes written to host disk during 50MB file upload.<br>- Streaming ISO 20022 XML parser consumes $< 40\text{ MB}$ RAM on 150MB XML.<br>- Circular 09 dual control tests pass (`test_maker_checker_fail_closed_on_self_approval`, `test_maker_checker_fail_closed_on_same_employee_id`, `test_hitl_token_constant_time_comparison`).<br>- Database error masking verified (`test_database_error_masked_with_correlation_id`).<br>- Memory scrubber tests pass (V8 heap zeroed on unmount).<br>- Compile-time release guard [Debt C3] verifies failure under default key. |
| **Phase 4: E2E Integration & Quality Gate** | 1. Playwright E2E browser test automation (Statement drag-drop -> Ingest -> Reconcile -> HITL Dual Control Approval).<br>2. 50,000-line bank statement stress benchmark under Web Gateway.<br>3. Adversarial challenge execution (Malformed files, Slowloris, replay attacks, concurrent race condition injection, human multi-account self-approval bypass).<br>4. Automated disaster recovery and rollback runbook verification.<br>5. Independent forensic audit signoff (`teamwork_preview_auditor`). | All components & scripts | 13 SP / 10 PD | P1 (Required) | **Exit Gate 4:**<br>- 50,000-line statement parsed and reconciled in $< 4.5\text{ s}$.<br>- Peak RAM consumption $\le 680\text{ MB}$ under 50k load.<br>- Playwright cross-browser tests pass 100% (Chrome, Edge, Firefox, Safari).<br>- Adversarial challenge test suite (6/6 scenarios) passes with 0 failures. |

---

## 3. Exhaustive API Parity Matrix

The following matrix maps every existing Tauri IPC command and MCP banking tool to its corresponding Web REST / WebSocket endpoint, specifying protocol, authorization claims, DTO schemas, and concurrency gating:

| # | Existing Tauri IPC Command | Target Web Protocol | Target Web Endpoint Route | Auth & RBAC Claims | Request Payload Schema (DTO) | Response Payload Schema (DTO) | Concurrency Gate & DB Lock | Desktop Backward Compatibility |
|---|---|---|---|---|---|---|---|---|
| **1** | `banking_get_overview` | `GET` | `/api/v1/banking/overview` | Authenticated Session (`Maker`, `Checker`, `Auditor`) | None | `BankingOverviewDto` (Account cards, balances, 30d forecast, discrepancy counts) | Readers Pool (`r2d2`, max 4), non-blocking | Delegated to `TauriAdapter.getOverview()` |
| **2** | `statement_ingest_file` | `POST` | `/api/v1/banking/statements/upload` | Authenticated Session (`Maker` only) | `multipart/form-data`: `file` (Binary stream, $\le 64\text{MB}$), `bank_hint` (Optional enum), `file_hash` (Optional SHA-256 idempotency key) | `StatementIngestResponseDto` (`statement_id`, `bank_code`, `file_hash`, `total_transactions`, `parse_duration_ms`) | `tokio::sync::Semaphore(2)`, In-memory streaming SAX parser (`quick-xml::Reader`), `spawn_blocking`, idempotency deduplication | Desktop continues using `statement_ingest_file` with local path |
| **3** | `banking_run_reconciliation` | `POST` | `/api/v1/banking/reconcile/run` | Authenticated Session (`Maker` only), `X-CSRF-Token` | Empty JSON `{}` | `ReconciliationSummaryDto` (`total`, `matched`, `pending_hitl`, `discrepancy_amount`, `duration_ms`) | Writer Pool checkout (`max 1`), serialized execution | Delegated to `TauriAdapter.runReconciliation()` |
| **4** | `banking_get_reconciliation_matrix` | `POST` | `/api/v1/banking/reconcile/matrix/query` | Authenticated Session (`Maker`, `Checker`, `Auditor`) | `ReconciliationQueryDto` (`filter`: `ALL\|MATCHED\|UNMATCHED\|PENDING_HITL`, `page`, `page_size`) | `ReconciliationMatrixDto` (`items`: array of transaction match pairs, `summary`: counts) | Readers Pool (`r2d2`), non-blocking | `TauriAdapter` converts query to native payload |
| **5** | `reconciliation_resolve_hitl` | `POST` | `/api/v1/banking/reconcile/hitl-resolve` | Authenticated Session (`Checker` only, asserting `employee_id` & `citizen_id_hash`), `X-CSRF-Token` | `ResolveHitlWebRequestDto` (`proposal_id`, `hitl_token`, `decision`, `decision_notes`). Client identity args prohibited. | `ResolveHitlWebResponseDto` (`success`, `proposal_id`, `checker_id`, `checker_employee_id`, `resolved_at`, `digital_signature`) | Writer Pool (`tx.execute`), atomic conditional SQL, constant-time `ct_eq`, masked DB errors | Server-asserted identity replaces client string args |
| **6** | `banking_get_compliance_status` | `GET` | `/api/v1/banking/compliance/status` | Authenticated Session (`Maker`, `Checker`, `Auditor`) | None | `ComplianceStatusDto` (`decree_13_compliant`, `zero_egress_verified`, `merkle_root_hash`, `audit_ledger_valid`) | Pure Read / Memory state inspection | Delegated to `TauriAdapter.getComplianceStatus()` |
| **7** | `toggle_ghost_mode` | `POST` | `/api/v1/system/window/ghost-mode` | Authenticated Admin | `GhostModeDto` (`enabled`: boolean) | `StandardSuccessDto` (`status`: `"OK"`) | Web: No-op / updates user preference cookie | Desktop: calls native window transparency |
| **8** | `set_eco_mode` | `POST` | `/api/v1/system/performance/eco-mode` | Authenticated Session | `EcoModeDto` (`eco_mode`: boolean) | `StandardSuccessDto` (`status`: `"OK"`) | Updates server background worker throttle state | Desktop: calls native thread scheduler |
| **9** | `update_interactive_zones` | `POST` | `/api/v1/system/window/interactive-zones` | Authenticated Session | `ZonesDto` (`zones`: array of rects) | `StandardSuccessDto` (`status`: `"OK"`) | Web: No-op (browser handles DOM events) | Desktop: calls native OS hit-testing |
| **10**| `open_dashboard` | `POST` | `/api/v1/system/navigation/dashboard` | Authenticated Session | None | `NavigationDto` (`target_url`: `"/dashboard"`) | Web: Client-side router redirect | Desktop: unhides/focuses native window |
| **11**| `open_setup` | `POST` | `/api/v1/system/navigation/setup` | Authenticated Session | None | `NavigationDto` (`target_url`: `"/setup"`) | Web: Client-side router redirect | Desktop: launches setup window |
| **12**| `vault_secret_present` | `POST` | `/api/v1/auth/secrets/check` | Authenticated Session (`Admin` only) | `SecretCheckDto` (`key`: string) | `SecretStatusDto` (`present`: boolean) | Server inspects database `system_key_vault` | Desktop: checks DPAPI vault snapshot |
| **13**| `store_vault_secret` | `POST` | `/api/v1/auth/secrets/store` | Authenticated Session (`Admin` only), `X-CSRF-Token` | `SecretStoreDto` (`key`: string, `value`: string) | `StandardSuccessDto` (`status`: `"STORED"`) | Writer Pool, encrypted via DEK/KEK envelope | Desktop: seals via Windows DPAPI |
| **14**| `delete_vault_secret` | `POST` | `/api/v1/auth/secrets/delete` | Authenticated Session (`Admin` only), `X-CSRF-Token` | `SecretDeleteDto` (`key`: string) | `StandardSuccessDto` (`status`: `"DELETED"`) | Writer Pool, deletes encrypted record | Desktop: deletes DPAPI vault key |
| **MCP 1** | `banking_reconcile` | `POST` | `/api/v1/banking/mcp/reconcile` | Authenticated Session or MCP Agent API Key | `McpReconcileRequestDto` (`statement_id`, `tolerance_vnd`, `date_window_days`) | `McpReconcileResponseDto` (`matched_count`, `unmatched_count`, `balance_invariant_holds`) | Writer Pool / ReconciliationEngine | Invoked directly by AI Agents or UI |
| **MCP 2** | `treasury_payment_order` | `POST` | `/api/v1/banking/mcp/payment-order` | Authenticated Session (`Maker` for propose, `Checker` for approve). Client-supplied `maker_id`/`checker_id` stripped/rejected. | `PaymentOrderDto` (`action`: `"PROPOSE"\|"APPROVE"`, `order_details`, `hitl_token`). Zero client-supplied identity claims. | `PaymentOrderResultDto` (`order_id`, `status`, `maker_employee_id`, `checker_employee_id`, `dual_signature`) | Writer Pool / Circular 09 Engine | Full Maker-Checker dual control with server-asserted human operator identity |
| **MCP 3** | `compliance_aml_screen` | `POST` | `/api/v1/banking/mcp/aml-screen` | Authenticated Session or Compliance Auditor | `AmlScreenRequestDto` (`transactions`: array, `redact_pii`: boolean) | `AmlScreenResponseDto` (`alerts`: array, `sanitized_transactions`: array, `zero_egress_verified`: boolean) | Read Pool / Sanitizer & AML Rules Engine | Real-time Decision 11 AML screening |
| **MCP 4** | `credit_risk_scoring` | `POST` | `/api/v1/banking/mcp/risk-scoring` | Authenticated Session or Risk Officer | `RiskScoreRequestDto` (`statements`: array, `forecast_horizon_days`: 30) | `RiskScoreResponseDto` (`dscr_basis_points`, `quick_ratio_basis_points`, `deficit_risk`: boolean) | Pure CPU / Read Pool | Integer-scaled financial health analytics |

---

## 4. OpenAPI 3.1 & WebSocket Schema Specifications

### 4.1. OpenAPI 3.1.0 Specification (Authoritative Banking Slice)

```yaml
openapi: 3.1.0
info:
  title: LIVA Banking Web Gateway API
  description: >
    Production-grade banking and treasury reconciliation API conforming to
    Vietnam Decree 13/2023/NĐ-CP (PDPD), Circular 09/2020/TT-NHNN (Dual Control),
    and Decision 11/2023/QĐ-TTg (AML/CTF).
  version: 1.0.0
servers:
  - url: https://banking.corp.vn/api/v1
    description: Production On-Premise Banking Gateway (TLS 1.3 Only)
  - url: https://127.0.0.1:8002/api/v1
    description: Localhost Air-Gapped Loopback Gateway

paths:
  /banking/overview:
    get:
      summary: Fetch aggregated banking balances and reconciliation KPIs
      description: Returns multi-bank card summaries (VCB, TCB, BIDV), discrepancy counters, and 30-day forecast.
      security:
        - SessionCookie: []
      responses:
        '200':
          description: Banking overview successfully retrieved
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/BankingOverviewDto'
        '401':
          $ref: '#/components/responses/UnauthorizedError'

  /banking/statements/upload:
    post:
      summary: Streaming in-memory multipart upload of bank statements
      description: >
        Uploads XLSX, CSV, PDF, or ISO 20022 XML statements directly into bounded RAM (< 64MB).
        Zero disk spooling ensures 100% compliance with Decree 13 Zero Data Egress.
      security:
        - SessionCookie: []
        - CsrfToken: []
      requestBody:
        required: true
        content:
          multipart/form-data:
            schema:
              type: object
              required:
                - file
              properties:
                file:
                  type: string
                  format: binary
                  description: Raw binary statement file (max 64MB)
                bank_hint:
                  type: string
                  enum: [VCB, TCB, BIDV, ISO20022, AUTO]
                  default: AUTO
      responses:
        '200':
          description: Statement successfully parsed and ingested into SQLite WAL
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/StatementIngestResponseDto'
        '400':
          description: Malformed statement or unsupported format
        '413':
          description: Payload Too Large (> 64MB)
        '429':
          description: Concurrency limit reached (max 2 concurrent uploads)

  /banking/reconcile/run:
    post:
      summary: Trigger 3-Tier deterministic reconciliation engine
      description: Executes exact 1:1, fuzzy heuristic, and 1:N/N:1 composite split solver against ERP records.
      security:
        - SessionCookie: []
        - CsrfToken: []
      responses:
        '200':
          description: Reconciliation execution summary
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ReconciliationSummaryDto'

  /banking/reconcile/matrix/query:
    post:
      summary: Query reconciliation transactions and match statuses
      description: POST-only query protecting customer PII from appearing in access logs or browser history.
      security:
        - SessionCookie: []
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/ReconciliationQueryDto'
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ReconciliationMatrixDto'

  /banking/reconcile/hitl-resolve:
    post:
      summary: Resolve HITL exception under Circular 09 Maker-Checker dual control
      description: >
        Authorizes or rejects a pending match override. Enforces server-asserted human operator identity
        (employee_id / citizen_id_hash) from HttpOnly cookie, single-use 15-min HMAC token, constant-time
        token validation (subtle::ConstantTimeEq), and strict multi-account self-approval prohibition.
        Client-supplied maker_id or checker_id parameters are strictly forbidden.
      security:
        - SessionCookie: []
        - CsrfToken: []
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/ResolveHitlWebRequestDto'
      responses:
        '200':
          description: Exception successfully resolved with non-repudiation signature
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ResolveHitlWebResponseDto'
        '400':
          description: Client attempted to supply unauthorized identity parameters (maker_id/checker_id)
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/StandardErrorDto'
        '403':
          description: Self-approval prohibited (same human operator/employee_id) or invalid token signature
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/StandardErrorDto'
        '500':
          description: Internal error with sanitized correlation ID (SQL errors masked)
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/MaskedDatabaseErrorResponse'

  /banking/mcp/payment-order:
    post:
      summary: Circular 09 Maker-Checker payment order initiation and authorization via MCP
      description: >
        Initiates (action=PROPOSE) or authorizes (action=APPROVE) high-value payment orders.
        Identity claims (maker/checker) are strictly asserted by the server gateway from the active session/mTLS context.
        Client-supplied maker_id or checker_id fields are rejected.
      security:
        - SessionCookie: []
        - CsrfToken: []
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/PaymentOrderDto'
      responses:
        '200':
          description: Payment order state transition successful
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/PaymentOrderResultDto'
        '400':
          description: Invalid request payload or client identity injection detected
        '403':
          description: Self-approval prohibited across human operator accounts

  /banking/compliance/status:
    get:
      summary: Verify Decree 13 and Merkle audit chain integrity
      security:
        - SessionCookie: []
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ComplianceStatusDto'

components:
  securitySchemes:
    SessionCookie:
      type: apiKey
      in: cookie
      name: __Host-LIVA-Session
    CsrfToken:
      type: apiKey
      in: header
      name: X-CSRF-Token

  schemas:
    BankingOverviewDto:
      type: object
      required:
        - total_balance
        - accounts
        - discrepancy_count
        - matched_ratio
      properties:
        total_balance:
          type: integer
          description: Total balance in VND (integer scaled)
        accounts:
          type: array
          items:
            $ref: '#/components/schemas/BankAccountSummary'
        discrepancy_count:
          type: integer
        matched_ratio:
          type: number
          format: float
        rolling_forecast_30d:
          type: array
          items:
            $ref: '#/components/schemas/CashflowForecastPoint'

    BankAccountSummary:
      type: object
      required:
        - bank_code
        - account_number
        - balance
      properties:
        bank_code:
          type: string
          enum: [VCB, TCB, BIDV, MBB, CTG, VBA]
        account_number:
          type: string
        balance:
          type: integer
        unreconciled_count:
          type: integer

    CashflowForecastPoint:
      type: object
      required:
        - date
        - projected_balance
        - is_deficit_risk
      properties:
        date:
          type: string
          format: date
        inflow:
          type: integer
        outflow:
          type: integer
        projected_balance:
          type: integer
        is_deficit_risk:
          type: boolean

    StatementIngestResponseDto:
      type: object
      required:
        - statement_id
        - bank_code
        - file_hash
        - total_transactions
        - parse_duration_ms
        - opening_balance
        - closing_balance
      properties:
        statement_id:
          type: string
          format: uuid
        bank_code:
          type: string
        filename:
          type: string
        file_hash:
          type: string
          description: SHA-256 idempotency digest of the statement binary stream
        total_transactions:
          type: integer
        parse_duration_ms:
          type: integer
        opening_balance:
          type: integer
        closing_balance:
          type: integer
        pii_masked_count:
          type: integer

    ReconciliationSummaryDto:
      type: object
      required:
        - total
        - matched
        - pending_hitl
        - discrepancy_amount
        - duration_ms
      properties:
        total:
          type: integer
        matched:
          type: integer
        pending_hitl:
          type: integer
        discrepancy_amount:
          type: integer
        duration_ms:
          type: integer

    ReconciliationQueryDto:
      type: object
      properties:
        filter:
          type: string
          enum: [ALL, MATCHED, UNMATCHED, PENDING_HITL]
          default: ALL
        account_number:
          type: string
        page:
          type: integer
          default: 1
        page_size:
          type: integer
          default: 50

    ReconciliationMatrixDto:
      type: object
      required:
        - items
        - total_records
      properties:
        items:
          type: array
          items:
            $ref: '#/components/schemas/ReconciliationItemDto'
        total_records:
          type: integer

    ReconciliationItemDto:
      type: object
      required:
        - tx_id
        - amount
        - match_status
      properties:
        tx_id:
          type: string
        booking_date:
          type: string
        amount:
          type: integer
        counterparty_name:
          type: string
        counterparty_account:
          type: string
        narration:
          type: string
        match_status:
          type: string
          enum: [MATCHED, UNMATCHED, PENDING_HITL, DISCREPANCY]
        hitl_token:
          type: string
          nullable: true

    ResolveHitlWebRequestDto:
      type: object
      required:
        - proposal_id
        - hitl_token
        - decision
      properties:
        proposal_id:
          type: string
          format: uuid
        hitl_token:
          type: string
          description: Single-use 15-min HMAC token issued during proposal creation
        decision:
          type: string
          enum: [APPROVE, REJECT]
        decision_notes:
          type: string

    ResolveHitlWebResponseDto:
      type: object
      required:
        - success
        - proposal_id
        - checker_id
        - checker_employee_id
        - resolved_at
        - digital_signature
      properties:
        success:
          type: boolean
        proposal_id:
          type: string
          format: uuid
        checker_id:
          type: string
        checker_employee_id:
          type: string
          description: Immutable HR employee ID of the checker
        resolved_at:
          type: integer
        digital_signature:
          type: string

    PaymentOrderDto:
      type: object
      description: MCP payment order payload. Server asserts maker/checker identity from authenticated session.
      required:
        - action
        - order_details
      properties:
        action:
          type: string
          enum: [PROPOSE, APPROVE]
        order_details:
          type: object
          required:
            - source_account
            - beneficiary_account
            - beneficiary_bank
            - amount_vnd
            - narration
          properties:
            source_account:
              type: string
            beneficiary_account:
              type: string
            beneficiary_bank:
              type: string
            amount_vnd:
              type: integer
              description: Integer-scaled amount in VND
            narration:
              type: string
        hitl_token:
          type: string
          description: Single-use 15-min HMAC token required when action=APPROVE

    PaymentOrderResultDto:
      type: object
      required:
        - order_id
        - status
        - maker_employee_id
        - checker_employee_id
        - dual_signature
      properties:
        order_id:
          type: string
          format: uuid
        status:
          type: string
          enum: [PENDING_CHECKER_APPROVAL, APPROVED, REJECTED]
        maker_employee_id:
          type: string
        checker_employee_id:
          type: string
          nullable: true
        dual_signature:
          type: string
          nullable: true

    StandardErrorDto:
      type: object
      required:
        - error_code
        - message
      properties:
        error_code:
          type: string
          example: SELF_APPROVAL_PROHIBITED
        message:
          type: string
          example: "Vi phạm kiểm soát kép: Người phê duyệt trùng người tạo đề xuất"

    MaskedDatabaseErrorResponse:
      type: object
      required:
        - error_code
        - message
        - correlation_id
      properties:
        error_code:
          type: string
          example: INTERNAL_DATABASE_ERROR
        message:
          type: string
          example: "Lỗi cơ sở dữ liệu nội bộ. Vui lòng liên hệ bộ phận hỗ trợ kỹ thuật."
        correlation_id:
          type: string
          format: uuid
          example: "8f7e2a1b-3c4d-5e6f-7a8b-9c0d1e2f3a4b"

    ComplianceStatusDto:
      type: object
      required:
        - decree_13_compliant
        - zero_egress_verified
        - merkle_root_hash
        - total_audit_records
      properties:
        decree_13_compliant:
          type: boolean
        zero_egress_verified:
          type: boolean
        merkle_root_hash:
          type: string
        total_audit_records:
          type: integer
        pii_scrubbed_count:
          type: integer

  responses:
    UnauthorizedError:
      description: Authentication cookie missing, invalid, or expired
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/StandardErrorDto'
```

---

### 4.2. WebSocket Event Stream Specification (`/api/v1/banking/events`)

The WebSocket endpoint provides real-time event distribution for statement parsing progress, reconciliation lifecycle updates, and compliance alerts.

- **Protocol:** `WSS` (WebSocket over TLS 1.3)
- **Path:** `/api/v1/banking/events`
- **Authentication Handshake:**
  - Standard browser WebSocket does not support custom headers.
  - The client transmits the session cookie automatically on handshake.
  - Optional ephemeral ticket parameter for cross-origin setups: `wss://banking.corp.vn/api/v1/banking/events?ticket=ws_ticket_...` (Single-use 30-second TTL).
- **Frame Limit:** Strict 1MB maximum frame size (`MAX_WS_MESSAGE_BYTES = 1048576`).
- **Heartbeat:** Ping/Pong interval every 30 seconds.

#### Message Payload Schemas:

1. **Statement Parsing Progress Event (`STATEMENT_PROGRESS`):**
```json
{
  "event_type": "STATEMENT_PROGRESS",
  "statement_id": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
  "filename": "VCB_Account_0011002233_Aug2026.xlsx",
  "parser_engine": "STREAMING_SAX_QUICK_XML",
  "file_hash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "memory_rss_mb": 34.2,
  "stage": "EXTRACTING",
  "progress_percent": 65,
  "processed_rows": 32500,
  "total_rows_estimated": 50000,
  "timestamp": 1726301245
}
```

2. **Reconciliation Completed Event (`RECONCILIATION_COMPLETED`):**
```json
{
  "event_type": "RECONCILIATION_COMPLETED",
  "run_id": "rec_run_8819",
  "total_transactions": 50000,
  "matched_count": 49900,
  "pending_hitl_count": 100,
  "match_ratio": 0.998,
  "duration_ms": 2340,
  "timestamp": 1726301280
}
```

3. **HITL Dual Control Proposal Created (`HITL_PROPOSAL_CREATED`):**
```json
{
  "event_type": "HITL_PROPOSAL_CREATED",
  "proposal_id": "prop_3301",
  "action_type": "MANUAL_SPLIT",
  "maker_user_id": "usr_maker_01",
  "maker_employee_id": "EMP-08819",
  "maker_citizen_id_hash": "e84d2f8...c19",
  "bank_tx_id": "vcb_tx_9921",
  "amount_vnd": 450000000,
  "expires_at": 1726302180,
  "timestamp": 1726301280
}
```

4. **Security / Audit Tamper Alert (`AUDIT_TAMPER_ALERT`):**
```json
{
  "event_type": "AUDIT_TAMPER_ALERT",
  "severity": "CRITICAL",
  "tampered_seq_id": 1422,
  "expected_hash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "actual_hash": "a1b2c3d4...",
  "message": "Phát hiện sai lệch chuỗi băm sổ cái kiểm toán tại bản ghi #1422",
  "timestamp": 1726301300
}
```

---

## 5. Automated Testing & Verification Strategy

The testing architecture employs a 5-tier verification hierarchy ensuring zero regressions, mathematically verified domain logic, and strict regulatory adherence:

```
                         5-TIER VERIFICATION HIERARCHY
┌─────────────────────────────────────────────────────────────────────────────┐
│ TIER 5: Security Penetration & Compliance Audit                             │
│  - OWASP Top 10 API Security Checklist                                      │
│  - Decree 13 Forensic Host Disk Check (zero PII temp files)                 │
│  - V8 Memory Heap Dump Inspection                                           │
└──────────────────────────────────────┬──────────────────────────────────────┘
                                       │
┌──────────────────────────────────────▼──────────────────────────────────────┐
│ TIER 4: Adversarial Stress & Chaos Test Suite                                │
│  - 50,000-line bank statement benchmark (< 4.5s, < 680MB RAM)               │
│  - Corrupted Excel/PDF/XML injections                                       │
│  - Replay attacks, CSRF forgery, and self-approval race condition injection  │
└──────────────────────────────────────┬──────────────────────────────────────┘
                                       │
┌──────────────────────────────────────▼──────────────────────────────────────┐
│ TIER 3: End-to-End Browser Flows (Playwright Automation)                    │
│  - Cross-browser execution (Chrome, Edge, Firefox, WebKit/Safari)           │
│  - Complete workflow: Upload -> Reconcile -> HITL Dual Control Resolution   │
│  - Ephemeral memory scrubbing verification on tab lock                       │
└──────────────────────────────────────┬──────────────────────────────────────┘
                                       │
┌──────────────────────────────────────▼──────────────────────────────────────┐
│ TIER 2: Web Gateway & SQLite Integration Tests                              │
│  - Axum mock test client with in-memory SQLite WAL                          │
│  - Multi-threaded connection pool checkout & checkout timeout resilience    │
│  - HttpOnly cookie & CSRF header validation                                 │
└──────────────────────────────────────┬──────────────────────────────────────┘
                                       │
┌──────────────────────────────────────▼──────────────────────────────────────┐
│ TIER 1: Core Unit Tests (Rust & TypeScript Vitest)                          │
│  - Parser sniffing & byte extraction (VCB, TCB, BIDV, ISO 20022)             │
│  - 3-tier deterministic reconciliation math (zero rounding errors)          │
│  - RFC 6962 Merkle tree inclusion proofs & HMAC forward audit chain         │
│  - TypeScript IPlatformAdapter polymorphic routing                          │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 5.1. Concrete Test Scripts & Commands

#### Tier 1: Unit Tests (Rust & Vitest)
```powershell
# 1. Verify Rust Native Core unit tests sequentially under bounded memory
cargo test -p liva-native-core --lib -j 2 -- --test-threads 2

# 2. Verify Release Key Guard [Debt C3]
cargo test -p liva-native-core --lib crypto::tests::test_check_release_key_guard -j 2 -- --test-threads 2

# 3. Verify Frontend Vitest Unit Tests & Coverage Ratchet
npm run test:coverage -w liva-ui
```

#### Tier 2: Web Gateway Integration Tests
```powershell
# 1. Execute Web Gateway REST route and session integration tests
cargo test -p liva-native-core --test web_gateway_integration_test -j 2 -- --test-threads 2

# 2. Execute SQLite WAL Actor multi-threaded concurrency test
cargo test -p liva-native-core --test db_actor_concurrency_test -j 2 -- --test-threads 2
```

#### Tier 3: Playwright End-to-End Browser Automation (`e2e-banking-flow.spec.ts`)
```typescript
import { test, expect } from '@playwright/test';

test.describe('LIVA Banking Web E2E Flow', () => {
  test('Complete Statement Ingestion, Reconciliation and Dual Control Approval', async ({ page, context }) => {
    // 1. Authenticate as Maker
    await page.goto('https://banking.corp.vn/login');
    await page.fill('#username', 'maker_accountant_01');
    await page.fill('#password', 'SecurePass123!');
    await page.click('#btnLogin');
    await expect(page).toHaveURL(/.*dashboard/);

    // 2. Drag and drop 50,000-line bank statement file
    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.click('#btnUploadStatement');
    const fileChooser = await fileChooserPromise;
    await fileChooser.setFiles('tests/fixtures/vcb_50k_statement.xlsx');

    // 3. Verify upload progress bar and zero disk spooling
    await expect(page.locator('.upload-progress-bar')).toBeVisible();
    await expect(page.locator('.ingest-status-badge')).toHaveText('HOÀN TẤT', { timeout: 10000 });

    // 4. Trigger Reconciliation
    await page.click('#btnRunReconcile');
    await expect(page.locator('.reconciliation-summary-card')).toBeVisible({ timeout: 5000 });
    await expect(page.locator('#statMatchedRatio')).toHaveText('99.8%');

    // 5. Maker proposes HITL resolution
    await page.click('.btn-hitl-propose:first-child');
    await page.fill('#hitlNotes', 'Đã đối chiếu hóa đơn VAT số 00921');
    await page.click('#btnSubmitProposal');
    await expect(page.locator('.proposal-status')).toHaveText('CHỜ DUYỆT (15m)');

    // 6. Logout Maker -> Clear-Site-Data triggered
    await page.click('#btnLogout');
    await expect(page).toHaveURL(/.*login/);

    // 7. Authenticate as Checker (Chief Accountant)
    await page.fill('#username', 'checker_chief_02');
    await page.fill('#password', 'ChiefSecurePass456!');
    await page.click('#btnLogin');
    await page.goto('https://banking.corp.vn/reconcile/approvals');

    // 8. Checker approves proposal
    await page.click('.btn-hitl-approve:first-child');
    await page.fill('#approvalNotes', 'Phê duyệt trích phí tự động');
    await page.click('#btnConfirmApproval');

    // 9. Verify state transition to APPROVED and digital signature generation
    await expect(page.locator('.approval-status-badge')).toHaveText('ĐÃ PHÊ DUYỆT');
    await expect(page.locator('.digital-sig-badge')).toContainText('HMAC-SHA256:');
  });
});
```

#### Tier 4: Adversarial Stress & Chaos Test Suite

```powershell
# 1. Execute 50,000-line statement benchmark under Web Gateway
cargo test -p liva-native-core --test banking_50k_benchmark -j 2 -- --nocapture

# 2. Execute Adversarial Challenge Suite (Malformed files, replay attacks, self-approval bypass)
cargo test -p liva-native-core --test banking_adversarial_challenge -j 2 -- --test-threads 2
```

##### Concrete Adversarial Test Scenarios & Invariant Verification:

1. **Scenario 1: Timing Side-Channel Invariance on HITL Token (`test_hitl_token_constant_time_comparison`):**
   - **Attack Simulation:** High-frequency script submits 10,000 candidate HITL tokens with incremental byte-prefix matches (0 to 31 matching bytes of the 32-byte HMAC).
   - **Assertion:** Latency distribution exhibits zero correlation with prefix match length ($\Delta t < 25\text{ ns}$ statistical noise floor). Verification is guaranteed constant-time via `subtle::ConstantTimeEq`.

2. **Scenario 2: Multi-Account Human Operator Self-Approval Attempt (`test_maker_checker_fail_closed_on_same_employee_id`):**
   - **Attack Simulation:** An operator holding dual credentials (`usr_alice_maker` and `usr_alice_checker`), both provisioned to `employee_id = "EMP-08819"` and `citizen_id_hash = "sha256_cccd_07919..."`, creates a proposal with account A and resolves it with account B across separate browser sessions.
   - **Assertion:** Gateway detects identical `employee_id` and aborts with `HTTP 403 Forbidden` (`SelfApprovalProhibited`). Transition row count remains 0; security alert logged to `banking_audit_chain`.

3. **Scenario 3: Streaming ISO 20022 XML Parsing Memory Ceiling (`test_streaming_xml_parser_bounded_memory`):**
   - **Attack Simulation:** Ingests a 150MB synthetic ISO 20022 `camt.053` XML statement containing 120,000 transactions.
   - **Assertion:** Streaming SAX reader (`quick-xml::Reader`) processes the stream chunk-by-chunk with peak RAM delta $\le 38.5\text{ MB}$, completely avoiding the $>630\text{ MB}$ DOM tree memory amplification risk.

4. **Scenario 4: Statement Upload Idempotency Deduplication (`test_statement_upload_idempotency_hash`):**
   - **Attack Simulation:** Client uploads statement, experiences a simulated TCP half-close, and immediately retries sending the same 25MB file.
   - **Assertion:** Gateway detects duplicate `file_hash` in SQLite, returns existing `statement_id` with `HTTP 409 Conflict`, and refuses duplicate transaction row insertion.

5. **Scenario 5: Database Error Masking & Information Concealment (`test_database_error_masked_with_correlation_id`):**
   - **Attack Simulation:** Malformed request triggers an SQLite schema constraint violation (`UNIQUE constraint failed: hitl_proposals.id`).
   - **Assertion:** Response body returns `{ "error_code": "INTERNAL_DATABASE_ERROR", "correlation_id": "<uuid>" }`. Internal table names, columns, and SQLite version numbers are completely absent from the client response.

---

## 6. Deployment Risk Matrix, Rollback & Disaster Recovery

### 6.1. Deployment Risk Matrix

| Risk Factor | Probability | Impact | Severity | Technical Trigger Condition | Preventive Mitigation & Fallback Action |
|---|---|---|---|---|---|
| **R1: Server OOM during Concurrent Statement Uploads** | Low | High | **HIGH** | Multiple accountants uploading $>50\text{MB}$ statements simultaneously, exceeding host RAM limit. | **Prevention:** In-memory streaming SAX parser (`quick-xml::Reader`) bounds XML memory to $<40\text{MB}$. Gateway `tokio::sync::Semaphore(2)` restricts concurrent heavy parsing to 2. Excess requests receive `HTTP 429 Too Many Requests`. |
| **R2: Circular 09 Self-Approval Bypass (Multi-Account or Race)** | Very Low | Critical | **CRITICAL** | Attacker uses dual login accounts or executes concurrent multithreaded approval requests to self-approve or double-spend tokens. | **Prevention:** Dual control identity tied to immutable natural person claims (`employee_id` / `citizen_id_hash`). Constant-time token verification (`subtle::ConstantTimeEq`). Atomic SQLite conditional update (`UPDATE ... WHERE status = 'PENDING' AND token_consumed = 0`). Affected rows return 0 for duplicate calls. |
| **R3: Browser DOM Freeze on 50k Row Rendering** | Medium | Medium | **MEDIUM** | Rendering 50,000 transactions directly in Vue template without DOM recycling. | **Prevention:** Mandatory virtual scrolling (`vue-virtual-scroller` / `recycle-scroller`) in `TransactionLedger.vue`. Only 30 rows rendered in DOM at any instant. |
| **R4: Release Guard Failure with Default Key [Debt C3]** | Low | Critical | **CRITICAL** | Production deployment launched with default encryption key string or weak passphrase. | **Prevention:** `crypto.rs` release guard triggers startup panic on default key. Argon2id KEK derivation validates minimum 128-bit entropy. Container manifests specify `cap_add: [IPC_LOCK]`. |
| **R5: Incomplete Session Invalidation on Tab Closure** | Medium | High | **HIGH** | User closes browser tab without clicking logout; session cookie remains in browser memory. | **Prevention:** `useMemoryScrubber` clears sensitive data and wipes `Uint8Array` secret buffers on `beforeunload`. Cookies have `SameSite=Strict; Max-Age=3600`. Gateway enforces 15-minute server-side inactivity timeout. |

---

### 6.2. Fail-Closed Rollback Strategy (1-Click Fallback to Desktop)

If a critical zero-day vulnerability, network outage, or web runtime defect occurs in production, the system provides a **deterministic 1-Click Rollback** to the proven native Desktop mode:

```
                       FAIL-CLOSED ROLLBACK SEQUENCE
[Production Web Incident Detected]
        │
        ▼
[Step 1: Gateway Lockdown]
  - Issue HTTP 503 Service Unavailable on Web Gateway
  - Revoke all active web session cookies (__Host-LIVA-Session)
  - Execute PRAGMA wal_checkpoint(TRUNCATE) on SQLite database
        │
        ▼
[Step 2: Client Fallback Redirection]
  - Web users redirected to: "Hệ thống đang chuyển sang chế độ Desktop an toàn"
  - Instruct users to launch local LIVA Banking Desktop Shell (Tauri v2)
        │
        ▼
[Step 3: Local Desktop Activation]
  - Desktop Tauri app connects directly to shared SQLite WAL file via native IPC
  - 100% native performance restored; 0 dependencies on web gateway
        │
        ▼
[Step 4: Post-Mortem & Patching]
  - Inspect forward HMAC audit ledger for unauthorized attempts
  - Verify Merkle tree root hash integrity
```

### 6.3. Disaster Recovery Protocol

1. **Database WAL Checkpoint & Snapshot:**
   - Automated hourly backup executes `VACUUM INTO '/var/backup/liva_snapshot_YYYYMMDD_HHMM.sqlite'`.
   - Backup file is immediately encrypted under KEK via AES-256-GCM and stored in air-gapped cold storage.
2. **Audit Trail Verification on Recovery:**
   - Upon system restoration, the engine automatically verifies the integrity of the forward HMAC audit ledger:
     ```rust
     let report = AuditLedger::verify_chain(&conn, &audit_key)?;
     if !report.is_intact {
         panic!("CRITICAL DISASTER RECOVERY ALERT: Audit chain corruption at seq_id {:?}", report.tampered_seq_id);
     }
     ```
3. **Key Rotation Protocol:**
   - KEK rotation in HashiCorp Vault / KMS does not require re-encrypting the entire SQLite database.
   - The engine un-wraps the DEK using the old KEK, wraps the DEK under the new KEK, and updates table `system_key_vault` in a single atomic transaction.
