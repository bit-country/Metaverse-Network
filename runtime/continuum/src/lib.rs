// This file is part of Metaverse.Network & Bit.Country.

// Copyright (C) 2020-2022 Metaverse.Network & Bit.Country .
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

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

use codec::{Decode, Encode, MaxEncodedLen};
use cumulus_primitives_core::ParaId;
use frame_support::traits::{
	Contains, Currency, EitherOfDiverse, EnsureOneOf, EnsureOrigin, EqualPrivilegeOnly, Get, InstanceFilter, Nothing,
	OnUnbalanced,
};
use frame_support::{
	construct_runtime, match_type, parameter_types,
	traits::{Everything, Imbalance, WithdrawReasons},
	weights::{
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND},
		ConstantMultiplier, DispatchClass, IdentityFee, Weight, WeightToFeeCoefficient, WeightToFeeCoefficients,
		WeightToFeePolynomial,
	},
	BoundedVec, PalletId, RuntimeDebug, WeakBoundedVec,
};
use frame_system::{
	limits::{BlockLength, BlockWeights},
	EnsureRoot, EnsureSigned, RawOrigin,
};
use orml_traits::location::{AbsoluteReserveProvider, RelativeReserveProvider, Reserve};
use orml_traits::{arithmetic::Zero, parameter_type_with_key, MultiCurrency};
pub use orml_xcm_support::{IsNativeConcrete, MultiCurrencyAdapter, MultiNativeAsset};
// XCM Imports
use orml_xcm_support::DepositToAlternative;
// Polkadot Imports
use pallet_xcm::{EnsureXcm, IsMajorityOfBody, XcmPassthrough};
use polkadot_parachain::primitives::Sibling;
use polkadot_runtime_common::{BlockHashCount, SlowAdjustingFeeUpdate};
use scale_info::prelude::vec;
use scale_info::TypeInfo;
use smallvec::smallvec;
use sp_api::impl_runtime_apis;
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::KeyTypeId, ConstBool, OpaqueMetadata};
use sp_runtime::traits::{AccountIdConversion, ConstU32, Convert, ConvertInto};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, IdentifyAccount, Verify},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, MultiSignature,
};
pub use sp_runtime::{MultiAddress, Perbill, Percent, Permill};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use xcm::latest::prelude::*;
use xcm_builder::{
	AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom, AllowTopLevelPaidExecutionFrom,
	AllowUnpaidExecutionFrom, CurrencyAdapter, EnsureXcmOrigin, FixedRateOfFungible, FixedWeightBounds, IsConcrete,
	NativeAsset, ParentAsSuperuser, ParentIsPreset, RelayChainAsNative, SiblingParachainAsNative,
	SiblingParachainConvertsVia, SignedAccountId32AsNative, SignedToAccountId32, SovereignSignedViaLocation,
	TakeRevenue, TakeWeightCredit, UsingComponents,
};
use xcm_executor::{traits::WithOriginFilter, Config, XcmExecutor};

use asset_manager::{BuyWeightRateOfForeignAsset, ForeignAssetMapping};
pub use constants::{currency::*, time::*};
use core_primitives::{NftAssetData, NftClassData};
// External imports
use currencies::BasicCurrencyAdapter;
use metaverse_runtime_common::{CurrencyHooks, FixedRateOfAsset};
use primitives::{Amount, ClassId, ForeignAssetIdMapping, FungibleTokenId, Moment, NftId, RoundIndex, TokenSymbol};

// XCM Imports
use crate::constants::parachains;
use crate::constants::xcm_fees::{ksm_per_second, native_per_second};

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

mod weights;

/// Constant values used within the runtime.
pub mod constants;

/// Base storage fee
pub const BASE_STORAGE_FEE: Balance = 1 * DOLLARS;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// An index to a block.
pub type BlockNumber = u32;

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, ()>;

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;

/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;

/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra>;

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
	OnRuntimeUpgrade,
>;

pub struct OnRuntimeUpgrade;

impl frame_support::traits::OnRuntimeUpgrade for OnRuntimeUpgrade {
	fn on_runtime_upgrade() -> Weight {
		frame_support::migrations::migrate_from_pallet_version_to_storage_version::<AllPalletsWithSystem>(
			&RocksDbWeight::get(),
		)
	}
}

/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
/// node's balance type.
///
/// This should typically create a mapping between the following ranges:
///   - `[0, MAXIMUM_BLOCK_WEIGHT]`
///   - `[Balance::min, Balance::max]`
///
/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
///   - Setting it to `0` will essentially disable the weight fee.
///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
pub struct WeightToFee;

impl WeightToFeePolynomial for WeightToFee {
	type Balance = Balance;
	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		// in Rococo, extrinsic base weight (smallest non-zero weight) is mapped to 1 CENTS:
		// in our template, we map to 1/10 of that, or 1/10 MILLIUNIT
		let p = RELAY_CENTS;
		let q = Balance::from(ExtrinsicBaseWeight::get().ref_time());
		smallvec![WeightToFeeCoefficient {
			degree: 1,
			negative: false,
			coeff_frac: Perbill::from_rational(p % q, q),
			coeff_integer: p / q,
		}]
	}
}

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
	use sp_runtime::{generic, traits::BlakeTwo256};

	use super::*;

	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;
}

impl_opaque_keys! {
	pub struct SessionKeys {
		pub aura: Aura,
	}
}

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("continuum-runtime"),
	impl_name: create_runtime_str!("continuum-runtime"),
	authoring_version: 1,
	spec_version: 8,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 0,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

// Filter call that we don't enable before governance launch
// Allow base system calls needed for block production and runtime upgrade
// Other calls will be disallowed
pub struct NormalCallFilter;

impl Contains<RuntimeCall> for NormalCallFilter {
	fn contains(c: &RuntimeCall) -> bool {
		let is_core = matches!(
			c,
			// Calls from Sudo
			RuntimeCall::Sudo(..)
			// Calls for runtime upgrade.
			| RuntimeCall::System(..)
			| RuntimeCall::Timestamp(..)
			// Calls that are present in each block
			| RuntimeCall::ParachainSystem(..)
			// Enable session
			| RuntimeCall::Session(..)
			// Enable collator selection
			| RuntimeCall::CollatorSelection(..)
			// Enable vesting
			| RuntimeCall::Vesting(..)
			// Enable ultility
			| RuntimeCall::Utility{..}
			// Enable multisign
			| RuntimeCall::Multisig(..)
			// Enable Crowdloan
			| RuntimeCall::Crowdloan{..}
			// Polkadot XCM
			| RuntimeCall::PolkadotXcm{..}
			// Orml XCM wrapper
			| RuntimeCall::OrmlXcm{..}
			| RuntimeCall::Balances(..)
			| RuntimeCall::XTokens(..)
		);

		if is_core {
			return true;
		};

		let is_emergency_stopped = emergency::EmergencyStoppedFilter::<Runtime>::contains(c);

		if is_emergency_stopped {
			// Not allow stopped tx
			return false;
		}
		true
	}
}

/// Maintenance mode RuntimeCall filter
pub struct MaintenanceFilter;

