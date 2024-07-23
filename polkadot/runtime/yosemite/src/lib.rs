// Copyright 2017-2020 Parity Technologies (UK) Ltd.
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

//! `infra-relay` runtime. This can be compiled with `#[no_std]`, ready for Wasm.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "512"]

use polkadot_runtime_common::{
	auctions, crowdloan, impl_runtime_weights, paras_registrar, ToAuthor,
	paras_sudo_wrapper, prod_or_fast, slots, BlockHashCount, BlockLength, SlowAdjustingFeeUpdate,
};

use polkadot_runtime_parachains::{
	assigner_coretime as parachains_assigner_coretime,
	assigner_on_demand as parachains_assigner_on_demand, configuration as parachains_configuration,
	configuration::{ParaConfigInterface, ActiveConfigHrmpChannelSizeAndCapacityRatio},
	coretime, disputes as parachains_disputes,
	disputes::slashing as parachains_slashing,
	dmp as parachains_dmp, hrmp as parachains_hrmp, inclusion as parachains_inclusion,
	inclusion::{AggregateMessageOrigin, UmpQueueId},
	initializer as parachains_initializer, origin as parachains_origin, paras as parachains_paras,
	paras_inherent as parachains_paras_inherent,
	runtime_api_impl::{
		v10 as parachains_runtime_api_impl, vstaging as vstaging_parachains_runtime_api_impl,
	},
	scheduler as parachains_scheduler, session_info as parachains_session_info,
	shared as parachains_shared, system_token_manager,
	system_token_manager::{OracleInterface, SystemTokenInterface},
};

use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use frame_support::{
	construct_runtime, derive_impl,
	genesis_builder_helper::{build_state, get_preset},
	parameter_types,
	traits::{
		tokens::{
			fungibles::{self, Balanced, Credit},
			pay::PayAssetFromAccount,
			Preservation::Preserve,
		},
		AsEnsureOriginWithArg, ConstU128, ConstU32, Contains, EitherOf, EitherOfDiverse,
		EverythingBut, InstanceFilter, KeyOwnerProofSystem, LockIdentifier, PrivilegeCmp,
		ProcessMessage, ProcessMessageError, Get,
	},
	weights::{ConstantMultiplier, WeightMeter},
	PalletId,
};
use frame_system::{EnsureRoot, EnsureWithSuccess};
use pallet_grandpa::{fg_primitives, AuthorityId as GrandpaId};
use pallet_identity::legacy::IdentityInfo;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_session::historical as session_historical;
use pallet_system_token_tx_payment::{HandleCredit, RewardOriginInfo, TransactionFeeCharger};
use pallet_transaction_payment::{FeeDetails, RuntimeDispatchInfo, FungibleAdapter};
use pallet_validator_management::{RewardInterface, SessionIndex};
use codec::{Decode, Encode, MaxEncodedLen};
use polkadot_primitives::{
	slashing, CoreIndex, AccountId, AccountIndex, ApprovalVotingParams, Balance, BlockNumber, CandidateEvent,
	CandidateHash, CommittedCandidateReceipt, CoreState, DisputeState, ExecutorParams,
	GroupRotationInfo, Hash, Id as ParaId, InboundDownwardMessage, InboundHrmpMessage, Moment,
	NodeFeatures, Nonce, OccupiedCoreAssumption, PersistedValidationData, ScrapedOnChainVotes,
	SessionInfo, Signature, ValidationCode, ValidationCodeHash, ValidatorId, ValidatorIndex,
	PARACHAIN_KEY_TYPE_ID,
};
use sp_core::OpaqueMetadata;
use sp_mmr_primitives as mmr;
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	infra::*,
	traits::{
		AccountIdConversion, AccountIdLookup, BlakeTwo256, Block as BlockT,
		Extrinsic as ExtrinsicT, OpaqueKeys, SaturatedConversion, Verify,
	},
	transaction_validity::{TransactionPriority, TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, FixedU128, KeyTypeId, Perbill, Percent, Permill,
};

use sp_std::{cmp::Ordering, collections::{btree_map::BTreeMap, vec_deque::VecDeque}, prelude::*};
#[cfg(any(feature = "std", test))]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use sp_genesis_builder::PresetId;
use static_assertions::const_assert;
use xcm::latest::{Junction, Location, Reanchorable, InteriorLocation};

pub use frame_system::Call as SystemCall;
pub use pallet_balances::Call as BalancesCall;

pub use pallet_timestamp::Call as TimestampCall;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

/// Constant values used within the runtime.
use yosemite_runtime_constants::{
	currency::*,
	fee::*,
	system_parachain::{ASSET_HUB_ID, BROKER_ID, coretime::TIMESLICE_PERIOD},
	time::*,
};

// Weights used in the runtime.
mod weights;

mod infra;
use infra::{OracleManager, ParaConfigHandler, RewardHandler, SystemTokenHandler};

mod genesis_config_presets;

// XCM
pub mod xcm_config;
use infra_asset_common::{
	local_and_foreign_assets::LocalFromLeft, AssetIdForNativeAssets, AssetIdForNativeAssetsConvert,
};
use xcm_config::{NativeAssetsPalletLocation, UniversalLocation};

impl_runtime_weights!(yosemite_runtime_constants);

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

/// Provides the `WASM_BINARY` build with `fast-runtime` feature enabled.
///
/// This is for example useful for local test chains.
#[cfg(feature = "std")]
pub mod fast_runtime_binary {
	include!(concat!(env!("OUT_DIR"), "/fast_runtime_binary.rs"));
}

/// Runtime version (yosemite).
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("yosemite"),
	impl_name: create_runtime_str!("bcl-yosemite"),
	authoring_version: 0,
	spec_version: 1_014_000,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};

/// The BABE epoch configuration at genesis.
pub const BABE_GENESIS_EPOCH_CONFIG: sp_consensus_babe::BabeEpochConfiguration =
	sp_consensus_babe::BabeEpochConfiguration {
		c: PRIMARY_PROBABILITY,
		allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryVRFSlots,
	};

/// Native version.
#[cfg(any(feature = "std", test))]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

type MoreThanHalfCouncil = EitherOfDiverse<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<AccountId, ValidatorCollective, 1, 2>,
>;

/// A type to identify calls to the Identity pallet. These will be filtered to prevent invocation,
/// locking the state of the pallet and preventing further updates to identities and sub-identities.
/// The locked state will be the genesis state of a new system chain and then removed from the Relay
/// Chain.
pub struct IsIdentityCall;
impl Contains<RuntimeCall> for IsIdentityCall {
	fn contains(c: &RuntimeCall) -> bool {
		matches!(c, RuntimeCall::Identity(_))
	}
}

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;
	pub const SS58Prefix: u8 = 0;
}

#[derive_impl(frame_system::config_preludes::RelayChainDefaultConfig)]
impl frame_system::Config for Runtime {
	type BaseCallFilter = EverythingBut<IsIdentityCall>;
	type BlockWeights = BlockWeights;
	type BlockLength = BlockLength;
	type DbWeight = RocksDbWeight;
	type Nonce = Nonce;
	type Hash = Hash;
	type AccountId = AccountId;
	type Block = Block;
	type BlockHashCount = BlockHashCount;
	type Version = Version;
	type AccountData = pallet_balances::AccountData<Balance>;
	type SystemWeightInfo = weights::frame_system::WeightInfo<Runtime>;
	type SS58Prefix = SS58Prefix;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_sudo::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type WeightInfo = weights::pallet_sudo::WeightInfo<Runtime>;
}

impl paras_sudo_wrapper::Config for Runtime {}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
		BlockWeights::get().max_block;
	pub const MaxScheduledPerBlock: u32 = 50;
	pub const NoPreimagePostponement: Option<u32> = Some(10);
}

type ScheduleOrigin = EitherOfDiverse<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<AccountId, ValidatorCollective, 1, 2>,
>;

/// Used the compare the privilege of an origin inside the scheduler.
pub struct OriginPrivilegeCmp;

impl PrivilegeCmp<OriginCaller> for OriginPrivilegeCmp {
	fn cmp_privilege(left: &OriginCaller, right: &OriginCaller) -> Option<Ordering> {
		if left == right {
			return Some(Ordering::Equal);
		}

		match (left, right) {
			// Root is greater than anything.
			(OriginCaller::system(frame_system::RawOrigin::Root), _) => Some(Ordering::Greater),
			// Check which one has more yes votes.
			(
				OriginCaller::Council(pallet_collective::RawOrigin::Members(l_yes_votes, l_count)),
				OriginCaller::Council(pallet_collective::RawOrigin::Members(r_yes_votes, r_count)),
			) => Some((l_yes_votes * r_count).cmp(&(r_yes_votes * l_count))),
			// For every other origin we don't care, as they are not used for `ScheduleOrigin`.
			_ => None,
		}
	}
}

impl pallet_scheduler::Config for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeEvent = RuntimeEvent;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = ScheduleOrigin;
	type MaxScheduledPerBlock = MaxScheduledPerBlock;
	type WeightInfo = weights::pallet_scheduler::WeightInfo<Runtime>;
	type OriginPrivilegeCmp = OriginPrivilegeCmp;
	type Preimages = Preimage;
}

