use crate::mock::*;
use crate::*;
use codec::Encode;
use frame_support::pallet_prelude::StorageVersion;
use frame_support::traits::GetStorageVersion;
use frame_support::{assert_noop, assert_ok};

fn commitment(byte: u8) -> [u8; 32] {
    [byte; 32]
}

// ── Payload builder helpers ────────────────────────────────────────────────

fn create_payload(
    commitment: [u8; 32],
    protocol_version: u8,
    operator: u64,
    nonce: u64,
) -> CreateOnBehalfPayload<u64> {
    CreateOnBehalfPayload {
        action: OnBehalfAction::Create,
        commitment,
        protocol_version,
        operator,
        nonce,
    }
}

fn update_payload(
    ats_id: u64,
    commitment: [u8; 32],
    protocol_version: u8,
    operator: u64,
    nonce: u64,
) -> UpdateOnBehalfPayload<u64> {
    UpdateOnBehalfPayload {
        action: OnBehalfAction::Update,
        ats_id,
        commitment,
        protocol_version,
        operator,
        nonce,
    }
}

fn revoke_payload(ats_id: u64, operator: u64, nonce: u64) -> RevokeOnBehalfPayload<u64> {
    RevokeOnBehalfPayload {
        action: OnBehalfAction::Revoke,
        ats_id,
        operator,
        nonce,
    }
}

/// Helper: find the total deposit amount for a given depositor in an ATS record.
fn deposit_for(record: &AtsRecordOf<Test>, who: u64) -> u64 {
    record
        .deposits
        .iter()
        .find(|e| e.depositor == who)
        .map(|e| e.amount)
        .unwrap_or(0)
}

// ── create ──────────────────────────────────────────────────────────────────

#[test]
fn create_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(1), 1));

        // Check ATS record
        let record = AtsRegistry::<Test>::get(0).expect("ATS should exist");
        assert_eq!(record.owner, ALICE);
        assert_eq!(record.version_count, 1);
        // Single deposit entry: base(100) + version(10) = 110
        assert_eq!(record.deposits.len(), 1);
        assert_eq!(deposit_for(&record, ALICE), 110);

        // Check version 0
        let version = AtsVersions::<Test>::get(0, 0).expect("version should exist");
        assert_eq!(version.commitment, commitment(1));
        assert_eq!(version.protocol_version, 1);

        // Check owner index
        let ids = OwnerIndex::<Test>::get(ALICE);
        assert_eq!(ids.into_inner(), vec![0]);

        // Check next ID incremented
        assert_eq!(NextAtsId::<Test>::get(), 1);

        // Check event
        System::assert_last_event(
            Event::<Test>::AtsCreated {
                ats_id: 0,
                owner: ALICE,
                commitment: commitment(1),
                protocol_version: 1,
                operator: None,
            }
            .into(),
        );
    });
}

#[test]
fn create_invalid_protocol_version() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Ats::create(RuntimeOrigin::signed(ALICE), commitment(1), 0),
            Error::<Test>::InvalidProtocolVersion
        );
    });
}

#[test]
fn create_insufficient_balance() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Ats::create(RuntimeOrigin::signed(999), commitment(1), 1),
            sp_runtime::TokenError::FundsUnavailable
        );
    });
}

#[test]
fn create_max_ats_per_account() {
    new_test_ext().execute_with(|| {
        // MaxAtsPerAccount = 5
        for i in 0..5 {
            assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(i), 1));
        }
        // 6th should fail
        assert_noop!(
            Ats::create(RuntimeOrigin::signed(ALICE), commitment(5), 1),
            Error::<Test>::MaxAtsPerAccountReached
        );
    });
}

// ── update ──────────────────────────────────────────────────────────────────

#[test]
fn update_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(1), 1));
        assert_ok!(Ats::update(
            RuntimeOrigin::signed(ALICE),
            0,
            commitment(2),
            1
        ));

        let record = AtsRegistry::<Test>::get(0).unwrap();
        assert_eq!(record.version_count, 2);
        // Deposits aggregated: base(100) + 2*version(10) = 120
        assert_eq!(deposit_for(&record, ALICE), 120);

        let v1 = AtsVersions::<Test>::get(0, 1).unwrap();
        assert_eq!(v1.commitment, commitment(2));

        System::assert_last_event(
            Event::<Test>::AtsUpdated {
                ats_id: 0,
                version: 1,
                commitment: commitment(2),
                protocol_version: 1,
                operator: None,
            }
            .into(),
        );
    });
}

#[test]
fn update_not_owner() {
    new_test_ext().execute_with(|| {
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(1), 1));
        assert_noop!(
            Ats::update(RuntimeOrigin::signed(BOB), 0, commitment(2), 1),
            Error::<Test>::NotOwner
        );
    });
}

#[test]
fn update_ats_not_found() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Ats::update(RuntimeOrigin::signed(ALICE), 999, commitment(2), 1),
            Error::<Test>::AtsNotFound
        );
    });
}

#[test]
fn update_max_versions() {
    new_test_ext().execute_with(|| {
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(0), 1));
        // MaxVersionsPerAts = 10, already have version 0, can add 9 more
        for i in 1..10u8 {
            assert_ok!(Ats::update(
                RuntimeOrigin::signed(ALICE),
                0,
                commitment(i),
                1
            ));
        }
        // 11th version should fail
        assert_noop!(
            Ats::update(RuntimeOrigin::signed(ALICE), 0, commitment(10), 1),
            Error::<Test>::MaxVersionsReached
        );
    });
}

