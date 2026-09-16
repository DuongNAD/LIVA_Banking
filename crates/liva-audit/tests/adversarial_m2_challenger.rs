//! Adversarial Stress Test Suite for Milestone M2
//! Challenger 1 empirical challenge against:
//! 1. Append-only database triggers on `audit_logs` (UPDATE, DELETE, batch mutation, ON CONFLICT DO UPDATE, REPLACE bypass)
//! 2. Segregation of Duties `CHECK (maker_id != checker_id)` on `quarantine_items`
//! 3. Cryptographic SHA-256 Hash Chains under exhaustive 1-byte tamper, UTF-8 Vietnamese payloads, and field omission analysis

use liva_audit::{
    compute_audit_row_hash, digest_payload, genesis_hash, verify_audit_db_chain, AuditDbRecord,
};
use rusqlite::{params, Connection};

fn setup_adversarial_db() -> Connection {
    let conn = Connection::open_in_memory().expect("Failed to create in-memory SQLite database");
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .expect("Failed to enable foreign keys");

    let schema_sql = include_str!("../../../migrations/20260916000001_init_banking_schema.sql");
    conn.execute_batch(schema_sql)
        .expect("Failed to apply schema migration");

    let triggers_sql = include_str!("../../../migrations/20260916000002_audit_log_triggers.sql");
    conn.execute_batch(triggers_sql)
        .expect("Failed to apply triggers migration");

    conn
}

fn seed_audit_records(conn: &Connection, count: usize) -> Vec<AuditDbRecord> {
    let mut records = Vec::with_capacity(count);
    let mut prev = genesis_hash();
    let base_ts = 1773000000i64;

    for i in 1..=count {
        let seq = i as u64;
        let ts = base_ts + (i as i64 * 10);
        let actor = format!("actor_{i}");
        let event = "SYSTEM_EVENT";
        let entity_type = "bank_account";
        let entity_id = format!("acc_{i}");
        let payload = format!(r#"{{"index":{i},"status":"OK"}}"#);
        let payload_digest = digest_payload(payload.as_bytes());
        let sig = format!("sig_{i}");
        let client_ip = "127.0.0.1";

        let row_h = compute_audit_row_hash(
            seq,
            &prev,
            ts,
            &actor,
            event,
            entity_type,
            &entity_id,
            &payload_digest,
            client_ip,
            &sig,
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
                sig, client_ip
            ],
        ).unwrap();

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
            signature: sig,
            client_ip: client_ip.to_string(),
        });

        prev = row_h;
    }

    records
}

// =========================================================================
// Dimension 1: Adversarial Challenge on DB Triggers (Append-Only Invariant)
// =========================================================================

#[test]
fn adversarial_test_audit_logs_batch_update_aborted() {
    let conn = setup_adversarial_db();
    seed_audit_records(&conn, 10);

    // Attack 1: Unconditional bulk UPDATE
    let res = conn.execute("UPDATE audit_logs SET actor_id = 'adversary'", []);
    assert!(res.is_err(), "Unconditional bulk UPDATE must be blocked by trigger");
    let err_str = res.unwrap_err().to_string();
    assert!(err_str.contains("INVARIANT_VIOLATION"), "Expected INVARIANT_VIOLATION, got {err_str}");

    // Attack 2: Scoped multi-row batch UPDATE
    let res2 = conn.execute("UPDATE audit_logs SET actor_id = 'adversary' WHERE seq_id >= 5", []);
    assert!(res2.is_err(), "Scoped batch UPDATE must be blocked by trigger");

    // Attack 3: Individual column targeted UPDATE (attempting to alter timestamp or prev_hash)
    let res3 = conn.execute("UPDATE audit_logs SET timestamp = 0 WHERE seq_id = 1", []);
    assert!(res3.is_err(), "Single column timestamp UPDATE must be blocked");

    let res4 = conn.execute("UPDATE audit_logs SET prev_hash = '0000' WHERE seq_id = 1", []);
    assert!(res4.is_err(), "Single column prev_hash UPDATE must be blocked");

    // Verify all 10 records remain completely intact and unaltered
    let count: i64 = conn.query_row("SELECT count(*) FROM audit_logs WHERE actor_id = 'adversary'", [], |r| r.get(0)).unwrap();
    assert_eq!(count, 0, "No records should have been updated by adversary");
}