parameter_types! {
	pub const PreimageBaseDeposit: Balance = deposit(2, 64);
	pub const PreimageByteDeposit: Balance = deposit(0, 1);
	pub const PreimageHoldReason: RuntimeHoldReason = RuntimeHoldReason::Preimage(pallet_preimage::HoldReason::Preimage);
}

impl pallet_preimage::Config for Runtime {
	type WeightInfo = weights::pallet_preimage::WeightInfo<Runtime>;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type ManagerOrigin = AuthorityOrigin;
	type Consideration = frame_support::traits::fungible::HoldConsideration<
		AccountId,
		Balances,
		PreimageHoldReason,
		frame_support::traits::LinearStoragePrice<
			PreimageBaseDeposit,
			PreimageByteDeposit,
			Balance,
		>,
	>;
}

parameter_types! {
	pub EpochDuration: u64 = EpochDurationInBlocks::get() as u64;
	pub const BondingDuration: u32 = 28;
	pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
	pub ReportLongevity: u64 =
		BondingDuration::get() as u64 * SessionsPerEra::get() as u64 * EpochDuration::get();
	pub const MaxNominatorRewardedPerValidator: u32 = 512;
}

impl pallet_babe::Config for Runtime {
	type EpochDuration = EpochDuration;
	type ExpectedBlockTime = ExpectedBlockTime;
	// session module is the trigger
	type EpochChangeTrigger = pallet_babe::ExternalTrigger;
	type DisabledValidators = Session;
	type WeightInfo = ();
	type MaxAuthorities = MaxAuthorities;
	type MaxNominators = MaxNominatorRewardedPerValidator;
	type KeyOwnerProof =
		<Historical as KeyOwnerProofSystem<(KeyTypeId, pallet_babe::AuthorityId)>>::Proof;
	type EquivocationReportSystem =
		pallet_babe::EquivocationReportSystem<Self, Offences, Historical, ReportLongevity>;
}

parameter_types! {
	pub const IndexDeposit: Balance = 10 * DOLLARS;
}

impl pallet_indices::Config for Runtime {
	type AccountIndex = AccountIndex;
	type Currency = Balances;
	type Deposit = IndexDeposit;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_indices::WeightInfo<Runtime>;
}

parameter_types! {
	pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = weights::pallet_balances_balances::WeightInfo<Runtime>;
	type FreezeIdentifier = ();
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type MaxFreezes = ConstU32<10>;
}

parameter_types! {
	pub const TransactionByteFee: Balance = 10 * MILLICENTS;
	/// This value increases the priority of `Operational` transactions by adding
	/// a "virtual tip" that's equal to the `OperationalFeeMultiplier * final_fee`.
	pub const OperationalFeeMultiplier: u8 = 5;
}

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = FungibleAdapter<Balances, ToAuthor<Runtime>>;
	type OperationalFeeMultiplier = OperationalFeeMultiplier;
	type WeightToFee = WeightToFee;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
}

pub struct BootstrapCallFilter;
impl frame_support::traits::Contains<RuntimeCall> for BootstrapCallFilter {
	#[cfg(not(feature = "fast-runtime"))]
	fn contains(call: &RuntimeCall) -> bool {
		match call {
			RuntimeCall::Council(
				pallet_collective::Call::propose { .. }
				| pallet_collective::Call::vote { .. }
				| pallet_collective::Call::close { .. },
			)
			| RuntimeCall::Democracy(pallet_democracy::Call::external_propose_majority {
				..
			})
			| RuntimeCall::Preimage(pallet_preimage::Call::note_preimage { .. })
			| RuntimeCall::Assets(
				NativeAssetsCall::create { .. } | NativeAssetsCall::mint { .. },
			)
			| RuntimeCall::Configuration(
				parachains_configuration::Call::end_bootstrap { .. }
				| parachains_configuration::Call::set_admin { .. }
				| parachains_configuration::Call::update_para_fee_rate { .. }
				| parachains_configuration::Call::update_fee_table { .. }
				| parachains_configuration::Call::update_runtime_state { .. },
			)
			| RuntimeCall::Sudo(..) => true,
			_ => false,
		}
	}
	#[cfg(feature = "fast-runtime")]
	fn contains(call: &RuntimeCall) -> bool {
		match call {
			_ => true,
		}
	}
}

parameter_types! {
	pub const RewardFraction: Perbill = Perbill::from_percent(80);
	pub const FeeTreasuryId: PalletId = PalletId(*b"infratrs");
}

impl pallet_system_token_tx_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SystemConfig = Configuration;
	type PoTHandler = ValidatorManagement;
	type Fungibles = NativeAndForeignAssets;
	type RewardFraction = RewardFraction;
	type RewardOrigin = RewardOrigin;
	type OnChargeSystemToken = TransactionFeeCharger<Runtime, SystemTokenConversion, CreditHandler>;
	type BootstrapCallFilter = BootstrapCallFilter;
	type PalletId = FeeTreasuryId;
}

pub struct RewardOrigin;
impl RewardOriginInfo for RewardOrigin {
	type Origin = u32;
	fn reward_origin_info() -> voting::RewardOrigin<Self::Origin> {
		voting::RewardOrigin::Local
	}
}

pub struct CreditHandler;
impl HandleCredit<AccountId, NativeAndForeignAssets> for CreditHandler {
	fn handle_credit(credit: Credit<AccountId, NativeAndForeignAssets>) {
		if let Some(author) = pallet_authorship::Pallet::<Runtime>::author() {
			let _ = <NativeAndForeignAssets as Balanced<AccountId>>::resolve(&author, credit);
		}
	}
}

impl pallet_system_token_conversion::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = SystemTokenBalance;
	type AssetKind = Location;
	type Fungibles = NativeAndForeignAssets;
	type SystemConfig = Configuration;
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}
impl pallet_timestamp::Config for Runtime {
	type Moment = u64;
	type OnTimestampSet = Babe;
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = weights::pallet_timestamp::WeightInfo<Runtime>;
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Babe>;
	type EventHandler = ImOnline;
}

impl_opaque_keys! {
	pub struct SessionKeys {
		pub grandpa: Grandpa,
		pub babe: Babe,
		pub im_online: ImOnline,
		pub para_validator: Initializer,
		pub para_assignment: ParaSessionInfo,
		pub authority_discovery: AuthorityDiscovery,
	}
}

/// Special `ValidatorIdOf` implementation that is just returning the input as result.
pub struct ValidatorIdOf;
impl sp_runtime::traits::Convert<AccountId, Option<AccountId>> for ValidatorIdOf {
	fn convert(a: AccountId) -> Option<AccountId> {
		Some(a)
	}
}

impl pallet_session::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = AccountId;
	type ValidatorIdOf = ValidatorIdOf;
	type ShouldEndSession = Babe;
	type NextSessionRotation = Babe;
	type SessionManager = ValidatorManagement;
	type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	type WeightInfo = weights::pallet_session::WeightInfo<Runtime>;
}

// ToDo: Should be changed
pub struct FullIdentificationOf;
impl sp_runtime::traits::Convert<AccountId, Option<()>> for FullIdentificationOf {
	fn convert(_: AccountId) -> Option<()> {
		Some(Default::default())
	}
}

impl pallet_session::historical::Config for Runtime {
	type FullIdentification = ();
	type FullIdentificationOf = FullIdentificationOf;
}

parameter_types! {
	// Minimum 4 CENTS/byte
	pub const BasicDeposit: Balance = deposit(1, 258);
	pub const ByteDeposit: Balance = deposit(0, 1);
	pub const FieldDeposit: Balance = deposit(0, 66);
	pub const SubAccountDeposit: Balance = deposit(1, 53);
	pub const MaxSubAccounts: u32 = 100;
	pub const MaxAdditionalFields: u32 = 100;
	pub const MaxRegistrars: u32 = 20;
}

impl pallet_identity::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type BasicDeposit = BasicDeposit;
	type ByteDeposit = ByteDeposit;
	type SubAccountDeposit = SubAccountDeposit;
	type MaxSubAccounts = MaxSubAccounts;
	type IdentityInformation = IdentityInfo<MaxAdditionalFields>;
	type MaxRegistrars = MaxRegistrars;
	type Slashed = Treasury;
	type ForceOrigin = EnsureRoot<Self::AccountId>;
	type RegistrarOrigin = EnsureRoot<Self::AccountId>;
	type OffchainSignature = Signature;
	type SigningPublicKey = <Signature as Verify>::Signer;
	type UsernameAuthorityOrigin = EnsureRoot<Self::AccountId>;
	type PendingUsernameExpiration = ConstU32<{ 7 * DAYS }>;
	type MaxSuffixLength = ConstU32<7>;
	type MaxUsernameLength = ConstU32<32>;
	type WeightInfo = weights::pallet_identity::WeightInfo<Runtime>;
}

