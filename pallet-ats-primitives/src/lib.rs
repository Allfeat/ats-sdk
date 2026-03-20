//! Shared primitive types for the Allfeat Timestamp Service (ATS) pallet.
//!
//! This crate contains off-chain payload types used for on-behalf (delegate/operator)
//! operations. These types only depend on `parity-scale-codec` and `scale-info`,
//! making them suitable for use in backends without pulling in the full Substrate stack.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use scale_info::TypeInfo;

/// ATS identifier type (auto-incremented `u64`).
pub type AtsId = u64;

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
