#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

/// Wasm binary unwrapped. If built with `SKIP_WASM_BUILD`, the function panics.
#[cfg(feature = "std")]
pub fn wasm_binary_unwrap() -> &'static [u8] {
	WASM_BINARY.expect(
		"Development wasm binary is not available. This means the client is built with \
		 `SKIP_WASM_BUILD` flag and it is only usable for production chains. Please rebuild with \
		 the flag disabled.",
	)
}

// External imports
use currencies::BasicCurrencyAdapter;
use orml_traits::parameter_type_with_key;

mod weights;

// primitives imports
use crate::opaque::SessionKeys;
//use pallet_evm::{EnsureAddressTruncated, HashedAddressMapping};
pub use estate::{MintingRateInfo, Range as MintingRange};
use pallet_grandpa::{fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList};
pub use parachain_staking::{InflationInfo, Range};
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::crypto::Public;
use sp_core::{
	crypto::KeyTypeId,
	u32_trait::{_1, _2, _3, _4},
	OpaqueMetadata, H160, U256,
};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{
		AccountIdConversion, AccountIdLookup, BlakeTwo256, Block as BlockT, ConvertInto, IdentifyAccount, NumberFor,
		Verify, Zero,
	},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, FixedPointNumber, MultiSignature, Percent, Perquintill,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

// A few exports that help ease life for downstream crates.
pub use frame_support::{
	construct_runtime, parameter_types,
	traits::{EnsureOrigin, KeyOwnerProofSystem, Randomness, StorageInfo},
	weights::{
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
		DispatchClass, IdentityFee, Weight,
	},
	PalletId, RuntimeDebug, StorageValue,
};
use frame_system::{
	limits::{BlockLength, BlockWeights},
	Config, EnsureOneOf, EnsureRoot, RawOrigin,
};
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
pub use pallet_transaction_payment::{CurrencyAdapter, Multiplier, TargetedFeeAdjustment};
use primitives::{Amount, Balance, BlockNumber, FungibleTokenId};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Permill};

/// Constant values used within the runtime.
pub mod constants;

use codec::{Decode, Encode, MaxEncodedLen};
use constants::{currency::*, time::*};
use estate::weights::WeightInfo;
// use metaverse::weights::WeightInfo;
#[cfg(feature = "runtime-benchmarks")]
use frame_benchmarking::frame_support::pallet_prelude::Get;
use frame_support::traits::{Contains, FindAuthor, InstanceFilter, Nothing};
use frame_support::ConsensusEngineId;
use scale_info::TypeInfo;
use sp_core::sp_std::marker::PhantomData;
use sp_runtime::traits::OpaqueKeys;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.

pub mod opaque {
	use super::*;
	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

	//	pub type Block = BlockP;
	//	/// Opaque block header type.
	//	pub type Header = HeaderP;
	//	/// Opaque block identifier type.
	//	pub type BlockId = BlockIdP;
	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;
	impl_opaque_keys! {
		pub struct SessionKeys {
			pub aura: Aura,
			pub grandpa: Grandpa,
		}
	}
}

// To learn more about runtime versioning and what each of the following value means:
//   https://substrate.dev/docs/en/knowledgebase/runtime/upgrades#runtime-versioning
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("metaverse-runtime"),
	impl_name: create_runtime_str!("metaverse-runtime"),
	authoring_version: 1,
	// The version of the runtime specification. A full node will not attempt to use its native
	//   runtime in substitute for the on-chain Wasm runtime unless all of `spec_name`,
	//   `spec_version`, and `authoring_version` are the same between Wasm and native.
	// This value is set to 100 to notify Polkadot-JS App (https://polkadot.js.org/apps) to use
	//   the compatible custom types.
	spec_version: 103,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

/// We assume that ~10% of the block weight is consumed by `on_initialize` handlers.
/// This is used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 2 seconds of compute with a 3 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight = 2 * WEIGHT_PER_SECOND;

