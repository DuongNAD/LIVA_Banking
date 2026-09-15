# LIVA Banking Web Migration: Master Architecture & Quality Dossier
**Project:** LIVA Banking Hybrid Web & Desktop Platform  
**Directory:** `teamwork_projects/liva_bank_web_migration/`  
**Target Completion:** 2026-09-14  
**Regulatory Compliance:** Decree 13/2023/NĐ-CP (PDPD), Circular 09/2020/TT-NHNN (Dual Control), Law on AML No. 14/2022/QH15, Decision 11/2023/QĐ-TTg, RFC 6962 (Merkle Trees), RFC 2104 (HMAC).

---

## 1. Executive Summary & Architectural Vision

LIVA Banking is transforming from a single-workstation desktop application into a dual-platform **Hybrid Web & Desktop** system. 

The core challenge of this migration lies in bridging two fundamentally divergent execution environments without sacrificing security or performance:
- **Desktop Mode (Tauri v2):** Retains 100% native execution speed, local OS file path access, Windows DPAPI / TPM 2.0 key escrow, native frameless window controls, and automated local hot-folder file monitoring.
- **Web Mode (Browser):** Operates seamlessly in standard modern browsers (Google Chrome, Microsoft Edge, Mozilla Firefox, Apple Safari) connecting to a local or on-premise Rust Web Gateway over HTTPS (TLS 1.3 only) and WebSocket/SSE.
- **Unified Domain Core (`liva-native-core`):** The underlying multi-bank parser engine (`VCB`, `TCB`, `BIDV`, `ISO 20022`, `Agribank`, `MBBank`, `VietinBank`), 3-tier deterministic reconciliation engine, Circular 09 Maker-Checker state machine, and SQLite WAL database actor remain **100% shared and untouched**.

This dossier provides the authoritative, production-grade blueprints, threat models, API specifications, and phased roadmap to deliver the web migration safely, rigorously, and without regressions.

---

## 2. Deliverable Catalog & Document Index

The technical specifications for the Web Migration project are structured into four comprehensive documents:

```
teamwork_projects/liva_bank_web_migration/
├── README.md                           # Master navigation index, executive summary, and ADR catalog
├── 01_ARCHITECTURAL_GAP_AUDIT.md       # R1: Deep runtime gap analysis, 12 coupled frontend files, file.path breakage
├── 02_HYBRID_ARCHITECTURE_BLUEPRINT.md # R2: Hybrid system blueprint, IPlatformAdapter TypeScript contracts, streaming ingest
├── 03_BANKING_SECURITY_COMPLIANCE.md   # R3: Decree 13 STRIDE model, Circular 09 Maker-Checker web assertion, envelope encryption
└── 04_MIGRATION_ROADMAP_QUALITY_GATE.md# R4: 4-phase roadmap, 14-command API parity matrix, OpenAPI 3.1 & WS specs, test suites
```

### Detailed Document Summaries:

| Document | Milestone | Core Focus & Authoritative Scope | Primary Contributors |
|---|---|---|---|
| [`01_ARCHITECTURAL_GAP_AUDIT.md`](./01_ARCHITECTURAL_GAP_AUDIT.md) | **M1 (R1)** | Catalog of 100% of runtime failure points in web browsers; deep audit of 12 coupled frontend files; W3C File sandbox barriers; Windows DPAPI and Win32 `MessageBoxW` modal breakages. | Worker 1 & Explorer 1, 2, 3 |
| [`02_HYBRID_ARCHITECTURE_BLUEPRINT.md`](./02_HYBRID_ARCHITECTURE_BLUEPRINT.md) | **M2 (R2)** | Complete hybrid system blueprint with Mermaid flowcharts; idiomatic TypeScript `IPlatformAdapter`, `TauriAdapter`, and `WebAdapter` implementations; in-memory streaming multipart upload pipeline (< 64MB). | Worker 1 & Explorer 1, 2 |
| [`03_BANKING_SECURITY_COMPLIANCE.md`](./03_BANKING_SECURITY_COMPLIANCE.md) | **M3 (R3)** | Complete STRIDE threat model across 7 browser attack vectors; Decree 13 Zero Data Egress technical controls (TLS 1.3, CSP Level 3, anti-caching headers, `useMemoryScrubber` composable); Circular 09 Maker-Checker server-asserted identity (`__Host-` cookies, 15-min HMAC tokens, atomic SQLite transitions); Two-Tier Envelope Encryption (Vault/KMS/Argon2id KEK + DEK) maintaining Debt C3 release guard. | Worker 2 & Explorer 3 |
| [`04_MIGRATION_ROADMAP_QUALITY_GATE.md`](./04_MIGRATION_ROADMAP_QUALITY_GATE.md) | **M4 (R4)** | 4-phase migration roadmap with quantitative test milestones, effort estimations (68 SP / 50 PD); exhaustive API parity matrix mapping all 14 Tauri commands to REST/WS; complete OpenAPI 3.1 & WebSocket schemas; 5-tier automated verification strategy; deployment risk matrix, 1-click fallback to Desktop, and disaster recovery. | Worker 2 & Explorer 2, 3 |

