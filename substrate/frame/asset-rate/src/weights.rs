// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Autogenerated weights for `pallet_asset_rate`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 32.0.0
//! DATE: 2024-04-08, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `runner-anb7yjbi-project-674-concurrent-0`, CPU: `Intel(R) Xeon(R) CPU @ 2.60GHz`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("dev")`, DB CACHE: `1024`

// Executed Command:
// ./target/production/substrate-node
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=pallet_asset_rate
// --no-storage-info
// --no-median-slopes
// --no-min-squares
// --extrinsic=*
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./substrate/frame/asset-rate/src/weights.rs
// --header=./substrate/HEADER-APACHE2
// --template=./substrate/.maintain/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for `pallet_asset_rate`.
pub trait WeightInfo {
	fn create() -> Weight;
	fn update() -> Weight;
	fn remove() -> Weight;
}

/// Weights for `pallet_asset_rate` using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: `AssetRate::ConversionRateToNative` (r:1 w:1)
	/// Proof: `AssetRate::ConversionRateToNative` (`max_values`: None, `max_size`: Some(36), added: 2511, mode: `MaxEncodedLen`)
	fn create() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `76`
		//  Estimated: `3501`
		// Minimum execution time: 9_816_000 picoseconds.
		Weight::from_parts(10_076_000, 3501)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `AssetRate::ConversionRateToNative` (r:1 w:1)
	/// Proof: `AssetRate::ConversionRateToNative` (`max_values`: None, `max_size`: Some(36), added: 2511, mode: `MaxEncodedLen`)
	fn update() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `137`
		//  Estimated: `3501`
		// Minimum execution time: 10_164_000 picoseconds.
		Weight::from_parts(10_598_000, 3501)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `AssetRate::ConversionRateToNative` (r:1 w:1)
	/// Proof: `AssetRate::ConversionRateToNative` (`max_values`: None, `max_size`: Some(36), added: 2511, mode: `MaxEncodedLen`)
	fn remove() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `137`
		//  Estimated: `3501`
		// Minimum execution time: 10_837_000 picoseconds.
		Weight::from_parts(11_050_000, 3501)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
}

// For backwards compatibility and tests.
impl WeightInfo for () {
	/// Storage: `AssetRate::ConversionRateToNative` (r:1 w:1)
	/// Proof: `AssetRate::ConversionRateToNative` (`max_values`: None, `max_size`: Some(36), added: 2511, mode: `MaxEncodedLen`)
	fn create() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `76`
		//  Estimated: `3501`
		// Minimum execution time: 9_816_000 picoseconds.
		Weight::from_parts(10_076_000, 3501)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `AssetRate::ConversionRateToNative` (r:1 w:1)
	/// Proof: `AssetRate::ConversionRateToNative` (`max_values`: None, `max_size`: Some(36), added: 2511, mode: `MaxEncodedLen`)
	fn update() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `137`
		//  Estimated: `3501`
		// Minimum execution time: 10_164_000 picoseconds.
		Weight::from_parts(10_598_000, 3501)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `AssetRate::ConversionRateToNative` (r:1 w:1)
	/// Proof: `AssetRate::ConversionRateToNative` (`max_values`: None, `max_size`: Some(36), added: 2511, mode: `MaxEncodedLen`)
	fn remove() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `137`
		//  Estimated: `3501`
		// Minimum execution time: 10_837_000 picoseconds.
		Weight::from_parts(11_050_000, 3501)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
}