parameter_types! {
//	pub const Version: RuntimeVersion = VERSION;
//	pub const BlockHashCount: BlockNumber = 2400;
//	/// We allow for 2 seconds of compute with a 3 second average block time.
//	pub BlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights
//		::with_sensible_defaults(2 * WEIGHT_PER_SECOND, NORMAL_DISPATCH_RATIO);
//	pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
//		::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
//
//
	pub const BlockHashCount: BlockNumber = 2400;
	pub const Version: RuntimeVersion = VERSION;
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

	pub const SS58Prefix: u8 = 42;
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = frame_support::traits::Everything;
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = RuntimeBlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = RuntimeBlockLength;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type Call = Call;
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
	type Event = Event;
	/// The ubiquitous origin type.
	type Origin = Origin;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// Version of the runtime.
	type Version = Version;
	/// Converts a module to the index of the module in `construct_runtime!`.
	///
	/// This type is being generated by `construct_runtime!`.
	type PalletInfo = PalletInfo;
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = ();
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = ();
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
	/// The set code logic, just the default since we're not a parachain.
	type OnSetCode = ();
}

parameter_types! {
	pub const MetaverseNetworkTreasuryPalletId: PalletId = PalletId(*b"bit/trsy");
	pub const NftPalletId: PalletId = PalletId(*b"bit/bnft");
	pub const SwapPalletId: PalletId = PalletId(*b"bit/swap");
	pub const BitMiningTreasury: PalletId = PalletId(*b"cb/minig");
	pub const MaxAuthorities: u32 = 50;
}

impl pallet_randomness_collective_flip::Config for Runtime {}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type MaxAuthorities = MaxAuthorities;
	type DisabledValidators = ();
}

impl pallet_grandpa::Config for Runtime {
	type Event = Event;
	type Call = Call;

	type KeyOwnerProofSystem = ();

	type KeyOwnerProof = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

	type KeyOwnerIdentification =
		<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::IdentificationTuple;

	type HandleEquivocation = ();

	type WeightInfo = ();
	type MaxAuthorities = MaxAuthorities;
}

impl pallet_utility::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Aura;
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 500;
	pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = MaxLocks;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const TransactionByteFee: Balance = 1;
	pub const OperationalFeeMultiplier: u8 = 5;
	pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
	pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(1, 100_000);
	pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000_000u128);
}

impl pallet_transaction_payment::Config for Runtime {
	type OnChargeTransaction = CurrencyAdapter<Balances, ()>;
	type TransactionByteFee = TransactionByteFee;
	type OperationalFeeMultiplier = OperationalFeeMultiplier;
	type WeightToFee = IdentityFee<Balance>;
	type FeeMultiplierUpdate = TargetedFeeAdjustment<Self, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;
}

impl pallet_sudo::Config for Runtime {
	type Event = Event;
	type Call = Call;
}

parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = 5 * DAYS;
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 10;
}

// Council related pallets
type CouncilCollective = pallet_collective::Instance1;

impl pallet_collective::Config<CouncilCollective> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

// Metaverse network related pallets

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: FungibleTokenId| -> Balance {
		Zero::zero()
	};
}

parameter_types! {
	pub TreasuryModuleAccount: AccountId = MetaverseNetworkTreasuryPalletId::get().into_account();
}

impl orml_tokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = FungibleTokenId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = orml_tokens::TransferDust<Runtime, TreasuryModuleAccount>;
	type MaxLocks = MaxLocks;
	type DustRemovalWhitelist = Nothing;
}

parameter_types! {
	pub const GetNativeCurrencyId: FungibleTokenId = FungibleTokenId::NativeToken(0);
}

impl currencies::Config for Runtime {
	type Event = Event;
	type MultiSocialCurrency = Tokens;
	type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;
	type GetNativeCurrencyId = GetNativeCurrencyId;
}

parameter_types! {
	pub CreateClassDeposit: Balance = 500 * MILLICENTS;
	pub CreateAssetDeposit: Balance = 100 * MILLICENTS;
	pub MaxBatchTransfer: u32 = 100;
	pub MaxBatchMinting: u32 = 1000;
	pub MaxNftMetadata: u32 = 1024;
	pub PromotionIncentive: Balance = 1 * DOLLARS;
}