---

## 3. Architectural Decision Records (ADRs)

### ADR-01: Hybrid Platform Abstraction Layer (`IPlatformAdapter`)
- **Status:** APPROVED
- **Context:** `liva-ui` directly invoked Tauri IPC (`invokeBackend`, `@tauri-apps/api/*`), causing runtime exceptions outside Tauri desktop.
- **Decision:** Introduce a polymorphic TypeScript interface `IPlatformAdapter` injected at application boot:
  - `TauriAdapter`: Uses `@tauri-apps/api/core` and OS dialogs, guaranteeing 100% desktop fidelity and zero performance overhead.
  - `WebAdapter`: Uses HTTP `fetch` (with `credentials: 'same-origin'`), Server-Sent Events (SSE), and HTML5 File APIs.
- **Consequences:** Eliminates hardcoded `isTauri()` checks across UI components. Unlocks seamless web deployments while keeping desktop code identical.

### ADR-02: Zero-Disk In-Memory Streaming Multipart Ingestion
- **Status:** APPROVED
- **Context:** Desktop mode passed local filesystem paths (`file.path`) to `std::fs::read`. In web browsers, `file.path` is `undefined`. Spooling temporary files to server disk violates Decree 13/2023/NĐ-CP Zero Data Egress.
- **Decision:** Implement in-memory streaming multipart upload directly in the Rust Web Gateway using `multer` and `bytes::BytesMut`. Buffers are capped at 64MB under backpressure (`tokio::sync::Semaphore(2)`), parsed in-memory via `tokio::task::spawn_blocking`, encrypted into SQLite WAL, and zeroized from RAM. Zero bytes are written to disk.
- **Consequences:** 100% compliance with Decree 13. High throughput with $< 680\text{ MB}$ peak RAM even under 50,000-line statement load.

### ADR-03: Two-Tier Server-Side Envelope Encryption (KEK/DEK)
- **Status:** APPROVED
- **Context:** Desktop mode used Windows DPAPI (`CryptProtectData`) and blocking GUI dialogs (`MessageBoxW`), which fail in headless Linux / Docker / Web environments.
- **Decision:** Implement a two-tier envelope encryption architecture:
  - Key Encryption Key (KEK): Escrowed in HashiCorp Vault Transit Engine, PKCS#11 HSM, Linux Kernel Keyring, or Argon2id passphrase derivation.
  - Data Encryption Key (DEK): 256-bit AES-256-GCM v2 key held in memory-locked (`mlock`) segments.
  - Release Guard Continuity: Maintain Debt C3 compile-time panic guard against `DEFAULT_ENCRYPTION_KEY`.
  - Zero-Knowledge Web Client: Browser clients never receive or process master encryption keys.
- **Consequences:** Enables secure headless on-premise and container deployments with zero key exposure.

