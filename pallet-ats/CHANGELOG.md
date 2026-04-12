# Changelog

## [0.4.0](https://github.com/Allfeat/ats-sdk/compare/pallet-ats-v0.3.0...pallet-ats-v0.4.0) (2026-04-12)


### ⚠ BREAKING CHANGES

* Substrate dependency versions are incompatible with previous Polkadot SDK releases.

### Features

* bump Substrate dependencies to latest versions ([aac8725](https://github.com/Allfeat/ats-sdk/commit/aac8725c24b2053bc55f2c3bdf6d7c4d528fb8ff))

## [0.3.0](https://github.com/Allfeat/ats-sdk/compare/pallet-ats-v0.2.5...pallet-ats-v0.3.0) (2026-03-21)


### ⚠ BREAKING CHANGES

* storage v2 — AtsRecord replaces depositor/base_deposit with deposits: BoundedVec<DepositEntry>, VersionRecord becomes VersionInfo (no depositor/deposit fields), HoldReason::VersionDeposit removed.

### Features

* aggregate deposits per depositor and slim version records ([93538b6](https://github.com/Allfeat/ats-sdk/commit/93538b658dcbf52faa2738abc7e40de88fdf74fc))

## [0.2.5](https://github.com/Allfeat/ats-sdk/compare/pallet-ats-v0.2.4...pallet-ats-v0.2.5) (2026-03-20)


### Bug Fixes

* format entropy as SecretUri for sr25519_generate ([d765752](https://github.com/Allfeat/ats-sdk/commit/d765752cef5370eaf97d38f839a0885c2646a1c0))

## [0.2.4](https://github.com/Allfeat/ats-sdk/compare/pallet-ats-v0.2.3...pallet-ats-v0.2.4) (2026-03-20)


### Bug Fixes

* use deterministic seed in BenchmarkHelper default impl ([a5fb393](https://github.com/Allfeat/ats-sdk/commit/a5fb393981a1cf8faafadf6fb68e3ffc6e32c2e7))

## [0.2.3](https://github.com/Allfeat/ats-sdk/compare/pallet-ats-v0.2.2...pallet-ats-v0.2.3) (2026-03-20)


### Bug Fixes

* align BenchmarkHelper trait with pallet-verify-signature pattern ([fafe109](https://github.com/Allfeat/ats-sdk/commit/fafe109967d7432a28f64092473af4679fd1240e))

## [0.2.2](https://github.com/Allfeat/ats-sdk/compare/pallet-ats-v0.2.1...pallet-ats-v0.2.2) (2026-03-20)


### Features

* implement real on-behalf benchmarks with BenchmarkHelper trait ([3e62b7b](https://github.com/Allfeat/ats-sdk/commit/3e62b7bb2fa0f848dc13dad787b6eca7617f09e0))

## [0.2.1](https://github.com/Allfeat/ats-sdk/compare/pallet-ats-v0.2.0...pallet-ats-v0.2.1) (2026-03-20)


### Features

* extract off-chain payload types into pallet-ats-primitives ([e41a853](https://github.com/Allfeat/ats-sdk/commit/e41a85319e04754dfb1f76cebd914a9c4c1128e2))


### Bug Fixes

* revert manual pallet-ats bump and add version to path dep ([1b0b065](https://github.com/Allfeat/ats-sdk/commit/1b0b06546520b8282cf205be6374f584d8ccf674))

## [0.2.0](https://github.com/Allfeat/ats-sdk/compare/pallet-ats-v0.1.1...pallet-ats-v0.2.0) (2026-03-19)


### ⚠ BREAKING CHANGES

* Config trait now requires OffchainSignature and Signer associated types. AtsRecord and VersionRecord storage layout changed (migration provided).

### Features

* add on-behalf (delegate/operator) extrinsics for pallet-ats ([2a5240a](https://github.com/Allfeat/ats-sdk/commit/2a5240ac8b329058276d0e728a657d283e562b1f))

## [0.1.1](https://github.com/Allfeat/ats-sdk/compare/pallet-ats-v0.1.0...pallet-ats-v0.1.1) (2026-03-11)


### Features

* add pallet-ats Substrate pallet for on-chain ATS commitments ([89b8bec](https://github.com/Allfeat/ats-sdk/commit/89b8bec3f8227630f9c37d1fb9c5de3dd99ffb50))
