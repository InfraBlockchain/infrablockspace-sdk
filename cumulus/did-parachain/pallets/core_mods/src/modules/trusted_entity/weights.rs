//! Autogenerated weights for trusted_entity
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2022-08-01, STEPS: `[50, ]`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Native), WASM-EXECUTION: Interpreted, CHAIN: Some("infradid-mainnet"), DB CACHE: 128

// Executed Command:
// ./target/productioninfradid
// benchmark
// --execution=native
// --chain=infradid-mainnet
// --pallet=trusted_entity
// --extra
// --extrinsic=*
// --repeat=20
// --steps=50
// --template=node/module-weight-template.hbs
// --output=./pallets/core_mods/src/modules/trusted_entity/weights.rs

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for revoke.
pub trait WeightInfo {
    fn add_issuer_sr25519(r: u32) -> Weight;
    fn add_issuer_ed25519(r: u32) -> Weight;
    fn remove_issuer_sr25519(r: u32) -> Weight;
    fn remove_issuer_ed25519(r: u32) -> Weight;
    fn add_verifier_sr25519(r: u32) -> Weight;
    fn add_verifier_ed25519(r: u32) -> Weight;
    fn remove_verifier_sr25519(r: u32) -> Weight;
    fn remove_verifier_ed25519(r: u32) -> Weight;
    fn remove_authorizer_sr25519() -> Weight;
    fn remove_authorizer_ed25519() -> Weight;
    fn new_authorizer(c: u32) -> Weight;
}

