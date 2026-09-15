---
title: "LIVA Banking Reconciliation"
tags:
  - liva/skill
  - liva/banking
  - liva/reconciliation
  - liva/audit
author: "codex"
last_update: "2026-09-14T20:11:06+07:00"
---

# LIVA Banking Reconciliation

Canonical instructions live in `.claude/skills/liva-banking-reconciliation/SKILL.md` and the mirrored `.agents/skills/liva-banking-reconciliation/SKILL.md`.

Use this skill to parse multi-bank statements (Vietcombank, Techcombank, BIDV, VietinBank, MBBank, Agribank, and ISO 20022 camt.053 XML), verify mathematical balance invariants, and execute zero-hallucination 3-tier reconciliation (1:1 exact, fuzzy heuristic, 1:N / N:1 composite split solver) against the ERP ledger.