#[test]
fn adversarial_test_audit_logs_delete_variants_aborted() {
    let conn = setup_adversarial_db();
    seed_audit_records(&conn, 5);

    // Attack 1: Unconditional bulk DELETE (truncate emulation)
    let res = conn.execute("DELETE FROM audit_logs", []);
    assert!(res.is_err(), "Bulk DELETE must be blocked by trigger");
    let err_str = res.unwrap_err().to_string();
    assert!(err_str.contains("INVARIANT_VIOLATION"), "Expected INVARIANT_VIOLATION on DELETE, got {err_str}");

    // Attack 2: Targeted DELETE with WHERE clause
    let res2 = conn.execute("DELETE FROM audit_logs WHERE seq_id = 3", []);
    assert!(res2.is_err(), "Targeted DELETE must be blocked by trigger");

    // Attack 3: Complex subquery DELETE
    let res3 = conn.execute("DELETE FROM audit_logs WHERE seq_id IN (SELECT seq_id FROM audit_logs WHERE seq_id <= 2)", []);
    assert!(res3.is_err(), "Subquery DELETE must be blocked by trigger");

    // Verify record count is still exactly 5
    let remaining: i64 = conn.query_row("SELECT count(*) FROM audit_logs", [], |r| r.get(0)).unwrap();
    assert_eq!(remaining, 5, "Audit log count must remain intact");
}

#[test]
fn adversarial_test_audit_logs_on_conflict_do_update_blocked() {
    let conn = setup_adversarial_db();
    seed_audit_records(&conn, 3);

    // Attack: INSERT ... ON CONFLICT(seq_id) DO UPDATE
    // The DO UPDATE clause triggers BEFORE UPDATE, which is aborted by trg_audit_logs_no_update.
    let upsert_res = conn.execute(
        "INSERT INTO audit_logs (
            seq_id, prev_hash, row_hash, timestamp, actor_id,
            event_type, entity_type, entity_id, payload_digest,
            signature, client_ip
         ) VALUES (1, 'prev', 'row', 1000, 'impostor', 'EV', 'ET', 'EID', 'PL', 'SIG', '127.0.0.1')
         ON CONFLICT(seq_id) DO UPDATE SET actor_id = 'impostor'",
        [],
    );
    assert!(upsert_res.is_err(), "ON CONFLICT DO UPDATE must be blocked by BEFORE UPDATE trigger");
    let upsert_err = upsert_res.unwrap_err().to_string();
    assert!(
        upsert_err.contains("INVARIANT_VIOLATION"),
        "ON CONFLICT DO UPDATE did not trigger invariant violation: {upsert_err}"
    );

    // Confirm original seq_id = 1 was NOT altered
    let original_actor: String = conn.query_row(
        "SELECT actor_id FROM audit_logs WHERE seq_id = 1",
        [],
        |r| r.get(0),
    ).unwrap();
    assert_eq!(original_actor, "actor_1", "seq_id = 1 actor_id must remain 'actor_1'");
}