impl Contains<RuntimeCall> for MaintenanceFilter {
	fn contains(c: &RuntimeCall) -> bool {
		match c {
			RuntimeCall::Auction(_) => false,
			RuntimeCall::Balances(_) => false,
			RuntimeCall::Currencies(_) => false,
			RuntimeCall::Crowdloan(_) => false,
			RuntimeCall::Continuum(_) => false,
			RuntimeCall::Economy(_) => false,
			RuntimeCall::Estate(_) => false,
			RuntimeCall::Mining(_) => false,
			RuntimeCall::Metaverse(_) => false,
			RuntimeCall::Nft(_) => false,
			RuntimeCall::OrmlXcm(_) => false,
			RuntimeCall::PolkadotXcm(_) => false,
			RuntimeCall::Treasury(_) => false,
			RuntimeCall::Vesting(_) => false,
			RuntimeCall::XTokens(_) => false,
			_ => true,
		}
	}
}

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;

	// This part is copied from Substrate's `bin/node/runtime/src/lib.rs`.
	//  The `RuntimeBlockLength` and `RuntimeBlockWeights` exist here because the
	// `DeletionWeightLimit` and `DeletionQueueDepth` depend on those to parameterize
	// the lazy contract deletion.
	pub RuntimeBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
	pub const SS58Prefix: u16 = 268;
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type RuntimeCall = RuntimeCall;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = AccountIdLookup<AccountId, ()>;
	/// The index type for storing how many extrinsics an account has signed.
	type Index = Index;
	/// The index type for blocks.
	type BlockNumber = BlockNumber;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The header type.
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	/// The ubiquitous origin type.
	type RuntimeOrigin = RuntimeOrigin;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// Runtime version.
	type Version = Version;
	/// Converts a module to an index of this module in the runtime.
	type PalletInfo = PalletInfo;
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = ();
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = Emergency;
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = ();
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = RuntimeBlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = RuntimeBlockLength;
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
	/// The action to take on a Runtime Upgrade
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const UncleGenerations: u32 = 0;
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type EventHandler = CollatorSelection;
}

parameter_types! {
	pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = MaxLocks;
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type HoldIdentifier = ();
	type FreezeIdentifier = ();
	type MaxHolds = ConstU32<0>;
	type MaxFreezes = ConstU32<0>;
}

type NegativeImbalance = <Balances as Currency<AccountId>>::NegativeImbalance;

pub struct DealWithFees;

impl OnUnbalanced<NegativeImbalance> for DealWithFees {
	fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance>) {
		if let Some(fees) = fees_then_tips.next() {
			// for fees, 50% to treasury, 50% to author
			let mut split = fees.ration(50, 50);
			if let Some(tips) = fees_then_tips.next() {
				// for tips, if any, 80% to treasury, 20% to staking pot (though this can be anything)
				tips.ration_merge_into(50, 50, &mut split);
			}
			Treasury::on_unbalanced(split.0);
			Balances::resolve_creating(&CollatorSelection::account_id(), split.1);
		}
	}
}

parameter_types! {
	/// Relay Chain `TransactionByteFee` / 10
	pub const TransactionByteFee: Balance = 100 * MILLICENTS;
	pub const OperationalFeeMultiplier: u8 = 5;
}

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, DealWithFees>;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type WeightToFee = WeightToFee;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
	type OperationalFeeMultiplier = OperationalFeeMultiplier;
}

impl pallet_sudo::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type WeightInfo = pallet_sudo::weights::SubstrateWeight<Runtime>;
}

// Currencies implementation
// Metaverse network related pallets
parameter_types! {
	pub const MetaverseNetworkTreasuryPalletId: PalletId = PalletId(*b"bit/trsy");
	pub const NftPalletId: PalletId = PalletId(*b"bit/bnft");
	pub const SwapPalletId: PalletId = PalletId(*b"bit/swap");
	pub const BitMiningTreasuryPalletId: PalletId = PalletId(*b"cb/minig");
	pub const LocalMetaverseFundPalletId: PalletId = PalletId(*b"bit/meta");
	pub const CollatorPotPalletId: PalletId = PalletId(*b"bcPotStk");
	pub const EconomyTreasuryPalletId: PalletId = PalletId(*b"bit/econ");
	pub const LandTreasuryPalletId: PalletId = PalletId(*b"bit/land");
}

// Treasury and Bounty
parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: Balance = 1 * DOLLARS;
	pub const ProposalBondMaximum: Balance = 50 * DOLLARS;
	pub const SpendPeriod: BlockNumber = 1 * DAYS;
	pub const Burn: Permill = Permill::from_percent(0); // No burn
	pub const TipCountdown: BlockNumber = 1 * DAYS;
	pub const TipFindersFee: Percent = Percent::from_percent(20);
	pub const TipReportDepositBase: Balance = 1 * DOLLARS;
	pub const DataDepositPerByte: Balance = 1 * CENTS;
	pub const BountyDepositBase: Balance = 1 * DOLLARS;
	pub const BountyDepositPayoutDelay: BlockNumber = 1 * DAYS;
	pub const BountyUpdatePeriod: BlockNumber = 14 * DAYS;
	pub const MaximumReasonLength: u32 = 16384;
	pub const CuratorDepositMultiplier: Permill = Permill::from_percent(50);
	pub CuratorDepositMin: Balance = 1 * DOLLARS;
	pub CuratorDepositMax: Balance = 100 * DOLLARS;
	pub const BountyValueMinimum: Balance = 5 * DOLLARS;
	pub const MaxApprovals: u32 = 100;
}

type CouncilCollective = pallet_collective::Instance1;
type TechnicalCommitteeCollective = pallet_collective::Instance2;

// Council
pub type EnsureRootOrAllCouncilCollective = EitherOfDiverse<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 1>,
>;

type EnsureRootOrHalfCouncilCollective = EitherOfDiverse<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 2>,
>;

type EnsureRootOrTwoThirdsCouncilCollective = EitherOfDiverse<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 2, 3>,
>;

// Technical Committee

pub type EnsureRootOrAllTechnicalCommittee = EitherOfDiverse<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCommitteeCollective, 1, 1>,
>;

type EnsureRootOrHalfTechnicalCommittee = EitherOfDiverse<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCommitteeCollective, 1, 2>,
>;

type EnsureRootOrTwoThirdsTechnicalCommittee = EitherOfDiverse<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCommitteeCollective, 2, 3>,
>;

impl pallet_treasury::Config for Runtime {
	type PalletId = MetaverseNetworkTreasuryPalletId;
	type Currency = Balances;
	type ApproveOrigin = EnsureRootOrTwoThirdsCouncilCollective;
	type RejectOrigin = EnsureRootOrHalfCouncilCollective;
	type RuntimeEvent = RuntimeEvent;
	type OnSlash = Treasury;
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type BurnDestination = ();
	type SpendFunds = Bounties;
	type WeightInfo = ();
	type MaxApprovals = MaxApprovals;
	type SpendOrigin = frame_support::traits::NeverEnsureOrigin<Balance>;
	type ProposalBondMaximum = ProposalBondMaximum;
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
	type DataDepositPerByte = DataDepositPerByte;
	type MaximumReasonLength = MaximumReasonLength;
	type WeightInfo = ();
	type ChildBountyManager = ();
}

parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = 5 * DAYS;
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 10;
	pub MaxProposalWeight: Weight = Perbill::from_percent(50) * RuntimeBlockWeights::get().max_block;
}

impl pallet_collective::Config<CouncilCollective> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
	type SetMembersOrigin = EnsureRoot<Self::AccountId>;
	type MaxProposalWeight = MaxProposalWeight;
}

parameter_types! {
	pub const TechnicalCommitteeMotionDuration: BlockNumber = 5 * DAYS;
	pub const TechnicalCommitteeMaxProposals: u32 = 100;
	pub const TechnicalCouncilMaxMembers: u32 = 10;
}

impl pallet_collective::Config<TechnicalCommitteeCollective> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = TechnicalCommitteeMotionDuration;
	type MaxProposals = TechnicalCommitteeMaxProposals;
	type MaxMembers = TechnicalCouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
	type SetMembersOrigin = EnsureRoot<Self::AccountId>;
	type MaxProposalWeight = MaxProposalWeight;
}

