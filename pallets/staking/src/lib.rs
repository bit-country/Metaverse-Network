#![cfg_attr(not(feature = "std"), no_std)]
#![feature(drain_filter)]
#![recursion_limit = "128"]

use codec::{Decode, Encode, HasCompact};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{
        DispatchErrorWithPostInfo, DispatchResult, DispatchResultWithPostInfo, WithPostDispatchInfo,
    },
    ensure,
    storage::IterableStorageMap,
    traits::{
        Currency, CurrencyToVote, EnsureOrigin, EstimateNextNewSession, Get, Imbalance, IsSubType,
        LockIdentifier, LockableCurrency, OnUnbalanced, UnixTime, WithdrawReasons,
    },
    weights::{
        constants::{WEIGHT_PER_MICROS, WEIGHT_PER_NANOS},
        Weight,
    },
};
use frame_system::{
    self as system, ensure_none, ensure_root, ensure_signed, offchain::SendTransactionTypes,
};
use pallet_session::historical;
use sp_npos_elections::{
    generate_solution_type, is_score_better, seq_phragmen, to_support_map, Assignment,
    CompactSolution, ElectionResult as PrimitiveElectionResult, ElectionScore, EvaluateSupport,
    ExtendedBalance, PerThing128, SupportMap, VoteWeight,
};
use sp_runtime::{
    curve::PiecewiseLinear,
    traits::{
        AtLeast32BitUnsigned, CheckedSub, Convert, Dispatchable, SaturatedConversion, Saturating,
        StaticLookup, Zero,
    },
    transaction_validity::{
        InvalidTransaction, TransactionPriority, TransactionSource, TransactionValidity,
        TransactionValidityError, ValidTransaction,
    },
    DispatchError, PerU16, Perbill, Percent, RuntimeDebug,
};
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_staking::{
    offence::{Offence, OffenceDetails, OffenceError, OnOffenceHandler, ReportOffence},
    SessionIndex,
};
use sp_std::{
    collections::btree_map::BTreeMap,
    convert::{From, TryInto},
    mem::size_of,
    prelude::*,
    result,
};
pub use weights::WeightInfo;

const STAKING_ID: LockIdentifier = *b"staking ";
pub const MAX_UNLOCKING_CHUNKS: usize = 32;
pub const MAX_NOMINATIONS: usize = <CompactAssignments as CompactSolution>::LIMIT;

pub(crate) const LOG_TARGET: &'static str = "staking";

// syntactic sugar for logging.
#[macro_export]
macro_rules! log {
	($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
		frame_support::debug::$level!(
			target: crate::LOG_TARGET,
			$patter $(, $values)*
		)
	};
}

/// Data type used to index nominators in the compact type
pub type NominatorIndex = u32;

/// Data type used to index validators in the compact type.
pub type ValidatorIndex = u16;

// Ensure the size of both ValidatorIndex and NominatorIndex. They both need to be well below usize.
static_assertions::const_assert!(size_of::<ValidatorIndex>() <= size_of::<usize>());
static_assertions::const_assert!(size_of::<NominatorIndex>() <= size_of::<usize>());
static_assertions::const_assert!(size_of::<ValidatorIndex>() <= size_of::<u32>());
static_assertions::const_assert!(size_of::<NominatorIndex>() <= size_of::<u32>());

/// Maximum number of stakers that can be stored in a snapshot.
pub(crate) const MAX_VALIDATORS: usize = ValidatorIndex::max_value() as usize;
pub(crate) const MAX_NOMINATORS: usize = NominatorIndex::max_value() as usize;

/// Counter for the number of eras that have passed.
pub type EraIndex = u32;

/// Counter for the number of "reward" points earned by a given validator.
pub type RewardPoint = u32;

// Note: Maximum nomination limit is set here -- 16.
generate_solution_type!(
    #[compact]
    pub struct CompactAssignments::<NominatorIndex, ValidatorIndex, OffchainAccuracy>(16)
);

/// Accuracy used for on-chain election.
pub type ChainAccuracy = Perbill;

/// Accuracy used for off-chain election. This better be small.
pub type OffchainAccuracy = PerU16;

/// Balance of an account.
pub type Balance = u128;

/// Power of an account.
pub type Power = u32;