#[test]
fn update_preserves_version_history() {
    new_test_ext().execute_with(|| {
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(0), 1));
        assert_ok!(Ats::update(
            RuntimeOrigin::signed(ALICE),
            0,
            commitment(1),
            1
        ));
        assert_ok!(Ats::update(
            RuntimeOrigin::signed(ALICE),
            0,
            commitment(2),
            1
        ));

        // All three versions should exist
        for i in 0..3u8 {
            let v = AtsVersions::<Test>::get(0, u32::from(i)).unwrap();
            assert_eq!(v.commitment, commitment(i));
        }
    });
}

#[test]
fn update_invalid_protocol_version() {
    new_test_ext().execute_with(|| {
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(1), 1));
        assert_noop!(
            Ats::update(RuntimeOrigin::signed(ALICE), 0, commitment(2), 0),
            Error::<Test>::InvalidProtocolVersion
        );
    });
}

// ── revoke ──────────────────────────────────────────────────────────────────

#[test]
fn revoke_single_version() {
    new_test_ext().execute_with(|| {
        let balance_before = Balances::free_balance(ALICE);
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(1), 1));
        assert_ok!(Ats::revoke(RuntimeOrigin::signed(ALICE), 0));

        // ATS should be gone
        assert!(AtsRegistry::<Test>::get(0).is_none());
        assert!(AtsVersions::<Test>::get(0, 0).is_none());

        // Owner index should be empty
        assert!(OwnerIndex::<Test>::get(ALICE).is_empty());

        // All deposits returned
        assert_eq!(Balances::free_balance(ALICE), balance_before);

        System::assert_last_event(
            Event::<Test>::AtsRevoked {
                ats_id: 0,
                owner: ALICE,
                operator: None,
            }
            .into(),
        );
    });
}

#[test]
fn revoke_multiple_versions() {
    new_test_ext().execute_with(|| {
        let balance_before = Balances::free_balance(ALICE);
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(0), 1));
        assert_ok!(Ats::update(
            RuntimeOrigin::signed(ALICE),
            0,
            commitment(1),
            1
        ));
        assert_ok!(Ats::update(
            RuntimeOrigin::signed(ALICE),
            0,
            commitment(2),
            1
        ));
        assert_ok!(Ats::revoke(RuntimeOrigin::signed(ALICE), 0));

        // All gone
        assert!(AtsRegistry::<Test>::get(0).is_none());
        for v in 0..3 {
            assert!(AtsVersions::<Test>::get(0, v).is_none());
        }

        // All deposits returned (base + 3 * version)
        assert_eq!(Balances::free_balance(ALICE), balance_before);
    });
}

#[test]
fn revoke_not_owner() {
    new_test_ext().execute_with(|| {
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(1), 1));
        assert_noop!(
            Ats::revoke(RuntimeOrigin::signed(BOB), 0),
            Error::<Test>::NotOwner
        );
    });
}

#[test]
fn revoke_not_found() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Ats::revoke(RuntimeOrigin::signed(ALICE), 999),
            Error::<Test>::AtsNotFound
        );
    });
}

#[test]
fn revoke_double_revoke() {
    new_test_ext().execute_with(|| {
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(1), 1));
        assert_ok!(Ats::revoke(RuntimeOrigin::signed(ALICE), 0));
        assert_noop!(
            Ats::revoke(RuntimeOrigin::signed(ALICE), 0),
            Error::<Test>::AtsNotFound
        );
    });
}

#[test]
fn revoke_deposits_returned() {
    new_test_ext().execute_with(|| {
        let balance_before = Balances::free_balance(ALICE);
        // base=100 + 5*version=50 = 150 total
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(0), 1));
        for i in 1..5u8 {
            assert_ok!(Ats::update(
                RuntimeOrigin::signed(ALICE),
                0,
                commitment(i),
                1
            ));
        }

        let balance_after_creates = Balances::free_balance(ALICE);
        assert_eq!(balance_before - balance_after_creates, 100 + 5 * 10); // 150 held

        assert_ok!(Ats::revoke(RuntimeOrigin::signed(ALICE), 0));
        assert_eq!(Balances::free_balance(ALICE), balance_before);
    });
}

// ── owner index ─────────────────────────────────────────────────────────────

#[test]
fn owner_index_updated_on_create() {
    new_test_ext().execute_with(|| {
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(0), 1));
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(1), 1));
        let ids = OwnerIndex::<Test>::get(ALICE);
        assert_eq!(ids.into_inner(), vec![0, 1]);
    });
}

#[test]
fn owner_index_updated_on_revoke() {
    new_test_ext().execute_with(|| {
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(0), 1));
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(1), 1));
        assert_ok!(Ats::revoke(RuntimeOrigin::signed(ALICE), 0));
        let ids = OwnerIndex::<Test>::get(ALICE);
        assert_eq!(ids.into_inner(), vec![1]);
    });
}

#[test]
fn owner_index_multiple_owners() {
    new_test_ext().execute_with(|| {
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(0), 1));
        assert_ok!(Ats::create(RuntimeOrigin::signed(BOB), commitment(1), 1));

        assert_eq!(OwnerIndex::<Test>::get(ALICE).into_inner(), vec![0]);
        assert_eq!(OwnerIndex::<Test>::get(BOB).into_inner(), vec![1]);
    });
}

#[test]
fn auto_increment_id() {
    new_test_ext().execute_with(|| {
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(0), 1));
        assert_ok!(Ats::create(RuntimeOrigin::signed(BOB), commitment(1), 1));
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(2), 1));

        assert_eq!(NextAtsId::<Test>::get(), 3);
        assert!(AtsRegistry::<Test>::get(0).is_some());
        assert!(AtsRegistry::<Test>::get(1).is_some());
        assert!(AtsRegistry::<Test>::get(2).is_some());
    });
}

#[test]
fn create_and_immediately_revoke() {
    new_test_ext().execute_with(|| {
        let balance_before = Balances::free_balance(ALICE);
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(1), 1));
        assert_ok!(Ats::revoke(RuntimeOrigin::signed(ALICE), 0));
        assert_eq!(Balances::free_balance(ALICE), balance_before);
        assert!(AtsRegistry::<Test>::get(0).is_none());
    });
}