parameter_types! {
	pub const LaunchPeriod: BlockNumber = 7 * DAYS;
	pub const VotingPeriod: BlockNumber = 7 * DAYS;
	pub const FastTrackVotingPeriod: BlockNumber = 2 * HOURS;
	pub const InstantAllowed: bool = true;
	pub const MinimumDeposit: Balance = 10000 * DOLLARS;
	pub const EnactmentPeriod: BlockNumber = 7 * DAYS;
	pub const CooloffPeriod: BlockNumber = 7 * MINUTES;
	pub const MaxVotes: u32 = 100;
	pub const MaxProposals: u32 = 50;
	pub const MaxBlacklisted: u32 = 100;
	pub const MaxDeposits: u32 = 100;
}

impl pallet_democracy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type EnactmentPeriod = EnactmentPeriod;
	type LaunchPeriod = LaunchPeriod;
	type VotingPeriod = VotingPeriod;
	type VoteLockingPeriod = EnactmentPeriod;
	type MinimumDeposit = MinimumDeposit;
	type SubmitOrigin = EnsureSigned<AccountId>;
	/// A straight majority of the council can decide what their next motion is.
	type ExternalOrigin = EnsureRootOrHalfCouncilCollective;
	/// A super-majority can have the next scheduled referendum be a straight majority-carries vote.
	type ExternalMajorityOrigin = EnsureRootOrTwoThirdsCouncilCollective;
	/// A unanimous council can have the next scheduled referendum be a straight default-carries
	/// (NTB) vote.
	type ExternalDefaultOrigin = EnsureRootOrAllCouncilCollective;
	//type SubmitOrigin = EnsureSigned<AccountId>;
	/// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
	/// be tabled immediately and with a shorter voting/enactment period.
	type FastTrackOrigin = EnsureRootOrTwoThirdsTechnicalCommittee;
	type InstantOrigin = EnsureRootOrAllTechnicalCommittee;
	type InstantAllowed = InstantAllowed;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	/// To cancel a proposal which has been passed, 2/3 of the council must agree to it.
	type CancellationOrigin = EnsureRootOrTwoThirdsCouncilCollective;
	/// To cancel a proposal before it has been passed, the technical committee must be unanimous or
	/// Root must agree.
	type CancelProposalOrigin = EnsureRootOrAllTechnicalCommittee;
	type BlacklistOrigin = EnsureRoot<AccountId>;
	/// Any single technical committee member may veto a coming council proposal, however they can
	/// only do it once and it lasts only for the cooloff period.
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, CouncilCollective>;
	type CooloffPeriod = CooloffPeriod;
	type Slash = ();
	type Scheduler = Scheduler;
	type PalletsOrigin = OriginCaller;
	type MaxVotes = MaxVotes;
	type WeightInfo = pallet_democracy::weights::SubstrateWeight<Runtime>;
	type MaxProposals = MaxProposals;
	type Preimages = Preimage;
	type MaxDeposits = MaxDeposits;
	type MaxBlacklisted = MaxBlacklisted;
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: FungibleTokenId| -> Balance {
		EXISTENTIAL_DEPOSIT
	};
}

parameter_types! {
	pub TreasuryModuleAccount: AccountId = MetaverseNetworkTreasuryPalletId::get().into_account_truncating();
}

impl orml_tokens::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = FungibleTokenId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type CurrencyHooks = CurrencyHooks<Runtime, TreasuryModuleAccount>;
	type MaxLocks = MaxLocks;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type DustRemovalWhitelist = Nothing;
}

parameter_types! {
	pub const BaseXcmWeight: Weight = Weight::from_ref_time(100_000_000);
	pub const MaxAssetsForTransfer: usize = 2;
	pub SelfLocation: MultiLocation = MultiLocation::new(1, X1(Parachain(ParachainInfo::get().into())));
	pub UniversalLocation: InteriorMultiLocation = X2(GlobalConsensus(RelayNetwork::get()), Parachain(ParachainInfo::parachain_id().into()));
}

pub struct AccountIdToMultiLocation;

impl Convert<AccountId, MultiLocation> for AccountIdToMultiLocation {
	fn convert(account: AccountId) -> MultiLocation {
		X1(AccountId32 {
			network: None,
			id: account.into(),
		})
		.into()
	}
}

pub struct MultiLocationsFilter;

impl Contains<MultiLocation> for MultiLocationsFilter {
	fn contains(m: &MultiLocation) -> bool {
		true
	}
}

parameter_type_with_key! {
	pub ParachainMinFee: |_location: MultiLocation| -> Option<u128> {
		Some(u128::MAX)
	};
}

impl orml_xtokens::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type CurrencyId = FungibleTokenId;
	type CurrencyIdConvert = FungibleTokenIdConvert;
	type AccountIdToMultiLocation = AccountIdToMultiLocation;
	type SelfLocation = SelfLocation;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type BaseXcmWeight = BaseXcmWeight;
	type UniversalLocation = UniversalLocation;
	type MaxAssetsForTransfer = MaxAssetsForTransfer;
	type MinXcmFee = ParachainMinFee;
	type MultiLocationsFilter = MultiLocationsFilter;
	type ReserveProvider = AbsoluteReserveProvider;
}

impl orml_unknown_tokens::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
}

impl orml_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SovereignOrigin = EnsureRoot<AccountId>;
}

parameter_types! {
	pub const GetNativeCurrencyId: FungibleTokenId = FungibleTokenId::NativeToken(0);
}

impl currencies::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MultiSocialCurrency = Tokens;
	type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type WeightInfo = weights::module_currencies::WeightInfo<Runtime>;
}

parameter_types! {
	pub const ReservedXcmpWeight: Weight =  Weight::from_ref_time(MAXIMUM_BLOCK_WEIGHT.ref_time() / 4);
	pub const ReservedDmpWeight: Weight =  Weight::from_ref_time(MAXIMUM_BLOCK_WEIGHT.ref_time() / 4);
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SelfParaId = parachain_info::Pallet<Runtime>;
	type DmpMessageHandler = DmpQueue;
	type ReservedDmpWeight = ReservedDmpWeight;
	type OutboundXcmpMessageSource = XcmpQueue;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type CheckAssociatedRelayNumber = cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
	type OnSystemEvent = ();
}

impl pallet_insecure_randomness_collective_flip::Config for Runtime {}

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_aura_ext::Config for Runtime {}

parameter_types! {
	pub const RocLocation: MultiLocation = MultiLocation::parent();
	pub const RelayNetwork: NetworkId = NetworkId::Kusama;
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
	pub SelfParaChainId: cumulus_primitives_core::ParaId = ParachainInfo::parachain_id();
}

