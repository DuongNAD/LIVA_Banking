//! Adversarial Challenge Test Suite 2 for Milestone M2 Remediation Verification
//! Independent empirical challenge testing:
//! 1. SQLite INSERT OR REPLACE / REPLACE INTO trigger rejection on audit_logs.
//! 2. client_ip and signature cryptographic binding into compute_audit_row_hash and verify_audit_db_chain with 1-byte tamper detection.
//! 3. Segregation of duties on quarantine_items.

use liva_audit::{
    compute_audit_row_hash, digest_payload, genesis_hash, verify_audit_db_chain, AuditDbRecord,
};
use rusqlite::{params, Connection};

fn setup_challenger_db() -> Connection {
    let conn = Connection::open_in_memory().expect("Failed to create in-memory SQLite database");
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .expect("Failed to enable foreign keys");

    let schema_sql = include_str!("../../../migrations/20260916000001_init_banking_schema.sql");
    conn.execute_batch(schema_sql)
        .expect("Failed to apply initial schema migration");

    let triggers_sql = include_str!("../../../migrations/20260916000002_audit_log_triggers.sql");
    conn.execute_batch(triggers_sql)
        .expect("Failed to apply audit triggers migration");

    conn
}

fn seed_audit_chain(conn: &Connection, count: usize) -> Vec<AuditDbRecord> {
    let mut records = Vec::with_capacity(count);
    let mut prev = genesis_hash();
    let base_ts = 1773000000i64;

    for i in 1..=count {
        let seq = i as u64;
        let ts = base_ts + (i as i64 * 30);
        let actor = format!("bank_officer_{i}");
        let event = if i % 2 == 0 { "POST_TX" } else { "RECONCILE_TX" };
        let entity_type = "bank_transactions";
        let entity_id = format!("tx_entry_{i:04}");
        let payload = format!(r#"{{"tx_id":{i},"amount_vnd":{},"status":"COMMITTED"}}"#, i * 250_000);
        let payload_digest = digest_payload(payload.as_bytes());
        let client_ip = if i % 3 == 0 {
            "10.24.1.15".to_string()
        } else if i % 3 == 1 {
            "192.168.100.42".to_string()
        } else {
            "::1".to_string()
        };
        let signature = format!("ed25519_sig_{i:04}_{actor}");

        let row_h = compute_audit_row_hash(
            seq,
            &prev,
            ts,
            &actor,
            event,
            entity_type,
            &entity_id,
            &payload_digest,
            &client_ip,
            &signature,
        );

        conn.execute(
            "INSERT INTO audit_logs (
                prev_hash, row_hash, timestamp, actor_id,
                event_type, entity_type, entity_id, payload_digest,
                signature, client_ip
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                prev, row_h, ts, actor,
                event, entity_type, entity_id, payload_digest,
                signature, client_ip
            ],
        ).expect("Failed to seed audit log entry");

        records.push(AuditDbRecord {
            seq_id: seq,
            prev_hash: prev,
            row_hash: row_h.clone(),
            timestamp: ts,
            actor_id: actor,
            event_type: event.to_string(),
            entity_type: entity_type.to_string(),
            entity_id,
            payload_digest,
            signature,
            client_ip,
        });

        prev = row_h;
    }

    records
}

// =========================================================================
// Dimension 1: Empirical Verification of SQLite REPLACE / INSERT OR REPLACE
// =========================================================================

