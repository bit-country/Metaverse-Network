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

use frame_support::pallet_prelude::*;
use frame_support::traits::{Currency, ExistenceRequirement, VestingSchedule};
use frame_support::{dispatch::DispatchResult, ensure, traits::Get, PalletId};
use frame_system::pallet_prelude::*;
use frame_system::{ensure_root, ensure_signed};
use pallet_vesting::{Pallet as VestingModule, VestingInfo};
use scale_info::TypeInfo;
use sp_runtime::traits::Convert;
use sp_runtime::{
	traits::{AccountIdConversion, One, Saturating, Zero},
	DispatchError,
};
use sp_std::{convert::TryInto, vec::Vec};

use auction_manager::{Auction, CheckAuctionItemHandler};
use core_primitives::*;
pub use pallet::*;
use primitives::{
	estate::Estate, Balance, EstateId, ItemId, MetaverseId, UndeployedLandBlock, UndeployedLandBlockId,
	UndeployedLandBlockType,
};
pub use weights::WeightInfo;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::traits::{Currency, ExistenceRequirement, Imbalance, ReservableCurrency, VestingSchedule};
	use pallet_vesting::VestingInfo;
	use sp_runtime::traits::{CheckedAdd, CheckedSub, Convert, ConvertInto, StaticLookup, Zero};

	use primitives::UndeployedLandBlockId;

	use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_vesting::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Currency
		type Currency: Currency<Self::AccountId>;
		/// Vesting schedule
		type VestingSchedule: VestingSchedule<Self::AccountId>;
		/// Convert block number to balance
		type BlockNumberToBalance: Convert<Self::BlockNumber, BalanceOf<Self>>;
		/// Weight implementation
		type WeightInfo: WeightInfo;
	}

	/// allowed origins
	#[pallet::storage]
	#[pallet::getter(fn crowdloan_accepted_origin)]
	pub type CrowdloanDistributorOrigins<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, (), OptionQuery>;

	pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	pub type VestingBalanceOf<T> =
		<<T as pallet_vesting::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::event]
	#[pallet::generate_deposit(pub (crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Beneficial Account Id, Amount
		TokenTransferred(T::AccountId, BalanceOf<T>),
		/// Beneficial AccountId, Amount
		VestedTokenTransferred(T::AccountId, VestingInfo<BalanceOf<T>, T::BlockNumber>),
		/// AccountId, Schedule Index
		RemovedRewardVestingSchedule(T::AccountId, u32),
		/// Distributor AccountId
		AddedDistributorOrigin(T::AccountId),
		/// Distributor AccountId
		RemovedDistributorOrigin(T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// No permission
		NoPermission,
		/// Already got existing vesting info
		UserAlreadyGotExistingVestingInfo,
		/// Already set as distributor origin
		AlreadySetAsDistributorOrigin,
		/// Distributor origin does not exist
		DistributorOriginDoesNotExist,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(< T as pallet::Config >::WeightInfo::transfer_unlocked_reward())]
		pub fn transfer_unlocked_reward(
			origin: OriginFor<T>,
			to: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(Self::is_accepted_origin(&who), Error::<T>::NoPermission);

			<T as pallet::Config>::Currency::transfer(&who, &to, amount, ExistenceRequirement::KeepAlive)?;
			Self::deposit_event(Event::<T>::TokenTransferred(to, amount));

			Ok(().into())
		}

		#[pallet::weight(< T as pallet::Config >::WeightInfo::transfer_vested_reward())]
		pub fn transfer_vested_reward(
			origin: OriginFor<T>,
			to: <T::Lookup as StaticLookup>::Source,
			schedule: VestingInfo<VestingBalanceOf<T>, T::BlockNumber>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;

			ensure!(Self::is_accepted_origin(&who), Error::<T>::NoPermission);
			let target = T::Lookup::lookup(to.clone())?;
			// Get existing vesting schedule
			let vesting_info = T::VestingSchedule::vesting_balance(&target);
			// Ensure user doesn't have any vested reward
			ensure!(vesting_info == None, Error::<T>::UserAlreadyGotExistingVestingInfo);

			VestingModule::<T>::vested_transfer(origin, to, schedule)?;

			Ok(().into())
		}

		#[pallet::weight(< T as pallet::Config >::WeightInfo::remove_distributor_origin())]
		pub fn remove_vested_reward(
			origin: OriginFor<T>,
			to: T::AccountId,
			schedule_index: u32,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			T::VestingSchedule::remove_vesting_schedule(&to, schedule_index)?;

			Self::deposit_event(Event::<T>::RemovedRewardVestingSchedule(to, schedule_index));

			Ok(().into())
		}

		#[pallet::weight(< T as pallet::Config >::WeightInfo::set_distributor_origin())]
		pub fn set_distributor_origin(origin: OriginFor<T>, to: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(
				!Self::is_accepted_origin(&to),
				Error::<T>::AlreadySetAsDistributorOrigin
			);

			CrowdloanDistributorOrigins::<T>::insert(to.clone(), ());
			Self::deposit_event(Event::AddedDistributorOrigin(to));

			Ok(())
		}

		#[pallet::weight(< T as pallet::Config >::WeightInfo::remove_distributor_origin())]
		pub fn remove_distributor_origin(origin: OriginFor<T>, to: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(Self::is_accepted_origin(&to), Error::<T>::DistributorOriginDoesNotExist);

			CrowdloanDistributorOrigins::<T>::remove(to.clone());
			Self::deposit_event(Event::RemovedDistributorOrigin(to));

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn is_accepted_origin(who: &T::AccountId) -> bool {
		let accepted_origin = Self::crowdloan_accepted_origin(who);
		accepted_origin == Some(())
	}
}
