//! Cryptographic Merkle Audit Trail and Configuration Sealing.

pub mod merkle;
pub mod seal;

pub use merkle::{
    compute_merkle_root, hash_leaf, hash_node, BinaryMerkleTree, MerkleAuditError,
    MerkleInclusionProof, ProofStep, SiblingPosition, RFC6962_LEAF_PREFIX, RFC6962_NODE_PREFIX,
};
pub use seal::{
    canonical_json_string, compute_config_merkle_tree, compute_config_seal, create_config_seal,
    verify_config_integrity, ConfigSealError, ConfigSealRecord,
};
