//! Core commitment logic: generation, verification, and creator hashing.

use sha2::{Digest, Sha256};

use crate::canonical::{canonical_encode_creator, canonical_encode_title};
use crate::error::AtsError;
use crate::hash::Hash;
use crate::merkle::{MERKLE_DEPTH, MerkleProof, MerkleTree};
use crate::model::{AtsInput, AtsProof, Creator, OnChainCommitment, PROTOCOL_VERSION};
use crate::validate::validate_input;

// ===== Public API =====

/// Hash a media file with SHA-256.
///
/// # Errors
/// Returns [`AtsError::EmptyMedia`] if `data` is empty.
pub fn hash_media(data: &[u8]) -> Result<Hash, AtsError> {
    if data.is_empty() {
        return Err(AtsError::EmptyMedia);
    }
    Ok(Hash::from_bytes(Sha256::digest(data).into()))
}

/// Hash a single creator: `SHA-256(canonical_encode(creator))`.
#[must_use]
pub fn hash_creator(creator: &Creator) -> Hash {
    let bytes = canonical_encode_creator(creator);
    Hash::from_bytes(Sha256::digest(&bytes).into())
}

/// Generate a full ATS proof from user input and media bytes.
///
/// The returned [`AtsProof`] contains:
/// - `on_chain`: the commitment + protocol version (to submit on-chain)
/// - `media_hash`: SHA-256 of the media file
/// - `merkle_tree`: full Merkle tree (root, leaves, and proofs accessible via methods)
///
/// # Errors
/// Returns an [`AtsError`] if any input validation fails.
pub fn generate_commitment(input: &AtsInput, media: &[u8]) -> Result<AtsProof, AtsError> {
    validate_input(input)?;
    let media_hash = hash_media(media)?;

    let creator_hashes: Vec<Hash> = input.creators.iter().map(hash_creator).collect();
    let tree = MerkleTree::build(&creator_hashes);

    let commitment = compute_commitment(media_hash, &input.title, tree.root());

    Ok(AtsProof {
        on_chain: OnChainCommitment {
            commitment,
            protocol_version: PROTOCOL_VERSION,
        },
        media_hash,
        merkle_tree: tree,
    })
}

/// Verify that an [`AtsInput`] + media bytes produce the given on-chain commitment.
///
/// # Errors
/// Returns an [`AtsError`] if input validation fails.
pub fn verify_commitment(
    input: &AtsInput,
    media: &[u8],
    expected: &OnChainCommitment,
) -> Result<bool, AtsError> {
    let proof = generate_commitment(input, media)?;
    Ok(proof.on_chain == *expected)
}

/// Generate a Merkle inclusion proof for the creator at `index`.
///
/// Rebuilds the tree from scratch. If you already have an [`AtsProof`],
/// prefer [`AtsProof::creator_proof`] to avoid recomputation.
///
/// # Errors
/// Returns an [`AtsError`] if input validation fails or `index` is out of bounds.
pub fn generate_creator_proof(input: &AtsInput, index: usize) -> Result<MerkleProof, AtsError> {
    validate_input(input)?;
    if index >= input.creators.len() {
        return Err(AtsError::CreatorIndexOutOfBounds {
            index,
            total: input.creators.len(),
        });
    }

    let creator_hashes: Vec<Hash> = input.creators.iter().map(hash_creator).collect();
    let tree = MerkleTree::build(&creator_hashes);
    Ok(tree.proof(index))
}

/// Verify a creator's inclusion given their data, a Merkle proof, and the expected root.
///
/// Returns `false` if the proof is invalid or the creator doesn't match.
#[must_use]
pub fn verify_creator_inclusion(
    creator: &Creator,
    proof: &MerkleProof,
    merkle_root: &Hash,
) -> bool {
    if proof.len() != MERKLE_DEPTH {
        return false;
    }
    let leaf = hash_creator(creator);
    MerkleTree::verify_proof(leaf, proof, *merkle_root)
}

// ===== Internal =====