### ADR-04: Server-Asserted Maker-Checker Identity & Human Operator Dual Control
- **Status:** APPROVED
- **Context:** Legacy desktop IPC accepted `maker_id` and `checker_id` as client-supplied string parameters. In a web environment, this allows trivial self-approval bypass and impersonation. Furthermore, human operators possessing multiple login accounts (e.g. `user_maker` and `user_checker`) could bypass dual control if checks only evaluate mutable username strings.
- **Decision:** 
  1. Eliminate client-supplied identity parameters entirely from REST APIs and MCP tools (`treasury_payment_order`). All Maker and Checker claims are asserted server-side from `__Host-LIVA-Session; Secure; HttpOnly; SameSite=Strict` cookies.
  2. Bind dual control identity to immutable natural person identifiers (`employee_id` and `citizen_id_hash`), strictly enforcing that the maker and checker are two distinct human operators.
  3. State transitions require single-use 15-minute HMAC tokens verified strictly via constant-time comparison (`subtle::ConstantTimeEq`) to eliminate timing side-channels.
  4. Transitions execute via atomic SQLite conditional updates (`UPDATE ... WHERE status = 'PENDING' AND token_consumed = 0 AND expires_at >= now`).
- **Consequences:** Absolute adherence to Circular 09/2020/TT-NHNN Articles 16 & 18. Zero possibility of self-approval (even via multi-account manipulation), timing side-channels, or race-condition double spending.

### ADR-05: Strict Content Security Policy (CSP Level 3), Pinned Egress & Browser Memory Hygiene
- **Status:** APPROVED
- **Context:** Browser environments are vulnerable to XSS, top-level window redirection, unrestricted WebSocket egress, persistent disk caching, and memory retention in the V8 garbage collector.
- **Decision:** 
  1. Web Gateway injects Strict CSP Level 3 pinning WebSocket connections strictly to `connect-src 'self' wss://banking.corp.vn;` (wildcard `wss:` removed), adds `navigate-to 'self';`, and enforces anti-caching headers (`Cache-Control: no-store, private`).
  2. Implement `useMemoryScrubber` composable in `liva-ui` to actively sever object references upon component unmount and after 180s tab inactivity. Raw sensitive buffers must use `Uint8Array` with in-place zeroization via `crypto.getRandomValues()`.
  3. Document honest V8 heap memory boundaries: V8 strings are immutable primitives; setting properties to empty strings does not zero C++ char buffers until GC collects pages. Defense-in-depth relies on server-side PII masking before client transit.
- **Consequences:** Complete elimination of arbitrary WebSocket exfiltration and top-level redirection attacks. Prevents client-side PII leakage on shared workstations and neutralizes DOM exfiltration vectors.

---

## 4. Regulatory Compliance Assurance Statement

The architecture specified in this dossier strictly complies with all applicable Vietnamese financial and data privacy statutes:

```
                            REGULATORY ASSURANCE MATRIX
┌──────────────────────────────────────┬──────────────────────────────────────┐
│ Regulatory Framework                 │ Technical Control & Enforcement      │
├──────────────────────────────────────┼──────────────────────────────────────┤
│ Decree 13/2023/NĐ-CP (Art. 4, 9, 13)  │ - In-memory multipart streaming (SAX)│
│ Vietnam Personal Data Protection     │ - Zero disk spooling                 │
│                                      │ - Real-time PII regex sanitization   │
│                                      │ - Strict CSP L3 (wss:// pinned,      │
│                                      │   navigate-to 'self')                │
│                                      │ - Uint8Array crypto zeroization      │
│                                      │ - Masked DB errors (correlation ID)  │
│                                      │ - TLS 1.3 PFS ciphers only           │
├──────────────────────────────────────┼──────────────────────────────────────┤
│ Circular 09/2020/TT-NHNN (Art. 16, 18)│ - Server-asserted identity cookies   │
│ Dual Control / 4-Eyes Principle      │ - Natural person dual control        │
│                                      │   (employee_id / citizen_id_hash)    │
│                                      │ - Anti-multi-account self-approval   │
│                                      │ - Constant-time token verification   │
│                                      │   (subtle::ConstantTimeEq)           │
│                                      │ - MCP tool server-asserted identity  │
│                                      │ - Single-use 15-min HMAC tokens      │
│                                      │ - Atomic SQLite state transitions    │
│                                      │ - Non-repudiation digital signatures │
├──────────────────────────────────────┼──────────────────────────────────────┤
│ Law on AML No. 14/2022/QH15 &        │ - Decision 11/2023/QĐ-TTg rules      │
│ Decision 11/2023/QĐ-TTg              │ - High-value threshold (>= 400M VND) │
│                                      │ - Structuring & velocity detection   │
├──────────────────────────────────────┼──────────────────────────────────────┤
│ RFC 6962 & RFC 2104                  │ - Binary Merkle Tree O(log N) proofs │
│ Tamper-Evident Audit Ledgers         │ - Forward HMAC-SHA256 audit chain    │
│                                      │   (banking_audit_chain table)        │
│                                      │ - Argon2id KEK entropy >= 128 bits   │
│                                      │ - Container cap_add: [IPC_LOCK]      │
└──────────────────────────────────────┴──────────────────────────────────────┘
```

