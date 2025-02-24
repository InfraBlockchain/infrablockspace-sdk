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

//! Configuration manager for the Polkadot runtime parachains logic.
//!
//! Configuration can change only at session boundaries and is buffered until then.

use crate::{inclusion::MAX_UPWARD_MESSAGE_SIZE_BOUND, shared};
use frame_support::{pallet_prelude::*, DefaultNoBound};
use frame_system::pallet_prelude::*;
use parachain_primitives::primitives::{MAX_HORIZONTAL_MESSAGE_NUM, MAX_UPWARD_MESSAGE_NUM};
use parity_scale_codec::{Decode, Encode};
use primitives::{
	AsyncBackingParams, Balance, ExecutorParams, SessionIndex, LEGACY_MIN_BACKING_VOTES,
	MAX_CODE_SIZE, MAX_HEAD_DATA_SIZE, MAX_POV_SIZE, ON_DEMAND_DEFAULT_QUEUE_MAX_SIZE,
};
use sp_arithmetic::traits::AtLeast32BitUnsigned;
use sp_runtime::{infra::*, traits::Zero, Perbill};
use sp_std::prelude::*;

type SystemTokenBalanceOf<T> = <<T as Config>::ParaConfigHandler as ParaConfigInterface>::Balance;
type DestIdOf<T> = <<T as Config>::ParaConfigHandler as ParaConfigInterface>::DestId;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod migration;

pub use pallet::*;

const LOG_TARGET: &str = "runtime::configuration";

/// All configuration of the runtime with respect to paras.
#[derive(
	Clone,
	Encode,
	Decode,
	PartialEq,
	sp_core::RuntimeDebug,
	scale_info::TypeInfo,
	serde::Serialize,
	serde::Deserialize,
)]
pub struct HostConfiguration<BlockNumber> {
	// NOTE: This structure is used by parachains via merkle proofs. Therefore, this struct
	// requires special treatment.
	//
	// A parachain requested this struct can only depend on the subset of this struct.
	// Specifically, only a first few fields can be depended upon. These fields cannot be changed
	// without corresponding migration of the parachains.
	/**
	 * The parameters that are required for the parachains.
	 */

	/// The maximum validation code size, in bytes.
	pub max_code_size: u32,
	/// The maximum head-data size, in bytes.
	pub max_head_data_size: u32,
	/// Total number of individual messages allowed in the parachain -> relay-chain message queue.
	pub max_upward_queue_count: u32,
	/// Total size of messages allowed in the parachain -> relay-chain message queue before which
	/// no further messages may be added to it. If it exceeds this then the queue may contain only
	/// a single message.
	pub max_upward_queue_size: u32,
	/// The maximum size of an upward message that can be sent by a candidate.
	///
	/// This parameter affects the size upper bound of the `CandidateCommitments`.
	pub max_upward_message_size: u32,
	/// The maximum number of messages that a candidate can contain.
	///
	/// This parameter affects the size upper bound of the `CandidateCommitments`.
	pub max_upward_message_num_per_candidate: u32,
	/// The maximum number of outbound HRMP messages can be sent by a candidate.
	///
	/// This parameter affects the upper bound of size of `CandidateCommitments`.
	pub hrmp_max_message_num_per_candidate: u32,
	/// The minimum period, in blocks, between which parachains can update their validation code.
	///
	/// This number is used to prevent parachains from spamming the relay chain with validation
	/// code upgrades. The only thing it controls is the number of blocks the
	/// `UpgradeRestrictionSignal` is set for the parachain in question.
	///
	/// If PVF pre-checking is enabled this should be greater than the maximum number of blocks
	/// PVF pre-checking can take. Intuitively, this number should be greater than the duration
	/// specified by [`pvf_voting_ttl`](Self::pvf_voting_ttl). Unlike,
	/// [`pvf_voting_ttl`](Self::pvf_voting_ttl), this parameter uses blocks as a unit.
	#[cfg_attr(feature = "std", serde(alias = "validation_upgrade_frequency"))]
	pub validation_upgrade_cooldown: BlockNumber,
	/// The delay, in blocks, after which an upgrade of the validation code is applied.
	///
	/// The upgrade for a parachain takes place when the first candidate which has relay-parent >=
	/// the relay-chain block where the upgrade is scheduled. This block is referred as to
	/// `expected_at`.
	///
	/// `expected_at` is determined when the upgrade is scheduled. This happens when the candidate
	/// that signals the upgrade is enacted. Right now, the relay-parent block number of the
	/// candidate scheduling the upgrade is used to determine the `expected_at`. This may change in
	/// the future with [#4601].
	///
	/// When PVF pre-checking is enabled, the upgrade is scheduled only after the PVF pre-check has
	/// been completed.
	///
	/// Note, there are situations in which `expected_at` in the past. For example, if
	/// [`paras_availability_period`](Self::paras_availability_period) is less than the delay set
	/// by this field or if PVF pre-check took more time than the delay. In such cases, the upgrade
	/// is further at the earliest possible time determined by
	/// [`minimum_validation_upgrade_delay`](Self::minimum_validation_upgrade_delay).
	///
	/// The rationale for this delay has to do with relay-chain reversions. In case there is an
	/// invalid candidate produced with the new version of the code, then the relay-chain can
	/// revert [`validation_upgrade_delay`](Self::validation_upgrade_delay) many blocks back and
	/// still find the new code in the storage by hash.
	///
	/// [#4601]: https://github.com/paritytech/polkadot/issues/4601
	pub validation_upgrade_delay: BlockNumber,
	/// Asynchronous backing parameters.
	pub async_backing_params: AsyncBackingParams,

	/**
	 * The parameters that are not essential, but still may be of interest for parachains.
	 */

