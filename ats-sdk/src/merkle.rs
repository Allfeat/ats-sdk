//! Fixed-depth SHA-256 Merkle tree for creator inclusion proofs.
//!
//! - Depth: [`MERKLE_DEPTH`] (32 leaves max)
//! - Unused leaves: padded with `SHA-256(0x00)` (zero leaf)
//! - Internal nodes: `SHA-256(left_child || right_child)`

use alloc::vec::Vec;
use sha2::{Digest, Sha256};

use crate::hash::Hash;

/// Merkle tree depth (fixed by protocol).
pub const MERKLE_DEPTH: usize = 5;

/// Number of leaves in the fixed-depth tree: `2^MERKLE_DEPTH`.
pub const NUM_LEAVES: usize = 1 << MERKLE_DEPTH;

/// A Merkle inclusion proof: list of `(sibling_hash, sibling_is_left)`.
///
/// - `sibling_is_left = true`  → `H(sibling || current)`
/// - `sibling_is_left = false` → `H(current || sibling)`
pub type MerkleProof = Vec<(Hash, bool)>;

/// Compute the zero-leaf value: `SHA-256(0x00)`.
#[must_use]
pub fn zero_leaf() -> Hash {
    let digest = Sha256::digest([0x00]);
    Hash::from_bytes(digest.into())
}

/// A fixed-depth SHA-256 Merkle tree.
///
/// `layers[0]` contains the 32 leaves, `layers[MERKLE_DEPTH]` contains
/// the single root node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerkleTree {
    layers: Vec<Vec<Hash>>,
}

impl MerkleTree {
    /// Build a Merkle tree from the given leaves (at most [`NUM_LEAVES`]).
    /// Unused slots are padded with [`zero_leaf()`].
    ///
    /// # Panics
    /// Panics if `leaves.len() > NUM_LEAVES`.
    #[must_use]
    pub fn build(leaves: &[Hash]) -> Self {
        assert!(
            leaves.len() <= NUM_LEAVES,
            "too many leaves: {} > {NUM_LEAVES}",
            leaves.len()
        );

        let zero = zero_leaf();
        let mut padded = Vec::with_capacity(NUM_LEAVES);
        padded.extend_from_slice(leaves);
        padded.resize(NUM_LEAVES, zero);

        let mut layers: Vec<Vec<Hash>> = Vec::with_capacity(MERKLE_DEPTH + 1);
        layers.push(padded);

        for depth in 0..MERKLE_DEPTH {
            let prev = &layers[depth];
            let mut next = Vec::with_capacity(prev.len() / 2);
            for pair in prev.chunks_exact(2) {
                next.push(hash_pair(pair[0], pair[1]));
            }
            layers.push(next);
        }

        debug_assert_eq!(layers.len(), MERKLE_DEPTH + 1);
        debug_assert_eq!(layers[MERKLE_DEPTH].len(), 1);

        Self { layers }
    }

    /// Return the Merkle root.
    #[must_use]
    pub fn root(&self) -> Hash {
        self.layers[MERKLE_DEPTH][0]
    }

    /// Return the leaf hashes (all 32, including zero-leaf padding).
    #[must_use]
    pub fn leaves(&self) -> &[Hash] {
        &self.layers[0]
    }

    /// Generate an inclusion proof for the leaf at `index`.
    ///
    /// Returns [`MERKLE_DEPTH`] sibling entries `(sibling_hash, sibling_is_left)`.
    ///
    /// # Panics
    /// Panics if `index >= NUM_LEAVES`.
    #[must_use]
    pub fn proof(&self, index: usize) -> MerkleProof {
        assert!(index < NUM_LEAVES, "leaf index out of range: {index}");

        let mut proof = Vec::with_capacity(MERKLE_DEPTH);
        let mut idx = index;
        for depth in 0..MERKLE_DEPTH {
            let sibling_idx = idx ^ 1;
            let sibling_is_left = idx & 1 == 1;
            proof.push((self.layers[depth][sibling_idx], sibling_is_left));
            idx >>= 1;
        }

        debug_assert_eq!(proof.len(), MERKLE_DEPTH);
        proof
    }

