#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]
// The `large_enum_variant` warning originates from `construct_runtime` macro.
#![allow(clippy::large_enum_variant)]
#![allow(clippy::unnecessary_mut_passed)]
#![allow(clippy::or_fun_call)]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

// use frame_support::traits::Currency;
//Custom import
use orml_currencies::{BasicCurrencyAdapter, Currency};

pub use constants::{currency::*, time::*};
use sp_api::impl_runtime_apis;
use sp_core::{
    crypto::KeyTypeId,
    u32_trait::{_1, _2, _3, _4},
    OpaqueMetadata, H160,
};

use sp_runtime::traits::{
    AccountIdLookup, BlakeTwo256, Block as BlockT, Convert, IdentifyAccount, NumberFor, OpaqueKeys,
    SaturatedConversion, StaticLookup,
};
use sp_runtime::{
    create_runtime_str,
    curve::PiecewiseLinear,
    generic, impl_opaque_keys,
    traits::{AccountIdConversion, Zero},
    transaction_validity::{TransactionPriority, TransactionSource, TransactionValidity},
    ApplyExtrinsicResult, DispatchResult, FixedPointNumber, ModuleId, MultiSignature, Perquintill,
};
use sp_std::{collections::btree_set::BTreeSet, prelude::*};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

use frame_system::{EnsureOneOf, EnsureRoot};
use orml_tokens::CurrencyAdapter as TokenCurrencyAdapter;
use orml_traits::{
    create_median_value_data_provider, parameter_type_with_key, DataFeeder, DataProviderExtended,
};
use pallet_grandpa::fg_primitives;
use pallet_grandpa::{AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList};
use pallet_session::historical as pallet_session_historical;

use cumulus_primitives_core::{relay_chain::Balance as RelayChainBalance, ParaId};
use pallet_transaction_payment::{FeeDetails, RuntimeDispatchInfo};

// XCM imports
use frame_system::limits::{BlockLength, BlockWeights};
use orml_xcm_support::{
    CurrencyIdConverter, IsConcreteWithGeneralKey, MultiCurrencyAdapter, NativePalletAssetOr,
    XcmHandler as XcmHandlerT,
};
use polkadot_parachain::primitives::Sibling;
use xcm::v0::{Junction, MultiLocation, NetworkId, Xcm};
use xcm_builder::{
    AccountId32Aliases, ChildParachainConvertsVia, CurrencyAdapter, LocationInverter,
    ParentIsDefault, RelayChainAsNative, SiblingParachainAsNative, SiblingParachainConvertsVia,
    SignedAccountId32AsNative, SovereignSignedViaLocation,
};
use xcm_executor::{
    traits::{IsConcrete, NativeAsset},
    Config, XcmExecutor,
};

mod weights;

// A few exports that help ease life for downstream crates.
pub use frame_support::{
    construct_runtime, parameter_types,
    traits::{Contains, ContainsLengthBound, KeyOwnerProofSystem, Randomness, U128CurrencyToVote},
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
        DispatchClass, IdentityFee, Weight,
    },
    StorageValue,
};
pub use pallet_balances::Call as BalancesCall;
pub use pallet_staking::StakerStatus;
pub use pallet_timestamp::Call as TimestampCall;
pub use primitives;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Percent, Permill};

pub use primitives::*;

//Bit.Country constants
mod constants;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
// pub mod opaque {
//     use super::*;

//     pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

//     /// Opaque block type.
//     pub type Block = generic::Block<Header, UncheckedExtrinsic>;

//     pub type SessionHandlers = ();
// }

impl_opaque_keys! {
    pub struct SessionKeys {
        // pub grandpa: Grandpa,
        // pub babe: Babe,
    }
}

pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("bitcountry-node"),
    impl_name: create_runtime_str!("bitcountry-node"),
    authoring_version: 1,
    spec_version: 1,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
};

/// This determines the average expected block time that we are targetting.
/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
/// up by `pallet_aura` to implement `fn slot_duration()`.
///
/// Change this to adjust the block time.
pub const MILLISECS_PER_BLOCK: u64 = 6000;

pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

// 1 in 4 blocks (on average, not counting collisions) will be primary babe blocks.
pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