#[test]
fn adversarial_test_audit_logs_demonstrate_replace_bypass_vulnerability() {
    let conn = setup_adversarial_db();
    seed_audit_records(&conn, 3);

    // Attack 1: INSERT OR REPLACE collision on seq_id = 1
    // The BEFORE INSERT trigger trg_audit_logs_no_replace intercepts this and aborts before SQLite can replace.
    let replace_res = conn.execute(
        "INSERT OR REPLACE INTO audit_logs (
            seq_id, prev_hash, row_hash, timestamp, actor_id,
            event_type, entity_type, entity_id, payload_digest,
            signature, client_ip
         ) VALUES (1, 'forged_prev', 'forged_row', 9999, 'impostor_replaced', 'EV', 'ET', 'EID', 'PL', 'SIG', '127.0.0.1')",
        [],
    );

    assert!(
        replace_res.is_err(),
        "INSERT OR REPLACE must be aborted by trg_audit_logs_no_replace trigger!"
    );
    let err_str = replace_res.unwrap_err().to_string();
    assert!(
        err_str.contains("INVARIANT_VIOLATION"),
        "Expected INVARIANT_VIOLATION, got: {err_str}"
    );

    // Verify record 1 remains completely intact and uncorrupted
    let preserved_actor: String = conn.query_row(
        "SELECT actor_id FROM audit_logs WHERE seq_id = 1",
        [],
        |r| r.get(0),
    ).unwrap();
    assert_eq!(
        preserved_actor, "actor_1",
        "Record 1 was altered despite trigger protection!"
    );

    // Attack 2: REPLACE INTO syntax directly
    let replace_into_res = conn.execute(
        "REPLACE INTO audit_logs (
            seq_id, prev_hash, row_hash, timestamp, actor_id,
            event_type, entity_type, entity_id, payload_digest,
            signature, client_ip
         ) VALUES (2, 'forged_prev', 'forged_row', 9999, 'impostor_replaced_2', 'EV', 'ET', 'EID', 'PL', 'SIG', '127.0.0.1')",
        [],
    );
    assert!(
        replace_into_res.is_err(),
        "REPLACE INTO must be aborted by trg_audit_logs_no_replace trigger!"
    );
    let replace_into_err = replace_into_res.unwrap_err().to_string();
    assert!(
        replace_into_err.contains("INVARIANT_VIOLATION"),
        "Expected INVARIANT_VIOLATION on REPLACE INTO, got: {replace_into_err}"
    );

    // Attack 3: INSERT OR REPLACE with row_hash collision (omitting seq_id)
    let existing_hash: String = conn.query_row(
        "SELECT row_hash FROM audit_logs WHERE seq_id = 1",
        [],
        |r| r.get(0),
    ).unwrap();
    let collision_res = conn.execute(
        "INSERT OR REPLACE INTO audit_logs (
            prev_hash, row_hash, timestamp, actor_id,
            event_type, entity_type, entity_id, payload_digest,
            signature, client_ip
         ) VALUES ('new_prev', ?1, 9999, 'impostor_collision', 'EV', 'ET', 'EID', 'PL', 'SIG', '127.0.0.1')",
        params![existing_hash],
    );
    assert!(
        collision_res.is_err(),
        "INSERT OR REPLACE with colliding row_hash must be aborted by trigger!"
    );
    let collision_err = collision_res.unwrap_err().to_string();
    assert!(
        collision_err.contains("INVARIANT_VIOLATION"),
        "Expected INVARIANT_VIOLATION on row_hash collision, got: {collision_err}"
    );
}

#[test]
fn adversarial_test_audit_logs_transaction_atomicity_on_trigger_abort() {
    let mut conn = setup_adversarial_db();
    seed_audit_records(&conn, 2);

    let tx = conn.transaction().unwrap();

    // 1. Insert a legal entity inside transaction
    let ent_res = tx.execute(
        "INSERT INTO legal_entities (id, tax_id, legal_name, created_at, updated_at)
         VALUES ('ent_atomic', '0999999999', 'ATOMIC TEST CORP', 1000, 1000)",
        [],
    );
    assert!(ent_res.is_ok());

    // 2. Attempt illegal UPDATE on audit_logs -> fails trigger
    let bad_res = tx.execute("UPDATE audit_logs SET actor_id = 'malicious' WHERE seq_id = 1", []);
    assert!(bad_res.is_err());

    // 3. Rollback
    let rollback_res = tx.rollback();
    assert!(rollback_res.is_ok());

    // 4. Verify that legal_entities row was NOT persisted
    let ent_count: i64 = conn.query_row(
        "SELECT count(*) FROM legal_entities WHERE id = 'ent_atomic'",
        [],
        |r| r.get(0),
    ).unwrap();
    assert_eq!(ent_count, 0, "Aborted transaction must roll back completely");
}

