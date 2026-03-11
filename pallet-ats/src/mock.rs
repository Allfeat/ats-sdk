use crate as pallet_ats;
use frame_support::{construct_runtime, derive_impl, parameter_types};
use sp_runtime::BuildStorage;

pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;

construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        Ats: pallet_ats,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = frame_system::mocking::MockBlock<Test>;
    type AccountId = u64;
    type Lookup = sp_runtime::traits::IdentityLookup<u64>;
    type AccountData = pallet_balances::AccountData<u64>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
    type AccountStore = System;
    type RuntimeHoldReason = RuntimeHoldReason;
}

parameter_types! {
    pub const BaseDeposit: u64 = 100;
    pub const VersionDeposit: u64 = 10;
    pub const MaxVersionsPerAts: u32 = 10;
    pub const MaxAtsPerAccount: u32 = 5;
}

impl pallet_ats::Config for Test {
    type RuntimeHoldReason = RuntimeHoldReason;
    type Currency = Balances;
    type BaseDeposit = BaseDeposit;
    type VersionDeposit = VersionDeposit;
    type MaxVersionsPerAts = MaxVersionsPerAts;
    type MaxAtsPerAccount = MaxAtsPerAccount;
    type WeightInfo = ();
}

/// Build test externalities with funded accounts.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(ALICE, 10_000), (BOB, 10_000), (CHARLIE, 10_000)],
        ..Default::default()
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    let mut ext = sp_io::TestExternalities::from(storage);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