parameter_types! {
	pub LaunchPeriod: BlockNumber = prod_or_fast!(1 * MINUTES, 1, "DOT_LAUNCH_PERIOD");
	pub VotingPeriod: BlockNumber = prod_or_fast!(1 * MINUTES, 1 * MINUTES, "DOT_VOTING_PERIOD");
	pub FastTrackVotingPeriod: BlockNumber = prod_or_fast!(1 * MINUTES, 1 * MINUTES, "DOT_FAST_TRACK_VOTING_PERIOD");
	pub const MinimumDeposit: Balance = 100 * DOLLARS;
	pub EnactmentPeriod: BlockNumber = prod_or_fast!(1 * MINUTES, 1, "DOT_ENACTMENT_PERIOD");
	pub CooloffPeriod: BlockNumber = prod_or_fast!(1 * MINUTES, 1, "DOT_COOLOFF_PERIOD");
	pub const InstantAllowed: bool = true;
	pub const MaxVotes: u32 = 100;
	pub const MaxProposals: u32 = 100;
}

impl pallet_democracy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type EnactmentPeriod = EnactmentPeriod;
	type VoteLockingPeriod = EnactmentPeriod;
	type LaunchPeriod = LaunchPeriod;
	type VotingPeriod = VotingPeriod;
	type MinimumDeposit = MinimumDeposit;
	type SubmitOrigin = frame_system::EnsureSigned<AccountId>;
	/// A straight majority of the council can decide what their next motion is.
	type ExternalOrigin = EitherOfDiverse<
		pallet_collective::EnsureProportionAtLeast<AccountId, ValidatorCollective, 1, 2>,
		EnsureRoot<AccountId>,
	>;
	/// A 60% super-majority can have the next scheduled referendum be a straight majority-carries
	/// vote.
	type ExternalMajorityOrigin = EitherOfDiverse<
		pallet_collective::EnsureProportionAtLeast<AccountId, ValidatorCollective, 3, 5>,
		EnsureRoot<AccountId>,
	>;
	/// A unanimous council can have the next scheduled referendum be a straight default-carries
	/// (NTB) vote.
	type ExternalDefaultOrigin = EitherOfDiverse<
		pallet_collective::EnsureProportionAtLeast<AccountId, ValidatorCollective, 1, 1>,
		EnsureRoot<AccountId>,
	>;
	/// Two thirds of the technical committee can have an `ExternalMajority/ExternalDefault` vote
	/// be tabled immediately and with a shorter voting/enactment period.
	type FastTrackOrigin = EitherOfDiverse<
		pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 2, 3>,
		EnsureRoot<AccountId>,
	>;
	type InstantOrigin = EitherOfDiverse<
		pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 1, 1>,
		EnsureRoot<AccountId>,
	>;
	type InstantAllowed = InstantAllowed;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	// To cancel a proposal which has been passed, 2/3 of the council must agree to it.
	type CancellationOrigin = EitherOfDiverse<
		pallet_collective::EnsureProportionAtLeast<AccountId, ValidatorCollective, 2, 3>,
		EnsureRoot<AccountId>,
	>;
	// To cancel a proposal before it has been passed, the technical committee must be unanimous or
	// Root must agree.
	type CancelProposalOrigin = EitherOfDiverse<
		pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 1, 1>,
		EnsureRoot<AccountId>,
	>;
	type BlacklistOrigin = AuthorityOrigin;
	// Any single technical committee member may veto a coming council proposal, however they can
	// only do it once and it lasts only for the cooloff period.
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCollective>;
	type CooloffPeriod = CooloffPeriod;
	type Slash = Treasury;
	type Scheduler = Scheduler;
	type PalletsOrigin = OriginCaller;
	type MaxVotes = MaxVotes;
	type WeightInfo = weights::pallet_democracy::WeightInfo<Runtime>;
	type MaxProposals = MaxProposals;
	type Preimages = Preimage;
	type MaxDeposits = ConstU32<100>;
	type MaxBlacklisted = ConstU32<100>;
}

parameter_types! {
	pub CouncilMotionDuration: BlockNumber = prod_or_fast!(7 * DAYS, 2 * MINUTES, "DOT_MOTION_DURATION");
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 100;
}

pub type ValidatorCollective = pallet_collective::Instance1;
impl pallet_collective::Config<ValidatorCollective> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type SetMembersOrigin = AuthorityOrigin;
	type MaxProposalWeight = MaximumSchedulerWeight;
	type Proposal = RuntimeCall;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = weights::pallet_collective_council::WeightInfo<Runtime>;
}

parameter_types! {
	pub const CandidacyBond: Balance = 100 * DOLLARS;
	// 1 storage item created, key size is 32 bytes, value size is 16+16.
	pub const VotingBondBase: Balance = deposit(1, 64);
	// additional data per vote is 32 bytes (account id).
	pub const VotingBondFactor: Balance = deposit(0, 32);
	/// Weekly council elections; scaling up to monthly eventually.
	pub TermDuration: BlockNumber = prod_or_fast!(7 * DAYS, 2 * MINUTES, "DOT_TERM_DURATION");
	/// 13 members initially, to be increased to 23 eventually.
	pub const DesiredMembers: u32 = 13;
	pub const DesiredRunnersUp: u32 = 20;
	pub const MaxVoters: u32 = 10 * 1000;
	pub const MaxVotesPerVoter: u32 = 16;
	pub const MaxCandidates: u32 = 1000;
	pub const PhragmenElectionPalletId: LockIdentifier = *b"phrelect";
}
// Make sure that there are no more than `MaxMembers` members elected via phragmen.
const_assert!(DesiredMembers::get() <= CouncilMaxMembers::get());

impl pallet_elections_phragmen::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type PalletId = PhragmenElectionPalletId;
	type Currency = Balances;
	type ChangeMembers = Council;
	type InitializeMembers = Council;
	type CurrencyToVote = polkadot_runtime_common::CurrencyToVote;
	type CandidacyBond = CandidacyBond;
	type VotingBondBase = VotingBondBase;
	type VotingBondFactor = VotingBondFactor;
	type LoserCandidate = Treasury;
	type KickedMember = Treasury;
	type DesiredMembers = DesiredMembers;
	type DesiredRunnersUp = DesiredRunnersUp;
	type TermDuration = TermDuration;
	type MaxVoters = MaxVoters;
	type MaxVotesPerVoter = MaxVotesPerVoter;
	type MaxCandidates = MaxCandidates;
	type WeightInfo = weights::pallet_elections_phragmen::WeightInfo<Runtime>;
}

parameter_types! {
	pub const TechnicalMotionDuration: BlockNumber = 7 * DAYS;
	pub const TechnicalMaxProposals: u32 = 100;
	pub const TechnicalMaxMembers: u32 = 100;
}

pub type TechnicalCollective = pallet_collective::Instance2;
impl pallet_collective::Config<TechnicalCollective> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type SetMembersOrigin = AuthorityOrigin;
	type Proposal = RuntimeCall;
	type MotionDuration = TechnicalMotionDuration;
	type MaxProposals = TechnicalMaxProposals;
	type MaxMembers = TechnicalMaxMembers;
	type MaxProposalWeight = MaximumSchedulerWeight;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = weights::pallet_collective_technical_committee::WeightInfo<Runtime>;
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: Balance = 100 * DOLLARS;
	pub const ProposalBondMaximum: Balance = 500 * DOLLARS;
	pub const SpendPeriod: BlockNumber = 24 * DAYS;
	pub const Burn: Permill = Permill::from_percent(1);
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub TreasuryAccount: AccountId = Treasury::account_id();
	pub const TipCountdown: BlockNumber = 1 * DAYS;
	pub const TipFindersFee: Percent = Percent::from_percent(20);
	pub const TipReportDepositBase: Balance = 1 * DOLLARS;
	pub const DataDepositPerByte: Balance = 1 * CENTS;
	pub const MaxApprovals: u32 = 100;
	pub const MaxAuthorities: u32 = 100_000;
	pub const MaxKeys: u32 = 10_000;
	pub const MaxPeerInHeartbeats: u32 = 10_000;
	pub const RootSpendOriginMaxAmount: Balance = Balance::MAX;
	pub const SpendPayoutPeriod: BlockNumber = 30 * DAYS;
	pub const MaxBalance: Balance = Balance::max_value();
}

/// Either origin by governance or 3/5 validator origin
type AuthorityOrigin = EitherOfDiverse<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<AccountId, ValidatorCollective, 3, 5>,
>;

impl pallet_treasury::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type PalletId = TreasuryPalletId;
	type Currency = Balances;
	type RejectOrigin = AuthorityOrigin;
	type SpendOrigin = EnsureWithSuccess<EnsureRoot<AccountId>, AccountId, MaxBalance>;
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type BurnDestination = ();
	type SpendFunds = Bounties;
	type MaxApprovals = MaxApprovals;
	type AssetKind = Location;
	type Beneficiary = AccountId;
	type BeneficiaryLookup = Indices;
	type Paymaster = PayAssetFromAccount<NativeAndForeignAssets, TreasuryAccount>;
	type BalanceConverter = AssetRate;
	type PayoutPeriod = SpendPayoutPeriod;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
	type WeightInfo = weights::pallet_treasury::WeightInfo<Runtime>;
}

