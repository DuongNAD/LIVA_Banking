//! RFC 6962 Binary Merkle Tree for Cryptographic Transaction Audit Trails.
//!
//! Provides mathematically verifiable inclusion proofs (Merkle Paths) for banking
//! transactions with O(log N) complexity, independent verification in < 1 ms,
//! zero exposure of unrelated customer data, and strict domain separation
//! protecting against Second-Preimage Attacks per RFC 6962 Section 2.1.

use crate::banking::models::TransactionRecord;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// RFC 6962 Domain Separator byte for leaf nodes: 0x00.
pub const RFC6962_LEAF_PREFIX: u8 = 0x00;

/// RFC 6962 Domain Separator byte for internal nodes: 0x01.
pub const RFC6962_NODE_PREFIX: u8 = 0x01;

/// Position of a sibling node relative to the verification path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SiblingPosition {
    Left,
    Right,
}

/// A single step in an O(log N) Merkle Inclusion Proof.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofStep {
    pub sibling_hash: [u8; 32],
    pub position: SiblingPosition,
}

/// Standalone cryptographic inclusion proof verifying a transaction belongs to a Merkle root.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MerkleInclusionProof {
    pub leaf_index: usize,
    pub total_leaves: usize,
    pub leaf_hash: [u8; 32],
    pub proof_path: Vec<ProofStep>,
    pub root_hash: [u8; 32],
}

impl MerkleInclusionProof {
    /// Verifies the inclusion proof against original unhashed leaf data.
    pub fn verify(&self, leaf_data: &[u8]) -> bool {
        let computed_leaf = hash_leaf(leaf_data);
        if computed_leaf != self.leaf_hash {
            return false;
        }
        self.verify_leaf_hash(&computed_leaf)
    }

    /// Verifies the inclusion proof directly using the leaf hash.
    pub fn verify_leaf_hash(&self, leaf_hash: &[u8; 32]) -> bool {
        let mut current = *leaf_hash;
        for step in &self.proof_path {
            current = match step.position {
                SiblingPosition::Right => hash_node(&current, &step.sibling_hash),
                SiblingPosition::Left => hash_node(&step.sibling_hash, &current),
            };
        }
        current == self.root_hash
    }

    /// Measures the verification duration and confirms execution in < 1 ms.
    pub fn verify_with_duration(&self, leaf_data: &[u8]) -> (bool, std::time::Duration) {
        let start = std::time::Instant::now();
        let valid = self.verify(leaf_data);
        (valid, start.elapsed())
    }

    /// Returns hexadecimal representation of root hash.
    pub fn root_hex(&self) -> String {
        hex::encode(self.root_hash)
    }

    /// Returns hexadecimal representation of target leaf hash.
    pub fn leaf_hex(&self) -> String {
        hex::encode(self.leaf_hash)
    }
}

/// Serialized representation of a transaction for inclusion as a Merkle tree leaf.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionAuditLeaf {
    pub row_id: usize,
    pub tx_date: i64,
    pub doc_ref: Option<String>,
    pub tx_type: String,
    pub amount: u64,
    pub counterparty_account: Option<String>,
    pub counterparty_name: Option<String>,
    pub narration: String,
    pub reconciled_status: String,
}

impl TransactionAuditLeaf {
    pub fn from_record(tx: &TransactionRecord, status: &str) -> Self {
        Self {
            row_id: tx.row_id,
            tx_date: tx.tx_date,
            doc_ref: tx.doc_ref.clone(),
            tx_type: tx.tx_type.to_string(),
            amount: tx.amount,
            counterparty_account: tx.counterparty_account.clone(),
            counterparty_name: tx.counterparty_name.clone(),
            narration: tx.narration.clone(),
            reconciled_status: status.to_string(),
        }
    }

    pub fn to_leaf_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }
}

/// Errors raised during Merkle tree construction or audit proof generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MerkleAuditError {
    EmptyTree,
    IndexOutOfBounds { index: usize, total: usize },
    InvalidProof,
}

impl std::fmt::Display for MerkleAuditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MerkleAuditError::EmptyTree => write!(f, "Merkle Tree is empty; cannot generate proof"),
            MerkleAuditError::IndexOutOfBounds { index, total } => {
                write!(
                    f,
                    "Leaf index {index} out of bounds (total leaves: {total})"
                )
            }
            MerkleAuditError::InvalidProof => write!(f, "Cryptographic proof verification failed"),
        }
    }
}

impl std::error::Error for MerkleAuditError {}

/// Computes RFC 6962 leaf hash: SHA-256(0x00 || data).
pub fn hash_leaf(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update([RFC6962_LEAF_PREFIX]);
    hasher.update(data);
    hasher.finalize().into()
}