/// A timestamp: milliseconds since the unix epoch.
/// `u64` is enough to represent a duration of half a billion years, when the
/// time scale is milliseconds.
pub type TsInMs = u64;

pub type AccountId<T> = <T as frame_system::Config>::AccountId;
pub type BlockNumber<T> = <T as frame_system::Config>::BlockNumber;

/// The balance type of this module.
pub type NativeCurrencyBalance<T> = <NativeCurrencyCurrency<T> as Currency<AccountId<T>>>::Balance;
pub type NativeCurrencyPositiveImbalance<T> =
    <NativeCurrency<T> as Currency<AccountId<T>>>::PositiveImbalance;
pub type NativeCurrencyNegativeImbalance<T> =
    <NativeCurrency<T> as Currency<AccountId<T>>>::NegativeImbalance;

/// The balance type of this module.
pub type StakingCurrencyBalance<T> = <StakingCurrency<T> as Currency<AccountId<T>>>::Balance;
pub type StakingCurrencyPositiveImbalance<T> =
    <StakingCurrency<T> as Currency<AccountId<T>>>::PositiveImbalance;
pub type StakingCurrencyNegativeImbalance<T> =
    <StakingCurrency<T> as Currency<AccountId<T>>>::NegativeImbalance;

pub type ExposureT<T> = Exposure<AccountId<T>, RingBalance<T>, KtonBalance<T>>;
pub type ElectionResultT<T> = ElectionResult<AccountId<T>, RingBalance<T>, KtonBalance<T>>;

type NativeCurrency<T> = <T as Config>::NativeCurrency;
type StakingCurrency<T> = <T as Config>::StakingCurrency;

/// Information regarding the active era (era in used in session).
#[derive(Encode, Decode, RuntimeDebug)]
pub struct ActiveEraInfo {
    /// Index of era.
    pub index: EraIndex,
    /// Moment of start expressed as millisecond from `$UNIX_EPOCH`.
    ///
    /// Start can be none if start hasn't been set for the era yet,
    /// Start is set on the first on_finalize of the era to guarantee usage of `Time`.
    start: Option<u64>,
}

/// Reward points of an era. Used to split era total payout between validators.
///
/// This points will be used to reward validators and their respective nominators.
#[derive(PartialEq, Encode, Decode, Default, RuntimeDebug)]
pub struct EraRewardPoints<AccountId: Ord> {
    /// Total number of points. Equals the sum of reward points for each validator.
    total: RewardPoint,
    /// The reward points earned by a given validator.
    individual: BTreeMap<AccountId, RewardPoint>,
}

/// Indicates the initial status of the staker.
#[derive(RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum StakerStatus<AccountId> {
    /// Chilling.
    Idle,
    /// Declared desire in validating or already participating in it.
    Validator,
    /// Nominating for a group of other stakers.
    Nominator(Vec<AccountId>),
}

/// A destination account for payment.
#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, RuntimeDebug)]
pub enum RewardDestination<AccountId> {
    /// Pay into the stash account, increasing the amount at stake accordingly.
    Staked,
    /// Pay into the stash account, not increasing the amount at stake.
    Stash,
    /// Pay into the controller account.
    Controller,
    /// Pay into a specified account.
    Account(AccountId),
}

impl<AccountId> Default for RewardDestination<AccountId> {
    fn default() -> Self {
        RewardDestination::Staked
    }
}

/// Preference of what happens regarding validation.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct ValidatorPrefs {
    /// Reward that validator takes up-front; only the rest is split between themselves and
    /// nominators.
    #[codec(compact)]
    pub commission: Perbill,
    /// Whether or not this validator is accepting more nominations. If `true`, then no nominator
    /// who is not already nominating this validator may nominate them. By default, validators
    /// are accepting nominations.
    pub blocked: bool,
}

impl Default for ValidatorPrefs {
    fn default() -> Self {
        ValidatorPrefs {
            commission: Default::default(),
            blocked: false,
        }
    }
}

/// Just a Balance/BlockNumber tuple to encode when a chunk of funds will be unlocked.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct UnlockChunk<Balance: HasCompact> {
    /// Amount of funds to be unlocked.
    #[codec(compact)]
    value: Balance,
    /// Era number at which point it'll be unlocked.
    #[codec(compact)]
    era: EraIndex,
}

