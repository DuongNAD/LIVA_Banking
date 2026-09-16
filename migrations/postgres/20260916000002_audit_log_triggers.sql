-- Migration: 20260916000002_audit_log_triggers.sql
-- Description: Enforce append-only invariants on audit_logs via DB triggers (PostgreSQL dialect)
-- Guarantees that any direct UPDATE, DELETE, or TRUNCATE query on audit_logs fails unconditionally.

-- Trigger function: Raise exception on forbidden mutations
CREATE OR REPLACE FUNCTION fn_forbid_audit_logs_mutation()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'INVARIANT_VIOLATION: audit_logs is append-only; % operation is strictly prohibited.', TG_OP;
END;
$$ LANGUAGE plpgsql;

-- Trigger: Forbid UPDATE on audit_logs
DROP TRIGGER IF EXISTS trg_audit_logs_no_update ON audit_logs;
CREATE TRIGGER trg_audit_logs_no_update
BEFORE UPDATE ON audit_logs
FOR EACH ROW
EXECUTE FUNCTION fn_forbid_audit_logs_mutation();

-- Trigger: Forbid DELETE on audit_logs
DROP TRIGGER IF EXISTS trg_audit_logs_no_delete ON audit_logs;
CREATE TRIGGER trg_audit_logs_no_delete
BEFORE DELETE ON audit_logs
FOR EACH ROW
EXECUTE FUNCTION fn_forbid_audit_logs_mutation();

-- Trigger: Forbid TRUNCATE on audit_logs (statement-level)
DROP TRIGGER IF EXISTS trg_audit_logs_no_truncate ON audit_logs;
CREATE TRIGGER trg_audit_logs_no_truncate
BEFORE TRUNCATE ON audit_logs
FOR EACH STATEMENT
EXECUTE FUNCTION fn_forbid_audit_logs_mutation();