#[test]
fn challenge2_test_sqlite_insert_or_replace_variants_strictly_aborted() {
    let conn = setup_challenger_db();
    let records = seed_audit_chain(&conn, 5);
    assert_eq!(records.len(), 5);

    // 1. Attack Scenario: INSERT OR REPLACE by primary key seq_id = 1
    let attack_1 = conn.execute(
        "INSERT OR REPLACE INTO audit_logs (
            seq_id, prev_hash, row_hash, timestamp, actor_id,
            event_type, entity_type, entity_id, payload_digest,
            signature, client_ip
         ) VALUES (1, 'forged_prev_1', 'forged_row_1', 111111, 'malicious_impostor', 'FORGERY', 'tx', 'tx_001', 'digest', 'sig', '1.2.3.4')",
        [],
    );
    assert!(attack_1.is_err(), "INSERT OR REPLACE by seq_id must be rejected");
    let err_msg1 = attack_1.unwrap_err().to_string();
    assert!(
        err_msg1.contains("INVARIANT_VIOLATION") && err_msg1.contains("INSERT OR REPLACE"),
        "Unexpected error for INSERT OR REPLACE by seq_id: {err_msg1}"
    );

    // 2. Attack Scenario: REPLACE INTO keyword directly
    let attack_2 = conn.execute(
        "REPLACE INTO audit_logs (
            seq_id, prev_hash, row_hash, timestamp, actor_id,
            event_type, entity_type, entity_id, payload_digest,
            signature, client_ip
         ) VALUES (2, 'forged_prev_2', 'forged_row_2', 222222, 'malicious_impostor_2', 'FORGERY', 'tx', 'tx_002', 'digest', 'sig', '1.2.3.4')",
        [],
    );
    assert!(attack_2.is_err(), "REPLACE INTO must be rejected");
    let err_msg2 = attack_2.unwrap_err().to_string();
    assert!(
        err_msg2.contains("INVARIANT_VIOLATION"),
        "Unexpected error for REPLACE INTO: {err_msg2}"
    );

    // 3. Attack Scenario: INSERT OR REPLACE by row_hash collision (without seq_id)
    let existing_hash_3 = &records[2].row_hash; // seq_id = 3
    let attack_3 = conn.execute(
        "INSERT OR REPLACE INTO audit_logs (
            prev_hash, row_hash, timestamp, actor_id,
            event_type, entity_type, entity_id, payload_digest,
            signature, client_ip
         ) VALUES ('forged_prev_3', ?1, 333333, 'malicious_collision', 'FORGERY', 'tx', 'tx_003', 'digest', 'sig', '1.2.3.4')",
        params![existing_hash_3],
    );
    assert!(attack_3.is_err(), "INSERT OR REPLACE on row_hash collision must be rejected");
    let err_msg3 = attack_3.unwrap_err().to_string();
    assert!(
        err_msg3.contains("INVARIANT_VIOLATION"),
        "Unexpected error on row_hash collision: {err_msg3}"
    );

    // 4. Attack Scenario: String-coerced seq_id in INSERT OR REPLACE ('1' as TEXT)
    let attack_4 = conn.execute(
        "INSERT OR REPLACE INTO audit_logs (
            seq_id, prev_hash, row_hash, timestamp, actor_id,
            event_type, entity_type, entity_id, payload_digest,
            signature, client_ip
         ) VALUES ('1', 'forged_prev_str', 'forged_row_str', 444444, 'malicious_coerced', 'FORGERY', 'tx', 'tx_001', 'digest', 'sig', '1.2.3.4')",
        [],
    );
    assert!(attack_4.is_err(), "INSERT OR REPLACE with string seq_id must be rejected");
    let err_msg4 = attack_4.unwrap_err().to_string();
    assert!(
        err_msg4.contains("INVARIANT_VIOLATION"),
        "Unexpected error on string seq_id: {err_msg4}"
    );

    // 5. Attack Scenario: INSERT OR IGNORE must NOT overwrite existing records
    let ignore_res = conn.execute(
        "INSERT OR IGNORE INTO audit_logs (
            seq_id, prev_hash, row_hash, timestamp, actor_id,
            event_type, entity_type, entity_id, payload_digest,
            signature, client_ip
         ) VALUES (1, 'ignore_prev', 'ignore_row', 555555, 'ignored_actor', 'IGNORE', 'tx', 'tx_001', 'digest', 'sig', '1.2.3.4')",
        [],
    );
    // In SQLite, INSERT OR IGNORE either ignores or aborts. If it executes with 0 changes:
    if let Ok(affected) = ignore_res {
        assert_eq!(affected, 0, "INSERT OR IGNORE must not modify any rows");
    }

    // 6. Confirm database integrity: seq_id 1 to 5 must remain completely unmodified
    let row1_actor: String = conn.query_row(
        "SELECT actor_id FROM audit_logs WHERE seq_id = 1",
        [],
        |r| r.get(0),
    ).unwrap();
    assert_eq!(row1_actor, records[0].actor_id);

    let row2_actor: String = conn.query_row(
        "SELECT actor_id FROM audit_logs WHERE seq_id = 2",
        [],
        |r| r.get(0),
    ).unwrap();
    assert_eq!(row2_actor, records[1].actor_id);

    let row3_actor: String = conn.query_row(
        "SELECT actor_id FROM audit_logs WHERE seq_id = 3",
        [],
        |r| r.get(0),
    ).unwrap();
    assert_eq!(row3_actor, records[2].actor_id);

    // 7. Verify full chain from DB is still intact
    let mut stmt = conn.prepare(
        "SELECT seq_id, prev_hash, row_hash, timestamp, actor_id, event_type, entity_type, entity_id, payload_digest, signature, client_ip
         FROM audit_logs ORDER BY seq_id ASC"
    ).unwrap();
    let db_records: Vec<AuditDbRecord> = stmt.query_map([], |row| {
        Ok(AuditDbRecord {
            seq_id: row.get::<_, i64>(0)? as u64,
            prev_hash: row.get(1)?,
            row_hash: row.get(2)?,
            timestamp: row.get(3)?,
            actor_id: row.get(4)?,
            event_type: row.get(5)?,
            entity_type: row.get(6)?,
            entity_id: row.get(7)?,
            payload_digest: row.get(8)?,
            signature: row.get(9)?,
            client_ip: row.get(10)?,
        })
    }).unwrap().collect::<Result<Vec<_>, _>>().unwrap();

    let report = verify_audit_db_chain(&db_records);
    assert!(report.is_intact, "Audit chain must remain intact after all attack attempts");
    assert_eq!(report.total_records, 5);
}