/// The ledger of a (bonded) stash.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct StakingLedger<AccountId, Balance: HasCompact> {
    /// The stash account whose balance is actually locked and at stake.
    pub stash: AccountId,
    /// The total amount of the stash's balance that we are currently accounting for.
    /// It's just `active` plus all the `unlocking` balances.
    #[codec(compact)]
    pub total: Balance,
    /// The total amount of the stash's balance that will be at stake in any forthcoming
    /// rounds.
    #[codec(compact)]
    pub active: Balance,
    /// Any balance that is becoming free, which may eventually be transferred out
    /// of the stash (assuming it doesn't get slashed first).
    pub unlocking: Vec<UnlockChunk<Balance>>,
    /// List of eras for which the stakers behind a validator have claimed rewards. Only updated
    /// for validators.
    pub claimed_rewards: Vec<EraIndex>,
}

impl<AccountId, Balance: HasCompact + Copy + Saturating + AtLeast32BitUnsigned>
    StakingLedger<AccountId, Balance>
{
    /// Remove entries from `unlocking` that are sufficiently old and reduce the
    /// total by the sum of their balances.
    fn consolidate_unlocked(self, current_era: EraIndex) -> Self {
        let mut total = self.total;
        let unlocking = self
            .unlocking
            .into_iter()
            .filter(|chunk| {
                if chunk.era > current_era {
                    true
                } else {
                    total = total.saturating_sub(chunk.value);
                    false
                }
            })
            .collect();

        Self {
            stash: self.stash,
            total,
            active: self.active,
            unlocking,
            claimed_rewards: self.claimed_rewards,
        }
    }

    /// Re-bond funds that were scheduled for unlocking.
    fn rebond(mut self, value: Balance) -> Self {
        let mut unlocking_balance: Balance = Zero::zero();

        while let Some(last) = self.unlocking.last_mut() {
            if unlocking_balance + last.value <= value {
                unlocking_balance += last.value;
                self.active += last.value;
                self.unlocking.pop();
            } else {
                let diff = value - unlocking_balance;

                unlocking_balance += diff;
                self.active += diff;
                last.value -= diff;
            }

            if unlocking_balance >= value {
                break;
            }
        }

        self
    }
}

impl<AccountId, Balance> StakingLedger<AccountId, Balance>
where
    Balance: AtLeast32BitUnsigned + Saturating + Copy,
{
    /// Slash the validator for a given amount of balance. This can grow the value
    /// of the slash in the case that the validator has less than `minimum_balance`
    /// active funds. Returns the amount of funds actually slashed.
    ///
    /// Slashes from `active` funds first, and then `unlocking`, starting with the
    /// chunks that are closest to unlocking.
    fn slash(&mut self, mut value: Balance, minimum_balance: Balance) -> Balance {
        let pre_total = self.total;
        let total = &mut self.total;
        let active = &mut self.active;

        let slash_out_of =
            |total_remaining: &mut Balance, target: &mut Balance, value: &mut Balance| {
                let mut slash_from_target = (*value).min(*target);

                if !slash_from_target.is_zero() {
                    *target -= slash_from_target;

                    // don't leave a dust balance in the staking system.
                    if *target <= minimum_balance {
                        slash_from_target += *target;
                        *value += sp_std::mem::replace(target, Zero::zero());
                    }

                    *total_remaining = total_remaining.saturating_sub(slash_from_target);
                    *value -= slash_from_target;
                }
            };

        slash_out_of(total, active, &mut value);

        let i = self
            .unlocking
            .iter_mut()
            .map(|chunk| {
                slash_out_of(total, &mut chunk.value, &mut value);
                chunk.value
            })
            .take_while(|value| value.is_zero()) // take all fully-consumed chunks out.
            .count();

        // kill all drained chunks.
        let _ = self.unlocking.drain(..i);

        pre_total.saturating_sub(*total)
    }
}

/// A record of the nominations made by a specific account.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct Nominations<AccountId> {
    /// The targets of nomination.
    pub targets: Vec<AccountId>,
    /// The era the nominations were submitted.
    ///
    /// Except for initial nominations which are considered submitted at era 0.
    pub submitted_in: EraIndex,
    /// Whether the nominations have been suppressed. This can happen due to slashing of the
    /// validators, or other events that might invalidate the nomination.
    ///
    /// NOTE: this for future proofing and is thus far not used.
    pub suppressed: bool,
}

