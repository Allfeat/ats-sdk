//! Migration from v0 to v1: adds `depositor` field to `AtsRecord` and `VersionRecord`.
//!
//! For existing records, `depositor` is set to the `owner` of the ATS entry, since all
//! pre-v1 deposits were paid by the owner directly.
//!
//! **Note:** This migration writes v1-format records. A subsequent v1->v2 migration
//! (`MigrateV1ToV2`) will convert them to the current v2 format.

use crate::*;
use alloc::vec::Vec;
use codec::{Decode, Encode};
use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade, weights::Weight};
use frame_system::pallet_prelude::BlockNumberFor;

#[cfg(feature = "try-runtime")]
use frame_support::ensure;

/// Old ATS record format (v0) without `depositor` field.
#[derive(Encode, Decode)]
struct V0AtsRecord<AccountId, BlockNumber, Balance> {
    owner: AccountId,
    created_at: BlockNumber,
    version_count: u32,
    base_deposit: Balance,
}

/// Old version record format (v0) without `AccountId` generic and `depositor` field.
#[derive(Encode, Decode)]
struct V0VersionRecord<BlockNumber, Balance> {
    commitment: [u8; 32],
    protocol_version: u8,
    created_at: BlockNumber,
    deposit: Balance,
}

/// V1 ATS record format (with `depositor` and `base_deposit`).
#[derive(Encode, Decode)]
struct V1AtsRecord<AccountId, BlockNumber, Balance> {
    owner: AccountId,
    depositor: AccountId,
    created_at: BlockNumber,
    version_count: u32,
    base_deposit: Balance,
}

/// V1 version record format (with `depositor` and `deposit`).
#[derive(Encode, Decode)]
struct V1VersionRecord<AccountId, BlockNumber, Balance> {
    commitment: [u8; 32],
    protocol_version: u8,
    depositor: AccountId,
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
                "Migration v0->v1 skipped: on-chain version is {:?}",
                on_chain_version
            );
            return Weight::zero();
        }

        log::info!(target: "pallet-ats", "Running migration v0->v1");

        let mut ats_count: u64 = 0;
        let mut version_count: u64 = 0;

        type V0AtsRecordOf<T> = V0AtsRecord<
            <T as frame_system::Config>::AccountId,
            BlockNumberFor<T>,
            pallet::BalanceOf<T>,
        >;

        // Phase 1: Migrate AtsRegistry and collect (ats_id, owner, version_count)
        let mut ats_info: Vec<(AtsId, T::AccountId, u32)> = Vec::new();

        let raw_entries: Vec<(AtsId, V0AtsRecordOf<T>)> =
            frame_support::storage::migration::storage_key_iter::<
                AtsId,
                V0AtsRecordOf<T>,
                frame_support::Blake2_128Concat,
            >(pallet::Pallet::<T>::name().as_bytes(), b"AtsRegistry")
            .collect();

        for (ats_id, old_record) in raw_entries {
            let new_record = V1AtsRecord {
                owner: old_record.owner.clone(),
                depositor: old_record.owner.clone(),
                created_at: old_record.created_at,
                version_count: old_record.version_count,
                base_deposit: old_record.base_deposit,
            };
            // Write v1 format directly to storage
            let key = pallet::AtsRegistry::<T>::hashed_key_for(ats_id);
            frame_support::storage::unhashed::put_raw(&key, &new_record.encode());
            ats_info.push((ats_id, old_record.owner, old_record.version_count));
            ats_count += 1;
        }

        // Phase 2: Migrate AtsVersions
        for (ats_id, owner, count) in &ats_info {
            for v in 0..*count {
                let key = pallet::AtsVersions::<T>::hashed_key_for(ats_id, v);
                if let Some(old_version) = frame_support::storage::unhashed::get::<
                    V0VersionRecord<BlockNumberFor<T>, pallet::BalanceOf<T>>,
                >(&key)
                {
                    let new_version = V1VersionRecord {
                        commitment: old_version.commitment,
                        protocol_version: old_version.protocol_version,
                        depositor: owner.clone(),
                        created_at: old_version.created_at,
                        deposit: old_version.deposit,
                    };
                    frame_support::storage::unhashed::put_raw(&key, &new_version.encode());
                    version_count += 1;
                }
            }
        }

        // Update storage version
        StorageVersion::new(1).put::<pallet::Pallet<T>>();

        log::info!(
            target: "pallet-ats",
            "Migration v0->v1 complete: migrated {} ATS records and {} version records",
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
        // Count using raw iteration since current types don't match v0
        let count = frame_support::storage::migration::storage_key_iter::<
            AtsId,
            V0AtsRecord<T::AccountId, BlockNumberFor<T>, pallet::BalanceOf<T>>,
            frame_support::Blake2_128Concat,
        >(pallet::Pallet::<T>::name().as_bytes(), b"AtsRegistry")
        .count() as u64;
        log::info!(target: "pallet-ats", "v0->v1 pre_upgrade: {} ATS records found", count);
        Ok(count.encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
        let old_count = u64::decode(&mut &state[..]).map_err(|_| {
            sp_runtime::TryRuntimeError::Other("Failed to decode pre_upgrade state")
        })?;

        // Count v1 records via raw iteration
        let new_count = frame_support::storage::migration::storage_key_iter::<
            AtsId,
            V1AtsRecord<T::AccountId, BlockNumberFor<T>, pallet::BalanceOf<T>>,
            frame_support::Blake2_128Concat,
        >(pallet::Pallet::<T>::name().as_bytes(), b"AtsRegistry")
        .count() as u64;

        ensure!(
            old_count == new_count,
            sp_runtime::TryRuntimeError::Other("ATS record count mismatch after migration")
        );

        // Verify all records have depositor set
        for (_, record) in frame_support::storage::migration::storage_key_iter::<
            AtsId,
            V1AtsRecord<T::AccountId, BlockNumberFor<T>, pallet::BalanceOf<T>>,
            frame_support::Blake2_128Concat,
        >(pallet::Pallet::<T>::name().as_bytes(), b"AtsRegistry")
        {
            ensure!(
                record.depositor == record.owner,
                sp_runtime::TryRuntimeError::Other(
                    "depositor should equal owner after v0->v1 migration"
                )
            );
        }

        log::info!(
            target: "pallet-ats",
            "v0->v1 post_upgrade: verified {} records",
            new_count,
        );
        Ok(())
    }
}
