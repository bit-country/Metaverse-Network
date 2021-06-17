#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use codec::{Decode, Encode};
use frame_support::{
    dispatch::DispatchResult, ensure,
    traits::{Get, Vec}, IterableStorageDoubleMap,
};
use frame_system::{self as system, ensure_root, ensure_signed};
use primitives::{Balance, CountryId, ProposalId, ReferendumId};
use sp_runtime::{traits::{AccountIdConversion, Zero, CheckedDiv, CheckedAdd}, DispatchError, ModuleId, RuntimeDebug};

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

        #[pallet::constant]
        type OneBlock: Get<Self::BlockNumber>;

        #[pallet::constant]
        type DefaultMaxParametersPerProposal: Get<u8>;
        
        #[pallet::constant]
        type DefaultMaxProposalsPerCountry: Get<u8>;

        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        type CountryInfo: BCCountry<Self::AccountId>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        /// Initialisation
        fn on_initialize(now: T::BlockNumber) -> Weight {
            for (country_id,proposal_id, proposal_info) in <Proposals<T>>::iter() {
                if proposal_info.referendum_launch_block == now {
                    Self::start_referendum(country_id, proposal_id,now);
                }
            }
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
    pub type NextProposalId<T: Config> = StorageValue<_, ProposalId, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn proposals_per_country)]
    pub type TotalProposalsPerCountry<T: Config> = StorageMap<_, Twox64Concat, CountryId, u8, ValueQuery>;

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
    #[pallet::getter(fn referendum_jury)]
    pub type ReferendumJuryOf<T: Config> = StorageMap<_, Twox64Concat,  CountryId, T::AccountId, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn voting_record)]
    pub type VotingOf<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, VotingRecord, ValueQuery>;

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
        VoteRecorded(T::AccountId,ReferendumId, bool),
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
        ReferendumDoesNotExist,
        ReferendumIsOver,
        JuryNotSelected,
        DepositNotFound,
        ProposalIdOverflow,
        ReferendumIdOverflow,
        ProposalQueueOverflow,
        TallyOverflow,
        AccountHasNotVoted,
        AccountAlreadyVoted

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
            ensure!(Self::is_member(from.clone(),country),  Error::<T>::AccountNotCountryMember);
            ensure!(T::Currency::free_balance(&from) >= balance, Error::<T>::InsufficientBalance);
            
            let current_block = <frame_system::Module<T>>::block_number();
            let mut launch_block: T::BlockNumber = current_block;
            match Self::referendum_parameters(country) {
                Some(country_referendum_params) => {
                    ensure!(Self::proposals_per_country(country) < country_referendum_params.max_proposals_per_country, Error::<T>::ProposalQueueFull);
                    ensure!(proposal_parameters.len() <= country_referendum_params.max_params_per_proposal.into(), Error::<T>::TooManyProposalParameters);   
                    if country_referendum_params.min_proposal_launch_period.is_zero() {
                        launch_block += country_referendum_params.min_proposal_launch_period;
                    }
                    else {
                        launch_block += T::DefaultProposalLaunchPeriod::get();
                    }
                },
                None => {
                    ensure!(Self::proposals_per_country(country) < T::DefaultMaxProposalsPerCountry::get(), Error::<T>::ProposalQueueFull);
                    ensure!(proposal_parameters.len() <= T::DefaultMaxParametersPerProposal::get().into(), Error::<T>::TooManyProposalParameters);   
                    launch_block += T::DefaultProposalLaunchPeriod::get();
                }, 
            }
            let proposal_info = ProposalInfo {
                proposed_by: from.clone(),
                parameters: proposal_parameters,
                description: proposal_description,
                referendum_launch_block: launch_block,
            };
            let proposal_id = Self::get_next_proposal_id()?;         
            <Proposals<T>>::insert(country, proposal_id, proposal_info);
            Self::update_proposals_per_country_number(country,true);
            T::Currency::reserve(&from, balance);
            <DepositOf<T>>::insert(proposal_id, (&from, balance));
            Self::deposit_event(Event::ProposalSubmitted(from, country, proposal_id)); 
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
            ensure!(proposal_info.referendum_launch_block > <frame_system::Module<T>>::block_number(), Error::<T>::ProposalIsReferendum);
            <Proposals<T>>::remove(country, proposal);
            Self::update_proposals_per_country_number(country,false);
            T::Currency::unreserve(&from, Self::deposit(proposal).ok_or(Error::<T>::DepositNotFound)?.1); 
            <DepositOf<T>>::remove(proposal);
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
            let mut proposal_info = Self::proposals(country,proposal).ok_or(Error::<T>::ProposalDoesNotExist)?;
            ensure!(T::CountryInfo::check_ownership(&from, &country), Error::<T>::AccountNotCountryOwner);
            let current_block_number = <frame_system::Module<T>>::block_number();
            ensure!(proposal_info.referendum_launch_block > current_block_number, Error::<T>::ProposalIsReferendum);
            proposal_info.referendum_launch_block = current_block_number + T::OneBlock::get();
            <Proposals<T>>::remove(country,proposal);
            <Proposals<T>>::insert(country,proposal,proposal_info);
            Self::deposit_event(Event::ProposalFastTracked(from.clone(), country, proposal));
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn vote(
            origin: OriginFor<T>, 
            referendum: ReferendumId,
            vote_aye: bool
        ) -> DispatchResultWithPostInfo {
            let from = ensure_signed(origin)?;
            let mut status = Self::referendum_status(referendum)?;
            ensure!(Self::is_member(from.clone(),status.country), Error::<T>::AccountNotCountryMember);
            <VotingOf<T>>::try_mutate(from.clone(),|mut voting_record| -> DispatchResultWithPostInfo {
                let mut votes = &mut voting_record.votes;
                match votes.binary_search_by_key(&referendum, |i| i.0) {
                    Ok(i) => Err(Error::<T>::AccountAlreadyVoted.into()),
                    Err(i) => {
                        let vote = Vote {
                            aye: vote_aye,
                            //balance: T::Currency::free_balance(&from)
                        };
                        
                        votes.insert(i, (referendum,vote.clone()));

                        <ReferendumInfoOf<T>>::try_mutate(referendum,|referendum_info| -> DispatchResultWithPostInfo {
                            status.tally.add(vote).ok_or(Error::<T>::TallyOverflow)?;
                            *referendum_info = Some(ReferendumInfo::Ongoing(status));
                            
                            Ok(().into())
                        });
                        Self::deposit_event(Event::VoteRecorded(from, referendum, vote_aye));
                        Ok(().into())
                    }
                }
            })
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn remove_vote(
            origin: OriginFor<T>, 
            referendum:  ReferendumId,
        ) -> DispatchResultWithPostInfo {
            let from = ensure_signed(origin)?;
            let mut status = Self::referendum_status(referendum)?;
            <VotingOf<T>>::try_mutate(from.clone(),|mut voting_record| -> DispatchResultWithPostInfo {
                let mut votes = &mut voting_record.votes;
                match votes.binary_search_by_key(&referendum, |i| i.0) {
                    Ok(i) => {
                        let vote =  votes.remove(i).1;
                        <ReferendumInfoOf<T>>::try_mutate(referendum,|referendum_info| -> DispatchResultWithPostInfo {
                            status.tally.remove(vote).ok_or(Error::<T>::TallyOverflow)?;
                            *referendum_info = Some(ReferendumInfo::Ongoing(status));
                            
                            Ok(().into())
                        });
                        Self::deposit_event(Event::VoteRemoved(from, referendum));
                        Ok(().into())
                    }
                    Err(i) => Err(Error::<T>::AccountHasNotVoted.into()),
                }
            })
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn emergency_cancel_referendum(
            origin: OriginFor<T>, 
            referendum:  ReferendumId,
        ) -> DispatchResultWithPostInfo {
            let from = ensure_signed(origin)?;
            let referendum_info = Self::referendum_info(referendum).ok_or(Error::<T>::ReferendumDoesNotExist)?;
            match referendum_info {
                ReferendumInfo::Ongoing(referendum_status) => {
                    ensure!(from == Self::referendum_jury(referendum_status.country).ok_or(Error::<T>::JuryNotSelected)?,  Error::<T>::InsufficientPrivileges);
                    ensure!(!Self::does_proposal_changes_jury(referendum_status.clone())?,  Error::<T>::InsufficientPrivileges);
                    <Proposals<T>>::remove(referendum_status.country,referendum_status.proposal);
                    <ReferendumInfoOf<T>>::remove(referendum);
                    Self::update_proposals_per_country_number(referendum_status.country, false);
                    T::Currency::unreserve(&from, Self::deposit(referendum_status.proposal).ok_or(Error::<T>::DepositNotFound)?.1); 
                    <DepositOf<T>>::remove(referendum_status.proposal);
                    Self::deposit_event(Event::ReferendumCancelled(referendum)); 
                },
                _ => (),
            }
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        fn start_referendum(country_id: CountryId, proposal_id: ProposalId, current_block: T::BlockNumber) -> Result<u64,DispatchError> {
            let referendum_id = Self::get_next_referendum_id()?;
            
            let mut referendum_end = current_block;
            let mut referendum_threshold = VoteThreshold::RelativeMajority;
            match Self::referendum_parameters(country_id) {
                Some(country_referendum_params) => {
                    referendum_end += country_referendum_params.voting_period;
                    match  country_referendum_params.voting_threshold {
                        Some(defined_threshold) => referendum_threshold = defined_threshold,
                        None => {},
                    } 
                },
                None => referendum_end += T::DefaultVotingPeriod::get(),
            }

            let initial_tally = Tally{
                ayes: Zero::zero(),
                nays: Zero::zero(),
                turnout: Zero::zero()
            };

            let referendum_status = ReferendumStatus{
                end: referendum_end,
                country: country_id,
                proposal: proposal_id,
                tally: initial_tally, 
                threshold: Some(referendum_threshold.clone())
            };
            let referendum_info = ReferendumInfo::Ongoing(referendum_status);
            <ReferendumInfoOf<T>>::insert(referendum_id,referendum_info);
            Self::deposit_event(Event::ReferendumStarted(referendum_id, referendum_threshold));
            Ok(referendum_id)
        }

        fn is_member(account: T::AccountId, country: CountryId) -> bool {
            // ТO DO: finish implementation
            true
        }

        fn get_next_proposal_id() -> Result<ProposalId, DispatchError> {
            <NextProposalId<T>>::try_mutate(|next_id| -> Result<ProposalId, DispatchError> {
                let current_id = *next_id;
                *next_id = next_id.checked_add(1).ok_or(Error::<T>::ProposalIdOverflow)?;
                Ok(current_id)
            })
        }

        fn get_next_referendum_id() -> Result<ReferendumId, DispatchError> {
            <NextReferendumId<T>>::try_mutate(|next_id| -> Result<ReferendumId, DispatchError> {
                let current_id = *next_id;
                *next_id = next_id.checked_add(1).ok_or(Error::<T>::ReferendumIdOverflow)?;
                Ok(current_id)
            })
        }

        fn update_proposals_per_country_number(country: CountryId, is_proposal_added: bool) -> DispatchResult {
            <TotalProposalsPerCountry<T>>::try_mutate(country, |number_of_proposals| -> DispatchResult {
                if is_proposal_added {
                    *number_of_proposals = number_of_proposals.checked_add(1).ok_or(Error::<T>::ProposalQueueOverflow)?;
                }
                else {
                    *number_of_proposals = number_of_proposals.checked_sub(1).ok_or(Error::<T>::ProposalQueueOverflow)?;
                }
                Ok(())
            })
        }

        fn does_proposal_changes_jury(referendum_status: ReferendumStatus<T::BlockNumber>) -> Result<bool, DispatchError> {
            let proposal_parameters = Self::proposals(referendum_status.country,referendum_status.proposal).ok_or(Error::<T>::ProposalDoesNotExist)?.parameters;
            let mut result = false;
            for parameter in proposal_parameters.iter() {
                match parameter {
                    CountryParameter::SetReferendumJury(a) =>{
                        result = true;
                        break;
                    },
                    _ => continue,
                }
            }
            Ok(result)
        }

        fn referendum_status(referendum_id: ReferendumId) -> Result<ReferendumStatus<T::BlockNumber>, DispatchError> {
            let info = Self::referendum_info(referendum_id).ok_or(Error::<T>::ReferendumDoesNotExist)?;
            Self::ensure_ongoing(info.into())
        }

        /// Ok if the given referendum is active, Err otherwise
        fn ensure_ongoing(r: ReferendumInfo<T::BlockNumber>)
        -> Result<ReferendumStatus<T::BlockNumber>, DispatchError>
        {
             match r {
                ReferendumInfo::Ongoing(s) => Ok(s),
                _ => Err(Error::<T>::ReferendumIsOver.into()),
            }
        }

        fn enact_proposal(proposal: ProposalId) -> DispatchResult {
            Ok(())
        }
    
        fn update_country_parameters(new_parameters: Vec<u8>) -> DispatchResult {
            Ok(())
        }
    
    }
}