	/// The maximum POV block size, in bytes.
	pub max_pov_size: u32,
	/// The maximum size of a message that can be put in a downward message queue.
	///
	/// Since we require receiving at least one DMP message the obvious upper bound of the size is
	/// the PoV size. Of course, there is a lot of other different things that a parachain may
	/// decide to do with its PoV so this value in practice will be picked as a fraction of the PoV
	/// size.
	pub max_downward_message_size: u32,
	/// The maximum number of outbound HRMP channels a parachain is allowed to open.
	pub hrmp_max_parachain_outbound_channels: u32,
	/// The deposit that the sender should provide for opening an HRMP channel.
	pub hrmp_sender_deposit: Balance,
	/// The deposit that the recipient should provide for accepting opening an HRMP channel.
	pub hrmp_recipient_deposit: Balance,
	/// The maximum number of messages allowed in an HRMP channel at once.
	pub hrmp_channel_max_capacity: u32,
	/// The maximum total size of messages in bytes allowed in an HRMP channel at once.
	pub hrmp_channel_max_total_size: u32,
	/// The maximum number of inbound HRMP channels a parachain is allowed to accept.
	pub hrmp_max_parachain_inbound_channels: u32,
	/// The maximum size of a message that could ever be put into an HRMP channel.
	///
	/// This parameter affects the upper bound of size of `CandidateCommitments`.
	pub hrmp_channel_max_message_size: u32,
	/// The executor environment parameters
	pub executor_params: ExecutorParams,

	/**
	 * Parameters that will unlikely be needed by parachains.
	 */

	/// How long to keep code on-chain, in blocks. This should be sufficiently long that disputes
	/// have concluded.
	pub code_retention_period: BlockNumber,
	/// The amount of execution cores to dedicate to on demand execution.
	pub on_demand_cores: u32,
	/// The number of retries that a on demand author has to submit their block.
	pub on_demand_retries: u32,
	/// The maximum queue size of the pay as you go module.
	pub on_demand_queue_max_size: u32,
	/// The target utilization of the spot price queue in percentages.
	pub on_demand_target_queue_utilization: Perbill,
	/// How quickly the fee rises in reaction to increased utilization.
	/// The lower the number the slower the increase.
	pub on_demand_fee_variability: Perbill,
	/// The minimum amount needed to claim a slot in the spot pricing queue.
	pub on_demand_base_fee: Balance,
	/// The number of blocks an on demand claim stays in the scheduler's claimqueue before getting
	/// cleared. This number should go reasonably higher than the number of blocks in the async
	/// backing lookahead.
	pub on_demand_ttl: BlockNumber,
	/// How often parachain groups should be rotated across parachains.
	///
	/// Must be non-zero.
	pub group_rotation_frequency: BlockNumber,
	/// The minimum availability period, in blocks.
	///
	/// This is the minimum amount of blocks after a core became occupied that validators have time
	/// to make the block available.
	///
	/// This value only has effect on group rotations. If backers backed something at the end of
	/// their rotation, the occupied core affects the backing group that comes afterwards. We limit
	/// the effect one backing group can have on the next to `paras_availability_period` blocks.
	///
	/// Within a group rotation there is no timeout as backers are only affecting themselves.
	///
	/// Must be at least 1. With a value of 1, the previous group will not be able to negatively
	/// affect the following group at the expense of a tight availability timeline at group
	/// rotation boundaries.
	pub paras_availability_period: BlockNumber,
	/// The amount of blocks ahead to schedule paras.
	pub scheduling_lookahead: u32,
	/// The maximum number of validators to have per core.
	///
	/// `None` means no maximum.
	pub max_validators_per_core: Option<u32>,
	/// The maximum number of validators to use for parachain consensus, period.
	///
	/// `None` means no maximum.
	pub max_validators: Option<u32>,
	/// The amount of sessions to keep for disputes.
	pub dispute_period: SessionIndex,
	/// How long after dispute conclusion to accept statements.
	pub dispute_post_conclusion_acceptance_period: BlockNumber,
	/// The amount of consensus slots that must pass between submitting an assignment and
	/// submitting an approval vote before a validator is considered a no-show.
	///
	/// Must be at least 1.
	pub no_show_slots: u32,
	/// The number of delay tranches in total.
	pub n_delay_tranches: u32,
	/// The width of the zeroth delay tranche for approval assignments. This many delay tranches
	/// beyond 0 are all consolidated to form a wide 0 tranche.
	pub zeroth_delay_tranche_width: u32,
	/// The number of validators needed to approve a block.
	pub needed_approvals: u32,
	/// The number of samples to do of the `RelayVRFModulo` approval assignment criterion.
	pub relay_vrf_modulo_samples: u32,
	/// If an active PVF pre-checking vote observes this many number of sessions it gets
	/// automatically rejected.
	///
	/// 0 means PVF pre-checking will be rejected on the first observed session unless the voting
	/// gained supermajority before that the session change.
	pub pvf_voting_ttl: SessionIndex,
	/// The lower bound number of blocks an upgrade can be scheduled.
	///
	/// Typically, upgrade gets scheduled
	/// [`validation_upgrade_delay`](Self::validation_upgrade_delay) relay-chain blocks after
	/// the relay-parent of the parablock that signalled the validation code upgrade. However,
	/// in the case a pre-checking voting was concluded in a longer duration the upgrade will be
	/// scheduled to the next block.
	///
	/// That can disrupt parachain inclusion. Specifically, it will make the blocks that were
	/// already backed invalid.
	///
	/// To prevent that, we introduce the minimum number of blocks after which the upgrade can be
	/// scheduled. This number is controlled by this field.
	///
	/// This value should be greater than
	/// [`paras_availability_period`](Self::paras_availability_period).
	pub minimum_validation_upgrade_delay: BlockNumber,
	/// The minimum number of valid backing statements required to consider a parachain candidate
	/// backable.
	pub minimum_backing_votes: u32,
}

