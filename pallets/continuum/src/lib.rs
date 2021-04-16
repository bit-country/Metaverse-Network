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
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
    traits::{Get, Vec}, StorageMap, StorageValue,
};
use frame_system::{self as system, ensure_root, ensure_signed};
use primitives::{Balance, CountryId, CurrencyId, SpotId};
use sp_runtime::{
    traits::{AccountIdConversion, One, Zero},
    DispatchError, ModuleId, RuntimeDebug,
};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

pub use pallet::*;
use crate::pallet::{Config, Pallet, ActiveAuctionSlots};

mod vote;
mod types;

pub use vote::{Vote, Voting, AccountVote};
pub use types::{ReferendumInfo, ReferendumStatus, ContinuumSpotTally};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;


#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
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
    use sp_std::vec::Vec;
    use frame_system::pallet_prelude::*;
    use frame_support::pallet_prelude::*;
    use sp_runtime::traits::Zero;
    use crate::{AuctionSlot, SpotEOI, SpotId, ReferendumInfo, Voting};
    use frame_support::dispatch::DispatchResult;
    use frame_support::traits::Currency;

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
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

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
    #[pallet::getter(fn get_continuum_referedum)]
    pub type ReferendumInfoOf<T: Config> = StorageMap<_, Twox64Concat, SpotId, ReferendumInfo<T::AccountId, T::BlockNumber, T::Hash, BalanceOf<T>>, OptionQuery>;

    /// All votes of a particular voter
    #[pallet::storage]
    #[pallet::getter(fn get_continuum_referedum)]
    pub type VotingOf<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Voting<BalanceOf<T>, T::AccountId, T::BlockNumber>, ValueQuery>;

    /// Get max bound
    #[pallet::storage]
    #[pallet::getter(fn get_max_bound)]
    pub type MaxBound<T: Config> = StorageValue<_, (u32, u32), ValueQuery>;

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
        /// Referendum is invalid
        ReferendumIsInValid,
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
        pub fn neighbor_vote(origin: OriginFor<T>, id: SpotId, rejects: Vec<T::AccountId>) -> DispatchResultWithPostInfo {
            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T>
{
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
        GNPSlots::<T>::insert(now, started_gnp_auction_slots);
        //TODO Emit event Auction slot start GNP

        // Remove the old active auction slots
        ActiveAuctionSlots::<T>::remove(&current_active_session_id);

        Self::eoi_to_auction_slots(current_active_session_id, now)?;
        CurrentIndex::<T>::set(now);
        //TODO Emit event
        Ok(().into())
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

    fn try_vote(who: &T::AccountId, spot_id: SpotId, bidder: AccountId, vote: AccountVote<T::AccountId>) -> DispatchResult {
        let mut status = Self::referendum_status(spot_id)?;

        VotingOf::<T>::try_mutate(who, |mut voting| -> DispatchResult {
            let mut votes = &voting.votes;
            match votes.binary_search_by_key(&spot_id, |i| i.0) {
                //Already votedd
                Ok(i) => {}
                Err(i) => {}
            }
            Ok(())
        })
    }

    fn referendum_status(spot_id: SpotId) -> Result<ReferendumStatus<T::AccountId, T::BlockNumber, T::Hash, BalanceOf<T>>, DispatchError> {
        let info = ReferendumInfoOf::<T>::get(spot_id).ok_or(Error::<T>::ReferendumIsInValid)?;

        Self::ensure_ongoing(info.into())
    }

    /// Ok if the given referendum is active, Err otherwise
    fn ensure_ongoing(r: ReferendumInfo<T::AccountId, T::BlockNumber, T::Hash, BalanceOf<T>>)
                      -> Result<ReferendumStatus<T::AccountId, T::BlockNumber, T::Hash, BalanceOf<T>>, DispatchError>
    {
        match r {
            ReferendumInfo::Ongoing(s) => Ok(s),
            _ => Err(Error::<T>::ReferendumInvalid.into()),
        }
    }

    fn do_register(who: &T::AccountId, spot_id: &SpotId) -> SpotId {
        return 5;
    }
}
