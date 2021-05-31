#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use codec::{Decode, Encode};
use frame_support::{
    dispatch::DispatchResult, ensure,
    traits::{Get, Vec},
};
use frame_system::{self as system, ensure_root, ensure_signed};
use primitives::{Balance, CountryId, ProposalId, ReferendumId};
use sp_runtime::{traits::{AccountIdConversion, One, Zero, CheckedDiv, CheckedAdd}, DispatchError, ModuleId, RuntimeDebug, FixedPointNumber};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::vec;
use bc_country::{BCCountry, Country};
use frame_support::traits::{Currency, ReservableCurrency, LockableCurrency};

mod types;

pub use types::*;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod mock;

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
        
        #[pallet::constant]
        type DefaultVotingPeriod: Get<Self::BlockNumber>;

        #[pallet::constant]
        type DefaultEnactmentPeriod: Get<Self::BlockNumber>;
       
        #[pallet::constant]
        type DefaultProposalLaunchPeriod: Get<Self::BlockNumber>;
        
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        /// Initialization
        fn on_initialize(now: T::BlockNumber) -> Weight {
            0
        }

        /// Finalization
        fn on_finalize(now: T::BlockNumber)  {
            
        }
    }

    #[pallet::storage]
    #[pallet::getter(fn current_proposal)]
    pub type NextProrposalId<T: Config> = StorageValue<_, ProposalId, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn current_referendum)]
    pub type NextrReferendumlId<T: Config> = StorageValue<_, ReferendumId, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    #[pallet::metadata()]
    pub enum Event<T: Config> {
        /// A bid is placed. [auction_id, bidder, bidding_amount]
        ProposalSubmitted(T::AccountId, CountryId, ProposalId),
        ProposalCancelled(T::AccountId, CountryId, ProposalId),
        ProposalFastTracked(T::AccountId, CountryId, ProposalId),
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
        AccountNotCountryOwner,
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
            country: CountryId, 
            balance: Balance,
            proposal_info: Vec<u8>
        ) -> DispatchResultWithPostInfo {
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn cancel_proposal(
            origin: OriginFor<T>, 
            proposal: ProposalId
        ) -> DispatchResultWithPostInfo {
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn fast_track_proposal(
            origin: OriginFor<T>, 
            proposal: ProposalId
        ) -> DispatchResultWithPostInfo {
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn vote(
            origin: OriginFor<T>, 
            referendum: ReferendumId,
            vote_aye: bool
        ) -> DispatchResultWithPostInfo {
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn remove_vote(
            origin: OriginFor<T>, 
            referendum:  ReferendumId,
        ) -> DispatchResultWithPostInfo {
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn emergency_cancel_referendum(
            origin: OriginFor<T>, 
            referendum:  ReferendumId,
        ) -> DispatchResultWithPostInfo {
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        fn update_referendum_tally(referendum: ReferendumId, vote: Vote, remove: bool) -> DispatchResult {
            Ok(())
        }
    
        fn enact_proposal(proposal: ProposalId) -> DispatchResult {
            Ok(())
        }
    
        fn update_country_parameters(new_parameters: Vec<u8>) -> DispatchResult {
            Ok(())
        }
    
    }
}


    