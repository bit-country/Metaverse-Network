#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use codec::{Decode, Encode};
use frame_support::{
    dispatch::DispatchResult, ensure,
    traits::{Get, Vec},
};
use frame_system::{self as system, ensure_root, ensure_signed};
use primitives::{Balance, CountryId};
use sp_runtime::{traits::{AccountIdConversion, One, Zero, CheckedDiv, CheckedAdd}, DispatchError, ModuleId, RuntimeDebug, FixedPointNumber};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::vec;
use bc_country::{BCCountry, Country};
use frame_support::traits::{Currency, ReservableCurrency, LockableCurrency};

mod types;

pub use types::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::OriginFor;
    use super::*;

    pub(crate) type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        /// Initialization
        fn on_initialize(now: T::BlockNumber) -> Weight {
            
        }

        /// Finalization
        fn on_finalize(now: T::BlockNumber) -> Weight {
            
        }
    }

    #[pallet::storage]
    #[pallet::getter(fn current_session)]
    pub type NextProrposalId<T: Config> = StorageValue<_, Twox64Concat, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn current_session)]
    pub type NextrReferendumlId<T: Config> = StorageValue<_, Twox64Concat, u64, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    #[pallet::metadata()]
    pub enum Event<T: Config> {
        /// A bid is placed. [auction_id, bidder, bidding_amount]
        ProposalSubmitted(T::AccountId, CountryId, u64),
        ProposalCancelled(T::AccountId, CountryId, u64),
        ProposalFastTracked(T::AccountId, CountryId, u64),
        ReferendumStarted(ReferendumId, VoteThreshold),
        ReferendumPassed(ReferendumId),
        ReferendumNotPassed(ReferendumId),
        ReferendumCancelled(ReferendumId),
        VoteRecorded(T::AccountId,ReferendumId, Vote),
        VoteRemoved(T::AccountId,ReferendumId),

    }

    #[pallet::error]
    pub enum Error<T> {
        AccountNotCountryMember,
        InsufficientDeposit,
        ProposalParametersOutOfScope,
        TooManyProposalParameters,
        ProposalQueueFull,
        NotProposalCreator,
        InsufficientPrivileges
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn propose(
            origin: OriginFor<T>, 
            country: ConuntryId, 
            balance: Balance,
            proposal_info: Vec<u8>
        ) -> DispatchResultWithPostInfo {
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn cancel_proposal(
            origin: OriginFor<T>, 
            proposal: u64
        ) -> DispatchResultWithPostInfo {
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn fast_track_proposal(
            origin: OriginFor<T>, 
            proposal: u64
        ) -> DispatchResultWithPostInfo {
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn vote(
            origin: OriginFor<T>, 
            referendum: u64,
            vote_aye: bool
        ) -> DispatchResultWithPostInfo {
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn remove_vote(
            origin: OriginFor<T>, 
            referendum: u64,
        ) -> DispatchResultWithPostInfo {
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn emergency_cancel_referendum(
            origin: OriginFor<T>, 
            referendum: u64,
        ) -> DispatchResultWithPostInfo {
        }
    }
}

impl<T: Config> Pallet<T> {
    fn update_referendum_tally(referendum: u64, vote: Vote, remove: bool) -> DispatchResult {
    }

    fn enact_proposal(proposal: u64) -> DispatchResult {
    }

    fn update_country_parameters_(new_parameters: Vec<u8>) -> DispatchResult {
    }

}
    