parameter_types! {
	pub KsmPerSecond: (AssetId, u128, u128) = (MultiLocation::parent().into(), ksm_per_second(), 0);
	pub GenericNeerPerSecond: (AssetId, u128, u128) = (
		MultiLocation::new(
			1,
			X2(Parachain(ParachainInfo::parachain_id().into()), Junction::from(BoundedVec::try_from(FungibleTokenId::NativeToken(0).encode()).unwrap()))
		).into(),
		native_per_second(),
		0
	);
	pub NeerPerSecond: (AssetId, u128, u128) = (
		MultiLocation::new(
			1,
			X2(Parachain(2096), Junction::from(BoundedVec::try_from(FungibleTokenId::NativeToken(0).encode()).unwrap()))
		).into(),
		native_per_second(),
		0
	);
	pub NeerX0PerSecond: (AssetId, u128, u128) = (
		MultiLocation::new(
			0,
			X1(Junction::from(BoundedVec::try_from(FungibleTokenId::NativeToken(0).encode()).unwrap()))
		).into(),
		native_per_second(),
		0
	);
	pub NeerX1PerSecond: (AssetId, u128, u128) = (
		MultiLocation::new(
			1,
			X2(Parachain(3096), Junction::from(BoundedVec::try_from(FungibleTokenId::NativeToken(0).encode()).unwrap())),
		).into(),
		native_per_second(),
		0
	);
	pub NeerX2PerSecond: (AssetId, u128, u128) = (
		MultiLocation::new(
			1,
			X2(Parachain(parachains::karura::ID), Junction::from(BoundedVec::try_from(FungibleTokenId::NativeToken(0).encode()).unwrap())),
		).into(),
		native_per_second(),
		0
	);
	pub BitPerSecond: (AssetId, u128, u128) = (
		MultiLocation::new(
			1,
			X2(Parachain(2096), Junction::from(BoundedVec::try_from(FungibleTokenId::MiningResource(0).encode()).unwrap())),
		).into(),
		native_per_second(),
		0
	);
	pub BitX1PerSecond: (AssetId, u128, u128) = (
		MultiLocation::new(
			0,
			X1(Junction::from(BoundedVec::try_from(FungibleTokenId::MiningResource(0).encode()).unwrap())),
		).into(),
		native_per_second(),
		0
	);
	pub KUsdPerSecond: (AssetId, u128, u128) = (
		MultiLocation::new(
			1,
			X2(Parachain(parachains::karura::ID), Junction::from(BoundedVec::try_from(parachains::karura::KUSD_KEY.to_vec()).unwrap()))
		).into(),
		// kUSD:KSM = 400:1
		ksm_per_second() * 400,
		0
	);
	pub KarPerSecond: (AssetId, u128, u128) = (
		MultiLocation::new(
			1,
			X2(Parachain(parachains::karura::ID), Junction::from(BoundedVec::try_from(parachains::karura::KAR_KEY.to_vec()).unwrap()))
		).into(),
		// KAR:KSM = 50:1
		ksm_per_second() * 50,
		0
	);

	pub BaseRate: u128 = native_per_second();
}

pub struct ToTreasury;

impl TakeRevenue for ToTreasury {
	fn take_revenue(revenue: MultiAsset) {
		if let MultiAsset {
			id: Concrete(location),
			fun: Fungible(amount),
		} = revenue
		{
			if let Some(currency_id) = FungibleTokenIdConvert::convert(location) {
				let _ = Currencies::deposit(currency_id, &TreasuryModuleAccount::get(), amount);
			}
		}
	}
}

/// Trader - The means of purchasing weight credit for XCM execution.
/// We need to ensure we have at least one rule per token we want to handle or else
/// the xcm executor won't know how to charge fees for a transfer of said token.
pub type Trader = (
	FixedRateOfFungible<KsmPerSecond, ToTreasury>,
	FixedRateOfFungible<GenericNeerPerSecond, ToTreasury>,
	FixedRateOfAsset<BaseRate, ToTreasury, BuyWeightRateOfForeignAsset<Runtime>>,
	FixedRateOfFungible<NeerPerSecond, ToTreasury>,
	FixedRateOfFungible<NeerX0PerSecond, ToTreasury>,
	FixedRateOfFungible<NeerX1PerSecond, ToTreasury>,
	FixedRateOfFungible<NeerX2PerSecond, ToTreasury>,
	FixedRateOfFungible<BitPerSecond, ToTreasury>,
	FixedRateOfFungible<BitX1PerSecond, ToTreasury>,
	FixedRateOfFungible<KarPerSecond, ToTreasury>,
	FixedRateOfFungible<KUsdPerSecond, ToTreasury>,
);

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the default `AccountId`.
	ParentIsPreset<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountId>,
	// Straight up local `AccountId32` origins just alias directly to `AccountId`.
	AccountId32Aliases<RelayNetwork, AccountId>,
);

/// Means for transacting assets on this chain.
pub type LocalAssetTransactor = MultiCurrencyAdapter<
	// Use this currency:
	Currencies,
	// Tokens,
	UnknownTokens,
	// Use this currency when it is a fungible asset matching the given location or name:
	IsNativeConcrete<FungibleTokenId, FungibleTokenIdConvert>,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// Do a simple punn to convert an AccountId32 MultiLocation into a native chain account ID:
	LocationToAccountId,
	// We don't track any teleports.
	FungibleTokenId,
	FungibleTokenIdConvert,
	DepositToAlternative<TreasuryModuleAccount, Currencies, FungibleTokenId, AccountId, Balance>,
>;

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
	// Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
	// recognised.
	RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
	// Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
	// recognised.
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
	// Superuser converter for the Relay-chain (Parent) location. This will allow it to issue a
	// transaction from the Root origin.
	ParentAsSuperuser<RuntimeOrigin>,
	// Native signed account converter; this just converts an `AccountId32` origin into a normal
	// `Origin::Signed` origin of the same 32-byte value.
	SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	XcmPassthrough<RuntimeOrigin>,
);

parameter_types! {
	// One XCM operation is 100_000_000 weight - almost certainly a conservative estimate.
	pub UnitWeightCost: Weight = Weight::from_ref_time(100_000_000);
	pub const MaxInstructions: u32 = 100;
}

match_type! {
	pub type ParentOrParentsExecutivePlurality: impl Contains<MultiLocation> = {
		MultiLocation { parents: 1, interior: Here } |
		MultiLocation { parents: 1, interior: X1(Plurality { id: BodyId::Executive, .. }) }
	};
}

fn native_currency_location(id: FungibleTokenId) -> MultiLocation {
	MultiLocation::new(
		1,
		X2(
			Parachain(ParachainInfo::parachain_id().into()),
			Junction::from(BoundedVec::try_from(id.encode()).unwrap()),
		),
	)
}

/// **************************************
// Below is for the network of Kusama.
/// **************************************
pub struct FungibleTokenIdConvert;

impl Convert<FungibleTokenId, Option<MultiLocation>> for FungibleTokenIdConvert {
	fn convert(id: FungibleTokenId) -> Option<MultiLocation> {
		use FungibleTokenId::{FungibleToken, MiningResource, NativeToken, Stable};
		match id {
			// KSM
			NativeToken(1) => Some(MultiLocation::parent()),
			// Karura currencyId types
			NativeToken(2) => Some(MultiLocation::new(
				1,
				X2(
					Parachain(parachains::karura::ID),
					Junction::from(BoundedVec::try_from(parachains::karura::KAR_KEY.to_vec()).unwrap()),
				),
			)),
			Stable(0) => Some(MultiLocation::new(
				1,
				X2(
					Parachain(parachains::karura::ID),
					Junction::from(BoundedVec::try_from(parachains::karura::KUSD_KEY.to_vec()).unwrap()),
				),
			)),
			FungibleToken(token_id) => ForeignAssetMapping::<Runtime>::get_multi_location(token_id),
			_ => Some(native_currency_location(id)),
		}
	}
}