impl nft::Config for Runtime {
	type Event = Event;
	type CreateClassDeposit = CreateClassDeposit;
	type CreateAssetDeposit = CreateAssetDeposit;
	type Currency = Balances;
	type MultiCurrency = Currencies;
	type WeightInfo = weights::module_nft::WeightInfo<Runtime>;
	type PalletId = NftPalletId;
	type AuctionHandler = Auction;
	type MaxBatchTransfer = MaxBatchTransfer;
	type MaxBatchMinting = MaxBatchMinting;
	type MaxMetadata = MaxNftMetadata;
	type MiningResourceId = MiningResourceCurrencyId;
	type PromotionIncentive = PromotionIncentive;
}

parameter_types! {
	pub MaxClassMetadata: u32 = 1024;
	pub MaxTokenMetadata: u32 = 1024;
}

impl orml_nft::Config for Runtime {
	type ClassId = u32;
	type TokenId = u64;
	type ClassData = nft::NftClassData<Balance>;
	type TokenData = nft::NftAssetData<Balance>;
	type MaxClassMetadata = MaxClassMetadata;
	type MaxTokenMetadata = MaxTokenMetadata;
}

parameter_types! {
	pub MaxMetaverseMetadata: u32 = 1024;
	pub MinContribution: Balance = 1 * DOLLARS;
}

impl metaverse::Config for Runtime {
	type Event = Event;
	type MetaverseTreasury = MetaverseNetworkTreasuryPalletId;
	type Currency = Balances;
	type MaxMetaverseMetadata = MaxMetaverseMetadata;
	type MinContribution = MinContribution;
	type MetaverseCouncil = EnsureRootOrHalfMetaverseCouncil;
	type WeightInfo = weights::module_metaverse::WeightInfo<Runtime>;
}

parameter_types! {
	pub const MinimumLandPrice: Balance = 10 * DOLLARS;
	pub const LandTreasuryPalletId: PalletId = PalletId(*b"bit/land");
	pub const MinBlocksPerLandIssuanceRound: u32 = 20;
	pub const MinimumStake: Balance = 5 * DOLLARS;
}

impl estate::Config for Runtime {
	type Event = Event;
	type LandTreasury = LandTreasuryPalletId;
	type MetaverseInfoSource = Metaverse;
	type Currency = Balances;
	type MinimumLandPrice = MinimumLandPrice;
	type CouncilOrigin = pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>;
	type AuctionHandler = Auction;
	type MinBlocksPerRound = MinBlocksPerLandIssuanceRound;
	type WeightInfo = weights::module_estate::WeightInfo<Runtime>;
	type MinimumStake = MinimumStake;
	type RewardPaymentDelay = RewardPaymentDelay;
}

parameter_types! {
	pub const AuctionTimeToClose: u32 = 100; // Default 100800 Blocks
	pub const ContinuumSessionDuration: BlockNumber = 100; // Default 43200 Blocks
	pub const SpotAuctionChillingDuration: BlockNumber = 100; // Default 43200 Blocks
	pub const MinimumAuctionDuration: BlockNumber = 30; // Minimum duration is 300 blocks
	pub const RoyaltyFee: u16 = 10; // Loyalty fee 0.1%
}

impl auction::Config for Runtime {
	type Event = Event;
	type AuctionTimeToClose = AuctionTimeToClose;
	type Handler = Auction;
	type Currency = Balances;
	type ContinuumHandler = Continuum;
	type FungibleTokenCurrency = Tokens;
	type MetaverseInfoSource = Metaverse;
	type MinimumAuctionDuration = MinimumAuctionDuration;
	type EstateHandler = Estate;
	type RoyaltyFee = RoyaltyFee;
}

