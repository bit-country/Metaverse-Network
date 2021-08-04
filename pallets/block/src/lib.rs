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

use codec::{Decode, Encode};
use frame_support::ensure;
use frame_system::{ensure_root, ensure_signed};
use primitives::{Balance, CountryId, LandId, CurrencyId};
use sp_runtime::{traits::{AccountIdConversion, One}, DispatchError, ModuleId, RuntimeDebug};
use bc_country::*;
use sp_std::vec::Vec;
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;


#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub use pallet::*;
use frame_support::dispatch::DispatchResult;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::traits::{ReservableCurrency, LockableCurrency, Currency, ExistenceRequirement};

    #[pallet::pallet]
    #[pallet::generate_store(trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        #[pallet::constant]
        type LandTreasury: Get<ModuleId>;
        /// Source of Country Info
        type CountryInfoSource: BCCountry<Self::AccountId>;
        /// Currency
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
        /// Minimum Land Price
        type MinimumLandPrice: Get<BalanceOf<Self>>;
        /// Council origin which allows to update max bound
        type CouncilOrigin: EnsureOrigin<Self::Origin>;
    }

    type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::storage]
    #[pallet::getter(fn next_land_id)]
    pub type NextLandId<T: Config> = StorageValue<_, LandId, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_land_owner)]
    pub type LandOwner<T: Config> =
    StorageDoubleMap<_, Twox64Concat, LandId, Twox64Concat, T::AccountId, (), OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_lands_by_owner)]
    pub type LandByOwner<T: Config> =
    StorageMap<_, Blake2_128Concat, T::AccountId, Vec<LandId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn all_lands_count)]
    pub(super) type AllLandsCount<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_land_blocks)]
    pub type LandBlocks<T: Config> =
    StorageDoubleMap<_, Twox64Concat, CountryId, Twox64Concat, (i32, i32), (), OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_land_info)]
    pub type LandInfo<T: Config> =
    StorageMap<_, Blake2_128Concat, LandId, (CountryId, (i32, i32)), OptionQuery>;

    /// Get max bound
    #[pallet::storage]
    #[pallet::getter(fn get_max_bound)]
    pub type MaxBound<T: Config> = StorageValue<_, (i32, i32), ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    #[pallet::metadata(T::AccountId = "AccountId")]
    pub enum Event<T: Config> {
        NewLandCreated(Vec<LandId>),
        TransferredLand(LandId, T::AccountId, T::AccountId),
        NewLandBlockPurchased(LandId, CountryId, (i32, i32)),
        NewMaxBoundSet((i32, i32))
    }

    #[pallet::error]
    pub enum Error<T> {
        //No permission
        NoPermission,
        //No available bitcountry id
        NoAvailableBitCountryId,
        NoAvailableLandId,
        InsufficientFund,
        LandIdAlreadyExist,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub(super) fn set_max_bounds(origin: OriginFor<T>, new_bound: (i32, i32)) -> DispatchResultWithPostInfo {
            // Only execute by governance
            T::CouncilOrigin::ensure_origin(origin)?;

            MaxBound::<T>::set(new_bound);
            Self::deposit_event(Event::<T>::NewMaxBoundSet(new_bound));

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub(super) fn buy_land_block(origin: OriginFor<T>, bc_id: CountryId, coordinate: (i32, i32)) -> DispatchResultWithPostInfo {
            // Check ownership
            let sender = ensure_signed(origin)?;

            ensure!(T::CountryInfoSource::check_ownership(&sender, &bc_id), Error::<T>::NoPermission);

            // Check minimum balance and transfer
            let minimum_land_price = T::MinimumLandPrice::get();
            ensure!(T::Currency::free_balance(&sender) > minimum_land_price, Error::<T>::InsufficientFund);
            let land_treasury = Self::account_id();
            T::Currency::transfer(&sender, &land_treasury, minimum_land_price, ExistenceRequirement::KeepAlive)?;

            // Generate new land id
            let new_land_id = Self::get_new_land_id()?;

            // Add to land owners
            LandOwner::<T>::insert(new_land_id, &sender, ());
            Self::add_land_to_new_owner(new_land_id, &sender);

            // Update total land count
            let total_land_count = Self::all_lands_count();
            let new_total_land_count = total_land_count.checked_add(One::one()).ok_or("Overflow adding new count to total lands")?;
            AllLandsCount::<T>::put(new_total_land_count);

            // Update land info
            LandInfo::<T>::insert(new_land_id, (bc_id, coordinate));

            // Update land blocks
            LandBlocks::<T>::insert(bc_id, coordinate, ());

            Self::deposit_event(Event::<T>::NewLandBlockPurchased(new_land_id.clone(), bc_id, coordinate));

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub(super) fn buy_land(origin: OriginFor<T>, bc_id: CountryId, quantity: u8) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(T::CountryInfoSource::check_ownership(&sender, &bc_id), Error::<T>::NoPermission);

            let minimum_land_price = T::MinimumLandPrice::get();
            let total_cost = minimum_land_price * Into::<BalanceOf<T>>::into(quantity);
            ensure!(T::Currency::free_balance(&sender) > total_cost, Error::<T>::InsufficientFund);
            let land_treasury = Self::account_id();
            T::Currency::transfer(&sender, &land_treasury, total_cost, ExistenceRequirement::KeepAlive)?;

            let mut new_land_ids: Vec<LandId> = Vec::new();

            for _ in 0..quantity {
                let land_id = Self::get_new_land_id()?;
                new_land_ids.push(land_id);

                LandOwner::<T>::insert(land_id, &sender, ());

                Self::add_land_to_new_owner(land_id, &sender);
            }

            let total_land_count = Self::all_lands_count();

            let new_total_land_count = total_land_count.checked_add(quantity.into()).ok_or("Overflow adding new count to total lands")?;
            AllLandsCount::<T>::put(new_total_land_count);
            Self::deposit_event(Event::<T>::NewLandCreated(new_land_ids.clone()));

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub(super) fn transfer_land(origin: OriginFor<T>, to: T::AccountId, land_id: LandId) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // Get owner of the land
            LandOwner::<T>::try_mutate_exists(
                &land_id, &who, |land_by_owner| -> DispatchResultWithPostInfo {
                    //ensure there is record of the land owner with land id, account id and delete them
                    ensure!(land_by_owner.is_some(), Error::<T>::NoPermission);

                    if who == to {
                        // no change needed
                        return Ok(().into());
                    }

                    *land_by_owner = None;
                    LandOwner::<T>::insert(land_id.clone(), to.clone(), ());

                    Self::add_land_to_new_owner(land_id, &who);
                    Self::deposit_event(Event::<T>::TransferredLand(land_id.clone(), who.clone(), to));

                    Ok(().into())
                })
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}

impl<T: Config> Module<T> {
    /// Reads the nonce from storage, increments the stored nonce, and returns
    /// the encoded nonce to the caller.

    fn get_new_land_id() -> Result<LandId, DispatchError> {
        let land_id = NextLandId::<T>::try_mutate(|id| -> Result<LandId, DispatchError> {
            let current_id = *id;
            *id = id
                .checked_add(One::one())
                .ok_or(Error::<T>::NoAvailableLandId)?;
            Ok(current_id)
        })?;
        Ok(land_id)
    }

    fn account_id() -> T::AccountId {
        T::LandTreasury::get().into_account()
    }

    fn add_land_to_new_owner(land_id: LandId, sender: &T::AccountId) -> DispatchResult {
        if LandOwner::<T>::contains_key(land_id, &sender) {
            LandByOwner::<T>::try_mutate(
                &sender,
                |land_ids| -> DispatchResult {
                    // Check if the asset_id already in the owner
                    ensure!(!land_ids.iter().any(|i| land_id == *i), Error::<T>::LandIdAlreadyExist);
                    land_ids.push(land_id);
                    Ok(())
                },
            )?;
        } else {
            let mut new_land_vec = Vec::<LandId>::new();
            new_land_vec.push(land_id);
            LandByOwner::<T>::insert(&sender, new_land_vec)
        }
        Ok(())
    }
}