impl<BlockNumber: Default + From<u32>> Default for HostConfiguration<BlockNumber> {
	fn default() -> Self {
		Self {
			async_backing_params: AsyncBackingParams {
				max_candidate_depth: 0,
				allowed_ancestry_len: 0,
			},
			group_rotation_frequency: 1u32.into(),
			paras_availability_period: 1u32.into(),
			no_show_slots: 1u32.into(),
			validation_upgrade_cooldown: Default::default(),
			validation_upgrade_delay: 2u32.into(),
			code_retention_period: Default::default(),
			max_code_size: Default::default(),
			max_pov_size: Default::default(),
			max_head_data_size: Default::default(),
			on_demand_cores: Default::default(),
			on_demand_retries: Default::default(),
			scheduling_lookahead: 1,
			max_validators_per_core: Default::default(),
			max_validators: None,
			dispute_period: 6,
			dispute_post_conclusion_acceptance_period: 100.into(),
			n_delay_tranches: Default::default(),
			zeroth_delay_tranche_width: Default::default(),
			needed_approvals: Default::default(),
			relay_vrf_modulo_samples: Default::default(),
			max_upward_queue_count: Default::default(),
			max_upward_queue_size: Default::default(),
			max_downward_message_size: Default::default(),
			max_upward_message_size: Default::default(),
			max_upward_message_num_per_candidate: Default::default(),
			hrmp_sender_deposit: Default::default(),
			hrmp_recipient_deposit: Default::default(),
			hrmp_channel_max_capacity: Default::default(),
			hrmp_channel_max_total_size: Default::default(),
			hrmp_max_parachain_inbound_channels: Default::default(),
			hrmp_channel_max_message_size: Default::default(),
			hrmp_max_parachain_outbound_channels: Default::default(),
			hrmp_max_message_num_per_candidate: Default::default(),
			pvf_voting_ttl: 2u32.into(),
			minimum_validation_upgrade_delay: 2.into(),
			executor_params: Default::default(),
			on_demand_queue_max_size: ON_DEMAND_DEFAULT_QUEUE_MAX_SIZE,
			on_demand_base_fee: 10_000_000u128,
			on_demand_fee_variability: Perbill::from_percent(3),
			on_demand_target_queue_utilization: Perbill::from_percent(25),
			on_demand_ttl: 5u32.into(),
			minimum_backing_votes: LEGACY_MIN_BACKING_VOTES,
		}
	}
}

/// Enumerates the possible inconsistencies of `HostConfiguration`.
#[derive(Debug)]
pub enum InconsistentError<BlockNumber> {
	/// `group_rotation_frequency` is set to zero.
	ZeroGroupRotationFrequency,
	/// `paras_availability_period` is set to zero.
	ZeroParasAvailabilityPeriod,
	/// `no_show_slots` is set to zero.
	ZeroNoShowSlots,
	/// `max_code_size` exceeds the hard limit of `MAX_CODE_SIZE`.
	MaxCodeSizeExceedHardLimit { max_code_size: u32 },
	/// `max_head_data_size` exceeds the hard limit of `MAX_HEAD_DATA_SIZE`.
	MaxHeadDataSizeExceedHardLimit { max_head_data_size: u32 },
	/// `max_pov_size` exceeds the hard limit of `MAX_POV_SIZE`.
	MaxPovSizeExceedHardLimit { max_pov_size: u32 },
	/// `minimum_validation_upgrade_delay` is less than `paras_availability_period`.
	MinimumValidationUpgradeDelayLessThanChainAvailabilityPeriod {
		minimum_validation_upgrade_delay: BlockNumber,
		paras_availability_period: BlockNumber,
	},
	/// `validation_upgrade_delay` is less than or equal 1.
	ValidationUpgradeDelayIsTooLow { validation_upgrade_delay: BlockNumber },
	/// Maximum UMP message size ([`MAX_UPWARD_MESSAGE_SIZE_BOUND`]) exceeded.
	MaxUpwardMessageSizeExceeded { max_message_size: u32 },
	/// Maximum HRMP message num ([`MAX_HORIZONTAL_MESSAGE_NUM`]) exceeded.
	MaxHorizontalMessageNumExceeded { max_message_num: u32 },
	/// Maximum UMP message num ([`MAX_UPWARD_MESSAGE_NUM`]) exceeded.
	MaxUpwardMessageNumExceeded { max_message_num: u32 },
	/// Maximum number of HRMP outbound channels exceeded.
	MaxHrmpOutboundChannelsExceeded,
	/// Maximum number of HRMP inbound channels exceeded.
	MaxHrmpInboundChannelsExceeded,
	/// `minimum_backing_votes` is set to zero.
	ZeroMinimumBackingVotes,
}

impl<BlockNumber> HostConfiguration<BlockNumber>
where
	BlockNumber: Zero + PartialOrd + sp_std::fmt::Debug + Clone + From<u32>,
{
	/// Checks that this instance is consistent with the requirements on each individual member.
	///
	/// # Errors
	///
	/// This function returns an error if the configuration is inconsistent.
	pub fn check_consistency(&self) -> Result<(), InconsistentError<BlockNumber>> {
		use InconsistentError::*;

		if self.group_rotation_frequency.is_zero() {
			return Err(ZeroGroupRotationFrequency)
		}

		if self.paras_availability_period.is_zero() {
			return Err(ZeroParasAvailabilityPeriod)
		}

		if self.no_show_slots.is_zero() {
			return Err(ZeroNoShowSlots)
		}

		if self.max_code_size > MAX_CODE_SIZE {
			return Err(MaxCodeSizeExceedHardLimit { max_code_size: self.max_code_size })
		}

		if self.max_head_data_size > MAX_HEAD_DATA_SIZE {
			return Err(MaxHeadDataSizeExceedHardLimit {
				max_head_data_size: self.max_head_data_size,
			})
		}

		if self.max_pov_size > MAX_POV_SIZE {
			return Err(MaxPovSizeExceedHardLimit { max_pov_size: self.max_pov_size })
		}

		if self.minimum_validation_upgrade_delay <= self.paras_availability_period {
			return Err(MinimumValidationUpgradeDelayLessThanChainAvailabilityPeriod {
				minimum_validation_upgrade_delay: self.minimum_validation_upgrade_delay.clone(),
				paras_availability_period: self.paras_availability_period.clone(),
			})
		}

		if self.validation_upgrade_delay <= 1.into() {
			return Err(ValidationUpgradeDelayIsTooLow {
				validation_upgrade_delay: self.validation_upgrade_delay.clone(),
			})
		}

		if self.max_upward_message_size > crate::inclusion::MAX_UPWARD_MESSAGE_SIZE_BOUND {
			return Err(MaxUpwardMessageSizeExceeded {
				max_message_size: self.max_upward_message_size,
			})
		}

		if self.hrmp_max_message_num_per_candidate > MAX_HORIZONTAL_MESSAGE_NUM {
			return Err(MaxHorizontalMessageNumExceeded {
				max_message_num: self.hrmp_max_message_num_per_candidate,
			})
		}

		if self.max_upward_message_num_per_candidate > MAX_UPWARD_MESSAGE_NUM {
			return Err(MaxUpwardMessageNumExceeded {
				max_message_num: self.max_upward_message_num_per_candidate,
			})
		}

		if self.hrmp_max_parachain_outbound_channels > crate::hrmp::HRMP_MAX_OUTBOUND_CHANNELS_BOUND
		{
			return Err(MaxHrmpOutboundChannelsExceeded)
		}

		if self.hrmp_max_parachain_inbound_channels > crate::hrmp::HRMP_MAX_INBOUND_CHANNELS_BOUND {
			return Err(MaxHrmpInboundChannelsExceeded)
		}

		if self.minimum_backing_votes.is_zero() {
			return Err(ZeroMinimumBackingVotes)
		}

		Ok(())
	}

	/// Checks that this instance is consistent with the requirements on each individual member.
	///
	/// # Panics
	///
	/// This function panics if the configuration is inconsistent.
	pub fn panic_if_not_consistent(&self) {
		if let Err(err) = self.check_consistency() {
			panic!("Host configuration is inconsistent: {:?}\nCfg:\n{:#?}", err, self);
		}
	}
}

