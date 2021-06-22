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

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::traits::{ReservableCurrency, LockableCurrency, Currency};

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


    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    #[pallet::metadata(T::AccountId = "AccountId")]
    pub enum Event<T: Config> {
        NewLandCreated(LandId),
        TransferredLand(LandId, T::AccountId, T::AccountId),
    }

    #[pallet::error]
    pub enum Error<T> {
        //Land Info not found
        LandInfoNotFound,
        //Country Id not found
        LandIdNotFound,
        //No permission
        NoPermission,
        //No available bitcountry id
        NoAvailableBitCountryId,
        NoAvailableLandId,
        InsufficientFund,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub(super) fn buy_land(origin: OriginFor<T>, bc_id: CountryId, quantity: u8) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(T::CountryInfoSource::check_ownership(&sender, &bc_id), Error::<T>::NoPermission);

            let minimum_land_price = T::MinimumLandPrice::get();
            let total_cost = minimum_land_price * Into::<BalanceOf<T>>::into(quantity);
            ensure!(T::Currency::free_balance(&sender) > total_cost, Error::<T>::InsufficientFund);
            let land_treasury = Self::account_id();
            ensure!(T::Currency::transfer(&sender, &land_treasury, total_cost, ExistenceRequirement::KeepAlive), Error::<T>::InsufficientFund);

            let mut new_land_ids: Vec<LandId> = Vec::new();

            for _ in 0..quantity {
                let land_id = Self::get_new_land_id()?;
                new_land_ids.push(land_id);

                if LandOwner::<T>::contains_key(land_id, &sender) {
                    AssetsByOwner::<T>::try_mutate(
                        &sender,
                        |asset_ids| -> DispatchResult {
                            // Check if the asset_id already in the owner
                            ensure!(!asset_ids.iter().any(|i| asset_id == *i), Error::<T>::AssetIdAlreadyExist);
                            asset_ids.push(asset_id);
                            Ok(())
                        },
                    )?;
                } else {
                    let mut assets = Vec::<AssetId>::new();
                    assets.push(asset_id);
                    AssetsByOwner::<T>::insert(&sender, assets)
                }
            }
            //Country treasury
            let country_fund = CountryFund {
                vault: fund_id,
                value: 0,
                backing: 0, //0 BCG backing for now,
                currency_id: Default::default(),
            };
            CountryTresury::<T>::insert(country_id, country_fund);

            CountryOwner::<T>::insert(country_id, owner, ());

            let total_country_count = Self::all_countries_count();

            let new_total_country_count = total_country_count.checked_add(One::one()).ok_or("Overflow adding new count to total bitcountry")?;
            AllCountriesCount::<T>::put(new_total_country_count);
            Self::deposit_event(Event::<T>::NewCountryCreated(country_id.clone()));

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub(super) fn transfer_land(origin: OriginFor<T>, to: T::AccountId, country_id: CountryId) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // Get owner of the bitcountry
            CountryOwner::<T>::try_mutate_exists(
                &country_id, &who, |country_by_owner| -> DispatchResultWithPostInfo {
                    //ensure there is record of the bitcountry owner with bitcountry id, account id and delete them
                    ensure!(country_by_owner.is_some(), Error::<T>::NoPermission);

                    if who == to {
                        // no change needed
                        return Ok(().into());
                    }

                    *country_by_owner = None;
                    CountryOwner::<T>::insert(country_id.clone(), to.clone(), ());

                    Countries::<T>::try_mutate_exists(
                        &country_id,
                        |country| -> DispatchResultWithPostInfo{
                            let mut country_record = country.as_mut().ok_or(Error::<T>::NoPermission)?;
                            country_record.owner = to.clone();
                            Self::deposit_event(Event::<T>::TransferredCountry(country_id, who.clone(), to.clone()));

                            Ok(().into())
                        },
                    )
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
}

impl<T: Config> BCCountry<T::AccountId> for Module<T>
{
    fn check_ownership(who: &T::AccountId, country_id: &CountryId) -> bool {
        Self::get_country_owner(country_id, who) == Some(())
    }

    fn get_country(country_id: CountryId) -> Option<Country<T::AccountId>> {
        Self::get_country(country_id)
    }

    fn get_country_token(country_id: CountryId) -> Option<CurrencyId> {
        if let Some(country) = Self::get_country(country_id) {
            return Some(country.currency_id);
        }
        None
    }
}