// =========================================================================
// Dimension 2: Adversarial Challenge on Maker-Checker Segregation
// =========================================================================

#[test]
fn adversarial_test_maker_checker_comprehensive_matrix() {
    let conn = setup_adversarial_db();

    // Seed prerequisites
    conn.execute(
        "INSERT INTO legal_entities (id, tax_id, legal_name, created_at, updated_at)
         VALUES ('ent_mc', '0112233445', 'MC TESTING ENTITY', 1000, 1000)",
        [],
    ).unwrap();
    conn.execute(
        "INSERT INTO bank_accounts (id, entity_id, bank_code, account_number_enc, account_number_hash, account_name, created_at, updated_at)
         VALUES ('acc_mc', 'ent_mc', 'VCB', 'enc_mc', 'hash_mc', 'MC ACC', 1000, 1000)",
        [],
    ).unwrap();
    conn.execute(
        "INSERT INTO bank_transactions (id, account_id, statement_fingerprint, txn_hash, tx_date, tx_type, amount, is_credit, balance_after, narration, created_at)
         VALUES ('tx_mc', 'acc_mc', 'fp_mc', 'tx_mc_hash', 1000, 'PAYMENT', 1000000, 0, 99000000, 'MC test tx', 1000)",
        [],
    ).unwrap();

    let insert_quarantine = |id: &str, maker: &str, checker: Option<&str>| {
        conn.execute(
            "INSERT INTO quarantine_items (
                id, bank_tx_id, account_id, amount, quarantine_reason,
                hitl_token, maker_id, checker_id, expires_at, created_at
             ) VALUES (?1, 'tx_mc', 'acc_mc', 1000000, 'TEST', ?2, ?3, ?4, 2000, 1000)",
            params![id, format!("token_{id}"), maker, checker],
        )
    };

    // 1. Both identical non-empty string -> MUST FAIL
    let r1 = insert_quarantine("q_same_usr", "user_alice", Some("user_alice"));
    assert!(r1.is_err(), "Same maker and checker must fail CHECK");

    // 2. Both empty string -> MUST FAIL ('"" != ""' is FALSE)
    let r2 = insert_quarantine("q_empty_both", "", Some(""));
    assert!(r2.is_err(), "Both empty string must fail CHECK");

    // 3. Maker set, checker NULL (Pending state) -> MUST SUCCEED (NULL evaluates to UNKNOWN, passes check)
    let r3 = insert_quarantine("q_pending", "user_alice", None);
    assert!(r3.is_ok(), "NULL checker must succeed for initial review state");

    // 4. Maker and Checker distinct -> MUST SUCCEED
    let r4 = insert_quarantine("q_distinct", "user_alice", Some("user_bob"));
    assert!(r4.is_ok(), "Distinct maker and checker must succeed");

    // 5. Update q_pending setting checker == maker -> MUST FAIL
    let r5 = conn.execute(
        "UPDATE quarantine_items SET checker_id = 'user_alice' WHERE id = 'q_pending'",
        [],
    );
    assert!(r5.is_err(), "Updating checker to equal maker must fail CHECK");

    // 6. Update q_distinct setting maker == checker -> MUST FAIL
    let r6 = conn.execute(
        "UPDATE quarantine_items SET maker_id = 'user_bob' WHERE id = 'q_distinct'",
        [],
    );
    assert!(r6.is_err(), "Updating maker to equal checker must fail CHECK");

    // 7. Update q_pending to distinct checker -> MUST SUCCEED
    let r7 = conn.execute(
        "UPDATE quarantine_items SET checker_id = 'user_carol', status = 'APPROVED' WHERE id = 'q_pending'",
        [],
    );
    assert!(r7.is_ok(), "Updating checker to distinct reviewer must succeed");
}