pub trait WeightInfo {
	fn set_config_with_block_number() -> Weight;
	fn set_config_with_u32() -> Weight;
	fn set_config_with_option_u32() -> Weight;
	fn set_config_with_balance() -> Weight;
	fn set_hrmp_open_request_ttl() -> Weight;
	fn set_config_with_executor_params() -> Weight;
	fn set_config_with_perbill() -> Weight;
}

pub struct TestWeightInfo;
impl WeightInfo for TestWeightInfo {
	fn set_config_with_block_number() -> Weight {
		Weight::MAX
	}
	fn set_config_with_u32() -> Weight {
		Weight::MAX
	}
	fn set_config_with_option_u32() -> Weight {
		Weight::MAX
	}
	fn set_config_with_balance() -> Weight {
		Weight::MAX
	}
	fn set_hrmp_open_request_ttl() -> Weight {
		Weight::MAX
	}
	fn set_config_with_executor_params() -> Weight {
		Weight::MAX
	}
	fn set_config_with_perbill() -> Weight {
		Weight::MAX
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	/// The current storage version.
	///
	/// v0-v1: <https://github.com/paritytech/polkadot/pull/3575>
	/// v1-v2: <https://github.com/paritytech/polkadot/pull/4420>
	/// v2-v3: <https://github.com/paritytech/polkadot/pull/6091>
	/// v3-v4: <https://github.com/paritytech/polkadot/pull/6345>
	/// v4-v5: <https://github.com/paritytech/polkadot/pull/6937>
	///      + <https://github.com/paritytech/polkadot/pull/6961>
	///      + <https://github.com/paritytech/polkadot/pull/6934>
	/// v5-v6: <https://github.com/paritytech/polkadot/pull/6271> (remove UMP dispatch queue)
	/// v6-v7: <https://github.com/paritytech/polkadot/pull/7396>
	/// v7-v8: <https://github.com/paritytech/polkadot/pull/6969>
	/// v8-v9: <https://github.com/paritytech/polkadot/pull/7577>
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(9);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + shared::Config {
		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// The configuration interface for parachain
		type ParaConfigHandler: ParaConfigInterface<AccountId = Self::AccountId>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The new value for a configuration parameter is invalid.
		InvalidNewValue,
		/// The bootstrap has already ended.
		AlreadyEndedBootstrap,
	}

	/// The active configuration for the current session.
	#[pallet::storage]
	#[pallet::whitelist_storage]
	#[pallet::getter(fn config)]
	pub(crate) type ActiveConfig<T: Config> =
		StorageValue<_, HostConfiguration<BlockNumberFor<T>>, ValueQuery>;

	/// System Token configuration for `InfraRelay` Runtime
	#[pallet::storage]
	#[pallet::getter(fn active_system_config)]
	pub type ActiveSystemConfig<T: Config> = StorageValue<_, SystemConfig, ValueQuery>;

	/// Pending configuration changes.
	///
	/// This is a list of configuration changes, each with a session index at which it should
	/// be applied.
	///
	/// The list is sorted ascending by session index. Also, this list can only contain at most
	/// 2 items: for the next session and for the `scheduled_session`.
	#[pallet::storage]
	pub(crate) type PendingConfigs<T: Config> =
		StorageValue<_, Vec<(SessionIndex, HostConfiguration<BlockNumberFor<T>>)>, ValueQuery>;

	/// If this is set, then the configuration setters will bypass the consistency checks. This
	/// is meant to be used only as the last resort.
	#[pallet::storage]
	pub(crate) type BypassConsistencyCheck<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::storage]
	pub(crate) type RuntimeState<T: Config> = StorageValue<_, Mode, ValueQuery>;