// ── on-behalf: create ──────────────────────────────────────────────────────

#[test]
fn create_on_behalf_works() {
    new_test_ext().execute_with(|| {
        let alice_bal_before = Balances::free_balance(ALICE);
        let bob_bal_before = Balances::free_balance(BOB);

        let payload = create_payload(commitment(1), 1, BOB, 0);
        assert_ok!(Ats::create_on_behalf(
            RuntimeOrigin::signed(BOB),
            ALICE,
            commitment(1),
            1,
            0,
            sign(ALICE, &payload),
        ));

        // ATS record: owner=ALICE, deposits from BOB
        let record = AtsRegistry::<Test>::get(0).expect("ATS should exist");
        assert_eq!(record.owner, ALICE);
        assert_eq!(record.deposits.len(), 1);
        assert_eq!(deposit_for(&record, BOB), 110); // base(100) + version(10)

        // Owner index tracks ALICE
        assert_eq!(OwnerIndex::<Test>::get(ALICE).into_inner(), vec![0]);

        // BOB paid deposits (100 base + 10 version = 110)
        assert_eq!(Balances::free_balance(BOB), bob_bal_before - 110);
        // ALICE balance unchanged
        assert_eq!(Balances::free_balance(ALICE), alice_bal_before);

        // Event includes operator
        System::assert_last_event(
            Event::<Test>::AtsCreated {
                ats_id: 0,
                owner: ALICE,
                commitment: commitment(1),
                protocol_version: 1,
                operator: Some(BOB),
            }
            .into(),
        );
    });
}

#[test]
fn create_on_behalf_invalid_signature() {
    new_test_ext().execute_with(|| {
        let payload = create_payload(commitment(1), 1, BOB, 0);
        assert_noop!(
            Ats::create_on_behalf(
                RuntimeOrigin::signed(BOB),
                ALICE,
                commitment(1),
                1,
                0,
                sign(BOB, &payload), // wrong signer — should be ALICE
            ),
            Error::<Test>::InvalidSignature
        );
    });
}

#[test]
fn create_on_behalf_invalid_nonce() {
    new_test_ext().execute_with(|| {
        let payload = create_payload(commitment(1), 1, BOB, 1);
        assert_noop!(
            Ats::create_on_behalf(
                RuntimeOrigin::signed(BOB),
                ALICE,
                commitment(1),
                1,
                1, // wrong nonce — should be 0
                sign(ALICE, &payload),
            ),
            Error::<Test>::InvalidNonce
        );
    });
}

#[test]
fn create_on_behalf_nonce_increments() {
    new_test_ext().execute_with(|| {
        assert_eq!(OnBehalfNonce::<Test>::get(ALICE), 0);

        let payload = create_payload(commitment(1), 1, BOB, 0);
        assert_ok!(Ats::create_on_behalf(
            RuntimeOrigin::signed(BOB),
            ALICE,
            commitment(1),
            1,
            0,
            sign(ALICE, &payload),
        ));

        assert_eq!(OnBehalfNonce::<Test>::get(ALICE), 1);
    });
}

#[test]
fn create_on_behalf_replay_fails() {
    new_test_ext().execute_with(|| {
        let payload = create_payload(commitment(1), 1, BOB, 0);
        assert_ok!(Ats::create_on_behalf(
            RuntimeOrigin::signed(BOB),
            ALICE,
            commitment(1),
            1,
            0,
            sign(ALICE, &payload),
        ));

        // Replay with same nonce fails
        let payload2 = create_payload(commitment(2), 1, BOB, 0);
        assert_noop!(
            Ats::create_on_behalf(
                RuntimeOrigin::signed(BOB),
                ALICE,
                commitment(2),
                1,
                0, // stale nonce
                sign(ALICE, &payload2),
            ),
            Error::<Test>::InvalidNonce
        );
    });
}

#[test]
fn create_on_behalf_operator_insufficient_balance() {
    new_test_ext().execute_with(|| {
        // Operator 999 has no funds
        let payload = create_payload(commitment(1), 1, 999, 0);
        assert_noop!(
            Ats::create_on_behalf(
                RuntimeOrigin::signed(999),
                ALICE,
                commitment(1),
                1,
                0,
                sign(ALICE, &payload),
            ),
            sp_runtime::TokenError::FundsUnavailable
        );
    });
}

#[test]
fn create_on_behalf_respects_max_ats_per_account() {
    new_test_ext().execute_with(|| {
        // Fill up ALICE's quota with direct creates
        for i in 0..5u8 {
            assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(i), 1));
        }

        // On-behalf create should also respect ALICE's limit
        let payload = create_payload(commitment(5), 1, BOB, 0);
        assert_noop!(
            Ats::create_on_behalf(
                RuntimeOrigin::signed(BOB),
                ALICE,
                commitment(5),
                1,
                0,
                sign(ALICE, &payload),
            ),
            Error::<Test>::MaxAtsPerAccountReached
        );
    });
}

#[test]
fn create_on_behalf_deposits_held_from_operator() {
    new_test_ext().execute_with(|| {
        let bob_before = Balances::free_balance(BOB);
        let alice_before = Balances::free_balance(ALICE);

        let payload = create_payload(commitment(1), 1, BOB, 0);
        assert_ok!(Ats::create_on_behalf(
            RuntimeOrigin::signed(BOB),
            ALICE,
            commitment(1),
            1,
            0,
            sign(ALICE, &payload),
        ));

        // BOB's balance decreased by base + version deposit
        assert_eq!(Balances::free_balance(BOB), bob_before - 110);
        // ALICE's balance unchanged
        assert_eq!(Balances::free_balance(ALICE), alice_before);
    });
}

