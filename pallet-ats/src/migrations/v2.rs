//! Migration from v1 to v2: aggregate deposits per depositor and slim down version records.
//!
//! - `AtsRecord` loses `depositor` and `base_deposit`, gains `deposits: BoundedVec<DepositEntry>`.
//! - `VersionRecord` (v1) becomes `VersionInfo` (v2) — drops `depositor` and `deposit` fields.
//! - `HoldReason::VersionDeposit` is removed; everything uses `HoldReason::AtsDeposit`.
//!
//! **Note:** this migration only transforms storage layout. It does **not** re-issue holds
//! because the pallet has not been deployed on-chain yet. If the pallet were already live,
//! a hold migration (release old holds, re-hold under single reason) would be needed.

use crate::*;
use alloc::vec::Vec;
use codec::{Decode, Encode};
use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade, weights::Weight};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::Saturating;

#[cfg(feature = "try-runtime")]
use frame_support::ensure;

/// V1 ATS record format (with `depositor` and `base_deposit`).
#[derive(Encode, Decode)]
struct OldAtsRecord<AccountId, BlockNumber, Balance> {
    owner: AccountId,
    depositor: AccountId,
    created_at: BlockNumber,
    version_count: u32,
    base_deposit: Balance,
}

/// V1 version record format (with `depositor` and `deposit`).
#[derive(Encode, Decode)]
struct OldVersionRecord<AccountId, BlockNumber, Balance> {
    commitment: [u8; 32],
    protocol_version: u8,
    depositor: AccountId,
    created_at: BlockNumber,
    deposit: Balance,
}

type OldAtsRecordOf<T> =
    OldAtsRecord<<T as frame_system::Config>::AccountId, BlockNumberFor<T>, pallet::BalanceOf<T>>;

type OldVersionRecordOf<T> = OldVersionRecord<
    <T as frame_system::Config>::AccountId,
    BlockNumberFor<T>,
    pallet::BalanceOf<T>,
>;

/// Migrate storage from v1 to v2.
///
/// Aggregates per-version deposits into per-depositor entries in `AtsRecord`,
/// and strips deposit info from version records.
pub struct MigrateV1ToV2<T>(core::marker::PhantomData<T>);

impl<T: crate::pallet::Config> OnRuntimeUpgrade for MigrateV1ToV2<T> {
    fn on_runtime_upgrade() -> Weight {
        let on_chain_version = pallet::Pallet::<T>::on_chain_storage_version();
        if on_chain_version != 1 {
            log::info!(
                target: "pallet-ats",
                "Migration v1->v2 skipped: on-chain version is {:?}",
                on_chain_version
            );
            return Weight::zero();
        }

        log::info!(target: "pallet-ats", "Running migration v1->v2");

        let mut ats_count: u64 = 0;
        let mut version_count: u64 = 0;

        // Collect old ATS records
        let raw_entries: Vec<(AtsId, OldAtsRecordOf<T>)> =
            frame_support::storage::migration::storage_key_iter::<
                AtsId,
                OldAtsRecordOf<T>,
                frame_support::Blake2_128Concat,
            >(pallet::Pallet::<T>::name().as_bytes(), b"AtsRegistry")
            .collect();

        for (ats_id, old_record) in raw_entries {
            // Aggregate deposits: start with the base deposit from the ATS record's depositor
            let mut deposit_map: Vec<DepositEntry<T::AccountId, pallet::BalanceOf<T>>> = Vec::new();

            // Add base deposit
            Self::add_deposit(
                &mut deposit_map,
                old_record.depositor.clone(),
                old_record.base_deposit,
            );

            // Process each version record to aggregate version deposits
            for v in 0..old_record.version_count {
                let key = pallet::AtsVersions::<T>::hashed_key_for(ats_id, v);
                if let Some(old_version) =
                    frame_support::storage::unhashed::get::<OldVersionRecordOf<T>>(&key)
                {
                    Self::add_deposit(&mut deposit_map, old_version.depositor, old_version.deposit);

                    // Write slimmed-down version record
                    let new_version = VersionInfo {
                        commitment: old_version.commitment,
                        protocol_version: old_version.protocol_version,
                        created_at: old_version.created_at,
                    };
                    pallet::AtsVersions::<T>::insert(ats_id, v, new_version);
                    version_count += 1;
                }
            }

            // Build BoundedVec from deposit_map
            let deposits: BoundedVec<
                DepositEntry<T::AccountId, pallet::BalanceOf<T>>,
                ConstU32<MAX_UNIQUE_DEPOSITORS>,
            > = deposit_map.try_into().expect(
                "at most version_count+1 unique depositors, bounded by MAX_UNIQUE_DEPOSITORS; qed",
            );

            let new_record = AtsRecord {
                owner: old_record.owner,
                created_at: old_record.created_at,
                version_count: old_record.version_count,
                deposits,
            };
            pallet::AtsRegistry::<T>::insert(ats_id, new_record);
            ats_count += 1;
        }

        // Update storage version
        StorageVersion::new(2).put::<pallet::Pallet<T>>();

        log::info!(
            target: "pallet-ats",
            "Migration v1->v2 complete: migrated {} ATS records and {} version records",
            ats_count,
            version_count,
        );

        T::DbWeight::get().reads_writes(
            ats_count + version_count,
            ats_count + version_count + 1, // +1 for storage version
        )
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
        let count = pallet::AtsRegistry::<T>::iter().count() as u64;
        log::info!(target: "pallet-ats", "v1->v2 pre_upgrade: {} ATS records found", count);
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

        // Verify all records have non-empty deposits
        for (_, record) in pallet::AtsRegistry::<T>::iter() {
            ensure!(
                !record.deposits.is_empty(),
                sp_runtime::TryRuntimeError::Other(
                    "deposits should be non-empty after v1->v2 migration"
                )
            );
        }

        log::info!(
            target: "pallet-ats",
            "v1->v2 post_upgrade: verified {} records",
            new_count,
        );
        Ok(())
    }
}

impl<T: crate::pallet::Config> MigrateV1ToV2<T> {
    /// Add a deposit amount to the deposit map, merging with an existing entry if present.
    fn add_deposit(
        map: &mut Vec<DepositEntry<T::AccountId, pallet::BalanceOf<T>>>,
        depositor: T::AccountId,
        amount: pallet::BalanceOf<T>,
    ) {
        if let Some(entry) = map.iter_mut().find(|e| e.depositor == depositor) {
            entry.amount = entry.amount.saturating_add(amount);
        } else {
            map.push(DepositEntry { depositor, amount });
        }
    }
}
