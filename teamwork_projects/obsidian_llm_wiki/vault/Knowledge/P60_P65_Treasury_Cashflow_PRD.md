---
title: "P60_P65_Treasury_Cashflow_PRD"
tags:
  - liva/knowledge
  - liva/treasury
  - liva/cashflow
  - liva/p60
  - liva/p61
  - liva/p62
  - liva/p65
author: "architect"
last_update: "2026-09-15T20:20:00Z"
---

# Product Requirements Document (PRD): P60–P65 Treasury Management & Rolling Cashflow Sentinel

## 1. Executive Summary & Regulatory Mandate
- **Target Persona**: **R02 — Thủ quỹ / Chuyên viên thanh toán (Maker)**, **R03 — Giám đốc tài chính (CFO / Approver)**, **R04 — Kế toán trưởng (Checker)**, **R07 — Quản trị rủi ro thanh khoản**.
- **Regulatory Framework**:
  - Circular 09/2020/TT-NHNN on Safety & Security in Online Banking & Electronic Fund Transfers.
  - Law on Prevention of Money Laundering (AML) 2022.
  - Zero Data Egress: HSM keys, OTP secrets, bank tokens, and payment vouchers remain strictly on-premise.
- **Mission**: Provide multi-bank cash concentration monitoring, proactive 30/90-day cash deficit forecasting (Rolling Cashflow Sentinel), and an ironclad Maker-Checker payment disbursement gateway ($maker \ne checker$) with single-use cryptographic tokens.

---

## 2. Architecture & Modules Decomposition

```
          ┌──────────────────────────────────────────┐
          │  P60: Cash Concentration Multi-Bank Pool │ ── Real-time balance aggregation
          └────────────────────┬─────────────────────┘    (VCB, TCB, BIDV, MBB)
                               │
                               ▼
          ┌──────────────────────────────────────────┐
          │  P61: Rolling 30/90-day Sentinel Engine  │ ── Cash runway & deficit alerts
          └────────────────────┬─────────────────────┘    (Operating reserve guardrails)
                               │
                               ▼
          ┌──────────────────────────────────────────┐
          │  P62: Payment Orders & Dual Control Gate │ ── Circular 09 Maker-Checker
          └──────────────────────────────────────────┘    (maker_id != checker_id, HSM OTP)
```

---

## 3. Cash Concentration Policies (P60)

| Account Type | Bank Partner | Target Buffer (Min) | Sweeping Threshold | Concentration Action |
|---|---|---|---|---|
| Main Operating Account | Vietcombank (VCB) | $1,000,000,000\text{ VND}$ | Surplus $> 2,000,000,000\text{ VND}$ | Concentration Master Pool |
| Collections Account | Techcombank (TCB) | $200,000,000\text{ VND}$ | Sweep surplus $> 500,000,000\text{ VND}$ | Daily auto-sweep to VCB Master |
| Payroll & Tax Account | BIDV | $300,000,000\text{ VND}$ | Sweep if deficit $< 300,000,000\text{ VND}$ | Top-up from VCB Master Pool |
| Secondary Collections | MBBank (MBB) | $100,000,000\text{ VND}$ | Sweep surplus $> 300,000,000\text{ VND}$ | Daily auto-sweep to VCB Master |

---

## 4. Rolling Cashflow Sentinel SLA & Metrics (P61)

1. **Liquidity Runway**:
   $$\text{Runway (Days)} = \frac{\text{Total Available Cash}}{\text{Average Daily Net Burn Rate}}$$
2. **Deficit Warning Barrier**:
   - **RED ALERT (Critical)**: Projected cash drops below Minimum Safe Operating Reserve ($500,000,000\text{ VND}$) within 14 days.
   - **AMBER WARNING**: Projected cash drops below Target Operating Buffer within 30 days.
   - **GREEN**: Positive liquidity runway $> 90$ days.

---

## 5. Dual Control Payment Gate Invariants (P62)

1. **Segregation of Duties (SoD)**:
   $$\text{payment\_maker\_id} \ne \text{payment\_checker\_id}$$
2. **High-Value Threshold Approval**:
   - Payments $\ge 500,000,000\text{ VND}$ require 3-tier sign-off: Maker (Thủ quỹ) $\to$ Checker (Kế toán trưởng) $\to$ Final Authorizer (CFO / Tổng giám đốc).
3. **Idempotency Protection**:
   - Payment order UUIDv4 (`payment_guid`) ensures network retry cannot execute duplicate wire transfers.
