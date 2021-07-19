#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

//use codec::{Decode, Encode};

use frame_system::ensure_signed;
use primitives::{CountryId, ProposalId, ReferendumId};
use sp_std::prelude::*;
use sp_runtime::traits::{Zero, Dispatchable};
use bc_country::BCCountry;
use frame_support::{
    ensure, weights::Weight,
    traits::{Currency, ReservableCurrency, LockIdentifier, Vec, schedule::{Named as ScheduleNamed, DispatchTime} },
    dispatch::DispatchResult,
};


mod types;
pub use types::*;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod mock;

const GOVERNANCE_ID: LockIdentifier = *b"bcgovern";

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use sp_runtime::DispatchResult;
    use frame_support::{
		pallet_prelude::*, Parameter,
		weights::{DispatchClass, Pays}, traits::EnsureOrigin, dispatch::DispatchResultWithPostInfo,
	};
	use frame_system::{pallet_prelude::*, ensure_signed, ensure_root};
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

        /// Overarching type of all pallets origins.
		type PalletsOrigin: From<frame_system::RawOrigin<Self::AccountId>>;

        type Proposal: Parameter + Dispatchable<Origin=Self::Origin> + From<Call<Self>>;

        /// The Scheduler.
		type Scheduler: ScheduleNamed<Self::BlockNumber, Self::Proposal, Self::PalletsOrigin>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

   
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
        ProposalEnacted(CountryId, ProposalId),
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
        InvalidProposalParameters,
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
        AccountAlreadyVoted,
        InvalidJuryAddress,
        InvalidReferendumOutcome,
        ReferendumParametersDoesNotExist

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
            ensure!(T::CountryInfo::is_member(&from, &country), Error::<T>::AccountNotCountryMember);
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
        pub fn try_vote(
            origin: OriginFor<T>, 
            referendum: ReferendumId,
            vote_aye: bool
        ) -> DispatchResultWithPostInfo {
            let from = ensure_signed(origin)?;
            let mut status = Self::referendum_status(referendum)?;
            ensure!(T::CountryInfo::is_member(&from, &status.country), Error::<T>::AccountNotCountryMember);
            <VotingOf<T>>::try_mutate(from.clone(),|mut voting_record| -> DispatchResultWithPostInfo {
                let votes = &mut voting_record.votes;
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
        pub fn try_remove_vote(
            origin: OriginFor<T>, 
            referendum:  ReferendumId,
        ) -> DispatchResultWithPostInfo {
            let from = ensure_signed(origin)?;
            let mut status = Self::referendum_status(referendum)?;
            <VotingOf<T>>::try_mutate(from.clone(),|voting_record| -> DispatchResultWithPostInfo {
                let mut votes = &mut voting_record.votes;
                match votes.binary_search_by_key(&referendum, |i| i.0) {
                    Ok(i) => {
                        let vote = votes.remove(i).1;
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
        
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn enact_proposal(
			origin: OriginFor<T>,
			proposal: ProposalId,
			country: CountryId,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			Self::do_enact_proposal(proposal, country);
            Self::deposit_event(Event::ProposalEnacted(country, proposal));
            Ok(().into())
		}

    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        /// Initialization
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
            for (referendum_id,referendum_info) in <ReferendumInfoOf<T>>::iter() {
                match referendum_info {
                    ReferendumInfo::Ongoing(status) => {
                        if status.end == now {
                            Self::finalize_vote(referendum_id,status);
                        }
                           
                    }
                    _ => (),
                }
            }
        }
    }
    
    impl<T: Config> Pallet<T> {
        fn start_referendum(country_id: CountryId, proposal_id: ProposalId, current_block: T::BlockNumber) -> Result<u64,DispatchError> {
            let referendum_id = Self::get_next_referendum_id()?;
            
            let mut referendum_end = current_block;
            let mut referendum_threshold = VoteThreshold::RelativeMajority;
            match Self::referendum_parameters(country_id) {
                Some(country_referendum_params) => {
                    referendum_end = current_block + country_referendum_params.voting_period;
                    match  country_referendum_params.voting_threshold {
                        Some(defined_threshold) => referendum_threshold = defined_threshold,
                        None => {},
                    } 
                },
                None => referendum_end = current_block + T::DefaultVotingPeriod::get(),
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
        fn ensure_ongoing(r: ReferendumInfo<T::BlockNumber>) -> Result<ReferendumStatus<T::BlockNumber>, DispatchError>
        {
             match r {
                ReferendumInfo::Ongoing(s) => Ok(s),
                _ => Err(Error::<T>::ReferendumIsOver.into()),
            }
        }

        fn find_referendum_result(threshold: Option<VoteThreshold>, tally: Tally) -> Result<bool, DispatchError> {

            if tally.turnout == 0  {
                return Ok(false);
            }

            match threshold {
                Some(ref threshold_type) => {
                    match threshold_type {
                        VoteThreshold::StandardQualifiedMajority => Ok((tally.ayes as f64 / tally.turnout as f64) > 0.72), 
                        VoteThreshold::TwoThirdsSupermajority => Ok((tally.ayes as f64 / tally.turnout as f64) > 0.6666), 
                        VoteThreshold::ThreeFifthsSupermajority =>  Ok((tally.ayes as f64 / tally.turnout as f64) > 0.6), 
                        VoteThreshold::ReinforcedQualifiedMajority =>  Ok((tally.ayes as f64 / tally.turnout as f64) > 0.55), 
                        VoteThreshold::RelativeMajority => Ok(tally.ayes > tally.nays),
                        _ => Err(Error::<T>::InvalidReferendumOutcome.into()),
                    }
                }
                // If no threshold is selected, the proposal will pass with relative majority
                None => Ok(tally.ayes > tally.nays),    
            }

        }

        fn finalize_vote(referendum_id: ReferendumId, referendum_status: ReferendumStatus<T::BlockNumber>) -> DispatchResult {
            
            // Return deposit
            let deposit_info = Self::deposit(referendum_status.proposal).ok_or(Error::<T>::InsufficientBalance)?;
            <DepositOf<T>>::remove(referendum_status.proposal);
            T::Currency::unreserve(&deposit_info.0, deposit_info.1); 

            // Check if referendum passes
            let does_referendum_passed = Self::find_referendum_result(referendum_status.threshold.clone() , referendum_status.tally.clone())?;
           
            // Update referendum info
            <ReferendumInfoOf<T>>::try_mutate(referendum_id,|referendum_info| -> DispatchResult {
                *referendum_info = Some(ReferendumInfo::Finished {
                    passed: does_referendum_passed,
                    end: referendum_status.end
                });
                
                Ok(())
            });

            // Enact proposal if it passed the threshold
            if does_referendum_passed {
                //Self::do_enact_proposal(referendum_status.proposal, referendum_status.country
                let mut when = referendum_status.end;
                match Self::referendum_parameters(referendum_status.country) {
                    Some(current_params) =>  when += current_params.enactment_period ,
                    None =>  when += T::DefaultEnactmentPeriod::get(),
                }
				if T::Scheduler::schedule_named(
					(GOVERNANCE_ID, referendum_id).encode(),
					DispatchTime::At(when),
					None,
					63,
					frame_system::RawOrigin::Root.into(),
					Call::enact_proposal(referendum_status.proposal, referendum_status.country).into(),
				).is_err() {
					frame_support::print("LOGIC ERROR: bake_referendum/schedule_named failed");
				}
                else {
                    Self::deposit_event(Event::ReferendumPassed(referendum_id));
                }

            } else {
                Self::deposit_event(Event::ReferendumNotPassed(referendum_id));
            }
            
            Ok(()) 
        }

        fn do_enact_proposal(proposal: ProposalId, country: CountryId) -> DispatchResult {
            let proposal_parameters = Self::proposals(country,proposal).ok_or(Error::<T>::InvalidProposalParameters)?.parameters;
            let mut are_referendum_params_updated = false;
            let mut new_referendum_parameters: ReferendumParameters<T::BlockNumber>;
            match Self::referendum_parameters(country) {
                Some(current_params) => new_referendum_parameters = current_params,
                None =>  {
                        new_referendum_parameters = ReferendumParameters {
                        voting_threshold: Some(VoteThreshold::RelativeMajority),
                        min_proposal_launch_period: T::DefaultProposalLaunchPeriod::get(),
                        voting_period: T::DefaultVotingPeriod::get(),
                        enactment_period: T::DefaultEnactmentPeriod::get(),
                        max_params_per_proposal: T::DefaultMaxParametersPerProposal::get(),
                        max_proposals_per_country: T::DefaultMaxProposalsPerCountry::get()
                    };
                    are_referendum_params_updated = true;
                }
            };
           
            for parameter in proposal_parameters.iter() {
                match parameter {
                    CountryParameter::MaxProposals(new_max_proposals) => {
                        new_referendum_parameters.max_proposals_per_country = *new_max_proposals;
                        are_referendum_params_updated = true;
                    },
                    CountryParameter::MaxParametersPerProposal(new_max_params) => {
                        new_referendum_parameters.max_params_per_proposal = *new_max_params;
                        are_referendum_params_updated = true;
                     },
                    CountryParameter::SetReferendumJury(new_jury_address) => {
                        // TO DO: Finish Implementation
                        //  <ReferendumJuryOf<T>>::try_mutate(country,|jury| -> DispatchResult {
                           // let new_acc: AccountId = T::AccountId::decode(new_jury_address.as_mut()).expect("error");
                           // *jury = Some(new_acc);
                            // Ok(())
                      //  });
                    },//),
                    _ => {}, // implement more options when new parameters are included
                }
            }
            if are_referendum_params_updated  {
                <ReferendumParametersOf<T>>::try_mutate(country,|referendum_params| -> DispatchResult {
                    *referendum_params = Some(new_referendum_parameters);
                    Ok(())
                });
            }
            Ok(())
        }

    }
}