	#[pallet::genesis_config]
	#[derive(DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		pub config: HostConfiguration<BlockNumberFor<T>>,
		pub system_config: SystemConfig,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			self.config.panic_if_not_consistent();
			self.system_config.panic_if_not_validated();
			ActiveConfig::<T>::put(&self.config);
			ActiveSystemConfig::<T>::put(self.system_config.clone());
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set the validation upgrade cooldown.
		#[pallet::call_index(0)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_block_number(),
			DispatchClass::Operational,
		))]
		pub fn set_validation_upgrade_cooldown(
			origin: OriginFor<T>,
			new: BlockNumberFor<T>,
		) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.validation_upgrade_cooldown = new;
			})
		}

		/// Set the validation upgrade delay.
		#[pallet::call_index(1)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_block_number(),
			DispatchClass::Operational,
		))]
		pub fn set_validation_upgrade_delay(
			origin: OriginFor<T>,
			new: BlockNumberFor<T>,
		) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.validation_upgrade_delay = new;
			})
		}

		/// Set the acceptance period for an included candidate.
		#[pallet::call_index(2)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_block_number(),
			DispatchClass::Operational,
		))]
		pub fn set_code_retention_period(
			origin: OriginFor<T>,
			new: BlockNumberFor<T>,
		) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.code_retention_period = new;
			})
		}

		/// Set the max validation code size for incoming upgrades.
		#[pallet::call_index(3)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_u32(),
			DispatchClass::Operational,
		))]
		pub fn set_max_code_size(origin: OriginFor<T>, new: u32) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.max_code_size = new;
			})
		}

		/// Set the max POV block size for incoming upgrades.
		#[pallet::call_index(4)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_u32(),
			DispatchClass::Operational,
		))]
		pub fn set_max_pov_size(origin: OriginFor<T>, new: u32) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.max_pov_size = new;
			})
		}

		/// Set the max head data size for paras.
		#[pallet::call_index(5)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_u32(),
			DispatchClass::Operational,
		))]
		pub fn set_max_head_data_size(origin: OriginFor<T>, new: u32) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.max_head_data_size = new;
			})
		}

		/// Set the number of on demand execution cores.
		#[pallet::call_index(6)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_u32(),
			DispatchClass::Operational,
		))]
		pub fn set_on_demand_cores(origin: OriginFor<T>, new: u32) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.on_demand_cores = new;
			})
		}

		/// Set the number of retries for a particular on demand.
		#[pallet::call_index(7)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_u32(),
			DispatchClass::Operational,
		))]
		pub fn set_on_demand_retries(origin: OriginFor<T>, new: u32) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.on_demand_retries = new;
			})
		}

		/// Set the parachain validator-group rotation frequency
		#[pallet::call_index(8)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_block_number(),
			DispatchClass::Operational,
		))]
		pub fn set_group_rotation_frequency(
			origin: OriginFor<T>,
			new: BlockNumberFor<T>,
		) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.group_rotation_frequency = new;
			})
		}

		/// Set the availability period for paras.
		#[pallet::call_index(9)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_block_number(),
			DispatchClass::Operational,
		))]
		pub fn set_paras_availability_period(
			origin: OriginFor<T>,
			new: BlockNumberFor<T>,
		) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.paras_availability_period = new;
			})
		}

		/// Set the scheduling lookahead, in expected number of blocks at peak throughput.
		#[pallet::call_index(11)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_u32(),
			DispatchClass::Operational,
		))]
		pub fn set_scheduling_lookahead(origin: OriginFor<T>, new: u32) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.scheduling_lookahead = new;
			})
		}

		/// Set the maximum number of validators to assign to any core.
		#[pallet::call_index(12)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_option_u32(),
			DispatchClass::Operational,
		))]
		pub fn set_max_validators_per_core(
			origin: OriginFor<T>,
			new: Option<u32>,
		) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.max_validators_per_core = new;
			})
		}

		/// Set the maximum number of validators to use in parachain consensus.
		#[pallet::call_index(13)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_option_u32(),
			DispatchClass::Operational,
		))]
		pub fn set_max_validators(origin: OriginFor<T>, new: Option<u32>) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.max_validators = new;
			})
		}

		/// Set the dispute period, in number of sessions to keep for disputes.
		#[pallet::call_index(14)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_u32(),
			DispatchClass::Operational,
		))]
		pub fn set_dispute_period(origin: OriginFor<T>, new: SessionIndex) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.dispute_period = new;
			})
		}

		/// Set the dispute post conclusion acceptance period.
		#[pallet::call_index(15)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_block_number(),
			DispatchClass::Operational,
		))]
		pub fn set_dispute_post_conclusion_acceptance_period(
			origin: OriginFor<T>,
			new: BlockNumberFor<T>,
		) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.dispute_post_conclusion_acceptance_period = new;
			})
		}

		/// Set the no show slots, in number of number of consensus slots.
		/// Must be at least 1.
		#[pallet::call_index(18)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_u32(),
			DispatchClass::Operational,
		))]
		pub fn set_no_show_slots(origin: OriginFor<T>, new: u32) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.no_show_slots = new;
			})
		}

		/// Set the total number of delay tranches.
		#[pallet::call_index(19)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_u32(),
			DispatchClass::Operational,
		))]
		pub fn set_n_delay_tranches(origin: OriginFor<T>, new: u32) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.n_delay_tranches = new;
			})
		}

		/// Set the zeroth delay tranche width.
		#[pallet::call_index(20)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_u32(),
			DispatchClass::Operational,
		))]
		pub fn set_zeroth_delay_tranche_width(origin: OriginFor<T>, new: u32) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.zeroth_delay_tranche_width = new;
			})
		}

		/// Set the number of validators needed to approve a block.
		#[pallet::call_index(21)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_u32(),
			DispatchClass::Operational,
		))]
		pub fn set_needed_approvals(origin: OriginFor<T>, new: u32) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.needed_approvals = new;
			})
		}

		/// Set the number of samples to do of the `RelayVRFModulo` approval assignment criterion.
		#[pallet::call_index(22)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_u32(),
			DispatchClass::Operational,
		))]
		pub fn set_relay_vrf_modulo_samples(origin: OriginFor<T>, new: u32) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.relay_vrf_modulo_samples = new;
			})
		}

		/// Sets the maximum items that can present in a upward dispatch queue at once.
		#[pallet::call_index(23)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_u32(),
			DispatchClass::Operational,
		))]
		pub fn set_max_upward_queue_count(origin: OriginFor<T>, new: u32) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.max_upward_queue_count = new;
			})
		}

		/// Sets the maximum total size of items that can present in a upward dispatch queue at
		/// once.
		#[pallet::call_index(24)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_u32(),
			DispatchClass::Operational,
		))]
		pub fn set_max_upward_queue_size(origin: OriginFor<T>, new: u32) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(new <= MAX_UPWARD_MESSAGE_SIZE_BOUND, Error::<T>::InvalidNewValue);

			Self::schedule_config_update(|config| {
				config.max_upward_queue_size = new;
			})
		}

		/// Set the critical downward message size.
		#[pallet::call_index(25)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_u32(),
			DispatchClass::Operational,
		))]
		pub fn set_max_downward_message_size(origin: OriginFor<T>, new: u32) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.max_downward_message_size = new;
			})
		}

		/// Sets the maximum size of an upward message that can be sent by a candidate.
		#[pallet::call_index(27)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_u32(),
			DispatchClass::Operational,
		))]
		pub fn set_max_upward_message_size(origin: OriginFor<T>, new: u32) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.max_upward_message_size = new;
			})
		}

		/// Sets the maximum number of messages that a candidate can contain.
		#[pallet::call_index(28)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_u32(),
			DispatchClass::Operational,
		))]
		pub fn set_max_upward_message_num_per_candidate(
			origin: OriginFor<T>,
			new: u32,
		) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.max_upward_message_num_per_candidate = new;
			})
		}

		/// Sets the number of sessions after which an HRMP open channel request expires.
		#[pallet::call_index(29)]
		#[pallet::weight((
			T::WeightInfo::set_hrmp_open_request_ttl(),
			DispatchClass::Operational,
		))]
		// Deprecated, but is not marked as such, because that would trigger warnings coming from
		// the macro.
		pub fn set_hrmp_open_request_ttl(_origin: OriginFor<T>, _new: u32) -> DispatchResult {
			Err("this doesn't have any effect".into())
		}

		/// Sets the amount of funds that the sender should provide for opening an HRMP channel.
		#[pallet::call_index(30)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_balance(),
			DispatchClass::Operational,
		))]
		pub fn set_hrmp_sender_deposit(origin: OriginFor<T>, new: Balance) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.hrmp_sender_deposit = new;
			})
		}

		/// Sets the amount of funds that the recipient should provide for accepting opening an HRMP
		/// channel.
		#[pallet::call_index(31)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_balance(),
			DispatchClass::Operational,
		))]
		pub fn set_hrmp_recipient_deposit(origin: OriginFor<T>, new: Balance) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.hrmp_recipient_deposit = new;
			})
		}

		/// Sets the maximum number of messages allowed in an HRMP channel at once.
		#[pallet::call_index(32)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_u32(),
			DispatchClass::Operational,
		))]
		pub fn set_hrmp_channel_max_capacity(origin: OriginFor<T>, new: u32) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.hrmp_channel_max_capacity = new;
			})
		}

		/// Sets the maximum total size of messages in bytes allowed in an HRMP channel at once.
		#[pallet::call_index(33)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_u32(),
			DispatchClass::Operational,
		))]
		pub fn set_hrmp_channel_max_total_size(origin: OriginFor<T>, new: u32) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.hrmp_channel_max_total_size = new;
			})
		}

		/// Sets the maximum number of inbound HRMP channels a parachain is allowed to accept.
		#[pallet::call_index(34)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_u32(),
			DispatchClass::Operational,
		))]
		pub fn set_hrmp_max_parachain_inbound_channels(
			origin: OriginFor<T>,
			new: u32,
		) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.hrmp_max_parachain_inbound_channels = new;
			})
		}

		/// Sets the maximum size of a message that could ever be put into an HRMP channel.
		#[pallet::call_index(36)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_u32(),
			DispatchClass::Operational,
		))]
		pub fn set_hrmp_channel_max_message_size(origin: OriginFor<T>, new: u32) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.hrmp_channel_max_message_size = new;
			})
		}

		/// Sets the maximum number of outbound HRMP channels a parachain is allowed to open.
		#[pallet::call_index(37)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_u32(),
			DispatchClass::Operational,
		))]
		pub fn set_hrmp_max_parachain_outbound_channels(
			origin: OriginFor<T>,
			new: u32,
		) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.hrmp_max_parachain_outbound_channels = new;
			})
		}

		/// Sets the maximum number of outbound HRMP messages can be sent by a candidate.
		#[pallet::call_index(39)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_u32(),
			DispatchClass::Operational,
		))]
		pub fn set_hrmp_max_message_num_per_candidate(
			origin: OriginFor<T>,
			new: u32,
		) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.hrmp_max_message_num_per_candidate = new;
			})
		}

		/// Set the number of session changes after which a PVF pre-checking voting is rejected.
		#[pallet::call_index(42)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_u32(),
			DispatchClass::Operational,
		))]
		pub fn set_pvf_voting_ttl(origin: OriginFor<T>, new: SessionIndex) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.pvf_voting_ttl = new;
			})
		}

		/// Sets the minimum delay between announcing the upgrade block for a parachain until the
		/// upgrade taking place.
		///
		/// See the field documentation for information and constraints for the new value.
		#[pallet::call_index(43)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_block_number(),
			DispatchClass::Operational,
		))]
		pub fn set_minimum_validation_upgrade_delay(
			origin: OriginFor<T>,
			new: BlockNumberFor<T>,
		) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.minimum_validation_upgrade_delay = new;
			})
		}

		/// Setting this to true will disable consistency checks for the configuration setters.
		/// Use with caution.
		#[pallet::call_index(44)]
		#[pallet::weight((
			T::DbWeight::get().writes(1),
			DispatchClass::Operational,
		))]
		pub fn set_bypass_consistency_check(origin: OriginFor<T>, new: bool) -> DispatchResult {
			ensure_root(origin)?;
			BypassConsistencyCheck::<T>::put(new);
			Ok(())
		}

		/// Set the asynchronous backing parameters.
		#[pallet::call_index(45)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_option_u32(), // The same size in bytes.
			DispatchClass::Operational,
		))]
		pub fn set_async_backing_params(
			origin: OriginFor<T>,
			new: AsyncBackingParams,
		) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.async_backing_params = new;
			})
		}

		/// Set PVF executor parameters.
		#[pallet::call_index(46)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_executor_params(),
			DispatchClass::Operational,
		))]
		pub fn set_executor_params(origin: OriginFor<T>, new: ExecutorParams) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.executor_params = new;
			})
		}

		/// Set the on demand (parathreads) base fee.
		#[pallet::call_index(47)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_balance(),
			DispatchClass::Operational,
		))]
		pub fn set_on_demand_base_fee(origin: OriginFor<T>, new: Balance) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.on_demand_base_fee = new;
			})
		}

		/// Set the on demand (parathreads) fee variability.
		#[pallet::call_index(48)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_perbill(),
			DispatchClass::Operational,
		))]
		pub fn set_on_demand_fee_variability(origin: OriginFor<T>, new: Perbill) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.on_demand_fee_variability = new;
			})
		}

		/// Set the on demand (parathreads) queue max size.
		#[pallet::call_index(49)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_option_u32(),
			DispatchClass::Operational,
		))]
		pub fn set_on_demand_queue_max_size(origin: OriginFor<T>, new: u32) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.on_demand_queue_max_size = new;
			})
		}
		/// Set the on demand (parathreads) fee variability.
		#[pallet::call_index(50)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_perbill(),
			DispatchClass::Operational,
		))]
		pub fn set_on_demand_target_queue_utilization(
			origin: OriginFor<T>,
			new: Perbill,
		) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.on_demand_target_queue_utilization = new;
			})
		}
		/// Set the on demand (parathreads) ttl in the claimqueue.
		#[pallet::call_index(51)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_block_number(),
			DispatchClass::Operational
		))]
		pub fn set_on_demand_ttl(origin: OriginFor<T>, new: BlockNumberFor<T>) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.on_demand_ttl = new;
			})
		}
		/// Set the minimum backing votes threshold.
		#[pallet::call_index(52)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_u32(),
			DispatchClass::Operational
		))]
		pub fn set_minimum_backing_votes(origin: OriginFor<T>, new: u32) -> DispatchResult {
			ensure_root(origin)?;
			Self::schedule_config_update(|config| {
				config.minimum_backing_votes = new;
			})
		}

		// TODO: Benchmark weight!
		#[pallet::call_index(53)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_u32(),
			DispatchClass::Operational
		))]
		pub fn update_system_config(origin: OriginFor<T>, new: SystemConfig) -> DispatchResult {
			// TODO: Use `scheudle_config_update`
			ensure_root(origin)?;
			ActiveSystemConfig::<T>::put(new);
			Ok(())
		}

		// TODO: Benchmark weight!
		#[pallet::call_index(54)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_u32(),
			DispatchClass::Operational
		))]
		pub fn update_fee_table(
			origin: OriginFor<T>,
			dest: DestIdOf<T>,
			pallet_name: Vec<u8>,
			call_name: Vec<u8>,
			fee: SystemTokenBalanceOf<T>,
		) -> DispatchResult {
			// TODO: Use `scheudle_config_update`
			ensure_root(origin)?;
			T::ParaConfigHandler::update_fee_table(dest, pallet_name, call_name, fee);
			Ok(())
		}

		// TODO: Benchmark weight!
		#[pallet::call_index(55)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_u32(),
			DispatchClass::Operational
		))]
		pub fn update_para_fee_rate(
			origin: OriginFor<T>,
			dest: DestIdOf<T>,
			fee_rate: SystemTokenBalanceOf<T>,
		) -> DispatchResult {
			// TODO: Use `scheudle_config_update`
			ensure_root(origin)?;
			T::ParaConfigHandler::update_para_fee_rate(dest, fee_rate);
			Ok(())
		}

		// TODO: Benchmark weight!
		#[pallet::call_index(56)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_u32(),
			DispatchClass::Operational
		))]
		pub fn update_runtime_state(origin: OriginFor<T>, dest: DestIdOf<T>) -> DispatchResult {
			// TODO: Use `scheudle_config_update`
			ensure_root(origin)?;
			T::ParaConfigHandler::update_runtime_state(dest);
			Ok(())
		}

		// TODO: Benchmark weight!
		#[pallet::call_index(57)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_u32(),
			DispatchClass::Operational
		))]
		pub fn set_admin(
			origin: OriginFor<T>,
			dest: DestIdOf<T>,
			who: T::AccountId,
		) -> DispatchResult {
			ensure_root(origin)?;
			T::ParaConfigHandler::set_admin(dest, who);
			Ok(())
		}

		#[pallet::call_index(58)]
		#[pallet::weight((
			T::WeightInfo::set_config_with_u32(),
			DispatchClass::Operational
		))]
		pub fn end_bootstrap(origin: OriginFor<T>) -> DispatchResult {
			ensure_root(origin)?;
			RuntimeState::<T>::try_mutate(|s| -> DispatchResult {
				if *s == Mode::Normal {
					return Err(Error::<T>::AlreadyEndedBootstrap.into())
				}
				*s = Mode::Normal;
				Ok(())
			})?;
			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn integrity_test() {
			assert_eq!(
				&ActiveConfig::<T>::hashed_key(),
				primitives::well_known_keys::ACTIVE_CONFIG,
				"`well_known_keys::ACTIVE_CONFIG` doesn't match key of `ActiveConfig`! Make sure that the name of the\
				 configuration pallet is `Configuration` in the runtime!",
			);
		}
	}
}