// =========================================================================
// Dimension 3: Adversarial Challenge on SHA-256 Continuous Hash Chains
// =========================================================================

#[test]
fn adversarial_test_hash_chain_empty_and_genesis() {
    let empty_report = verify_audit_db_chain(&[]);
    assert!(empty_report.is_intact);
    assert_eq!(empty_report.total_records, 0);
    assert_eq!(empty_report.genesis_hash, genesis_hash());
    assert_eq!(empty_report.latest_hash, genesis_hash());

    let bad_genesis_rec = AuditDbRecord {
        seq_id: 1,
        prev_hash: "bogus_genesis_hash".to_string(),
        row_hash: "any_hash".to_string(),
        timestamp: 1000,
        actor_id: "alice".to_string(),
        event_type: "EVENT".to_string(),
        entity_type: "ENTITY".to_string(),
        entity_id: "ID".to_string(),
        payload_digest: "digest".to_string(),
        signature: "sig".to_string(),
        client_ip: "127.0.0.1".to_string(),
    };
    let gen_report = verify_audit_db_chain(&[bad_genesis_rec]);
    assert!(!gen_report.is_intact, "Corrupted genesis must fail verification");
    assert_eq!(gen_report.tampered_seq_id, Some(1));
}

#[test]
fn adversarial_test_hash_chain_exhaustive_field_tampering() {
    let conn = setup_adversarial_db();
    let records = seed_audit_records(&conn, 10);

    let clean_report = verify_audit_db_chain(&records);
    assert!(clean_report.is_intact);
    assert_eq!(clean_report.total_records, 10);

    let target_idx = 4; // seq_id = 5

    // 1. Tamper seq_id
    {
        let mut tampered = records.clone();
        tampered[target_idx].seq_id = 99;
        let rep = verify_audit_db_chain(&tampered);
        assert!(!rep.is_intact);
        assert_eq!(rep.tampered_seq_id, Some(99));
        assert!(rep.error_message.unwrap().contains("Monotonic sequence violation"));
    }

    // 2. Tamper prev_hash
    {
        let mut tampered = records.clone();
        let mut h = tampered[target_idx].prev_hash.clone();
        let first_char = if h.starts_with('0') { '1' } else { '0' };
        h.replace_range(0..1, &first_char.to_string());
        tampered[target_idx].prev_hash = h;
        let rep = verify_audit_db_chain(&tampered);
        assert!(!rep.is_intact);
        assert_eq!(rep.tampered_seq_id, Some(5));
        assert!(rep.error_message.unwrap().contains("Chain broken at seq_id 5"));
    }

    // 3. Tamper timestamp
    {
        let mut tampered = records.clone();
        tampered[target_idx].timestamp += 1;
        let rep = verify_audit_db_chain(&tampered);
        assert!(!rep.is_intact);
        assert_eq!(rep.tampered_seq_id, Some(5));
        assert!(rep.error_message.unwrap().contains("Hash verification mismatch at seq_id 5"));
    }

    // 4. Tamper actor_id
    {
        let mut tampered = records.clone();
        tampered[target_idx].actor_id.push('x');
        let rep = verify_audit_db_chain(&tampered);
        assert!(!rep.is_intact);
        assert_eq!(rep.tampered_seq_id, Some(5));
    }

    // 5. Tamper event_type
    {
        let mut tampered = records.clone();
        tampered[target_idx].event_type = "UNAUTHORIZED_TRANSFER".to_string();
        let rep = verify_audit_db_chain(&tampered);
        assert!(!rep.is_intact);
        assert_eq!(rep.tampered_seq_id, Some(5));
    }

    // 6. Tamper entity_type
    {
        let mut tampered = records.clone();
        tampered[target_idx].entity_type = "credit_card".to_string();
        let rep = verify_audit_db_chain(&tampered);
        assert!(!rep.is_intact);
        assert_eq!(rep.tampered_seq_id, Some(5));
    }

    // 7. Tamper entity_id
    {
        let mut tampered = records.clone();
        tampered[target_idx].entity_id = "acc_9999".to_string();
        let rep = verify_audit_db_chain(&tampered);
        assert!(!rep.is_intact);
        assert_eq!(rep.tampered_seq_id, Some(5));
    }

    // 8. Tamper payload_digest
    {
        let mut tampered = records.clone();
        let mut bytes = tampered[target_idx].payload_digest.as_bytes().to_vec();
        bytes[0] ^= 0x01;
        tampered[target_idx].payload_digest = String::from_utf8(bytes).unwrap();
        let rep = verify_audit_db_chain(&tampered);
        assert!(!rep.is_intact);
        assert_eq!(rep.tampered_seq_id, Some(5));
    }

    // 9. Tamper row_hash
    {
        let mut tampered = records.clone();
        tampered[target_idx].row_hash = "deadbeef".repeat(8);
        let rep = verify_audit_db_chain(&tampered);
        assert!(!rep.is_intact);
        assert_eq!(rep.tampered_seq_id, Some(5));
    }

    // 10. Tamper client_ip
    {
        let mut tampered = records.clone();
        tampered[target_idx].client_ip = "192.168.1.99".to_string();
        let rep = verify_audit_db_chain(&tampered);
        assert!(!rep.is_intact, "client_ip tampering must be detected!");
        assert_eq!(rep.tampered_seq_id, Some(5));
        assert!(rep.error_message.unwrap().contains("Hash verification mismatch at seq_id 5"));
    }

    // 11. Tamper signature
    {
        let mut tampered = records.clone();
        tampered[target_idx].signature = "FORGED_SIGNATURE".to_string();
        let rep = verify_audit_db_chain(&tampered);
        assert!(!rep.is_intact, "signature tampering must be detected!");
        assert_eq!(rep.tampered_seq_id, Some(5));
        assert!(rep.error_message.unwrap().contains("Hash verification mismatch at seq_id 5"));
    }
}

