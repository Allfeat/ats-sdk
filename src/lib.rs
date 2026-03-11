//! # ATS SDK — Allfeat Timestamp Service
//!
//! Reference implementation for generating and verifying tamper-proof,
//! timestamped, privacy-preserving commitments for musical works on the
//! Allfeat blockchain.
//!
//! ## Quick Start
//!
//! ```rust
//! use ats_sdk::{generate_commitment, verify_commitment, generate_creator_proof, verify_creator_inclusion};
//! use ats_sdk::{AtsInput, Creator, Role};
//!
//! let input = AtsInput {
//!     title: "Ma Chanson".into(),
//!     creators: vec![Creator {
//!         full_name: "Alice Dupont".into(),
//!         email: "alice@example.com".into(),
//!         roles: vec![Role::Author],
//!         ipi: None,
//!         isni: None,
//!     }],
//! };
//!
//! let media = b"raw audio bytes...";
//! let proof = generate_commitment(&input, media).unwrap();
//!
//! // Verify the commitment
//! assert!(verify_commitment(&input, media, &proof.on_chain).unwrap());
//!
//! // Generate a Merkle proof for creator 0
//! let mproof = generate_creator_proof(&input, 0).unwrap();
//! assert!(verify_creator_inclusion(&input.creators[0], &mproof, &proof.creators_merkle_root()));
//! ```

#![deny(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]

pub mod canonical;
pub mod commitment;
/// Error types for the ATS SDK.
pub mod error;
/// SHA-256 hash newtype wrapper.
pub mod hash;
pub mod merkle;
/// Data types: inputs, proofs, and on-chain structures.
pub mod model;
pub mod validate;

// Re-export public API at crate root for ergonomics.
pub use commitment::{
    generate_commitment, generate_creator_proof, hash_creator, hash_media, verify_commitment,
    verify_creator_inclusion,
};
pub use error::AtsError;
pub use hash::Hash;
pub use merkle::{MerkleProof, MerkleTree, MERKLE_DEPTH};
pub use model::{
    AtsInput, AtsProof, Creator, OnChainCommitment, Role, MAX_CREATORS, PROTOCOL_VERSION,
};
pub use validate::validate_input;
