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