// ── on-behalf: update ──────────────────────────────────────────────────────

#[test]
fn update_on_behalf_works() {
    new_test_ext().execute_with(|| {
        // Create directly by ALICE
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(1), 1));

        // Update on behalf by BOB
        let payload = update_payload(0, commitment(2), 1, BOB, 0);
        assert_ok!(Ats::update_on_behalf(
            RuntimeOrigin::signed(BOB),
            ALICE,
            0,
            commitment(2),
            1,
            0,
            sign(ALICE, &payload),
        ));

        let record = AtsRegistry::<Test>::get(0).unwrap();
        assert_eq!(record.version_count, 2);
        // Two depositors: ALICE (base+v0=110) and BOB (v1=10)
        assert_eq!(record.deposits.len(), 2);
        assert_eq!(deposit_for(&record, ALICE), 110);
        assert_eq!(deposit_for(&record, BOB), 10);

        let v1 = AtsVersions::<Test>::get(0, 1).unwrap();
        assert_eq!(v1.commitment, commitment(2));

        System::assert_last_event(
            Event::<Test>::AtsUpdated {
                ats_id: 0,
                version: 1,
                commitment: commitment(2),
                protocol_version: 1,
                operator: Some(BOB),
            }
            .into(),
        );
    });
}

#[test]
fn update_on_behalf_not_owner() {
    new_test_ext().execute_with(|| {
        // Create by ALICE
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(1), 1));

        // BOB tries to update as if CHARLIE is the owner — should fail
        let payload = update_payload(0, commitment(2), 1, BOB, 0);
        assert_noop!(
            Ats::update_on_behalf(
                RuntimeOrigin::signed(BOB),
                CHARLIE,
                0,
                commitment(2),
                1,
                0,
                sign(CHARLIE, &payload),
            ),
            Error::<Test>::NotOwner
        );
    });
}

#[test]
fn update_on_behalf_invalid_signature() {
    new_test_ext().execute_with(|| {
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(1), 1));

        let payload = update_payload(0, commitment(2), 1, BOB, 0);
        assert_noop!(
            Ats::update_on_behalf(
                RuntimeOrigin::signed(BOB),
                ALICE,
                0,
                commitment(2),
                1,
                0,
                sign(BOB, &payload), // wrong signer
            ),
            Error::<Test>::InvalidSignature
        );
    });
}

#[test]
fn update_on_behalf_deposits_from_operator() {
    new_test_ext().execute_with(|| {
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(1), 1));

        let bob_before = Balances::free_balance(BOB);
        let alice_before = Balances::free_balance(ALICE);

        let payload = update_payload(0, commitment(2), 1, BOB, 0);
        assert_ok!(Ats::update_on_behalf(
            RuntimeOrigin::signed(BOB),
            ALICE,
            0,
            commitment(2),
            1,
            0,
            sign(ALICE, &payload),
        ));

        // BOB paid version deposit
        assert_eq!(Balances::free_balance(BOB), bob_before - 10);
        // ALICE unchanged (her previous deposits still held but not affected)
        assert_eq!(Balances::free_balance(ALICE), alice_before);
    });
}

// ── on-behalf: revoke ──────────────────────────────────────────────────────

#[test]
fn revoke_on_behalf_works() {
    new_test_ext().execute_with(|| {
        let bob_before = Balances::free_balance(BOB);

        // Create on behalf: BOB pays deposits
        let payload = create_payload(commitment(1), 1, BOB, 0);
        assert_ok!(Ats::create_on_behalf(
            RuntimeOrigin::signed(BOB),
            ALICE,
            commitment(1),
            1,
            0,
            sign(ALICE, &payload),
        ));

        // Revoke on behalf
        let payload = revoke_payload(0, BOB, 1);
        assert_ok!(Ats::revoke_on_behalf(
            RuntimeOrigin::signed(BOB),
            ALICE,
            0,
            1, // nonce incremented from create
            sign(ALICE, &payload),
        ));

        // ATS gone
        assert!(AtsRegistry::<Test>::get(0).is_none());

        // BOB gets deposits back
        assert_eq!(Balances::free_balance(BOB), bob_before);

        System::assert_last_event(
            Event::<Test>::AtsRevoked {
                ats_id: 0,
                owner: ALICE,
                operator: Some(BOB),
            }
            .into(),
        );
    });
}

#[test]
fn revoke_on_behalf_mixed_depositors() {
    new_test_ext().execute_with(|| {
        let alice_before = Balances::free_balance(ALICE);
        let bob_before = Balances::free_balance(BOB);

        // ALICE creates directly (depositor=ALICE for base + v0)
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(1), 1));

        // BOB updates on behalf (depositor=BOB for v1)
        let payload = update_payload(0, commitment(2), 1, BOB, 0);
        assert_ok!(Ats::update_on_behalf(
            RuntimeOrigin::signed(BOB),
            ALICE,
            0,
            commitment(2),
            1,
            0,
            sign(ALICE, &payload),
        ));

        // Revoke on behalf
        let payload = revoke_payload(0, BOB, 1);
        assert_ok!(Ats::revoke_on_behalf(
            RuntimeOrigin::signed(BOB),
            ALICE,
            0,
            1,
            sign(ALICE, &payload),
        ));

        // ALICE gets her deposits back (base + v0 version)
        assert_eq!(Balances::free_balance(ALICE), alice_before);
        // BOB gets his v1 deposit back
        assert_eq!(Balances::free_balance(BOB), bob_before);
    });
}

#[test]
fn revoke_on_behalf_invalid_signature() {
    new_test_ext().execute_with(|| {
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(1), 1));

        let payload = revoke_payload(0, BOB, 0);
        assert_noop!(
            Ats::revoke_on_behalf(
                RuntimeOrigin::signed(BOB),
                ALICE,
                0,
                0,
                sign(BOB, &payload), // wrong signer
            ),
            Error::<Test>::InvalidSignature
        );
    });
}

