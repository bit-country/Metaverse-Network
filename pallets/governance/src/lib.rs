#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use codec::{Decode, Encode};
use frame_support::{
    dispatch::DispatchResult, ensure,
    traits::{Get, Vec},
};
use frame_system::{self as system, ensure_root, ensure_signed};
use primitives::{Balance, CountryId, ProposalId, ReferendumId};
use sp_runtime::{traits::{AccountIdConversion, One, Zero, CheckedDiv, CheckedAdd}, DispatchError, ModuleId, RuntimeDebug};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::vec;
use sp_std::alloc::System;
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

        type CountryInfo: BCCountry<Self::AccountId>;
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
    #[pallet::getter(fn proposals)]
    pub type Proposals<T: Config> = StorageDoubleMap<_, Twox64Concat, CountryId, Twox64Concat, ProposalId, ProposalInfo<T::AccountId,T::BlockNumber,CountryParameter>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn next_proposal)]
    pub type NextProrposalId<T: Config> = StorageValue<_, ProposalId, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn deposit)]
    pub type DepositOf<T: Config> = StorageMap<_, Twox64Concat, ProposalId, (T::AccountId, BalanceOf<T>), OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn referendum_info)]
    pub type ReferendumInfoOf<T: Config> = StorageMap<_, Twox64Concat,  ReferendumId, ReferendumInfo<T::BlockNumber>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn next_referendum)]
    pub type NextReferendumId<T: Config> = StorageValue<_, ReferendumId, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn referendum_parameters)]
    pub type ReferendumParametersOf<T: Config> = StorageMap<_, Twox64Concat, CountryId, ReferendumParameters<T::BlockNumber>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn voting_record)]
    pub type VotingOf<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, VotingRecord<Balance>, ValueQuery>;

   

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    #[pallet::metadata()]
    pub enum Event<T: Config> {
        ReferendumParametersUpdated(CountryId),
        ProposalSubmitted(T::AccountId, CountryId, ProposalId),
        ProposalCancelled(T::AccountId, CountryId, ProposalId),
        ProposalFastTracked(T::AccountId, CountryId, ProposalId),
        ReferendumStarted(ReferendumId, VoteThreshold),
        ReferendumPassed(ReferendumId),
        ReferendumNotPassed(ReferendumId),
        ReferendumCancelled(ReferendumId),
        VoteRecorded(T::AccountId,ReferendumId, Vote<Balance>),
        VoteRemoved(T::AccountId,ReferendumId),

    }

    #[pallet::error]
    pub enum Error<T> {
        AccountNotCountryMember,
        AccountNotCountryOwner,
        ReferendumParametersOutOfScope,
        InsufficientBalance,
        ProposalParametersOutOfScope,
        TooManyProposalParameters,
        ProposalQueueFull,
        ProposalDoesNotExist,
        ProposalIsReferendum,
        NotProposalCreator,
        InsufficientPrivileges,
        ReferendumDoesNotExist
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn update_referendum_parameters(
            origin: OriginFor<T>, 
            country: CountryId,
            new_referendum_parameters: ReferendumParameters<T::BlockNumber>
        ) -> DispatchResultWithPostInfo {
            let from = ensure_signed(origin)?;
            ensure!(T::CountryInfo::check_ownership(&from, &country), Error::<T>::AccountNotCountryOwner);
            <ReferendumParametersOf<T>>::remove(country);
            <ReferendumParametersOf<T>>::insert(country, new_referendum_parameters);
            Self::deposit_event(Event::ReferendumParametersUpdated(country));
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn propose(
            origin: OriginFor<T>, 
            country: CountryId, 
            balance: BalanceOf<T>,
            proposal_parameters: Vec<CountryParameter>,
            proposal_description: Vec<u8>
        ) -> DispatchResultWithPostInfo {
            let from = ensure_signed(origin)?;
            Ok(().into())
        }

        /// Cancel proposal if you are the proposal owner, the proposal exist, and it has not launched as a referendum yet
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn cancel_proposal(
            origin: OriginFor<T>, 
            proposal: ProposalId,
            country: CountryId
        ) -> DispatchResultWithPostInfo {
            let from = ensure_signed(origin)?;
            let proposal_info = Self::proposals(country,proposal).ok_or(Error::<T>::ProposalDoesNotExist)?;
            ensure!(proposal_info.proposed_by == from, Error::<T>::NotProposalCreator);
            let current_block_number = <frame_system::Module<T>>::block_number();
            ensure!(proposal_info.referendum_launch_block < current_block_number, Error::<T>::ProposalIsReferendum);
            <Proposals<T>>::remove(country, proposal);
            Self::deposit_event(Event::ProposalCancelled(from, country, proposal));
            Ok(().into())
        }

        /// Fast track proposal to referendum if you are the country owner and it has not launched as a referendum yet
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn fast_track_proposal(
            origin: OriginFor<T>, 
            proposal: ProposalId,
            country: CountryId
        ) -> DispatchResultWithPostInfo {
            let from = ensure_signed(origin)?;
            let proposal_info = Self::proposals(country,proposal).ok_or(Error::<T>::ProposalDoesNotExist)?;
            let current_block_number = <frame_system::Module<T>>::block_number();
            ensure!(T::CountryInfo::check_ownership(&from, &country), Error::<T>::AccountNotCountryOwner);
            ensure!(proposal_info.referendum_launch_block < current_block_number, Error::<T>::ProposalIsReferendum);
            
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn vote(
            origin: OriginFor<T>, 
            referendum: ReferendumId,
            vote_aye: bool
        ) -> DispatchResultWithPostInfo {
            let from = ensure_signed(origin)?;
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn remove_vote(
            origin: OriginFor<T>, 
            referendum:  ReferendumId,
        ) -> DispatchResultWithPostInfo {
            let from = ensure_signed(origin)?;
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn emergency_cancel_referendum(
            origin: OriginFor<T>, 
            referendum:  ReferendumId,
        ) -> DispatchResultWithPostInfo {
            let from = ensure_signed(origin)?;
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        fn update_referendum_tally(referendum: ReferendumId, vote: Vote<Balance>, is_vote_removed: bool) -> DispatchResult {
            
            if is_vote_removed {

            }
            else {

            }
            
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


    