// =========================================================================
// Dimension 2: Cryptographic Binding & Exhaustive 1-Byte Tamper Detection
// =========================================================================

#[test]
fn challenge2_test_client_ip_and_signature_cryptographic_binding_exhaustive() {
    let conn = setup_challenger_db();
    let count = 20;
    let records = seed_audit_chain(&conn, count);

    // Baseline: Chain must verify intact
    let clean_report = verify_audit_db_chain(&records);
    assert!(clean_report.is_intact);
    assert_eq!(clean_report.total_records, count);

    // Test 1: For EVERY record in the 20-record chain, tamper client_ip by 1 byte
    for i in 0..count {
        let mut tampered = records.clone();
        let orig_ip = &tampered[i].client_ip;
        let mut ip_bytes = orig_ip.as_bytes().to_vec();
        // Flip bit in last byte
        let last_idx = ip_bytes.len() - 1;
        ip_bytes[last_idx] ^= 0x01;
        tampered[i].client_ip = String::from_utf8_lossy(&ip_bytes).to_string();

        let report = verify_audit_db_chain(&tampered);
        assert!(
            !report.is_intact,
            "1-byte tamper in client_ip at record index {i} (seq_id {}) was not detected!",
            i + 1
        );
        assert_eq!(
            report.tampered_seq_id,
            Some((i + 1) as u64),
            "Expected tampered_seq_id to be Some({}), got {:?}",
            i + 1,
            report.tampered_seq_id
        );
        assert!(
            report.error_message.as_ref().unwrap().contains("Hash verification mismatch"),
            "Expected hash mismatch error message, got: {:?}",
            report.error_message
        );
    }

    // Test 2: For EVERY record in the 20-record chain, tamper signature by 1 byte
    for i in 0..count {
        let mut tampered = records.clone();
        let orig_sig = &tampered[i].signature;
        let mut sig_bytes = orig_sig.as_bytes().to_vec();
        // Flip bit in first byte
        sig_bytes[0] ^= 0x01;
        tampered[i].signature = String::from_utf8_lossy(&sig_bytes).to_string();

        let report = verify_audit_db_chain(&tampered);
        assert!(
            !report.is_intact,
            "1-byte tamper in signature at record index {i} (seq_id {}) was not detected!",
            i + 1
        );
        assert_eq!(
            report.tampered_seq_id,
            Some((i + 1) as u64),
            "Expected tampered_seq_id to be Some({}), got {:?}",
            i + 1,
            report.tampered_seq_id
        );
    }

    // Test 3: Length prefix delimiter collision resistance between client_ip and signature
    // Verify that shifting characters between client_ip and signature yields distinct hashes
    let ip_a = "192.168.1.1";
    let sig_a = "00SECURE_SIG";
    let ip_b = "192.168.1.10";
    let sig_b = "0SECURE_SIG";
    let ip_c = "192.168.1.100";
    let sig_c = "SECURE_SIG";

    let hash_a = compute_audit_row_hash(1, "prev", 100, "act", "EV", "ENT", "id", "dig", ip_a, sig_a);
    let hash_b = compute_audit_row_hash(1, "prev", 100, "act", "EV", "ENT", "id", "dig", ip_b, sig_b);
    let hash_c = compute_audit_row_hash(1, "prev", 100, "act", "EV", "ENT", "id", "dig", ip_c, sig_c);

    assert_ne!(hash_a, hash_b, "Hash collision between shifted IP and signature!");
    assert_ne!(hash_b, hash_c, "Hash collision between shifted IP and signature!");
    assert_ne!(hash_a, hash_c, "Hash collision between shifted IP and signature!");
}

// =========================================================================
// Dimension 3: Segregation of Duties on quarantine_items
// =========================================================================

