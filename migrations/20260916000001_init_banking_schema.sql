-- Migration: 20260916000001_init_banking_schema.sql
-- Description: Core Banking Harness schema for 10 entities (SQLite / ANSI compatible)
-- Entities:
--   1. legal_entities
--   2. fiscal_periods
--   3. counterparties
--   4. counterparty_aliases
--   5. holidays
--   6. bank_accounts
--   7. bank_profiles
--   8. bank_transactions
--   9. quarantine_items (with Segregation of Duties: CHECK maker_id != checker_id)
--  10. audit_logs (append-only audit trail with sequence and hash chain anchors)

-- 1. legal_entities: Corporate entities registered in LIVA Banking
CREATE TABLE IF NOT EXISTS legal_entities (
    id TEXT PRIMARY KEY,
    tax_id TEXT NOT NULL UNIQUE,
    legal_name TEXT NOT NULL,
    trading_name TEXT,
    country TEXT NOT NULL DEFAULT 'VN',
    base_currency TEXT NOT NULL DEFAULT 'VND',
    address TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_legal_entities_tax_id ON legal_entities(tax_id);

-- 2. fiscal_periods: Accounting fiscal periods under Circular 200/2014/TT-BTC
CREATE TABLE IF NOT EXISTS fiscal_periods (
    id TEXT PRIMARY KEY,
    entity_id TEXT NOT NULL REFERENCES legal_entities(id),
    period_code TEXT NOT NULL,
    start_date BIGINT NOT NULL,
    end_date BIGINT NOT NULL,
    status TEXT NOT NULL DEFAULT 'OPEN',
    opening_balance BIGINT NOT NULL DEFAULT 0,
    closing_balance BIGINT,
    closed_by TEXT,
    closed_at BIGINT,
    created_at BIGINT NOT NULL,
    CONSTRAINT uq_entity_period UNIQUE(entity_id, period_code)
);
CREATE INDEX IF NOT EXISTS idx_fiscal_periods_dates ON fiscal_periods(entity_id, start_date, end_date);

-- 3. counterparties: Master directory of customers and vendors
CREATE TABLE IF NOT EXISTS counterparties (
    id TEXT PRIMARY KEY,
    entity_id TEXT NOT NULL REFERENCES legal_entities(id),
    partner_code TEXT NOT NULL,
    legal_name TEXT NOT NULL,
    tax_id TEXT,
    normalized_name TEXT NOT NULL,
    partner_type TEXT NOT NULL DEFAULT 'CUSTOMER',
    default_account TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    CONSTRAINT uq_entity_partner UNIQUE(entity_id, partner_code)
);
CREATE INDEX IF NOT EXISTS idx_counterparties_normalized ON counterparties(normalized_name);
CREATE INDEX IF NOT EXISTS idx_counterparties_tax ON counterparties(tax_id);

-- 4. counterparty_aliases: Mapping from narration strings to canonical counterparties
CREATE TABLE IF NOT EXISTS counterparty_aliases (
    id TEXT PRIMARY KEY,
    counterparty_id TEXT NOT NULL REFERENCES counterparties(id) ON DELETE CASCADE,
    alias_raw TEXT NOT NULL,
    alias_normalized TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'MANUAL',
    confidence REAL NOT NULL DEFAULT 1.0,
    created_at BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_aliases_normalized ON counterparty_aliases(alias_normalized);
CREATE INDEX IF NOT EXISTS idx_aliases_counterparty ON counterparty_aliases(counterparty_id);

-- 5. holidays: Vietnamese banking calendar holidays for cutoff adjustments
CREATE TABLE IF NOT EXISTS holidays (
    id TEXT PRIMARY KEY,
    holiday_date TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    is_banking_holiday BOOLEAN NOT NULL DEFAULT TRUE,
    country TEXT NOT NULL DEFAULT 'VN',
    created_at BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_holidays_date ON holidays(holiday_date);

-- 6. bank_accounts: Corporate bank accounts with balance tracking
CREATE TABLE IF NOT EXISTS bank_accounts (
    id TEXT PRIMARY KEY,
    entity_id TEXT NOT NULL REFERENCES legal_entities(id),
    bank_code TEXT NOT NULL,
    account_number_enc TEXT NOT NULL,
    account_number_hash TEXT NOT NULL,
    account_name TEXT NOT NULL,
    currency TEXT NOT NULL DEFAULT 'VND',
    opening_balance BIGINT NOT NULL DEFAULT 0,
    current_balance BIGINT NOT NULL DEFAULT 0,
    gl_account_code TEXT NOT NULL DEFAULT '1121',
    status TEXT NOT NULL DEFAULT 'ACTIVE',
    last_synced_at BIGINT,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    CONSTRAINT uq_bank_account_hash UNIQUE(bank_code, account_number_hash)
);
CREATE INDEX IF NOT EXISTS idx_bank_accounts_entity ON bank_accounts(entity_id);
CREATE INDEX IF NOT EXISTS idx_bank_accounts_lookup ON bank_accounts(bank_code, status);

-- 7. bank_profiles: Configurable parser metadata, column mappings, and fee rules
CREATE TABLE IF NOT EXISTS bank_profiles (
    id TEXT PRIMARY KEY,
    bank_code TEXT NOT NULL,
    profile_name TEXT NOT NULL,
    file_format TEXT NOT NULL,
    delimiter TEXT,
    header_row_index INTEGER NOT NULL DEFAULT 0,
    data_start_row INTEGER NOT NULL DEFAULT 1,
    date_format TEXT NOT NULL DEFAULT 'DD/MM/YYYY',
    column_mapping_json TEXT NOT NULL,
    fee_rules_json TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    CONSTRAINT uq_bank_profile UNIQUE(bank_code, profile_name)
);
CREATE INDEX IF NOT EXISTS idx_bank_profiles_lookup ON bank_profiles(bank_code, file_format, is_active);

-- 8. bank_transactions: Canonical normalized transactions with idempotency hashes
CREATE TABLE IF NOT EXISTS bank_transactions (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES bank_accounts(id),
    statement_fingerprint TEXT NOT NULL,
    txn_hash TEXT NOT NULL UNIQUE,
    tx_date BIGINT NOT NULL,
    value_date BIGINT,
    doc_ref TEXT,
    tx_type TEXT NOT NULL,
    amount BIGINT NOT NULL,
    is_credit BOOLEAN NOT NULL,
    balance_after BIGINT NOT NULL,
    narration TEXT NOT NULL,
    normalized_narration TEXT,
    counterparty_account TEXT,
    counterparty_name TEXT,
    counterparty_bank TEXT,
    reconciled_status TEXT NOT NULL DEFAULT 'UNMATCHED',
    reconciled_match_id TEXT,
    created_at BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_bank_tx_fingerprint ON bank_transactions(statement_fingerprint);
CREATE INDEX IF NOT EXISTS idx_bank_tx_lookup ON bank_transactions(account_id, tx_date, amount, reconciled_status);
CREATE INDEX IF NOT EXISTS idx_bank_tx_reconcile ON bank_transactions(reconciled_status, tx_date);

-- 9. quarantine_items: Isolation queue enforcing Four-Eyes Principle / Segregation of Duties
CREATE TABLE IF NOT EXISTS quarantine_items (
    id TEXT PRIMARY KEY,
    bank_tx_id TEXT NOT NULL REFERENCES bank_transactions(id),
    account_id TEXT NOT NULL REFERENCES bank_accounts(id),
    amount BIGINT NOT NULL,
    quarantine_reason TEXT NOT NULL,
    confidence_score REAL,
    hitl_token TEXT NOT NULL UNIQUE,
    maker_id TEXT NOT NULL,
    checker_id TEXT,
    status TEXT NOT NULL DEFAULT 'PENDING_REVIEW',
    resolution_action TEXT,
    resolution_notes TEXT,
    created_at BIGINT NOT NULL,
    resolved_at BIGINT,
    expires_at BIGINT NOT NULL,
    CONSTRAINT chk_maker_checker CHECK (maker_id != checker_id)
);
CREATE INDEX IF NOT EXISTS idx_quarantine_tx ON quarantine_items(bank_tx_id);
CREATE INDEX IF NOT EXISTS idx_quarantine_status ON quarantine_items(status, expires_at);
CREATE INDEX IF NOT EXISTS idx_quarantine_token ON quarantine_items(hitl_token);

-- 10. audit_logs: Cryptographic append-only log with SHA-256 hash chaining
CREATE TABLE IF NOT EXISTS audit_logs (
    seq_id INTEGER PRIMARY KEY AUTOINCREMENT,
    prev_hash TEXT NOT NULL,
    row_hash TEXT NOT NULL UNIQUE,
    timestamp BIGINT NOT NULL,
    actor_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    payload_digest TEXT NOT NULL,
    signature TEXT NOT NULL,
    client_ip TEXT NOT NULL DEFAULT '127.0.0.1'
);
CREATE INDEX IF NOT EXISTS idx_audit_logs_lookup ON audit_logs(entity_type, entity_id, timestamp);