impl pallet_asset_rate::Config for Runtime {
	type CreateOrigin = AuthorityOrigin;
	type RemoveOrigin = AuthorityOrigin;
	type UpdateOrigin = AuthorityOrigin;
	type Currency = Balances;
	type AssetKind = Location;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_asset_rate::WeightInfo<Runtime>;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

parameter_types! {
	pub const BountyDepositBase: Balance = 1 * DOLLARS;
	pub const BountyDepositPayoutDelay: BlockNumber = 8 * DAYS;
	pub const BountyUpdatePeriod: BlockNumber = 90 * DAYS;
	pub const MaximumReasonLength: u32 = 16384;
	pub const CuratorDepositMultiplier: Permill = Permill::from_percent(50);
	pub const CuratorDepositMin: Balance = 10 * DOLLARS;
	pub const CuratorDepositMax: Balance = 200 * DOLLARS;
	pub const BountyValueMinimum: Balance = 10 * DOLLARS;
}

impl pallet_bounties::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type BountyDepositBase = BountyDepositBase;
	type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
	type BountyUpdatePeriod = BountyUpdatePeriod;
	type CuratorDepositMultiplier = CuratorDepositMultiplier;
	type CuratorDepositMin = CuratorDepositMin;
	type CuratorDepositMax = CuratorDepositMax;
	type BountyValueMinimum = BountyValueMinimum;
	type ChildBountyManager = ChildBounties;
	type DataDepositPerByte = DataDepositPerByte;
	type MaximumReasonLength = MaximumReasonLength;
	type WeightInfo = weights::pallet_bounties::WeightInfo<Runtime>;
	type OnSlash = Treasury;
}

parameter_types! {
	pub const MaxActiveChildBountyCount: u32 = 100;
	pub const ChildBountyValueMinimum: Balance = BountyValueMinimum::get() / 10;
}

impl pallet_child_bounties::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MaxActiveChildBountyCount = MaxActiveChildBountyCount;
	type ChildBountyValueMinimum = ChildBountyValueMinimum;
	type WeightInfo = weights::pallet_child_bounties::WeightInfo<Runtime>;
}

impl pallet_tips::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type DataDepositPerByte = DataDepositPerByte;
	type MaximumReasonLength = MaximumReasonLength;
	type Tippers = PhragmenElection;
	type TipCountdown = TipCountdown;
	type TipFindersFee = TipFindersFee;
	type TipReportDepositBase = TipReportDepositBase;
	type MaxTipAmount = ConstU128<{ 500 * DOLLARS }>;
	type WeightInfo = weights::pallet_tips::WeightInfo<Runtime>;
	type OnSlash = Treasury;
}

impl pallet_offences::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
	type OnOffenceHandler = ();
}

impl pallet_authority_discovery::Config for Runtime {
	type MaxAuthorities = MaxAuthorities;
}

parameter_types! {
	pub NposSolutionPriority: TransactionPriority =
		Perbill::from_percent(90) * TransactionPriority::max_value();
	pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
}

impl pallet_im_online::Config for Runtime {
	type AuthorityId = ImOnlineId;
	type RuntimeEvent = RuntimeEvent;
	type ValidatorSet = Historical;
	type NextSessionRotation = Babe;
	type ReportUnresponsiveness = Offences;
	type UnsignedPriority = ImOnlineUnsignedPriority;
	type WeightInfo = weights::pallet_im_online::WeightInfo<Runtime>;
	type MaxKeys = MaxKeys;
	type MaxPeerInHeartbeats = MaxPeerInHeartbeats;
}

parameter_types! {
	pub MaxSetIdSessionEntries: u32 = BondingDuration::get() * SessionsPerEra::get();
}

impl pallet_grandpa::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;

	type WeightInfo = ();
	type MaxAuthorities = MaxAuthorities;
	type MaxNominators = MaxNominatorRewardedPerValidator;
	type MaxSetIdSessionEntries = MaxSetIdSessionEntries;

	type KeyOwnerProof = <Historical as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

	type EquivocationReportSystem =
		pallet_grandpa::EquivocationReportSystem<Self, Offences, Historical, ReportLongevity>;
}

/// Submits a transaction with the node's public and signature type. Adheres to the signed extension
/// format of the chain.
impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
	RuntimeCall: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: RuntimeCall,
		public: <Signature as Verify>::Signer,
		account: AccountId,
		nonce: <Runtime as frame_system::Config>::Nonce,
	) -> Option<(RuntimeCall, <UncheckedExtrinsic as ExtrinsicT>::SignaturePayload)> {
		use sp_runtime::traits::StaticLookup;
		// take the biggest period possible.
		let period =
			BlockHashCount::get().checked_next_power_of_two().map(|c| c / 2).unwrap_or(2) as u64;

		let current_block = System::block_number()
			.saturated_into::<u64>()
			// The `System::block_number` is initialized with `n+1`,
			// so the actual block number is `n`.
			.saturating_sub(1);
		let extra: SignedExtra = (
			frame_system::CheckNonZeroSender::<Runtime>::new(),
			frame_system::CheckSpecVersion::<Runtime>::new(),
			frame_system::CheckTxVersion::<Runtime>::new(),
			frame_system::CheckGenesis::<Runtime>::new(),
			frame_system::CheckMortality::<Runtime>::from(generic::Era::mortal(
				period,
				current_block,
			)),
			frame_system::CheckNonce::<Runtime>::from(nonce),
			frame_system::CheckWeight::<Runtime>::new(),
			pallet_system_token_tx_payment::ChargeSystemToken::<Runtime>::new(),
		);
		let raw_payload = SignedPayload::new(call, extra)
			.map_err(|e| {
				log::warn!("Unable to create signed payload: {:?}", e);
			})
			.ok()?;
		let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
		let (call, extra, _) = raw_payload.deconstruct();
		let address = <Runtime as frame_system::Config>::Lookup::unlookup(account);
		Some((call, (address, signature, extra)))
	}
}

impl frame_system::offchain::SigningTypes for Runtime {
	type Public = <Signature as Verify>::Signer;
	type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
	RuntimeCall: From<C>,
{
	type Extrinsic = UncheckedExtrinsic;
	type OverarchingCall = RuntimeCall;
}

parameter_types! {
	pub const ParathreadDeposit: Balance = 500 * DOLLARS;
	pub const MaxRetries: u32 = 3;
}

impl pallet_utility::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = weights::pallet_utility::WeightInfo<Runtime>;
}

parameter_types! {
	// One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
	pub const DepositBase: Balance = deposit(1, 88);
	// Additional storage item size of 32 bytes.
	pub const DepositFactor: Balance = deposit(0, 32);
	pub const MaxSignatories: u32 = 100;
}

impl pallet_multisig::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type DepositBase = DepositBase;
	type DepositFactor = DepositFactor;
	type MaxSignatories = MaxSignatories;
	type WeightInfo = weights::pallet_multisig::WeightInfo<Runtime>;
}

parameter_types! {
	// One storage item; key size 32, value size 8; .
	pub const ProxyDepositBase: Balance = deposit(1, 8);
	// Additional storage item size of 33 bytes.
	pub const ProxyDepositFactor: Balance = deposit(0, 33);
	pub const MaxProxies: u16 = 32;
	pub const AnnouncementDepositBase: Balance = deposit(1, 8);
	pub const AnnouncementDepositFactor: Balance = deposit(0, 66);
	pub const MaxPending: u16 = 32;
}

/// The type used to represent the kinds of proxying allowed.
#[derive(
	Copy,
	Clone,
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Encode,
	Decode,
	sp_runtime::RuntimeDebug,
	MaxEncodedLen,
	scale_info::TypeInfo,
)]
pub enum ProxyType {
	Any = 0,
	NonTransfer = 1,
	Governance = 2,
	// Skip 4 as it is now removed (was SudoBalances)
	IdentityJudgement = 5,
	CancelProxy = 6,
	Auction = 7,
}

#[cfg(test)]
mod proxy_type_tests {
	use super::*;

	#[derive(
		Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, sp_runtime::RuntimeDebug,
	)]
	pub enum OldProxyType {
		Any,
		NonTransfer,
		Governance,
		SudoBalances,
		IdentityJudgement,
	}

	#[test]
	fn proxy_type_decodes_correctly() {
		for (i, j) in vec![
			(OldProxyType::Any, ProxyType::Any),
			(OldProxyType::NonTransfer, ProxyType::NonTransfer),
			(OldProxyType::Governance, ProxyType::Governance),
			(OldProxyType::IdentityJudgement, ProxyType::IdentityJudgement),
		]
		.into_iter()
		{
			assert_eq!(i.encode(), j.encode());
		}
		assert!(ProxyType::decode(&mut &OldProxyType::SudoBalances.encode()[..]).is_err());
	}
}

impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}
impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::NonTransfer => matches!(
				c,
				RuntimeCall::System(..) |
				RuntimeCall::Scheduler(..) |
				RuntimeCall::Babe(..) |
				RuntimeCall::Timestamp(..) |
				RuntimeCall::Indices(pallet_indices::Call::claim{..}) |
				RuntimeCall::Indices(pallet_indices::Call::free{..}) |
				RuntimeCall::Indices(pallet_indices::Call::freeze{..}) |
				// Specifically omitting Indices `transfer`, `force_transfer`
				// Specifically omitting the entire Balances pallet
				RuntimeCall::Session(..) |
				RuntimeCall::Grandpa(..) |
				RuntimeCall::ImOnline(..) |
				RuntimeCall::Democracy(..) |
				RuntimeCall::Council(..) |
				RuntimeCall::TechnicalCommittee(..) |
				RuntimeCall::PhragmenElection(..) |
				RuntimeCall::Treasury(..) |
				RuntimeCall::Bounties(..) |
				RuntimeCall::ChildBounties(..) |
				RuntimeCall::Tips(..) |
				// Specifically omitting Vesting `vested_transfer`, and `force_vested_transfer`
				RuntimeCall::Utility(..) |
				RuntimeCall::Identity(..) |
				RuntimeCall::Proxy(..) |
				RuntimeCall::Multisig(..) |
				RuntimeCall::Registrar(paras_registrar::Call::register {..}) |
				RuntimeCall::Registrar(paras_registrar::Call::deregister {..}) |
				// Specifically omitting Registrar `swap`
				RuntimeCall::Registrar(paras_registrar::Call::reserve {..}) |
				RuntimeCall::Crowdloan(..) |
				RuntimeCall::Slots(..) |
				RuntimeCall::Auctions(..) // Specifically omitting the entire XCM Pallet
			),
			ProxyType::Governance => matches!(
				c,
				RuntimeCall::Democracy(..)
					| RuntimeCall::Council(..)
					| RuntimeCall::TechnicalCommittee(..)
					| RuntimeCall::PhragmenElection(..)
					| RuntimeCall::Treasury(..)
					| RuntimeCall::Bounties(..)
					| RuntimeCall::Tips(..)
					| RuntimeCall::Utility(..)
					| RuntimeCall::ChildBounties(..)
			),
			ProxyType::IdentityJudgement => matches!(
				c,
				RuntimeCall::Identity(pallet_identity::Call::provide_judgement { .. })
					| RuntimeCall::Utility(..)
			),
			ProxyType::CancelProxy => {
				matches!(c, RuntimeCall::Proxy(pallet_proxy::Call::reject_announcement { .. }))
			},
			ProxyType::Auction => matches!(
				c,
				RuntimeCall::Auctions(..)
					| RuntimeCall::Crowdloan(..)
					| RuntimeCall::Registrar(..)
					| RuntimeCall::Slots(..)
			),
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(ProxyType::Any, _) => true,
			(_, ProxyType::Any) => false,
			(ProxyType::NonTransfer, _) => true,
			_ => false,
		}
	}
}

impl pallet_proxy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type MaxProxies = MaxProxies;
	type WeightInfo = weights::pallet_proxy::WeightInfo<Runtime>;
	type MaxPending = MaxPending;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
}

impl parachains_origin::Config for Runtime {}

impl parachains_configuration::Config for Runtime {
	type WeightInfo = weights::runtime_parachains_configuration::WeightInfo<Runtime>;
	type ParaConfigHandler = ParaConfigHandler;
}

impl parachains_shared::Config for Runtime {
	type DisabledValidators = Session;
}

impl parachains_session_info::Config for Runtime {
	type ValidatorSet = Historical;
}

// ToDo: Should be changed
pub struct RewardValidators;
impl polkadot_runtime_parachains::inclusion::RewardValidators for RewardValidators {
	fn reward_backing(_: impl IntoIterator<Item = ValidatorIndex>) {}
	fn reward_bitfields(_: impl IntoIterator<Item = ValidatorIndex>) {}
}

impl parachains_inclusion::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type DisputesHandler = ParasDisputes;
	type RewardValidators = RewardValidators;
	type PoTHandler = ValidatorManagement;
	type MessageQueue = MessageQueue;
	type WeightInfo = weights::runtime_parachains_inclusion::WeightInfo<Runtime>;
}

parameter_types! {
	pub const SessionsPerEra: u32 = 3;
	pub const BlocksPerYear: BlockNumber = YEAR;
}

impl pallet_validator_management::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type BlocksPerYear = BlocksPerYear;
	type SessionsPerEra = SessionsPerEra;
	type RewardHandler = RewardHandler;
	type Fungibles = NativeAndForeignAssets;
	type Score = SystemTokenWeight;
	type HigherPrecisionScore = softfloat::F64;
	type NextNewSession = Session;
	type SessionInterface = Self;
	type CollectiveInterface = Council;
}

parameter_types! {
	pub const MaxOriginalUsedParaIds: u32 = 10;
	pub const MaxSystemTokens: u32 = 10;
	pub const AssetHubId: u32 = ASSET_HUB_ID;
	pub const SystemTokenManagerId: PalletId = PalletId(*b"systoken");
}

impl system_token_manager::Config for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeEvent = RuntimeEvent;
	type Fungibles = NativeAndForeignAssets;
	type HigherPrecision = softfloat::F64;
	type SystemTokenId = Location;
	type UniversalLocation = UniversalLocation;
	type SystemTokenHandler = SystemTokenHandler;
	type OracleManager = OracleManager;
	type StringLimit = StringLimit;
	type MaxSystemTokens = MaxSystemTokens;
	type MaxOriginalUsedParaIds = MaxOriginalUsedParaIds;
	type AssetHubId = AssetHubId;
	type PalletId = SystemTokenManagerId;
}

parameter_types! {
	pub const ParasUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
}

impl parachains_paras::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::runtime_parachains_paras::WeightInfo<Runtime>;
	type UnsignedPriority = ParasUnsignedPriority;
	type QueueFootprinter = ParaInclusion;
	type NextSessionRotation = Babe;
	type OnNewHead = Registrar;
	type AssignCoretime = CoretimeAssignmentProvider;
}

parameter_types! {
	/// Amount of weight that can be spent per block to service messages.
	///
	/// # WARNING
	///
	/// This is not a good value for para-chains since the `Scheduler` already uses up to 80% block weight.
	pub MessageQueueServiceWeight: Weight = Perbill::from_percent(20) * BlockWeights::get().max_block;
	pub const MessageQueueHeapSize: u32 = 32 * 1024;
	pub const MessageQueueMaxStale: u32 = 96;
}

/// Message processor to handle any messages that were enqueued into the `MessageQueue` pallet.
pub struct MessageProcessor;
impl ProcessMessage for MessageProcessor {
	type Origin = AggregateMessageOrigin;

	fn process_message(
		message: &[u8],
		origin: Self::Origin,
		meter: &mut WeightMeter,
		id: &mut [u8; 32],
	) -> Result<bool, ProcessMessageError> {
		let para = match origin {
			AggregateMessageOrigin::Ump(UmpQueueId::Para(para)) => para,
		};
		xcm_builder::ProcessXcmMessage::<
			Junction,
			xcm_executor::XcmExecutor<xcm_config::XcmConfig>,
			RuntimeCall,
		>::process_message(message, Junction::Parachain(para.into()), meter, id)
	}
}

impl pallet_message_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Size = u32;
	type HeapSize = MessageQueueHeapSize;
	type MaxStale = MessageQueueMaxStale;
	type ServiceWeight = MessageQueueServiceWeight;
	type IdleMaxServiceWeight = MessageQueueServiceWeight;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type MessageProcessor = MessageProcessor;
	#[cfg(feature = "runtime-benchmarks")]
	type MessageProcessor =
		pallet_message_queue::mock_helpers::NoopMessageProcessor<AggregateMessageOrigin>;
	type QueueChangeHandler = ParaInclusion;
	type QueuePausedQuery = ();
	type WeightInfo = weights::pallet_message_queue::WeightInfo<Runtime>;
}

impl parachains_dmp::Config for Runtime {}

parameter_types! {
	pub const HrmpChannelSizeAndCapacityWithSystemRatio: Percent = Percent::from_percent(100);
}

impl parachains_hrmp::Config for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeEvent = RuntimeEvent;
	type ChannelManager = AuthorityOrigin;
	type Currency = Balances;
	type DefaultChannelSizeAndCapacityWithSystem = ActiveConfigHrmpChannelSizeAndCapacityRatio<
		Runtime,
		HrmpChannelSizeAndCapacityWithSystemRatio,
	>;
	type VersionWrapper = crate::XcmPallet;
	type WeightInfo = weights::runtime_parachains_hrmp::WeightInfo<Self>;
}

impl parachains_paras_inherent::Config for Runtime {
	type WeightInfo = weights::runtime_parachains_paras_inherent::WeightInfo<Runtime>;
}

impl parachains_scheduler::Config for Runtime {
	type AssignmentProvider = CoretimeAssignmentProvider;
}

parameter_types! {
	pub const BrokerId: u32 = BROKER_ID;
	pub const BrokerPalletId: PalletId = PalletId(*b"py/broke");
	pub MaxXcmTransactWeight: Weight = Weight::from_parts(200_000_000, 20_000);
}

pub struct BrokerPot;
impl Get<InteriorLocation> for BrokerPot {
	fn get() -> InteriorLocation {
		Junction::AccountId32 { network: None, id: BrokerPalletId::get().into_account_truncating() }
			.into()
	}
}

impl coretime::Config for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type BrokerId = BrokerId;
	type BrokerPotLocation = BrokerPot;
	type WeightInfo = weights::runtime_parachains_coretime::WeightInfo<Runtime>;
	type SendXcm = crate::xcm_config::XcmRouter;
	type AssetTransactor = crate::xcm_config::NativeIssuedFungiblesTransactor;
	type AccountToLocation = xcm_builder::AliasesIntoAccountId32<
		xcm_config::ThisNetwork,
		<Runtime as frame_system::Config>::AccountId,
	>;
	type MaxXcmTransactWeight = MaxXcmTransactWeight;
}