impl Convert<MultiLocation, Option<FungibleTokenId>> for FungibleTokenIdConvert {
	fn convert(location: MultiLocation) -> Option<FungibleTokenId> {
		use FungibleTokenId::{FungibleToken, MiningResource, NativeToken, Stable};

		// NativeToken
		// 0 => NUUM
		// 1 => DOT

		// Stable
		// 0 => KUSD

		// Build mining material
		// Mining resource
		// 0 => BIT

		if location == MultiLocation::parent() {
			return Some(NativeToken(1));
		}

		if let Some(currency_id) = ForeignAssetMapping::<Runtime>::get_currency_id(location.clone()) {
			return Some(currency_id);
		}

		match location.clone() {
			MultiLocation {
				parents: 1,
				interior: X2(Parachain(para_id), GeneralKey { length, data }),
			} => match para_id {
				// Local testing para chain id
				3446 | 4446 => match FungibleTokenId::decode(&mut &data[..]) {
					Ok(NativeToken(0)) => Some(FungibleTokenId::NativeToken(0)),
					Ok(MiningResource(0)) => Some(FungibleTokenId::MiningResource(0)),
					_ => None,
				},

				_ => None,
			},
			MultiLocation { parents, interior } if parents == 0 => match interior {
				X1(GeneralKey { length, data }) => {
					// decode the general key
					if let Ok(currency_id) = FungibleTokenId::decode(&mut &data[..]) {
						match currency_id {
							NativeToken(0) | MiningResource(0) => Some(currency_id),
							_ => None,
						}
					} else {
						None
					}
				}
				_ => None,
			},
			_ => None,
		}
	}
}

impl Convert<MultiAsset, Option<FungibleTokenId>> for FungibleTokenIdConvert {
	fn convert(asset: MultiAsset) -> Option<FungibleTokenId> {
		if let MultiAsset {
			id: Concrete(location), ..
		} = asset
		{
			Self::convert(location)
		} else {
			None
		}
	}
}

pub type Barrier = (TakeWeightCredit, AllowTopLevelPaidExecutionFrom<Everything>);

// A call filter for the XCM Transact instruction. This is a temporary measure until we properly
/// account for proof size weights.
///
/// Calls that are allowed through this filter must:
/// 1. Have a fixed weight;
/// 2. Cannot lead to another call being made
/// 3. Have a defined proof size weight, e.g. no unbounded vecs in call parameters. - TODO:
/// shouldn't max XCM weight handle this?
pub struct SafeCallFilter;

impl SafeCallFilter {
	// 1. RuntimeCall::Multisig(..) - contains `Vec` in argument so we should avoid this
	// 2. RuntimeCall::EVM(..) & RuntimeCall::Ethereum(..) have to be prohibited since we cannot measure
	// PoV size properly 3. RuntimeCall::Contracts(..) it should be safe to allow for such calls but
	// perhaps it's better to do more delibrate testing on Shibuya/RocStar.

	/// Checks whether the base (non-composite) call is allowed to be executed via `Transact` XCM
	/// instruction.
	pub fn allow_base_call(call: &RuntimeCall) -> bool {
		match call {
			RuntimeCall::System(..)
			| RuntimeCall::Balances(..)
			| RuntimeCall::Vesting(..)
			| RuntimeCall::Currencies(..)
			| RuntimeCall::PolkadotXcm(..)
			| RuntimeCall::Session(..) => true,
			_ => false,
		}
	}
}

impl Contains<RuntimeCall> for SafeCallFilter {
	fn contains(call: &RuntimeCall) -> bool {
		#[cfg(feature = "runtime-benchmarks")]
		{
			if matches!(call, RuntimeCall::System(frame_system::Call::remark_with_event { .. })) {
				return true;
			}
		}

		Self::allow_base_call(call)
	}
}

pub struct XcmConfig;

impl xcm_executor::Config for XcmConfig {
	type AssetTrap = PolkadotXcm;
	type AssetClaims = PolkadotXcm;
	// How to withdraw and deposit an asset.
	type AssetTransactor = LocalAssetTransactor;
	type Barrier = Barrier;
	type RuntimeCall = RuntimeCall;
	type XcmSender = XcmRouter;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	type IsReserve = MultiNativeAsset<AbsoluteReserveProvider>;
	type IsTeleporter = ();
	// Should be enough to allow teleportation of ROC
	type UniversalLocation = UniversalLocation;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type Trader = Trader;
	type ResponseHandler = PolkadotXcm;
	type SubscriptionService = PolkadotXcm;

	type PalletInstancesInfo = AllPalletsWithSystem;
	type MaxAssetsIntoHolding = ConstU32<64>;
	type AssetLocker = ();
	type AssetExchanger = ();
	type FeeManager = ();
	type MessageExporter = ();
	type UniversalAliases = Nothing;
	type CallDispatcher = WithOriginFilter<SafeCallFilter>;
	type SafeCallFilter = SafeCallFilter;
}

parameter_types! {
	pub const MaxDownwardMessageWeight: Weight =  Weight::from_ref_time(MAXIMUM_BLOCK_WEIGHT.ref_time() / 10);
	pub const RelayNetworkId: NetworkId = NetworkId::Kusama;
}

/// No local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetworkId>;

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm, ()>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
);

#[cfg(feature = "runtime-benchmarks")]
parameter_types! {
	pub ReachableDest: Option<MultiLocation> = Some(Parent.into());
}

impl pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmExecuteFilter = Nothing;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = Nothing;
	type XcmReserveTransferFilter = Everything;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type UniversalLocation = UniversalLocation;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;

	type Currency = Balances;
	type CurrencyMatcher = ();
	type TrustedLockers = ();
	type SovereignAccountOf = ();
	type MaxLockers = ConstU32<8>;
	type WeightInfo = pallet_xcm::TestWeightInfo;
	type MaxRemoteLockConsumers = ConstU32<0>;
	type RemoteLockConsumerIdentifier = ();
	#[cfg(feature = "runtime-benchmarks")]
	type ReachableDest = ReachableDest;
	type AdminOrigin = EnsureRoot<AccountId>;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = PolkadotXcm;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	type PriceForSiblingDelivery = ();
	type WeightInfo = ();
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}

parameter_types! {
	pub const Period: u32 = DAYS;
	pub const Offset: u32 = 0;
	pub const MaxAuthorities: u32 = 100_000;
	pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(33);
}

impl pallet_session::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	// we don't have stash and controller, thus we don't need the convert as well.
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
	type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
	type SessionManager = CollatorSelection;
	// Essentially just Aura, but lets be pedantic.
	type SessionHandler = <SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	type WeightInfo = ();
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = MaxAuthorities;
}

parameter_types! {
	pub const MaxCandidates: u32 = 10;
	pub const MinCandidates: u32 = 5;
	pub const MaxInvulnerables: u32 = 100;
	pub const ExecutiveBody: BodyId = BodyId::Executive;
}

// We allow root and the Relay Chain council to execute privileged collator selection operations.
pub type CollatorSelectionUpdateOrigin =
	EnsureOneOf<EnsureRoot<AccountId>, EnsureXcm<IsMajorityOfBody<RocLocation, ExecutiveBody>>>;

impl pallet_collator_selection::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type UpdateOrigin = CollatorSelectionUpdateOrigin;
	type PotId = CollatorPotPalletId;
	type MaxCandidates = MaxCandidates;
	type MinCandidates = MinCandidates;
	type MaxInvulnerables = MaxInvulnerables;
	// should be a multiple of session or things will get inconsistent
	type KickThreshold = Period;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ValidatorRegistration = Session;
	type WeightInfo = ();
}

impl pallet_utility::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
}

parameter_types! {
	// One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
	pub const DepositBase: Balance = deposit(1, 88);
	// Additional storage item size of 32 bytes.
	pub const DepositFactor: Balance = deposit(0, 32);
	pub const MaxSignatories: u16 = 100;
}

impl pallet_multisig::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type DepositBase = DepositBase;
	type DepositFactor = DepositFactor;
	type MaxSignatories = MaxSignatories;
	type WeightInfo = ();
}

// Metaverse related implementation
pub struct EnsureRootOrMetaverseTreasury;

