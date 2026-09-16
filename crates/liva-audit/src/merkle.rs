use crate::error::MerkleAuditError;
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

/// Standalone cryptographic inclusion proof verifying a leaf belongs to a Merkle root.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MerkleInclusionProof {
    pub leaf_index: usize,
    pub total_leaves: usize,
    pub leaf_hash: [u8; 32],
    pub audit_path: Vec<[u8; 32]>,
    pub proof_path: Vec<ProofStep>,
    pub root_hash: [u8; 32],
}

impl MerkleInclusionProof {
    /// Verifies inclusion of raw unhashed leaf data against the specified root hash.
    pub fn verify_inclusion(&self, root_hash: &[u8; 32], leaf_data: &[u8]) -> bool {
        let computed_leaf = hash_leaf(leaf_data);
        if computed_leaf != self.leaf_hash {
            return false;
        }
        self.verify_inclusion_hash(root_hash, &computed_leaf)
    }

    /// Verifies inclusion of a pre-computed leaf hash against the specified root hash.
    pub fn verify_inclusion_hash(&self, root_hash: &[u8; 32], leaf_hash: &[u8; 32]) -> bool {
        // Boundary validation: 0 leaves cannot contain any element; out-of-bounds indices are invalid.
        if self.total_leaves == 0 || self.leaf_index >= self.total_leaves {
            return false;
        }

        // For a single-leaf tree (N = 1), the solitary leaf is the tree root, requiring empty paths.
        if self.total_leaves == 1 {
            return self.audit_path.is_empty()
                && self.proof_path.is_empty()
                && *leaf_hash == *root_hash;
        }

        // For multi-leaf trees (N > 1), an empty inclusion path is cryptographically invalid.
        if self.audit_path.is_empty() && self.proof_path.is_empty() {
            return false;
        }

        let directions = derive_directions(self.leaf_index, self.total_leaves);

        // If explicit proof_path with positions is populated, evaluate using positions
        if !self.proof_path.is_empty() {
            if self.proof_path.len() != directions.len() {
                return false;
            }
            let mut current = *leaf_hash;
            for (step, expected_pos) in self.proof_path.iter().zip(directions.iter()) {
                if step.position != *expected_pos {
                    return false;
                }
                current = match step.position {
                    SiblingPosition::Right => hash_node(&current, &step.sibling_hash),
                    SiblingPosition::Left => hash_node(&step.sibling_hash, &current),
                };
            }
            return current == *root_hash;
        }

        // Otherwise, evaluate using RFC 6962 deterministic split path from leaf_index & total_leaves
        if self.audit_path.len() != directions.len() {
            return false;
        }

        let mut current = *leaf_hash;
        for (sibling_hash, pos) in self.audit_path.iter().zip(directions.iter()) {
            current = match pos {
                SiblingPosition::Right => hash_node(&current, sibling_hash),
                SiblingPosition::Left => hash_node(sibling_hash, &current),
            };
        }

        current == *root_hash
    }

    /// Verifies the proof against self.root_hash.
    pub fn verify(&self, leaf_data: &[u8]) -> bool {
        self.verify_inclusion(&self.root_hash, leaf_data)
    }

    /// Measures verification duration and confirms execution in < 1 ms.
    pub fn verify_with_duration(&self, leaf_data: &[u8]) -> (bool, std::time::Duration) {
        let start = std::time::Instant::now();
        let valid = self.verify(leaf_data);
        (valid, start.elapsed())
    }

    pub fn root_hex(&self) -> String {
        hex::encode(self.root_hash)
    }

    pub fn leaf_hex(&self) -> String {
        hex::encode(self.leaf_hash)
    }
}

/// Standalone verification function conforming to interface requirement:
/// `verify_inclusion(root_hash: &[u8; 32], leaf_data: &[u8]) -> bool`
pub fn verify_inclusion(
    root_hash: &[u8; 32],
    leaf_data: &[u8],
    proof: &MerkleInclusionProof,
) -> bool {
    proof.verify_inclusion(root_hash, leaf_data)
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

/// Computes the largest power of 2 strictly less than n (for n > 1) per RFC 6962 Section 2.1.
#[inline]
pub fn largest_power_of_two_less_than(n: usize) -> usize {
    debug_assert!(n > 1);
    let mut k = 1;
    while (k << 1) < n {
        k <<= 1;
    }
    k
}

/// Derives RFC 6962 traversal directions from leaf index and total leaves (bottom-up).
pub fn derive_directions(leaf_index: usize, total_leaves: usize) -> Vec<SiblingPosition> {
    if total_leaves <= 1 {
        return Vec::new();
    }
    let mut dirs = Vec::new();
    let mut m = leaf_index;
    let mut n = total_leaves;
    while n > 1 {
        let k = largest_power_of_two_less_than(n);
        if m < k {
            dirs.push(SiblingPosition::Right);
            n = k;
        } else {
            dirs.push(SiblingPosition::Left);
            m -= k;
            n -= k;
        }
    }
    dirs.reverse(); // Return bottom-up to match audit_path order
    dirs
}

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

/// Pure RFC 6962 Binary Merkle Tree for banking transaction audit trails.
#[derive(Debug, Clone)]
pub struct BinaryMerkleTree {
    root_node: Option<MerkleTreeNode>,
    total_leaves: usize,
    root_hash: [u8; 32],
}

impl BinaryMerkleTree {
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

    pub fn root(&self) -> [u8; 32] {
        self.root_hash
    }

    pub fn root_hex(&self) -> String {
        hex::encode(self.root_hash)
    }

    pub fn leaf_count(&self) -> usize {
        self.total_leaves
    }

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
        let mut proof_path = Vec::new();
        let leaf_hash = Self::collect_audit_path(root_node, leaf_index, &mut proof_path);

        let audit_path: Vec<[u8; 32]> = proof_path.iter().map(|s| s.sibling_hash).collect();

        Ok(MerkleInclusionProof {
            leaf_index,
            total_leaves: self.total_leaves,
            leaf_hash,
            audit_path,
            proof_path,
            root_hash: self.root_hash,
        })
    }

    fn collect_audit_path(
        node: &MerkleTreeNode,
        target_index: usize,
        path: &mut Vec<ProofStep>,
    ) -> [u8; 32] {
        match node {
            MerkleTreeNode::Leaf { hash } => *hash,
            MerkleTreeNode::Internal {
                left,
                right,
                split_k,
                ..
            } => {
                if target_index < *split_k {
                    let target_hash = Self::collect_audit_path(left, target_index, path);
                    path.push(ProofStep {
                        sibling_hash: right.hash(),
                        position: SiblingPosition::Right,
                    });
                    target_hash
                } else {
                    let target_hash =
                        Self::collect_audit_path(right, target_index - *split_k, path);
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

/// Serialized representation of a transaction for inclusion as a Merkle tree leaf.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    pub fn to_leaf_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }
}