/// Compute the final commitment hash.
///
/// `preimage = [version: u8] || [media_hash: 32B] || [title: canonical] || [merkle_root: 32B]`
fn compute_commitment(media_hash: Hash, title: &str, merkle_root: Hash) -> Hash {
    let title_bytes = canonical_encode_title(title);

    let mut hasher = Sha256::new();
    hasher.update([PROTOCOL_VERSION]);
    hasher.update(media_hash.as_bytes());
    hasher.update(&title_bytes);
    hasher.update(merkle_root.as_bytes());
    Hash::from_bytes(hasher.finalize().into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Role;

    fn alice() -> Creator {
        Creator {
            full_name: "Alice Dupont".into(),
            email: "alice@example.com".into(),
            roles: vec![Role::Author, Role::Composer],
            ipi: Some("00012345678".into()),
            isni: None,
        }
    }

    fn bob() -> Creator {
        Creator {
            full_name: "Bob Martin".into(),
            email: "bob@example.com".into(),
            roles: vec![Role::Arranger],
            ipi: None,
            isni: Some("0000000121032683".into()),
        }
    }

    fn sample_input() -> AtsInput {
        AtsInput {
            title: "Ma Chanson".into(),
            creators: vec![alice(), bob()],
        }
    }

    // ----- hash_media -----

    #[test]
    fn hash_media_empty_rejected() {
        assert!(matches!(hash_media(&[]), Err(AtsError::EmptyMedia)));
    }

    #[test]
    fn hash_media_deterministic() {
        let data = b"hello world";
        let a = hash_media(data).unwrap();
        let b = hash_media(data).unwrap();
        assert_eq!(a, b);
    }

    // ----- Commitment generation -----

    #[test]
    fn generate_commitment_roundtrip() {
        let input = sample_input();
        let media = b"test media content";
        let proof = generate_commitment(&input, media).unwrap();
        assert!(verify_commitment(&input, media, &proof.on_chain).unwrap());
    }

    #[test]
    fn commitment_changes_with_title() {
        let media = b"test";
        let input_a = sample_input();
        let mut input_b = sample_input();
        input_b.title = "Different Title".into();

        let a = generate_commitment(&input_a, media).unwrap();
        let b = generate_commitment(&input_b, media).unwrap();
        assert_ne!(a.on_chain.commitment, b.on_chain.commitment);
    }

    #[test]
    fn commitment_changes_with_media() {
        let input = sample_input();
        let a = generate_commitment(&input, b"media A").unwrap();
        let b = generate_commitment(&input, b"media B").unwrap();
        assert_ne!(a.on_chain.commitment, b.on_chain.commitment);
    }

    #[test]
    fn commitment_changes_with_creators() {
        let media = b"test";
        let input_a = sample_input();
        let mut input_b = sample_input();
        input_b.creators[0].full_name = "Charlie".into();

        let a = generate_commitment(&input_a, media).unwrap();
        let b = generate_commitment(&input_b, media).unwrap();
        assert_ne!(a.on_chain.commitment, b.on_chain.commitment);
    }

    #[test]
    fn verify_rejects_wrong_commitment() {
        let input = sample_input();
        let media = b"test";

        let wrong = OnChainCommitment {
            commitment: Hash::from_bytes([0xAA; 32]),
            protocol_version: PROTOCOL_VERSION,
        };
        assert!(!verify_commitment(&input, media, &wrong).unwrap());
    }

    #[test]
    fn verify_rejects_wrong_version() {
        let input = sample_input();
        let media = b"test";
        let proof = generate_commitment(&input, media).unwrap();

        let wrong = OnChainCommitment {
            commitment: proof.on_chain.commitment,
            protocol_version: 99,
        };
        assert!(!verify_commitment(&input, media, &wrong).unwrap());
    }

    // ----- Creator proofs -----

    #[test]
    fn creator_proof_roundtrip() {
        let input = sample_input();
        let media = b"test";
        let proof = generate_commitment(&input, media).unwrap();

        for i in 0..input.creators.len() {
            let mproof = generate_creator_proof(&input, i).unwrap();
            assert!(verify_creator_inclusion(
                &input.creators[i],
                &mproof,
                &proof.creators_merkle_root(),
            ));
        }
    }

    #[test]
    fn creator_proof_index_out_of_bounds() {
        let input = sample_input();
        let result = generate_creator_proof(&input, 99);
        assert!(matches!(
            result,
            Err(AtsError::CreatorIndexOutOfBounds {
                index: 99,
                total: 2
            })
        ));
    }

    #[test]
    fn creator_proof_rejects_wrong_creator() {
        let input = sample_input();
        let media = b"test";
        let proof = generate_commitment(&input, media).unwrap();

        let mproof = generate_creator_proof(&input, 0).unwrap();
        assert!(!verify_creator_inclusion(
            &input.creators[1],
            &mproof,
            &proof.creators_merkle_root(),
        ));
    }

    #[test]
    fn protocol_version_is_1() {
        let input = sample_input();
        let media = b"test";
        let proof = generate_commitment(&input, media).unwrap();
        assert_eq!(proof.on_chain.protocol_version, 1);
    }

    #[test]
    fn proof_contains_32_leaves() {
        let input = sample_input();
        let media = b"test";
        let proof = generate_commitment(&input, media).unwrap();
        assert_eq!(proof.creator_leaves().len(), 32);
    }
}