impl continuum::Config for Runtime {
	type Event = Event;
	type SessionDuration = ContinuumSessionDuration;
	type SpotAuctionChillingDuration = SpotAuctionChillingDuration;
	type EmergencyOrigin = pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>;
	type AuctionHandler = Auction;
	type AuctionDuration = SpotAuctionChillingDuration;
	type ContinuumTreasury = MetaverseNetworkTreasuryPalletId;
	type Currency = Balances;
	type MetaverseInfoSource = Metaverse;
}

impl tokenization::Config for Runtime {
	type Event = Event;
	type TokenId = u64;
	type MetaverseMultiCurrency = Currencies;
	type FungibleTokenTreasury = MetaverseNetworkTreasuryPalletId;
	type MetaverseInfoSource = Metaverse;
	type LiquidityPoolManager = Swap;
	type MinVestedTransfer = MinVestedTransfer;
	type VestedTransferOrigin = EnsureRootOrMetaverseTreasury;
}

parameter_types! {
	pub const SwapFee: (u32, u32) = (1, 20); //0.05%
}

impl swap::Config for Runtime {
	type Event = Event;
	type PalletId = SwapPalletId;
	type FungibleTokenCurrency = Tokens;
	type NativeCurrency = Balances;
	type GetSwapFee = SwapFee;
}

pub struct EnsureRootOrMetaverseTreasury;

impl EnsureOrigin<Origin> for EnsureRootOrMetaverseTreasury {
	type Success = AccountId;

