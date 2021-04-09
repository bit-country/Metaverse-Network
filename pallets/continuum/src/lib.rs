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


#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
        traits::Get, StorageMap, StorageValue,
    };
    use frame_system::{self as system, ensure_root, ensure_signed};
    use primitives::{Balance, CountryId, CurrencyId};
    use sp_runtime::{
        traits::{AccountIdConversion, One},
        DispatchError, ModuleId, RuntimeDebug,
    };
    use sp_std::vec::Vec;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn register_interest(origin: OriginFor<T>, spot_id: u64) -> DispatchResultWithPostInfo {
            Ok(().into())
        }
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn vote_bidder_rejection(origin: OriginFor<T>, bidders: Vec<AccountId>) -> DispatchResultWithPostInfo {
            Ok(().into())
        }
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_max_bounds(origin: OriginFor<T>, new_bound: (u64, u64)) -> DispatchResultWithPostInfo {
            Ok(().into())
        }
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_new_auction_rate(origin: OriginFor<T>, new_rate: u32) -> DispatchResultWithPostInfo {
            Ok(().into())
        }
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn bid(origin: OriginFor<T>, id: AuctionId, value: T::Balance) -> DispatchResultWithPostInfo {
            Ok(().into())
        }
    }
}