/// A struct that holds the configuration that was active before the session change and optionally
/// a configuration that became active after the session change.
pub struct SessionChangeOutcome<BlockNumber> {
	/// Previously active configuration.
	pub prev_config: HostConfiguration<BlockNumber>,
	/// If new configuration was applied during the session change, this is the new configuration.
	pub new_config: Option<HostConfiguration<BlockNumber>>,
}

impl<T: Config> Pallet<T> {
	/// Called by the initializer to initialize the configuration pallet.
	pub(crate) fn initializer_initialize(_now: BlockNumberFor<T>) -> Weight {
		Weight::zero()
	}

	/// Called by the initializer to finalize the configuration pallet.
	pub(crate) fn initializer_finalize() {}

	/// Called by the initializer to note that a new session has started.
	///
	/// Returns the configuration that was actual before the session change and the configuration
	/// that became active after the session change. If there were no scheduled changes, both will
	/// be the same.
	pub(crate) fn initializer_on_new_session(
		session_index: &SessionIndex,
	) -> SessionChangeOutcome<BlockNumberFor<T>> {
		let pending_configs = <PendingConfigs<T>>::get();
		let prev_config = ActiveConfig::<T>::get();

		// No pending configuration changes, so we're done.
		if pending_configs.is_empty() {
			return SessionChangeOutcome { prev_config, new_config: None }
		}

		let (mut past_and_present, future) = pending_configs
			.into_iter()
			.partition::<Vec<_>, _>(|&(apply_at_session, _)| apply_at_session <= *session_index);

		if past_and_present.len() > 1 {
			// This should never happen since we schedule configuration changes only into the future
			// sessions and this handler called for each session change.
			log::error!(
				target: LOG_TARGET,
				"Skipping applying configuration changes scheduled sessions in the past",
			);
		}

		let new_config = past_and_present.pop().map(|(_, config)| config);
		if let Some(ref new_config) = new_config {
			// Apply the new configuration.
			ActiveConfig::<T>::put(new_config);
		}

		<PendingConfigs<T>>::put(future);

		SessionChangeOutcome { prev_config, new_config }
	}

