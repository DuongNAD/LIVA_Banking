//! Tamper-evident Audit Ledger with Forward-Chained HMAC-SHA256 Hashes.
//!
//! Enforces non-repudiation and tamper detection for Decree 13 and Circular 09 compliance:
//! H_k = HMAC-SHA256(K_audit, H_{k-1} || timestamp || actor || event_type || payload_digest)

use rusqlite::{Connection, params};
use sha2::{Digest, Sha256};

pub const GENESIS_SEED: &[u8] = b"LIVA_BANKING_GENESIS_2026";

/// Pure RFC 2104 HMAC-SHA256 implementation ensuring exact single-version SHA-256 dependency.
pub fn compute_hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    const BLOCK_SIZE: usize = 64;
    let mut key_block = [0u8; BLOCK_SIZE];
    if key.len() > BLOCK_SIZE {
        let mut hasher = Sha256::new();
        hasher.update(key);
        let digest = hasher.finalize();
        key_block[..32].copy_from_slice(&digest);
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }

    let mut o_key_pad = [0u8; BLOCK_SIZE];
    let mut i_key_pad = [0u8; BLOCK_SIZE];
    for i in 0..BLOCK_SIZE {
        o_key_pad[i] = key_block[i] ^ 0x5c;
        i_key_pad[i] = key_block[i] ^ 0x36;
    }

    let mut inner_hasher = Sha256::new();
    inner_hasher.update(&i_key_pad);
    inner_hasher.update(message);
    let inner_hash = inner_hasher.finalize();

    let mut outer_hasher = Sha256::new();
    outer_hasher.update(&o_key_pad);
    outer_hasher.update(&inner_hash);
    let result = outer_hasher.finalize();

    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

