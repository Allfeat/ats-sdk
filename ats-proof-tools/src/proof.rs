use ats_sdk::{Hash, MerkleProof};
use serde::{Deserialize, Serialize};

/// A proof step in a Merkle inclusion proof.
#[derive(Serialize, Deserialize, Clone)]
pub struct ProofStepExport {
    pub sibling: String,
    pub is_left: bool,
}

/// Parse a hex string (with or without `0x` prefix) into an `ats_sdk::Hash`.
pub fn parse_hex_hash(hex_str: &str) -> Result<Hash, String> {
    let trimmed = hex_str.trim().strip_prefix("0x").unwrap_or(hex_str.trim());
    let bytes = hex::decode(trimmed).map_err(|e| format!("invalid hex: {e}"))?;
    let arr: [u8; 32] = bytes
        .try_into()
        .map_err(|_| "hash must be exactly 32 bytes (64 hex chars)".to_string())?;
    Ok(Hash::from_bytes(arr))
}

/// Parse a JSON array of proof steps into a `MerkleProof`.
pub fn parse_merkle_proof_json(json: &str) -> Result<MerkleProof, String> {
    let steps: Vec<ProofStepExport> =
        serde_json::from_str(json).map_err(|e| format!("invalid JSON: {e}"))?;
    steps
        .into_iter()
        .map(|s| {
            let hash = parse_hex_hash(&s.sibling)?;
            Ok((hash, s.is_left))
        })
        .collect()
}