	/// Return the session index that should be used for any future scheduled changes.
	fn scheduled_session() -> SessionIndex {
		shared::Pallet::<T>::scheduled_session()
	}

	/// Forcibly set the active config. This should be used with extreme care, and typically
	/// only when enabling parachains runtime pallets for the first time on a chain which has
	/// been running without them.
	pub fn force_set_active_config(config: HostConfiguration<BlockNumberFor<T>>) {
		ActiveConfig::<T>::set(config);
	}

	/// This function should be used to update members of the configuration.
	///
	/// This function is used to update the configuration in a way that is safe. It will check the
	/// resulting configuration and ensure that the update is valid. If the update is invalid, it
	/// will check if the previous configuration was valid. If it was invalid, we proceed with
	/// updating the configuration, giving a chance to recover from such a condition.
	///
	/// The actual configuration change take place after a couple of sessions have passed. In case
	/// this function is called more than once in a session, then the pending configuration change
	/// will be updated and the changes will be applied at once.
	// NOTE: Explicitly tell rustc not to inline this because otherwise heuristics note the incoming
	// closure making it's attractive to inline. However, in this case, we will end up with lots of
	// duplicated code (making this function to show up in the top of heaviest functions) only for
	// the sake of essentially avoiding an indirect call. Doesn't worth it.
	#[inline(never)]
	pub(crate) fn schedule_config_update(
		updater: impl FnOnce(&mut HostConfiguration<BlockNumberFor<T>>),
	) -> DispatchResult {
		let mut pending_configs = <PendingConfigs<T>>::get();

		// 1. pending_configs = [] No pending configuration changes.
		//
		//    That means we should use the active config as the base configuration. We will insert
		//    the new pending configuration as (cur+2, new_config) into the list.
		//
		// 2. pending_configs = [(cur+2, X)] There is a configuration that is pending for the
		//    scheduled session.
		//
		//    We will use X as the base configuration. We can update the pending configuration X
		//    directly.
		//
		// 3. pending_configs = [(cur+1, X)] There is a pending configuration scheduled and it will
		//    be applied in the next session.
		//
		//    We will use X as the base configuration. We need to schedule a new configuration
		// change    for the `scheduled_session` and use X as the base for the new configuration.
		//
		// 4. pending_configs = [(cur+1, X), (cur+2, Y)] There is a pending configuration change in
		//    the next session and for the scheduled session. Due to case №3, we can be sure that Y
		//    is based on top of X. This means we can use Y as the base configuration and update Y
		//    directly.
		//
		// There cannot be (cur, X) because those are applied in the session change handler for the
		// current session.

		// First, we need to decide what we should use as the base configuration.
		let mut base_config = pending_configs
			.last()
			.map(|(_, config)| config.clone())
			.unwrap_or_else(Self::config);
		let base_config_consistent = base_config.check_consistency().is_ok();

		// Now, we need to decide what the new configuration should be.
		// We also move the `base_config` to `new_config` to empahsize that the base config was
		// destroyed by the `updater`.
		updater(&mut base_config);
		let new_config = base_config;

		if BypassConsistencyCheck::<T>::get() {
			// This will emit a warning each configuration update if the consistency check is
			// bypassed. This is an attempt to make sure the bypass is not accidentally left on.
			log::warn!(
				target: LOG_TARGET,
				"Bypassing the consistency check for the configuration change!",
			);
		} else if let Err(e) = new_config.check_consistency() {
			if base_config_consistent {
				// Base configuration is consistent and the new configuration is inconsistent.
				// This means that the value set by the `updater` is invalid and we can return
				// it as an error.
				log::warn!(
					target: LOG_TARGET,
					"Configuration change rejected due to invalid configuration: {:?}",
					e,
				);
				return Err(Error::<T>::InvalidNewValue.into())
			} else {
				// The configuration was already broken, so we can as well proceed with the update.
				// You cannot break something that is already broken.
				//
				// That will allow to call several functions and ultimately return the configuration
				// into consistent state.
				log::warn!(
					target: LOG_TARGET,
					"The new configuration is broken but the old is broken as well. Proceeding",
				);
			}
		}

		let scheduled_session = Self::scheduled_session();

		if let Some(&mut (_, ref mut config)) = pending_configs
			.iter_mut()
			.find(|&&mut (apply_at_session, _)| apply_at_session >= scheduled_session)
		{
			*config = new_config;
		} else {
			// We are scheduling a new configuration change for the scheduled session.
			pending_configs.push((scheduled_session, new_config));
		}

		<PendingConfigs<T>>::put(pending_configs);

		Ok(())
	}
}