	fn try_origin(o: Origin) -> Result<Self::Success, Origin> {
		Into::<Result<RawOrigin<AccountId>, Origin>>::into(o).and_then(|o| match o {
			RawOrigin::Root => Ok(MetaverseNetworkTreasuryPalletId::get().into_account()),
			RawOrigin::Signed(caller) => {
				if caller == MetaverseNetworkTreasuryPalletId::get().into_account() {
					Ok(caller)
				} else {
					Err(Origin::from(Some(caller)))
				}
			}
			r => Err(Origin::from(r)),
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_origin() -> Origin {
		Origin::from(RawOrigin::Signed(Default::default()))
	}
}

pub type EnsureRootOrHalfMetaverseCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, CouncilCollective>,
>;

parameter_types! {
	pub const MinVestedTransfer: Balance = 10 * DOLLARS;
}

impl pallet_vesting::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type BlockNumberToBalance = ConvertInto;
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = pallet_vesting::weights::SubstrateWeight<Runtime>;
	const MAX_VESTING_SCHEDULES: u32 = 100;
}

parameter_types! {
	//Mining Resource Currency Id
	pub const MiningResourceCurrencyId: FungibleTokenId = FungibleTokenId::MiningResource(0);
}

impl mining::Config for Runtime {
	type Event = Event;
	type MiningCurrency = Currencies;
	type BitMiningTreasury = BitMiningTreasury;
	type BitMiningResourceId = MiningResourceCurrencyId;
	type AdminOrigin = EnsureRootOrMetaverseTreasury;
}

parameter_types! {
	pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(17);
}

impl pallet_session::Config for Runtime {
	type Event = Event;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = parachain_staking::ValidatorOf<Self>;
	type ShouldEndSession = Staking;
	type NextSessionRotation = Staking;
	type SessionManager = Staking;
	type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	type WeightInfo = pallet_session::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	/// Minimum round length is 2 minutes (10 * 12 second block times)
	pub const MinBlocksPerRound: u32 = 10;
	/// Default BlocksPerRound is every hour (300 * 12 second block times)
	pub const DefaultBlocksPerRound: u32 = 30;
	/// Collator candidate exits are delayed by 2 hours (2 * 300 * block_time)
	pub const LeaveCandidatesDelay: u32 = 2;
	/// Nominator exits are delayed by 2 hours (2 * 300 * block_time)
	pub const LeaveNominatorsDelay: u32 = 2;
	/// Nomination revocations are delayed by 2 hours (2 * 300 * block_time)
	pub const RevokeNominationDelay: u32 = 2;
	/// Reward payments are delayed by 2 hours (2 * 300 * block_time)
	pub const RewardPaymentDelay: u32 = 2;
	/// Minimum 8 collators selected per round, default at genesis and minimum forever after
	pub const MinSelectedCandidates: u32 = 8;
	/// Maximum 100 nominators per collator
	pub const MaxNominatorsPerCollator: u32 = 100;
	/// Maximum 100 collators per nominator
	pub const MaxCollatorsPerNominator: u32 = 100;
	/// Default fixed percent a collator takes off the top of due rewards is 20%
	pub const DefaultCollatorCommission: Perbill = Perbill::from_percent(20);
	/// Default percent of inflation set aside for parachain bond every round
	pub const DefaultParachainBondReservePercent: Percent = Percent::from_percent(30);
	/// Minimum stake required to become a collator is 1_000
	pub const MinCollatorStk: u128 = 1 * DOLLARS;
	/// Minimum stake required to be reserved to be a candidate is 1_000
	pub const MinCollatorCandidateStk: u128 = 1 * DOLLARS;
	/// Minimum stake required to be reserved to be a nominator is 5
	pub const MinNominatorStk: u128 = 5 * DOLLARS;
}

impl parachain_staking::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type MonetaryGovernanceOrigin = EnsureRoot<AccountId>;
	type MinBlocksPerRound = MinBlocksPerRound;
	type DefaultBlocksPerRound = DefaultBlocksPerRound;
	type LeaveCandidatesDelay = LeaveCandidatesDelay;
	type LeaveNominatorsDelay = LeaveNominatorsDelay;
	type RevokeNominationDelay = RevokeNominationDelay;
	type RewardPaymentDelay = RewardPaymentDelay;
	type MinSelectedCandidates = MinSelectedCandidates;
	type MaxNominatorsPerCollator = MaxNominatorsPerCollator;
	type MaxCollatorsPerNominator = MaxCollatorsPerNominator;
	type DefaultCollatorCommission = DefaultCollatorCommission;
	type DefaultParachainBondReservePercent = DefaultParachainBondReservePercent;
	type MinCollatorStk = MinCollatorStk;
	type MinCollatorCandidateStk = MinCollatorCandidateStk;
	type MinNomination = MinNominatorStk;
	type MinNominatorStk = MinNominatorStk;
	type WeightInfo = parachain_staking::weights::SubstrateWeight<Runtime>;
}

pub struct FindAuthorTruncated<F>(PhantomData<F>);

impl<F: FindAuthor<u32>> FindAuthor<H160> for FindAuthorTruncated<F> {
	fn find_author<'a, I>(digests: I) -> Option<H160>
	where
		I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
	{
		if let Some(author_index) = F::find_author(digests) {
			let authority_id = Aura::authorities()[author_index as usize].clone();
			return Some(H160::from_slice(&authority_id.to_raw_vec()[4..24]));
		}
		None
	}
}

//parameter_types! {
//	pub const ChainId: u64 = 42;
//	pub BlockGasLimit: U256 = U256::from(u32::max_value());
//}

//// EVM config
//impl pallet_evm::Config for Runtime {
//	//    type FeeCalculator = pallet_dynamic_fee::Module<Self>;
//	type FeeCalculator = ();
//	type GasWeightMapping = ();
//	//	type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
//	//    type BlockHashMapping = ();
//	type CallOrigin = EnsureAddressTruncated;
//	type WithdrawOrigin = EnsureAddressTruncated;
//	type AddressMapping = HashedAddressMapping<BlakeTwo256>;
//	type Currency = Balances;
//	type Event = Event;
//	type Runner = pallet_evm::runner::stack::Runner<Self>;
//	type Precompiles = (
//		pallet_evm_precompile_simple::ECRecover,
//		pallet_evm_precompile_simple::Sha256,
//		pallet_evm_precompile_simple::Ripemd160,
//		pallet_evm_precompile_simple::Identity,
//		pallet_evm_precompile_modexp::Modexp,
//		pallet_evm_precompile_simple::ECRecoverPublicKey,
//		pallet_evm_precompile_sha3fips::Sha3FIPS256,
//		pallet_evm_precompile_sha3fips::Sha3FIPS512,
//	);
//	type ChainId = ChainId;
//	type BlockGasLimit = BlockGasLimit;
//	type OnChargeTransaction = ();
//	type FindAuthor = FindAuthorTruncated<Aura>;
//}

parameter_types! {
	pub const OneBlock: BlockNumber = 1;
	pub const MinimumProposalDeposit: Balance = 50 * DOLLARS;
	pub const DefaultPreimageByteDeposit: Balance = 1 * DOLLARS;
	pub const DefaultVotingPeriod: u32 = 100;
	pub const DefaultLocalVoteLockingPeriod: u32 = 28;
	pub const DefaultEnactmentPeriod: u32 = 10;
	pub const DefaultProposalLaunchPeriod: u32 = 15;
	pub const DefaultMaxParametersPerProposal: u8 = 20;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
		RuntimeBlockWeights::get().max_block;
	pub const MaxScheduledPerBlock: u32 = 50;
}

impl pallet_scheduler::Config for Runtime {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type MaxScheduledPerBlock = MaxScheduledPerBlock;
	type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const LaunchPeriod: BlockNumber = 10 * MINUTES;
	pub const VotingPeriod: BlockNumber = 10 * MINUTES;
	pub const FastTrackVotingPeriod: BlockNumber = 10 * MINUTES;
	pub const InstantAllowed: bool = true;
	pub const MinimumDeposit: Balance = 100 * DOLLARS;
	pub const EnactmentPeriod: BlockNumber = 10 * MINUTES;
	pub const CooloffPeriod: BlockNumber = 10 * MINUTES;
	pub const PreimageByteDeposit: Balance = 1 * CENTS;
	pub const MaxVotes: u32 = 50;
	pub const MaxProposals: u32 = 50;
}

impl pallet_democracy::Config for Runtime {
	type Proposal = Call;
	type Event = Event;
	type Currency = Balances;
	type EnactmentPeriod = EnactmentPeriod;
	type LaunchPeriod = LaunchPeriod;
	type VotingPeriod = VotingPeriod;
	type VoteLockingPeriod = EnactmentPeriod;
	// Same as EnactmentPeriod
	type MinimumDeposit = MinimumDeposit;
	/// A straight majority of the council can decide what their next motion is.
	type ExternalOrigin = pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, CouncilCollective>;
	/// A super-majority can have the next scheduled referendum be a straight majority-carries vote.
	type ExternalMajorityOrigin = pallet_collective::EnsureProportionAtLeast<_3, _4, AccountId, CouncilCollective>;
	/// A unanimous council can have the next scheduled referendum be a straight default-carries
	/// (NTB) vote.
	type ExternalDefaultOrigin = pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, CouncilCollective>;
	/// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
	/// be tabled immediately and with a shorter voting/enactment period.
	type FastTrackOrigin = pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, CouncilCollective>;
	type InstantOrigin = pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, CouncilCollective>;
	type InstantAllowed = InstantAllowed;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	// To cancel a proposal which has been passed, 2/3 of the council must agree to it.
	type CancellationOrigin = pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, CouncilCollective>;
	// To cancel a proposal before it has been passed, the technical committee must be unanimous or
	// Root must agree.
	type CancelProposalOrigin = EnsureOneOf<
		AccountId,
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, CouncilCollective>,
	>;
	type BlacklistOrigin = EnsureRoot<AccountId>;
	// Any single technical committee member may veto a coming council proposal, however they can
	// only do it once and it lasts only for the cool-off period.
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, CouncilCollective>;
	type CooloffPeriod = CooloffPeriod;
	type PreimageByteDeposit = PreimageByteDeposit;
	type OperationalPreimageOrigin = pallet_collective::EnsureMember<AccountId, CouncilCollective>;
	type Slash = ();
	type Scheduler = Scheduler;
	type PalletsOrigin = OriginCaller;
	type MaxVotes = MaxVotes;
	type WeightInfo = pallet_democracy::weights::SubstrateWeight<Runtime>;
	type MaxProposals = MaxProposals;
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum ProposalType {
	Any,
	JustMetaverse,
}

impl Default for ProposalType {
	fn default() -> Self {
		Self::JustMetaverse
	}
}

impl InstanceFilter<Call> for ProposalType {
	fn filter(&self, c: &Call) -> bool {
		match self {
			ProposalType::Any => true,
			ProposalType::JustMetaverse => matches!(c, Call::Metaverse(..)),
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		self == &ProposalType::Any || self == o
	}
}

impl governance::Config for Runtime {
	type Event = Event;
	type DefaultPreimageByteDeposit = DefaultPreimageByteDeposit;
	type MinimumProposalDeposit = MinimumProposalDeposit;
	type DefaultProposalLaunchPeriod = DefaultProposalLaunchPeriod;
	type DefaultVotingPeriod = DefaultVotingPeriod;
	type DefaultEnactmentPeriod = DefaultEnactmentPeriod;
	type DefaultLocalVoteLockingPeriod = DefaultLocalVoteLockingPeriod;
	type DefaultMaxParametersPerProposal = DefaultMaxParametersPerProposal;
	type OneBlock = OneBlock;
	type Currency = Balances;
	type Slash = ();
	type MetaverseInfo = Metaverse;
	type PalletsOrigin = OriginCaller;
	type Proposal = Call;
	type Scheduler = Scheduler;
	type MetaverseLandInfo = Estate;
	type MetaverseCouncil = EnsureRootOrMetaverseTreasury;
	type ProposalType = ProposalType;
}

//parameter_types! {
//	pub const LocalChainId: chainbridge::ChainId = 1;
//	pub const ProposalLifetime: BlockNumber = 5 * MINUTES;
//}
//
//impl chainbridge::Config for Runtime {
//	type Event = Event;
//	type AdminOrigin = EnsureRoot<AccountId>;
//	type Proposal = Call;
//	type ChainId = LocalChainId;
//	type ProposalLifetime = ProposalLifetime;
//}
//
//parameter_types! {
//	//Testing ERC 20 Resource Id
//	pub const BridgeTokenId: [u8; 32] =
// hex_literal::hex!("000000000000000000000000000000c76ebe4a02bbc34786d860b355f5a5ce00");
//}
//
//impl modules_chainsafe::Config for Runtime {
//	type Event = Event;
//	type BridgeOrigin = chainbridge::EnsureBridge<Runtime>;
//	type Currency = Currencies;
//	type NativeCurrencyId = GetNativeCurrencyId;
//	type BridgeTokenId = BridgeTokenId;
//}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		// Core
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		Aura: pallet_aura::{Pallet, Config<T>},
		Grandpa: pallet_grandpa::{Pallet, Call, Storage, Config, Event},
		Utility: pallet_utility::{Pallet, Call, Event},

