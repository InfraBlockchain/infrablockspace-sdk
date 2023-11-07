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

//! Autogenerated weights for `pallet_bridge_parachains`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-05-31, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `bm4`, CPU: `Intel(R) Core(TM) i7-7700K CPU @ 4.20GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("bridge-hub-rococo-dev"), DB CACHE: 1024

// Executed Command:
// ./artifacts/polkadot-parachain
// benchmark
// pallet
// --chain=bridge-hub-rococo-dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_bridge_parachains
// --extrinsic=*
// --steps=50
// --repeat=20
// --json
// --header=./file_header.txt
// --output=./parachains/runtimes/bridge-hubs/bridge-hub-rococo/src/weights/pallet_bridge_parachains.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_bridge_parachains`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_bridge_parachains::WeightInfo for WeightInfo<T> {
	/// Storage: BridgeWococoParachain PalletOperatingMode (r:1 w:0)
	/// Proof: BridgeWococoParachain PalletOperatingMode (max_values: Some(1), max_size: Some(1), added: 496, mode: MaxEncodedLen)
	/// Storage: BridgeWococoGrandpa ImportedHeaders (r:1 w:0)
	/// Proof: BridgeWococoGrandpa ImportedHeaders (max_values: Some(1024), max_size: Some(68), added: 1553, mode: MaxEncodedLen)
	/// Storage: BridgeWococoParachain ParasInfo (r:1 w:1)
	/// Proof: BridgeWococoParachain ParasInfo (max_values: Some(1), max_size: Some(60), added: 555, mode: MaxEncodedLen)
	/// Storage: BridgeWococoParachain ImportedParaHashes (r:1 w:1)
	/// Proof: BridgeWococoParachain ImportedParaHashes (max_values: Some(64), max_size: Some(64), added: 1054, mode: MaxEncodedLen)
	/// Storage: BridgeWococoParachain ImportedParaHeads (r:0 w:1)
	/// Proof: BridgeWococoParachain ImportedParaHeads (max_values: Some(64), max_size: Some(196), added: 1186, mode: MaxEncodedLen)
	/// The range of component `p` is `[1, 2]`.
	/// The range of component `p` is `[1, 2]`.
	fn submit_parachain_heads_with_n_parachains(_p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `367`
		//  Estimated: `2543`
		// Minimum execution time: 34_759_000 picoseconds.
		Weight::from_parts(35_709_034, 0)
			.saturating_add(Weight::from_parts(0, 2543))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: BridgeWococoParachain PalletOperatingMode (r:1 w:0)
	/// Proof: BridgeWococoParachain PalletOperatingMode (max_values: Some(1), max_size: Some(1), added: 496, mode: MaxEncodedLen)
	/// Storage: BridgeWococoGrandpa ImportedHeaders (r:1 w:0)
	/// Proof: BridgeWococoGrandpa ImportedHeaders (max_values: Some(1024), max_size: Some(68), added: 1553, mode: MaxEncodedLen)
	/// Storage: BridgeWococoParachain ParasInfo (r:1 w:1)
	/// Proof: BridgeWococoParachain ParasInfo (max_values: Some(1), max_size: Some(60), added: 555, mode: MaxEncodedLen)
	/// Storage: BridgeWococoParachain ImportedParaHashes (r:1 w:1)
	/// Proof: BridgeWococoParachain ImportedParaHashes (max_values: Some(64), max_size: Some(64), added: 1054, mode: MaxEncodedLen)
	/// Storage: BridgeWococoParachain ImportedParaHeads (r:0 w:1)
	/// Proof: BridgeWococoParachain ImportedParaHeads (max_values: Some(64), max_size: Some(196), added: 1186, mode: MaxEncodedLen)
	fn submit_parachain_heads_with_1kb_proof() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `367`
		//  Estimated: `2543`
		// Minimum execution time: 36_005_000 picoseconds.
		Weight::from_parts(36_492_000, 0)
			.saturating_add(Weight::from_parts(0, 2543))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: BridgeWococoParachain PalletOperatingMode (r:1 w:0)
	/// Proof: BridgeWococoParachain PalletOperatingMode (max_values: Some(1), max_size: Some(1), added: 496, mode: MaxEncodedLen)
	/// Storage: BridgeWococoGrandpa ImportedHeaders (r:1 w:0)
	/// Proof: BridgeWococoGrandpa ImportedHeaders (max_values: Some(1024), max_size: Some(68), added: 1553, mode: MaxEncodedLen)
	/// Storage: BridgeWococoParachain ParasInfo (r:1 w:1)
	/// Proof: BridgeWococoParachain ParasInfo (max_values: Some(1), max_size: Some(60), added: 555, mode: MaxEncodedLen)
	/// Storage: BridgeWococoParachain ImportedParaHashes (r:1 w:1)
	/// Proof: BridgeWococoParachain ImportedParaHashes (max_values: Some(64), max_size: Some(64), added: 1054, mode: MaxEncodedLen)
	/// Storage: BridgeWococoParachain ImportedParaHeads (r:0 w:1)
	/// Proof: BridgeWococoParachain ImportedParaHeads (max_values: Some(64), max_size: Some(196), added: 1186, mode: MaxEncodedLen)
	fn submit_parachain_heads_with_16kb_proof() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `367`
		//  Estimated: `2543`
		// Minimum execution time: 62_374_000 picoseconds.
		Weight::from_parts(62_977_000, 0)
			.saturating_add(Weight::from_parts(0, 2543))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
}