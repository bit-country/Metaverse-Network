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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

//mod tests;
//mod mock;

use frame_support::{
	pallet_prelude::*,
	traits::{CallMetadata, Contains, GetCallMetadata, PalletInfoAccess},
	transactional,
};
use frame_system::pallet_prelude::*;
use sp_runtime::DispatchResult;
use sp_std::{prelude::*, vec::Vec};
use primitives::{ClassId, TokenId};
use core_primitives::NFTTrait;

pub use pallet::*;
// pub use weights::WeightInfo;

//#[cfg(feature = "runtime-benchmarks")]
//pub mod benchmarking;

//pub mod weights;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::traits::{Currency, ReservableCurrency};
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Currency type
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		/// NFT trait required for minting NFTs
		type NFTSource: NFTTrait<Self::AccountId, BalanceOf<Self>, ClassId = ClassId, TokenId = TokenId>;
		/// Accounts that can set start migration
		type MigrationOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;
		// /// Extrinsics' weights
		//type WeightInfo: WeightInfo;
	}
	pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::error]
	pub enum Error<T> {
		/// Migration is already active
		MigrationInProgress,
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// Started the nft migration
		MigrationStarted,
		/// Comleted the nft migration
		MigrationCompleted,
	}

	#[pallet::storage]
	#[pallet::getter(fn is_migration_active)]
	pub type ActiveMigrationStatus<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// Hook that is called every time a new block is initialized.
		fn on_initialize(block_number: BlockNumberFor<T>) -> Weight {
			if Self::is_migration_active() {
				// TODO: Fetch Pioneer collection data from DB
				// TODO: Create collections
				// TODO: Fetch Pioneer class data from DB
				// TODO: Mint new classes
				// TODO: Fetch Pioneer token data from DB
				// TODO: Mint new tokens
			}
			Weight::from_parts(0, 0)
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(T::DbWeight::get().read + 2 * T::DbWeight::get().write)]
		pub fn start_migration(origin: OriginFor<T>) -> DispatchResult {
			T::MigrationOrigin::ensure_origin(origin)?;
			ensure!(!Self::is_migration_active(), Error::<T>::MigrationInProgress);
			ActiveMigrationStatus::<T>::put(true);
			Self::deposit_event(Event::<T>::MigrationStarted);
			Ok(())
		}

	}
}

