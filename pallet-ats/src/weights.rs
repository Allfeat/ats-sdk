use frame_support::weights::Weight;

/// Weight information for pallet-ats extrinsics.
pub trait WeightInfo {
    /// Weight for creating a new ATS entry.
    fn create() -> Weight;
    /// Weight for updating an ATS entry with a new version.
    fn update() -> Weight;
    /// Weight for revoking an ATS entry with `v` versions.
    fn revoke(v: u32) -> Weight;
    /// Weight for creating an ATS entry on behalf of an owner.
    fn create_on_behalf() -> Weight;
    /// Weight for updating an ATS entry on behalf of an owner.
    fn update_on_behalf() -> Weight;
    /// Weight for revoking an ATS entry on behalf of an owner with `v` versions.
    fn revoke_on_behalf(v: u32) -> Weight;
}

/// Placeholder weight implementation for development and testing.
impl WeightInfo for () {
    fn create() -> Weight {
        Weight::from_parts(10_000, 0)
    }

    fn update() -> Weight {
        Weight::from_parts(10_000, 0)
    }

    fn revoke(v: u32) -> Weight {
        Weight::from_parts(
            10_000_u64.saturating_add(5_000_u64.saturating_mul(u64::from(v))),
            0,
        )
    }

    fn create_on_behalf() -> Weight {
        Weight::from_parts(20_000, 0)
    }

    fn update_on_behalf() -> Weight {
        Weight::from_parts(20_000, 0)
    }

    fn revoke_on_behalf(v: u32) -> Weight {
        Weight::from_parts(
            20_000_u64.saturating_add(5_000_u64.saturating_mul(u64::from(v))),
            0,
        )
    }
}