// ── on-behalf: owner retains direct rights ─────────────────────────────────

#[test]
fn owner_can_update_after_on_behalf_create() {
    new_test_ext().execute_with(|| {
        let payload = create_payload(commitment(1), 1, BOB, 0);
        assert_ok!(Ats::create_on_behalf(
            RuntimeOrigin::signed(BOB),
            ALICE,
            commitment(1),
            1,
            0,
            sign(ALICE, &payload),
        ));

        // ALICE can still update directly
        assert_ok!(Ats::update(
            RuntimeOrigin::signed(ALICE),
            0,
            commitment(2),
            1
        ));

        let record = AtsRegistry::<Test>::get(0).unwrap();
        // BOB has base+v0=110, ALICE has v1=10
        assert_eq!(deposit_for(&record, BOB), 110);
        assert_eq!(deposit_for(&record, ALICE), 10);
    });
}

#[test]
fn owner_can_revoke_after_on_behalf_create() {
    new_test_ext().execute_with(|| {
        let bob_before = Balances::free_balance(BOB);

        let payload = create_payload(commitment(1), 1, BOB, 0);
        assert_ok!(Ats::create_on_behalf(
            RuntimeOrigin::signed(BOB),
            ALICE,
            commitment(1),
            1,
            0,
            sign(ALICE, &payload),
        ));

        // ALICE revokes directly — BOB still gets deposits back
        assert_ok!(Ats::revoke(RuntimeOrigin::signed(ALICE), 0));

        assert_eq!(Balances::free_balance(BOB), bob_before);
    });
}

// ── on-behalf: nonce behavior ──────────────────────────────────────────────

#[test]
fn nonce_is_per_owner() {
    new_test_ext().execute_with(|| {
        let payload_alice = create_payload(commitment(1), 1, BOB, 0);
        assert_ok!(Ats::create_on_behalf(
            RuntimeOrigin::signed(BOB),
            ALICE,
            commitment(1),
            1,
            0,
            sign(ALICE, &payload_alice),
        ));

        let payload_charlie = create_payload(commitment(2), 1, BOB, 0);
        assert_ok!(Ats::create_on_behalf(
            RuntimeOrigin::signed(BOB),
            CHARLIE,
            commitment(2),
            1,
            0,
            sign(CHARLIE, &payload_charlie),
        ));

        assert_eq!(OnBehalfNonce::<Test>::get(ALICE), 1);
        assert_eq!(OnBehalfNonce::<Test>::get(CHARLIE), 1);
    });
}

#[test]
fn nonce_survives_revoke() {
    new_test_ext().execute_with(|| {
        let payload = create_payload(commitment(1), 1, BOB, 0);
        assert_ok!(Ats::create_on_behalf(
            RuntimeOrigin::signed(BOB),
            ALICE,
            commitment(1),
            1,
            0,
            sign(ALICE, &payload),
        ));

        let payload = revoke_payload(0, BOB, 1);
        assert_ok!(Ats::revoke_on_behalf(
            RuntimeOrigin::signed(BOB),
            ALICE,
            0,
            1,
            sign(ALICE, &payload),
        ));

        // Nonce is 2, not reset
        assert_eq!(OnBehalfNonce::<Test>::get(ALICE), 2);

        // Next create must use nonce 2
        let payload = create_payload(commitment(2), 1, BOB, 2);
        assert_ok!(Ats::create_on_behalf(
            RuntimeOrigin::signed(BOB),
            ALICE,
            commitment(2),
            1,
            2,
            sign(ALICE, &payload),
        ));
    });
}

// ── on-behalf: payload stability ───────────────────────────────────────────
// These tests verify that the signature is cryptographically bound to the
// exact payload content. A signature valid for one set of parameters MUST
// fail if any parameter is changed.

#[test]
fn create_on_behalf_tampered_commitment_fails() {
    new_test_ext().execute_with(|| {
        // Sign payload with commitment(1)
        let payload = create_payload(commitment(1), 1, BOB, 0);
        let sig = sign(ALICE, &payload);

        // Submit with commitment(2) — signature should not match
        assert_noop!(
            Ats::create_on_behalf(
                RuntimeOrigin::signed(BOB),
                ALICE,
                commitment(2), // tampered
                1,
                0,
                sig,
            ),
            Error::<Test>::InvalidSignature
        );
    });
}

#[test]
fn create_on_behalf_tampered_protocol_version_fails() {
    new_test_ext().execute_with(|| {
        let payload = create_payload(commitment(1), 1, BOB, 0);
        let sig = sign(ALICE, &payload);

        assert_noop!(
            Ats::create_on_behalf(
                RuntimeOrigin::signed(BOB),
                ALICE,
                commitment(1),
                2, // tampered protocol version
                0,
                sig,
            ),
            Error::<Test>::InvalidSignature
        );
    });
}

#[test]
fn create_on_behalf_wrong_operator_in_payload_fails() {
    new_test_ext().execute_with(|| {
        // ALICE signs a payload authorizing CHARLIE as operator
        let payload = create_payload(commitment(1), 1, CHARLIE, 0);
        let sig = sign(ALICE, &payload);

        // BOB submits — operator mismatch in payload vs actual caller
        assert_noop!(
            Ats::create_on_behalf(
                RuntimeOrigin::signed(BOB), // not CHARLIE
                ALICE,
                commitment(1),
                1,
                0,
                sig,
            ),
            Error::<Test>::InvalidSignature
        );
    });
}