		// Governance
		Council: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>},

		// Token & Related
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Currencies: currencies::{ Pallet, Storage, Call, Event<T>},
		Tokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>},
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage},
		Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>},
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>},

		// Metaverse & Related
		OrmlNFT: orml_nft::{Pallet, Storage},
		Nft: nft::{Pallet, Call, Storage, Event<T>},
		Auction: auction::{Pallet, Call ,Storage, Event<T>},
		Metaverse: metaverse::{Pallet, Call, Storage, Event<T>},
		Continuum: continuum::{Pallet, Call, Storage, Config<T>, Event<T>},
		Tokenization: tokenization:: {Pallet, Call, Storage, Event<T>},
		Swap: swap:: {Pallet, Call, Storage ,Event<T>},
		Vesting: pallet_vesting::{Pallet, Call, Storage, Event<T>, Config<T>},
		Mining: mining:: {Pallet, Call, Storage ,Event<T>},
		Estate: estate::{Pallet, Call, Storage, Event<T>, Config},
		// Governance
		Governance: governance::{Pallet, Call ,Storage, Event<T>},
		Democracy: pallet_democracy::{Pallet, Call, Storage, Config<T>, Event<T>},

		// External consensus support
		Staking: parachain_staking::{Pallet, Call, Storage, Event<T>, Config<T>},
		Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>},