parameter_types! {
	pub const OnDemandTrafficDefaultValue: FixedU128 = FixedU128::from_u32(1);
	// Keep 2 timeslices worth of revenue information.
	pub const MaxHistoricalRevenue: BlockNumber = 2 * TIMESLICE_PERIOD;
	pub const OnDemandPalletId: PalletId = PalletId(*b"py/ondmd");
}

impl parachains_assigner_on_demand::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type TrafficDefaultValue = OnDemandTrafficDefaultValue;
	type WeightInfo = weights::runtime_parachains_assigner_on_demand::WeightInfo<Runtime>;
	type MaxHistoricalRevenue = MaxHistoricalRevenue;
	type PalletId = OnDemandPalletId;
}

impl parachains_assigner_coretime::Config for Runtime {}

impl parachains_initializer::Config for Runtime {
	type Randomness = pallet_babe::RandomnessFromOneEpochAgo<Runtime>;
	type ForceOrigin = EnsureRoot<AccountId>;
	type WeightInfo = weights::runtime_parachains_initializer::WeightInfo<Runtime>;
	type CoretimeOnNewSession = Coretime;
}

impl parachains_disputes::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RewardValidators = ();
	type SlashingHandler = ();
	type WeightInfo = weights::runtime_parachains_disputes::WeightInfo<Runtime>;
}

impl parachains_slashing::Config for Runtime {
	type KeyOwnerProofSystem = Historical;
	type KeyOwnerProof =
		<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, ValidatorId)>>::Proof;
	type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
		KeyTypeId,
		ValidatorId,
	)>>::IdentificationTuple;
	type HandleReports = parachains_slashing::SlashingReportHandler<
		Self::KeyOwnerIdentification,
		Offences,
		ReportLongevity,
	>;
	type WeightInfo = parachains_slashing::TestWeightInfo;
	type BenchmarkingConfig = parachains_slashing::BenchConfig<200>;
}

parameter_types! {
	// Mostly arbitrary deposit price, but should provide an adequate incentive not to spam reserve
	// `ParaId`s.
	pub const ParaDeposit: Balance = 100 * DOLLARS;
	pub const ParaDataByteDeposit: Balance = deposit(0, 1);
}

impl paras_registrar::Config for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type OnSwap = (Crowdloan, Slots);
	type ParaDeposit = ParaDeposit;
	type DataDepositPerByte = ParaDataByteDeposit;
	type WeightInfo = weights::runtime_common_paras_registrar::WeightInfo<Runtime>;
}

parameter_types! {
	// 12 weeks = 3 months per lease period -> 8 lease periods ~ 2 years
	pub LeasePeriod: BlockNumber = prod_or_fast!(12 * WEEKS, 12 * WEEKS, "DOT_LEASE_PERIOD");
	// infrablockspace Genesis was on May 26, 2020.
	// Target Parachain Onboarding Date: Dec 15, 2021.
	// Difference is 568 days.
	// We want a lease period to start on the target onboarding date.
	// 568 % (12 * 7) = 64 day offset
	pub LeaseOffset: BlockNumber = prod_or_fast!(64 * DAYS, 0, "DOT_LEASE_OFFSET");
}

impl slots::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type Registrar = Registrar;
	type LeasePeriod = LeasePeriod;
	type LeaseOffset = LeaseOffset;
	type ForceOrigin = MoreThanHalfCouncil;
	type WeightInfo = weights::runtime_common_slots::WeightInfo<Runtime>;
}

parameter_types! {
	pub const CrowdloanId: PalletId = PalletId(*b"py/cfund");
	// Accounts for 10_000 contributions, each using 48 bytes (16 bytes for balance, and 32 bytes
	// for a memo).
	pub const SubmissionDeposit: Balance = deposit(1, 480_000);
	// The minimum crowdloan contribution.
	pub const MinContribution: Balance = 5 * DOLLARS;
	pub const RemoveKeysLimit: u32 = 1000;
	// Allow 32 bytes for an additional memo to a crowdloan.
	pub const MaxMemoLength: u8 = 32;
}

impl crowdloan::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type PalletId = CrowdloanId;
	type SubmissionDeposit = SubmissionDeposit;
	type MinContribution = MinContribution;
	type RemoveKeysLimit = RemoveKeysLimit;
	type Registrar = Registrar;
	type Auctioneer = Auctions;
	type MaxMemoLength = MaxMemoLength;
	type WeightInfo = weights::runtime_common_crowdloan::WeightInfo<Runtime>;
}

parameter_types! {
	// The average auction is 7 days long, so this will be 70% for ending period.
	// 5 Days = 72000 Blocks @ 6 sec per block
	pub const EndingPeriod: BlockNumber = 5 * DAYS;
	// ~ 1000 samples per day -> ~ 20 blocks per sample -> 2 minute samples
	pub const SampleLength: BlockNumber = 2 * MINUTES;
}

type AuctionInitiate = EitherOfDiverse<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<AccountId, ValidatorCollective, 2, 3>,
>;

impl auctions::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Leaser = Slots;
	type Registrar = Registrar;
	type EndingPeriod = EndingPeriod;
	type SampleLength = SampleLength;
	type Randomness = pallet_babe::RandomnessFromOneEpochAgo<Runtime>;
	type InitiateOrigin = AuctionInitiate;
	type WeightInfo = weights::runtime_common_auctions::WeightInfo<Runtime>;
}

parameter_types! {
	pub const DepositToCreateAsset: Balance = 1 * DOLLARS;
	pub const DepositToMaintainAsset: Balance = deposit(1, 16);
	pub const ApprovalDeposit: Balance = EXISTENTIAL_DEPOSIT;
	pub const StringLimit: u32 = 50;
	/// Key = 32 bytes, Value = 36 bytes (32+1+1+1+1)
	pub const MetadataDepositBase: Balance = deposit(1, 68);
	pub const MetadataDepositPerByte: Balance = deposit(0, 1);
}

pub type NativeAssetsInstance = pallet_assets::Instance1;
type NativeAssetsCall = pallet_assets::Call<Runtime, NativeAssetsInstance>;
impl pallet_assets::Config<NativeAssetsInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = AssetIdForNativeAssets;
	type AssetIdParameter = codec::Compact<AssetIdForNativeAssets>;
	type SystemTokenWeight = SystemTokenWeight;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<AccountId>>;
	type ForceOrigin = AuthorityOrigin;
	type AssetDeposit = DepositToCreateAsset;
	type AssetAccountDeposit = DepositToMaintainAsset;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = StringLimit;
	type Freezer = ();
	type Extra = ();
	type CallbackHandle = ();
	type WeightInfo = ();
	type RemoveItemsLimit = ConstU32<1000>;
}

pub type ForeignAssetsInstance = pallet_assets::Instance2;
impl pallet_assets::Config<ForeignAssetsInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = Location;
	type AssetIdParameter = Location;
	type SystemTokenWeight = SystemTokenWeight;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<AccountId>>;
	type ForceOrigin = AuthorityOrigin;
	type AssetDeposit = DepositToCreateAsset;
	type AssetAccountDeposit = DepositToMaintainAsset;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = StringLimit;
	type Freezer = ();
	type Extra = ();
	type CallbackHandle = ();
	type WeightInfo = ();
	type RemoveItemsLimit = ConstU32<1000>;
}

pub struct ReanchorHandler;
impl ReanchorSystemToken<Location> for ReanchorHandler {
	type Error = ();
	fn reanchor_system_token(l: &mut Location) -> Result<(), Self::Error> {
		let target = Location::here();
		let context = UniversalLocation::get();
		l.reanchor(&target, &context).map_err(|_| {})?;
		Ok(())
	}
}

/// Union fungibles implementation for `Assets` and `ForeignAssets`.
pub type NativeAndForeignAssets = fungibles::UnionOf<
	Assets,
	ForeignAssets,
	LocalFromLeft<
		AssetIdForNativeAssetsConvert<NativeAssetsPalletLocation>,
		AssetIdForNativeAssets,
		Location,
	>,
	Location,
	AccountId,
	ReanchorHandler,
>;

