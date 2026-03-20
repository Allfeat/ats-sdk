# Changelog

## [0.3.0](https://github.com/Allfeat/ats-sdk/compare/pallet-ats-v0.2.1...pallet-ats-v0.3.0) (2026-03-20)


### ⚠ BREAKING CHANGES

* Config trait now requires OffchainSignature and Signer associated types. AtsRecord and VersionRecord storage layout changed (migration provided).

### Features

* add on-behalf (delegate/operator) extrinsics for pallet-ats ([2a5240a](https://github.com/Allfeat/ats-sdk/commit/2a5240ac8b329058276d0e728a657d283e562b1f))
* add pallet-ats Substrate pallet for on-chain ATS commitments ([89b8bec](https://github.com/Allfeat/ats-sdk/commit/89b8bec3f8227630f9c37d1fb9c5de3dd99ffb50))
* extract off-chain payload types into pallet-ats-primitives ([e41a853](https://github.com/Allfeat/ats-sdk/commit/e41a85319e04754dfb1f76cebd914a9c4c1128e2))

## [0.2.0](https://github.com/Allfeat/ats-sdk/compare/pallet-ats-v0.1.1...pallet-ats-v0.2.0) (2026-03-19)


### ⚠ BREAKING CHANGES

* Config trait now requires OffchainSignature and Signer associated types. AtsRecord and VersionRecord storage layout changed (migration provided).

### Features

* add on-behalf (delegate/operator) extrinsics for pallet-ats ([2a5240a](https://github.com/Allfeat/ats-sdk/commit/2a5240ac8b329058276d0e728a657d283e562b1f))

## [0.1.1](https://github.com/Allfeat/ats-sdk/compare/pallet-ats-v0.1.0...pallet-ats-v0.1.1) (2026-03-11)


### Features

* add pallet-ats Substrate pallet for on-chain ATS commitments ([89b8bec](https://github.com/Allfeat/ats-sdk/commit/89b8bec3f8227630f9c37d1fb9c5de3dd99ffb50))
