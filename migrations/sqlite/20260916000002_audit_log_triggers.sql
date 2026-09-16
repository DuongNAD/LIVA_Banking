-- Migration: 20260916000002_audit_log_triggers.sql
-- Description: Enforce append-only invariants on audit_logs via DB triggers (SQLite dialect)
-- Guarantees that any direct UPDATE or DELETE query on audit_logs fails unconditionally.

-- Trigger: Forbid UPDATE on audit_logs
CREATE TRIGGER IF NOT EXISTS trg_audit_logs_no_update
BEFORE UPDATE ON audit_logs
FOR EACH ROW
BEGIN
    SELECT RAISE(ABORT, 'INVARIANT_VIOLATION: audit_logs is append-only; UPDATE operation is strictly prohibited.');
END;

-- Trigger: Forbid DELETE on audit_logs
CREATE TRIGGER IF NOT EXISTS trg_audit_logs_no_delete
BEFORE DELETE ON audit_logs
FOR EACH ROW
BEGIN
    SELECT RAISE(ABORT, 'INVARIANT_VIOLATION: audit_logs is append-only; DELETE operation is strictly prohibited.');
END;

-- Trigger: Forbid INSERT OR REPLACE / conflict mutation on audit_logs
CREATE TRIGGER IF NOT EXISTS trg_audit_logs_no_replace
BEFORE INSERT ON audit_logs
WHEN (NEW.seq_id IS NOT NULL AND EXISTS (SELECT 1 FROM audit_logs WHERE seq_id = NEW.seq_id))
   OR EXISTS (SELECT 1 FROM audit_logs WHERE row_hash = NEW.row_hash)
BEGIN
    SELECT RAISE(ABORT, 'INVARIANT_VIOLATION: audit_logs is append-only; INSERT OR REPLACE / collision mutation is strictly prohibited.');
END;
