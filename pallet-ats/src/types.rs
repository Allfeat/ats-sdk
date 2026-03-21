use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::{BoundedVec, ConstU32};
use scale_info::TypeInfo;

/// Maximum number of unique depositors per ATS entry.
///
/// In practice this is 1-2 (owner + possibly one operator). The bound of 16
/// is generous to cover exotic multi-operator scenarios.
pub const MAX_UNIQUE_DEPOSITORS: u32 = 16;

/// Lightweight version record (no deposit/depositor info).
///
/// Deposit tracking is aggregated in [`AtsRecord::deposits`] instead.
#[derive(Clone, Encode, Decode, MaxEncodedLen, TypeInfo, PartialEq, Eq, Debug)]
pub struct VersionInfo<BlockNumber> {
    /// SHA-256 commitment hash (32 bytes).
    pub commitment: [u8; 32],
    /// Protocol version used to generate the commitment.
    pub protocol_version: u8,
    /// Block number when this version was created.
    pub created_at: BlockNumber,
}

/// Aggregated deposit entry for a single depositor within an ATS entry.
///
/// Multiple operations (base deposit + version deposits) by the same depositor
/// are summed into a single entry, enabling O(d) release on revocation instead
/// of O(v) where d = unique depositors and v = total versions.
#[derive(Clone, Encode, Decode, MaxEncodedLen, TypeInfo, PartialEq, Eq, Debug)]
pub struct DepositEntry<AccountId, Balance> {
    /// Account that paid the deposit.
    pub depositor: AccountId,
    /// Total amount held from this depositor for this ATS entry.
    pub amount: Balance,
}

/// ATS registry record with aggregated deposits per depositor.
#[derive(Clone, Encode, Decode, MaxEncodedLen, TypeInfo, PartialEq, Eq, Debug)]
pub struct AtsRecord<AccountId, BlockNumber, Balance> {
    /// Account that owns this ATS entry.
    pub owner: AccountId,
    /// Block number when this ATS entry was created.
    pub created_at: BlockNumber,
    /// Number of versions (including the initial version 0).
    pub version_count: u32,
    /// Aggregated deposits by depositor. Each depositor appears at most once.
    pub deposits: BoundedVec<DepositEntry<AccountId, Balance>, ConstU32<MAX_UNIQUE_DEPOSITORS>>,
}
