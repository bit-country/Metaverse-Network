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
use primitives::{Balance, CountryId, CurrencyId, SocialTokenCurrencyId};
use sp_runtime::{traits::{AccountIdConversion, One}, DispatchError, ModuleId, RuntimeDebug, DispatchResult};
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

    #[pallet::pallet]
    #[pallet::generate_store(trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        #[pallet::constant]
        type ModuleId: Get<ModuleId>;
    }

    #[pallet::storage]
    #[pallet::getter(fn next_country_id)]
    pub type NextCountryId<T: Config> = StorageValue<_, CountryId, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_country)]
    pub type Countries<T: Config> =
    StorageMap<_, Twox64Concat, CountryId, Country<T::AccountId>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_country_owner)]
    pub type CountryOwner<T: Config> =
    StorageDoubleMap<_, Twox64Concat, CountryId, Twox64Concat, T::AccountId, (), OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn all_countries_count)]
    pub(super) type AllCountriesCount<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_freezing_country)]
    pub(super) type FreezingCountries<T: Config> =
    StorageMap<_, Twox64Concat, CountryId, (), OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn is_init)]
    pub(super) type Init<T: Config> = StorageValue<_, bool, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn nonce)]
    pub(super) type Nonce<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    #[pallet::metadata(T::AccountId = "AccountId")]
    pub enum Event<T: Config> {
        NewCountryCreated(CountryId),
        TransferredCountry(CountryId, T::AccountId, T::AccountId),
        CountryFreezed(CountryId),
        CountryDestroyed(CountryId),
        CountryUnFreezed(CountryId),
        CountryMintedNewCurrency(CountryId, SocialTokenCurrencyId),
    }

    #[pallet::error]
    pub enum Error<T> {
        //Country Info not found
        CountryInfoNotFound,
        //Country Id not found
        CountryIdNotFound,
        //No permission
        NoPermission,
        //No available bitcountry id
        NoAvailableCountryId,
        SocialTokenAlreadyIssued,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub(super) fn create_bc(origin: OriginFor<T>, metadata: Vec<u8>) -> DispatchResultWithPostInfo {
            let owner = ensure_signed(origin)?;

            let country_id = Self::new_country(&owner, metadata)?;

            CountryOwner::<T>::insert(country_id, owner, ());

            let total_country_count = Self::all_countries_count();

            let new_total_country_count = total_country_count.checked_add(One::one()).ok_or("Overflow adding new count to total bitcountry")?;
            AllCountriesCount::<T>::put(new_total_country_count);
            Self::deposit_event(Event::<T>::NewCountryCreated(country_id.clone()));

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub(super) fn transfer_country(origin: OriginFor<T>, to: T::AccountId, country_id: CountryId) -> DispatchResultWithPostInfo {
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

        #[pallet::weight(10_000)]
        pub(super) fn freeze_country(origin: OriginFor<T>, country_id: CountryId) -> DispatchResultWithPostInfo {
            //Only Council can free a bitcountry
            ensure_root(origin)?;

            FreezingCountries::<T>::insert(country_id, ());
            Self::deposit_event(Event::<T>::CountryFreezed(country_id));

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub(super) fn unfreeze_country(origin: OriginFor<T>, country_id: CountryId) -> DispatchResultWithPostInfo {
            //Only Council can free a bitcountry
            ensure_root(origin)?;

            FreezingCountries::<T>::try_mutate(country_id, |freeze_country| -> DispatchResultWithPostInfo{
                ensure!(freeze_country.take().is_some(), Error::<T>::CountryInfoNotFound);

                Self::deposit_event(Event::<T>::CountryUnFreezed(country_id));
                Ok(().into())
            })
        }

        #[pallet::weight(10_000)]
        pub(super) fn destroy_country(origin: OriginFor<T>, country_id: CountryId) -> DispatchResultWithPostInfo {
            //Only Council can destroy a bitcountry
            ensure_root(origin)?;

            Countries::<T>::try_mutate(country_id, |country_info| -> DispatchResultWithPostInfo{
                let t = country_info.take().ok_or(Error::<T>::CountryInfoNotFound)?;

                CountryOwner::<T>::remove(&country_id, t.owner.clone());
                Self::deposit_event(Event::<T>::CountryDestroyed(country_id));

                Ok(().into())
            })
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}

impl<T: Config> Pallet<T> {
    /// Reads the nonce from storage, increments the stored nonce, and returns
    /// the encoded nonce to the caller.

    fn new_country(owner: &T::AccountId, metadata: Vec<u8>) -> Result<CountryId, DispatchError> {
        let country_id = NextCountryId::<T>::try_mutate(|id| -> Result<CountryId, DispatchError> {
            let current_id = *id;
            *id = id
                .checked_add(One::one())
                .ok_or(Error::<T>::NoAvailableCountryId)?;
            Ok(current_id)
        })?;

        let country_info = Country {
            owner: owner.clone(),
            currency_id: SocialTokenCurrencyId::SocialToken(0),
            metadata,
        };

        Countries::<T>::insert(country_id, country_info);

        Ok(country_id)
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

    fn get_country_token(country_id: CountryId) -> Option<SocialTokenCurrencyId> {
        if let Some(country) = Self::get_country(country_id) {
            return Some(country.currency_id);
        }
        None
    }

    fn update_country_token(country_id: CountryId, currency_id: SocialTokenCurrencyId) -> Result<(), DispatchError> {
        Countries::<T>::try_mutate_exists(
            &country_id,
            |country| {
                let mut country_record = country.as_mut().ok_or(Error::<T>::NoPermission)?;

                ensure!(country_record.currency_id == SocialTokenCurrencyId::SocialToken(0), Error::<T>::SocialTokenAlreadyIssued);

                country_record.currency_id = currency_id.clone();
                Self::deposit_event(Event::<T>::CountryMintedNewCurrency(country_id, currency_id));
                Ok(())
            },
        )
    }
}