#[test]
fn create_on_behalf_cross_action_replay_fails() {
    new_test_ext().execute_with(|| {
        // ALICE signs a Create payload
        let create_pl = create_payload(commitment(1), 1, BOB, 0);
        let sig = sign(ALICE, &create_pl);

        // First create succeeds
        assert_ok!(Ats::create_on_behalf(
            RuntimeOrigin::signed(BOB),
            ALICE,
            commitment(1),
            1,
            0,
            sig,
        ));

        // Try to use a Create-shaped signature for an Update call
        // The pallet rebuilds an UpdateOnBehalfPayload internally, so the bytes differ
        let fake_update_sig = sign(ALICE, &create_payload(commitment(2), 1, BOB, 1));
        assert_noop!(
            Ats::update_on_behalf(
                RuntimeOrigin::signed(BOB),
                ALICE,
                0,
                commitment(2),
                1,
                1,
                fake_update_sig,
            ),
            Error::<Test>::InvalidSignature
        );
    });
}

#[test]
fn update_on_behalf_tampered_commitment_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(1), 1));

        // Sign for commitment(2)
        let payload = update_payload(0, commitment(2), 1, BOB, 0);
        let sig = sign(ALICE, &payload);

        // Submit with commitment(3) — tampered
        assert_noop!(
            Ats::update_on_behalf(
                RuntimeOrigin::signed(BOB),
                ALICE,
                0,
                commitment(3), // tampered
                1,
                0,
                sig,
            ),
            Error::<Test>::InvalidSignature
        );
    });
}

#[test]
fn update_on_behalf_tampered_ats_id_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(1), 1));
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(2), 1));

        // Sign for ats_id=0
        let payload = update_payload(0, commitment(3), 1, BOB, 0);
        let sig = sign(ALICE, &payload);

        // Submit for ats_id=1 — tampered
        assert_noop!(
            Ats::update_on_behalf(
                RuntimeOrigin::signed(BOB),
                ALICE,
                1, // tampered ats_id
                commitment(3),
                1,
                0,
                sig,
            ),
            Error::<Test>::InvalidSignature
        );
    });
}

#[test]
fn revoke_on_behalf_tampered_ats_id_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(1), 1));
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(2), 1));

        // Sign for ats_id=0
        let payload = revoke_payload(0, BOB, 0);
        let sig = sign(ALICE, &payload);

        // Submit for ats_id=1 — tampered
        assert_noop!(
            Ats::revoke_on_behalf(
                RuntimeOrigin::signed(BOB),
                ALICE,
                1, // tampered
                0,
                sig,
            ),
            Error::<Test>::InvalidSignature
        );
    });
}

#[test]
fn revoke_on_behalf_cross_action_replay_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(1), 1));

        // Sign a Revoke payload
        let revoke_pl = revoke_payload(0, BOB, 0);
        let sig = sign(ALICE, &revoke_pl);

        // Try to use it for an Update — action field differs
        assert_noop!(
            Ats::update_on_behalf(
                RuntimeOrigin::signed(BOB),
                ALICE,
                0,
                commitment(2),
                1,
                0,
                sig,
            ),
            Error::<Test>::InvalidSignature
        );
    });
}

// ── Payload encoding determinism ───────────────────────────────────────────

#[test]
fn payload_encoding_is_deterministic() {
    // Same inputs must always produce the same encoded bytes
    let p1 = create_payload(commitment(1), 1, BOB, 0);
    let p2 = create_payload(commitment(1), 1, BOB, 0);
    assert_eq!(p1.encode(), p2.encode());

    // Different inputs must produce different encoded bytes
    let p3 = create_payload(commitment(2), 1, BOB, 0);
    assert_ne!(p1.encode(), p3.encode());

    let p4 = create_payload(commitment(1), 2, BOB, 0);
    assert_ne!(p1.encode(), p4.encode());

    let p5 = create_payload(commitment(1), 1, CHARLIE, 0);
    assert_ne!(p1.encode(), p5.encode());

    let p6 = create_payload(commitment(1), 1, BOB, 1);
    assert_ne!(p1.encode(), p6.encode());
}

#[test]
fn different_action_types_produce_different_encodings() {
    // Create and Update payloads with overlapping fields must differ
    // due to the OnBehalfAction discriminant
    let create_bytes = CreateOnBehalfPayload {
        action: OnBehalfAction::Create,
        commitment: commitment(1),
        protocol_version: 1,
        operator: BOB,
        nonce: 0,
    }
    .encode();

    let update_bytes = UpdateOnBehalfPayload {
        action: OnBehalfAction::Update,
        ats_id: 0,
        commitment: commitment(1),
        protocol_version: 1,
        operator: BOB,
        nonce: 0,
    }
    .encode();

    let revoke_bytes = RevokeOnBehalfPayload {
        action: OnBehalfAction::Revoke,
        ats_id: 0,
        operator: BOB,
        nonce: 0,
    }
    .encode();

    assert_ne!(create_bytes, update_bytes);
    assert_ne!(create_bytes, revoke_bytes);
    assert_ne!(update_bytes, revoke_bytes);
}

// ── Cross-language test vectors ────────────────────────────────────────────
// These test vectors document the exact SCALE-encoded byte layout of each
// payload type. External implementations (TypeScript, Python, mobile) MUST
// produce these exact bytes for signature verification to succeed.
//
// SCALE encoding rules used here (all integers are little-endian):
//   - enum variant: single byte index (Create=0x00, Update=0x01, Revoke=0x02)
//   - [u8; 32]: 32 raw bytes, no length prefix
//   - u8: 1 byte
//   - u64: 8 bytes LE
//   - AccountId (u64 in test runtime, 32 bytes on Allfeat mainnet): encoded as
//     the runtime's native AccountId SCALE encoding
//   - struct fields are concatenated in declaration order, no separators

