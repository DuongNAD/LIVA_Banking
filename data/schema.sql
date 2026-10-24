-- ==============================================================================
-- LIVA Banking Core — SQL Relational Schema
-- Standard: ISO 20022, Circular 09/2020/TT-NHNN, Decree 13/2023/ND-CP
-- Compatible: SQLite 3, PostgreSQL, MySQL
-- ==============================================================================

PRAGMA foreign_keys = ON;

-- 1. Banking Settlement Accounts & Liquidity Channels
CREATE TABLE IF NOT EXISTS accounts (
    id TEXT PRIMARY KEY,
    bank_code TEXT NOT NULL,                -- VCB, TCB, BIDV, CTG, MBB
    channel_code TEXT NOT NULL,             -- CITAD, NAPAS, BILATERAL, SWIFT
    bank_name TEXT NOT NULL,
    channel_name TEXT NOT NULL,
    clearing_authority TEXT NOT NULL,
    account_number TEXT NOT NULL UNIQUE,
    account_name TEXT NOT NULL,
    balance_vnd INTEGER NOT NULL DEFAULT 0, -- Số dư hiện tại (VND, bigint)
    opening_balance_vnd INTEGER NOT NULL DEFAULT 0,
    minimum_reserve_vnd INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_accounts_bank ON accounts(bank_code);
CREATE INDEX IF NOT EXISTS idx_accounts_channel ON accounts(channel_code);

-- 2. Core Banking General Ledger Invoices & Receivables/Payables
CREATE TABLE IF NOT EXISTS invoices (
    id TEXT PRIMARY KEY,
    invoice_no TEXT NOT NULL UNIQUE,
    partner_code TEXT NOT NULL,
    partner_name TEXT NOT NULL,
    tax_code TEXT NOT NULL,
    contract_no TEXT,
    expected_amount INTEGER NOT NULL,       -- Số tiền dự kiến (VND, bigint)
    direction TEXT NOT NULL CHECK(direction IN ('PAYABLE', 'RECEIVABLE')),
    due_date TEXT NOT NULL,
    issue_date TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'PENDING' CHECK(status IN ('PENDING', 'MATCHED', 'PARTIALLY_MATCHED', 'DISPUTED')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_invoices_status ON invoices(status);
CREATE INDEX IF NOT EXISTS idx_invoices_partner ON invoices(partner_code);

-- 3. Interbank Bank Statements (Multi-Bank Ingestion)
CREATE TABLE IF NOT EXISTS bank_statements (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    bank_code TEXT NOT NULL,
    channel_code TEXT NOT NULL,
    txn_ref TEXT NOT NULL UNIQUE,
    booking_date TEXT NOT NULL,
    value_date TEXT NOT NULL,
    debit_amount INTEGER NOT NULL DEFAULT 0,    -- Phát sinh Nợ (VND, bigint)
    credit_amount INTEGER NOT NULL DEFAULT 0,   -- Phát sinh Có (VND, bigint)
    balance_after INTEGER NOT NULL,             -- Số dư sau giao dịch
    fee_amount INTEGER NOT NULL DEFAULT 0,      -- Phí chuyển khoản liên ngân hàng
    partner_name TEXT,
    counterpart_account TEXT,
    description TEXT NOT NULL,
    invoice_ref TEXT,                           -- Mã hóa đơn đối chiếu nếu có
    match_status TEXT NOT NULL DEFAULT 'UNMATCHED' CHECK(match_status IN ('UNMATCHED', 'EXACT_MATCH', 'FUZZY_MATCH', 'SPLIT_MATCH', 'EXCEPTION', 'RESOLVED')),
    reconciliation_tier INTEGER DEFAULT 0,      -- 1: Exact, 2: Fuzzy, 3: Split
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_statements_account ON bank_statements(account_id);
CREATE INDEX IF NOT EXISTS idx_statements_status ON bank_statements(match_status);
CREATE INDEX IF NOT EXISTS idx_statements_date ON bank_statements(booking_date);

-- 4. Maker-Checker Payment Vouchers (Circular 09/2020/TT-NHNN)
CREATE TABLE IF NOT EXISTS payment_vouchers (
    id TEXT PRIMARY KEY,
    voucher_no TEXT NOT NULL UNIQUE,
    source_account_id TEXT NOT NULL REFERENCES accounts(id),
    target_bank TEXT NOT NULL,
    target_account TEXT NOT NULL,
    beneficiary_name TEXT NOT NULL,
    amount_vnd INTEGER NOT NULL,
    channel TEXT NOT NULL CHECK(channel IN ('CITAD', 'NAPAS', 'BILATERAL', 'SWIFT')),
    purpose TEXT NOT NULL,
    maker_id TEXT NOT NULL,
    maker_name TEXT NOT NULL,
    checker_id TEXT,
    checker_name TEXT,
    status TEXT NOT NULL DEFAULT 'DRAFT' CHECK(status IN ('DRAFT', 'PENDING_APPROVAL', 'APPROVED', 'REJECTED', 'EXECUTED')),
    otp_code TEXT,
    otp_verified_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    approved_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_vouchers_status ON payment_vouchers(status);
CREATE INDEX IF NOT EXISTS idx_vouchers_maker ON payment_vouchers(maker_id);

-- 5. AML / CTF Suspicious Transaction Surveillance (Decree 13/2023 & Circular 09/2023)
CREATE TABLE IF NOT EXISTS aml_alerts (
    id TEXT PRIMARY KEY,
    statement_id TEXT REFERENCES bank_statements(id) ON DELETE SET NULL,
    rule_code TEXT NOT NULL,            -- AML-SMURF, AML-PASSTHROUGH, AML-NIGHT-VELOCITY, AML-HIGH-VALUE
    severity TEXT NOT NULL CHECK(severity IN ('CRITICAL', 'HIGH', 'MEDIUM', 'LOW')),
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    trigger_amount INTEGER NOT NULL,
    partner_name TEXT,
    is_str_reported INTEGER NOT NULL DEFAULT 0, -- 1 if STR Annex II generated
    str_report_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_aml_severity ON aml_alerts(severity);
CREATE INDEX IF NOT EXISTS idx_aml_rule ON aml_alerts(rule_code);

-- 6. Forward Merkle Audit Tree Ledger (SHA-256 Tamper-Evident Chain)
CREATE TABLE IF NOT EXISTS merkle_ledger (
    seq_num INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL,           -- VOUCHER_CREATED, VOUCHER_APPROVED, RECON_MATCHED, AML_ALERT
    entity_id TEXT NOT NULL,
    payload_hash TEXT NOT NULL,         -- SHA-256(canonical JSON)
    prev_root TEXT,
    current_root TEXT NOT NULL,         -- Root hash of Merkle tree
    actor_id TEXT NOT NULL,
    timestamp TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_merkle_seq ON merkle_ledger(seq_num);
