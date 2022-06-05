// This file is part of Metaverse.Network & Bit.Country.

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

//! # Continuum Spot Module
//!
//! ## Overview
//!
//! Metaverse network is a multi-metaverse protocol so individual metaverse need to be connected and
//! located in the map. The map called Continuum on mainnet and Pioneer on canary network.
//!
//! Continuum Slot Protocol will determine the location of the metaverse by going through auction
//! and buy spot process. The slot will go through the auction process, highest bid before end time
//! will secure the slot unless the allow buy now feature is enabled, the slot will secure with
//! fixed price.
//!
//! The core module of Continuum Spot protocol. Continuum Spot engine is responsible for handling
//! slot registration, slot expression of interest, slot auction and good neighborhood voting
//! protocol.
//!
//! Continuum Spot Auction Process (rotate every x block):
//! - Slot Registration (Express of Interest) - metaverse owner can register for their favourite
//!   slot
//! - Highest registered slot will move to Auction slots.
//! - The Auction slot will move to Good neighborhood protocol to start voting by neighbor of the
//!   spot
//! - Simple majority negative voting applied - the bidder who has more than 51% vote nay will be
//!   rejected
//! - The auction will start on pallet_auction.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use codec::{Decode, Encode};
#[cfg(feature = "std")]
use frame_support::traits::{Currency, GenesisBuild, Get, LockableCurrency, ReservableCurrency};
use frame_support::{dispatch::DispatchResult, ensure, transactional, PalletId};
use frame_system::{ensure_root, ensure_signed};
use scale_info::TypeInfo;
use sp_runtime::traits::CheckedAdd;
use sp_runtime::{
	traits::{AccountIdConversion, One, Zero},
	DispatchError, Perbill, RuntimeDebug,
};
use sp_std::vec;
use sp_std::vec::Vec;

use auction_manager::{Auction, AuctionType, CheckAuctionItemHandler, ListingLevel};
use core_primitives::MetaverseTrait;
pub use pallet::*;
use primitives::{continuum::MapTrait, ItemId, MapSpotId, MetaverseId, SpotId};
pub use types::*;
pub use vote::*;

mod types;
mod vote;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum ContinuumAuctionSlotStatus {
	/// Accept participation
	AcceptParticipates,
	/// Progressing at Good Neighborhood Protocol
	GNPStarted,
	/// Auction confirmed
	GNPConfirmed,
}

/// Information of EOI on Continuum spot
#[cfg_attr(feature = "std", derive(PartialEq, Eq))]
#[derive(Encode, Decode, Clone, RuntimeDebug, TypeInfo)]
pub struct SpotEOI<AccountId> {
	spot_id: SpotId,
	participants: Vec<AccountId>,
}

/// Information of an active auction slot
#[cfg_attr(feature = "std", derive(PartialEq, Eq))]
#[derive(Encode, Decode, Clone, RuntimeDebug, TypeInfo)]
pub struct AuctionSlot<BlockNumber, AccountId> {
	spot_id: SpotId,
	participants: Vec<AccountId>,
	active_session_index: BlockNumber,
	status: ContinuumAuctionSlotStatus,
}

#[frame_support::pallet]
pub mod pallet {
	use frame_support::traits::ExistenceRequirement;
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::OriginFor;

	use core_primitives::TokenType;
	use primitives::{AuctionId, MapSpotId};

	use super::*;

