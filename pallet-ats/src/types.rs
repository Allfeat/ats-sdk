use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

/// ATS registry record storing ownership, depositor, and deposit information.
#[derive(Clone, Encode, Decode, MaxEncodedLen, TypeInfo, PartialEq, Eq, Debug)]
pub struct AtsRecord<AccountId, BlockNumber, Balance> {
    /// Account that owns this ATS entry.
    pub owner: AccountId,
    /// Account that paid the base deposit (may differ from owner in on-behalf flows).
    pub depositor: AccountId,
    /// Block number when this ATS entry was created.
    pub created_at: BlockNumber,
    /// Number of versions (including the initial version 0).
    pub version_count: u32,
    /// Base deposit held for this ATS entry.
    pub base_deposit: Balance,
}

/// Version record for an ATS entry.
#[derive(Clone, Encode, Decode, MaxEncodedLen, TypeInfo, PartialEq, Eq, Debug)]
pub struct VersionRecord<AccountId, BlockNumber, Balance> {
    /// SHA-256 commitment hash (32 bytes).
    pub commitment: [u8; 32],
    /// Protocol version used to generate the commitment.
    pub protocol_version: u8,
    /// Account that paid the deposit for this version.
    pub depositor: AccountId,
    /// Block number when this version was created.
    pub created_at: BlockNumber,
    /// Deposit held for this version.
    pub deposit: Balance,
}

/// Actions that can be performed on behalf of an owner.
///
/// Used in signed payloads to prevent cross-action replay attacks.
#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Debug)]
pub enum OnBehalfAction {
    /// Create a new ATS entry.
    Create,
    /// Update an existing ATS entry.
    Update,
    /// Revoke an existing ATS entry.
    Revoke,
}

/// Signed payload for creating an ATS entry on behalf of an owner.
#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Debug)]
pub struct CreateOnBehalfPayload<AccountId> {
    /// The action type (must be [`OnBehalfAction::Create`]).
    pub action: OnBehalfAction,
    /// The SHA-256 commitment hash.
    pub commitment: [u8; 32],
    /// The protocol version.
    pub protocol_version: u8,
    /// The operator account authorized to act on behalf.
    pub operator: AccountId,
    /// Nonce for replay protection.
    pub nonce: u64,
}

/// Signed payload for updating an ATS entry on behalf of an owner.
#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Debug)]
pub struct UpdateOnBehalfPayload<AccountId> {
    /// The action type (must be [`OnBehalfAction::Update`]).
    pub action: OnBehalfAction,
    /// The ATS identifier to update.
    pub ats_id: u64,
    /// The SHA-256 commitment hash.
    pub commitment: [u8; 32],
    /// The protocol version.
    pub protocol_version: u8,
    /// The operator account authorized to act on behalf.
    pub operator: AccountId,
    /// Nonce for replay protection.
    pub nonce: u64,
}

/// Signed payload for revoking an ATS entry on behalf of an owner.
#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Debug)]
pub struct RevokeOnBehalfPayload<AccountId> {
    /// The action type (must be [`OnBehalfAction::Revoke`]).
    pub action: OnBehalfAction,
    /// The ATS identifier to revoke.
    pub ats_id: u64,
    /// The operator account authorized to act on behalf.
    pub operator: AccountId,
    /// Nonce for replay protection.
    pub nonce: u64,
}