#[test]
fn adversarial_test_hash_chain_permutations_and_truncations() {
    let conn = setup_adversarial_db();
    let records = seed_audit_records(&conn, 6);

    // 1. Out of order swap
    {
        let mut swapped = records.clone();
        swapped.swap(1, 2);
        let rep = verify_audit_db_chain(&swapped);
        assert!(!rep.is_intact, "Swapped records must fail verification");
        assert_eq!(rep.tampered_seq_id, Some(3));
    }

    // 2. Duplicate record injection
    {
        let mut dups = records.clone();
        dups.insert(2, records[1].clone());
        let rep = verify_audit_db_chain(&dups);
        assert!(!rep.is_intact, "Duplicate record injection must fail verification");
        assert_eq!(rep.tampered_seq_id, Some(2));
    }

    // 3. Chain truncation / Missing tail
    {
        let truncated = &records[0..3];
        let rep = verify_audit_db_chain(truncated);
        assert!(rep.is_intact);
        assert_eq!(rep.total_records, 3);
        assert_ne!(
            rep.latest_hash, records[5].row_hash,
            "Truncated chain head hash must not equal original chain head hash"
        );
    }
}

#[test]
fn adversarial_test_delimiter_collision_resistance() {
    let h1 = compute_audit_row_hash(
        1,
        "prev",
        1000,
        "alice:ADMIN",
        "EVENT",
        "TYPE",
        "ID",
        "DIGEST",
        "127.0.0.1",
        "sig",
    );

    let h2 = compute_audit_row_hash(
        1,
        "prev",
        1000,
        "alice",
        "ADMIN:EVENT",
        "TYPE",
        "ID",
        "DIGEST",
        "127.0.0.1",
        "sig",
    );

    assert_ne!(h1, h2, "Length-prefixed fields must prevent delimiter injection collision!");

    let h3 = compute_audit_row_hash(1, "prev", 1000, "", ":", "TYPE", "ID", "DIGEST", "127.0.0.1", "sig");
    let h4 = compute_audit_row_hash(1, "prev", 1000, ":", "", "TYPE", "ID", "DIGEST", "127.0.0.1", "sig");
    assert_ne!(h3, h4, "Empty strings must not collide with colon delimiters");

    let h5 = compute_audit_row_hash(1, "prev", 1000, "alice", "EVENT", "TYPE", "ID", "DIGEST", "127.0.0.1:SIG", "");
    let h6 = compute_audit_row_hash(1, "prev", 1000, "alice", "EVENT", "TYPE", "ID", "DIGEST", "127.0.0.1", "SIG");
    assert_ne!(h5, h6, "Delimiter collision between client_ip and signature must be prevented!");
}

