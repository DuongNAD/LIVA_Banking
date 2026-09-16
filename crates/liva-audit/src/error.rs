use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum MerkleAuditError {
    #[error("Merkle Tree is empty; cannot generate proof")]
    EmptyTree,

    #[error("Leaf index {index} out of bounds (total leaves: {total})")]
    IndexOutOfBounds { index: usize, total: usize },

    #[error("Cryptographic proof verification failed")]
    InvalidProof,
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum AuditError {
    #[error("Audit verification failed at seq_id {seq_id}: {reason}")]
    TamperedRecord { seq_id: u64, reason: String },

    #[error("Audit log is empty")]
    EmptyLedger,
}