impl EnsureOrigin<RuntimeOrigin> for EnsureRootOrMetaverseTreasury {
	type Success = AccountId;

	fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
		Into::<Result<RawOrigin<AccountId>, RuntimeOrigin>>::into(o).and_then(|o| match o {
			RawOrigin::Root => Ok(MetaverseNetworkTreasuryPalletId::get().into_account_truncating()),
			RawOrigin::Signed(caller) => {
				if caller == MetaverseNetworkTreasuryPalletId::get().into_account_truncating() {
					Ok(caller)
				} else {
					Err(RuntimeOrigin::from(Some(caller)))
				}
			}
			r => Err(RuntimeOrigin::from(r)),
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<RuntimeOrigin, ()> {
		Ok(RuntimeOrigin::from(RawOrigin::Root))
	}
}

parameter_types! {
	pub PreimageBaseDeposit: Balance = deposit(2, 64);
	pub PreimageByteDeposit: Balance = 10 * CENTS;
}

impl pallet_preimage::Config for Runtime {
	type WeightInfo = ();
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type ManagerOrigin = EnsureRoot<AccountId>;
	type BaseDeposit = PreimageBaseDeposit;
	type ByteDeposit = PreimageByteDeposit;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
		RuntimeBlockWeights::get().max_block;
	pub const MaxScheduledPerBlock: u32 = 50;
}

impl pallet_scheduler::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type MaxScheduledPerBlock = MaxScheduledPerBlock;
	type WeightInfo = ();
	type Preimages = Preimage;
}

parameter_types! {
	pub const MinVestedTransfer: Balance = 10 * DOLLARS;
	pub UnvestedFundsAllowedWithdrawReasons: WithdrawReasons =
		WithdrawReasons::except(WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE);
}

impl pallet_vesting::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type BlockNumberToBalance = ConvertInto;
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = pallet_vesting::weights::SubstrateWeight<Runtime>;
	type UnvestedFundsAllowedWithdrawReasons = UnvestedFundsAllowedWithdrawReasons;
	const MAX_VESTING_SCHEDULES: u32 = 100;
}

parameter_types! {
	//Mining Resource Currency Id
	pub const MiningResourceCurrencyId: FungibleTokenId = FungibleTokenId::MiningResource(0);
	pub const TreasuryStakingReward: Perbill = Perbill::from_percent(1);
	pub MiningStorageDeposit: Balance = BASE_STORAGE_FEE;
}

impl mining::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MiningCurrency = Currencies;
	type BitMiningTreasury = BitMiningTreasuryPalletId;
	type BitMiningResourceId = MiningResourceCurrencyId;
	type EstateHandler = Estate;
	type AdminOrigin = EnsureRootOrMetaverseTreasury;
	type MetaverseStakingHandler = Metaverse;
	type TreasuryStakingReward = TreasuryStakingReward;
	type NetworkTreasuryAccount = TreasuryModuleAccount;
	type StorageDepositFee = MiningStorageDeposit;
	type Currency = Balances;
	type WeightInfo = weights::module_mining::WeightInfo<Runtime>;
}

parameter_types! {
	pub AssetMintingFee: Balance = 1 * DOLLARS;
	pub ClassMintingFee: Balance = 10 * DOLLARS;
	pub MaxBatchTransfer: u32 = 100;
	pub MaxBatchMinting: u32 = 1000;
	pub MaxNftMetadata: u32 = 1024;
	pub StorageDepositFee: Balance =  BASE_STORAGE_FEE;
}

impl nft::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type Treasury = MetaverseNetworkTreasuryPalletId;
	type MultiCurrency = Currencies;
	type WeightInfo = weights::module_nft::WeightInfo<Runtime>;
	type PalletId = NftPalletId;
	type AuctionHandler = Auction;
	type MaxBatchTransfer = MaxBatchTransfer;
	type MaxBatchMinting = MaxBatchMinting;
	type MaxMetadata = MaxNftMetadata;
	type MiningResourceId = MiningResourceCurrencyId;
	type AssetMintingFee = AssetMintingFee;
	type ClassMintingFee = ClassMintingFee;
	type StorageDepositFee = StorageDepositFee;
	type OffchainSignature = Signature;
	type OffchainPublic = <Signature as Verify>::Signer;
}

parameter_types! {
	pub MaxClassMetadata: u32 = 1024;
	pub MaxTokenMetadata: u32 = 1024;
}

impl orml_nft::Config for Runtime {
	type ClassId = ClassId;
	type TokenId = NftId;
	type Currency = Balances;
	type ClassData = NftClassData<Balance>;
	type TokenData = NftAssetData<Balance>;
	type MaxClassMetadata = MaxClassMetadata;
	type MaxTokenMetadata = MaxTokenMetadata;
}

parameter_types! {
	pub MaxMetaverseMetadata: u32 = 1024;
	pub MinContribution: Balance = 50 * DOLLARS;
	pub MaxNumberOfStakersPerMetaverse: u32 = 512;
	pub MetaverseStorageFee: Balance = 2 * BASE_STORAGE_FEE;
}

impl metaverse::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type NetworkTreasury = TreasuryModuleAccount;
	type MetaverseTreasury = LocalMetaverseFundPalletId;
	type Currency = Balances;
	type MaxMetaverseMetadata = MaxMetaverseMetadata;
	type MinContribution = MinContribution;
	type MetaverseCouncil = EnsureRootOrHalfCouncilCollective;
	type WeightInfo = weights::module_metaverse::WeightInfo<Runtime>;
	type MetaverseRegistrationDeposit = MinContribution;
	type MinStakingAmount = MinContribution;
	type MaxNumberOfStakersPerMetaverse = MaxNumberOfStakersPerMetaverse;
	type MultiCurrency = Currencies;
	type NFTHandler = Nft;
	type StorageDepositFee = MetaverseStorageFee;
}

parameter_types! {
	pub const MinimumLandPrice: Balance = 10 * DOLLARS;
	pub const MinBlocksPerLandIssuanceRound: u32 = 20;
	pub const MinimumStake: Balance = 100 * DOLLARS;
	pub const MaximumEstateStake: Balance = 1000 * DOLLARS;
	pub const RewardPaymentDelay: u32 = 2;
	pub const DefaultMaxBound: (i32,i32) = (-1000,1000);
	pub const NetworkFee: Balance = 10 * DOLLARS; // Network fee
	pub const MaxOffersPerEstate: u32 = 100;
	pub const MinLeasePricePerBlock: Balance = 1 * CENTS;
	pub const MaxLeasePeriod: u32 = 1000000;
	pub const LeaseOfferExpiryPeriod: u32 = 10000;
	pub const EstateStorageFee: Balance = BASE_STORAGE_FEE;
}

impl estate::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type LandTreasury = LandTreasuryPalletId;
	type MetaverseInfoSource = Metaverse;
	type Currency = Balances;
	type MinimumLandPrice = MinimumLandPrice;
	type CouncilOrigin = EnsureRoot<AccountId>;
	type AuctionHandler = Auction;
	type MinBlocksPerRound = MinBlocksPerLandIssuanceRound;
	type WeightInfo = weights::module_estate::WeightInfo<Runtime>;
	type MinimumStake = MinimumStake;
	type RewardPaymentDelay = RewardPaymentDelay;
	type NFTTokenizationSource = Nft;
	type DefaultMaxBound = DefaultMaxBound;
	type NetworkFee = NetworkFee;
	type MaxOffersPerEstate = MaxOffersPerEstate;
	type MinLeasePricePerBlock = MinLeasePricePerBlock;
	type MaxLeasePeriod = MaxLeasePeriod;
	type LeaseOfferExpiryPeriod = LeaseOfferExpiryPeriod;
	type BlockNumberToBalance = ConvertInto;
	type StorageDepositFee = EstateStorageFee;
}

