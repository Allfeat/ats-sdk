# ATS SDK — Allfeat Timestamp Service

Reference Rust implementation for generating and verifying tamper-proof, timestamped, privacy-preserving commitments for musical works on the [Allfeat](https://allfeat.org) blockchain.

## What it does

ATS lets creators register a musical work by submitting metadata (media file, title, contributors). The SDK produces a **commitment hash** that is posted on-chain — providing an immutable, timestamped proof of existence without revealing any private data.

- Only the commitment hash goes on-chain. All actual data stays off-chain.
- A **Merkle tree** over creators enables **selective disclosure**: any single creator can prove their inclusion without revealing the others.
- The protocol is based on **SHA-256** — no ZK, no trusted setup, runs everywhere.

## Quick start

Add to your `Cargo.toml`:

```toml
[dependencies]
ats-sdk = { path = "." }
```

### Generate a commitment

```rust
use ats_sdk::{generate_commitment, AtsInput, Creator, Role};

let input = AtsInput {
    title: "Ma Chanson".into(),
    creators: vec![Creator {
        full_name: "Alice Dupont".into(),
        email: "alice@example.com".into(),
        roles: vec![Role::Author, Role::Composer],
        ipi: None,
        isni: None,
    }],
};

let proof = generate_commitment(&input, b"raw audio bytes...").unwrap();

// Submit on-chain:
//   proof.on_chain.commitment  — H256 (32 bytes)
//   proof.on_chain.protocol_version — u8
```

### Verify a commitment

```rust
use ats_sdk::verify_commitment;

let is_valid = verify_commitment(&input, b"raw audio bytes...", &proof.on_chain).unwrap();
assert!(is_valid);
```

### Selective disclosure (Merkle proofs)

A creator can prove they are part of a registered work without revealing the other creators:

```rust
use ats_sdk::{generate_creator_proof, verify_creator_inclusion};

// Generate proof for creator at index 0
let merkle_proof = generate_creator_proof(&input, 0).unwrap();

// Anyone can verify with: creator data + proof + merkle root (from certificate)
let valid = verify_creator_inclusion(
    &input.creators[0],
    &merkle_proof,
    &proof.creators_merkle_root(),
);
assert!(valid);
```

If you already have an `AtsProof`, use the convenience method to avoid rebuilding the tree:

```rust
let merkle_proof = proof.creator_proof(0);
```

## API

| Function | Description |
|---|---|
| `generate_commitment(input, media)` | Validate input, hash everything, return full `AtsProof` |
| `verify_commitment(input, media, expected)` | Recompute and compare against an on-chain commitment |
| `generate_creator_proof(input, index)` | Build a Merkle inclusion proof for one creator |
| `verify_creator_inclusion(creator, proof, root)` | Verify a creator's Merkle proof against a root |
| `hash_media(data)` | SHA-256 of raw media bytes |
| `hash_creator(creator)` | SHA-256 of a canonically encoded creator |
| `validate_input(input)` | Validate input before committing (called automatically) |

## Data model

### Creator

| Field | Type | Required | Constraints |
|---|---|---|---|
| `full_name` | `String` | yes | non-empty |
| `email` | `String` | yes | non-empty |
| `roles` | `Vec<Role>` | yes | at least one: `Author`, `Composer`, `Arranger`, `Adapter` |
| `ipi` | `Option<String>` | no | 1-11 digits |
| `isni` | `Option<String>` | no | exactly 16 chars `[0-9X]` |

### Limits

- 1 to 32 creators per work (Merkle tree depth 5)
- Media: any file type, must be non-empty
- Title: non-empty string

## Hashing pipeline

```
1. media_hash       = SHA-256(file_bytes)
2. creator_leaf[i]  = SHA-256(canonical_encode(creator[i]))
3. merkle_root      = MerkleTree(creator_leaves, depth=5, pad=SHA-256(0x00))
4. commitment       = SHA-256(version || media_hash || canonical(title) || merkle_root)
```

The **canonical encoding** is deterministic and language-agnostic:
- Strings: `[length as u32 LE][UTF-8 bytes]`
- Optional strings: `0x00` for None, `0x01 + string encoding` for Some
- Roles: deduplicated, sorted by tag ascending, `[count as u8][tag bytes...]`
- Creator field order: `full_name`, `email`, `roles`, `ipi`, `isni`

## On-chain footprint

Only two values are submitted on-chain per deposit:

| Field | Type | Description |
|---|---|---|
| `commitment` | `H256` | The final binding hash |
| `protocol_version` | `u8` | Currently `1` |

Everything else (media hash, title, creators, Merkle tree) stays off-chain in the certificate.

## Project structure

```
src/
  lib.rs           Public API, re-exports, crate docs
  model.rs         AtsInput, Creator, Role, AtsProof, OnChainCommitment
  hash.rs          Hash newtype wrapper (Display as hex, From, AsRef)
  canonical.rs     Deterministic byte serialization
  merkle.rs        Fixed-depth SHA-256 Merkle tree (build, prove, verify)
  commitment.rs    Core logic (generate, verify, hash)
  validate.rs      Input validation
  error.rs         AtsError enum
tests/
  integration.rs   End-to-end tests, cross-language test vectors
```

## Testing

```sh
cargo test
```

66 tests: 47 unit + 18 integration + 1 doctest. Covers:
- Canonical encoding byte-level correctness
- Merkle tree construction, proof generation, and verification (all 32 leaves)
- Commitment generation, verification, and sensitivity to any data change
- Selective disclosure roundtrips
- All validation rules (11 error variants)
- Unicode support, role ordering invariance, duplicate role deduplication
- Cross-language test vectors (deterministic hash outputs for other implementations)

## Cross-language compatibility

The canonical encoding and hashing pipeline are designed to be reproducible in any language. Run the test vectors to get reference hash values:

```sh
cargo test test_vector -- --nocapture
```

Any conforming implementation (TypeScript, Python, etc.) **must** produce identical commitment hashes for the same inputs.

## License

MIT