#[test]
fn adversarial_test_vietnamese_unicode_hash_chain() {
    let prev = genesis_hash();
    let actor = "Nguyễn Văn Kiểm Toán";
    let event = "ĐỐI_SOÁT_TIỀN_VÀO";
    let entity_type = "tài_khoản_doanh_nghiệp";
    let entity_id = "TK_VCB_001_HÀ_NỘI";
    let payload = r#"{"nội_dung":"Chuyển khoản thanh toán hợp đồng kinh tế #88/2026","số_tiền":500000000}"#;
    let payload_digest = digest_payload(payload.as_bytes());
    let client_ip = "127.0.0.1";
    let signature = "sig_vn";

    let row_hash = compute_audit_row_hash(
        1,
        &prev,
        1773000000,
        actor,
        event,
        entity_type,
        entity_id,
        &payload_digest,
        client_ip,
        signature,
    );

    let record = AuditDbRecord {
        seq_id: 1,
        prev_hash: prev,
        row_hash,
        timestamp: 1773000000,
        actor_id: actor.to_string(),
        event_type: event.to_string(),
        entity_type: entity_type.to_string(),
        entity_id: entity_id.to_string(),
        payload_digest,
        signature: signature.to_string(),
        client_ip: client_ip.to_string(),
    };

    let report = verify_audit_db_chain(&[record]);
    assert!(report.is_intact, "Vietnamese UTF-8 unicode records must verify correctly");
}

#[test]
fn adversarial_test_client_ip_and_signature_omission_finding() {
    let conn = setup_adversarial_db();
    let records = seed_audit_records(&conn, 3);

    // Remediation verification:
    // AuditDbRecord fields `client_ip` and `signature` are cryptographically bound
    // into `compute_audit_row_hash`. Consequently, `verify_audit_db_chain` detects any
    // tampering with `client_ip` or `signature` with 100% certainty.

    // 1. Clean records verify successfully
    let clean_rep = verify_audit_db_chain(&records);
    assert!(clean_rep.is_intact, "Clean audit chain must verify intact");

    // 2. Tampering client_ip is detected
    let mut tampered_ip = records.clone();
    tampered_ip[1].client_ip = "192.168.1.100".to_string(); // Altered client IP
    let rep_ip = verify_audit_db_chain(&tampered_ip);
    assert!(
        !rep_ip.is_intact,
        "Tampering with client_ip MUST cause verification failure!"
    );
    assert_eq!(rep_ip.tampered_seq_id, Some(2));
    assert!(rep_ip.error_message.unwrap().contains("Hash verification mismatch at seq_id 2"));

    // 3. Tampering signature is detected
    let mut tampered_sig = records.clone();
    tampered_sig[1].signature = "FORGED_SIGNATURE".to_string(); // Forged signature
    let rep_sig = verify_audit_db_chain(&tampered_sig);
    assert!(
        !rep_sig.is_intact,
        "Tampering with signature MUST cause verification failure!"
    );
    assert_eq!(rep_sig.tampered_seq_id, Some(2));
    assert!(rep_sig.error_message.unwrap().contains("Hash verification mismatch at seq_id 2"));
}