parameter_types! {
	pub const AuctionTimeToClose: u32 = 100; // Default 100800 Blocks
	pub const ContinuumSessionDuration: BlockNumber = 43200; // Default 43200 Blocks
	pub const SpotAuctionChillingDuration: BlockNumber = 43200; // Default 43200 Blocks
	pub const MinimumAuctionDuration: BlockNumber = 300; // Minimum duration is 300 blocks
	pub const MaxFinality: u32 = 200; // Maximum finalize auctions per block
	pub const MaxBundleItem: u32 = 100; // Maximum number of item per bundle
	pub const NetworkFeeReserve: Balance = 1 * DOLLARS; // Network fee reserved when item is listed for auction
	pub const NetworkFeeCommission: Perbill = Perbill::from_percent(1); // Network fee collected after an auction is over
	pub const OfferDuration: BlockNumber = 100800; // Default 100800 Blocks
	pub const MinimumListingPrice: Balance = DOLLARS;
	pub const AntiSnipeDuration: BlockNumber = 50; // Minimum anti snipe duration is 50 blocks
	pub const AuctionStorageFee: Balance = 3 * BASE_STORAGE_FEE;
}

impl auction::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AuctionTimeToClose = AuctionTimeToClose;
	type Handler = Auction;
	type Currency = Balances;
	type ContinuumHandler = Continuum;
	type FungibleTokenCurrency = Tokens;
	type MetaverseInfoSource = Metaverse;
	type MinimumAuctionDuration = MinimumAuctionDuration;
	type EstateHandler = Estate;
	type MaxFinality = MaxFinality;
	type MaxBundleItem = MaxBundleItem;
	type NFTHandler = Nft;
	type NetworkFeeReserve = NetworkFeeReserve;
	type NetworkFeeCommission = NetworkFeeCommission;
	type WeightInfo = weights::module_auction::WeightInfo<Runtime>;
	type OfferDuration = OfferDuration;
	type MinimumListingPrice = MinimumListingPrice;
	type AntiSnipeDuration = AntiSnipeDuration;
	type StorageDepositFee = AuctionStorageFee;
}

parameter_types! {
	pub const ContinuumStorageDeposit: Balance = BASE_STORAGE_FEE;
}
impl continuum::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SessionDuration = ContinuumSessionDuration;
	type SpotAuctionChillingDuration = SpotAuctionChillingDuration;
	type EmergencyOrigin = EnsureRoot<AccountId>;
	type AuctionHandler = Auction;
	type AuctionDuration = SpotAuctionChillingDuration;
	type ContinuumTreasury = MetaverseNetworkTreasuryPalletId;
	type Currency = Balances;
	type MetaverseInfoSource = Metaverse;
	type StorageDepositFee = ContinuumStorageDeposit;
	type WeightInfo = weights::module_continuum::WeightInfo<Runtime>;
}

impl crowdloan::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type VestingSchedule = Vesting;
	type BlockNumberToBalance = ConvertInto;
	type WeightInfo = ();
}

parameter_types! {
	pub const MiningCurrencyId: FungibleTokenId = FungibleTokenId::MiningResource(0);
	pub const PowerAmountPerBlock: u32 = 100;
}

impl economy::Config for Runtime {
	type Currency = Balances;
	type EconomyTreasury = EconomyTreasuryPalletId;
	type RuntimeEvent = RuntimeEvent;
	type FungibleTokenCurrency = Currencies;
	type MinimumStake = MinimumStake;
	type MiningCurrencyId = MiningCurrencyId;
	type NFTHandler = Nft;
	type EstateHandler = Estate;
	type RoundHandler = Mining;
	type PowerAmountPerBlock = PowerAmountPerBlock;
	type WeightInfo = weights::module_economy::WeightInfo<Runtime>;
	type MaximumEstateStake = MaximumEstateStake;
}

impl emergency::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type EmergencyOrigin = EnsureRootOrHalfCouncilCollective;
	type NormalCallFilter = NormalCallFilter;
	type MaintenanceCallFilter = MaintenanceFilter;
	type WeightInfo = weights::module_emergency::WeightInfo<Runtime>;
}

parameter_types! {
	pub const MinimumCount: u32 = 5;
	pub const ExpiresIn: Moment = 1000 * 60 * 60 * 24; // 24 hours
	pub RootOperatorAccountId: AccountId = AccountId::from([0xffu8; 32]);
	pub const MaxHasDispatchedSize: u32 = 20;
	pub const OracleMaxMembers: u32 = 50;
	pub const MaxFeedValues: u32 = 10; // max 10 values allowd to feed in one call.
}

pub type OracleMembershipInstance = pallet_membership::Instance1;

impl pallet_membership::Config<OracleMembershipInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AddOrigin = EnsureRootOrHalfCouncilCollective;
	type RemoveOrigin = EnsureRootOrHalfCouncilCollective;
	type SwapOrigin = EnsureRootOrHalfCouncilCollective;
	type ResetOrigin = EnsureRootOrHalfCouncilCollective;
	type PrimeOrigin = EnsureRootOrHalfCouncilCollective;
	type MembershipInitialized = ();
	type MembershipChanged = RewardOracle;
	type MaxMembers = OracleMaxMembers;
	type WeightInfo = ();
}

type MiningRewardDataProvider = orml_oracle::Instance1;

impl orml_oracle::Config<MiningRewardDataProvider> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnNewData = ();
	type CombineData = orml_oracle::DefaultCombineData<Runtime, MinimumCount, ExpiresIn, MiningRewardDataProvider>;
	type Time = Timestamp;
	type OracleKey = RoundIndex;
	type OracleValue = BoundedVec<u8, MaxMetaverseMetadata>;
	type RootOperatorAccountId = RootOperatorAccountId;
	type Members = OracleMembership;
	type MaxHasDispatchedSize = MaxHasDispatchedSize;
	type MaxFeedValues = MaxFeedValues;
	type WeightInfo = ();
}

impl asset_manager::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type RegisterOrigin = EnsureRootOrHalfCouncilCollective;
}

parameter_types! {
	// One storage item; key size 32, value size 8; .
	pub ProxyDepositBase: Balance = deposit(1, 8);
	// Additional storage item size of 33 bytes.
	pub ProxyDepositFactor: Balance = deposit(0, 33);
	pub AnnouncementDepositBase: Balance = deposit(1, 8);
	pub AnnouncementDepositFactor: Balance = deposit(0, 66);
}

/// The type used to represent the kinds of proxying allowed.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum ProxyType {
	Any,
	CancelProxy,
	Governance,
	Auction,
	Economy,
	Nft,
}

impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}

impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			_ if matches!(c, RuntimeCall::Utility(..)) => true,
			ProxyType::Any => true,
			ProxyType::CancelProxy => matches!(c, RuntimeCall::Proxy(pallet_proxy::Call::reject_announcement { .. })),
			ProxyType::Governance => matches!(
				c,
				RuntimeCall::Democracy(..) | RuntimeCall::Council(..) | RuntimeCall::TechnicalCommittee(..)
			),
			ProxyType::Auction => matches!(
				c,
				RuntimeCall::Auction(auction::Call::bid { .. }) | RuntimeCall::Auction(auction::Call::buy_now { .. })
			),
			ProxyType::Economy => matches!(
				c,
				RuntimeCall::Economy(economy::Call::stake { .. }) | RuntimeCall::Economy(economy::Call::unstake { .. })
			),
			ProxyType::Nft => matches!(
				c,
				RuntimeCall::Nft(nft::Call::transfer { .. }) | RuntimeCall::Nft(nft::Call::transfer_batch { .. })
			),
		}
	}

	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(ProxyType::Any, _) => true,
			(_, ProxyType::Any) => false,
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
	type MaxProxies = ConstU32<32>;
	type WeightInfo = ();
	type MaxPending = ConstU32<32>;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
}

