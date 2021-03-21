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
