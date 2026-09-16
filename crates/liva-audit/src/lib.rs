//! # liva-audit
//!
//! High-performance, mathematically verifiable cryptographic audit crate for LIVA Banking.
//!
//! - **RFC 6962 Binary Merkle Tree**: 0x00 leaf / 0x01 node prefix, Second-Preimage attack resistance,
//!   O(log N) inclusion proofs, sub-millisecond verification.
//! - **HMAC-SHA256 Forward-Chained Ledger**: Append-only tamper-evident audit log with genesis anchoring.
//! - **1-Byte Tamper Detection**: Proves that modifying any byte in transaction data or audit proof
//!   causes 100% verification rejection.

#![deny(clippy::float_arithmetic)]

pub mod chain;
pub mod error;
pub mod merkle;

pub use chain::{
    compute_audit_row_hash, compute_hmac_sha256, digest_payload, genesis_hash, AuditDbRecord,
    AuditLedger, AuditRecord, AuditVerificationReport, verify_audit_db_chain, GENESIS_SEED,
};
pub use error::{AuditError, MerkleAuditError};
pub use merkle::{
    derive_directions, hash_leaf, hash_node, largest_power_of_two_less_than, verify_inclusion,
    BinaryMerkleTree, MerkleInclusionProof, ProofStep, SiblingPosition, TransactionAuditLeaf,
    RFC6962_LEAF_PREFIX, RFC6962_NODE_PREFIX,
};