parameter_types! {
	pub const CampaignDeposit: Balance = 1000 * DOLLARS;
	pub const MinimumRewardPool: Balance = 1000 * DOLLARS;
	pub const MinimumCampaignCoolingOffPeriod: BlockNumber = 7 * DAYS;
	pub const MinimumCampaignDuration: BlockNumber = 30 * MINUTES;
	pub const MaxSetRewardsListLength: u64 = 200;
	pub const MaxLeafNodes: u32 = 30;
	pub const RewardStorageFee: Balance = BASE_STORAGE_FEE;
}

impl reward::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type FungibleTokenCurrency = Currencies;
	type PalletId = MetaverseNetworkTreasuryPalletId;
	type MiningCurrencyId = MiningResourceCurrencyId;
	type MinimumRewardPool = MinimumRewardPool;
	type CampaignDeposit = CampaignDeposit;
	type MinimumCampaignDuration = MinimumCampaignDuration;
	type MinimumCampaignCoolingOffPeriod = MinimumCampaignCoolingOffPeriod;
	type MaxLeafNodes = MaxLeafNodes;
	type MaxSetRewardsListLength = MaxSetRewardsListLength;
	type AdminOrigin = EnsureRootOrMetaverseTreasury;
	type NFTHandler = Nft;
	type StorageDepositFee = RewardStorageFee;
	type WeightInfo = weights::module_reward::WeightInfo<Runtime>;
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		// System support stuff.
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>} = 0,
		ParachainSystem: cumulus_pallet_parachain_system::{
			Pallet, Call, Config, Storage, Inherent, Event<T>, ValidateUnsigned,
		} = 1,
		RandomnessCollectiveFlip: pallet_insecure_randomness_collective_flip::{Pallet, Storage} = 2,
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 3,
		ParachainInfo: parachain_info::{Pallet, Storage, Config} = 4,
		// Scheduler
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 5,
		Utility: pallet_utility::{Pallet, Call, Event} = 6,
		Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>} = 7,
		Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>} = 8,

		// Monetary.
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 10,
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage, Event<T>} = 11,

		// Sudo
		Sudo: pallet_sudo::{Pallet, Call, Storage, Config<T>, Event<T>} = 12,

		// Currencies
		Currencies: currencies::{ Pallet, Storage, Call, Event<T>} = 13,
		Tokens: orml_tokens::{Pallet, Storage, Event<T>} = 14,

		// Treasury
		Treasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>} = 15,
		Bounties: pallet_bounties::{Pallet, Call, Storage, Event<T>} = 16,


		// Collator support. The order of these 4 are important and shall not change.
		Authorship: pallet_authorship::{Pallet, Storage} = 20,
		CollatorSelection: pallet_collator_selection::{Pallet, Call, Storage, Event<T>, Config<T>} = 21,
		Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 22,
		Aura: pallet_aura::{Pallet, Storage, Config<T>} = 23,
		AuraExt: cumulus_pallet_aura_ext::{Pallet, Storage, Config} = 24,


		// XCM helpers.
		XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>} = 30,
		PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin} = 31,
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin} = 32,
		DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>} = 33,
		XTokens: orml_xtokens::{Pallet, Storage, Call, Event<T>} = 34,
		UnknownTokens: orml_unknown_tokens::{Pallet, Storage, Event} = 35,
		OrmlXcm: orml_xcm::{Pallet, Call, Event<T>} = 36,

		// Governance
		Council: pallet_collective::<Instance1>::{Pallet, Call, Storage ,Origin<T>, Event<T>} = 40,
		TechnicalCommittee: pallet_collective::<Instance2>::{Pallet, Call, Storage ,Origin<T>, Event<T>} = 41,
		Democracy: pallet_democracy::{Pallet, Call, Storage, Event<T>} = 42,

		// Pioneer pallets
		// Metaverse & Related
		Metaverse: metaverse::{Pallet, Call ,Storage, Event<T>} = 50,

		Vesting: pallet_vesting::{Pallet, Call ,Storage, Event<T>} = 53,
		Mining: mining:: {Pallet, Call ,Storage ,Event<T>} = 54,
		Emergency: emergency::{Pallet, Call, Storage, Event<T>} = 55,
		RewardOracle: orml_oracle::<Instance1>::{Pallet, Storage, Call, Event<T>} = 56,
		OracleMembership: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>} = 57,

		OrmlNFT: orml_nft::{Pallet, Storage} = 60,
		Nft: nft::{Call, Pallet, Storage, Event<T>} = 61,
		Auction: auction::{Call, Pallet ,Storage, Event<T>} = 62,

		Continuum: continuum::{Call, Pallet, Storage, Event<T>} = 63,
		Estate: estate::{Call, Pallet, Storage, Event<T>, Config} = 64,
		Economy: economy::{Pallet, Call, Storage, Event<T>} = 65,
		AssetManager: asset_manager::{Pallet, Call, Storage, Event<T>} = 66,
		// Proxy
		Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>} = 67,
		// Reward mechanism
		Reward: reward::{Pallet, Call, Storage ,Event<T>} = 68,
		// Crowdloan
		Crowdloan: crowdloan::{Pallet, Call, Storage, Event<T>} = 70,
	}
);

impl_runtime_apis! {
	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities().into_inner()
		}
	}

	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
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

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
		fn account_nonce(account: AccountId) -> Index {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info(header: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info(header)
		}
	}


	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{list_benchmark, Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;
			use frame_system_benchmarking::Pallet as SystemBench;
			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;

			let mut list = Vec::<BenchmarkList>::new();

			list_benchmark!(list, extra, frame_system, SystemBench::<Runtime>);
			list_benchmark!(list, extra, pallet_balances, Balances);
			list_benchmark!(list, extra, pallet_timestamp, Timestamp);
			list_benchmark!(list, extra, pallet_collator_selection, CollatorSelection);

			let storage_info = AllPalletsWithSystem::storage_info();

			return (list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};

			use frame_system_benchmarking::Pallet as SystemBench;
			impl frame_system_benchmarking::Config for Runtime {}

			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;
			impl cumulus_pallet_session_benchmarking::Config for Runtime {}

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
			];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);

			add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
			add_benchmark!(params, batches, pallet_balances, Balances);
			add_benchmark!(params, batches, pallet_session, SessionBench::<Runtime>);
			add_benchmark!(params, batches, pallet_timestamp, Timestamp);
			add_benchmark!(params, batches, pallet_collator_selection, CollatorSelection);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}
}

struct CheckInherents;

impl cumulus_pallet_parachain_system::CheckInherents<Block> for CheckInherents {
	fn check_inherents(
		block: &Block,
		relay_state_proof: &cumulus_pallet_parachain_system::RelayChainStateProof,
	) -> sp_inherents::CheckInherentsResult {
		let relay_chain_slot = relay_state_proof
			.read_slot()
			.expect("Could not read the relay chain slot from the proof");

		let inherent_data = cumulus_primitives_timestamp::InherentDataProvider::from_relay_chain_slot_and_duration(
			relay_chain_slot,
			sp_std::time::Duration::from_secs(6),
		)
		.create_inherent_data()
		.expect("Could not create the timestamp inherent data");

		inherent_data.check_extrinsics(block)
	}
}

cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
	CheckInherents = CheckInherents,
}