construct_runtime! {
	pub enum Runtime
	{
		// Basic stuff; balances is uncallable initially.
		System: frame_system = 0,
		Scheduler: pallet_scheduler = 1,
		Preimage: pallet_preimage = 10,
		// Asset rate.
		AssetRate: pallet_asset_rate = 39,

		// InfraBlockchain Support
		SystemTokenManager: system_token_manager = 21,
		Assets: pallet_assets::<Instance1> = 22,
		ForeignAssets: pallet_assets::<Instance2> = 23,
		SystemTokenConversion: pallet_system_token_conversion = 24,

		// Babe must be before session.
		Babe: pallet_babe = 2,

		Timestamp: pallet_timestamp = 3,
		Indices: pallet_indices = 4,
		Balances: pallet_balances = 5,

		TransactionPayment: pallet_transaction_payment = 31,
		SystemTokenTxPayment: pallet_system_token_tx_payment = 32,

		// Consensus support.
		// Authorship must be before session in order to note author in the correct session and era
		// for im-online and staking.
		Authorship: pallet_authorship = 7,
		Offences: pallet_offences = 8,
		Historical: session_historical = 33,

		// This should be above Session Pallet
		ValidatorManagement: pallet_validator_management = 25,
		Session: pallet_session = 9,
		Grandpa: pallet_grandpa = 11,
		ImOnline: pallet_im_online = 12,
		AuthorityDiscovery: pallet_authority_discovery = 13,

		// Governance stuff.
		Democracy: pallet_democracy = 14,
		Council: pallet_collective::<Instance1> = 15,
		TechnicalCommittee: pallet_collective::<Instance2> = 16,
		PhragmenElection: pallet_elections_phragmen = 17,
		Treasury: pallet_treasury = 19,

		// Cunning utilities. Usable initially.
		Utility: pallet_utility = 27,

		// Identity. Late addition.
		Identity: pallet_identity = 28,

		// Proxy module. Late addition.
		Proxy: pallet_proxy = 29,

		// Multisig dispatch. Late addition.
		Multisig: pallet_multisig = 30,

		// Bounties modules.
		Bounties: pallet_bounties = 34,
		ChildBounties: pallet_child_bounties = 38,

		// Tips module.
		Tips: pallet_tips = 35,

		// Parachains pallets. Start indices at 50 to leave room.
		ParachainsOrigin: parachains_origin = 50,
		Configuration: parachains_configuration = 51,
		ParasShared: parachains_shared = 52,
		ParaInclusion: parachains_inclusion = 53,
		ParaInherent: parachains_paras_inherent = 54,
		ParaScheduler: parachains_scheduler = 55,
		Paras: parachains_paras = 56,
		Initializer: parachains_initializer = 57,
		Dmp: parachains_dmp = 58,
		Hrmp: parachains_hrmp = 60,
		ParaSessionInfo: parachains_session_info = 61,
		ParasDisputes: parachains_disputes = 62,
		ParasSlashing: parachains_slashing = 63,
		MessageQueue: pallet_message_queue = 64,
		CoretimeAssignmentProvider: parachains_assigner_coretime = 65,
		OnDemandAssignmentProvider: parachains_assigner_on_demand = 66,

		// Parachain Onboarding Pallets. Start indices at 70 to leave room.
		Registrar: paras_registrar = 70,
		Slots: slots = 71,
		Auctions: auctions = 72,
		Crowdloan: crowdloan = 73,
		Coretime: coretime = 74,

		Sudo: pallet_sudo = 81,
		ParasSudoWrapper: paras_sudo_wrapper = 82,

		// Pallet for sending XCM.
		XcmPallet: pallet_xcm = 99,
	}
}

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// `BlockId` type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The `SignedExtension` to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckMortality<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_system_token_tx_payment::ChargeSystemToken<Runtime>,
);

/// All migrations that will run on the next runtime upgrade.
///
/// Should be cleared after every release.
pub type Migrations = ();

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
	Migrations,
>;

/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	define_benchmarks!(
		// infrablockspace
		// NOTE: Make sure to prefix these with `runtime_common::` so
		// the that path resolves correctly in the generated file.
		[polkadot_runtime_common::auctions, Auctions]
		[polkadot_runtime_common::claims, Claims]
		[polkadot_runtime_common::crowdloan, Crowdloan]
		[polkadot_runtime_common::slots, Slots]
		[polkadot_runtime_common::paras_registrar, Registrar]
		[runtime_parachains::configuration, Configuration]
		[runtime_parachains::disputes, ParasDisputes]
		[runtime_parachains::hrmp, Hrmp]
		[runtime_parachains::initializer, Initializer]
		[runtime_parachains::paras, Paras]
		[runtime_parachains::paras_inherent, ParaInherent]
		[runtime_parachains::ump, Ump]
		// Substrate
		[pallet_bags_list, VoterList]
		[pallet_balances, Balances]
		[frame_benchmarking::baseline, Baseline::<Runtime>]
		[pallet_bounties, Bounties]
		[pallet_child_bounties, ChildBounties]
		[pallet_collective, Council]
		[pallet_collective, TechnicalCommittee]
		[pallet_democracy, Democracy]
		[pallet_elections_phragmen, PhragmenElection]
		[pallet_election_provider_multi_phase, ElectionProviderMultiPhase]
		[frame_election_provider_support, ElectionProviderBench::<Runtime>]
		[pallet_fast_unstake, FastUnstake]
		[pallet_identity, Identity]
		[pallet_im_online, ImOnline]
		[pallet_indices, Indices]
		[pallet_multisig, Multisig]
		[pallet_nomination_pools, NominationPoolsBench::<Runtime>]
		[pallet_offences, OffencesBench::<Runtime>]
		[pallet_preimage, Preimage]
		[pallet_proxy, Proxy]
		[pallet_scheduler, Scheduler]
		[pallet_session, SessionBench::<Runtime>]
		[pallet_staking, Staking]
		[frame_system, SystemBench::<Runtime>]
		[pallet_timestamp, Timestamp]
		[pallet_tips, Tips]
		[pallet_treasury, Treasury]
		[pallet_utility, Utility]
		[pallet_vesting, Vesting]
		// XCM
		[pallet_xcm, XcmPallet]
	);
}

