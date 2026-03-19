//! Migration from v0 to v1: adds `depositor` field to `AtsRecord` and `VersionRecord`.
//!
//! For existing records, `depositor` is set to the `owner` of the ATS entry, since all
//! pre-v1 deposits were paid by the owner directly.

use crate::*;
use alloc::vec::Vec;
use codec::{Decode, Encode};
use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade, weights::Weight};
use frame_system::pallet_prelude::BlockNumberFor;

#[cfg(feature = "try-runtime")]
use frame_support::ensure;

/// Old ATS record format (v0) without `depositor` field.
#[derive(Encode, Decode)]
struct OldAtsRecord<AccountId, BlockNumber, Balance> {
    owner: AccountId,
    created_at: BlockNumber,
    version_count: u32,
    base_deposit: Balance,
}

/// Old version record format (v0) without `AccountId` generic and `depositor` field.
#[derive(Encode, Decode)]
struct OldVersionRecord<BlockNumber, Balance> {
    commitment: [u8; 32],
    protocol_version: u8,
    created_at: BlockNumber,
    deposit: Balance,
}

/// Migrate storage from v0 to v1.
///
/// Adds `depositor` field to both `AtsRecord` and `VersionRecord`, setting it to the
/// owner of the ATS entry for all existing records.
pub struct MigrateV0ToV1<T>(core::marker::PhantomData<T>);

impl<T: crate::pallet::Config> OnRuntimeUpgrade for MigrateV0ToV1<T> {
    fn on_runtime_upgrade() -> Weight {
        let on_chain_version = pallet::Pallet::<T>::on_chain_storage_version();
        if on_chain_version != 0 {
            log::info!(
                target: "pallet-ats",
                "Migration v0→v1 skipped: on-chain version is {:?}",
                on_chain_version
            );
            return Weight::zero();
        }

        log::info!(target: "pallet-ats", "Running migration v0→v1");

        let mut ats_count: u64 = 0;
        let mut version_count: u64 = 0;

        // Phase 1: Migrate AtsRegistry and collect (ats_id, owner, version_count)
        let mut ats_info: Vec<(AtsId, T::AccountId, u32)> = Vec::new();

        type OldAtsRecordOf<T> = OldAtsRecord<
            <T as frame_system::Config>::AccountId,
            BlockNumberFor<T>,
            pallet::BalanceOf<T>,
        >;

        let raw_entries: Vec<(AtsId, OldAtsRecordOf<T>)> =
            frame_support::storage::migration::storage_key_iter::<
                AtsId,
                OldAtsRecordOf<T>,
                frame_support::Blake2_128Concat,
            >(pallet::Pallet::<T>::name().as_bytes(), b"AtsRegistry")
            .collect();

        for (ats_id, old_record) in raw_entries {
            let new_record = AtsRecord {
                owner: old_record.owner.clone(),
                depositor: old_record.owner.clone(),
                created_at: old_record.created_at,
                version_count: old_record.version_count,
                base_deposit: old_record.base_deposit,
            };
            pallet::AtsRegistry::<T>::insert(ats_id, new_record);
            ats_info.push((ats_id, old_record.owner, old_record.version_count));
            ats_count += 1;
        }

        // Phase 2: Migrate AtsVersions
        // We use `unhashed::get` with the full key from `hashed_key_for` because
        // `get_storage_value` would double the pallet/storage prefix.
        for (ats_id, owner, count) in &ats_info {
            for v in 0..*count {
                let key = pallet::AtsVersions::<T>::hashed_key_for(ats_id, v);
                if let Some(old_version) = frame_support::storage::unhashed::get::<
                    OldVersionRecord<BlockNumberFor<T>, pallet::BalanceOf<T>>,
                >(&key)
                {
                    let new_version = VersionRecord {
                        commitment: old_version.commitment,
                        protocol_version: old_version.protocol_version,
                        depositor: owner.clone(),
                        created_at: old_version.created_at,
                        deposit: old_version.deposit,
                    };
                    pallet::AtsVersions::<T>::insert(ats_id, v, new_version);
                    version_count += 1;
                }
            }
        }

        // Update storage version
        StorageVersion::new(1).put::<pallet::Pallet<T>>();

        log::info!(
            target: "pallet-ats",
            "Migration v0→v1 complete: migrated {} ATS records and {} version records",
            ats_count,
            version_count,
        );

        // Weight: reads + writes for each record
        T::DbWeight::get().reads_writes(
            ats_count + version_count,
            ats_count + version_count + 1, // +1 for storage version
        )
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
        let count = pallet::AtsRegistry::<T>::iter().count() as u64;
        log::info!(target: "pallet-ats", "v0→v1 pre_upgrade: {} ATS records found", count);
        Ok(count.encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
        let old_count = u64::decode(&mut &state[..]).map_err(|_| {
            sp_runtime::TryRuntimeError::Other("Failed to decode pre_upgrade state")
        })?;
        let new_count = pallet::AtsRegistry::<T>::iter().count() as u64;
        ensure!(
            old_count == new_count,
            sp_runtime::TryRuntimeError::Other("ATS record count mismatch after migration")
        );

        // Verify all records have depositor set
        for (_, record) in pallet::AtsRegistry::<T>::iter() {
            ensure!(
                record.depositor == record.owner,
                sp_runtime::TryRuntimeError::Other(
                    "depositor should equal owner after v0→v1 migration"
                )
            );
        }

        log::info!(
            target: "pallet-ats",
            "v0→v1 post_upgrade: verified {} records",
            new_count,
        );
        Ok(())
    }
}
