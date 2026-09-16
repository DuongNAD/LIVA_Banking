# LIVA Banking Database Migrations

This directory contains versioned SQL migrations for the LIVA Banking Harness Client-Server V2 architecture.

## Supported Dialects

1. **Root (`migrations/`)**: Universal ANSI/SQLite migrations matching local test harnesses, desktop embedded mode, and edge deployments.
2. **SQLite (`migrations/sqlite/`)**: Tailored for SQLite / SQLCipher with `INTEGER PRIMARY KEY AUTOINCREMENT` and `SELECT RAISE(ABORT, ...)` triggers.
3. **PostgreSQL (`migrations/postgres/`)**: Tailored for PostgreSQL production deployments with `BIGSERIAL PRIMARY KEY` and PL/pgSQL statement & row-level triggers preventing `UPDATE`, `DELETE`, and `TRUNCATE`.

## Entities Covered (10 Tables)

1. `legal_entities` - Corporate entity master (tax ID, legal name, base currency).
2. `fiscal_periods` - Accounting periods with balance tracking and uniqueness on `(entity_id, period_code)`.
3. `counterparties` - Counterparty master (customers, vendors) with normalized names.
4. `counterparty_aliases` - Narration string alias mappings linked to counterparties with cascading delete.
5. `holidays` - Vietnamese banking clearing calendar holidays.
6. `bank_accounts` - Corporate bank accounts with balance tracking and hashed account lookup.
7. `bank_profiles` - Ingest parser definitions, column mappings, and fee calculation rules.
8. `bank_transactions` - Canonical transactions with `txn_hash` idempotency uniqueness and `statement_fingerprint`.
9. `quarantine_items` - Review queue enforcing Segregation of Duties via `CONSTRAINT chk_maker_checker CHECK (maker_id != checker_id)`.
10. `audit_logs` - Cryptographic tamper-evident append-only log with SHA-256 hash chaining and DB trigger protection.

## DB Invariants Enforced

- **Append-Only `audit_logs`**: DB triggers forbid any `UPDATE` or `DELETE` at the engine level.
- **Segregation of Duties (Four-Eyes Principle)**: `CHECK (maker_id != checker_id)` prevents the submitter from approving their own quarantined transaction.
- **Continuous SHA-256 Hash Chain**: Every record links to the previous row via `prev_hash` anchored at `LIVA_BANKING_GENESIS_2026`.