/// The amount of exposure (to slashing) than an individual nominator has.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, RuntimeDebug)]
pub struct IndividualExposure<AccountId, Balance: HasCompact> {
    /// The stash account of the nominator in question.
    pub who: AccountId,
    /// Amount of funds exposed.
    #[codec(compact)]
    pub value: Balance,
}

/// A snapshot of the stake backing a single validator in the system.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct Exposure<AccountId, Balance: HasCompact> {
    /// The total balance backing this validator.
    #[codec(compact)]
    pub total: Balance,
    /// The validator's own stash that is exposed.
    #[codec(compact)]
    pub own: Balance,
    /// The portions of nominators stashes that are exposed.
    pub others: Vec<IndividualExposure<AccountId, Balance>>,
}

/// A pending slash record. The value of the slash has been computed but not applied yet,
/// rather deferred for several eras.
#[derive(Encode, Decode, Default, RuntimeDebug)]
pub struct UnappliedSlash<AccountId, Balance: HasCompact> {
    /// The stash ID of the offending validator.
    validator: AccountId,
    /// The validator's own slash.
    own: Balance,
    /// All other slashed stakers and amounts.
    others: Vec<(AccountId, Balance)>,
    /// Reporters of the offence; bounty payout recipients.
    reporters: Vec<AccountId>,
    /// The amount of payout.
    payout: Balance,
}

/// Indicate how an election round was computed.
#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode, RuntimeDebug)]
pub enum ElectionCompute {
    /// Result was forcefully computed on chain at the end of the session.
    OnChain,
    /// Result was submitted and accepted to the chain via a signed transaction.
    Signed,
    /// Result was submitted and accepted to the chain via an unsigned transaction (by an
    /// authority).
    Unsigned,
}

/// The result of an election round.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct ElectionResult<AccountId, Balance: HasCompact> {
    /// Flat list of validators who have been elected.
    elected_stashes: Vec<AccountId>,
    /// Flat list of new exposures, to be updated in the [`Exposure`] storage.
    exposures: Vec<(AccountId, Exposure<AccountId, Balance>)>,
    /// Type of the result. This is kept on chain only to track and report the best score's
    /// submission type. An optimisation could remove this.
    compute: ElectionCompute,
}

/// The status of the upcoming (offchain) election.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub enum ElectionStatus<BlockNumber> {
    /// Nothing has and will happen for now. submission window is not open.
    Closed,
    /// The submission window has been open since the contained block number.
    Open(BlockNumber),
}

/// Some indications about the size of the election. This must be submitted with the solution.
///
/// Note that these values must reflect the __total__ number, not only those that are present in the
/// solution. In short, these should be the same size as the size of the values dumped in
/// `SnapshotValidators` and `SnapshotNominators`.
#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode, RuntimeDebug, Default)]
pub struct ElectionSize {
    /// Number of validators in the snapshot of the current election round.
    #[codec(compact)]
    pub validators: ValidatorIndex,
    /// Number of nominators in the snapshot of the current election round.
    #[codec(compact)]
    pub nominators: NominatorIndex,
}

impl<BlockNumber: PartialEq> ElectionStatus<BlockNumber> {
    pub fn is_open_at(&self, n: BlockNumber) -> bool {
        *self == Self::Open(n)
    }

    pub fn is_closed(&self) -> bool {
        match self {
            Self::Closed => true,
            _ => false,
        }
    }

    pub fn is_open(&self) -> bool {
        !self.is_closed()
    }
}

impl<BlockNumber> Default for ElectionStatus<BlockNumber> {
    fn default() -> Self {
        Self::Closed
    }
}

/// Means for interacting with a specialized version of the `session` trait.
///
/// This is needed because `Staking` sets the `ValidatorIdOf` of the `pallet_session::Config`
pub trait SessionInterface<AccountId>: frame_system::Config {
    /// Disable a given validator by stash ID.
    ///
    /// Returns `true` if new era should be forced at the end of this session.
    /// This allows preventing a situation where there is too many validators
    /// disabled and block production stalls.
    fn disable_validator(validator: &AccountId) -> Result<bool, ()>;
    /// Get the validators from session.
    fn validators() -> Vec<AccountId>;
    /// Prune historical session tries up to but not including the given index.
    fn prune_historical_up_to(up_to: SessionIndex);
}