/// Runtime configuration set by parent chain which is mostly Relay Chain
pub trait ParaConfigInterface {
	/// InfraBlockchain AccountId type
	type AccountId: Parameter;
	/// Destination ID type
	type DestId: Parameter;
	/// Balance type of System Token
	type Balance: Parameter + AtLeast32BitUnsigned;

	/// Set admin for InfraParaCore of `dest_id` Runtime
	fn set_admin(dest_id: Self::DestId, who: Self::AccountId);
	/// Update fee table for `dest_id` Runtime
	fn update_fee_table(
		dest_id: Self::DestId,
		pallet_name: Vec<u8>,
		call_name: Vec<u8>,
		fee: Self::Balance,
	);
	/// Update fee rate for `dest_id` Runtime
	fn update_para_fee_rate(dest_id: Self::DestId, fee_rate: Self::Balance);
	/// Set runtime state for `dest_id` Runtime
	fn update_runtime_state(dest_id: Self::DestId);
}

impl<T: Config> RuntimeConfigProvider<SystemTokenBalanceOf<T>> for Pallet<T>
where
	SystemTokenBalanceOf<T>: From<u128>,
{
	type Error = ();

	fn system_config() -> Result<SystemConfig, Self::Error> {
		Ok(ActiveSystemConfig::<T>::get())
	}

	fn para_fee_rate() -> Result<SystemTokenBalanceOf<T>, Self::Error> {
		Ok(ActiveSystemConfig::<T>::get().base_system_token_detail.base_weight.into())
	}

	fn fee_for(_ext: ExtrinsicMetadata) -> Option<SystemTokenBalanceOf<T>> {
		None
	}

	fn runtime_state() -> Mode {
		RuntimeState::<T>::get()
	}
}