/// Computes RFC 6962 internal node hash: SHA-256(0x01 || left || right).
pub fn hash_node(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update([RFC6962_NODE_PREFIX]);
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

/// Largest power of 2 strictly less than n (for n > 1) per RFC 6962 Section 2.1.
#[inline]
pub fn largest_power_of_two_less_than(n: usize) -> usize {
    debug_assert!(n > 1);
    let mut k = 1;
    while (k << 1) < n {
        k <<= 1;
    }
    k
}

/// Internal tree node structure preserving recursive splits for O(log N) proof generation.
#[derive(Debug, Clone)]
enum MerkleTreeNode {
    Leaf {
        hash: [u8; 32],
    },
    Internal {
        hash: [u8; 32],
        left: Box<MerkleTreeNode>,
        right: Box<MerkleTreeNode>,
        split_k: usize,
    },
}

impl MerkleTreeNode {
    pub fn hash(&self) -> [u8; 32] {
        match self {
            MerkleTreeNode::Leaf { hash, .. } => *hash,
            MerkleTreeNode::Internal { hash, .. } => *hash,
        }
    }
}

/// High-performance RFC 6962 Binary Merkle Tree for banking transaction audit trails.
#[derive(Debug, Clone)]
pub struct BinaryMerkleTree {
    root_node: Option<MerkleTreeNode>,
    total_leaves: usize,
    root_hash: [u8; 32],
}

impl BinaryMerkleTree {
    /// Builds an RFC 6962 Binary Merkle Tree from raw leaf byte slices.
    pub fn from_raw_leaves(leaves: &[Vec<u8>]) -> Self {
        let total = leaves.len();
        if total == 0 {
            let empty_hash: [u8; 32] = Sha256::digest([]).into();
            return Self {
                root_node: None,
                total_leaves: 0,
                root_hash: empty_hash,
            };
        }

        let root_node = Self::build_subtree(leaves);
        let root_hash = root_node.hash();

        Self {
            root_node: Some(root_node),
            total_leaves: total,
            root_hash,
        }
    }

    /// Builds a Merkle Tree from a slice of TransactionRecords.
    pub fn from_transactions(txs: &[TransactionRecord], default_status: &str) -> Self {
        let raw_leaves: Vec<Vec<u8>> = txs
            .iter()
            .map(|tx| TransactionAuditLeaf::from_record(tx, default_status).to_leaf_bytes())
            .collect();
        Self::from_raw_leaves(&raw_leaves)
    }

    fn build_subtree(leaves: &[Vec<u8>]) -> MerkleTreeNode {
        debug_assert!(!leaves.is_empty());
        if leaves.len() == 1 {
            let hash = hash_leaf(&leaves[0]);
            MerkleTreeNode::Leaf { hash }
        } else {
            let n = leaves.len();
            let k = largest_power_of_two_less_than(n);
            let left = Self::build_subtree(&leaves[..k]);
            let right = Self::build_subtree(&leaves[k..]);
            let hash = hash_node(&left.hash(), &right.hash());

            MerkleTreeNode::Internal {
                hash,
                left: Box::new(left),
                right: Box::new(right),
                split_k: k,
            }
        }
    }

    /// Returns the 32-byte SHA-256 Merkle root.
    pub fn root(&self) -> [u8; 32] {
        self.root_hash
    }

    /// Returns hexadecimal representation of the root hash.
    pub fn root_hex(&self) -> String {
        hex::encode(self.root_hash)
    }

    /// Returns total number of leaves in the tree.
    pub fn leaf_count(&self) -> usize {
        self.total_leaves
    }

    /// Generates an O(log N) Merkle Inclusion Proof for a transaction at `leaf_index`.
    pub fn generate_inclusion_proof(
        &self,
        leaf_index: usize,
    ) -> Result<MerkleInclusionProof, MerkleAuditError> {
        if self.total_leaves == 0 {
            return Err(MerkleAuditError::EmptyTree);
        }
        if leaf_index >= self.total_leaves {
            return Err(MerkleAuditError::IndexOutOfBounds {
                index: leaf_index,
                total: self.total_leaves,
            });
        }

        let root_node = self.root_node.as_ref().unwrap();
        let mut path = Vec::new();
        let leaf_hash = Self::collect_audit_path(root_node, leaf_index, &mut path);

        Ok(MerkleInclusionProof {
            leaf_index,
            total_leaves: self.total_leaves,
            leaf_hash,
            proof_path: path,
            root_hash: self.root_hash,
        })
    }

    fn collect_audit_path(
        node: &MerkleTreeNode,
        target_index: usize,
        path: &mut Vec<ProofStep>,
    ) -> [u8; 32] {
        match node {
            MerkleTreeNode::Leaf { hash, .. } => *hash,
            MerkleTreeNode::Internal {
                left,
                right,
                split_k,
                ..
            } => {
                if target_index < *split_k {
                    let target_hash = Self::collect_audit_path(left, target_index, path);
                    // Right child is the sibling
                    path.push(ProofStep {
                        sibling_hash: right.hash(),
                        position: SiblingPosition::Right,
                    });
                    target_hash
                } else {
                    let target_hash =
                        Self::collect_audit_path(right, target_index - *split_k, path);
                    // Left child is the sibling
                    path.push(ProofStep {
                        sibling_hash: left.hash(),
                        position: SiblingPosition::Left,
                    });
                    target_hash
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rfc6962_domain_separation_second_preimage_defense() {
        // Demonstrate Second-Preimage attack resistance:
        // An attacker creates a leaf containing the concatenation of two node hashes (L || R).
        let left_hash = [0x11u8; 32];
        let right_hash = [0x22u8; 32];

        // Valid internal node hash
        let internal_node = hash_node(&left_hash, &right_hash);

        // Attacker payload = Left || Right
        let mut fake_leaf_data = Vec::with_capacity(64);
        fake_leaf_data.extend_from_slice(&left_hash);
        fake_leaf_data.extend_from_slice(&right_hash);

        // Computed leaf hash with 0x00 prefix
        let fake_leaf_hash = hash_leaf(&fake_leaf_data);

        // The leaf hash and internal node hash CANNOT match due to RFC 6962 0x00 vs 0x01 prefix!
        assert_ne!(
            internal_node, fake_leaf_hash,
            "RFC 6962 domain separation MUST prevent Second-Preimage collisions"
        );
    }

    #[test]
    fn test_merkle_tree_inclusion_proof_verification_all_leaves() {
        let raw_leaves: Vec<Vec<u8>> = (0..17)
            .map(|i| format!("transaction-audit-payload-test-number-{i}").into_bytes())
            .collect();

        let tree = BinaryMerkleTree::from_raw_leaves(&raw_leaves);
        assert_eq!(tree.leaf_count(), 17);

        for (i, leaf) in raw_leaves.iter().enumerate() {
            let proof = tree
                .generate_inclusion_proof(i)
                .expect("Proof generation must succeed");

            assert_eq!(proof.leaf_index, i);
            assert_eq!(proof.total_leaves, 17);
            assert_eq!(proof.root_hash, tree.root());

            // Inclusion proof verification must pass
            assert!(
                proof.verify(leaf),
                "Proof verification for leaf {i} must succeed"
            );

            // Verification with wrong data must fail
            let wrong_data = b"tampered-transaction-data";
            assert!(
                !proof.verify(wrong_data),
                "Proof verification with tampered data must fail"
            );
        }
    }

    #[test]
    fn test_merkle_proof_verification_sub_millisecond_benchmark() {
        // Build a tree with 5,000 transactions
        let count = 5_000;
        let raw_leaves: Vec<Vec<u8>> = (0..count)
            .map(|i| format!("tx_batch_record_iso20022_vcb_seq_{i}").into_bytes())
            .collect();

        let tree = BinaryMerkleTree::from_raw_leaves(&raw_leaves);

        // Select an arbitrary transaction
        let target_idx = 3_456;
        let target_leaf = &raw_leaves[target_idx];

        let proof = tree
            .generate_inclusion_proof(target_idx)
            .expect("Inclusion proof must be generated");

        // Proof length must be O(log N): ceil(log2(5000)) = 13
        assert!(
            proof.proof_path.len() <= 14,
            "Proof path length must be O(log N), actual: {}",
            proof.proof_path.len()
        );

        // Measure verification duration
        let (is_valid, duration) = proof.verify_with_duration(target_leaf);
        assert!(
            is_valid,
            "Merkle inclusion proof must be cryptographically valid"
        );

        // Verify that execution is strictly < 1 ms (1,000,000 ns)
        assert!(
            duration < std::time::Duration::from_millis(1),
            "Verification must complete in < 1 ms, actual: {:?}",
            duration
        );
    }

    #[test]
    fn test_merkle_tamper_detection() {
        let leaves: Vec<Vec<u8>> = vec![
            b"tx-1".to_vec(),
            b"tx-2".to_vec(),
            b"tx-3".to_vec(),
            b"tx-4".to_vec(),
        ];
        let tree = BinaryMerkleTree::from_raw_leaves(&leaves);

        let mut proof = tree.generate_inclusion_proof(1).unwrap();
        assert!(proof.verify(b"tx-2"));

        // 1. Tamper with sibling hash
        proof.proof_path[0].sibling_hash[0] ^= 0xFF;
        assert!(
            !proof.verify(b"tx-2"),
            "Tampered sibling hash must invalidate proof"
        );

        // Reset and tamper with position
        let mut proof2 = tree.generate_inclusion_proof(1).unwrap();
        proof2.proof_path[0].position = match proof2.proof_path[0].position {
            SiblingPosition::Left => SiblingPosition::Right,
            SiblingPosition::Right => SiblingPosition::Left,
        };
        assert!(
            !proof2.verify(b"tx-2"),
            "Inverted sibling position must invalidate proof"
        );
    }
}
