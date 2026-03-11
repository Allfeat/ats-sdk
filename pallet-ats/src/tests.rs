use crate::mock::*;
use crate::*;
use frame_support::{assert_noop, assert_ok};

fn commitment(byte: u8) -> [u8; 32] {
    [byte; 32]
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
        assert_eq!(record.base_deposit, 100);

        // Check version 0
        let version = AtsVersions::<Test>::get(0, 0).expect("version should exist");
        assert_eq!(version.commitment, commitment(1));
        assert_eq!(version.protocol_version, 1);
        assert_eq!(version.deposit, 10);

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

        let v1 = AtsVersions::<Test>::get(0, 1).unwrap();
        assert_eq!(v1.commitment, commitment(2));

        System::assert_last_event(
            Event::<Test>::AtsUpdated {
                ats_id: 0,
                version: 1,
                commitment: commitment(2),
                protocol_version: 1,
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
