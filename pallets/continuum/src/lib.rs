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
use frame_support::{
	dispatch::DispatchResult,
	ensure, log,
	traits::{Currency, Get, LockableCurrency, ReservableCurrency},
	transactional, PalletId,
};
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
pub use weights::WeightInfo;

mod types;
mod vote;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod weights;

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
	use sp_arithmetic::traits::UniqueSaturatedInto;

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
		type AuctionHandler: Auction<Self::AccountId, Self::BlockNumber, Balance = BalanceOf<Self>>
			+ CheckAuctionItemHandler<BalanceOf<Self>>;
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
		/// Weight implementation for estate extrinsics
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	/// Continuum Spot
	#[pallet::storage]
	#[pallet::getter(fn get_continuum_spot)]
	pub type ContinuumSpots<T: Config> = StorageMap<_, Twox64Concat, SpotId, ContinuumSpot, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_map_spot)]
	pub type MapSpots<T: Config> = StorageMap<_, Twox64Concat, MapSpotId, MapSpot<T::AccountId>>;

	#[pallet::storage]
	#[pallet::getter(fn get_map_spot_owner)]
	pub type MapSpotOwner<T: Config> = StorageMap<_, Twox64Concat, MapSpotId, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn get_map_metaverse)]
	pub type MetaverseMap<T: Config> = StorageMap<_, Twox64Concat, MetaverseId, MapSpotId>;

	/// Continuum Spot Position
	#[pallet::storage]
	#[pallet::getter(fn get_continuum_position)]
	pub type ContinuumCoordinates<T: Config> = StorageMap<_, Twox64Concat, (i32, i32), SpotId, ValueQuery>;

	/// Get max bound
	#[pallet::storage]
	#[pallet::getter(fn get_max_bound)]
	pub type MaxBound<T: Config> = StorageValue<_, (i32, i32), ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_spot_id)]
	pub type NextContinuumSpotId<T: Config> = StorageValue<_, SpotId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn allow_buy_now)]
	pub type AllowBuyNow<T: Config> = StorageValue<_, bool, ValueQuery>;

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
		ContinuumSpotTransferred(T::AccountId, T::AccountId, MapSpotId),
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
		MapSpotAlreadyExits,
		/// You are not the owner of the metaverse
		NotMetaverseOwner,
		/// Metaverse already secured the spot
		MetaverseAlreadyGotSpot,
		/// Auction is not for map spot or does not exists
		InvalidSpotAuction,
		/// Metaverse has no deployed land.
		MetaverseHasNotDeployedAnyLand,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Issue new map slot
		#[pallet::weight(T::WeightInfo::issue_map_slot())]
		pub fn issue_map_slot(origin: OriginFor<T>, coordinate: (i32, i32), slot_type: TokenType) -> DispatchResult {
			ensure_root(origin)?;

			ensure!(
				!MapSpots::<T>::contains_key(&coordinate),
				Error::<T>::MapSpotAlreadyExits
			);

			let max_bound = MaxBound::<T>::get();
			ensure!(
				(coordinate.0 >= max_bound.0 && max_bound.1 >= coordinate.0)
					&& (coordinate.1 >= max_bound.0 && max_bound.1 >= coordinate.1),
				Error::<T>::SpotIsOutOfBound
			);

			let map_slot = MapSpot {
				metaverse_id: None,
				owner: Self::account_id(),
				slot_type,
			};

			MapSpots::<T>::insert(coordinate.clone(), map_slot);

			Self::deposit_event(Event::<T>::NewMapSpotIssued(coordinate, Self::account_id()));
			Ok(())
		}

		/// Create new map auction
		#[pallet::weight(T::WeightInfo::create_new_auction())]
		pub fn create_new_auction(
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

			if matches!(auction_type, AuctionType::BuyNow) {
				ensure!(AllowBuyNow::<T>::get(), Error::<T>::ContinuumBuyNowIsDisabled);
			}

			let now = <frame_system::Pallet<T>>::block_number();
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
		#[pallet::weight(T::WeightInfo::buy_map_spot())]
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

			ensure!(
				!MetaverseMap::<T>::contains_key(&metaverse_id),
				Error::<T>::MetaverseAlreadyGotSpot
			);

			ensure!(
				T::MetaverseInfoSource::check_if_metaverse_has_any_land(metaverse_id.clone())?,
				Error::<T>::MetaverseHasNotDeployedAnyLand
			);

			let auction_item = T::AuctionHandler::auction_item(auction_id).ok_or(Error::<T>::InvalidSpotAuction)?;

			ensure!(auction_item.item_id.is_map_spot(), Error::<T>::InvalidSpotAuction);

			// Swap metaverse_id of the spot_id
			let spot_detail = auction_item
				.item_id
				.get_map_spot_detail()
				.ok_or(Error::<T>::InvalidSpotAuction)?;
			T::AuctionHandler::update_auction_item(auction_id, ItemId::Spot(*spot_detail.0, metaverse_id))?;

			T::AuctionHandler::buy_now_handler(sender, auction_id, value)?;
			Ok(())
		}

		/// Buy continuum slot with fixed price
		#[pallet::weight(T::WeightInfo::bid_map_spot())]
		#[transactional]
		pub fn bid_map_spot(
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

			ensure!(
				!MetaverseMap::<T>::contains_key(&metaverse_id),
				Error::<T>::MetaverseAlreadyGotSpot
			);

			ensure!(
				T::MetaverseInfoSource::check_if_metaverse_has_any_land(metaverse_id.clone())?,
				Error::<T>::MetaverseHasNotDeployedAnyLand
			);

			let auction_item = T::AuctionHandler::auction_item(auction_id).ok_or(Error::<T>::InvalidSpotAuction)?;

			ensure!(auction_item.item_id.is_map_spot(), Error::<T>::InvalidSpotAuction);

			// Swap metaverse_id of the spot_id
			let spot_detail = auction_item
				.item_id
				.get_map_spot_detail()
				.ok_or(Error::<T>::InvalidSpotAuction)?;
			T::AuctionHandler::update_auction_item(auction_id, ItemId::Spot(*spot_detail.0, metaverse_id))?;

			T::AuctionHandler::auction_bid_handler(sender, auction_id, value)?;

			Ok(())
		}

		#[pallet::weight(T::WeightInfo::set_allow_buy_now())]
		/// Whether council enable buy now option
		pub fn set_allow_buy_now(origin: OriginFor<T>, enable: bool) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			AllowBuyNow::<T>::set(enable);
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
	}
}