#[test]
fn challenge2_test_quarantine_items_segregation_of_duties_adversarial() {
    let conn = setup_challenger_db();

    // 1. Seed prerequisite entities
    conn.execute(
        "INSERT INTO legal_entities (id, tax_id, legal_name, created_at, updated_at)
         VALUES ('ent_sod', '0312345678', 'VIETNAM FINANCIAL HARNESS CORP', 1773000000, 1773000000)",
        [],
    ).unwrap();

    conn.execute(
        "INSERT INTO bank_accounts (id, entity_id, bank_code, account_number_enc, account_number_hash, account_name, created_at, updated_at)
         VALUES ('acc_sod', 'ent_sod', 'BIDV', 'enc_sod_acc', 'hash_sod_acc', 'BIDV TREASURY ACC', 1773000000, 1773000000)",
        [],
    ).unwrap();

    conn.execute(
        "INSERT INTO bank_transactions (id, account_id, statement_fingerprint, txn_hash, tx_date, tx_type, amount, is_credit, balance_after, narration, created_at)
         VALUES ('tx_sod_01', 'acc_sod', 'fp_sod_01', 'hash_sod_tx01', 1773000010, 'PAYMENT', 50000000, 0, 950000000, 'Suspicious outflow', 1773000010)",
        [],
    ).unwrap();

    // 2. Attack: Self-approval attempt on INSERT (maker_id == checker_id == 'usr_alice')
    let self_approve_insert = conn.execute(
        "INSERT INTO quarantine_items (
            id, bank_tx_id, account_id, amount, quarantine_reason,
            confidence_score, hitl_token, maker_id, checker_id,
            status, created_at, expires_at
         ) VALUES ('q_self_01', 'tx_sod_01', 'acc_sod', 50000000, 'FRAUD_SUSPICION',
                   0.25, 'tok_self_01', 'usr_alice', 'usr_alice',
                   'APPROVED', 1773000010, 1773086410)",
        [],
    );
    assert!(self_approve_insert.is_err(), "Self-approval on INSERT must fail CHECK constraint");
    let err_sod_insert = self_approve_insert.unwrap_err().to_string();
    assert!(
        err_sod_insert.contains("CHECK constraint failed") || err_sod_insert.contains("chk_maker_checker"),
        "Unexpected error for SoD violation on INSERT: {err_sod_insert}"
    );

    // 3. Legitimate initial creation: maker_id set, checker_id NULL (Pending state)
    let legit_create = conn.execute(
        "INSERT INTO quarantine_items (
            id, bank_tx_id, account_id, amount, quarantine_reason,
            confidence_score, hitl_token, maker_id, checker_id,
            status, created_at, expires_at
         ) VALUES ('q_legit_01', 'tx_sod_01', 'acc_sod', 50000000, 'FRAUD_SUSPICION',
                   0.25, 'tok_legit_01', 'usr_alice', NULL,
                   'PENDING_REVIEW', 1773000010, 1773086410)",
        [],
    );
    assert!(legit_create.is_ok(), "Initial creation with NULL checker must succeed: {:?}", legit_create.err());

    // 4. Attack: Reviewer attempts to resolve and set checker_id = maker_id ('usr_alice')
    let self_approve_update = conn.execute(
        "UPDATE quarantine_items SET checker_id = 'usr_alice', status = 'APPROVED', resolved_at = 1773000500 WHERE id = 'q_legit_01'",
        [],
    );
    assert!(self_approve_update.is_err(), "Self-approval on UPDATE must fail CHECK constraint");
    let err_sod_update = self_approve_update.unwrap_err().to_string();
    assert!(
        err_sod_update.contains("CHECK constraint failed") || err_sod_update.contains("chk_maker_checker"),
        "Unexpected error for SoD violation on UPDATE: {err_sod_update}"
    );

    // 5. Legitimate four-eyes approval: checker_id = 'usr_bob' (maker != checker)
    let legit_approve = conn.execute(
        "UPDATE quarantine_items SET checker_id = 'usr_bob', status = 'APPROVED', resolved_at = 1773000500 WHERE id = 'q_legit_01'",
        [],
    );
    assert!(legit_approve.is_ok(), "Legitimate four-eyes approval must succeed: {:?}", legit_approve.err());

    // 6. Attack: Attempt to subvert resolved record by setting maker_id = 'usr_bob'
    let maker_tamper = conn.execute(
        "UPDATE quarantine_items SET maker_id = 'usr_bob' WHERE id = 'q_legit_01'",
        [],
    );
    assert!(maker_tamper.is_err(), "Mutating maker_id to match checker_id must fail CHECK constraint");
}
