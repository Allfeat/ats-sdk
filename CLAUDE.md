# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

ATS (Allfeat Timestamp Service) — a Cargo workspace containing:

- **`ats-sdk`** — Rust reference implementation for generating and verifying tamper-proof, timestamped, privacy-preserving commitments for musical works. Only the commitment hash goes on-chain; all data stays off-chain. A Merkle tree over creators enables selective disclosure.
- **`pallet-ats`** — Substrate pallet that stores ATS commitments on-chain on the Allfeat blockchain. Supports versioning, deposit holds, and hard-delete revocation.

## Commands

```bash
# Workspace-wide
cargo check --workspace --all-targets               # Check all crates
cargo test --workspace                               # Run all tests (66 SDK + 23 pallet)
cargo clippy --workspace --all-targets -- -D warnings # Lint (zero warnings required)
cargo fmt --all --check                              # Check formatting
RUSTDOCFLAGS="-D missing_docs -D warnings" cargo doc --workspace --no-deps  # Build docs

# SDK only
cargo test -p ats-sdk                                # Run SDK tests (47 unit + 18 integration + 1 doctest)
cargo test -p ats-sdk test_vector -- --nocapture     # Run cross-language test vectors with hash output

# Pallet only
cargo test -p pallet-ats                             # Run pallet tests (23 tests)
cargo check -p pallet-ats --features runtime-benchmarks  # Check benchmarks compile
cargo check -p pallet-ats --features try-runtime         # Check try-runtime compiles
```

## Architecture

Cargo workspace with two members. Rust edition 2024, stable toolchain.

### ats-sdk

Two runtime dependencies: `sha2` (SHA-256) and `thiserror`.

#### Module responsibilities

- **`commitment.rs`** — Core logic: `generate_commitment`, `verify_commitment`, `generate_creator_proof`, `verify_creator_inclusion`, `hash_media`, `hash_creator`. This is the main entry point for all operations.
- **`canonical.rs`** — Deterministic byte serialization for cross-language reproducibility. Strings encoded as `[u32 LE length][UTF-8 bytes]`, optionals with `0x00`/`0x01` prefix, roles sorted and deduplicated.
- **`merkle.rs`** — Fixed-depth (5) SHA-256 Merkle tree. 32 leaf slots, unused padded with `SHA-256(0x00)`. Proofs are `Vec<(Hash, bool)>` (sibling + position).
- **`model.rs`** — Data types: `AtsInput`, `Creator`, `Role` (4 variants), `AtsProof`, `OnChainCommitment`.
- **`validate.rs`** — Input validation (called automatically by `generate_commitment`). 11 error variants covering all constraints.
- **`hash.rs`** — `Hash` newtype over `[u8; 32]` with hex Display, conversions.
- **`error.rs`** — `AtsError` enum using `thiserror`.
- **`lib.rs`** — Re-exports the public API at crate root.

#### Hashing pipeline

```
media_hash    = SHA-256(file_bytes)
creator_leaf  = SHA-256(canonical_encode(creator))
merkle_root   = MerkleTree(creator_leaves, depth=5, pad=SHA-256(0x00))
commitment    = SHA-256(version || media_hash || canonical(title) || merkle_root)
```

### pallet-ats

Substrate pallet (`no_std`). Dependencies: `frame-support` v45, `frame-system` v45, `pallet-balances` v46 (dev), matching the Allfeat runtime.

#### Storage

- **`NextAtsId`** — `StorageValue<u64>`: auto-incrementing counter
- **`AtsRegistry`** — `StorageMap<AtsId, AtsRecord>`: owner, created_at, version_count, base_deposit
- **`AtsVersions`** — `StorageDoubleMap<AtsId, u32, VersionRecord>`: commitment `[u8;32]`, protocol_version, created_at, deposit
- **`OwnerIndex`** — `StorageMap<AccountId, BoundedVec<AtsId>>`: per-owner index (bounded by `MaxAtsPerAccount`)

#### Extrinsics

- **`create(commitment, protocol_version)`** — Creates ATS + version 0, holds base + version deposit
- **`update(ats_id, commitment, protocol_version)`** — Adds version N, holds version deposit (owner-only)
- **`revoke(ats_id)`** — Hard-deletes ATS + all versions, releases all deposits (owner-only)

#### Deposit model

Uses `fungible::hold` (not deprecated `Currency::reserve`). `HoldReason` enum: `AtsDeposit`, `VersionDeposit`. Full restitution on revocation.

## Code conventions

- `#![deny(unsafe_code)]` — no unsafe code allowed (ats-sdk)
- `#![deny(clippy::all)]` and `#![warn(clippy::pedantic)]` — strict linting (ats-sdk)
- All public items must have documentation (enforced by CI with `RUSTDOCFLAGS="-D missing_docs"`)
- Canonical encoding must be deterministic and language-agnostic — any change requires updating cross-language test vectors
- Protocol constants: `PROTOCOL_VERSION = 1`, `MERKLE_DEPTH = 5`, `MAX_CREATORS = 32`

## CI

GitHub Actions runs: check → test, clippy, doc, pallet-features (parallel after check), and fmt (independent). All must pass. Release uses release-please with per-package components and crates.io publishing.
