//! Autogenerated weights for blob
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2022-08-01, STEPS: `[50, ]`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Native), WASM-EXECUTION: Interpreted, CHAIN: Some("mainnet"), DB CACHE: 128

// Executed Command:
// ./target/production/dock-node
// benchmark
// --execution=native
// --chain=mainnet
// --pallet=blob
// --extra
// --extrinsic=*
// --repeat=20
// --steps=50
// --template=node/module-weight-template.hbs
// --output=./pallets/core/src/modules/blob/weights.rs

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{
	traits::Get,
	weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for blob.
pub trait WeightInfo {
	fn new_sr25519(s: u32) -> Weight;
	fn new_ed25519(s: u32) -> Weight;
	fn new_secp256k1(s: u32) -> Weight;
}

/// Weights for blob using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn new_sr25519(s: u32) -> Weight {
		Weight::from_ref_time(48_757_000_u64)
			// Standard Error: 1_000
			.saturating_add(Weight::from_ref_time(31_000_u64).saturating_mul(s as u64))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	fn new_ed25519(s: u32) -> Weight {
		Weight::from_ref_time(48_672_000_u64)
			// Standard Error: 0
			.saturating_add(Weight::from_ref_time(9_000_u64).saturating_mul(s as u64))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	fn new_secp256k1(s: u32) -> Weight {
		Weight::from_ref_time(152_477_000_u64)
			// Standard Error: 2_000
			.saturating_add(Weight::from_ref_time(2_000_u64).saturating_mul(s as u64))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn new_sr25519(s: u32) -> Weight {
		Weight::from_ref_time(48_757_000_u64)
			// Standard Error: 1_000
			.saturating_add(Weight::from_ref_time(31_000_u64).saturating_mul(s as u64))
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	fn new_ed25519(s: u32) -> Weight {
		Weight::from_ref_time(48_672_000_u64)
			// Standard Error: 0
			.saturating_add(Weight::from_ref_time(9_000_u64).saturating_mul(s as u64))
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	fn new_secp256k1(s: u32) -> Weight {
		Weight::from_ref_time(152_477_000_u64)
			// Standard Error: 2_000
			.saturating_add(Weight::from_ref_time(2_000_u64).saturating_mul(s as u64))
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
}
