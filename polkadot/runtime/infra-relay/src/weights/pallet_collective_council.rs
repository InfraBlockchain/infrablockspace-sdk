// Copyright 2017-2022 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.
//! Autogenerated weights for `pallet_collective`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-02-27, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `bm4`, CPU: `Intel(R) Core(TM) i7-7700K CPU @ 4.20GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("infrablockchain-dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/infrablockspace
// benchmark
// pallet
// --chain=infrablockchain-dev
// --steps=50
// --repeat=20
// --pallet=pallet_collective
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --header=./file_header.txt
// --output=./runtime/infrablockspace/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_collective`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_collective::WeightInfo for WeightInfo<T> {
	/// Storage: Council Members (r:1 w:1)
	/// Proof Skipped: Council Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Council Proposals (r:1 w:0)
	/// Proof Skipped: Council Proposals (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Council Voting (r:100 w:100)
	/// Proof Skipped: Council Voting (max_values: None, max_size: None, mode: Measured)
	/// Storage: Council Prime (r:0 w:1)
	/// Proof Skipped: Council Prime (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `m` is `[0, 100]`.
	/// The range of component `n` is `[0, 100]`.
	/// The range of component `p` is `[0, 100]`.
	/// The range of component `m` is `[0, 100]`.
	/// The range of component `n` is `[0, 100]`.
	/// The range of component `p` is `[0, 100]`.
	fn set_members(m: u32, _n: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0 + m * (3233 ±0) + p * (3223 ±0)`
		//  Estimated: `16054 + m * (7809 ±16) + p * (10238 ±16)`
		// Minimum execution time: 17_913 nanoseconds.
		Weight::from_parts(18_394_000, 0)
			.saturating_add(Weight::from_parts(0, 16054))
			// Standard Error: 50_650
			.saturating_add(Weight::from_parts(5_723_694, 0).saturating_mul(m.into()))
			// Standard Error: 50_650
			.saturating_add(Weight::from_parts(8_485_753, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(p.into())))
			.saturating_add(T::DbWeight::get().writes(2))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(p.into())))
			.saturating_add(Weight::from_parts(0, 7809).saturating_mul(m.into()))
			.saturating_add(Weight::from_parts(0, 10238).saturating_mul(p.into()))
	}
	/// Storage: Council Members (r:1 w:0)
	/// Proof Skipped: Council Members (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[1, 100]`.
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[1, 100]`.
	fn execute(b: u32, m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `101 + m * (32 ±0)`
		//  Estimated: `597 + m * (32 ±0)`
		// Minimum execution time: 16_774 nanoseconds.
		Weight::from_parts(16_193_749, 0)
			.saturating_add(Weight::from_parts(0, 597))
			// Standard Error: 20
			.saturating_add(Weight::from_parts(1_790, 0).saturating_mul(b.into()))
			// Standard Error: 213
			.saturating_add(Weight::from_parts(14_817, 0).saturating_mul(m.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(Weight::from_parts(0, 32).saturating_mul(m.into()))
	}
	/// Storage: Council Members (r:1 w:0)
	/// Proof Skipped: Council Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Council ProposalOf (r:1 w:0)
	/// Proof Skipped: Council ProposalOf (max_values: None, max_size: None, mode: Measured)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[1, 100]`.
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[1, 100]`.
	fn propose_execute(b: u32, m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `101 + m * (32 ±0)`
		//  Estimated: `3174 + m * (64 ±0)`
		// Minimum execution time: 19_209 nanoseconds.
		Weight::from_parts(18_354_237, 0)
			.saturating_add(Weight::from_parts(0, 3174))
			// Standard Error: 23
			.saturating_add(Weight::from_parts(1_734, 0).saturating_mul(b.into()))
			// Standard Error: 240
			.saturating_add(Weight::from_parts(26_264, 0).saturating_mul(m.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(Weight::from_parts(0, 64).saturating_mul(m.into()))
	}
	/// Storage: Council Members (r:1 w:0)
	/// Proof Skipped: Council Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Council ProposalOf (r:1 w:1)
	/// Proof Skipped: Council ProposalOf (max_values: None, max_size: None, mode: Measured)
	/// Storage: Council Proposals (r:1 w:1)
	/// Proof Skipped: Council Proposals (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Council ProposalCount (r:1 w:1)
	/// Proof Skipped: Council ProposalCount (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Council Voting (r:0 w:1)
	/// Proof Skipped: Council Voting (max_values: None, max_size: None, mode: Measured)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[2, 100]`.
	/// The range of component `p` is `[1, 100]`.
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[2, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn propose_proposed(b: u32, m: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `423 + m * (32 ±0) + p * (36 ±0)`
		//  Estimated: `5690 + m * (165 ±0) + p * (180 ±0)`
		// Minimum execution time: 25_763 nanoseconds.
		Weight::from_parts(26_388_122, 0)
			.saturating_add(Weight::from_parts(0, 5690))
			// Standard Error: 50
			.saturating_add(Weight::from_parts(2_746, 0).saturating_mul(b.into()))
			// Standard Error: 523
			.saturating_add(Weight::from_parts(19_574, 0).saturating_mul(m.into()))
			// Standard Error: 516
			.saturating_add(Weight::from_parts(123_629, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
			.saturating_add(Weight::from_parts(0, 165).saturating_mul(m.into()))
			.saturating_add(Weight::from_parts(0, 180).saturating_mul(p.into()))
	}
	/// Storage: Council Members (r:1 w:0)
	/// Proof Skipped: Council Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Council Voting (r:1 w:1)
	/// Proof Skipped: Council Voting (max_values: None, max_size: None, mode: Measured)
	/// The range of component `m` is `[5, 100]`.
	/// The range of component `m` is `[5, 100]`.
	fn vote(m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `873 + m * (64 ±0)`
		//  Estimated: `4714 + m * (128 ±0)`
		// Minimum execution time: 22_830 nanoseconds.
		Weight::from_parts(23_632_337, 0)
			.saturating_add(Weight::from_parts(0, 4714))
			// Standard Error: 310
			.saturating_add(Weight::from_parts(43_607, 0).saturating_mul(m.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(Weight::from_parts(0, 128).saturating_mul(m.into()))
	}
	/// Storage: Council Voting (r:1 w:1)
	/// Proof Skipped: Council Voting (max_values: None, max_size: None, mode: Measured)
	/// Storage: Council Members (r:1 w:0)
	/// Proof Skipped: Council Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Council Proposals (r:1 w:1)
	/// Proof Skipped: Council Proposals (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Council ProposalOf (r:0 w:1)
	/// Proof Skipped: Council ProposalOf (max_values: None, max_size: None, mode: Measured)
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_early_disapproved(m: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `493 + m * (64 ±0) + p * (36 ±0)`
		//  Estimated: `5361 + m * (260 ±0) + p * (144 ±0)`
		// Minimum execution time: 28_031 nanoseconds.
		Weight::from_parts(28_174_563, 0)
			.saturating_add(Weight::from_parts(0, 5361))
			// Standard Error: 860
			.saturating_add(Weight::from_parts(29_198, 0).saturating_mul(m.into()))
			// Standard Error: 839
			.saturating_add(Weight::from_parts(123_119, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
			.saturating_add(Weight::from_parts(0, 260).saturating_mul(m.into()))
			.saturating_add(Weight::from_parts(0, 144).saturating_mul(p.into()))
	}
	/// Storage: Council Voting (r:1 w:1)
	/// Proof Skipped: Council Voting (max_values: None, max_size: None, mode: Measured)
	/// Storage: Council Members (r:1 w:0)
	/// Proof Skipped: Council Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Council ProposalOf (r:1 w:1)
	/// Proof Skipped: Council ProposalOf (max_values: None, max_size: None, mode: Measured)
	/// Storage: Council Proposals (r:1 w:1)
	/// Proof Skipped: Council Proposals (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_early_approved(b: u32, m: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `829 + b * (1 ±0) + m * (64 ±0) + p * (40 ±0)`
		//  Estimated: `8632 + b * (4 ±0) + m * (264 ±0) + p * (160 ±0)`
		// Minimum execution time: 39_531 nanoseconds.
		Weight::from_parts(42_121_105, 0)
			.saturating_add(Weight::from_parts(0, 8632))
			// Standard Error: 132
			.saturating_add(Weight::from_parts(1_767, 0).saturating_mul(b.into()))
			// Standard Error: 1_361
			.saturating_add(Weight::from_parts(151_993, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
			.saturating_add(Weight::from_parts(0, 4).saturating_mul(b.into()))
			.saturating_add(Weight::from_parts(0, 264).saturating_mul(m.into()))
			.saturating_add(Weight::from_parts(0, 160).saturating_mul(p.into()))
	}
	/// Storage: Council Voting (r:1 w:1)
	/// Proof Skipped: Council Voting (max_values: None, max_size: None, mode: Measured)
	/// Storage: Council Members (r:1 w:0)
	/// Proof Skipped: Council Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Council Prime (r:1 w:0)
	/// Proof Skipped: Council Prime (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Council Proposals (r:1 w:1)
	/// Proof Skipped: Council Proposals (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Council ProposalOf (r:0 w:1)
	/// Proof Skipped: Council ProposalOf (max_values: None, max_size: None, mode: Measured)
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_disapproved(m: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `513 + m * (64 ±0) + p * (36 ±0)`
		//  Estimated: `6430 + m * (325 ±0) + p * (180 ±0)`
		// Minimum execution time: 30_738 nanoseconds.
		Weight::from_parts(31_359_687, 0)
			.saturating_add(Weight::from_parts(0, 6430))
			// Standard Error: 445
			.saturating_add(Weight::from_parts(27_105, 0).saturating_mul(m.into()))
			// Standard Error: 434
			.saturating_add(Weight::from_parts(119_167, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
			.saturating_add(Weight::from_parts(0, 325).saturating_mul(m.into()))
			.saturating_add(Weight::from_parts(0, 180).saturating_mul(p.into()))
	}
	/// Storage: Council Voting (r:1 w:1)
	/// Proof Skipped: Council Voting (max_values: None, max_size: None, mode: Measured)
	/// Storage: Council Members (r:1 w:0)
	/// Proof Skipped: Council Members (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Council Prime (r:1 w:0)
	/// Proof Skipped: Council Prime (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Council ProposalOf (r:1 w:1)
	/// Proof Skipped: Council ProposalOf (max_values: None, max_size: None, mode: Measured)
	/// Storage: Council Proposals (r:1 w:1)
	/// Proof Skipped: Council Proposals (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_approved(b: u32, m: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `849 + b * (1 ±0) + m * (64 ±0) + p * (40 ±0)`
		//  Estimated: `9900 + b * (5 ±0) + m * (330 ±0) + p * (200 ±0)`
		// Minimum execution time: 41_645 nanoseconds.
		Weight::from_parts(43_504_215, 0)
			.saturating_add(Weight::from_parts(0, 9900))
			// Standard Error: 87
			.saturating_add(Weight::from_parts(2_337, 0).saturating_mul(b.into()))
			// Standard Error: 928
			.saturating_add(Weight::from_parts(21_553, 0).saturating_mul(m.into()))
			// Standard Error: 905
			.saturating_add(Weight::from_parts(148_246, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
			.saturating_add(Weight::from_parts(0, 5).saturating_mul(b.into()))
			.saturating_add(Weight::from_parts(0, 330).saturating_mul(m.into()))
			.saturating_add(Weight::from_parts(0, 200).saturating_mul(p.into()))
	}
	/// Storage: Council Proposals (r:1 w:1)
	/// Proof Skipped: Council Proposals (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Council Voting (r:0 w:1)
	/// Proof Skipped: Council Voting (max_values: None, max_size: None, mode: Measured)
	/// Storage: Council ProposalOf (r:0 w:1)
	/// Proof Skipped: Council ProposalOf (max_values: None, max_size: None, mode: Measured)
	/// The range of component `p` is `[1, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn disapprove_proposal(p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `258 + p * (32 ±0)`
		//  Estimated: `1269 + p * (96 ±0)`
		// Minimum execution time: 15_758 nanoseconds.
		Weight::from_parts(17_993_925, 0)
			.saturating_add(Weight::from_parts(0, 1269))
			// Standard Error: 504
			.saturating_add(Weight::from_parts(111_766, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(3))
			.saturating_add(Weight::from_parts(0, 96).saturating_mul(p.into()))
	}
}