impl<T: Config> SessionInterface<<T as frame_system::Config>::AccountId> for T
where
    T: pallet_session::Config<ValidatorId = <T as frame_system::Config>::AccountId>,
    T: pallet_session::historical::Config<
        FullIdentification = Exposure<<T as frame_system::Config>::AccountId, BalanceOf<T>>,
        FullIdentificationOf = ExposureOf<T>,
    >,
    T::SessionHandler: pallet_session::SessionHandler<<T as frame_system::Config>::AccountId>,
    T::SessionManager: pallet_session::SessionManager<<T as frame_system::Config>::AccountId>,
    T::ValidatorIdOf: Convert<
        <T as frame_system::Config>::AccountId,
        Option<<T as frame_system::Config>::AccountId>,
    >,
{
    fn disable_validator(validator: &<T as frame_system::Config>::AccountId) -> Result<bool, ()> {
        <pallet_session::Module<T>>::disable(validator)
    }

    fn validators() -> Vec<<T as frame_system::Config>::AccountId> {
        <pallet_session::Module<T>>::validators()
    }

    fn prune_historical_up_to(up_to: SessionIndex) {
        <pallet_session::historical::Module<T>>::prune_up_to(up_to);
    }
}

pub trait Config: frame_system::Config + SendTransactionTypes<Call<Self>> {
    /// The staking balance.
    type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;

    /// Time used for computing era duration.
    ///
    /// It is guaranteed to start being called from the first `on_finalize`. Thus value at genesis
    /// is not used.
    type UnixTime: UnixTime;

    /// Convert a balance into a number used for election calculation. This must fit into a `u64`
    /// but is allowed to be sensibly lossy. The `u64` is used to communicate with the
    /// [`sp_npos_elections`] crate which accepts u64 numbers and does operations in 128.
    /// Consequently, the backward convert is used convert the u128s from sp-elections back to a
    /// [`BalanceOf`].
    type CurrencyToVote: CurrencyToVote<BalanceOf<Self>>;

    /// Tokens have been minted and are unused for validator-reward.
    /// See [Era payout](./index.html#era-payout).
    type RewardRemainder: OnUnbalanced<NegativeImbalanceOf<Self>>;

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;

    /// Handler for the unbalanced reduction when slashing a staker.
    type Slash: OnUnbalanced<NegativeImbalanceOf<Self>>;

    /// Handler for the unbalanced increment when rewarding a staker.
    type Reward: OnUnbalanced<PositiveImbalanceOf<Self>>;

    /// Number of sessions per era.
    type SessionsPerEra: Get<SessionIndex>;

    /// Number of eras that staked funds must remain bonded for.
    type BondingDuration: Get<EraIndex>;

    /// Number of eras that slashes are deferred by, after computation.
    ///
    /// This should be less than the bonding duration. Set to 0 if slashes
    /// should be applied immediately, without opportunity for intervention.
    type SlashDeferDuration: Get<EraIndex>;

    /// The origin which can cancel a deferred slash. Root can always do this.
    type SlashCancelOrigin: EnsureOrigin<Self::Origin>;

    /// Interface for interacting with a session module.
    type SessionInterface: self::SessionInterface<Self::AccountId>;

    /// The NPoS reward curve used to define yearly inflation.
    /// See [Era payout](./index.html#era-payout).
    type RewardCurve: Get<&'static PiecewiseLinear<'static>>;

    /// Something that can estimate the next session change, accurately or as a best effort guess.
    type NextNewSession: EstimateNextNewSession<Self::BlockNumber>;

    /// The number of blocks before the end of the era from which election submissions are allowed.
    ///
    /// Setting this to zero will disable the offchain compute and only on-chain seq-phragmen will
    /// be used.
    ///
    /// This is bounded by being within the last session. Hence, setting it to a value more than the
    /// length of a session will be pointless.
    type ElectionLookahead: Get<Self::BlockNumber>;

