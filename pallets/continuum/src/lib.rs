// This file is part of Bit.Country.

// Copyright (C) 2020-2021 Bit.Country.
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
    dispatch::DispatchResult, ensure,
    traits::{Get, Vec},
};
use frame_system::{self as system, ensure_root, ensure_signed};
use primitives::{Balance, CountryId, CurrencyId, SpotId, ItemId, continuum::Continuum};
use sp_runtime::{traits::{AccountIdConversion, One, Zero, CheckedDiv, CheckedAdd}, DispatchError, ModuleId, RuntimeDebug, FixedPointNumber};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use auction_manager::{Auction};
use frame_support::traits::{Currency, ReservableCurrency, LockableCurrency};

// use crate::pallet::{Config, Pallet, ActiveAuctionSlots};

mod vote;
mod types;

pub use vote::*;
pub use types::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
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
#[derive(Encode, Decode, Clone, RuntimeDebug)]
pub struct SpotEOI<AccountId> {
    spot_id: SpotId,
    participants: Vec<AccountId>,
}

/// Information of an active auction slot
#[cfg_attr(feature = "std", derive(PartialEq, Eq))]
#[derive(Encode, Decode, Clone, RuntimeDebug)]
pub struct AuctionSlot<BlockNumber, AccountId> {
    spot_id: SpotId,
    participants: Vec<AccountId>,
    active_session_index: BlockNumber,
    status: ContinuumAuctionSlotStatus,
}

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::OriginFor;
    use super::*;

    pub(crate) type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// New Slot Duration
        /// How long the new auction slot will be released. If set to zero, no new auctions are generated
        type SessionDuration: Get<Self::BlockNumber>;
        /// Auction Slot Chilling Duration
        /// How long the participates in the New Auction Slots will get confirmed by neighbours
        type SpotAuctionChillingDuration: Get<Self::BlockNumber>;
        /// Emergency shutdown origin which allow cancellation in an emergency
        type EmergencyOrigin: EnsureOrigin<Self::Origin>;
        /// Auction Handler
        type AuctionHandler: Auction<Self::AccountId, Self::BlockNumber>;
        /// Auction duration
        type AuctionDuration: Get<Self::BlockNumber>;
        /// Continuum Treasury
        type ContinuumTreasury: Get<ModuleId>;
        /// Currency
        type Currency: ReservableCurrency<Self::AccountId>
        + LockableCurrency<Self::AccountId, Moment=Self::BlockNumber>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        /// Initialization
        fn on_initialize(now: T::BlockNumber) -> Weight {
            let auction_duration: T::BlockNumber = T::SessionDuration::get();
            if !auction_duration.is_zero() && (now % auction_duration).is_zero() {
                Self::rotate_auction_slots(now);
                T::BlockWeights::get().max_block
            } else {
                0
            }
        }
    }

    /// Get current active session
    #[pallet::storage]
    #[pallet::getter(fn current_session)]
    pub type CurrentIndex<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

    /// Continuum Spot
    #[pallet::storage]
    #[pallet::getter(fn get_continuum_spot)]
    pub type ContinuumSpots<T: Config> = StorageMap<_, Twox64Concat, SpotId, ContinuumSpot, OptionQuery>;

    /// Continuum Spot Position
    #[pallet::storage]
    #[pallet::getter(fn get_continuum_position)]
    pub type ContinuumCoordinates<T: Config> = StorageMap<_, Twox64Concat, (i32, i32), SpotId, OptionQuery>;

    /// Active Auction Slots of current session index that accepting participants
    #[pallet::storage]
    #[pallet::getter(fn get_active_auction_slots)]
    pub type ActiveAuctionSlots<T: Config> = StorageMap<_, Twox64Concat, T::BlockNumber, Vec<AuctionSlot<T::BlockNumber, T::AccountId>>, OptionQuery>;

    /// Active Auction Slots that is currently conducting GN Protocol
    #[pallet::storage]
    #[pallet::getter(fn get_active_gnp_slots)]
    pub type GNPSlots<T: Config> = StorageMap<_, Twox64Concat, T::BlockNumber, Vec<AuctionSlot<T::BlockNumber, T::AccountId>>, OptionQuery>;

    /// Active set of EOI on Continuum Spot
    #[pallet::storage]
    #[pallet::getter(fn get_eoi_set)]
    pub type EOISlots<T: Config> = StorageMap<_, Twox64Concat, T::BlockNumber, Vec<SpotEOI<T::AccountId>>, ValueQuery>;

    /// Information of Continuum Spot Referendum
    #[pallet::storage]
    #[pallet::getter(fn get_continuum_referendum)]
    pub type ReferendumInfoOf<T: Config> = StorageMap<_, Twox64Concat, SpotId, ReferendumInfo<T::AccountId, T::BlockNumber>, OptionQuery>;

    /// All votes of a particular voter
    #[pallet::storage]
    #[pallet::getter(fn get_voting_info)]
    pub type VotingOf<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Voting<T::AccountId>, ValueQuery>;

    /// Get max bound
    #[pallet::storage]
    #[pallet::getter(fn get_max_bound)]
    pub type MaxBound<T: Config> = StorageValue<_, (u32, u32), ValueQuery>;

    /// Record of all spot ids voting that in an emergency shut down
    #[pallet::storage]
    #[pallet::getter(fn get_cancellations)]
    pub type Cancellations<T: Config> = StorageMap<_, Twox64Concat, SpotId, bool, ValueQuery>;

    /// Maximum desired auction slots available per term
    #[pallet::storage]
    #[pallet::getter(fn get_max_desired_slot)]
    pub type MaxDesiredAuctionSlot<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T: Config> {
        // New express of interest
        NewExpressOfInterestAdded(T::AccountId, SpotId),
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
    }


    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn register_interest(origin: OriginFor<T>, spot_id: SpotId) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            //TODO Ensure AccountId own a country
            //TODO Spot has no owner

            // Get current active session
            let current_active_session_id = CurrentIndex::<T>::get();

            ensure!(EOISlots::<T>::contains_key(current_active_session_id), Error::<T>::NoActiveSession);

            // Mutate current active EOI Slot session
            EOISlots::<T>::try_mutate(current_active_session_id, |spot_eoi| -> DispatchResult {

                // Check if the interested Spot exists
                let interested_spot_index: Option<usize> = spot_eoi.iter().position(|x| x.spot_id == spot_id);
                match interested_spot_index {
                    // Already got participants
                    Some(index) => {
                        // Works on existing eoi index
                        let interested_spot = spot_eoi.get_mut(index).ok_or("No Spot EOI exist")?;

                        if interested_spot.participants.len() == 0 {
                            interested_spot.participants.push(sender);
                        } else {
                            interested_spot.participants.push(sender);
                        }
                    }
                    // No participants - add one
                    None => {
                        // No spot found - first one in EOI
                        let mut new_list: Vec<T::AccountId> = Vec::new();
                        new_list.push(sender);

                        let _spot_eoi = SpotEOI {
                            spot_id,
                            participants: new_list,
                        };
                        spot_eoi.push(_spot_eoi);
                    }
                }
                Ok(())
            })?;

            //TODO Emit event
            Ok(().into())
        }
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn enable_bidder_rejection_voting(origin: OriginFor<T>, spot_id: SpotId) -> DispatchResultWithPostInfo {
            let root = ensure_root(origin);
            //TODO Check if neighborhood
            //Enable democracy pallet
            //Propose bidder removal action on democracy
            Ok(().into())
        }
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_max_bounds(origin: OriginFor<T>, new_bound: (u32, u32)) -> DispatchResultWithPostInfo {
            // Only execute by governance
            ensure_root(origin);
            MaxBound::<T>::set(new_bound);
            //TODO Emit event
            Ok(().into())
        }
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_new_auction_rate(origin: OriginFor<T>, new_rate: u32) -> DispatchResultWithPostInfo {
            ensure_root(origin);
            MaxDesiredAuctionSlot::<T>::set(new_rate);
            //TODO Emit event
            Ok(().into())
        }
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn vote(origin: OriginFor<T>, id: SpotId, reject: AccountVote<T::AccountId>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            Self::try_vote(&sender, id, reject);
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn emergency_shutdown(origin: OriginFor<T>, spot_id: SpotId) -> DispatchResultWithPostInfo {
            // Only some origins can execute this function
            T::EmergencyOrigin::ensure_origin(origin)?;

            // let status = Self::referendum_status(spot_id)?;

            ensure!(!Cancellations::<T>::contains_key(spot_id), Error::<T>::AlreadyShutdown);

            Cancellations::<T>::insert(spot_id, true);
            ReferendumInfoOf::<T>::remove(spot_id);

            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T>
{
    fn account_id() -> T::AccountId {
        T::ContinuumTreasury::get().into_account()
    }
    //noinspection ALL
    fn rotate_auction_slots(now: T::BlockNumber) -> DispatchResult {
        // Get current active session
        let current_active_session_id = CurrentIndex::<T>::get();
        // Change status of all current active auction slots
        let mut active_auction_slots = <ActiveAuctionSlots<T>>::get(&current_active_session_id).ok_or(Error::<T>::NoActiveAuctionSlot)?;

        // Move current auctions slot to start GN Protocol
        let started_gnp_auction_slots: Vec<_> =
            active_auction_slots
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
        //TODO Emit event Auction slot start GNP

        // Remove the old active auction slots
        ActiveAuctionSlots::<T>::remove(&current_active_session_id);
        // Move EOI to Auction Slots
        Self::eoi_to_auction_slots(current_active_session_id, now)?;
        // Finalise due vote
        Self::finalize_vote(now);

        CurrentIndex::<T>::set(now);
        //TODO Emit event
        Ok(().into())
    }

    fn finalize_vote(now: T::BlockNumber) -> DispatchResult {
        let recent_slots = GNPSlots::<T>::get(now).ok_or(Error::<T>::NoActiveReferendum)?;

        for mut recent_slot in recent_slots.into_iter() {
            let referendum_info: ReferendumStatus<T::AccountId, T::BlockNumber> = Self::referendum_status(recent_slot.spot_id)?;

            if referendum_info.end == now {
                // let tallies = referendum_info.tallies;
                let banned_list: Vec<T::AccountId> =
                    referendum_info.tallies.into_iter()
                        .filter(|mut t| Self::check_approved(t) == true)
                        .map(|tally| tally.who)
                        .collect();

                for banned_account in banned_list {
                    let account_index = recent_slot.participants.iter().position(|x| *x == banned_account).unwrap();
                    recent_slot.participants.remove(account_index);
                    recent_slot.status = ContinuumAuctionSlotStatus::GNPConfirmed;
                }
                let treasury = Self::account_id();
                //From treasury spot
                T::AuctionHandler::create_auction(ItemId::Spot(recent_slot.spot_id, Default::default()), Some(now + T::AuctionDuration::get()), treasury, Default::default(), now);
                //TODO Emit event
            }
        }

        Ok(())
    }

    fn start_gnp_protocol(slots: Vec<AuctionSlot<T::BlockNumber, T::AccountId>>, end: T::BlockNumber) -> DispatchResult {
        for slot in slots {
            let end = end + T::SessionDuration::get();
            Self::start_referendum(end, slot.spot_id);
            //TODO Emit event
        }
        Ok(())
    }

    fn start_referendum(
        end: T::BlockNumber,
        spot_id: SpotId,
    ) -> Result<SpotId, DispatchError> {
        let spot = ContinuumSpots::<T>::get(spot_id).ok_or(Error::<T>::SpotNotFound)?;
        let neighbors = spot.find_neighbour();
        let mut available_neighbors: u8 = 0;

        for (x, y) in neighbors {
            match ContinuumCoordinates::<T>::get((x, y)) {
                Some(i) => {
                    available_neighbors = available_neighbors.checked_add(One::one()).ok_or("Overflow")?;
                }
                _ => ()
            }
        }

        let mut status: ReferendumStatus<T::AccountId, T::BlockNumber> = ReferendumStatus {
            end,
            spot_id,
            tallies: Default::default(),
        };

        for _i in 0..available_neighbors {
            let initial_tally: ContinuumSpotTally<T::AccountId> = ContinuumSpotTally {
                nays: One::one(),
                who: Default::default(),
                turnout: available_neighbors,
            };
            status.tallies.push(initial_tally);
        }

        let item: ReferendumInfo<T::AccountId, T::BlockNumber> = ReferendumInfo::Ongoing(status);
        ReferendumInfoOf::<T>::insert(spot_id, item);
        //TODO Emit event
        Ok(spot_id)
    }


    fn eoi_to_auction_slots(active_session: T::BlockNumber, now: T::BlockNumber) -> DispatchResult {
        // Get maximum desired slots
        let desired_slots = MaxDesiredAuctionSlot::<T>::get();

        // Get active EOI and add the top N to new Auction Slots
        let mut current_eoi_slots: Vec<SpotEOI<T::AccountId>> = EOISlots::<T>::get(active_session);

        current_eoi_slots.sort_by_key(|eoi_slot| eoi_slot.participants.len());
        // Get highest ranked slot

        let mut new_valid_auction_slot: Vec<AuctionSlot<T::BlockNumber, T::AccountId>> = Vec::new();
        let highest_ranked_sorted: Vec<SpotEOI<T::AccountId>> = current_eoi_slots.iter().map(|x| x.clone()).take(desired_slots as usize).collect::<Vec<SpotEOI<T::AccountId>>>();
        // Add highest ranked EOI to New Active Auction slot
        for (x, item) in highest_ranked_sorted.iter().enumerate() {
            let auction_slot = AuctionSlot {
                spot_id: item.spot_id,
                participants: item.participants.clone(),
                active_session_index: now,
                status: ContinuumAuctionSlotStatus::AcceptParticipates,
            };
            new_valid_auction_slot.push(auction_slot);
        }

        ActiveAuctionSlots::<T>::insert(now, new_valid_auction_slot);
        //Remove EOISlot
        EOISlots::<T>::remove(active_session);
        let empty_eoi_spots: Vec<SpotEOI<T::AccountId>> = Vec::new();
        //Add new EOISlot for current session - ensure active session has entry
        EOISlots::<T>::insert(now, empty_eoi_spots);
        //TODO Emit event
        Ok(())
    }

    fn try_vote(who: &T::AccountId, spot_id: SpotId, vote: AccountVote<T::AccountId>) -> DispatchResult {
        let mut status = Self::referendum_status(spot_id)?;

        VotingOf::<T>::try_mutate(who, |mut voting| -> DispatchResult {
            let mut votes = &mut voting.votes;
            match votes.binary_search_by_key(&spot_id, |i| i.0) {
                //Already voted
                Ok(i) => {}
                Err(i) => {
                    //Haven't vote for this spot id
                    // Add votes under user
                    let new_vote: AccountVote<T::AccountId> = vote.clone();
                    let who = new_vote.vote_who();
                    votes.insert(i, (spot_id, vote.clone()));

                    let mut tallies = status.tallies.clone();

                    //Find existing tally of bidder
                    for mut tally in status.tallies {
                        // Existing vote
                        if tally.who == who.who {
                            tally.add(vote.clone()).ok_or(Error::<T>::TallyOverflow)?
                        } else {
                            //Create new vote
                        }
                    }
                }
            }
            Ok(())
        })
    }

    fn referendum_status(spot_id: SpotId) -> Result<ReferendumStatus<T::AccountId, T::BlockNumber>, DispatchError> {
        let info = ReferendumInfoOf::<T>::get(spot_id).ok_or(Error::<T>::ReferendumIsInValid)?;
        Self::ensure_ongoing(info.into())
    }

    fn referendum_info(spot_id: SpotId) -> Result<ReferendumInfo<T::AccountId, T::BlockNumber>, DispatchError> {
        let info = ReferendumInfoOf::<T>::get(spot_id).ok_or(Error::<T>::ReferendumIsInValid.into());
        info
    }

    /// Ok if the given referendum is active, Err otherwise
    fn ensure_ongoing(r: ReferendumInfo<T::AccountId, T::BlockNumber>)
                      -> Result<ReferendumStatus<T::AccountId, T::BlockNumber>, DispatchError>
    {
        match r {
            ReferendumInfo::Ongoing(s) => Ok(s),
            _ => Err(Error::<T>::ReferendumIsInValid.into()),
        }
    }

    fn do_register(who: &T::AccountId, spot_id: &SpotId) -> SpotId {
        return 5;
    }


    pub fn get_spot(spot_id: SpotId) -> Result<ContinuumSpot, DispatchError> {
        ContinuumSpots::<T>::get(spot_id).ok_or(Error::<T>::SpotNotFound.into())
    }

    pub fn transfer_spot(spot_id: SpotId, from: &T::AccountId, to: &(T::AccountId, CountryId)) -> Result<SpotId, DispatchError> {
        Self::transfer_spot(spot_id, from, to)
    }

    pub fn check_approved(tally: &ContinuumSpotTally<T::AccountId>) -> bool {
        true
    }
}

impl<T: Config> Continuum<T::AccountId> for Pallet<T> {
    fn transfer_spot(spot_id: SpotId, from: &T::AccountId, to: &(T::AccountId, CountryId)) -> Result<SpotId, DispatchError> {
        ContinuumSpots::<T>::try_mutate(spot_id, |maybe_spot| -> Result<SpotId, DispatchError>{
            let treasury = Self::account_id();
            if *from != treasury {
                //TODO Check account Id own country spot.country
            }
            let mut spot = maybe_spot.take().ok_or(Error::<T>::SpotNotFound)?;
            spot.country = to.1;
            Ok(spot_id)
        })
    }
}