	pub(crate) type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// New Slot Duration
		/// How long the new auction slot will be released. If set to zero, no new auctions are
		/// generated
		#[pallet::constant]
		type SessionDuration: Get<Self::BlockNumber>;
		/// Auction Slot Chilling Duration
		/// How long the participates in the New Auction Slots will get confirmed by neighbours
		#[pallet::constant]
		type SpotAuctionChillingDuration: Get<Self::BlockNumber>;
		/// Emergency shutdown origin which allow cancellation in an emergency
		type EmergencyOrigin: EnsureOrigin<Self::Origin>;
		/// Auction Handler
		type AuctionHandler: Auction<Self::AccountId, Self::BlockNumber> + CheckAuctionItemHandler<BalanceOf<Self>>;
		/// Auction duration
		#[pallet::constant]
		type AuctionDuration: Get<Self::BlockNumber>;
		/// Continuum Treasury
		#[pallet::constant]
		type ContinuumTreasury: Get<PalletId>;
		/// Currency
		type Currency: ReservableCurrency<Self::AccountId>
			+ LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
		/// Source of Metaverse Network Info
		type MetaverseInfoSource: MetaverseTrait<Self::AccountId>;
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub initial_active_session: T::BlockNumber,
		pub initial_auction_rate: u8,
		pub initial_max_bound: (i32, i32),
		pub spot_price: BalanceOf<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				initial_active_session: Default::default(),
				initial_auction_rate: Default::default(),
				initial_max_bound: Default::default(),
				spot_price: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			CurrentIndex::<T>::set(self.initial_active_session);
			MaxDesiredAuctionSlot::<T>::set(self.initial_auction_rate);
			let eoi_slots: Vec<SpotEOI<T::AccountId>> = vec![];
			let gnp_slots: Vec<AuctionSlot<T::BlockNumber, T::AccountId>> = vec![];
			let active_auction_slots: Vec<AuctionSlot<T::BlockNumber, T::AccountId>> = vec![];
			EOISlots::<T>::insert(self.initial_active_session, eoi_slots);
			GNPSlots::<T>::insert(self.initial_active_session, gnp_slots);
			ActiveAuctionSlots::<T>::insert(self.initial_active_session, active_auction_slots);
			MaxBound::<T>::set(self.initial_max_bound);
			SpotPrice::<T>::set(self.spot_price);
		}
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	/// Get current active session
	#[pallet::storage]
	#[pallet::getter(fn current_session)]
	pub type CurrentIndex<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

	/// Continuum Spot
	#[pallet::storage]
	#[pallet::getter(fn get_continuum_spot)]
	pub type ContinuumSpots<T: Config> = StorageMap<_, Twox64Concat, SpotId, ContinuumSpot, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_map_spot)]
	pub type MapSpots<T: Config> = StorageMap<_, Twox64Concat, MapSpotId, MapSpot<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_map_spot_owner)]
	pub type MapSpotOwner<T: Config> = StorageMap<_, Twox64Concat, MapSpotId, T::AccountId, ValueQuery>;

	/// Continuum Spot Position
	#[pallet::storage]
	#[pallet::getter(fn get_continuum_position)]
	pub type ContinuumCoordinates<T: Config> = StorageMap<_, Twox64Concat, (i32, i32), SpotId, ValueQuery>;

	/// Active Auction Slots of current session index that accepting participants
	#[pallet::storage]
	#[pallet::getter(fn get_active_auction_slots)]
	pub type ActiveAuctionSlots<T: Config> =
		StorageMap<_, Twox64Concat, T::BlockNumber, Vec<AuctionSlot<T::BlockNumber, T::AccountId>>, OptionQuery>;

	/// Active Auction Slots that is currently conducting GN Protocol
	#[pallet::storage]
	#[pallet::getter(fn get_active_gnp_slots)]
	pub type GNPSlots<T: Config> =
		StorageMap<_, Twox64Concat, T::BlockNumber, Vec<AuctionSlot<T::BlockNumber, T::AccountId>>, OptionQuery>;

	/// Active set of EOI on Continuum Spot
	#[pallet::storage]
	#[pallet::getter(fn get_eoi_set)]
	pub type EOISlots<T: Config> = StorageMap<_, Twox64Concat, T::BlockNumber, Vec<SpotEOI<T::AccountId>>, ValueQuery>;

	/// Information of Continuum Spot Referendum
	#[pallet::storage]
	#[pallet::getter(fn get_continuum_referendum)]
	pub type ReferendumInfoOf<T: Config> =
		StorageMap<_, Twox64Concat, SpotId, ReferendumInfo<T::AccountId, T::BlockNumber>, OptionQuery>;

	/// All votes of a particular voter
	#[pallet::storage]
	#[pallet::getter(fn get_voting_info)]
	pub type VotingOf<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Voting<T::AccountId>>;

	/// Get max bound
	#[pallet::storage]
	#[pallet::getter(fn get_max_bound)]
	pub type MaxBound<T: Config> = StorageValue<_, (i32, i32), ValueQuery>;

	/// Record of all spot ids voting that in an emergency shut down
	#[pallet::storage]
	#[pallet::getter(fn get_cancellations)]
	pub type Cancellations<T: Config> = StorageMap<_, Twox64Concat, SpotId, bool, ValueQuery>;

	/// Maximum desired auction slots available per term
	#[pallet::storage]
	#[pallet::getter(fn get_max_desired_slot)]
	pub type MaxDesiredAuctionSlot<T: Config> = StorageValue<_, u8, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_spot_id)]
	pub type NextContinuumSpotId<T: Config> = StorageValue<_, SpotId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn allow_buy_now)]
	pub type AllowBuyNow<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn initial_spot_price)]
	pub type SpotPrice<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New express of interest
		NewExpressOfInterestAdded(T::AccountId, SpotId),
		/// New max bound set on continuum map
		NewMaxBoundSet((i32, i32)),
		/// Emergency shutdown is on
		ContinuumEmergencyShutdownEnabled(),
		/// Start new referendum
		NewContinuumReferendumStarted(T::BlockNumber, SpotId),
		/// Start new good neighbourhood protocol round
		NewContinuumNeighbourHoodProtocolStarted(T::BlockNumber, SpotId),
		/// Spot transferred
		ContinuumSpotTransferred(T::AccountId, T::AccountId, SpotId),
		/// New max auction slot set
		NewMaxAuctionSlotSet(u8),
		/// Rotated new auction slot
		NewAuctionSlotRotated(T::BlockNumber),
		/// Finalize vote
		FinalizedVote(SpotId),
		/// New Map Spot issued
		NewMapSpotIssued(MapSpotId, T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// No Active Auction Slot
		NoActiveAuctionSlot,
		/// No Active GNP List
		NoActiveGNP,
		/// Can't add EOI to Slot
		FailedEOIToSlot,
		/// Only send EOI once
		EOIAlreadyExists,
		/// No Active Session
		NoActiveSession,
		/// No Active Referendum
		NoActiveReferendum,
		/// Referendum is invalid
		ReferendumIsInValid,
		/// Tally Overflow
		TallyOverflow,
		/// Already shutdown
		AlreadyShutdown,
		/// Spot Not Found
		SpotNotFound,
		/// No permission
		NoPermission,
		/// Spot Owned
		SpotIsNotAvailable,
		/// Spot is out of bound
		SpotIsOutOfBound,
		/// Map Spot is not found
		MapSpotNotFound,
		/// Insufficient fund to buy
		InsufficientFund,
		/// Continuum Buynow is disable
		ContinuumBuyNowIsDisabled,
		/// Continuum Spot is in auction
		SpotIsInAuction,
		/// Map slot already exists
		MapSlotAlreadyExits,
		/// You are not the owner of the metaverse
		NotMetaverseOwner,
		/// Auction is not for map spot or does not exists
		InvalidSpotAuction,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Issue new map slot
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn issue_map_slot(origin: OriginFor<T>, coordinate: (i32, i32), slot_type: TokenType) -> DispatchResult {
			ensure_root(origin)?;

			ensure!(
				!MapSpots::<T>::contains_key(&coordinate),
				Error::<T>::MapSlotAlreadyExits
			);

			let max_bound = MaxBound::<T>::get();
			ensure!(
				(coordinate.0 >= max_bound.0 && max_bound.1 >= coordinate.0)
					&& (coordinate.1 >= max_bound.0 && max_bound.1 >= coordinate.1),
				Error::<T>::SpotIsOutOfBound
			);

			let map_slot = MapSpot {
				metaverse_id: None,
				owner: beneficiary,
				slot_type,
			};

			MapSpots::<T>::insert(coordinate.clone(), map_slot);
			MapSpotOwner::<T>::insert(coordinate.clone(), Self::account_id());

			Self::deposit_event(Event::<T>::NewMapSpotIssued(coordinate, Self::account_id()));

			Ok(())
		}

		/// Create new continuum slot with fixed price
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_new_buy_now(
			origin: OriginFor<T>,
			spot_id: MapSpotId,
			auction_type: AuctionType,
			value: BalanceOf<T>,
			end_time: T::BlockNumber,
		) -> DispatchResult {
			ensure_root(origin)?;

			let continuum_treasury = Self::account_id();

			// Ensure spot is belongs to treasury
			ensure!(
				Self::check_spot_ownership(&spot_id, &continuum_treasury)?,
				Error::<T>::NoPermission
			);

			if matches!(auction_type == AuctionType::BuyNow) {
				ensure!(AllowBuyNow::<T>::get() == true, Error::<T>::ContinuumBuyNowIsDisabled);
			}

			let now = <system::Pallet<T>>::block_number();
			T::AuctionHandler::create_auction(
				auction_type,
				ItemId::Spot(spot_id, Default::default()),
				Some(end_time),
				continuum_treasury,
				value,
				now,
				ListingLevel::Global,
				Perbill::from_percent(0u32),
			)?;

			Ok(())
		}

		/// Buy continuum slot with fixed price
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		#[transactional]
		pub fn buy_map_spot(
			origin: OriginFor<T>,
			auction_id: AuctionId,
			value: BalanceOf<T>,
			metaverse_id: MetaverseId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(
				T::MetaverseInfoSource::check_ownership(&sender, &metaverse_id),
				Error::<T>::NotMetaverseOwner
			);

			let auction_item = T::AuctionHandler::auction_item(auction_id)?;

			ensure!(auction_item.item_id.is_map_spot(), Error::<T>::InvalidSpotAuction);

			// Swap metaverse_id of the spot_id
			let spot_detail = auction_item.item_id.get_map_spot_detail()?;
			T::AuctionHandler::update_auction_item(auction_id, ItemId::Spot(*spot_detail.0, metaverse_id))?;

			T::AuctionHandler::buy_now_handler(sender, auction_id, value)?;

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		/// Whether council enable buy now option
		pub fn set_allow_buy_now(origin: OriginFor<T>, enable: bool) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			AllowBuyNow::<T>::set(enable);
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		/// Register continuum slot interest
		pub fn register_interest(
			origin: OriginFor<T>,
			metaverse_id: MetaverseId,
			coordinate: (i32, i32),
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			ensure!(
				T::MetaverseInfoSource::check_ownership(&sender, &metaverse_id),
				Error::<T>::NoPermission
			);
			let mut maybe_spot_id = Option::None;

			if ContinuumCoordinates::<T>::contains_key(coordinate) {
				let spot_id_from_coordinate = ContinuumCoordinates::<T>::get(coordinate);
				maybe_spot_id = Some(spot_id_from_coordinate);
			}

			let spot_id = Self::check_spot_ownership(maybe_spot_id, coordinate)?;

			// Get current active session
			let current_active_session_id = CurrentIndex::<T>::get();

			if EOISlots::<T>::contains_key(current_active_session_id) {
				// Mutate current active EOI Slot session
				EOISlots::<T>::try_mutate(current_active_session_id, |spot_eoi| -> DispatchResult {
					// Check if the interested Spot exists
					let interested_spot_index: Option<usize> = spot_eoi.iter().position(|x| x.spot_id == spot_id);
					match interested_spot_index {
						// Already got participants
						Some(index) => {
							// Works on existing eoi index
							let interested_spot = spot_eoi.get_mut(index).ok_or("No Spot EOI exist")?;

							interested_spot.participants.push(sender.clone());
						}
						// No participants - add one
						None => {
							// No spot found - first one in EOI
							let mut new_list: Vec<T::AccountId> = Vec::new();
							new_list.push(sender.clone());

							let _spot_eoi = SpotEOI {
								spot_id,
								participants: new_list,
							};
							spot_eoi.push(_spot_eoi);
						}
					}
					Ok(())
				})?;
			} else {
				// Never get to this logic but it's safe to handle it nicely.
				let mut eoi_slots: Vec<SpotEOI<T::AccountId>> = Vec::new();
				eoi_slots.push(SpotEOI {
					spot_id,
					participants: vec![sender.clone()],
				});
				EOISlots::<T>::insert(current_active_session_id, eoi_slots);
			}

			Self::deposit_event(Event::NewExpressOfInterestAdded(sender, spot_id));
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		// Council set maximum bound on continuum map
		pub fn set_max_bounds(origin: OriginFor<T>, new_bound: (i32, i32)) -> DispatchResultWithPostInfo {
			// Only execute by governance
			ensure_root(origin)?;
			MaxBound::<T>::set(new_bound.clone());
			Self::deposit_event(Event::NewMaxBoundSet(new_bound));
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		// Council set how many auction can run per period
		pub fn set_new_auction_rate(origin: OriginFor<T>, new_rate: u8) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			MaxDesiredAuctionSlot::<T>::set(new_rate.clone());
			Self::deposit_event(Event::NewMaxAuctionSlotSet(new_rate));
			Ok(().into())
		}
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn vote(origin: OriginFor<T>, id: SpotId, reject: AccountVote<T::AccountId>) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			Self::try_vote(&sender, id, reject)?;
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn emergency_shutdown(origin: OriginFor<T>, spot_id: SpotId) -> DispatchResultWithPostInfo {
			// Only some origins can execute this function
			T::EmergencyOrigin::ensure_origin(origin)?;

			ensure!(!Cancellations::<T>::contains_key(spot_id), Error::<T>::AlreadyShutdown);

			Cancellations::<T>::insert(spot_id, true);
			ReferendumInfoOf::<T>::remove(spot_id);

			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn account_id() -> T::AccountId {
		T::ContinuumTreasury::get().into_account()
	}
	//noinspection ALL
	// Started auction slot and referendum
	fn rotate_auction_slots(now: T::BlockNumber) -> DispatchResult {
		// Get current active session
		let current_active_session_id = CurrentIndex::<T>::get();
		// Change status of all current active auction slots
		// Move EOI to Auction Slots
		Self::eoi_to_auction_slots(current_active_session_id, now)?;
		// Finalise due vote
		Self::finalize_vote(now);
		let active_auction_slots = <ActiveAuctionSlots<T>>::get(&current_active_session_id);

		match active_auction_slots {
			Some(s) => {
				// Move current auctions slot to start GN Protocol
				if s.len() > 0 {
					let started_gnp_auction_slots: Vec<_> = s
						.iter()
						.map(|x| {
							let mut t = x.clone();
							t.status = ContinuumAuctionSlotStatus::GNPStarted;
							t
						})
						.collect();
					// Move active auction slots to GNP
					GNPSlots::<T>::insert(now, started_gnp_auction_slots.clone());
					// Start referedum
					Self::start_gnp_protocol(started_gnp_auction_slots, now)?;
				}
			}
			None => {}
		}
		// Remove the old active auction slots
		ActiveAuctionSlots::<T>::remove(&current_active_session_id);

		CurrentIndex::<T>::set(now.clone());
		Self::deposit_event(Event::NewAuctionSlotRotated(now));
		Ok(().into())
	}

	fn finalize_vote(now: T::BlockNumber) -> DispatchResult {
		let recent_slots = GNPSlots::<T>::get(now).ok_or(Error::<T>::NoActiveReferendum)?;

		for mut recent_slot in recent_slots.into_iter() {
			let referendum_info: ReferendumStatus<T::AccountId, T::BlockNumber> =
				Self::referendum_status(recent_slot.spot_id)?;

			if referendum_info.end == now {
				let banned_list: Vec<T::AccountId> = referendum_info
					.tallies
					.into_iter()
					.filter(|t| Self::check_approved(t) == true)
					.map(|tally| tally.who)
					.collect();

				for banned_account in banned_list {
					let account_index = recent_slot
						.participants
						.iter()
						.position(|x| *x == banned_account)
						.unwrap();
					recent_slot.participants.remove(account_index);
					recent_slot.status = ContinuumAuctionSlotStatus::GNPConfirmed;
				}
				let treasury = Self::account_id();
				// From treasury spot
				T::AuctionHandler::create_auction(
					AuctionType::Auction,
					ItemId::Spot(recent_slot.spot_id, Default::default()),
					Some(now + T::AuctionDuration::get()),
					treasury,
					Default::default(),
					now,
					ListingLevel::NetworkSpot(recent_slot.participants),
					Perbill::from_percent(0u32),
				)?;
				Self::deposit_event(Event::FinalizedVote(referendum_info.spot_id))
			}
		}

		Ok(())
	}

	fn start_gnp_protocol(
		slots: Vec<AuctionSlot<T::BlockNumber, T::AccountId>>,
		end: T::BlockNumber,
	) -> DispatchResult {
		for slot in slots {
			let end = end + T::SessionDuration::get();
			Self::start_referendum(end, slot.spot_id.clone())?;
			Self::deposit_event(Event::NewContinuumReferendumStarted(end, slot.spot_id));
		}
		Ok(())
	}

	fn start_referendum(end: T::BlockNumber, spot_id: SpotId) -> Result<SpotId, DispatchError> {
		let spot = ContinuumSpots::<T>::get(spot_id);
		let neighbors = spot.find_neighbour();
		let mut available_neighbors: u8 = 0;

		for (x, y) in neighbors {
			if ContinuumCoordinates::<T>::contains_key((x, y)) {
				available_neighbors = available_neighbors.checked_add(One::one()).ok_or("Overflow")?;
			}
		}

		let mut status: ReferendumStatus<T::AccountId, T::BlockNumber> = ReferendumStatus {
			end,
			spot_id,
			tallies: Default::default(),
		};

		let item: ReferendumInfo<T::AccountId, T::BlockNumber> = ReferendumInfo::Ongoing(status);
		ReferendumInfoOf::<T>::insert(spot_id, item);
		Self::deposit_event(Event::NewContinuumReferendumStarted(end, spot_id));
		Ok(spot_id)
	}

	fn eoi_to_auction_slots(active_session: T::BlockNumber, now: T::BlockNumber) -> DispatchResult {
		// Get maximum desired slots
		let desired_slots = MaxDesiredAuctionSlot::<T>::get();
		let session_duration = T::SessionDuration::get();

		// Get active EOI and add the top N to new Auction Slots
		let mut current_eoi_slots: Vec<SpotEOI<T::AccountId>> = EOISlots::<T>::get(active_session);

		current_eoi_slots.sort_by_key(|eoi_slot| eoi_slot.participants.len());
		// Get highest ranked slot
		let mut new_valid_auction_slot: Vec<AuctionSlot<T::BlockNumber, T::AccountId>> = Vec::new();
		let highest_ranked_sorted: Vec<SpotEOI<T::AccountId>> = current_eoi_slots
			.iter()
			.map(|x| x.clone())
			.take(desired_slots as usize)
			.collect::<Vec<SpotEOI<T::AccountId>>>();
		// Add highest ranked EOI to New Active Auction slot
		for (_x, item) in highest_ranked_sorted.iter().enumerate() {
			let auction_slot = AuctionSlot {
				spot_id: item.spot_id,
				participants: item.participants.clone(),
				active_session_index: now.checked_add(&session_duration).ok_or("Overflow")?,
				status: ContinuumAuctionSlotStatus::AcceptParticipates,
			};
			new_valid_auction_slot.push(auction_slot);
		}

		ActiveAuctionSlots::<T>::insert(now, new_valid_auction_slot);
		// Remove EOISlot
		EOISlots::<T>::remove(active_session);
		let empty_eoi_spots: Vec<SpotEOI<T::AccountId>> = Vec::new();
		// Add new EOISlot for current session - ensure active session has entry
		EOISlots::<T>::insert(now, empty_eoi_spots);
		Ok(())
	}

	fn try_vote(who: &T::AccountId, spot_id: SpotId, vote: AccountVote<T::AccountId>) -> DispatchResult {
		let status = Self::referendum_status(spot_id)?;

		let spot = ContinuumSpots::<T>::get(spot_id);
		let neighbors = spot.find_neighbour();
		let mut is_neighbour: bool = false;

		for (x, y) in neighbors {
			// if spot exists
			let neighbor_spot_id = ContinuumCoordinates::<T>::get((x, y));
			let continuum_spot = ContinuumSpots::<T>::get(neighbor_spot_id);
			if T::MetaverseInfoSource::check_ownership(&who, &continuum_spot.metaverse_id) {
				is_neighbour = true;
				break;
			}
		}

		ensure!(is_neighbour, Error::<T>::NoPermission);

		VotingOf::<T>::try_mutate(who, |maybe_voting| -> DispatchResult {
			match maybe_voting {
				Some(voting) => {
					let ref mut votes = voting.votes;
					match votes.binary_search_by_key(&spot_id, |i| i.0) {
						// Already voted
						Ok(_i) => {}
						Err(i) => {
							// Haven't vote for this spot id
							// Add votes under user
							let new_vote: AccountVote<T::AccountId> = vote.clone();
							let who = new_vote.vote_who();
							votes.insert(i, (spot_id, vote.clone()));

							// Find existing tally of bidder
							for mut tally in status.tallies {
								// Existing vote
								if tally.who == who.who {
									tally.add(vote.clone()).ok_or(Error::<T>::TallyOverflow)?
								}
							}
						}
					}
				}
				None => {
					// No voting exists
					let mut new_vote: Vec<(SpotId, AccountVote<T::AccountId>)> = Vec::new();
					new_vote.push((spot_id, vote.clone()));
					let vote_o = Voting { votes: new_vote };
					VotingOf::<T>::insert(who.clone(), vote_o);
				}
			}

			Ok(())
		})
	}

	fn referendum_status(spot_id: SpotId) -> Result<ReferendumStatus<T::AccountId, T::BlockNumber>, DispatchError> {
		let info = ReferendumInfoOf::<T>::get(spot_id).ok_or(Error::<T>::ReferendumIsInValid)?;
		Self::ensure_ongoing(info.into())
	}

	/// Ok if the given referendum is active, Err otherwise
	fn ensure_ongoing(
		r: ReferendumInfo<T::AccountId, T::BlockNumber>,
	) -> Result<ReferendumStatus<T::AccountId, T::BlockNumber>, DispatchError> {
		match r {
			ReferendumInfo::Ongoing(s) => Ok(s),
			_ => Err(Error::<T>::ReferendumIsInValid.into()),
		}
	}

	fn do_transfer_spot(
		spot_id: SpotId,
		from: &T::AccountId,
		to: &(T::AccountId, MetaverseId),
	) -> Result<SpotId, DispatchError> {
		Self::transfer_spot(spot_id, from, to)
	}

	fn check_approved(tally: &ContinuumSpotTally<T::AccountId>) -> bool {
		let nay_ratio = tally.turnout.checked_div(tally.nays).unwrap_or(0);
		let nay_percent = nay_ratio.checked_mul(100).unwrap_or(0);

		nay_percent > 51
	}

	fn check_spot_ownership(spot_id: &SpotId, owner: &T::AccountId) -> Result<bool, DispatchError> {
		let spot_owner = MapSpotOwner::<T>::get(spot_id).ok_or(Error::<T>::MapSpotNotFound)?;

		Ok(spot_owner == owner)
	}
}

impl<T: Config> MapTrait<T::AccountId> for Pallet<T> {
	fn transfer_spot(spot_id: MapSpotId, from: &T::AccountId, to: &T::AccountId) -> Result<MapSpotId, DispatchError> {
		ensure!(
			!T::AuctionHandler::check_item_in_auction(ItemId::Spot(spot_id)),
			Error::<T>::SpotIsInAuction
		);

		MapSpots::<T>::try_mutate(spot_id, |maybe_spot| -> Result<MapSpotId, DispatchError> {
			// Ensure only treasury can transferred
			let treasury = Self::account_id();
			ensure!(from == treasury, Error::<T>::NoPermission);

			let mut spot = maybe_spot;
			spot.owner = to;
			Ok(spot_id)
		})
	}
}