    /// The overarching call type.
    type Call: Dispatchable + From<Call<Self>> + IsSubType<Call<Self>> + Clone;

    /// Maximum number of balancing iterations to run in the offchain submission.
    ///
    /// If set to 0, balance_solution will not be executed at all.
    type MaxIterations: Get<u32>;

    /// The threshold of improvement that should be provided for a new solution to be accepted.
    type MinSolutionScoreBump: Get<Perbill>;

    /// The maximum number of nominators rewarded for each validator.
    ///
    /// For each validator only the `$MaxNominatorRewardedPerValidator` biggest stakers can claim
    /// their reward. This used to limit the i/o cost for the nominator payout.
    type MaxNominatorRewardedPerValidator: Get<u32>;

    /// A configuration for base priority of unsigned transactions.
    ///
    /// This is exposed so that it can be tuned for particular runtime, when
    /// multiple pallets send unsigned transactions.
    type UnsignedPriority: Get<TransactionPriority>;

    /// Maximum weight that the unsigned transaction can have.
    ///
    /// Chose this value with care. On one hand, it should be as high as possible, so the solution
    /// can contain as many nominators/validators as possible. On the other hand, it should be small
    /// enough to fit in the block.
    type OffchainSolutionWeightLimit: Get<Weight>;

    /// Weight information for extrinsics in this pallet.
    type WeightInfo: WeightInfo;
}

/// Mode of era-forcing.
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Forcing {
    /// Not forcing anything - just let whatever happen.
    NotForcing,
    /// Force a new era, then reset to `NotForcing` as soon as it is done.
    ForceNew,
    /// Avoid a new era indefinitely.
    ForceNone,
    /// Force a new era at the end of all sessions indefinitely.
    ForceAlways,
}

impl Default for Forcing {
    fn default() -> Self {
        Forcing::NotForcing
    }
}

// A value placed in storage that represents the current version of the Staking storage. This value
// is used by the `on_runtime_upgrade` logic to determine whether we run storage migration logic.
// This should match directly with the semantic versions of the Rust crate.
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug)]
enum Releases {
    V1_0_0Ancient,
    V2_0_0,
    V3_0_0,
    V4_0_0,
    V5_0_0,
}

impl Default for Releases {
    fn default() -> Self {
        Releases::V5_0_0
    }
}