#[test]
fn test_vector_create_payload() {
    // CreateOnBehalfPayload {
    //   action: Create (0x00),
    //   commitment: [0xab; 32],
    //   protocol_version: 1 (0x01),
    //   operator: 42u64 (0x2a00000000000000),
    //   nonce: 7u64 (0x0700000000000000),
    // }
    let payload = CreateOnBehalfPayload {
        action: OnBehalfAction::Create,
        commitment: [0xab; 32],
        protocol_version: 1,
        operator: 42u64,
        nonce: 7,
    };

    let bytes = payload.encode();
    let hex = hex_encode(&bytes);

    // Field breakdown:
    // 00                                                               action = Create
    // abababababababababababababababababababababababababababababababab     commitment (32 bytes)
    // 01                                                               protocol_version
    // 2a00000000000000                                                 operator = 42 (u64 LE)
    // 0700000000000000                                                 nonce = 7 (u64 LE)
    let expected = concat!(
        "00",                                                               // action
        "abababababababababababababababababababababababababababababababab", // commitment
        "01",                                                               // protocol_version
        "2a00000000000000",                                                 // operator
        "0700000000000000",                                                 // nonce
    );
    assert_eq!(hex, expected, "CreateOnBehalfPayload encoding mismatch");
    assert_eq!(bytes.len(), 1 + 32 + 1 + 8 + 8, "unexpected payload length");
}

#[test]
fn test_vector_update_payload() {
    // UpdateOnBehalfPayload {
    //   action: Update (0x01),
    //   ats_id: 5u64 (0x0500000000000000),
    //   commitment: [0xcd; 32],
    //   protocol_version: 2 (0x02),
    //   operator: 99u64 (0x6300000000000000),
    //   nonce: 3u64 (0x0300000000000000),
    // }
    let payload = UpdateOnBehalfPayload {
        action: OnBehalfAction::Update,
        ats_id: 5,
        commitment: [0xcd; 32],
        protocol_version: 2,
        operator: 99u64,
        nonce: 3,
    };

    let bytes = payload.encode();
    let hex = hex_encode(&bytes);

    let expected = concat!(
        "01",                                                               // action
        "0500000000000000",                                                 // ats_id
        "cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd", // commitment
        "02",                                                               // protocol_version
        "6300000000000000",                                                 // operator
        "0300000000000000",                                                 // nonce
    );
    assert_eq!(hex, expected, "UpdateOnBehalfPayload encoding mismatch");
    assert_eq!(
        bytes.len(),
        1 + 8 + 32 + 1 + 8 + 8,
        "unexpected payload length"
    );
}

#[test]
fn test_vector_revoke_payload() {
    // RevokeOnBehalfPayload {
    //   action: Revoke (0x02),
    //   ats_id: 12u64 (0x0c00000000000000),
    //   operator: 7u64 (0x0700000000000000),
    //   nonce: 0u64 (0x0000000000000000),
    // }
    let payload = RevokeOnBehalfPayload {
        action: OnBehalfAction::Revoke,
        ats_id: 12,
        operator: 7u64,
        nonce: 0,
    };

    let bytes = payload.encode();
    let hex = hex_encode(&bytes);

    let expected = concat!(
        "02",               // action
        "0c00000000000000", // ats_id
        "0700000000000000", // operator
        "0000000000000000", // nonce
    );
    assert_eq!(hex, expected, "RevokeOnBehalfPayload encoding mismatch");
    assert_eq!(bytes.len(), 1 + 8 + 8 + 8, "unexpected payload length");
}

#[test]
fn test_vector_action_enum_discriminants() {
    // External implementations need to know the exact discriminant values.
    assert_eq!(OnBehalfAction::Create.encode(), vec![0x00]);
    assert_eq!(OnBehalfAction::Update.encode(), vec![0x01]);
    assert_eq!(OnBehalfAction::Revoke.encode(), vec![0x02]);
}

/// Encode bytes as lowercase hex string (no prefix).
fn hex_encode(bytes: &[u8]) -> alloc::string::String {
    bytes.iter().map(|b| alloc::format!("{b:02x}")).collect()
}

// ── Migration v1→v2 ───────────────────────────────────────────────────────

/// V1 ATS record (matches the v1 on-chain layout).
#[derive(Encode)]
struct V1AtsRecord {
    owner: u64,
    depositor: u64,
    created_at: u64,
    version_count: u32,
    base_deposit: u64,
}

/// V1 version record (with depositor and deposit).
#[derive(Encode)]
struct V1VersionRecord {
    commitment: [u8; 32],
    protocol_version: u8,
    depositor: u64,
    created_at: u64,
    deposit: u64,
}

#[test]
fn migration_v1_to_v2_works() {
    use frame_support::traits::OnRuntimeUpgrade;

    new_test_ext().execute_with(|| {
        // Simulate on-chain storage version 1
        StorageVersion::new(1).put::<Ats>();

        // Write v1-format ATS record directly to storage
        let old_record = V1AtsRecord {
            owner: ALICE,
            depositor: ALICE,
            created_at: 1,
            version_count: 2,
            base_deposit: 100,
        };
        let key = AtsRegistry::<Test>::hashed_key_for(0u64);
        frame_support::storage::unhashed::put_raw(&key, &old_record.encode());

        // Write v1-format version records
        let old_v0 = V1VersionRecord {
            commitment: commitment(1),
            protocol_version: 1,
            depositor: ALICE,
            created_at: 1,
            deposit: 10,
        };
        let key_v0 = AtsVersions::<Test>::hashed_key_for(0u64, 0u32);
        frame_support::storage::unhashed::put_raw(&key_v0, &old_v0.encode());

        let old_v1 = V1VersionRecord {
            commitment: commitment(2),
            protocol_version: 1,
            depositor: ALICE,
            created_at: 1,
            deposit: 10,
        };
        let key_v1 = AtsVersions::<Test>::hashed_key_for(0u64, 1u32);
        frame_support::storage::unhashed::put_raw(&key_v1, &old_v1.encode());

        // Run migration
        migrations::v2::MigrateV1ToV2::<Test>::on_runtime_upgrade();

        // Verify ATS record migrated correctly
        let record = AtsRegistry::<Test>::get(0).expect("record should exist");
        assert_eq!(record.owner, ALICE);
        assert_eq!(record.version_count, 2);
        // All deposits aggregated: base(100) + v0(10) + v1(10) = 120
        assert_eq!(record.deposits.len(), 1);
        assert_eq!(deposit_for(&record, ALICE), 120);

        // Verify version records slimmed down (no depositor/deposit fields)
        let v0 = AtsVersions::<Test>::get(0, 0).expect("v0 should exist");
        assert_eq!(v0.commitment, commitment(1));

        let v1 = AtsVersions::<Test>::get(0, 1).expect("v1 should exist");
        assert_eq!(v1.commitment, commitment(2));

        // Verify storage version updated
        assert_eq!(Ats::on_chain_storage_version(), 2);
    });
}

