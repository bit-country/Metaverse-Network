#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use codec::{Decode, Encode, Input};

use frame_support::{
	dispatch::DispatchResult,
	ensure,
	traits::{
		schedule::{DispatchTime, Named as ScheduleNamed},
		Currency, InstanceFilter, LockIdentifier, ReservableCurrency,
	},
	weights::Weight,
};
use frame_system::ensure_signed;
use metaverse_primitive::MetaverseTrait;
use primitives::{MetaverseId, ProposalId, ReferendumId};
use sp_runtime::traits::{Dispatchable, Hash, Saturating, Zero};
use sp_std::prelude::*;

mod types;

pub use types::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

const GOVERNANCE_ID: LockIdentifier = *b"bcgovern";

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::sp_runtime::transaction_validity::InvalidTransaction::Call;
	use frame_support::{
		dispatch::DispatchResultWithPostInfo,
		pallet_prelude::*,
		traits::EnsureOrigin,
		weights::{DispatchClass, Pays},
		Parameter,
	};
	use frame_system::{ensure_root, ensure_signed, pallet_prelude::*};
	use metaverse_primitive::MetaverseLandTrait;
	use sp_runtime::DispatchResult;

	pub(crate) type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

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

		#[pallet::constant]
		type MinimumProposalDeposit: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type DefaultPreimageByteDeposit: Get<BalanceOf<Self>>;

		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		type MetaverseInfo: MetaverseTrait<Self::AccountId>;

		type MetaverseLandInfo: MetaverseLandTrait<Self::AccountId>;

		/// Overarching type of all pallets origins.
		type PalletsOrigin: From<frame_system::RawOrigin<Self::AccountId>>;

		type Proposal: Parameter + Dispatchable<Origin = Self::Origin> + From<Call<Self>>;

		type ProposalType: Parameter + Member + InstanceFilter<Self::Proposal>;
		/// The Scheduler.
		type Scheduler: ScheduleNamed<Self::BlockNumber, Self::Proposal, Self::PalletsOrigin>;
		/// Metaverse Council which collective of members
		type MetaverseCouncil: EnsureOrigin<Self::Origin>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::storage]
	#[pallet::getter(fn preimages)]
	pub type Preimages<T: Config> =
		StorageMap<_, Identity, T::Hash, PreimageStatus<T::AccountId, BalanceOf<T>, T::BlockNumber>>;

	#[pallet::storage]
	#[pallet::getter(fn proposals)]
	pub type Proposals<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		MetaverseId,
		Twox64Concat,
		ProposalId,
		ProposalInfo<T::AccountId, T::BlockNumber, T::Hash>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn next_proposal)]
	pub type NextProposalId<T: Config> = StorageValue<_, ProposalId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn proposals_per_country)]
	pub type TotalProposalsPerCountry<T: Config> = StorageMap<_, Twox64Concat, MetaverseId, u8, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn deposit)]
	pub type DepositOf<T: Config> = StorageMap<_, Twox64Concat, ProposalId, (T::AccountId, BalanceOf<T>), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn referendum_info)]
	pub type ReferendumInfoOf<T: Config> =
		StorageMap<_, Twox64Concat, ReferendumId, ReferendumInfo<T::BlockNumber>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_referendum)]
	pub type NextReferendumId<T: Config> = StorageValue<_, ReferendumId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn referendum_parameters)]
	pub type ReferendumParametersOf<T: Config> =
		StorageMap<_, Twox64Concat, MetaverseId, ReferendumParameters<T::BlockNumber>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn referendum_jury)]
	pub type ReferendumJuryOf<T: Config> = StorageMap<_, Twox64Concat, MetaverseId, T::AccountId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn voting_record)]
	pub type VotingOf<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, VotingRecord, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	#[pallet::metadata()]
	pub enum Event<T: Config> {
		PreimageNoted(T::Hash, T::AccountId, BalanceOf<T>),
		PreimageInvalid(MetaverseId, T::Hash, ProposalId),
		PreimageMissing(MetaverseId, T::Hash, ProposalId),
		PreimageUsed(T::Hash, T::AccountId, BalanceOf<T>),
		PreimageEnacted(MetaverseId, T::Hash, DispatchResult),
		ReferendumParametersUpdated(MetaverseId),
		ProposalSubmitted(T::AccountId, MetaverseId, ProposalId),
		ProposalCancelled(T::AccountId, MetaverseId, ProposalId),
		ProposalFastTracked(MetaverseId, ProposalId),
		ProposalEnacted(MetaverseId, ProposalId),
		ReferendumStarted(ReferendumId, VoteThreshold),
		ReferendumPassed(ReferendumId),
		ReferendumNotPassed(ReferendumId),
		ReferendumCancelled(ReferendumId),
		VoteRecorded(T::AccountId, ReferendumId, bool),
		VoteRemoved(T::AccountId, ReferendumId),
	}

	#[pallet::error]
	pub enum Error<T> {
		AccountIsNotMetaverseMember,
		AccountIsNotMetaverseOwner,
		ReferendumParametersOutOfScope,
		InsufficientBalance,
		DepositTooLow,
		ProposalParametersOutOfScope,
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
		ReferendumParametersDoesNotExist,
		PreimageMissing,
		PreimageInvalid,
		PreimageCallsOutOfScope,
		DuplicatePreimage,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Metaverse owner can update referendum parameters
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn update_referendum_parameters(
			origin: OriginFor<T>,
			metaverse_id: MetaverseId,
			new_referendum_parameters: ReferendumParameters<T::BlockNumber>,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;
			ensure!(
				T::MetaverseInfo::check_ownership(&from, &metaverse_id),
				Error::<T>::AccountIsNotMetaverseOwner
			);
			<ReferendumParametersOf<T>>::remove(metaverse_id);
			<ReferendumParametersOf<T>>::insert(metaverse_id, new_referendum_parameters);
			Self::deposit_event(Event::ReferendumParametersUpdated(metaverse_id));
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn note_preimage(origin: OriginFor<T>, encoded_proposal: Vec<u8>) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;
			ensure!(
				Self::is_preimage_valid(encoded_proposal.clone())?,
				Error::<T>::PreimageCallsOutOfScope
			);
			Self::note_preimage_inner(from, encoded_proposal.clone())?;
			Ok(().into())
		}

		/// Create new metaverse proposal
		/// Only metaverse members who own piece of land has the ability to vote on local metaverse
		/// proposal
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn propose(
			origin: OriginFor<T>,
			metaverse_id: MetaverseId,
			balance: BalanceOf<T>,
			preimage_hash: T::Hash,
			proposal_description: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;
			ensure!(
				T::MetaverseLandInfo::is_user_own_metaverse_land(&from, &metaverse_id),
				Error::<T>::AccountIsNotMetaverseMember
			);
			ensure!(balance >= T::MinimumProposalDeposit::get(), Error::<T>::DepositTooLow);
			ensure!(
				T::Currency::free_balance(&from) >= balance,
				Error::<T>::InsufficientBalance
			);
			ensure!(<Preimages<T>>::contains_key(preimage_hash), Error::<T>::PreimageInvalid);
			let launch_block = Self::get_proposal_launch_block(metaverse_id)?;
			let proposal_info = ProposalInfo {
				proposed_by: from.clone(),
				//  parameter: proposal_parameter,
				hash: preimage_hash,
				description: proposal_description,
				referendum_launch_block: launch_block,
			};

			let proposal_id = Self::get_next_proposal_id()?;
			<Proposals<T>>::insert(metaverse_id, proposal_id, proposal_info);

			Self::update_proposals_per_country_number(metaverse_id, true);
			T::Currency::reserve(&from, balance);
			<DepositOf<T>>::insert(proposal_id, (&from, balance));

			Self::start_referendum(metaverse_id, proposal_id, launch_block);
			Self::deposit_event(Event::ProposalSubmitted(from, metaverse_id, proposal_id));

			Ok(().into())
		}

		/// Cancel proposal if you are the proposal owner, the proposal exist, and it has not
		/// launched as a referendum yet
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn cancel_proposal(
			origin: OriginFor<T>,
			proposal: ProposalId,
			metaverse_id: MetaverseId,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;
			let proposal_info = Self::proposals(metaverse_id, proposal).ok_or(Error::<T>::ProposalDoesNotExist)?;

			ensure!(proposal_info.proposed_by == from, Error::<T>::NotProposalCreator);
			ensure!(
				proposal_info.referendum_launch_block > <frame_system::Pallet<T>>::block_number(),
				Error::<T>::ProposalIsReferendum
			);
			<Proposals<T>>::remove(metaverse_id, proposal);
			Self::update_proposals_per_country_number(metaverse_id, false);

			T::Currency::unreserve(&from, Self::deposit(proposal).ok_or(Error::<T>::DepositNotFound)?.1);
			<DepositOf<T>>::remove(proposal);

			Self::deposit_event(Event::ProposalCancelled(from, metaverse_id, proposal));
			Ok(().into())
		}

		/// Fast track proposal to referendum if you are metaverse council and it has not launched
		/// as a referendum yet
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn fast_track_proposal(
			origin: OriginFor<T>,
			proposal: ProposalId,
			metaverse_id: MetaverseId,
		) -> DispatchResultWithPostInfo {
			let from = T::MetaverseCouncil::ensure_origin(origin)?;

			let mut proposal_info = Self::proposals(metaverse_id, proposal).ok_or(Error::<T>::ProposalDoesNotExist)?;

			let current_block_number = <frame_system::Pallet<T>>::block_number();
			ensure!(
				proposal_info.referendum_launch_block > current_block_number,
				Error::<T>::ProposalIsReferendum
			);
			proposal_info.referendum_launch_block = current_block_number + T::OneBlock::get();
			<Proposals<T>>::remove(metaverse_id, proposal);
			<Proposals<T>>::insert(metaverse_id, proposal, proposal_info);
			Self::deposit_event(Event::ProposalFastTracked(metaverse_id, proposal));
			Ok(().into())
		}

		/// Vote for local metaverse proposal
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn try_vote(origin: OriginFor<T>, referendum: ReferendumId, vote_aye: bool) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;
			let mut status = Self::referendum_status(referendum)?;
			ensure!(
				T::MetaverseLandInfo::is_user_own_metaverse_land(&from, &status.country),
				Error::<T>::AccountIsNotMetaverseMember
			);
			<VotingOf<T>>::try_mutate(from.clone(), |mut voting_record| -> DispatchResultWithPostInfo {
				let votes = &mut voting_record.votes;
				match votes.binary_search_by_key(&referendum, |i| i.0) {
					Ok(i) => Err(Error::<T>::AccountAlreadyVoted.into()),
					Err(i) => {
						let vote = Vote {
							aye: vote_aye,
							//balance: T::Currency::free_balance(&from)
						};

						votes.insert(i, (referendum, vote.clone()));

						<ReferendumInfoOf<T>>::try_mutate(
							referendum,
							|referendum_info| -> DispatchResultWithPostInfo {
								status.tally.add(vote).ok_or(Error::<T>::TallyOverflow)?;
								*referendum_info = Some(ReferendumInfo::Ongoing(status));

								Ok(().into())
							},
						);
						Self::deposit_event(Event::VoteRecorded(from, referendum, vote_aye));
						Ok(().into())
					}
				}
			})
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn try_remove_vote(origin: OriginFor<T>, referendum: ReferendumId) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;
			let mut status = Self::referendum_status(referendum)?;
			<VotingOf<T>>::try_mutate(from.clone(), |voting_record| -> DispatchResultWithPostInfo {
				let mut votes = &mut voting_record.votes;
				match votes.binary_search_by_key(&referendum, |i| i.0) {
					Ok(i) => {
						let vote = votes.remove(i).1;
						<ReferendumInfoOf<T>>::try_mutate(
							referendum,
							|referendum_info| -> DispatchResultWithPostInfo {
								status.tally.remove(vote).ok_or(Error::<T>::TallyOverflow)?;
								*referendum_info = Some(ReferendumInfo::Ongoing(status));

								Ok(().into())
							},
						);
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
			referendum: ReferendumId,
		) -> DispatchResultWithPostInfo {
			T::MetaverseCouncil::ensure_origin(origin)?;

			let referendum_info = Self::referendum_info(referendum).ok_or(Error::<T>::ReferendumDoesNotExist)?;
			match referendum_info {
				ReferendumInfo::Ongoing(referendum_status) => {
					let proposal_info = Self::proposals(referendum_status.country, referendum_status.proposal)
						.ok_or(Error::<T>::InvalidProposalParameters)?;
					<ReferendumInfoOf<T>>::remove(referendum);
					<Proposals<T>>::remove(referendum_status.country, referendum_status.proposal);
					Self::update_proposals_per_country_number(referendum_status.country, false);
					T::Currency::unreserve(
						&proposal_info.proposed_by,
						Self::deposit(referendum_status.proposal)
							.ok_or(Error::<T>::DepositNotFound)?
							.1,
					);
					<DepositOf<T>>::remove(referendum_status.proposal);
					Self::deposit_event(Event::ReferendumCancelled(referendum));
				}
				_ => (),
			}
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn enact_proposal(
			origin: OriginFor<T>,
			proposal: ProposalId,
			metaverse_id: MetaverseId,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			Self::do_enact_proposal(proposal, metaverse_id);
			Self::deposit_event(Event::ProposalEnacted(metaverse_id, proposal));
			Ok(().into())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		/// Finalization
		fn on_finalize(now: T::BlockNumber) {
			for (referendum_id, referendum_info) in <ReferendumInfoOf<T>>::iter() {
				match referendum_info {
					ReferendumInfo::Ongoing(status) => {
						if status.end == now {
							Self::finalize_vote(referendum_id, status);
						}
					}
					_ => (),
				}
			}
		}
	}

	impl<T: Config> Pallet<T> {
		// See `note_preimage`
		fn note_preimage_inner(who: T::AccountId, encoded_proposal: Vec<u8>) -> DispatchResult {
			let proposal_hash = T::Hashing::hash(&encoded_proposal[..]);
			ensure!(
				!<Preimages<T>>::contains_key(&proposal_hash),
				Error::<T>::DuplicatePreimage
			);

			let deposit = <BalanceOf<T>>::from(encoded_proposal.len() as u32)
				.saturating_mul(T::DefaultPreimageByteDeposit::get());
			T::Currency::reserve(&who, deposit)?;

			let now = <frame_system::Pallet<T>>::block_number();
			let a = PreimageStatus::Available {
				data: encoded_proposal,
				provider: who.clone(),
				deposit,
				since: now,
				expiry: None,
			};
			<Preimages<T>>::insert(proposal_hash, a);

			Self::deposit_event(Event::<T>::PreimageNoted(proposal_hash, who, deposit));

			Ok(())
		}

		fn is_preimage_valid(encoded_proposal: Vec<u8>) -> Result<bool, DispatchError> {
			//TO DO: check whether preimage function calls are within a defined scope
			Ok(true)
		}

		fn does_preimage_updates_jury(encoded_proposal: Vec<u8>) -> Result<bool, DispatchError> {
			//TO DO: Check whether preimage updates jury
			Ok(false)
		}

		fn start_referendum(
			country_id: MetaverseId,
			proposal_id: ProposalId,
			current_block: T::BlockNumber,
		) -> Result<u64, DispatchError> {
			let referendum_id = Self::get_next_referendum_id()?;

			let mut referendum_end = current_block;
			let mut referendum_threshold = VoteThreshold::RelativeMajority;
			match Self::referendum_parameters(country_id) {
				Some(metaverse_referendum_params) => {
					referendum_end = current_block + metaverse_referendum_params.voting_period;
					match metaverse_referendum_params.voting_threshold {
						Some(defined_threshold) => referendum_threshold = defined_threshold,
						None => {}
					}
				}
				None => referendum_end = current_block + T::DefaultVotingPeriod::get(),
			}

			let initial_tally = Tally {
				ayes: Zero::zero(),
				nays: Zero::zero(),
				turnout: Zero::zero(),
			};

			let referendum_status = ReferendumStatus {
				end: referendum_end,
				country: country_id,
				proposal: proposal_id,
				tally: initial_tally,
				threshold: Some(referendum_threshold.clone()),
			};
			let referendum_info = ReferendumInfo::Ongoing(referendum_status);
			<ReferendumInfoOf<T>>::insert(referendum_id, referendum_info);

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

		fn get_proposal_launch_block(country: MetaverseId) -> Result<T::BlockNumber, DispatchError> {
			let current_block = <frame_system::Pallet<T>>::block_number();
			match Self::referendum_parameters(country) {
				Some(metaverse_referendum_params) => {
					ensure!(
						Self::proposals_per_country(country) < metaverse_referendum_params.max_proposals_per_country,
						Error::<T>::ProposalQueueFull
					);
					if metaverse_referendum_params.min_proposal_launch_period.is_zero() {
						Ok(current_block + metaverse_referendum_params.min_proposal_launch_period)
					} else {
						Ok(current_block + T::DefaultProposalLaunchPeriod::get())
					}
				}
				None => {
					ensure!(
						Self::proposals_per_country(country) < T::DefaultMaxProposalsPerCountry::get(),
						Error::<T>::ProposalQueueFull
					);
					Ok(current_block + T::DefaultProposalLaunchPeriod::get())
				}
			}
		}

		fn pre_image_data_len(preimage_hash: T::Hash) -> Result<u32, DispatchError> {
			// To decode the `data` field of Available variant we need:
			// * one byte for the variant
			// * at most 5 bytes to decode a `Compact<u32>`
			let mut buf = [0u8; 6];
			let key = <Preimages<T>>::hashed_key_for(preimage_hash);
			let bytes = sp_io::storage::read(&key, &mut buf, 0).ok_or_else(|| Error::<T>::PreimageMissing)?;
			// The value may be smaller that 6 bytes.
			let mut input = &buf[0..buf.len().min(bytes as usize)];

			match input.read_byte() {
				Ok(1) => (), // Check that input exists and is second variant.
				Ok(0) => return Err(Error::<T>::PreimageMissing.into()),
				_ => {
					sp_runtime::print("Failed to decode `PreimageStatus` variant");
					return Err(Error::<T>::PreimageMissing.into());
				}
			}

			// Decode the length of the vector.
			let len = codec::Compact::<u32>::decode(&mut input)
				.map_err(|_| {
					sp_runtime::print("Failed to decode `PreimageStatus` variant");
					DispatchError::from(Error::<T>::PreimageMissing)
				})?
				.0;

			Ok(len)
		}

		fn update_proposals_per_country_number(country: MetaverseId, is_proposal_added: bool) -> DispatchResult {
			<TotalProposalsPerCountry<T>>::try_mutate(country, |number_of_proposals| -> DispatchResult {
				if is_proposal_added {
					*number_of_proposals = number_of_proposals
						.checked_add(1)
						.ok_or(Error::<T>::ProposalQueueOverflow)?;
				} else {
					*number_of_proposals = number_of_proposals
						.checked_sub(1)
						.ok_or(Error::<T>::ProposalQueueOverflow)?;
				}
				Ok(())
			})
		}

		fn referendum_status(referendum_id: ReferendumId) -> Result<ReferendumStatus<T::BlockNumber>, DispatchError> {
			let info = Self::referendum_info(referendum_id).ok_or(Error::<T>::ReferendumDoesNotExist)?;
			Self::ensure_ongoing(info.into())
		}

		/// Ok if the given referendum is active, Err otherwise
		fn ensure_ongoing(
			r: ReferendumInfo<T::BlockNumber>,
		) -> Result<ReferendumStatus<T::BlockNumber>, DispatchError> {
			match r {
				ReferendumInfo::Ongoing(s) => Ok(s),
				_ => Err(Error::<T>::ReferendumIsOver.into()),
			}
		}

		fn find_referendum_result(threshold: Option<VoteThreshold>, tally: Tally) -> Result<bool, DispatchError> {
			if tally.turnout == 0 {
				return Ok(false);
			}

			match threshold {
				Some(ref threshold_type) => match threshold_type {
					VoteThreshold::StandardQualifiedMajority => Ok((tally.ayes as f64 / tally.turnout as f64) > 0.72),
					VoteThreshold::TwoThirdsSupermajority => Ok((tally.ayes as f64 / tally.turnout as f64) > 0.6666),
					VoteThreshold::ThreeFifthsSupermajority => Ok((tally.ayes as f64 / tally.turnout as f64) > 0.6),
					VoteThreshold::ReinforcedQualifiedMajority => Ok((tally.ayes as f64 / tally.turnout as f64) > 0.55),
					VoteThreshold::RelativeMajority => Ok(tally.ayes > tally.nays),
					_ => Err(Error::<T>::InvalidReferendumOutcome.into()),
				},
				// If no threshold is selected, the proposal will pass with relative majority
				None => Ok(tally.ayes > tally.nays),
			}
		}

		fn finalize_vote(
			referendum_id: ReferendumId,
			referendum_status: ReferendumStatus<T::BlockNumber>,
		) -> DispatchResult {
			// Return deposit
			let deposit_info = Self::deposit(referendum_status.proposal).ok_or(Error::<T>::InsufficientBalance)?;
			<DepositOf<T>>::remove(referendum_status.proposal);
			T::Currency::unreserve(&deposit_info.0, deposit_info.1);

			// Check if referendum passes
			let does_referendum_passed =
				Self::find_referendum_result(referendum_status.threshold.clone(), referendum_status.tally.clone())?;

			// Update referendum info
			<ReferendumInfoOf<T>>::try_mutate(referendum_id, |referendum_info| -> DispatchResult {
				*referendum_info = Some(ReferendumInfo::Finished {
					passed: does_referendum_passed,
					end: referendum_status.end,
				});

				Ok(())
			});

			// Enact proposal if it passed the threshold
			if does_referendum_passed {
				//Self::do_enact_proposal(referendum_status.proposal, referendum_status.country
				let mut when = referendum_status.end;
				match Self::referendum_parameters(referendum_status.country) {
					Some(current_params) => when += current_params.enactment_period,
					None => when += T::DefaultEnactmentPeriod::get(),
				}

				if T::Scheduler::schedule_named(
					(GOVERNANCE_ID, referendum_id).encode(),
					DispatchTime::At(when),
					None,
					63,
					frame_system::RawOrigin::Root.into(),
					Call::enact_proposal(referendum_status.proposal, referendum_status.country).into(),
				)
				.is_err()
				{
					frame_support::print("LOGIC ERROR: bake_referendum/schedule_named failed");
				} else {
					Self::deposit_event(Event::ReferendumPassed(referendum_id));
				}
			} else {
				Self::deposit_event(Event::ReferendumNotPassed(referendum_id));
			}

			Ok(())
		}

		fn do_enact_proposal(proposal_id: ProposalId, country: MetaverseId) -> DispatchResult {
			let proposal_info = Self::proposals(country, proposal_id).ok_or(Error::<T>::InvalidProposalParameters)?;
			let preimage = <Preimages<T>>::take(&proposal_info.hash);
			if let Some(PreimageStatus::Available {
				data,
				provider,
				deposit,
				..
			}) = preimage
			{
				if let Ok(proposal) = T::Proposal::decode(&mut &data[..]) {
					ensure!(T::ProposalType::filter(proposal), Error::<T>::PreimageInvalid);

					Self::deposit_event(Event::<T>::PreimageUsed(proposal_info.hash, provider, deposit));
					let result = proposal
						.dispatch(frame_system::RawOrigin::Root.into())
						.map(|_| ())
						.map_err(|e| e.error);

					Self::deposit_event(Event::<T>::PreimageEnacted(country, proposal_info.hash, result));

					Ok(())
				} else {
					Self::deposit_event(Event::<T>::PreimageInvalid(country, proposal_info.hash, proposal_id));
					Err(Error::<T>::PreimageInvalid.into())
				}
			} else {
				Self::deposit_event(Event::<T>::PreimageMissing(country, proposal_info.hash, proposal_id));
				Err(Error::<T>::PreimageMissing.into())
			}
		}
	}
}