impl<T: Config> Pallet<T> {
	pub fn account_id() -> T::AccountId {
		T::ContinuumTreasury::get().into_account()
	}
	// noinspection ALL
	fn check_spot_ownership(spot_id: &MapSpotId, owner: &T::AccountId) -> Result<bool, DispatchError> {
		let spot_info = MapSpots::<T>::get(spot_id).ok_or(Error::<T>::MapSpotNotFound)?;
		Ok(spot_info.owner == *owner)
	}
}

impl<T: Config> MapTrait<T::AccountId> for Pallet<T> {
	fn transfer_spot(
		spot_id: MapSpotId,
		from: T::AccountId,
		to: (T::AccountId, MetaverseId),
	) -> Result<MapSpotId, DispatchError> {
		MapSpots::<T>::try_mutate(spot_id, |maybe_spot| -> Result<MapSpotId, DispatchError> {
			// Ensure only treasury can transferred
			let treasury = Self::account_id();
			ensure!(from == treasury, Error::<T>::NoPermission);

			let mut spot = maybe_spot.as_mut().ok_or(Error::<T>::MapSpotNotFound)?;
			spot.owner = to.clone().0;
			spot.metaverse_id = Some(to.1);

			Self::deposit_event(Event::<T>::ContinuumSpotTransferred(from, to.0, spot_id));
			MetaverseMap::<T>::insert(to.1, spot_id);

			Ok(spot_id)
		})
	}
}
