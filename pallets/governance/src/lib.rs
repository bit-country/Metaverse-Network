// This file is part of Metaverse.Network & Bit.Country.

// The multi-metaverse governance module is inspired by frame democracy of how to store hash
// and preimages. Ref: https://github.com/paritytech/substrate/tree/master/frame/democracy

// Copyright (C) 2020-2022 Metaverse.Network & Bit.Country .
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use codec::{Decode, Encode};
use frame_support::{
	dispatch::DispatchResult,
	ensure,
	pallet_prelude::*,
	traits::{
		schedule::{DispatchTime, Named as ScheduleNamed},
		Currency, Get, InstanceFilter, LockIdentifier, LockableCurrency, OnUnbalanced, ReservableCurrency,
		WithdrawReasons,
	},
};
use sp_runtime::traits::{Dispatchable, Hash, Saturating, Zero};
use sp_std::prelude::*;

use metaverse_primitive::MetaverseTrait;
pub use pallet::*;
use primitives::{MetaverseId, ProposalId, ReferendumId};
pub use types::*;

mod types;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

const GOVERNANCE_ID: LockIdentifier = *b"bcgovern";

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResultWithPostInfo, traits::EnsureOrigin, Parameter};
	use frame_system::{ensure_root, ensure_signed, pallet_prelude::*};
	use sp_runtime::DispatchResult;

	use metaverse_primitive::MetaverseLandTrait;

	use super::*;

	pub(crate) type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	pub(crate) type NegativeImbalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Constant equivalent to one block
		#[pallet::constant]
		type OneBlock: Get<Self::BlockNumber>;

		/// Default preimage byte deposit
		#[pallet::constant]
		type DefaultPreimageByteDeposit: Get<BalanceOf<Self>>;

		/// Minimum proposal deposit balance
		#[pallet::constant]
		type MinimumProposalDeposit: Get<BalanceOf<Self>>;

		/// Default launch period for a proposal
		#[pallet::constant]
		type DefaultProposalLaunchPeriod: Get<u32>;

		/// Default voting period for a local referendum
		#[pallet::constant]
		type DefaultVotingPeriod: Get<u32>;

		/// Default proposal enactment period after a local referendum passes
		#[pallet::constant]
		type DefaultEnactmentPeriod: Get<u32>;

		/// Default vote locking period for a local referendum
		#[pallet::constant]
		type DefaultLocalVoteLockingPeriod: Get<u32>;

		/// Default max number of proposals in a local referendum queue
		#[pallet::constant]
		type DefaultMaxProposalsPerMetaverse: Get<u32>;

		/// Native currency type that handles voting
		type Currency: ReservableCurrency<Self::AccountId>
			+ LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;

		/// Slashing handler
		type Slash: OnUnbalanced<NegativeImbalanceOf<Self>>;

		/// Metaverse info trait for getting information from metaverse
		type MetaverseInfo: MetaverseTrait<Self::AccountId>;

		/// Metaverse land info trait for getting information from a land
		type MetaverseLandInfo: MetaverseLandTrait<Self::AccountId>;

		/// Overarching type of all pallets origins.
		type PalletsOrigin: From<frame_system::RawOrigin<Self::AccountId>>;

		/// The proposal.
		type Proposal: Parameter + Dispatchable<Origin = Self::Origin> + From<Call<Self>>;

		/// The proposal type for filtering preimages
		type ProposalType: Parameter + Member + Default + InstanceFilter<Self::Proposal>;

		/// The Scheduler.
		type Scheduler: ScheduleNamed<Self::BlockNumber, Self::Proposal, Self::PalletsOrigin>;

		/// Metaverse Council which collective of members
		type MetaverseCouncil: EnsureOrigin<Self::Origin>;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::storage]
	#[pallet::getter(fn preimages)]
	/// Indexes local governance preimages status by metaverse ID and preimage hash.
	pub type Preimages<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		MetaverseId,
		Identity,
		T::Hash,
		PreimageStatus<T::AccountId, BalanceOf<T>, T::BlockNumber>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn proposals)]
	/// Index local governance proposal information by metaverse ID and proposal ID.
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
	/// Track the next local governance proposal ID.
	pub type NextProposalId<T: Config> = StorageValue<_, ProposalId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn proposals_per_metaverse)]
	/// Store number of proposals for each metaverse.
	pub type TotalProposalsPerMetaverse<T: Config> = StorageMap<_, Twox64Concat, MetaverseId, u8, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn deposit_of)]
	/// Store deposits for each local governance proposal.
	pub type DepositOf<T: Config> = StorageMap<_, Twox64Concat, ProposalId, (Vec<T::AccountId>, BalanceOf<T>)>;

	#[pallet::storage]
	#[pallet::getter(fn referendum_info)]
	/// Index local referendum information by metaverse ID and referendum ID.
	pub type ReferendumInfoOf<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		MetaverseId,
		Twox64Concat,
		ReferendumId,
		ReferendumInfo<T::BlockNumber, BalanceOf<T>, T::Hash>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn next_referendum)]
	/// Track local governance referendum ID.
	pub type NextReferendumId<T: Config> = StorageValue<_, ReferendumId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn referendum_parameters)]
	/// Store local governance referendum parameters for each metaverse
	pub type ReferendumParametersOf<T: Config> =
		StorageMap<_, Twox64Concat, MetaverseId, ReferendumParameters<T::BlockNumber>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn voting_record)]
	/// Store voting records for each account
	pub type VotingOf<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, VotingRecord<BalanceOf<T>, T::BlockNumber>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Noted a local governance preimage
		PreimageNoted(MetaverseId, T::Hash, T::AccountId, BalanceOf<T>),
		/// Local governance preimage is invalid
		PreimageInvalid(MetaverseId, T::Hash, ReferendumId),
		/// Local governance preimage is missing
		PreimageMissing(MetaverseId, T::Hash, ReferendumId),
		/// Local governance preimage is used
		PreimageUsed(MetaverseId, T::Hash, T::AccountId, BalanceOf<T>),
		/// Local governance preimage is enacted
		PreimageEnacted(MetaverseId, T::Hash, DispatchResult),
		/// Local governance referendum parameters are updated
		ReferendumParametersUpdated(MetaverseId),
		/// Local governance proposal is refused
		ProposalRefused(MetaverseId, T::Hash),
		/// Local governance proposal is submitted
		ProposalSubmitted(T::AccountId, MetaverseId, ProposalId),
		/// Local governance proposal is cancelled
		ProposalCancelled(MetaverseId, ProposalId),
		/// Local governance proposal is fast-tracked
		ProposalFastTracked(MetaverseId, ProposalId),
		/// Local governance proposal is enacted
		ProposalEnacted(MetaverseId, ReferendumId),
		/// Local governance referendum has started
		ReferendumStarted(MetaverseId, ProposalId, ReferendumId, VoteThreshold),
		/// Local governance referendum has passed
		ReferendumPassed(ReferendumId),
		/// Local governance referendum has not passed
		ReferendumNotPassed(ReferendumId),
		/// Local governance referendum is canceled
		ReferendumCancelled(ReferendumId),
		/// Vote for local governance referendum is recorded
		VoteRecorded(T::AccountId, ReferendumId, bool),
		/// Removed a vote for local governance referendum
		VoteRemoved(T::AccountId, ReferendumId),
		/// Local governance proposal is seconded
		Seconded(T::AccountId, ProposalId),
		/// Local governance proposal is added to the proposal queue
		Tabled(ProposalId, BalanceOf<T>, Vec<T::AccountId>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Account is not a metaverse member
		AccountIsNotMetaverseMember,
		/// Account is not a metaverse owner
		AccountIsNotMetaverseOwner,
		/// Referendum parameters are out of scope
		ReferendumParametersOutOfScope,
		/// Insufficient funds
		InsufficientBalance,
		/// Deposit is less than the minimum deposit for local governance proposal
		DepositTooLow,
		/// Proposal parameters are out of scope
		ProposalParametersOutOfScope,
		/// Invalid proposal parameters
		InvalidProposalParameters,
		/// Proposal queue is full
		ProposalQueueFull,
		/// Proposal does not exist
		ProposalDoesNotExist,
		/// Account is not a creator of a given proposal
		NotProposalCreator,
		/// Non-existing referendum
		ReferendumDoesNotExist,
		/// Referendum voting period has passed
		ReferendumIsOver,
		/// Cannot find deposit
		DepositNotFound,
		/// Proposal ID is out of scope
		ProposalIdOverflow,
		/// Referendum ID is out of scope
		ReferendumIdOverflow,
		/// Proposal queue overflows
		ProposalQueueOverflow,
		/// Voting tally overflow
		TallyOverflow,
		/// Account has not voted in a given local referendum
		AccountHasNotVoted,
		/// Account has already voted in a given local referendum
		AccountAlreadyVoted,
		/// Invalid referendum outcome
		InvalidReferendumOutcome,
		/// Invalid referendum parameter value
		InvalidReferendumParameterValue,
		/// Referendum parameter does not exist
		ReferendumParametersDoesNotExist,
		/// Missing preimage
		PreimageMissing,
		/// Invalid preimage
		PreimageInvalid,
		/// Calls in a preimage are out local governance scope
		PreimageCallsOutOfScope,
		/// Preimage is duplicated
		DuplicatePreimage,
		/// Proposal is missing
		ProposalMissing,
		/// Invalid upper bound
		WrongUpperBound,
		/// No proposals are waiting to launch as referendums
		NoneWaiting,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Metaverse owner can update the metaverse referendum parameters
		///
		/// The dispatch origin for this call must be _Signed_. Only owner of metaverse can make
		/// this call
		/// - `metaverse_id`: the metaverse ID which will have its referendum parameters updated
		/// - `new_referendum_parameters`: the updated referendum parameters
		///
		/// Emits `ReferendumParametersUpdated` if successful.
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

		/// Create proposal preimage for a metaverse-level referendums
		///
		/// The dispatch origin for this call must be _Signed_. Only owner of land in the
		/// given metaverse can make this call
		/// - `metaverse_id`: the metaverse ID for which local referendums's preimage will be created.
		/// - `encoded_proposal`: encoded version of the proposal
		///
		/// Emits `PreimageNoted` if successful and preimage is valid.
		/// Emits `ProposalRefused` if successful and preimage is invalid.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn note_preimage(
			origin: OriginFor<T>,
			metaverse_id: MetaverseId,
			encoded_proposal: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;
			ensure!(
				T::MetaverseLandInfo::is_user_own_metaverse_land(&from, &metaverse_id),
				Error::<T>::AccountIsNotMetaverseMember
			);
			Self::note_preimage_inner(from, metaverse_id, encoded_proposal.clone())?;
			Ok(().into())
		}

		/// Create new metaverse-level referendum proposal if provided valid proposal hash and sufficient funds.
		///
		/// The dispatch origin for this call must be _Signed_.
		/// Only metaverse members who own piece of land has the ability to create local metaverse
		/// proposal
		/// - `metaverse_id`: the metaverse ID for which local referendums' proposal will be created
		/// - `balance`: deposit for the proposal
		/// - `preimage_hash`: hash of the selected preimage that will be part of the proposal
		/// - `proposal_description`: description of the proposal encoded as vector of numbers
		///
		/// Emits `Tabled` if successful.
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
			ensure!(
				<Preimages<T>>::contains_key(metaverse_id, preimage_hash),
				Error::<T>::PreimageInvalid
			);
			let preimage = Self::preimages(metaverse_id, preimage_hash);
			if let Some(PreimageStatus::Available {
				data,
				provider,
				deposit,
				..
			}) = preimage
			{
				if let Ok(proposal) = T::Proposal::decode(&mut &data[..]) {
					let proposal_type = T::ProposalType::default();
					if !proposal_type.filter(&proposal) {
						T::Slash::on_unbalanced(T::Currency::slash_reserved(&provider, deposit).0);
						Self::deposit_event(Event::<T>::ProposalRefused(metaverse_id, preimage_hash));
						Err(Error::<T>::PreimageInvalid.into())
					} else {
						let launch_block = Self::get_proposal_launch_block(metaverse_id)?;
						let proposal_info = ProposalInfo {
							proposed_by: from.clone(),
							hash: preimage_hash,
							title: proposal_description.clone(),
							referendum_launch_block: launch_block,
						};

						let proposal_id = Self::get_next_proposal_id()?;
						<Proposals<T>>::insert(metaverse_id, proposal_id, proposal_info);

						Self::update_proposals_per_metaverse_number(metaverse_id, true);
						T::Currency::reserve(&from, balance);
						<DepositOf<T>>::insert(proposal_id, (&[&from][..], balance));

						Self::deposit_event(Event::ProposalSubmitted(from, metaverse_id, proposal_id));

						let mut metaverse_has_referendum_running: bool = false;
						for (_, referendum_info) in ReferendumInfoOf::<T>::iter_prefix(metaverse_id) {
							match referendum_info {
								ReferendumInfo::Ongoing(status) => {
									metaverse_has_referendum_running = true;
									break;
								}
								_ => (),
							}
						}
						if !metaverse_has_referendum_running {
							if let Some((depositors, deposit)) = <DepositOf<T>>::take(proposal_id) {
								<Proposals<T>>::remove(metaverse_id, proposal_id);
								Self::update_proposals_per_metaverse_number(metaverse_id, false);
								// refund depositors
								for d in &depositors {
									T::Currency::unreserve(d, deposit);
								}
								Self::deposit_event(Event::Tabled(proposal_id, deposit, depositors));
								Self::start_referendum(
									metaverse_id,
									proposal_id,
									preimage_hash,
									proposal_description,
									launch_block,
								);
							}
						}

						Ok(().into())
					}
				} else {
					T::Slash::on_unbalanced(T::Currency::slash_reserved(&provider, deposit).0);
					Self::deposit_event(Event::<T>::ProposalRefused(metaverse_id, preimage_hash));
					Err(Error::<T>::PreimageInvalid.into())
				}
			} else {
				Self::deposit_event(Event::<T>::ProposalRefused(metaverse_id, preimage_hash));
				Err(Error::<T>::PreimageMissing.into())
			}
		}

		/// Cancel proposal if it exist for a selected metaverse
		/// and it has not launched as a referendum yet
		///
		/// The dispatch origin for this call must be _Signed_.
		/// Only the creator of a proposal can use this call.
		/// - `proposal`: the ID of the proposal that will be cancelled
		/// - `metaverse_id`: the metaverse ID of the proposal
		///
		/// Emits `ProposalCancelled` if successful.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn cancel_proposal(
			origin: OriginFor<T>,
			proposal: ProposalId,
			metaverse_id: MetaverseId,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			let proposal_info = Self::proposals(metaverse_id, proposal).ok_or(Error::<T>::ProposalDoesNotExist)?;
			if let Some((depositors, deposit)) = <DepositOf<T>>::take(proposal) {
				<Proposals<T>>::remove(metaverse_id, proposal);
				Self::update_proposals_per_metaverse_number(metaverse_id, false); // slash depositors
				for d in &depositors {
					T::Slash::on_unbalanced(T::Currency::slash_reserved(d, deposit).0);
				}
			}

			Self::deposit_event(Event::ProposalCancelled(metaverse_id, proposal));
			Ok(().into())
		}

		/// Fast track proposal to referendum if it exist for a selected metaverse
		/// and has not launched as a referendum yet
		///
		/// The dispatch origin for this call must be _Signed_.
		/// Only metaverse council can use this call.
		/// - `proposal`: the ID of the proposal that will be fast tracked
		/// - `metaverse_id`: the metaverse ID of the proposal
		///
		/// Emits `ProposalFastTracked` if successful.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn fast_track_proposal(
			origin: OriginFor<T>,
			proposal: ProposalId,
			metaverse_id: MetaverseId,
		) -> DispatchResultWithPostInfo {
			T::MetaverseCouncil::ensure_origin(origin)?;

			let mut proposal_info = Self::proposals(metaverse_id, proposal).ok_or(Error::<T>::ProposalDoesNotExist)?;

			let current_block_number = <frame_system::Pallet<T>>::block_number();
			proposal_info.referendum_launch_block = current_block_number + T::OneBlock::get();
			<Proposals<T>>::remove(metaverse_id, proposal);
			<Proposals<T>>::insert(metaverse_id, proposal, proposal_info);
			Self::deposit_event(Event::ProposalFastTracked(metaverse_id, proposal));
			Ok(().into())
		}

		/// Second a proposal to referendum if it exist and second upper bound is not reached
		///
		/// The dispatch origin for this call must be _Signed_.
		/// Only metaverse council can use this call.
		/// - `proposal`: the ID of the proposal that will be fast tracked
		/// - `seconds_upper_bound`: selected upper bound for proposal seconding
		///
		/// Emits `Seconded` if successful.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn second(origin: OriginFor<T>, proposal: ProposalId, seconds_upper_bound: u32) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let seconds = Self::len_of_deposit_of(proposal).ok_or_else(|| Error::<T>::ProposalMissing)?;
			ensure!(seconds <= seconds_upper_bound, Error::<T>::WrongUpperBound);
			let mut deposit = Self::deposit_of(proposal).ok_or(Error::<T>::ProposalMissing)?;
			T::Currency::reserve(&who, deposit.1)?;
			deposit.0.push(who.clone());
			<DepositOf<T>>::insert(proposal, deposit);
			Self::deposit_event(Event::Seconded(who, proposal));
			Ok(())
		}

		/// Vote for a local metaverse referendum if an account has not voted yet.
		/// and has sufficient balance for the vote
		///
		/// The dispatch origin for this call must be _Signed_.
		/// Only metaverse members who own piece of land has the ability to vote on local metaverse
		/// referendum
		/// - `metaverse`: the metaverse ID of the local referendum
		/// - `referendum`: the referendum ID for the vote
		/// - `vote`: the vote value, balance, and conviction
		///
		/// Emits `VoteRecorded` if successful.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn try_vote(
			origin: OriginFor<T>,
			metaverse: MetaverseId,
			referendum: ReferendumId,
			vote: Vote<BalanceOf<T>>,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;
			let mut status = Self::referendum_status(metaverse, referendum)?;
			ensure!(
				T::MetaverseLandInfo::is_user_own_metaverse_land(&from, &status.metaverse),
				Error::<T>::AccountIsNotMetaverseMember
			);
			ensure!(
				vote.balance <= T::Currency::free_balance(&from),
				Error::<T>::InsufficientBalance
			);
			<VotingOf<T>>::try_mutate(from.clone(), |voting_record| -> DispatchResultWithPostInfo {
				let votes = &mut voting_record.votes;
				match votes.binary_search_by_key(&referendum, |i| i.0) {
					Ok(_i) => Err(Error::<T>::AccountAlreadyVoted.into()),
					Err(i) => {
						votes.insert(i, (referendum, vote.clone()));

						<ReferendumInfoOf<T>>::try_mutate(
							metaverse,
							referendum,
							|referendum_info| -> DispatchResultWithPostInfo {
								status.tally.add(vote.clone()).ok_or(Error::<T>::TallyOverflow)?;
								*referendum_info = Some(ReferendumInfo::Ongoing(status));

								Ok(().into())
							},
						);
						T::Currency::extend_lock(GOVERNANCE_ID, &from, vote.balance, WithdrawReasons::TRANSFER);
						Self::deposit_event(Event::VoteRecorded(from, referendum, vote.aye));
						Ok(().into())
					}
				}
			})
		}

		/// Remove vote for a local metaverse referendum.
		///
		/// The dispatch origin for this call must be _Signed_.
		/// Only metaverse members who voted on a referendum can use this call
		/// - `referendum`: the referendum ID for the vote
		/// - `metaverse`: the metaverse ID of the local referendum
		///
		/// Emits `VoteRemoved` if successful.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn try_remove_vote(
			origin: OriginFor<T>,
			referendum: ReferendumId,
			metaverse: MetaverseId,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;
			let info = ReferendumInfoOf::<T>::get(&metaverse, &referendum);
			<VotingOf<T>>::try_mutate(from.clone(), |voting_record| -> DispatchResultWithPostInfo {
				let votes = &mut voting_record.votes;
				match votes.binary_search_by_key(&referendum, |i| i.0) {
					Ok(i) => {
						let vote = votes.remove(i).1;
						match info {
							Some(ReferendumInfo::Ongoing(mut status)) => {
								status.tally.remove(vote).ok_or(Error::<T>::TallyOverflow)?;
								ReferendumInfoOf::<T>::insert(&metaverse, &referendum, ReferendumInfo::Ongoing(status));
								Self::deposit_event(Event::VoteRemoved(from, referendum));
							}
							Some(ReferendumInfo::Finished { end, passed, title }) => {
								let prior = &mut voting_record.prior;
								if let Some((lock_periods, balance)) = vote.locked_if(passed) {
									let mut lock_value: T::BlockNumber =
										ReferendumParameters::default().local_vote_locking_period;
									match Self::referendum_parameters(metaverse) {
										Some(metaverse_referendum_params) => {
											lock_value = metaverse_referendum_params.local_vote_locking_period;
										}
										None => (),
									}
									let unlock_at = end + lock_value * lock_periods.into();
									let now = frame_system::Pallet::<T>::block_number();
									if now < unlock_at {
										prior.accumulate(unlock_at, balance);
									}
								}
							}
							None => (),
						}
						Ok(().into())
					}
					Err(_i) => Err(Error::<T>::AccountHasNotVoted.into()),
				}
			})
		}

		/// Emergency cancel ongoing referendum
		///
		/// The dispatch origin for this call must be _Signed_.
		/// Only metaverse council can use this call.
		/// - `metaverse`: the metaverse ID of the referendum
		/// - `referendum`: the referendum ID which will be canceled
		///
		/// Emits `ReferendumCancelled` if successful.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn emergency_cancel_referendum(
			origin: OriginFor<T>,
			metaverse: MetaverseId,
			referendum: ReferendumId,
		) -> DispatchResultWithPostInfo {
			T::MetaverseCouncil::ensure_origin(origin)?;

			let referendum_info =
				Self::referendum_info(metaverse, referendum).ok_or(Error::<T>::ReferendumDoesNotExist)?;
			match referendum_info {
				ReferendumInfo::Ongoing(referendum_status) => {
					<ReferendumInfoOf<T>>::remove(metaverse, referendum);
					Self::update_proposals_per_metaverse_number(referendum_status.metaverse, false);
					<DepositOf<T>>::remove(referendum_status.proposal);
					Self::deposit_event(Event::ReferendumCancelled(referendum));
				}
				_ => (),
			}
			Ok(().into())
		}

		/// Unlock balance after referendum ended and conviction period is over.
		///
		/// The dispatch origin for this call must be _Signed_.
		/// - `target`: the account which funds will be unlocked
		///
		/// Emits nothing if successful.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn unlock_balance(origin: OriginFor<T>, target: T::AccountId) -> DispatchResult {
			ensure_signed(origin)?;
			Self::update_lock(&target);
			Ok(())
		}

		/// Enacting proposal after successfully passed referendum
		///
		/// The dispatch origin for this call must be _Root_.
		/// - `proposal_id`: the ID of the proposal
		/// - `metaverse_id`: the metaverse ID of the proposal
		/// - `referendum_id`: the referendum ID of the proposal
		/// - `proposal_hash`: the hash of the proposal
		///
		/// Emits `ProposalEnacted` and 'PreimageEnacted' if successful.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn enact_proposal(
			origin: OriginFor<T>,
			proposal: ProposalId,
			metaverse_id: MetaverseId,
			referendum_id: ReferendumId,
			proposal_hash: T::Hash,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			Self::do_enact_proposal(proposal, metaverse_id, referendum_id, proposal_hash);

			Ok(().into())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		/// Hooks that call every new block finalized.
		fn on_finalize(now: T::BlockNumber) {
			for (metaverse_id, referendum_id, referendum_info) in <ReferendumInfoOf<T>>::iter() {
				match referendum_info {
					ReferendumInfo::Ongoing(status) => {
						if status.end == now {
							Self::finalize_vote(metaverse_id, referendum_id, status);
							Self::launch_public(now, metaverse_id);
						}
					}
					_ => (),
				}
			}
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Get the amount locked in support of `proposal`; `None` if proposal isn't a valid proposal
	/// index.
	pub fn backing_for(proposal: ProposalId) -> Option<BalanceOf<T>> {
		Self::deposit_of(proposal).map(|(l, d)| d * (l.len() as u32).into())
	}

	/// Reads the length of account in DepositOf without getting the complete value in the runtime.
	///
	/// Return 0 if no deposit for this proposal.
	fn len_of_deposit_of(proposal: ProposalId) -> Option<u32> {
		// DepositOf first tuple element is a vec, decoding its len is equivalent to decode a
		// `Compact<u32>`.
		decode_compact_u32_at(&<DepositOf<T>>::hashed_key_for(proposal))
	}

	/// Internal creation of preimage for proposal
	fn note_preimage_inner(who: T::AccountId, metaverse_id: MetaverseId, encoded_proposal: Vec<u8>) -> DispatchResult {
		let preimage_hash = T::Hashing::hash(&encoded_proposal[..]);
		ensure!(
			!<Preimages<T>>::contains_key(&metaverse_id, &preimage_hash),
			Error::<T>::DuplicatePreimage
		);

		let deposit =
			<BalanceOf<T>>::from(encoded_proposal.len() as u32).saturating_mul(T::DefaultPreimageByteDeposit::get());

		if let Ok(proposal) = T::Proposal::decode(&mut &encoded_proposal[..]) {
			let proposal_type = T::ProposalType::default();
			if !proposal_type.filter(&proposal) {
				T::Slash::on_unbalanced(T::Currency::slash_reserved(&who, deposit).0);
				Self::deposit_event(Event::<T>::ProposalRefused(metaverse_id, preimage_hash));
				Err(Error::<T>::PreimageInvalid.into())
			} else {
				T::Currency::reserve(&who, deposit)?;

				let now = <frame_system::Pallet<T>>::block_number();
				let a = PreimageStatus::Available {
					data: encoded_proposal,
					provider: who.clone(),
					deposit,
					since: now,
					expiry: None,
				};
				<Preimages<T>>::insert(metaverse_id, preimage_hash, a);

				Self::deposit_event(Event::<T>::PreimageNoted(metaverse_id, preimage_hash, who, deposit));

				Ok(())
			}
		} else {
			T::Slash::on_unbalanced(T::Currency::slash_reserved(&who, deposit).0);
			Self::deposit_event(Event::<T>::ProposalRefused(metaverse_id, preimage_hash));
			Err(Error::<T>::PreimageInvalid.into())
		}
	}

	/// Internal initialization of a referendum
	fn start_referendum(
		metaverse_id: MetaverseId,
		proposal_id: ProposalId,
		proposal_hash: T::Hash,
		proposal_description: Vec<u8>,
		current_block: T::BlockNumber,
	) -> Result<u64, DispatchError> {
		let referendum_id = Self::get_next_referendum_id()?;

		let referendum_end;
		let mut referendum_threshold = ReferendumParameters::<T::BlockNumber>::default()
			.voting_threshold
			.ok_or("Invalid Default Referendum Threshold")?;
		match Self::referendum_parameters(metaverse_id) {
			Some(metaverse_referendum_params) => {
				referendum_end = current_block + metaverse_referendum_params.voting_period;
				match metaverse_referendum_params.voting_threshold {
					Some(defined_threshold) => referendum_threshold = defined_threshold,
					None => {}
				}
			}
			None => referendum_end = current_block + ReferendumParameters::default().voting_period,
		}

		let initial_tally = Tally {
			ayes: Zero::zero(),
			nays: Zero::zero(),
			turnout: Zero::zero(),
		};

		let referendum_status = ReferendumStatus {
			end: referendum_end,
			metaverse: metaverse_id,
			proposal: proposal_id,
			title: proposal_description,
			tally: initial_tally,
			proposal_hash: proposal_hash,
			threshold: referendum_threshold.clone(),
		};
		let referendum_info = ReferendumInfo::Ongoing(referendum_status);
		<ReferendumInfoOf<T>>::insert(metaverse_id, referendum_id, referendum_info);

		Self::deposit_event(Event::ReferendumStarted(
			metaverse_id,
			proposal_id,
			referendum_id,
			referendum_threshold,
		));
		Ok(referendum_id)
	}

	/// Table the waiting public proposal with the highest backing for a vote.
	fn launch_public(now: T::BlockNumber, metaverse_id: MetaverseId) -> DispatchResult {
		let launch_block = Self::get_proposal_launch_block(metaverse_id)?;
		if let Some((_, proposal)) = Proposals::<T>::iter_prefix(metaverse_id).enumerate().max_by_key(
			// defensive only: All current public proposals have an amount locked
			|x| Self::backing_for((x.1).0).unwrap_or_else(Zero::zero),
		) {
			let winner_proposal_id: u64;
			winner_proposal_id = proposal.0;
			let proposal_hash: T::Hash;
			proposal_hash = proposal.1.hash;
			if let Some((depositors, deposit)) = <DepositOf<T>>::take(winner_proposal_id) {
				<Proposals<T>>::remove(metaverse_id, winner_proposal_id);
				Self::update_proposals_per_metaverse_number(metaverse_id, false);
				// refund depositors
				for d in &depositors {
					T::Currency::unreserve(d, deposit);
				}
				Self::deposit_event(Event::Tabled(winner_proposal_id, deposit, depositors));
				Self::start_referendum(
					metaverse_id,
					winner_proposal_id,
					proposal_hash,
					proposal.1.title,
					launch_block,
				);
			}
			Ok(())
		} else {
			Err(Error::<T>::NoneWaiting)?
		}
	}

	/// Internal getter of next proposall ID
	fn get_next_proposal_id() -> Result<ProposalId, DispatchError> {
		<NextProposalId<T>>::try_mutate(|next_id| -> Result<ProposalId, DispatchError> {
			let current_id = *next_id;
			*next_id = next_id.checked_add(1).ok_or(Error::<T>::ProposalIdOverflow)?;
			Ok(current_id)
		})
	}

	/// Internal getter of next referendum ID
	fn get_next_referendum_id() -> Result<ReferendumId, DispatchError> {
		<NextReferendumId<T>>::try_mutate(|next_id| -> Result<ReferendumId, DispatchError> {
			let current_id = *next_id;
			*next_id = next_id.checked_add(1).ok_or(Error::<T>::ReferendumIdOverflow)?;
			Ok(current_id)
		})
	}

	/// Internal getter of referendum launch block for a metaverse
	fn get_proposal_launch_block(metaverse_id: MetaverseId) -> Result<T::BlockNumber, DispatchError> {
		let current_block = <frame_system::Pallet<T>>::block_number();
		match Self::referendum_parameters(metaverse_id) {
			Some(metaverse_referendum_params) => {
				ensure!(
					Self::proposals_per_metaverse(metaverse_id)
						< metaverse_referendum_params.max_proposals_per_metaverse,
					Error::<T>::ProposalQueueFull
				);
				if !metaverse_referendum_params.min_proposal_launch_period.is_zero() {
					Ok(current_block + metaverse_referendum_params.min_proposal_launch_period)
				} else {
					Ok(current_block + ReferendumParameters::default().min_proposal_launch_period)
				}
			}
			None => {
				ensure!(
					Self::proposals_per_metaverse(metaverse_id)
						< ReferendumParameters::<T::BlockNumber>::default().max_proposals_per_metaverse,
					Error::<T>::ProposalQueueFull
				);
				Ok(current_block + ReferendumParameters::default().min_proposal_launch_period)
			}
		}
	}

	/// Internal update of number of proposals for a metaverse
	fn update_proposals_per_metaverse_number(metaverse_id: MetaverseId, is_proposal_added: bool) -> DispatchResult {
		<TotalProposalsPerMetaverse<T>>::try_mutate(metaverse_id, |number_of_proposals| -> DispatchResult {
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

	/// Internal getter of referendum status
	fn referendum_status(
		metaverse_id: MetaverseId,
		referendum_id: ReferendumId,
	) -> Result<ReferendumStatus<T::BlockNumber, BalanceOf<T>, T::Hash>, DispatchError> {
		let info = Self::referendum_info(metaverse_id, referendum_id).ok_or(Error::<T>::ReferendumDoesNotExist)?;
		Self::ensure_ongoing(info.into())
	}

	/// Ok if the given referendum is active, Err otherwise
	fn ensure_ongoing(
		r: ReferendumInfo<T::BlockNumber, BalanceOf<T>, T::Hash>,
	) -> Result<ReferendumStatus<T::BlockNumber, BalanceOf<T>, T::Hash>, DispatchError> {
		match r {
			ReferendumInfo::Ongoing(s) => Ok(s),
			_ => Err(Error::<T>::ReferendumIsOver.into()),
		}
	}

	/// Internal finalization of referendum voting
	fn finalize_vote(
		metaverse_id: MetaverseId,
		referendum_id: ReferendumId,
		referendum_status: ReferendumStatus<T::BlockNumber, BalanceOf<T>, T::Hash>,
	) -> DispatchResult {
		// Check if referendum passes
		let total_issuance = T::Currency::total_issuance();
		let is_referendum_approved = referendum_status
			.threshold
			.is_referendum_approved(referendum_status.tally.clone(), total_issuance);

		// Update referendum info
		<ReferendumInfoOf<T>>::try_mutate(metaverse_id, referendum_id, |referendum_info| -> DispatchResult {
			*referendum_info = Some(ReferendumInfo::Finished {
				title: referendum_status.title.clone(),
				passed: is_referendum_approved,
				end: referendum_status.end,
			});

			Ok(())
		});

		// Enact proposal if it passed the threshold
		if is_referendum_approved {
			let mut when = referendum_status.end;
			match Self::referendum_parameters(referendum_status.metaverse) {
				Some(current_params) => when += current_params.enactment_period,
				None => when += ReferendumParameters::default().enactment_period,
			}

			if T::Scheduler::schedule_named(
				(GOVERNANCE_ID, referendum_id).encode(),
				DispatchTime::At(when),
				None,
				63,
				frame_system::RawOrigin::Root.into(),
				Call::enact_proposal {
					proposal: referendum_status.proposal,
					metaverse_id: referendum_status.metaverse,
					referendum_id: referendum_id,
					proposal_hash: referendum_status.proposal_hash,
				}
				.into(),
			)
			.is_err()
			{
				frame_support::print("LOGIC ERROR: is_referendum_approved/schedule_named failed");
			} else {
				Self::deposit_event(Event::ReferendumPassed(referendum_id));
			}
		} else {
			let preimage = <Preimages<T>>::take(&metaverse_id, &referendum_status.proposal_hash);
			if let Some(PreimageStatus::Available {
				data,
				provider,
				deposit,
				..
			}) = preimage
			{
				T::Currency::unreserve(&provider, deposit);
			}
			Self::deposit_event(Event::ReferendumNotPassed(referendum_id));
		}

		Ok(())
	}

	/// Internal enacting of successfully passed proposal
	fn do_enact_proposal(
		proposal_id: ProposalId,
		metaverse_id: MetaverseId,
		referendum_id: ReferendumId,
		proposal_hash: T::Hash,
	) -> DispatchResult {
		let preimage = <Preimages<T>>::take(&metaverse_id, &proposal_hash);
		if let Some(PreimageStatus::Available {
			data,
			provider,
			deposit,
			..
		}) = preimage
		{
			if let Ok(proposal) = T::Proposal::decode(&mut &data[..]) {
				let proposal_type = T::ProposalType::default();
				if !proposal_type.filter(&proposal) {
					T::Slash::on_unbalanced(T::Currency::slash_reserved(&provider, deposit).0);
					Self::deposit_event(Event::<T>::PreimageInvalid(metaverse_id, proposal_hash, referendum_id));
					Err(Error::<T>::PreimageInvalid.into())
				} else {
					T::Currency::unreserve(&provider, deposit);
					Self::deposit_event(Event::<T>::PreimageUsed(metaverse_id, proposal_hash, provider, deposit));
					let result = proposal
						.dispatch(frame_system::RawOrigin::Root.into())
						.map(|_| ())
						.map_err(|e| e.error);

					Self::deposit_event(Event::<T>::PreimageEnacted(metaverse_id, proposal_hash, result));
					Self::deposit_event(Event::ProposalEnacted(metaverse_id, referendum_id));
					Ok(())
				}
			} else {
				T::Slash::on_unbalanced(T::Currency::slash_reserved(&provider, deposit).0);
				Self::deposit_event(Event::<T>::PreimageInvalid(metaverse_id, proposal_hash, referendum_id));
				Err(Error::<T>::PreimageInvalid.into())
			}
		} else {
			Self::deposit_event(Event::<T>::PreimageMissing(metaverse_id, proposal_hash, referendum_id));
			Err(Error::<T>::PreimageMissing.into())
		}
	}

	/// Internal update of locked funds for account
	fn update_lock(who: &T::AccountId) {
		let lock_needed = VotingOf::<T>::mutate(who, |voting| {
			voting.rejig(frame_system::Pallet::<T>::block_number());
			voting.locked_balance()
		});
		if lock_needed.is_zero() {
			T::Currency::remove_lock(GOVERNANCE_ID, who);
		} else {
			T::Currency::set_lock(GOVERNANCE_ID, who, lock_needed, WithdrawReasons::TRANSFER);
		}
	}
}

/// Decode `Compact<u32>` from the trie at given key.
fn decode_compact_u32_at(key: &[u8]) -> Option<u32> {
	// `Compact<u32>` takes at most 5 bytes.
	let mut buf = [0u8; 5];
	let bytes = sp_io::storage::read(&key, &mut buf, 0)?;
	// The value may be smaller than 5 bytes.
	let mut input = &buf[0..buf.len().min(bytes as usize)];
	match codec::Compact::<u32>::decode(&mut input) {
		Ok(c) => Some(c.0),
		Err(_) => {
			sp_runtime::print("Failed to decode compact u32 at:");
			sp_runtime::print(key);
			None
		}
	}
}
