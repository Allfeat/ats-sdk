use crate as pallet_ats;
use alloc::vec::Vec;
use codec::{Decode, DecodeWithMemTracking, Encode};
use frame_support::{construct_runtime, derive_impl, parameter_types};
use scale_info::TypeInfo;
use sp_runtime::BuildStorage;
use sp_runtime::traits::{IdentifyAccount, Verify};

pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;

// ── Test signature types ───────────────────────────────────────────────────

/// Test signer that maps directly to a `u64` account ID.
#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode, DecodeWithMemTracking, TypeInfo)]
pub struct TestSigner(pub u64);

impl IdentifyAccount for TestSigner {
    type AccountId = u64;

    fn into_account(self) -> u64 {
        self.0
    }
}

/// Test signature that validates both the signer identity **and** the signed payload.
///
/// Unlike the previous implementation that ignored message content, this version
/// ensures the signature is bound to the exact payload bytes. This catches:
/// - Payload tampering (e.g., modified commitment after signing)
/// - Cross-action replay (e.g., Create signature used for Update)
/// - Operator mismatch in payload vs caller
#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode, DecodeWithMemTracking, TypeInfo)]
pub struct TestSignature {
    /// The account ID of the signer.
    pub signer: u64,
    /// The exact SCALE-encoded payload bytes that were "signed".
    pub payload: Vec<u8>,
}

impl Verify for TestSignature {
    type Signer = TestSigner;

    fn verify<L: sp_runtime::traits::Lazy<[u8]>>(&self, mut msg: L, signer: &u64) -> bool {
        self.signer == *signer && self.payload == msg.get()
    }
}

/// Create a test signature for a SCALE-encodable payload.
pub fn sign(signer: u64, payload: &impl Encode) -> TestSignature {
    TestSignature {
        signer,
        payload: payload.encode(),
    }
}

// ── Runtime construction ───────────────────────────────────────────────────

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

/// Benchmark helper that generates valid [`TestSignature`]s.
#[cfg(feature = "runtime-benchmarks")]
pub struct TestBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl crate::BenchmarkHelper<TestSignature, u64> for TestBenchmarkHelper {
    fn create_signature(_entropy: &[u8], msg: &[u8]) -> (TestSignature, u64) {
        const BENCH_OWNER: u64 = 42;
        let sig = TestSignature {
            signer: BENCH_OWNER,
            payload: msg.to_vec(),
        };
        (sig, BENCH_OWNER)
    }
}

impl pallet_ats::Config for Test {
    type RuntimeHoldReason = RuntimeHoldReason;
    type Currency = Balances;
    type OffchainSignature = TestSignature;
    type Signer = TestSigner;
    type BaseDeposit = BaseDeposit;
    type VersionDeposit = VersionDeposit;
    type MaxVersionsPerAts = MaxVersionsPerAts;
    type MaxAtsPerAccount = MaxAtsPerAccount;
    type WeightInfo = ();
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = TestBenchmarkHelper;
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