---

## 5. Verification Checklist & Quality Gate Signoff

Before declaring production readiness, the migration implementation must achieve 100% signoff across the following quality gate checkpoints:

- [ ] **M1: Architecture Gap Remediation**
  - [ ] All 12 coupled frontend files refactored to consume `IPlatformAdapter`.
  - [ ] Desktop Tauri app builds and executes without any regression (`cargo check -j 2`).
  - [ ] Web browser client boots cleanly without uncaught `window.__TAURI_INTERNALS__` exceptions.

- [ ] **M2: Hybrid Abstraction & Streaming Ingestion**
  - [ ] `TauriAdapter` and `WebAdapter` pass 100% unit tests (`npm run test -w liva-ui`).
  - [ ] In-memory multipart upload parses XLSX, CSV, PDF, and XML without creating temporary disk files.
  - [ ] Streaming ISO 20022 SAX parser (`quick-xml::Reader`) consumes $<40\text{ MB}$ RAM on 150MB XML files.
  - [ ] Concurrency gate enforces maximum 2 concurrent uploads, shedding excess load with `HTTP 429`.
  - [ ] SHA-256 `file_hash` idempotency check rejects duplicate statement uploads.

- [ ] **M3: Banking Security & Compliance Hardening**
  - [ ] Web Gateway enforces TLS 1.3 only with PFS ciphers (`TLS_AES_256_GCM_SHA384`, `TLS_CHACHA20_POLY1305_SHA256`).
  - [ ] Strict CSP Level 3 header pins `connect-src 'self' wss://banking.corp.vn;` and enforces `navigate-to 'self';`.
  - [ ] Circular 09 dual control verified against multi-account human self-approval (`test_maker_checker_fail_closed_on_same_employee_id`).
  - [ ] Token signature verification executes in constant time via `subtle::ConstantTimeEq` (`test_hitl_token_constant_time_comparison`).
  - [ ] MCP tool `treasury_payment_order` strictly rejects client-supplied identity claims and asserts server session claims.
  - [ ] Single-use 15-minute HITL tokens verified against replay and expiry attacks (`test_maker_checker_token_ttl_expiration`).
  - [ ] Database error masking verified (`test_database_error_masked_with_correlation_id`).
  - [ ] Two-tier envelope encryption derives KEK with >= 128-bit entropy and container specifies `cap_add: [IPC_LOCK]`.
  - [ ] Debt C3 release guard aborts execution if `DEFAULT_ENCRYPTION_KEY` is referenced in release build.
  - [ ] `useMemoryScrubber` zeroes reactive arrays, wipes `Uint8Array` buffers, and triggers screen lock after 180s inactivity.

- [ ] **M4: Roadmap, Benchmarking & E2E Validation**
  - [ ] 50,000-line bank statement parsed and reconciled in $< 4.5\text{ s}$ under Web Gateway.
  - [ ] Peak RAM usage remains $\le 680\text{ MB}$ under 50k benchmark load.
  - [ ] Playwright E2E browser automation passes across Chrome, Edge, Firefox, and Safari.
  - [ ] Adversarial challenge test suite (6/6 scenarios) passes with 0 failures.
  - [ ] 1-Click fail-closed fallback to Desktop mode verified under simulated gateway outage.
  - [ ] Full signoff achieved from Independent Forensic Auditor (`teamwork_preview_auditor`).