/// Computes the genesis block hash H_0.
pub fn genesis_hash() -> String {
    let mut hasher = Sha256::new();
    hasher.update(GENESIS_SEED);
    hex::encode(hasher.finalize())
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditVerificationReport {
    pub is_intact: bool,
    pub total_records: usize,
    pub genesis_hash: String,
    pub latest_hash: String,
    pub tampered_seq_id: Option<i64>,
    pub error_message: Option<String>,
}

pub struct AuditLedger;

impl AuditLedger {
    /// Appends a new audit record to `banking_audit_chain` using forward hash chaining.
    pub fn append(
        conn: &Connection,
        audit_key: &[u8],
        event_type: &str,
        actor: &str,
        payload: &str,
    ) -> Result<String, String> {
        let now_ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        // 1. Get previous hash H_{k-1}
        let prev_hash: String = conn
            .query_row(
                "SELECT record_hash FROM banking_audit_chain ORDER BY seq_id DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| genesis_hash());

        // 2. Digest payload: SHA-256(payload)
        let mut p_hasher = Sha256::new();
        p_hasher.update(payload.as_bytes());
        let payload_digest = hex::encode(p_hasher.finalize());

        // 3. Compute H_k = HMAC-SHA256(K, prev_hash || ts || actor || event || digest)
        let mut msg = Vec::new();
        msg.extend_from_slice(prev_hash.as_bytes());
        msg.extend_from_slice(&now_ts.to_le_bytes());
        msg.extend_from_slice(actor.as_bytes());
        msg.extend_from_slice(event_type.as_bytes());
        msg.extend_from_slice(payload_digest.as_bytes());

        let mac_bytes = compute_hmac_sha256(audit_key, &msg);
        let record_hash = hex::encode(mac_bytes);
        let signature = record_hash.clone(); // Signature for verification

        // 4. Insert into banking_audit_chain
        conn.execute(
            "INSERT INTO banking_audit_chain (prev_hash, record_hash, timestamp, event_type, actor_principal, payload_digest, signature) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                prev_hash,
                record_hash,
                now_ts,
                event_type,
                actor,
                payload_digest,
                signature
            ],
        )
        .map_err(|e| format!("Failed to insert into banking_audit_chain: {e}"))?;

        Ok(record_hash)
    }

    /// Verifies the complete cryptographic integrity of the forward-chained audit ledger.
    pub fn verify(conn: &Connection, audit_key: &[u8]) -> Result<AuditVerificationReport, String> {
        let mut stmt = conn
            .prepare(
                "SELECT seq_id, prev_hash, record_hash, timestamp, event_type, actor_principal, payload_digest, signature \
                 FROM banking_audit_chain ORDER BY seq_id ASC",
            )
            .map_err(|e| format!("Failed to prepare audit verify query: {e}"))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                ))
            })
            .map_err(|e| format!("Query map failed: {e}"))?;

        let gen_hash = genesis_hash();
        let mut expected_prev_hash = gen_hash.clone();
        let mut latest_hash = gen_hash.clone();
        let mut count = 0;

        for res in rows {
            let (seq_id, prev_hash, record_hash, ts, event_type, actor, digest, signature) =
                res.map_err(|e| format!("Row read failed: {e}"))?;
            count += 1;

            // Invariant 1: prev_hash must strictly match the previous block's record_hash
            if prev_hash != expected_prev_hash {
                return Ok(AuditVerificationReport {
                    is_intact: false,
                    total_records: count,
                    genesis_hash: gen_hash,
                    latest_hash: record_hash,
                    tampered_seq_id: Some(seq_id),
                    error_message: Some(format!(
                        "Chain broken at seq_id {seq_id}: prev_hash mismatch (expected {expected_prev_hash}, got {prev_hash})"
                    )),
                });
            }

            // Invariant 2: record_hash must re-evaluate correctly under HMAC-SHA256
            let mut msg = Vec::new();
            msg.extend_from_slice(prev_hash.as_bytes());
            msg.extend_from_slice(&ts.to_le_bytes());
            msg.extend_from_slice(actor.as_bytes());
            msg.extend_from_slice(event_type.as_bytes());
            msg.extend_from_slice(digest.as_bytes());

            let calculated_hash = hex::encode(compute_hmac_sha256(audit_key, &msg));
            if calculated_hash != record_hash {
                return Ok(AuditVerificationReport {
                    is_intact: false,
                    total_records: count,
                    genesis_hash: gen_hash,
                    latest_hash: record_hash,
                    tampered_seq_id: Some(seq_id),
                    error_message: Some(format!(
                        "Hash recalculation mismatch at seq_id {seq_id}: block data was altered!"
                    )),
                });
            }

            // Invariant 3: signature must match both the computed HMAC-SHA256 hash and record_hash
            if signature != calculated_hash || signature != record_hash {
                return Ok(AuditVerificationReport {
                    is_intact: false,
                    total_records: count,
                    genesis_hash: gen_hash,
                    latest_hash: record_hash,
                    tampered_seq_id: Some(seq_id),
                    error_message: Some(format!(
                        "Signature verification failed at seq_id {seq_id}: stored signature does not match computed HMAC hash!"
                    )),
                });
            }

            expected_prev_hash = record_hash.clone();
            latest_hash = record_hash;
        }

        Ok(AuditVerificationReport {
            is_intact: true,
            total_records: count,
            genesis_hash: gen_hash,
            latest_hash,
            tampered_seq_id: None,
            error_message: None,
        })
    }

    /// Initializes the `banking_audit_chain` table and index if not already present.
    pub fn init_audit_table(conn: &Connection) -> Result<(), rusqlite::Error> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS banking_audit_chain (
                seq_id INTEGER PRIMARY KEY AUTOINCREMENT,
                prev_hash TEXT NOT NULL,
                record_hash TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                event_type TEXT NOT NULL,
                actor_principal TEXT NOT NULL,
                payload_digest TEXT NOT NULL,
                signature TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_banking_audit_seq ON banking_audit_chain(seq_id);",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE banking_audit_chain (
                seq_id INTEGER PRIMARY KEY AUTOINCREMENT,
                prev_hash TEXT NOT NULL,
                record_hash TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                event_type TEXT NOT NULL,
                actor_principal TEXT NOT NULL,
                payload_digest TEXT NOT NULL,
                signature TEXT NOT NULL
            )",
            [],
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_audit_ledger_chain_and_tamper_detection() {
        let conn = setup_test_db();
        let key = b"test_audit_hmac_secret_key_32bytes!";

        // 1. Append 3 records
        let h1 = AuditLedger::append(
            &conn,
            key,
            "STATEMENT_IMPORTED",
            "TauriDashboard",
            "vcb.xlsx",
        )
        .unwrap();
        let h2 = AuditLedger::append(
            &conn,
            key,
            "MATCH_AUTO",
            "ReconciliationEngine",
            "match_tx_1",
        )
        .unwrap();
        let h3 = AuditLedger::append(
            &conn,
            key,
            "HITL_CONFIRMED",
            "ChiefAccountant",
            "confirm_split",
        )
        .unwrap();

        assert_ne!(h1, h2);
        assert_ne!(h2, h3);

        // 2. Verify intact chain
        let report = AuditLedger::verify(&conn, key).unwrap();
        assert!(report.is_intact);
        assert_eq!(report.total_records, 3);
        assert_eq!(report.latest_hash, h3);
        assert!(report.tampered_seq_id.is_none());

        // 3. Simulate unauthorized database alteration on record 2
        conn.execute(
            "UPDATE banking_audit_chain SET payload_digest = 'tampered_digest' WHERE seq_id = 2",
            [],
        )
        .unwrap();

        // 4. Verify detects tampering
        let tampered_report = AuditLedger::verify(&conn, key).unwrap();
        assert!(!tampered_report.is_intact);
        assert_eq!(tampered_report.tampered_seq_id, Some(2));
    }

    #[test]
    fn test_audit_ledger_signature_tamper_detection() {
        let conn = setup_test_db();
        let key = b"test_audit_hmac_secret_key_32bytes!";

        let _h1 = AuditLedger::append(
            &conn,
            key,
            "STATEMENT_IMPORTED",
            "TauriDashboard",
            "vcb.xlsx",
        )
        .unwrap();
        let _h2 = AuditLedger::append(
            &conn,
            key,
            "MATCH_AUTO",
            "ReconciliationEngine",
            "match_tx_1",
        )
        .unwrap();

        // Intact chain passes
        let report = AuditLedger::verify(&conn, key).unwrap();
        assert!(report.is_intact);

        // Tamper signature column specifically
        conn.execute(
            "UPDATE banking_audit_chain SET signature = 'corrupted_signature_12345' WHERE seq_id = 2",
            [],
        )
        .unwrap();

        let tampered_report = AuditLedger::verify(&conn, key).unwrap();
        assert!(!tampered_report.is_intact);
        assert_eq!(tampered_report.tampered_seq_id, Some(2));
    }
}
