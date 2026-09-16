use serde::{Deserialize, Serialize};
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
    inner_hasher.update(i_key_pad);
    inner_hasher.update(message);
    let inner_hash = inner_hasher.finalize();

    let mut outer_hasher = Sha256::new();
    outer_hasher.update(o_key_pad);
    outer_hasher.update(inner_hash);
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

/// Computes SHA-256 digest of payload bytes.
pub fn digest_payload(payload: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload);
    hex::encode(hasher.finalize())
}

/// Constructs canonical HMAC message byte buffer with length prefixing and domain separators:
/// seq_id.to_le_bytes() || 0x00 || prev_hash || 0x00 || timestamp.to_le_bytes() || 0x00 ||
/// actor_len.to_le_bytes() || actor || 0x00 || event_len.to_le_bytes() || event || 0x00 || payload_digest
pub fn build_hmac_message(
    seq_id: u64,
    prev_hash: &str,
    timestamp: i64,
    actor: &str,
    event: &str,
    payload_digest: &str,
) -> Vec<u8> {
    let mut msg = Vec::with_capacity(
        8 + 1 + prev_hash.len() + 1 + 8 + 1 + 8 + actor.len() + 1 + 8 + event.len() + 1 + payload_digest.len(),
    );
    msg.extend_from_slice(&seq_id.to_le_bytes());
    msg.push(0x00);
    msg.extend_from_slice(prev_hash.as_bytes());
    msg.push(0x00);
    msg.extend_from_slice(&timestamp.to_le_bytes());
    msg.push(0x00);
    msg.extend_from_slice(&(actor.len() as u64).to_le_bytes());
    msg.extend_from_slice(actor.as_bytes());
    msg.push(0x00);
    msg.extend_from_slice(&(event.len() as u64).to_le_bytes());
    msg.extend_from_slice(event.as_bytes());
    msg.push(0x00);
    msg.extend_from_slice(payload_digest.as_bytes());
    msg
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditRecord {
    pub seq_id: u64,
    pub prev_hash: String,
    pub record_hash: String,
    pub timestamp: i64,
    pub event_type: String,
    pub actor_principal: String,
    pub payload_digest: String,
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditVerificationReport {
    pub is_intact: bool,
    pub total_records: usize,
    pub genesis_hash: String,
    pub latest_hash: String,
    pub tampered_seq_id: Option<u64>,
    pub error_message: Option<String>,
}

/// Pure in-memory append-only tamper-evident audit ledger.
#[derive(Debug, Clone)]
pub struct AuditLedger {
    audit_key: Vec<u8>,
    records: Vec<AuditRecord>,
    latest_hash: String,
}

impl AuditLedger {
    pub fn new(audit_key: &[u8]) -> Self {
        let gen = genesis_hash();
        Self {
            audit_key: audit_key.to_vec(),
            records: Vec::new(),
            latest_hash: gen,
        }
    }

    pub fn append(&mut self, actor: &str, event: &str, payload: &str) -> &AuditRecord {
        let now_ts = chrono::Utc::now().timestamp();
        self.append_with_timestamp(now_ts, actor, event, payload)
    }

    pub fn append_with_timestamp(
        &mut self,
        timestamp: i64,
        actor: &str,
        event: &str,
        payload: &str,
    ) -> &AuditRecord {
        let seq_id = (self.records.len() + 1) as u64;
        let prev_hash = self.latest_hash.clone();
        let payload_digest = digest_payload(payload.as_bytes());

        let msg = build_hmac_message(seq_id, &prev_hash, timestamp, actor, event, &payload_digest);
        let mac_bytes = compute_hmac_sha256(&self.audit_key, &msg);
        let record_hash = hex::encode(mac_bytes);
        let signature = record_hash.clone();

        self.latest_hash = record_hash.clone();

        let record = AuditRecord {
            seq_id,
            prev_hash,
            record_hash,
            timestamp,
            event_type: event.to_string(),
            actor_principal: actor.to_string(),
            payload_digest,
            signature,
        };

        self.records.push(record);
        self.records.last().unwrap()
    }

    pub fn verify(&self) -> AuditVerificationReport {
        Self::verify_records(&self.audit_key, &self.records)
    }

    pub fn verify_records(audit_key: &[u8], records: &[AuditRecord]) -> AuditVerificationReport {
        let gen = genesis_hash();
        let mut expected_prev = gen.clone();
        let mut latest = gen.clone();

        for (idx, rec) in records.iter().enumerate() {
            let expected_seq = (idx + 1) as u64;

            // Invariant 0: seq_id must follow strict monotonic progression (1, 2, 3, ...)
            if rec.seq_id != expected_seq {
                return AuditVerificationReport {
                    is_intact: false,
                    total_records: idx + 1,
                    genesis_hash: gen,
                    latest_hash: rec.record_hash.clone(),
                    tampered_seq_id: Some(rec.seq_id),
                    error_message: Some(format!(
                        "Monotonic sequence violation at index {idx}: expected seq_id {expected_seq}, got {}",
                        rec.seq_id
                    )),
                };
            }

            // Invariant 1: prev_hash must chain exactly to previous block
            if rec.prev_hash != expected_prev {
                return AuditVerificationReport {
                    is_intact: false,
                    total_records: idx + 1,
                    genesis_hash: gen,
                    latest_hash: rec.record_hash.clone(),
                    tampered_seq_id: Some(rec.seq_id),
                    error_message: Some(format!(
                        "Chain broken at seq_id {}: expected prev_hash {expected_prev}, got {}",
                        rec.seq_id, rec.prev_hash
                    )),
                };
            }

            // Invariant 2: record_hash must evaluate correctly under HMAC-SHA256
            let msg = build_hmac_message(
                rec.seq_id,
                &rec.prev_hash,
                rec.timestamp,
                &rec.actor_principal,
                &rec.event_type,
                &rec.payload_digest,
            );
            let calculated_hash = hex::encode(compute_hmac_sha256(audit_key, &msg));

            if calculated_hash != rec.record_hash {
                return AuditVerificationReport {
                    is_intact: false,
                    total_records: idx + 1,
                    genesis_hash: gen,
                    latest_hash: rec.record_hash.clone(),
                    tampered_seq_id: Some(rec.seq_id),
                    error_message: Some(format!(
                        "HMAC verification mismatch at seq_id {}: record content was tampered!",
                        rec.seq_id
                    )),
                };
            }

            // Invariant 3: signature must match record_hash
            if rec.signature != calculated_hash || rec.signature != rec.record_hash {
                return AuditVerificationReport {
                    is_intact: false,
                    total_records: idx + 1,
                    genesis_hash: gen,
                    latest_hash: rec.record_hash.clone(),
                    tampered_seq_id: Some(rec.seq_id),
                    error_message: Some(format!(
                        "Signature verification failed at seq_id {}",
                        rec.seq_id
                    )),
                };
            }

            expected_prev = rec.record_hash.clone();
            latest = rec.record_hash.clone();
        }

        AuditVerificationReport {
            is_intact: true,
            total_records: records.len(),
            genesis_hash: gen,
            latest_hash: latest,
            tampered_seq_id: None,
            error_message: None,
        }
    }

    pub fn records(&self) -> &[AuditRecord] {
        &self.records
    }

    pub fn latest_hash(&self) -> &str {
        &self.latest_hash
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

/// Computes canonical SHA-256 row hash for database audit_logs table.
/// Encodes field lengths and byte values to prevent delimiter collisions.
#[allow(clippy::too_many_arguments)]
pub fn compute_audit_row_hash(
    seq_id: u64,
    prev_hash: &str,
    timestamp: i64,
    actor_id: &str,
    event_type: &str,
    entity_type: &str,
    entity_id: &str,
    payload_digest: &str,
    client_ip: &str,
    signature: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(seq_id.to_le_bytes());
    hasher.update(b":");
    hasher.update(prev_hash.as_bytes());
    hasher.update(b":");
    hasher.update(timestamp.to_le_bytes());
    hasher.update(b":");
    hasher.update((actor_id.len() as u32).to_le_bytes());
    hasher.update(actor_id.as_bytes());
    hasher.update(b":");
    hasher.update((event_type.len() as u32).to_le_bytes());
    hasher.update(event_type.as_bytes());
    hasher.update(b":");
    hasher.update((entity_type.len() as u32).to_le_bytes());
    hasher.update(entity_type.as_bytes());
    hasher.update(b":");
    hasher.update((entity_id.len() as u32).to_le_bytes());
    hasher.update(entity_id.as_bytes());
    hasher.update(b":");
    hasher.update(payload_digest.as_bytes());
    hasher.update(b":");
    hasher.update((client_ip.len() as u32).to_le_bytes());
    hasher.update(client_ip.as_bytes());
    hasher.update(b":");
    hasher.update((signature.len() as u32).to_le_bytes());
    hasher.update(signature.as_bytes());
    hex::encode(hasher.finalize())
}

/// Represents an immutable database audit log row as stored in the `audit_logs` table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditDbRecord {
    pub seq_id: u64,
    pub prev_hash: String,
    pub row_hash: String,
    pub timestamp: i64,
    pub actor_id: String,
    pub event_type: String,
    pub entity_type: String,
    pub entity_id: String,
    pub payload_digest: String,
    pub signature: String,
    pub client_ip: String,
}

/// Verifies a sequential chain of database audit log records against cryptographic invariants.
///
/// Invariants verified:
/// - Strict monotonic sequence progression (seq_id = 1, 2, 3...)
/// - Genesis anchoring on first record (`prev_hash == genesis_hash()`)
/// - Continuous predecessor hash linkage (`rec[i].prev_hash == rec[i-1].row_hash`)
/// - Canonical SHA-256 row hash integrity (`compute_audit_row_hash(...) == rec.row_hash`)
pub fn verify_audit_db_chain(records: &[AuditDbRecord]) -> AuditVerificationReport {
    let gen = genesis_hash();
    if records.is_empty() {
        return AuditVerificationReport {
            is_intact: true,
            total_records: 0,
            genesis_hash: gen.clone(),
            latest_hash: gen,
            tampered_seq_id: None,
            error_message: None,
        };
    }

    let mut expected_prev = gen.clone();
    let mut latest = gen.clone();

    for (idx, rec) in records.iter().enumerate() {
        let expected_seq = (idx + 1) as u64;

        // Invariant 0: seq_id must follow strict monotonic progression (1, 2, 3, ...)
        if rec.seq_id != expected_seq {
            return AuditVerificationReport {
                is_intact: false,
                total_records: idx + 1,
                genesis_hash: gen,
                latest_hash: rec.row_hash.clone(),
                tampered_seq_id: Some(rec.seq_id),
                error_message: Some(format!(
                    "Monotonic sequence violation at index {idx}: expected seq_id {expected_seq}, got {}",
                    rec.seq_id
                )),
            };
        }

        // Invariant 1: prev_hash must chain exactly to previous block
        if rec.prev_hash != expected_prev {
            return AuditVerificationReport {
                is_intact: false,
                total_records: idx + 1,
                genesis_hash: gen,
                latest_hash: rec.row_hash.clone(),
                tampered_seq_id: Some(rec.seq_id),
                error_message: Some(format!(
                    "Chain broken at seq_id {}: expected prev_hash {expected_prev}, got {}",
                    rec.seq_id, rec.prev_hash
                )),
            };
        }

        // Invariant 2: row_hash must evaluate correctly under canonical SHA-256
        let calculated_hash = compute_audit_row_hash(
            rec.seq_id,
            &rec.prev_hash,
            rec.timestamp,
            &rec.actor_id,
            &rec.event_type,
            &rec.entity_type,
            &rec.entity_id,
            &rec.payload_digest,
            &rec.client_ip,
            &rec.signature,
        );

        if calculated_hash != rec.row_hash {
            return AuditVerificationReport {
                is_intact: false,
                total_records: idx + 1,
                genesis_hash: gen,
                latest_hash: rec.row_hash.clone(),
                tampered_seq_id: Some(rec.seq_id),
                error_message: Some(format!(
                    "Hash verification mismatch at seq_id {}: record content was tampered!",
                    rec.seq_id
                )),
            };
        }

        expected_prev = rec.row_hash.clone();
        latest = rec.row_hash.clone();
    }

    AuditVerificationReport {
        is_intact: true,
        total_records: records.len(),
        genesis_hash: gen,
        latest_hash: latest,
        tampered_seq_id: None,
        error_message: None,
    }
}