    /// Verify an inclusion proof against an expected root.
    #[must_use]
    pub fn verify_proof(leaf: Hash, proof: &MerkleProof, expected_root: Hash) -> bool {
        if proof.len() != MERKLE_DEPTH {
            return false;
        }
        let mut current = leaf;
        for &(sibling, sibling_is_left) in proof {
            current = if sibling_is_left {
                hash_pair(sibling, current)
            } else {
                hash_pair(current, sibling)
            };
        }
        current == expected_root
    }
}

/// `SHA-256(left || right)`
fn hash_pair(left: Hash, right: Hash) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(left.as_bytes());
    hasher.update(right.as_bytes());
    Hash::from_bytes(hasher.finalize().into())
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::*;

    fn make_leaf(val: u8) -> Hash {
        Hash::from_bytes(Sha256::digest([val]).into())
    }

    #[test]
    fn zero_leaf_is_sha256_of_0x00() {
        let z = zero_leaf();
        let expected = Sha256::digest([0x00]);
        assert_eq!(z.as_bytes(), expected.as_slice());
    }

    #[test]
    fn single_leaf_tree() {
        let leaf = make_leaf(0x42);
        let tree = MerkleTree::build(&[leaf]);
        assert_eq!(tree.leaves()[0], leaf);
        let z = zero_leaf();
        for l in &tree.leaves()[1..] {
            assert_eq!(*l, z);
        }
    }

    #[test]
    fn root_is_deterministic() {
        let leaves: Vec<Hash> = (0..5).map(make_leaf).collect();
        let a = MerkleTree::build(&leaves);
        let b = MerkleTree::build(&leaves);
        assert_eq!(a.root(), b.root());
    }

    #[test]
    fn proof_roundtrip_all_leaves() {
        let leaves: Vec<Hash> = (0u8..10).map(make_leaf).collect();
        let tree = MerkleTree::build(&leaves);
        let root = tree.root();

        for i in 0..NUM_LEAVES {
            let proof = tree.proof(i);
            assert_eq!(proof.len(), MERKLE_DEPTH);
            assert!(
                MerkleTree::verify_proof(tree.leaves()[i], &proof, root),
                "proof failed for leaf {i}"
            );
        }
    }

    #[test]
    fn wrong_leaf_fails_verification() {
        let leaves: Vec<Hash> = (0u8..3).map(make_leaf).collect();
        let tree = MerkleTree::build(&leaves);
        let root = tree.root();
        let proof = tree.proof(0);
        let wrong_leaf = make_leaf(0xFF);
        assert!(!MerkleTree::verify_proof(wrong_leaf, &proof, root));
    }

    #[test]
    fn wrong_root_fails_verification() {
        let leaves: Vec<Hash> = (0u8..3).map(make_leaf).collect();
        let tree = MerkleTree::build(&leaves);
        let proof = tree.proof(0);
        let wrong_root = Hash::from_bytes([0xAA; 32]);
        assert!(!MerkleTree::verify_proof(
            tree.leaves()[0],
            &proof,
            wrong_root
        ));
    }

    #[test]
    fn full_tree_32_leaves() {
        let leaves: Vec<Hash> = (0u8..32).map(make_leaf).collect();
        let tree = MerkleTree::build(&leaves);
        let root = tree.root();
        for (i, leaf) in leaves.iter().enumerate() {
            let proof = tree.proof(i);
            assert!(MerkleTree::verify_proof(*leaf, &proof, root));
        }
    }

    #[test]
    #[should_panic(expected = "too many leaves")]
    fn too_many_leaves_panics() {
        let leaves: Vec<Hash> = (0u8..33).map(make_leaf).collect();
        let _ = MerkleTree::build(&leaves);
    }

    #[test]
    fn empty_leaves() {
        let tree = MerkleTree::build(&[]);
        let z = zero_leaf();
        for l in tree.leaves() {
            assert_eq!(*l, z);
        }
    }

    #[test]
    fn invalid_proof_length_rejected() {
        let leaves: Vec<Hash> = (0u8..2).map(make_leaf).collect();
        let tree = MerkleTree::build(&leaves);
        let root = tree.root();
        let mut proof = tree.proof(0);
        proof.pop();
        assert!(!MerkleTree::verify_proof(tree.leaves()[0], &proof, root));
    }
}