/// Weights for revoke using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn add_issuer_sr25519(r: u32) -> Weight {
        Weight::from_ref_time(51_886_000 as u64)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(744_000 as u64).saturating_mul(r as u64))
            .saturating_add(T::DbWeight::get().reads(4 as u64))
            .saturating_add(T::DbWeight::get().writes(3 as u64))
            .saturating_add(T::DbWeight::get().writes((1 as u64).saturating_mul(r as u64)))
    }
    fn add_issuer_ed25519(r: u32) -> Weight {
        Weight::from_ref_time(55_942_000 as u64)
            // Standard Error: 1_000
            .saturating_add(Weight::from_ref_time(718_000 as u64).saturating_mul(r as u64))
            .saturating_add(T::DbWeight::get().reads(4 as u64))
            .saturating_add(T::DbWeight::get().writes(3 as u64))
            .saturating_add(T::DbWeight::get().writes((1 as u64).saturating_mul(r as u64)))
    }
    fn remove_issuer_sr25519(r: u32) -> Weight {
        Weight::from_ref_time(67_695_000 as u64)
            // Standard Error: 1_000
            .saturating_add(Weight::from_ref_time(741_000 as u64).saturating_mul(r as u64))
            .saturating_add(T::DbWeight::get().reads(4 as u64))
            .saturating_add(T::DbWeight::get().writes(3 as u64))
            .saturating_add(T::DbWeight::get().writes((1 as u64).saturating_mul(r as u64)))
    }
    fn remove_issuer_ed25519(r: u32) -> Weight {
        Weight::from_ref_time(65_882_000 as u64)
            // Standard Error: 3_000
            .saturating_add(Weight::from_ref_time(747_000 as u64).saturating_mul(r as u64))
            .saturating_add(T::DbWeight::get().reads(4 as u64))
            .saturating_add(T::DbWeight::get().writes(3 as u64))
            .saturating_add(T::DbWeight::get().writes((1 as u64).saturating_mul(r as u64)))
    }
    fn add_verifier_sr25519(r: u32) -> Weight {
        Weight::from_ref_time(51_886_000 as u64)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(744_000 as u64).saturating_mul(r as u64))
            .saturating_add(T::DbWeight::get().reads(4 as u64))
            .saturating_add(T::DbWeight::get().writes(3 as u64))
            .saturating_add(T::DbWeight::get().writes((1 as u64).saturating_mul(r as u64)))
    }
    fn add_verifier_ed25519(r: u32) -> Weight {
        Weight::from_ref_time(55_942_000 as u64)
            // Standard Error: 1_000
            .saturating_add(Weight::from_ref_time(718_000 as u64).saturating_mul(r as u64))
            .saturating_add(T::DbWeight::get().reads(4 as u64))
            .saturating_add(T::DbWeight::get().writes(3 as u64))
            .saturating_add(T::DbWeight::get().writes((1 as u64).saturating_mul(r as u64)))
    }
    fn remove_verifier_sr25519(r: u32) -> Weight {
        Weight::from_ref_time(67_695_000 as u64)
            // Standard Error: 1_000
            .saturating_add(Weight::from_ref_time(741_000 as u64).saturating_mul(r as u64))
            .saturating_add(T::DbWeight::get().reads(4 as u64))
            .saturating_add(T::DbWeight::get().writes(3 as u64))
            .saturating_add(T::DbWeight::get().writes((1 as u64).saturating_mul(r as u64)))
    }
    fn remove_verifier_ed25519(r: u32) -> Weight {
        Weight::from_ref_time(65_882_000 as u64)
            // Standard Error: 3_000
            .saturating_add(Weight::from_ref_time(747_000 as u64).saturating_mul(r as u64))
            .saturating_add(T::DbWeight::get().reads(4 as u64))
            .saturating_add(T::DbWeight::get().writes(3 as u64))
            .saturating_add(T::DbWeight::get().writes((1 as u64).saturating_mul(r as u64)))
    }
    fn remove_authorizer_sr25519() -> Weight {
        Weight::from_ref_time(128_526_000 as u64)
            .saturating_add(T::DbWeight::get().reads(4 as u64))
            .saturating_add(T::DbWeight::get().writes(103 as u64))
    }
    fn remove_authorizer_ed25519() -> Weight {
        Weight::from_ref_time(122_116_000 as u64)
            .saturating_add(T::DbWeight::get().reads(4 as u64))
            .saturating_add(T::DbWeight::get().writes(103 as u64))
    }
    fn new_authorizer(c: u32) -> Weight {
        Weight::from_ref_time(9_069_000 as u64)
            // Standard Error: 1_000
            .saturating_add(Weight::from_ref_time(35_000 as u64).saturating_mul(c as u64))
            .saturating_add(T::DbWeight::get().reads(2 as u64))
            .saturating_add(T::DbWeight::get().writes(2 as u64))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn add_issuer_sr25519(r: u32) -> Weight {
        Weight::from_ref_time(51_886_000 as u64)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(744_000 as u64).saturating_mul(r as u64))
            .saturating_add(RocksDbWeight::get().reads(4 as u64))
            .saturating_add(RocksDbWeight::get().writes(3 as u64))
            .saturating_add(RocksDbWeight::get().writes((1 as u64).saturating_mul(r as u64)))
    }
    fn add_issuer_ed25519(r: u32) -> Weight {
        Weight::from_ref_time(55_942_000 as u64)
            // Standard Error: 1_000
            .saturating_add(Weight::from_ref_time(718_000 as u64).saturating_mul(r as u64))
            .saturating_add(RocksDbWeight::get().reads(4 as u64))
            .saturating_add(RocksDbWeight::get().writes(3 as u64))
            .saturating_add(RocksDbWeight::get().writes((1 as u64).saturating_mul(r as u64)))
    }
    fn remove_issuer_sr25519(r: u32) -> Weight {
        Weight::from_ref_time(67_695_000 as u64)
            // Standard Error: 1_000
            .saturating_add(Weight::from_ref_time(741_000 as u64).saturating_mul(r as u64))
            .saturating_add(RocksDbWeight::get().reads(4 as u64))
            .saturating_add(RocksDbWeight::get().writes(3 as u64))
            .saturating_add(RocksDbWeight::get().writes((1 as u64).saturating_mul(r as u64)))
    }
    fn remove_issuer_ed25519(r: u32) -> Weight {
        Weight::from_ref_time(65_882_000 as u64)
            // Standard Error: 3_000
            .saturating_add(Weight::from_ref_time(747_000 as u64).saturating_mul(r as u64))
            .saturating_add(RocksDbWeight::get().reads(4 as u64))
            .saturating_add(RocksDbWeight::get().writes(3 as u64))
            .saturating_add(RocksDbWeight::get().writes((1 as u64).saturating_mul(r as u64)))
    }
    fn add_verifier_sr25519(r: u32) -> Weight {
        Weight::from_ref_time(51_886_000 as u64)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(744_000 as u64).saturating_mul(r as u64))
            .saturating_add(RocksDbWeight::get().reads(4 as u64))
            .saturating_add(RocksDbWeight::get().writes(3 as u64))
            .saturating_add(RocksDbWeight::get().writes((1 as u64).saturating_mul(r as u64)))
    }
    fn add_verifier_ed25519(r: u32) -> Weight {
        Weight::from_ref_time(55_942_000 as u64)
            // Standard Error: 1_000
            .saturating_add(Weight::from_ref_time(718_000 as u64).saturating_mul(r as u64))
            .saturating_add(RocksDbWeight::get().reads(4 as u64))
            .saturating_add(RocksDbWeight::get().writes(3 as u64))
            .saturating_add(RocksDbWeight::get().writes((1 as u64).saturating_mul(r as u64)))
    }
    fn remove_verifier_sr25519(r: u32) -> Weight {
        Weight::from_ref_time(67_695_000 as u64)
            // Standard Error: 1_000
            .saturating_add(Weight::from_ref_time(741_000 as u64).saturating_mul(r as u64))
            .saturating_add(RocksDbWeight::get().reads(4 as u64))
            .saturating_add(RocksDbWeight::get().writes(3 as u64))
            .saturating_add(RocksDbWeight::get().writes((1 as u64).saturating_mul(r as u64)))
    }
    fn remove_verifier_ed25519(r: u32) -> Weight {
        Weight::from_ref_time(65_882_000 as u64)
            // Standard Error: 3_000
            .saturating_add(Weight::from_ref_time(747_000 as u64).saturating_mul(r as u64))
            .saturating_add(RocksDbWeight::get().reads(4 as u64))
            .saturating_add(RocksDbWeight::get().writes(3 as u64))
            .saturating_add(RocksDbWeight::get().writes((1 as u64).saturating_mul(r as u64)))
    }
    fn remove_authorizer_sr25519() -> Weight {
        Weight::from_ref_time(128_526_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(4 as u64))
            .saturating_add(RocksDbWeight::get().writes(103 as u64))
    }
    fn remove_authorizer_ed25519() -> Weight {
        Weight::from_ref_time(122_116_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(4 as u64))
            .saturating_add(RocksDbWeight::get().writes(103 as u64))
    }
    fn new_authorizer(c: u32) -> Weight {
        Weight::from_ref_time(9_069_000 as u64)
            // Standard Error: 1_000
            .saturating_add(Weight::from_ref_time(35_000 as u64).saturating_mul(c as u64))
            .saturating_add(RocksDbWeight::get().reads(2 as u64))
            .saturating_add(RocksDbWeight::get().writes(2 as u64))
    }
}
