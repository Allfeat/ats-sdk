use super::*;
use frame_benchmarking::v2::*;
use frame_support::traits::fungible::Mutate as _;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
    use super::*;

    /// Benchmark the `create` extrinsic.
    #[benchmark]
    fn create() {
        let caller: T::AccountId = whitelisted_caller();
        T::Currency::set_balance(&caller, 1_000_000u32.into());
        let commitment = [0u8; 32];

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), commitment, 1u8);

        assert!(pallet::AtsRegistry::<T>::contains_key(0));
    }

    /// Benchmark the `update` extrinsic.
    #[benchmark]
    fn update() {
        let caller: T::AccountId = whitelisted_caller();
        T::Currency::set_balance(&caller, 1_000_000u32.into());

        Pallet::<T>::create(RawOrigin::Signed(caller.clone()).into(), [0u8; 32], 1u8)
            .expect("create should succeed");

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), 0u64, [1u8; 32], 1u8);

        assert_eq!(pallet::AtsRegistry::<T>::get(0).unwrap().version_count, 2);
    }

    /// Benchmark the `revoke` extrinsic with `v` versions.
    #[benchmark]
    fn revoke(v: Linear<1, 100>) {
        let caller: T::AccountId = whitelisted_caller();
        T::Currency::set_balance(&caller, 1_000_000u32.into());

        Pallet::<T>::create(RawOrigin::Signed(caller.clone()).into(), [0u8; 32], 1u8)
            .expect("create should succeed");

        for i in 1..v {
            Pallet::<T>::update(
                RawOrigin::Signed(caller.clone()).into(),
                0u64,
                [i as u8; 32],
                1u8,
            )
            .expect("update should succeed");
        }

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), 0u64);

        assert!(!pallet::AtsRegistry::<T>::contains_key(0));
    }

    /// Placeholder benchmark for `create_on_behalf`.
    #[benchmark]
    fn create_on_behalf() {
        // Placeholder: proper benchmarking requires a BenchmarkHelper trait
        // to generate valid signatures for the concrete runtime crypto.
        let caller: T::AccountId = whitelisted_caller();
        T::Currency::set_balance(&caller, 1_000_000u32.into());
        let commitment = [0u8; 32];

        // Fall back to direct create for weight estimation
        #[extrinsic_call]
        create(RawOrigin::Signed(caller), commitment, 1u8);

        assert!(pallet::AtsRegistry::<T>::contains_key(0));
    }

    /// Placeholder benchmark for `update_on_behalf`.
    #[benchmark]
    fn update_on_behalf() {
        let caller: T::AccountId = whitelisted_caller();
        T::Currency::set_balance(&caller, 1_000_000u32.into());

        Pallet::<T>::create(RawOrigin::Signed(caller.clone()).into(), [0u8; 32], 1u8)
            .expect("create should succeed");

        // Fall back to direct update for weight estimation
        #[extrinsic_call]
        update(RawOrigin::Signed(caller), 0u64, [1u8; 32], 1u8);

        assert_eq!(pallet::AtsRegistry::<T>::get(0).unwrap().version_count, 2);
    }

    /// Placeholder benchmark for `revoke_on_behalf`.
    #[benchmark]
    fn revoke_on_behalf(v: Linear<1, 100>) {
        let caller: T::AccountId = whitelisted_caller();
        T::Currency::set_balance(&caller, 1_000_000u32.into());

        Pallet::<T>::create(RawOrigin::Signed(caller.clone()).into(), [0u8; 32], 1u8)
            .expect("create should succeed");

        for i in 1..v {
            Pallet::<T>::update(
                RawOrigin::Signed(caller.clone()).into(),
                0u64,
                [i as u8; 32],
                1u8,
            )
            .expect("update should succeed");
        }

        // Fall back to direct revoke for weight estimation
        #[extrinsic_call]
        revoke(RawOrigin::Signed(caller), 0u64);

        assert!(!pallet::AtsRegistry::<T>::contains_key(0));
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
