// Copyright (C) Parity Technologies (UK) Ltd.
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

//! Various implementations and utilities for matching and filtering `Location` and
//! `InteriorLocation` types.

use frame_support::traits::{Contains, Get};
use xcm::latest::{InteriorMultiLocation, MultiLocation, NetworkId};

/// An implementation of `Contains` that checks for `Location` or
/// `InteriorLocation` if starts with the provided type `T`.
pub struct StartsWith<T, L = MultiLocation>(sp_std::marker::PhantomData<(T, L)>);
impl<T: Get<L>, L: TryInto<MultiLocation> + Clone> Contains<L> for StartsWith<T, L> {
	fn contains(location: &L) -> bool {
		let latest_location: MultiLocation =
			if let Ok(location) = (*location).clone().try_into() { location } else { return false };
		let latest_t = if let Ok(location) = T::get().try_into() { location } else { return false };
		latest_location.starts_with(&latest_t)
	}
}
impl<T: Get<InteriorMultiLocation>> Contains<InteriorMultiLocation> for StartsWith<T> {
	fn contains(t: &InteriorMultiLocation) -> bool {
		t.starts_with(&T::get())
	}
}

/// An implementation of `Contains` that checks for `Location` or
/// `InteriorLocation` if starts with expected `GlobalConsensus(NetworkId)` provided as type
/// `T`.
pub struct StartsWithExplicitGlobalConsensus<T>(sp_std::marker::PhantomData<T>);
impl<T: Get<NetworkId>> Contains<MultiLocation> for StartsWithExplicitGlobalConsensus<T> {
	fn contains(location: &MultiLocation) -> bool {
		matches!(location.interior().global_consensus(), Ok(requested_network) if requested_network.eq(&T::get()))
	}
}
impl<T: Get<NetworkId>> Contains<InteriorMultiLocation> for StartsWithExplicitGlobalConsensus<T> {
	fn contains(location: &InteriorMultiLocation) -> bool {
		matches!(location.global_consensus(), Ok(requested_network) if requested_network.eq(&T::get()))
	}
}