decl_storage! {
    trait Store for Module<T: Config> as Staking {
        /// Number of eras to keep in history.
        ///
        /// Information is kept for eras in `[current_era - history_depth; current_era]`.
        ///
        /// Must be more than the number of eras delayed by session otherwise. I.e. active era must
        /// always be in history. I.e. `active_era > current_era - history_depth` must be
        /// guaranteed.
        HistoryDepth get(fn history_depth) config(): u32 = 84;

        /// The ideal number of staking participants.
        pub ValidatorCount get(fn validator_count) config(): u32;

        /// Minimum number of staking participants before emergency conditions are imposed.
        pub MinimumValidatorCount get(fn minimum_validator_count) config(): u32;

        /// Any validators that may never be slashed or forcibly kicked. It's a Vec since they're
        /// easy to initialize and the performance hit is minimal (we expect no more than four
        /// invulnerables) and restricted to testnets.
        pub Invulnerables get(fn invulnerables) config(): Vec<T::AccountId>;

        /// Map from all locked "stash" accounts to the controller account.
        pub Bonded get(fn bonded): map hasher(twox_64_concat) T::AccountId => Option<T::AccountId>;

        /// Map from all (unlocked) "controller" accounts to the info regarding the staking.
        pub Ledger get(fn ledger):
            map hasher(blake2_128_concat) T::AccountId
            => Option<StakingLedger<T::AccountId, BalanceOf<T>>>;

        /// Where the reward payment should be made. Keyed by stash.
        pub Payee get(fn payee): map hasher(twox_64_concat) T::AccountId => RewardDestination<T::AccountId>;

        /// The map from (wannabe) validator stash key to the preferences of that validator.
        pub Validators get(fn validators):
            map hasher(twox_64_concat) T::AccountId => ValidatorPrefs;

        /// The map from nominator stash key to the set of stash keys of all validators to nominate.
        pub Nominators get(fn nominators):
            map hasher(twox_64_concat) T::AccountId => Option<Nominations<T::AccountId>>;

        /// The current era index.
        ///
        /// This is the latest planned era, depending on how the Session pallet queues the validator
        /// set, it might be active or not.
        pub CurrentEra get(fn current_era): Option<EraIndex>;

        /// The active era information, it holds index and start.
        ///
        /// The active era is the era being currently rewarded. Validator set of this era must be
        /// equal to [`SessionInterface::validators`].
        pub ActiveEra get(fn active_era): Option<ActiveEraInfo>;

        /// The session index at which the era start for the last `HISTORY_DEPTH` eras.
        ///
        /// Note: This tracks the starting session (i.e. session index when era start being active)
        /// for the eras in `[CurrentEra - HISTORY_DEPTH, CurrentEra]`.
        pub ErasStartSessionIndex get(fn eras_start_session_index):
            map hasher(twox_64_concat) EraIndex => Option<SessionIndex>;

        /// Exposure of validator at era.
        ///
        /// This is keyed first by the era index to allow bulk deletion and then the stash account.
        ///
        /// Is it removed after `HISTORY_DEPTH` eras.
        /// If stakers hasn't been set or has been removed then empty exposure is returned.
        pub ErasStakers get(fn eras_stakers):
            double_map hasher(twox_64_concat) EraIndex, hasher(twox_64_concat) T::AccountId
            => Exposure<T::AccountId, BalanceOf<T>>;

        /// Clipped Exposure of validator at era.
        ///
        /// This is similar to [`ErasStakers`] but number of nominators exposed is reduced to the
        /// `T::MaxNominatorRewardedPerValidator` biggest stakers.
        /// (Note: the field `total` and `own` of the exposure remains unchanged).
        /// This is used to limit the i/o cost for the nominator payout.
        ///
        /// This is keyed fist by the era index to allow bulk deletion and then the stash account.
        ///
        /// Is it removed after `HISTORY_DEPTH` eras.
        /// If stakers hasn't been set or has been removed then empty exposure is returned.
        pub ErasStakersClipped get(fn eras_stakers_clipped):
            double_map hasher(twox_64_concat) EraIndex, hasher(twox_64_concat) T::AccountId
            => Exposure<T::AccountId, BalanceOf<T>>;

        /// Similar to `ErasStakers`, this holds the preferences of validators.
        ///
        /// This is keyed first by the era index to allow bulk deletion and then the stash account.
        ///
        /// Is it removed after `HISTORY_DEPTH` eras.
        // If prefs hasn't been set or has been removed then 0 commission is returned.
        pub ErasValidatorPrefs get(fn eras_validator_prefs):
            double_map hasher(twox_64_concat) EraIndex, hasher(twox_64_concat) T::AccountId
            => ValidatorPrefs;

        /// The total validator era payout for the last `HISTORY_DEPTH` eras.
        ///
        /// Eras that haven't finished yet or has been removed doesn't have reward.
        pub ErasValidatorReward get(fn eras_validator_reward):
            map hasher(twox_64_concat) EraIndex => Option<BalanceOf<T>>;

        /// Rewards for the last `HISTORY_DEPTH` eras.
        /// If reward hasn't been set or has been removed then 0 reward is returned.
        pub ErasRewardPoints get(fn eras_reward_points):
            map hasher(twox_64_concat) EraIndex => EraRewardPoints<T::AccountId>;

        /// The total amount staked for the last `HISTORY_DEPTH` eras.
        /// If total hasn't been set or has been removed then 0 stake is returned.
        pub ErasTotalStake get(fn eras_total_stake):
            map hasher(twox_64_concat) EraIndex => BalanceOf<T>;

        /// Mode of era forcing.
        pub ForceEra get(fn force_era) config(): Forcing;

        /// The percentage of the slash that is distributed to reporters.
        ///
        /// The rest of the slashed value is handled by the `Slash`.
        pub SlashRewardFraction get(fn slash_reward_fraction) config(): Perbill;

        /// The amount of currency given to reporters of a slash event which was
        /// canceled by extraordinary circumstances (e.g. governance).
        pub CanceledSlashPayout get(fn canceled_payout) config(): BalanceOf<T>;

        /// All unapplied slashes that are queued for later.
        pub UnappliedSlashes:
            map hasher(twox_64_concat) EraIndex => Vec<UnappliedSlash<T::AccountId, BalanceOf<T>>>;

        /// A mapping from still-bonded eras to the first session index of that era.
        ///
        /// Must contains information for eras for the range:
        /// `[active_era - bounding_duration; active_era]`
        BondedEras: Vec<(EraIndex, SessionIndex)>;

        /// All slashing events on validators, mapped by era to the highest slash proportion
        /// and slash value of the era.
        ValidatorSlashInEra:
            double_map hasher(twox_64_concat) EraIndex, hasher(twox_64_concat) T::AccountId
            => Option<(Perbill, BalanceOf<T>)>;

        /// All slashing events on nominators, mapped by era to the highest slash value of the era.
        NominatorSlashInEra:
            double_map hasher(twox_64_concat) EraIndex, hasher(twox_64_concat) T::AccountId
            => Option<BalanceOf<T>>;

        /// Slashing spans for stash accounts.
        SlashingSpans get(fn slashing_spans): map hasher(twox_64_concat) T::AccountId => Option<slashing::SlashingSpans>;

        /// Records information about the maximum slash of a stash within a slashing span,
        /// as well as how much reward has been paid out.
        SpanSlash:
            map hasher(twox_64_concat) (T::AccountId, slashing::SpanIndex)
            => slashing::SpanRecord<BalanceOf<T>>;

        /// The earliest era for which we have a pending, unapplied slash.
        EarliestUnappliedSlash: Option<EraIndex>;

        /// Snapshot of validators at the beginning of the current election window. This should only
        /// have a value when [`EraElectionStatus`] == `ElectionStatus::Open(_)`.
        pub SnapshotValidators get(fn snapshot_validators): Option<Vec<T::AccountId>>;

        /// Snapshot of nominators at the beginning of the current election window. This should only
        /// have a value when [`EraElectionStatus`] == `ElectionStatus::Open(_)`.
        pub SnapshotNominators get(fn snapshot_nominators): Option<Vec<T::AccountId>>;

        /// The next validator set. At the end of an era, if this is available (potentially from the
        /// result of an offchain worker), it is immediately used. Otherwise, the on-chain election
        /// is executed.
        pub QueuedElected get(fn queued_elected): Option<ElectionResult<T::AccountId, BalanceOf<T>>>;

        /// The score of the current [`QueuedElected`].
        pub QueuedScore get(fn queued_score): Option<ElectionScore>;

        /// Flag to control the execution of the offchain election. When `Open(_)`, we accept
        /// solutions to be submitted.
        pub EraElectionStatus get(fn era_election_status): ElectionStatus<T::BlockNumber>;

        /// True if the current **planned** session is final. Note that this does not take era
        /// forcing into account.
        pub IsCurrentSessionFinal get(fn is_current_session_final): bool = false;

        /// True if network has been upgraded to this version.
        /// Storage version of the pallet.
        ///
        /// This is set to v5.0.0 for new networks.
        StorageVersion build(|_: &GenesisConfig<T>| Releases::V5_0_0): Releases;
    }
    add_extra_genesis {
        config(stakers):
            Vec<(T::AccountId, T::AccountId, BalanceOf<T>, StakerStatus<T::AccountId>)>;
        build(|config: &GenesisConfig<T>| {
            for &(ref stash, ref controller, balance, ref status) in &config.stakers {
                assert!(
                    T::Currency::free_balance(&stash) >= balance,
                    "Stash does not have enough balance to bond."
                );
                let _ = <Module<T>>::bond(
                    T::Origin::from(Some(stash.clone()).into()),
                    T::Lookup::unlookup(controller.clone()),
                    balance,
                    RewardDestination::Staked,
                );
                let _ = match status {
                    StakerStatus::Validator => {
                        <Module<T>>::validate(
                            T::Origin::from(Some(controller.clone()).into()),
                            Default::default(),
                        )
                    },
                    StakerStatus::Nominator(votes) => {
                        <Module<T>>::nominate(
                            T::Origin::from(Some(controller.clone()).into()),
                            votes.iter().map(|l| T::Lookup::unlookup(l.clone())).collect(),
                        )
                    }, _ => Ok(())
                };
            }
        });
    }
}