//		EVM: pallet_evm::{Pallet, Config, Call, Storage, Event<T>},

		// Bridge
//		ChainBridge: chainbridge::{Pallet, Call, Storage, Event<T>},
//		BridgeTransfer: modules_chainsafe::{Pallet, Call, Event<T>, Storage}
	}
);

//pub struct TransactionConverter;
//
//impl fp_rpc::ConvertTransaction<UncheckedExtrinsic> for TransactionConverter {
//    fn convert_transaction(&self, transaction: pallet_ethereum::Transaction) -> UncheckedExtrinsic
// {        UncheckedExtrinsic::new_unsigned(pallet_ethereum::Call::<Runtime>::
// transact(transaction).into())    }
//}
//
//impl fp_rpc::ConvertTransaction<opaque::UncheckedExtrinsic> for TransactionConverter {
//    fn convert_transaction(&self, transaction: pallet_ethereum::Transaction) ->
// opaque::UncheckedExtrinsic {        let extrinsic =
//
// UncheckedExtrinsic::new_unsigned(pallet_ethereum::Call::<Runtime>::transact(transaction).into());
//        let encoded = extrinsic.encode();
//        opaque::UncheckedExtrinsic::decode(&mut &encoded[..]).expect("Encoded extrinsic is always
// valid")    }
//}

// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
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
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive =
	frame_executive::Executive<Runtime, Block, frame_system::ChainContext<Runtime>, Runtime, AllPallets>;

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block);
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
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

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities().into_inner()
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			opaque::SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl fg_primitives::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> GrandpaAuthorityList {
			Grandpa::grandpa_authorities()
		}

		fn current_set_id() -> fg_primitives::SetId {
			Grandpa::current_set_id()
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			_equivocation_proof: fg_primitives::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			_key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			None
		}

		fn generate_key_ownership_proof(
			_set_id: fg_primitives::SetId,
			_authority_id: GrandpaId,
		) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
			// NOTE: this is the only implementation possible since we've
			// defined our key owner proof type as a bottom type (i.e. a type
			// with no values).
			None
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
			use nft::benchmarking::Pallet as NftBench;
			use estate::benchmarking::EstateModule as EstateBench;
			use auction::benchmarking::AuctionModule as AuctionBench;
			use metaverse::benchmarking::MetaverseModule as MetaverseBench;

			let mut list = Vec::<BenchmarkList>::new();

			list_benchmark!(list, extra, frame_system, SystemBench::<Runtime>);
			list_benchmark!(list, extra, pallet_balances, Balances);
			list_benchmark!(list, extra, pallet_timestamp, Timestamp);
			list_benchmark!(list, extra, nft, NftBench::<Runtime>);
			list_benchmark!(list, extra, estate, EstateBench::<Runtime>);
			list_benchmark!(list, extra, auction, AuctionBench::<Runtime>);
			list_benchmark!(list, extra, metaverse, MetaverseBench::<Runtime>);
			list_benchmark!(list, extra, pallet_utility, Utility);

			let storage_info = AllPalletsWithSystem::storage_info();

			return (list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};

			use frame_system_benchmarking::Pallet as SystemBench;
			impl frame_system_benchmarking::Config for Runtime {}

			use nft::benchmarking::Pallet as NftBench;
			use estate::benchmarking::EstateModule as EstateBench;
			use auction::benchmarking::AuctionModule as AuctionBench;
			use metaverse::benchmarking::MetaverseModule as MetaverseBench;

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
			add_benchmark!(params, batches, pallet_timestamp, Timestamp);
			add_benchmark!(params, batches, nft, NftBench::<Runtime>);
			add_benchmark!(params, batches, estate, EstateBench::<Runtime>);
			add_benchmark!(params, batches, auction, AuctionBench::<Runtime>);
			add_benchmark!(params, batches, metaverse, MetaverseBench::<Runtime>);
			add_benchmark!(params, batches, pallet_utility, Utility);


			Ok(batches)
		}
	}
}