pub const BCGS: Balance = CENTS;

#[derive(codec::Encode, codec::Decode)]
pub enum XCMPMessage<XAccountId, XBalance> {
    /// Transfer tokens to the given account from the Parachain account.
    TransferToken(XAccountId, XBalance),
}

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

/// We assume that ~10% of the block weight is consumed by `on_initalize` handlers.
/// This is used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 2 seconds of compute with a 6 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight = 2 * WEIGHT_PER_SECOND;

parameter_types! {
    pub const BlockHashCount: BlockNumber = 250;
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
    type BaseCallFilter = ();
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
    type OnSetCode = ParachainSystem;
}

parameter_types! {
    pub const EpochDuration: u64 = EPOCH_DURATION_IN_SLOTS;
    pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
    where
        Call: From<C>,
{
    type OverarchingCall = Call;
    type Extrinsic = UncheckedExtrinsic;
}

// impl pallet_babe::Config for Runtime {
//     type EpochDuration = EpochDuration;
//     type ExpectedBlockTime = ExpectedBlockTime;
//     type EpochChangeTrigger = pallet_babe::ExternalTrigger;
//     type KeyOwnerProofSystem = Historical;
//     type KeyOwnerProof = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
//         KeyTypeId,
//         pallet_babe::AuthorityId,
//     )>>::Proof;
//     type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
//         KeyTypeId,
//         pallet_babe::AuthorityId,
//     )>>::IdentificationTuple;
//     type HandleEquivocation = pallet_babe::EquivocationHandler<Self::KeyOwnerIdentification, ()>; // Offences
//     type WeightInfo = ();
// }

// impl pallet_grandpa::Config for Runtime {
//     type Event = Event;
//     type Call = Call;

//     type KeyOwnerProofSystem = Historical;

//     type KeyOwnerProof =
//         <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

//     type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
//         KeyTypeId,
//         GrandpaId,
//     )>>::IdentificationTuple;

//     type HandleEquivocation = pallet_grandpa::EquivocationHandler<Self::KeyOwnerIdentification, ()>; // Offences

//     type WeightInfo = ();
// }

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

// parameter_types! {
//     pub const UncleGenerations: BlockNumber = 5;
// }

// impl pallet_authorship::Config for Runtime {
//     type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Babe>;
//     type UncleGenerations = UncleGenerations;
//     type FilterUncle = ();
//     type EventHandler = (Staking, ()); // ImOnline
// }

parameter_types! {
    pub const ExistentialDeposit: u128 = 500;
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Runtime {
    type MaxLocks = MaxLocks;
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
    pub const TransactionByteFee: Balance = BCGS / (1024 * 1024);
}

impl pallet_transaction_payment::Config for Runtime {
    type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = IdentityFee<Balance>;
    type FeeMultiplierUpdate = ();
}

impl pallet_sudo::Config for Runtime {
    type Event = Event;
    type Call = Call;
}

type EnsureRootOrHalfGeneralCouncil = EnsureOneOf<
    AccountId,
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, GeneralCouncilInstance>,
>;

type EnsureRootOrTwoThirdsGeneralCouncil = EnsureOneOf<
    AccountId,
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionMoreThan<_2, _3, AccountId, GeneralCouncilInstance>,
>;

type EnsureRootOrThreeFourthsGeneralCouncil = EnsureOneOf<
    AccountId,
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionMoreThan<_3, _4, AccountId, GeneralCouncilInstance>,
>;

parameter_types! {
    pub const GeneralCouncilMotionDuration: BlockNumber = 0;
    pub const GeneralCouncilMaxProposals: u32 = 100;
    pub const GeneralCouncilMaxMembers: u32 = 100;
}

type GeneralCouncilInstance = pallet_collective::Instance1;

impl pallet_collective::Config<GeneralCouncilInstance> for Runtime {
    type Origin = Origin;
    type Proposal = Call;
    type Event = Event;
    type MotionDuration = GeneralCouncilMotionDuration;
    type MaxProposals = GeneralCouncilMaxProposals;
    type MaxMembers = GeneralCouncilMaxMembers;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type WeightInfo = ();
}

pub struct GeneralCouncilProvider;

impl Contains<AccountId> for GeneralCouncilProvider {
    fn contains(who: &AccountId) -> bool {
        GeneralCouncil::is_member(who)
    }

    fn sorted_members() -> Vec<AccountId> {
        GeneralCouncil::members()
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn add(_: &AccountId) {
        todo!()
    }
}

impl ContainsLengthBound for GeneralCouncilProvider {
    fn max_len() -> usize {
        100
    }
    fn min_len() -> usize {
        0
    }
}

parameter_types! {
    pub const ProposalBond: Permill = Permill::from_percent(5);
    pub const ProposalBondMinimum: Balance = DOLLARS;
    pub const SpendPeriod: BlockNumber = DAYS;
    pub const Burn: Permill = Permill::from_percent(0);
    pub const TipCountdown: BlockNumber = DAYS;
    pub const TipFindersFee: Percent = Percent::from_percent(10);
    pub const TipReportDepositBase: Balance = DOLLARS;
    pub const SevenDays: BlockNumber = 7 * DAYS;
    pub const ZeroDay: BlockNumber = 0;
    pub const OneDay: BlockNumber = DAYS;
    pub const BountyDepositBase: Balance = DOLLARS;
    pub const BountyDepositPayoutDelay: BlockNumber = DAYS;
    pub const BountyUpdatePeriod: BlockNumber = 14 * DAYS;
    pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
    pub const BountyValueMinimum: Balance = 5 * DOLLARS;
    pub const DataDepositPerByte: Balance = CENTS;
    pub const MaximumReasonLength: u32 = 16384;
}

parameter_types! {
    pub const BitCountryTreasuryModuleId: ModuleId = ModuleId(*b"bit/trsy");
    pub const CountryFundModuleId: ModuleId = ModuleId(*b"bit/fund");
    pub const NftModuleId: ModuleId = ModuleId(*b"bit/bnft");
}

impl pallet_treasury::Config for Runtime {
    type ModuleId = BitCountryTreasuryModuleId;
    type Currency = Balances;
    type ApproveOrigin = EnsureRootOrHalfGeneralCouncil;
    type RejectOrigin = EnsureRootOrHalfGeneralCouncil;
    type Event = Event;
    type OnSlash = ();
    type ProposalBond = ProposalBond;
    type ProposalBondMinimum = ProposalBondMinimum;
    type SpendPeriod = SpendPeriod;
    type Burn = Burn;
    type BurnDestination = ();
    type SpendFunds = Bounties;
    type WeightInfo = ();
}

parameter_type_with_key! {
    pub ExistentialDeposits: |currency_id: CurrencyId| -> Balance {
        Zero::zero()
    };
}

parameter_types! {
    pub TreasuryModuleAccount: AccountId = BitCountryTreasuryModuleId::get().into_account();
}

impl orml_tokens::Config for Runtime {
    type Event = Event;
    type Balance = Balance;
    type Amount = Amount;
    type CurrencyId = CurrencyId;
    type WeightInfo = ();
    type ExistentialDeposits = ExistentialDeposits;
    type OnDust = orml_tokens::TransferDust<Runtime, TreasuryModuleAccount>;
}

parameter_types! {
    pub const GetNativeCurrencyId: CurrencyId = CurrencyId::BCG;
}

pub type BCGToken = BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;

impl orml_currencies::Config for Runtime {
    type Event = Event;
    type MultiCurrency = orml_tokens::Pallet<Runtime>;
    type NativeCurrency = BCGToken;
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type WeightInfo = ();
}

parameter_types! {
    pub CreateClassDeposit: Balance = 500 * MILLICENTS;
    pub CreateAssetDeposit: Balance = 100 * MILLICENTS;
}

impl nft::Config for Runtime {
    type Event = Event;
    type CreateClassDeposit = CreateClassDeposit;
    type CreateAssetDeposit = CreateAssetDeposit;
    type Randomness = RandomnessCollectiveFlip;
    type Currency = Currency<Runtime, GetNativeCurrencyId>;
    type WeightInfo = weights::module_nft::WeightInfo<Runtime>;
    type ModuleId = NftModuleId;
}

impl orml_nft::Config for Runtime {
    type ClassId = u32;
    type TokenId = u64;
    type ClassData = nft::NftClassData<Balance>;
    type TokenData = nft::NftAssetData<Balance>;
}

impl country::Trait for Runtime {
    type Event = Event;
    type ModuleId = CountryFundModuleId;
}

impl block::Trait for Runtime {
    type Event = Event;
    type RandomnessSource = RandomnessCollectiveFlip;
}

impl section::Trait for Runtime {
    type Event = Event;
    type BlockRandomnessSource = RandomnessCollectiveFlip;
}

parameter_types! {
    pub const AuctionTimeToClose: u32 = 100800;
}

impl auction::Config for Runtime {
    type Event = Event;
    type AuctionTimeToClose = AuctionTimeToClose;
    type AuctionId = u64;
    type Handler = Auction;
    type Currency = Balances;
}

impl tokenization::Config for Runtime {
    type Event = Event;
    type TokenId = u64;
    type MultiCurrency = orml_tokens::Module<Runtime>;
    type CountryCurrency = Currencies;
}

impl pallet_bounties::Config for Runtime {
    type Event = Event;
    type BountyDepositBase = BountyDepositBase;
    type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
    type BountyUpdatePeriod = BountyUpdatePeriod;
    type BountyCuratorDeposit = BountyCuratorDeposit;
    type BountyValueMinimum = BountyValueMinimum;
    type DataDepositPerByte = DataDepositPerByte;
    type MaximumReasonLength = MaximumReasonLength;
    type WeightInfo = ();
}

parameter_types! {
    pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(17);
}

// impl pallet_session::Config for Runtime {
//     type Event = Event;
//     type ValidatorId = <Self as frame_system::Config>::AccountId;
//     type ValidatorIdOf = pallet_staking::StashOf<Self>;
//     type ShouldEndSession = Babe;
//     type NextSessionRotation = Babe;
//     type SessionManager = pallet_session::historical::NoteHistoricalRoot<Self, Staking>;
//     type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
//     type Keys = SessionKeys;
//     type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
//     type WeightInfo = ();
// }

// impl pallet_session::historical::Config for Runtime {
//     type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
//     type FullIdentificationOf = pallet_staking::ExposureOf<Runtime>;
// }

// pallet_staking_reward_curve::build! {
//     const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
//         min_inflation: 0_025_000,
//         max_inflation: 0_100_000,
//         ideal_stake: 0_500_000,
//         falloff: 0_050_000,
//         max_piece_count: 40,
//         test_precision: 0_005_000,
//     );
// }

// parameter_types! {
//     pub const SessionsPerEra: sp_staking::SessionIndex = 3; // 3 hours
//     pub const BondingDuration: pallet_staking::EraIndex = 4; // 12 hours
//     pub const SlashDeferDuration: pallet_staking::EraIndex = 2; // 6 hours
//     pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
//     pub const MaxNominatorRewardedPerValidator: u32 = 64;
//     pub const ElectionLookahead: BlockNumber = EPOCH_DURATION_IN_BLOCKS / 4;
//     pub const MaxIterations: u32 = 5;
//     pub StakingUnsignedPriority: TransactionPriority =
//         Perbill::from_percent(90) * TransactionPriority::max_value();
//     // 0.05%. The higher the value, the more strict solution acceptance becomes.
//     pub MinSolutionScoreBump: Perbill = Perbill::from_rational_approximation(5u32, 10_000);
// }

// impl pallet_staking::Config for Runtime {
//     type Currency = Balances;
//     type UnixTime = Timestamp;
//     type CurrencyToVote = U128CurrencyToVote;
//     type RewardRemainder = BitCountryTreasury;
//     type Event = Event;
//     type Slash = BitCountryTreasury; // send the slashed funds to the pallet treasury.
//     type Reward = (); // rewards are minted from the void
//     type SessionsPerEra = SessionsPerEra;
//     type BondingDuration = BondingDuration;
//     type SlashDeferDuration = SlashDeferDuration;
//     /// A super-majority of the council can cancel the slash.
//     type SlashCancelOrigin = EnsureRootOrThreeFourthsGeneralCouncil;
//     type SessionInterface = Self;
//     type RewardCurve = RewardCurve;
//     type NextNewSession = Session;
//     type ElectionLookahead = ElectionLookahead;
//     type Call = Call;
//     type MaxIterations = MaxIterations;
//     type MinSolutionScoreBump = MinSolutionScoreBump;
//     type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
//     type UnsignedPriority = ();
//     type WeightInfo = ();
//     type OffchainSolutionWeightLimit = ();
// }

impl cumulus_pallet_parachain_system::Config for Runtime {
    type Event = Event;
    type OnValidationData = ();
    type SelfParaId = parachain_info::Module<Runtime>;
    type DownwardMessageHandlers = XcmHandler;
    type XcmpMessageHandlers = XcmHandler;
}

impl parachain_info::Config for Runtime {}

// parameter_types! {
//     pub const RococoLocation: MultiLocation = MultiLocation::X1(Junction::Parent);
//     pub const RococoNetwork: NetworkId = NetworkId::Polkadot;
//     pub RelayChainOrigin: Origin = xcm_handler::Origin::Relay.into();
//     pub Ancestry: MultiLocation = Junction::Parachain {
//         id: ParachainInfo::parachain_id().into()
//     }.into();
// }

parameter_types! {
    pub const PolkadotNetworkId: NetworkId = NetworkId::Polkadot;
}

pub struct AccountId32Convert;

impl Convert<AccountId, [u8; 32]> for AccountId32Convert {
    fn convert(account_id: AccountId) -> [u8; 32] {
        account_id.into()
    }
}

parameter_types! {
    pub BitCountryNetwork: NetworkId = NetworkId::Named("bitcountry".into());
    pub RelayChainOrigin: Origin = cumulus_pallet_xcm_handler::Origin::Relay.into();
    pub Ancestry: MultiLocation = MultiLocation::X1(Junction::Parachain {
        id: ParachainInfo::parachain_id().into(),
    });
    pub const RelayChainCurrencyId: CurrencyId = CurrencyId::DOT;
}

pub type LocationConverter = (
    ParentIsDefault<AccountId>,
    SiblingParachainConvertsVia<Sibling, AccountId>,
    ChildParachainConvertsVia<ParaId, AccountId>,
    AccountId32Aliases<BitCountryNetwork, AccountId>,
);

pub type LocalAssetTransactor = MultiCurrencyAdapter<
    Currencies,
    UnknownTokens,
    IsConcreteWithGeneralKey<CurrencyId, RelayToNative>,
    LocationConverter,
    AccountId,
    CurrencyIdConverter<CurrencyId, RelayChainCurrencyId>,
    CurrencyId,
>;

pub type LocalOriginConverter = (
    SovereignSignedViaLocation<LocationConverter, Origin>,
    RelayChainAsNative<RelayChainOrigin, Origin>,
    SiblingParachainAsNative<cumulus_pallet_xcm_handler::Origin, Origin>,
    SignedAccountId32AsNative<BitCountryNetwork, Origin>,
);

parameter_types! {
    pub NativeOrmlTokens: BTreeSet<(Vec<u8>, MultiLocation)> = {
        let mut t = BTreeSet::new();
        //TODO: might need to add other assets based on orml-tokens
        t.insert(("AUSD".into(), MultiLocation::X1(Junction::Parachain { id: 666 })));
        t
    };
}

pub struct XcmConfig;

impl Config for XcmConfig {
    type Call = Call;
    type XcmSender = XcmHandler;
    type AssetTransactor = LocalAssetTransactor;
    type OriginConverter = LocalOriginConverter;
    //TODO: might need to add other assets based on orml-tokens
    type IsReserve = NativePalletAssetOr<NativeOrmlTokens>;
    type IsTeleporter = ();
    type LocationInverter = LocationInverter<Ancestry>;
}

impl cumulus_pallet_xcm_handler::Config for Runtime {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type UpwardMessageSender = ParachainSystem;
    type XcmpMessageSender = ParachainSystem;
    type SendXcmOrigin = EnsureRoot<AccountId>;
    type AccountIdConverter = LocationConverter;
}

pub struct RelayToNative;

impl Convert<RelayChainBalance, Balance> for RelayToNative {
    fn convert(val: u128) -> Balance {
        // native is 18
        // relay is 12
        val * 1_000_000
    }
}

pub struct NativeToRelay;

impl Convert<Balance, RelayChainBalance> for NativeToRelay {
    fn convert(val: u128) -> Balance {
        // native is 18
        // relay is 12
        val / 1_000_000
    }
}

pub struct HandleXcm;

impl XcmHandlerT<AccountId> for HandleXcm {
    fn execute_xcm(origin: AccountId, xcm: Xcm) -> DispatchResult {
        XcmHandler::execute_xcm(origin, xcm)
    }
}

parameter_types! {
    pub SelfLocation: MultiLocation = (Junction::Parent, Junction::Parachain { id: ParachainInfo::parachain_id().into() }).into();
}

impl orml_xtokens::Config for Runtime {
    type Event = Event;
    type Balance = Balance;
    // type ToRelayChainBalance = NativeToRelay;
    type AccountId32Convert = AccountId32Convert;
    //TODO: change network id if kusama
    // type RelayChainNetworkId = PolkadotNetworkId;
    // type ParaId = ParachainInfo;
    type XcmHandler = HandleXcm;
    type SelfLocation = SelfLocation;
    type CurrencyId = CurrencyId;
}

impl orml_unknown_tokens::Config for Runtime {
    type Event = Event;
}

construct_runtime! {
    pub enum Runtime where
        Block = Block,
        NodeBlock = primitives::Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {

        //Core
        System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Call, Storage},

        //Token
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        TransactionPayment: pallet_transaction_payment::{Pallet, Storage},

        // Consensus & Staking
        // Authorship: pallet_authorship::{Pallet, Call, Storage, Inherent},
        // Babe: pallet_babe::{Pallet, Call, Storage, Config, Inherent, ValidateUnsigned},
        // Grandpa: pallet_grandpa::{Pallet, Call, Storage, Config, Event, ValidateUnsigned},
        // Staking: pallet_staking::{Pallet, Call, Config<T>, Storage, Event<T>},
        // Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>},
        // Historical: pallet_session_historical::{Pallet},

        // Governance
        GeneralCouncil: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>},

        //Treasury
        BitCountryTreasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>},
        Bounties: pallet_bounties::{Pallet, Call, Storage, Event<T>},

        //Bit.Country Core
        CountryModule: country::{Pallet, Call, Storage, Event<T>},
        BlockModule: block::{Pallet, Call, Storage, Event<T>},
        SectionModule: section::{Pallet, Call, Storage, Event<T>},
        OrmlNFT: orml_nft::{Pallet ,Storage, Config<T>},
        NftModule: nft::{Pallet, Call ,Storage, Event<T>},
        Auction: auction::{Pallet, Call ,Storage, Event<T>},
        Currencies: orml_currencies::{ Pallet, Storage, Call, Event<T>},
        Tokens: orml_tokens::{ Pallet, Storage, Call, Event<T>},
        TokenizationModule: tokenization:: {Pallet, Call, Storage, Event<T>},

        // Parachain
        ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Storage, Inherent, Event},
        ParachainInfo: parachain_info::{Pallet, Storage, Config},
        XcmHandler: cumulus_pallet_xcm_handler::{Pallet, Call ,Event<T>, Origin},
        XTokens: orml_xtokens::{Pallet, Storage, Call, Event<T>},
        UnknownTokens: orml_unknown_tokens::{Pallet, Storage, Event},
        //Dev
        Sudo: pallet_sudo::{Pallet, Call, Storage, Config<T>, Event<T>},
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
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllModules,
>;

#[cfg(not(feature = "disable-runtime-api"))]
impl_runtime_apis! {
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
            Runtime::metadata().into()
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

        fn random_seed() -> <Block as BlockT>::Hash {
            RandomnessCollectiveFlip::random_seed().0
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx)
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
    }

    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
            use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};

            use frame_system_benchmarking::Module as SystemBench;
            impl frame_system_benchmarking::Config for Runtime {}

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

            if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
            Ok(batches)
        }
    }
}

cumulus_pallet_parachain_system::register_validate_block!(Runtime, Executive);