sp_api::impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block);
		}

		fn initialize_block(header: &<Block as BlockT>::Header) -> sp_runtime::ExtrinsicInclusionMode {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}
		fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}

		fn metadata_versions() -> sp_std::vec::Vec<u32> {
			Runtime::metadata_versions()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	#[api_version(11)]
	impl polkadot_primitives::runtime_api::ParachainHost<Block> for Runtime {
		fn validators() -> Vec<ValidatorId> {
			parachains_runtime_api_impl::validators::<Runtime>()
		}

		fn validator_groups() -> (Vec<Vec<ValidatorIndex>>, GroupRotationInfo<BlockNumber>) {
			parachains_runtime_api_impl::validator_groups::<Runtime>()
		}

		fn availability_cores() -> Vec<CoreState<Hash, BlockNumber>> {
			parachains_runtime_api_impl::availability_cores::<Runtime>()
		}

		fn persisted_validation_data(para_id: ParaId, assumption: OccupiedCoreAssumption)
			-> Option<PersistedValidationData<Hash, BlockNumber>> {
			parachains_runtime_api_impl::persisted_validation_data::<Runtime>(para_id, assumption)
		}

		fn assumed_validation_data(
			para_id: ParaId,
			expected_persisted_validation_data_hash: Hash,
		) -> Option<(PersistedValidationData<Hash, BlockNumber>, ValidationCodeHash)> {
			parachains_runtime_api_impl::assumed_validation_data::<Runtime>(
				para_id,
				expected_persisted_validation_data_hash,
			)
		}

		fn check_validation_outputs(
			para_id: ParaId,
			outputs: polkadot_primitives::CandidateCommitments,
		) -> bool {
			parachains_runtime_api_impl::check_validation_outputs::<Runtime>(para_id, outputs)
		}

		fn session_index_for_child() -> SessionIndex {
			parachains_runtime_api_impl::session_index_for_child::<Runtime>()
		}

		fn validation_code(para_id: ParaId, assumption: OccupiedCoreAssumption)
			-> Option<ValidationCode> {
			parachains_runtime_api_impl::validation_code::<Runtime>(para_id, assumption)
		}

		fn candidate_pending_availability(para_id: ParaId) -> Option<CommittedCandidateReceipt<Hash>> {
			#[allow(deprecated)]
			parachains_runtime_api_impl::candidate_pending_availability::<Runtime>(para_id)
		}

		fn candidate_events() -> Vec<CandidateEvent<Hash>> {
			parachains_runtime_api_impl::candidate_events::<Runtime, _>(|ev| {
				match ev {
					RuntimeEvent::ParaInclusion(ev) => {
						Some(ev)
					}
					_ => None,
				}
			})
		}

		fn session_info(index: SessionIndex) -> Option<SessionInfo> {
			parachains_runtime_api_impl::session_info::<Runtime>(index)
		}

		fn session_executor_params(session_index: SessionIndex) -> Option<ExecutorParams> {
			parachains_runtime_api_impl::session_executor_params::<Runtime>(session_index)
		}

		fn dmq_contents(recipient: ParaId) -> Vec<InboundDownwardMessage<BlockNumber>> {
			parachains_runtime_api_impl::dmq_contents::<Runtime>(recipient)
		}

		fn inbound_hrmp_channels_contents(
			recipient: ParaId
		) -> BTreeMap<ParaId, Vec<InboundHrmpMessage<BlockNumber>>> {
			parachains_runtime_api_impl::inbound_hrmp_channels_contents::<Runtime>(recipient)
		}

		fn validation_code_by_hash(hash: ValidationCodeHash) -> Option<ValidationCode> {
			parachains_runtime_api_impl::validation_code_by_hash::<Runtime>(hash)
		}

		fn on_chain_votes() -> Option<ScrapedOnChainVotes<Hash>> {
			parachains_runtime_api_impl::on_chain_votes::<Runtime>()
		}

		fn submit_pvf_check_statement(
			stmt: polkadot_primitives::PvfCheckStatement,
			signature: polkadot_primitives::ValidatorSignature
		) {
			parachains_runtime_api_impl::submit_pvf_check_statement::<Runtime>(stmt, signature)
		}

		fn pvfs_require_precheck() -> Vec<ValidationCodeHash> {
			parachains_runtime_api_impl::pvfs_require_precheck::<Runtime>()
		}

		fn validation_code_hash(para_id: ParaId, assumption: OccupiedCoreAssumption)
			-> Option<ValidationCodeHash>
		{
			parachains_runtime_api_impl::validation_code_hash::<Runtime>(para_id, assumption)
		}

		fn disputes() -> Vec<(SessionIndex, CandidateHash, DisputeState<BlockNumber>)> {
			parachains_runtime_api_impl::get_session_disputes::<Runtime>()
		}

		fn unapplied_slashes(
		) -> Vec<(SessionIndex, CandidateHash, slashing::PendingSlashes)> {
			parachains_runtime_api_impl::unapplied_slashes::<Runtime>()
		}

		fn key_ownership_proof(
			validator_id: ValidatorId,
		) -> Option<slashing::OpaqueKeyOwnershipProof> {
			use codec::Encode;

			Historical::prove((PARACHAIN_KEY_TYPE_ID, validator_id))
				.map(|p| p.encode())
				.map(slashing::OpaqueKeyOwnershipProof::new)
		}

		fn submit_report_dispute_lost(
			dispute_proof: slashing::DisputeProof,
			key_ownership_proof: slashing::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			parachains_runtime_api_impl::submit_unsigned_slashing_report::<Runtime>(
				dispute_proof,
				key_ownership_proof,
			)
		}

		fn minimum_backing_votes() -> u32 {
			parachains_runtime_api_impl::minimum_backing_votes::<Runtime>()
		}

		fn para_backing_state(para_id: ParaId) -> Option<polkadot_primitives::async_backing::BackingState> {
			parachains_runtime_api_impl::backing_state::<Runtime>(para_id)
		}

		fn async_backing_params() -> polkadot_primitives::AsyncBackingParams {
			parachains_runtime_api_impl::async_backing_params::<Runtime>()
		}

		fn approval_voting_params() -> ApprovalVotingParams {
			parachains_runtime_api_impl::approval_voting_params::<Runtime>()
		}

		fn disabled_validators() -> Vec<ValidatorIndex> {
			parachains_runtime_api_impl::disabled_validators::<Runtime>()
		}

		fn node_features() -> NodeFeatures {
			parachains_runtime_api_impl::node_features::<Runtime>()
		}

		fn claim_queue() -> BTreeMap<CoreIndex, VecDeque<ParaId>> {
			vstaging_parachains_runtime_api_impl::claim_queue::<Runtime>()
		}

		fn candidates_pending_availability(para_id: ParaId) -> Vec<CommittedCandidateReceipt<Hash>> {
			vstaging_parachains_runtime_api_impl::candidates_pending_availability::<Runtime>(para_id)
		}
	}

	impl fg_primitives::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> Vec<(GrandpaId, u64)> {
			Grandpa::grandpa_authorities()
		}

		fn current_set_id() -> fg_primitives::SetId {
			Grandpa::current_set_id()
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			equivocation_proof: fg_primitives::EquivocationProof<
				<Block as BlockT>::Hash,
				sp_runtime::traits::NumberFor<Block>,
			>,
			key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			let key_owner_proof = key_owner_proof.decode()?;

			Grandpa::submit_unsigned_equivocation_report(
				equivocation_proof,
				key_owner_proof,
			)
		}

		fn generate_key_ownership_proof(
			_set_id: fg_primitives::SetId,
			authority_id: fg_primitives::AuthorityId,
		) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
			use codec::Encode;

			Historical::prove((fg_primitives::KEY_TYPE, authority_id))
				.map(|p| p.encode())
				.map(fg_primitives::OpaqueKeyOwnershipProof::new)
		}
	}

	impl sp_consensus_babe::BabeApi<Block> for Runtime {
		fn configuration() -> sp_consensus_babe::BabeConfiguration {
			let epoch_config = Babe::epoch_config().unwrap_or(BABE_GENESIS_EPOCH_CONFIG);
			sp_consensus_babe::BabeConfiguration {
				slot_duration: Babe::slot_duration(),
				epoch_length: EpochDurationInBlocks::get().into(),
				c: epoch_config.c,
				authorities: Babe::authorities().to_vec(),
				randomness: Babe::randomness(),
				allowed_slots: epoch_config.allowed_slots,
			}
		}

		fn current_epoch_start() -> sp_consensus_babe::Slot {
			Babe::current_epoch_start()
		}

		fn current_epoch() -> sp_consensus_babe::Epoch {
			Babe::current_epoch()
		}

		fn next_epoch() -> sp_consensus_babe::Epoch {
			Babe::next_epoch()
		}

		fn generate_key_ownership_proof(
			_slot: sp_consensus_babe::Slot,
			authority_id: sp_consensus_babe::AuthorityId,
		) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
			use codec::Encode;

			Historical::prove((sp_consensus_babe::KEY_TYPE, authority_id))
				.map(|p| p.encode())
				.map(sp_consensus_babe::OpaqueKeyOwnershipProof::new)
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
			key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			let key_owner_proof = key_owner_proof.decode()?;

			Babe::submit_unsigned_equivocation_report(
				equivocation_proof,
				key_owner_proof,
			)
		}
	}

	impl sp_authority_discovery::AuthorityDiscoveryApi<Block> for Runtime {
		fn authorities() -> Vec<AuthorityDiscoveryId> {
			parachains_runtime_api_impl::relevant_authority_ids::<Runtime>()
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<
		Block,
		Balance,
	> for Runtime {
		fn query_info(uxt: <Block as BlockT>::Extrinsic, len: u32) -> RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(uxt: <Block as BlockT>::Extrinsic, len: u32) -> FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall>
		for Runtime
	{
		fn query_call_info(call: RuntimeCall, len: u32) -> RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_call_info(call, len)
		}
		fn query_call_fee_details(call: RuntimeCall, len: u32) -> FeeDetails<Balance> {
			TransactionPayment::query_call_fee_details(call, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
			log::info!("try-runtime::on_runtime_upgrade infrablockspace.");
			let weight = Executive::try_runtime_upgrade(checks).unwrap();
			(weight, BlockWeights::get().max_block)
		}

		fn execute_block(
			block: Block,
			state_root_check: bool,
			signature_check: bool,
			select: frame_try_runtime::TryStateSelect,
		) -> Weight {
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here.
			Executive::try_execute_block(block, state_root_check, signature_check, select).unwrap()
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;

			use pallet_session_benchmarking::Pallet as SessionBench;
			use pallet_offences_benchmarking::Pallet as OffencesBench;
			use pallet_election_provider_support_benchmarking::Pallet as ElectionProviderBench;
			use pallet_nomination_pools_benchmarking::Pallet as NominationPoolsBench;
			use frame_system_benchmarking::Pallet as SystemBench;
			use frame_benchmarking::baseline::Pallet as Baseline;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();
			return (list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<
			Vec<frame_benchmarking::BenchmarkBatch>,
			sp_runtime::RuntimeString,
		> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, TrackedStorageKey};
			// Trying to add benchmarks directly to some pallets caused cyclic dependency issues.
			// To get around that, we separated the benchmarks into its own crate.
			use pallet_session_benchmarking::Pallet as SessionBench;
			use pallet_offences_benchmarking::Pallet as OffencesBench;
			use pallet_election_provider_support_benchmarking::Pallet as ElectionProviderBench;
			use pallet_nomination_pools_benchmarking::Pallet as NominationPoolsBench;
			use frame_system_benchmarking::Pallet as SystemBench;
			use frame_benchmarking::baseline::Pallet as Baseline;

			impl pallet_session_benchmarking::Config for Runtime {}
			impl pallet_offences_benchmarking::Config for Runtime {}
			impl pallet_election_provider_support_benchmarking::Config for Runtime {}
			impl frame_system_benchmarking::Config for Runtime {}
			impl frame_benchmarking::baseline::Config for Runtime {}
			impl pallet_nomination_pools_benchmarking::Config for Runtime {}

			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
				// Total Issuance
				hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
				// Execution Phase
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
				// Event Count
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
				// System Events
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
				// Treasury Account
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da95ecffd7b6c0f78751baa9d281e0bfa3a6d6f646c70792f74727372790000000000000000000000000000000000000000").to_vec().into(),
			];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);

			add_benchmarks!(params, batches);

			Ok(batches)
		}
	}

	impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {

		fn build_state(config: Vec<u8>) -> sp_genesis_builder::Result {
			build_state::<RuntimeGenesisConfig>(config)
		}

		fn get_preset(id: &Option<PresetId>) -> Option<Vec<u8>> {
			get_preset::<RuntimeGenesisConfig>(id, &genesis_config_presets::get_preset)
		}

		fn preset_names() -> Vec<PresetId> {
			vec![
				PresetId::from("local_testnet"),
				PresetId::from("development"),
			]
		}
	}
}