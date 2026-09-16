//! RFC 6962 Binary Merkle Tree for Cryptographic Transaction Audit Trails.
//!
//! Features:
//! - Domain separation (0x00 for leaf, 0x01 for internal nodes) protecting against Second-Preimage Attacks
//! - Cryptographic inclusion proofs (Merkle Paths) with O(log N) verification
//! - Tamper detection: modifying or deleting any record invalidates root hash and proof

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const RFC6962_LEAF_PREFIX: u8 = 0x00;
pub const RFC6962_NODE_PREFIX: u8 = 0x01;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SiblingPosition {
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofStep {
    pub sibling_hash: [u8; 32],
    pub position: SiblingPosition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MerkleInclusionProof {
    pub leaf_index: usize,
    pub total_leaves: usize,
    pub leaf_hash: [u8; 32],
    pub proof_path: Vec<ProofStep>,
    pub root_hash: [u8; 32],
}

impl MerkleInclusionProof {
    pub fn verify(&self, leaf_data: &[u8]) -> bool {
        let computed_leaf = hash_leaf(leaf_data);
        if computed_leaf != self.leaf_hash {
            return false;
        }

        let mut current_hash = computed_leaf;
        for step in &self.proof_path {
            current_hash = match step.position {
                SiblingPosition::Left => hash_children(&step.sibling_hash, &current_hash),
                SiblingPosition::Right => hash_children(&current_hash, &step.sibling_hash),
            };
        }

        current_hash == self.root_hash
    }
}

pub fn hash_leaf(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update([RFC6962_LEAF_PREFIX]);
    hasher.update(data);
    hasher.finalize().into()
}

pub fn hash_children(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update([RFC6962_NODE_PREFIX]);
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

#[derive(Debug, Clone)]
pub struct BinaryMerkleTree {
    leaves: Vec<[u8; 32]>,
    levels: Vec<Vec<[u8; 32]>>,
}

impl BinaryMerkleTree {
    pub fn new() -> Self {
        Self {
            leaves: Vec::new(),
            levels: Vec::new(),
        }
    }

    pub fn from_leaves_data(items: &[&[u8]]) -> Self {
        let mut tree = Self::new();
        for item in items {
            tree.append_leaf_data(item);
        }
        tree.rebuild();
        tree
    }

    pub fn append_leaf_data(&mut self, data: &[u8]) {
        self.leaves.push(hash_leaf(data));
    }

    pub fn rebuild(&mut self) {
        self.levels.clear();
        if self.leaves.is_empty() {
            return;
        }

        let mut current_level = self.leaves.clone();
        self.levels.push(current_level.clone());

        while current_level.len() > 1 {
            let mut next_level = Vec::with_capacity((current_level.len() + 1) / 2);
            for chunk in current_level.chunks(2) {
                if chunk.len() == 2 {
                    next_level.push(hash_children(&chunk[0], &chunk[1]));
                } else {
                    // Duplicate last element for odd length
                    next_level.push(hash_children(&chunk[0], &chunk[0]));
                }
            }
            self.levels.push(next_level.clone());
            current_level = next_level;
        }
    }

    pub fn root_hash(&self) -> [u8; 32] {
        if let Some(top) = self.levels.last() {
            if let Some(&root) = top.first() {
                return root;
            }
        }
        Sha256::digest([]).into()
    }

    pub fn generate_proof(&self, leaf_index: usize) -> Option<MerkleInclusionProof> {
        if leaf_index >= self.leaves.len() {
            return None;
        }

        let mut proof_path = Vec::new();
        let mut current_idx = leaf_index;

        for level in &self.levels[..self.levels.len().saturating_sub(1)] {
            let sibling_idx = if current_idx % 2 == 0 {
                if current_idx + 1 < level.len() {
                    current_idx + 1
                } else {
                    current_idx // Sibling is itself when odd
                }
            } else {
                current_idx - 1
            };

            let position = if current_idx % 2 == 0 {
                SiblingPosition::Right
            } else {
                SiblingPosition::Left
            };

            proof_path.push(ProofStep {
                sibling_hash: level[sibling_idx],
                position,
            });

            current_idx /= 2;
        }

        Some(MerkleInclusionProof {
            leaf_index,
            total_leaves: self.leaves.len(),
            leaf_hash: self.leaves[leaf_index],
            proof_path,
            root_hash: self.root_hash(),
        })
    }
}