#[test]
fn migration_v1_to_v2_mixed_depositors() {
    use frame_support::traits::OnRuntimeUpgrade;

    new_test_ext().execute_with(|| {
        StorageVersion::new(1).put::<Ats>();

        // ATS record with ALICE as owner/depositor
        let old_record = V1AtsRecord {
            owner: ALICE,
            depositor: ALICE,
            created_at: 1,
            version_count: 3,
            base_deposit: 100,
        };
        let key = AtsRegistry::<Test>::hashed_key_for(0u64);
        frame_support::storage::unhashed::put_raw(&key, &old_record.encode());

        // v0: depositor=ALICE
        let v0 = V1VersionRecord {
            commitment: commitment(1),
            protocol_version: 1,
            depositor: ALICE,
            created_at: 1,
            deposit: 10,
        };
        frame_support::storage::unhashed::put_raw(
            &AtsVersions::<Test>::hashed_key_for(0u64, 0u32),
            &v0.encode(),
        );

        // v1: depositor=BOB (on-behalf update)
        let v1 = V1VersionRecord {
            commitment: commitment(2),
            protocol_version: 1,
            depositor: BOB,
            created_at: 1,
            deposit: 10,
        };
        frame_support::storage::unhashed::put_raw(
            &AtsVersions::<Test>::hashed_key_for(0u64, 1u32),
            &v1.encode(),
        );

        // v2: depositor=BOB (another on-behalf update)
        let v2 = V1VersionRecord {
            commitment: commitment(3),
            protocol_version: 1,
            depositor: BOB,
            created_at: 1,
            deposit: 10,
        };
        frame_support::storage::unhashed::put_raw(
            &AtsVersions::<Test>::hashed_key_for(0u64, 2u32),
            &v2.encode(),
        );

        migrations::v2::MigrateV1ToV2::<Test>::on_runtime_upgrade();

        let record = AtsRegistry::<Test>::get(0).expect("record should exist");
        assert_eq!(record.deposits.len(), 2);
        // ALICE: base(100) + v0(10) = 110
        assert_eq!(deposit_for(&record, ALICE), 110);
        // BOB: v1(10) + v2(10) = 20
        assert_eq!(deposit_for(&record, BOB), 20);
    });
}

#[test]
fn migration_v2_skips_if_already_migrated() {
    use frame_support::traits::OnRuntimeUpgrade;

    new_test_ext().execute_with(|| {
        // Already at version 2
        StorageVersion::new(2).put::<Ats>();

        let weight = migrations::v2::MigrateV1ToV2::<Test>::on_runtime_upgrade();

        // Should return zero weight (skipped)
        assert_eq!(weight, frame_support::weights::Weight::zero());
    });
}

// ── Deposit aggregation ────────────────────────────────────────────────────

#[test]
fn deposits_aggregate_same_depositor() {
    new_test_ext().execute_with(|| {
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(0), 1));
        assert_ok!(Ats::update(
            RuntimeOrigin::signed(ALICE),
            0,
            commitment(1),
            1
        ));
        assert_ok!(Ats::update(
            RuntimeOrigin::signed(ALICE),
            0,
            commitment(2),
            1
        ));

        let record = AtsRegistry::<Test>::get(0).unwrap();
        // Only one deposit entry, aggregated
        assert_eq!(record.deposits.len(), 1);
        // base(100) + 3*version(10) = 130
        assert_eq!(deposit_for(&record, ALICE), 130);
    });
}

#[test]
fn deposits_track_multiple_depositors() {
    new_test_ext().execute_with(|| {
        // ALICE creates
        assert_ok!(Ats::create(RuntimeOrigin::signed(ALICE), commitment(0), 1));

        // BOB updates on behalf
        let payload = update_payload(0, commitment(1), 1, BOB, 0);
        assert_ok!(Ats::update_on_behalf(
            RuntimeOrigin::signed(BOB),
            ALICE,
            0,
            commitment(1),
            1,
            0,
            sign(ALICE, &payload),
        ));

        // CHARLIE updates on behalf
        let payload = update_payload(0, commitment(2), 1, CHARLIE, 1);
        assert_ok!(Ats::update_on_behalf(
            RuntimeOrigin::signed(CHARLIE),
            ALICE,
            0,
            commitment(2),
            1,
            1,
            sign(ALICE, &payload),
        ));

        let record = AtsRegistry::<Test>::get(0).unwrap();
        assert_eq!(record.deposits.len(), 3);
        assert_eq!(deposit_for(&record, ALICE), 110); // base + v0
        assert_eq!(deposit_for(&record, BOB), 10); // v1
        assert_eq!(deposit_for(&record, CHARLIE), 10); // v2
    });
}

// ── Storage version ────────────────────────────────────────────────────────

#[test]
fn storage_version_is_2() {
    new_test_ext().execute_with(|| {
        assert_eq!(Ats::in_code_storage_version(), 2);
    });
}
