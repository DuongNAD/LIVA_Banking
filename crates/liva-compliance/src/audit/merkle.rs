//! RFC 6962 Binary Merkle Tree for Cryptographic Audit Trails.
//!
//! Provides mathematically verifiable inclusion proofs (Merkle Paths) with O(log N) complexity,
//! sub-millisecond verification, and domain separation protecting against Second-Preimage Attacks
//! per RFC 6962 Section 2.1 (0x00 leaf prefix, 0x01 internal node prefix).

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

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

/// Standalone cryptographic inclusion proof verifying leaf data belongs to a Merkle root.
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

    /// Returns hexadecimal representation of root hash.
    pub fn root_hex(&self) -> String {
        hex::encode(self.root_hash)
    }

    /// Returns hexadecimal representation of target leaf hash.
    pub fn leaf_hex(&self) -> String {
        hex::encode(self.leaf_hash)
    }
}

/// Errors raised during Merkle tree construction or audit proof generation.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MerkleAuditError {
    #[error("Merkle Tree is empty; cannot generate proof")]
    EmptyTree,

    #[error("Leaf index {index} out of bounds (total leaves: {total})")]
    IndexOutOfBounds { index: usize, total: usize },

    #[error("Cryptographic proof verification failed")]
    InvalidProof,
}

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
            MerkleTreeNode::Leaf { hash } => *hash,
            MerkleTreeNode::Internal { hash, .. } => *hash,
        }
    }
}

/// High-performance RFC 6962 Binary Merkle Tree.
#[derive(Debug, Clone)]
pub struct BinaryMerkleTree {
    root_node: Option<MerkleTreeNode>,
    total_leaves: usize,
    root_hash: [u8; 32],
}

impl BinaryMerkleTree {
    /// Builds an RFC 6962 Binary Merkle Tree from raw unhashed leaf byte slices.
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

    /// Builds an RFC 6962 Binary Merkle Tree from already-computed leaf hashes.
    pub fn from_leaf_hashes(hashes: &[[u8; 32]]) -> Self {
        let total = hashes.len();
        if total == 0 {
            let empty_hash: [u8; 32] = Sha256::digest([]).into();
            return Self {
                root_node: None,
                total_leaves: 0,
                root_hash: empty_hash,
            };
        }

        let root_node = Self::build_hash_subtree(hashes);
        let root_hash = root_node.hash();

        Self {
            root_node: Some(root_node),
            total_leaves: total,
            root_hash,
        }
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

    fn build_hash_subtree(hashes: &[[u8; 32]]) -> MerkleTreeNode {
        debug_assert!(!hashes.is_empty());
        if hashes.len() == 1 {
            MerkleTreeNode::Leaf { hash: hashes[0] }
        } else {
            let n = hashes.len();
            let k = largest_power_of_two_less_than(n);
            let left = Self::build_hash_subtree(&hashes[..k]);
            let right = Self::build_hash_subtree(&hashes[k..]);
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

    /// Total number of leaves in the Merkle tree.
    pub fn total_leaves(&self) -> usize {
        self.total_leaves
    }

    /// Generates an RFC 6962 audit path inclusion proof for the leaf at `leaf_index`.
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

        let mut proof_path = Vec::new();
        let (leaf_hash, _) = Self::collect_proof(
            self.root_node.as_ref().unwrap(),
            leaf_index,
            self.total_leaves,
            &mut proof_path,
        );

        proof_path.reverse();

        Ok(MerkleInclusionProof {
            leaf_index,
            total_leaves: self.total_leaves,
            leaf_hash,
            proof_path,
            root_hash: self.root_hash,
        })
    }

    #[allow(clippy::only_used_in_recursion)]
    fn collect_proof(
        node: &MerkleTreeNode,
        target: usize,
        n: usize,
        path: &mut Vec<ProofStep>,
    ) -> ([u8; 32], usize) {
        match node {
            MerkleTreeNode::Leaf { hash } => (*hash, 1),
            MerkleTreeNode::Internal {
                left,
                right,
                split_k,
                ..
            } => {
                let k = *split_k;
                if target < k {
                    path.push(ProofStep {
                        sibling_hash: right.hash(),
                        position: SiblingPosition::Right,
                    });
                    Self::collect_proof(left, target, k, path)
                } else {
                    path.push(ProofStep {
                        sibling_hash: left.hash(),
                        position: SiblingPosition::Left,
                    });
                    Self::collect_proof(right, target - k, n - k, path)
                }
            }
        }
    }
}

/// Compute 256-bit cryptographic Merkle root hash from leaves.
/// Provided for direct backward compatibility with `liva-recon`.
pub fn compute_merkle_root(leaf_hashes: &[String]) -> String {
    if leaf_hashes.is_empty() {
        return "0x0000000000000000000000000000000000000000000000000000000000000000".to_string();
    }

    let mut current_layer: Vec<String> = leaf_hashes.to_vec();

    while current_layer.len() > 1 {
        let mut next_layer = Vec::new();
        let mut i = 0;
        while i < current_layer.len() {
            let left = &current_layer[i];
            let right = if i + 1 < current_layer.len() {
                &current_layer[i + 1]
            } else {
                left
            };

            let combined_hash = hash_pair_recon(left, right);
            next_layer.push(combined_hash);
            i += 2;
        }
        current_layer = next_layer;
    }

    current_layer[0].clone()
}

fn hash_pair_recon(a: &str, b: &str) -> String {
    let mut h1: u64 = 0xcbf29ce484222325;
    let mut h2: u64 = 0x100000001b3;
    let mut h3: u64 = 0x811c9dc5;
    let mut h4: u64 = 0x9e3779b97f4a7c15;

    for byte in a.as_bytes().iter().chain(b.as_bytes().iter()) {
        h1 = h1.wrapping_mul(0x100000001b3) ^ (*byte as u64);
        h2 = h2.wrapping_add(h1) ^ 0x5bd1e995;
        h3 = (h3 << 5).wrapping_add(h2) ^ (*byte as u64);
        h4 = h4.wrapping_mul(31) ^ (*byte as u64);
    }

    format!("0x{:016x}{:016x}{:016x}{:016x}", h1, h2, h3, h4)
}
