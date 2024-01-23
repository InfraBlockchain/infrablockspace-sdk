use frame_support::pallet_prelude::*;
use frame_system::{ensure_root, pallet_prelude::*};
use pallet_validator_election::VotingInterface;
use parity_scale_codec::Encode;
use sp_runtime::types::{fee::*, infra_core::*, token::*, vote::*};
use sp_std::vec::Vec;
use xcm::latest::prelude::*;

mod impls;
mod types;

pub use pallet::*;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct SystemTokenDetail {
	status: SystemTokenStatus,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum SystemTokenStatus {
	/// Potential System Token is requsted
	Requested,
	/// System Token is registered by RC governance
	Registered,
	/// System Token is suspended by some reasons(e.g malicious behavior detected)
	Suspend,
	/// System Token is deregistered by some reasons
	Deregistered,
}

#[frame_support::pallet(dev_mode)]
pub mod pallet {

	use super::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Updating vote type
		type VotingInterface: VotingInterface<Self>;
		/// Managing System Token
		type SystemTokenInterface: SystemTokenInterface;
		/// Type that interacts with local asset
		type LocalAssetManager: LocalAssetManager;
		/// Type that links asset with System Token
		type AssetLink: AssetLinkInterface<SystemTokenAssetId>;
		/// Type that delivers XCM messages
		type XcmRouter: SendXcm;
	}

	/// Base system token configuration for Runtime
	#[pallet::storage]
	pub type BaseConfiguration<T: Config> = StorageValue<_, BaseSystemTokenDetail, OptionQuery>;

	#[pallet::storage]
	pub type FeeRate<T: Config> = StorageValue<_, SystemTokenWeight, OptionQuery>;

	#[pallet::storage]
	pub(super) type RuntimeState<T: Config> = StorageValue<_, Mode, ValueQuery>;

	#[pallet::storage]
	pub type FeeTable<T: Config> =
		StorageMap<_, Twox128, ExtrinsicMetadata, SystemTokenBalance, OptionQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub base_detail: Option<(Fiat, SystemTokenWeight, SystemTokenDecimal)>,
		pub _phantom: sp_std::marker::PhantomData<T>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			if let Some(base_detail) = self.base_detail.clone() {
				BaseConfiguration::<T>::put(BaseSystemTokenDetail {
					currency: base_detail.0,
					weight: base_detail.1,
					decimal: base_detail.2,
				});
			}
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Voted {
			who: VoteAccountId,
			system_token_id: SystemTokenId,
			vote_weight: VoteWeight,
		},
		/// System Token has been regierested by Relay-chain governance
		Registered,
		/// System Token has been deregistered by Relay-chain governance
		Deregistered,
		/// Fee table for has been updated by Relay-chain governance
		FeeTableUpdated {
			extrinsic_metadata: ExtrinsicMetadata,
			fee: SystemTokenBalance,
		},
		/// Weight of System Token has been updated by Relay-chain governance
		SystemTokenWeightUpdated {
			asset_id: SystemTokenAssetId,
		},
		/// Bootstrap has been ended by Relay-chain governance.
		BootstrapEnded,
		/// Asset is linked since it has registered as System Token by Relay-chain governance
		AssetLinked {
			asset_id: SystemTokenAssetId,
			multi_loc: MultiLocation,
		},
		/// Asset is unlinked by Relay-chain governance
		AssetUnlinked {
			asset_id: SystemTokenAssetId,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Current Runtime state is not ready to change
		NotAllowedToChangeState,
		/// Error occured while registering System Token
		ErrorRegisterSystemToken,
		/// Error occured while updating weight of System Token
		ErrorUpdateWeight,
		/// Error occured while creating wrapped local asset
		ErrorCreateWrappedLocal,
		/// Error occured while linking asset
		ErrorLinkAsset,
		/// Error occured while deregistering asset
		ErrorDeregisterSystemToken,
		/// Base configuration should be set
		BaseNotConfigured,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Fee table for Runtime will be set by Relay-chain governance
		///
		/// Origin
		/// Relay-chain governance
		#[pallet::call_index(0)]
		pub fn set_fee_table(
			origin: OriginFor<T>,
			pallet_name: Vec<u8>,
			call_name: Vec<u8>,
			fee: SystemTokenBalance,
		) -> DispatchResult {
			ensure_root(origin)?;
			let extrinsic_metadata = ExtrinsicMetadata::new(pallet_name, call_name);
			FeeTable::<T>::insert(&extrinsic_metadata, fee);
			Self::deposit_event(Event::<T>::FeeTableUpdated { extrinsic_metadata, fee });
			Ok(())
		}

		/// Fee rate for Runtime will be set by Relay-chain governance
		///
		/// Origin
		/// Relay-chain governance
		#[pallet::call_index(1)]
		pub fn set_fee_rate(origin: OriginFor<T>, fee_rate: SystemTokenWeight) -> DispatchResult {
			ensure_root(origin)?;
			FeeRate::<T>::put(fee_rate);
			Ok(())
		}

		/// Set runtime state configuration
		///
		/// Origin
		/// Relay-chain governance
		#[pallet::call_index(2)]
		pub fn set_runtime_state(origin: OriginFor<T>) -> DispatchResult {
			ensure_root(origin)?;
			if RuntimeState::<T>::get() == Mode::Normal {
				return Ok(())
			}
			// TODO-1: Check whether it is allowed to change `Normal` state
			// ToDo-2: Check whether a parachain has enough system token to pay
			RuntimeState::<T>::put(Mode::Normal);
			Self::deposit_event(Event::<T>::BootstrapEnded);
			Ok(())
		}

		/// Description
		/// This method is for emergency case. Naturally it would be set automatically
		///
		/// Origin
		/// Relay-chain governance
		#[pallet::call_index(3)]
		pub fn set_system_token_weight(
			origin: OriginFor<T>,
			asset_id: SystemTokenAssetId,
			system_token_weight: SystemTokenWeight,
		) -> DispatchResult {
			ensure_root(origin)?;
			T::LocalAssetManager::update_system_token_weight(asset_id, system_token_weight)
				.map_err(|_| Error::<T>::ErrorUpdateWeight)?;
			Self::deposit_event(Event::<T>::SystemTokenWeightUpdated { asset_id });
			Ok(())
		}

		/// Description
		/// This method is for emergency case. Naturally it would be set automatically
		/// Origin
		/// Relay-chain governance
		#[pallet::call_index(4)]
		pub fn register_system_token(
			origin: OriginFor<T>,
			asset_id: SystemTokenAssetId,
			system_token_weight: SystemTokenWeight,
		) -> DispatchResult {
			ensure_root(origin)?;
			T::LocalAssetManager::promote(asset_id, system_token_weight)
				.map_err(|_| Error::<T>::ErrorRegisterSystemToken)?;
			Ok(())
		}

		/// Description
		/// This method is for emergency case. Naturally it would be set automatically
		///
		/// Origin
		/// Relay-chain governance
		#[pallet::call_index(5)]
		pub fn create_wrapped_local(
			origin: OriginFor<T>,
			asset_id: SystemTokenAssetId,
			currency_type: Fiat,
			min_balance: SystemTokenBalance,
			name: Vec<u8>,
			symbol: Vec<u8>,
			decimals: u8,
			asset_link_parent: u8,
			original: SystemTokenId,
			system_token_weight: SystemTokenWeight,
		) -> DispatchResult {
			ensure_root(origin)?;
			T::LocalAssetManager::create_wrapped_local(
				asset_id,
				currency_type,
				min_balance,
				name,
				symbol,
				decimals,
				system_token_weight,
			)
			.map_err(|_| Error::<T>::ErrorCreateWrappedLocal)?;
			T::AssetLink::link(&asset_id, asset_link_parent, original)
				.map_err(|_| Error::<T>::ErrorLinkAsset)?;
			Ok(())
		}

		#[pallet::call_index(6)]
		pub fn deregister_system_token(
			origin: OriginFor<T>,
			asset_id: SystemTokenAssetId,
		) -> DispatchResult {
			ensure_root(origin)?;
			T::LocalAssetManager::demote(asset_id)
				.map_err(|_| Error::<T>::ErrorRegisterSystemToken)?;
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {}
