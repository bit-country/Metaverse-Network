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
use primitives::{Balance, BitCountryId, CurrencyId, FungibleTokenId};
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
    pub type NextBitCountryId<T: Config> = StorageValue<_, BitCountryId, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_bitcountry)]
    pub type Countries<T: Config> =
    StorageMap<_, Twox64Concat, BitCountryId, Country<T::AccountId>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_country_owner)]
    pub type BitCountryOwner<T: Config> =
    StorageDoubleMap<_, Twox64Concat, BitCountryId, Twox64Concat, T::AccountId, (), OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn all_bitcountries_count)]
    pub(super) type AllCountriesCount<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_freezing_country)]
    pub(super) type FreezingCountries<T: Config> =
    StorageMap<_, Twox64Concat, BitCountryId, (), OptionQuery>;

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
        NewBitCountryCreated(BitCountryId),
        TransferredBitCountry(BitCountryId, T::AccountId, T::AccountId),
        BitCountryFreezed(BitCountryId),
        BitCountryDestroyed(BitCountryId),
        BitCountryUnfreezed(BitCountryId),
        BitCountryMintedNewCurrency(BitCountryId, FungibleTokenId),
    }

    #[pallet::error]
    pub enum Error<T> {
        // BitCountry info not found
        BitCountryInfoNotFound,
        // BitCountry Id not found
        BitCountryIdNotFound,
        // No permission
        NoPermission,
        // No available BitCountry id
        NoAvailableBitCountryId,
        // Fungible token already issued
        FungibleTokenAlreadyIssued,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub(super) fn create_bitcountry(origin: OriginFor<T>, metadata: Vec<u8>) -> DispatchResultWithPostInfo {
            let owner = ensure_signed(origin)?;

            let bitcountry_id = Self::new_bitcountry(&owner, metadata)?;

            BitCountryOwner::<T>::insert(bc_id, owner, ());

            let total_bitcountry_count = Self::all_bitcountries_count();

            let new_total_bitcountry_count = total_bitcountry_count.checked_add(One::one()).ok_or("Overflow adding new count to new_total_bitcountry_count")?;
            AllCountriesCount::<T>::put(new_total_bitcountry_count);
            Self::deposit_event(Event::<T>::NewBitCountryCreated(bitcountry_id.clone()));

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub(super) fn transfer_bitcountry(origin: OriginFor<T>, to: T::AccountId, bitcountry_id: BitCountryId) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // Get owner of the bitcountry
            BitCountryOwner::<T>::try_mutate_exists(
                &bitcountry_id, &who, |bitcountry_by_owner| -> DispatchResultWithPostInfo {
                    // Ensure there is record of the bitcountry owner with bitcountry id, account id and delete them
                    ensure!(bitcountry_by_owner.is_some(), Error::<T>::NoPermission);

                    if who == to {
                        // No change needed
                        return Ok(().into());
                    }

                    *bitcountry_by_owner = None;
                    BitCountryOwner::<T>::insert(bitcountry_id.clone(), to.clone(), ());

                    Countries::<T>::try_mutate_exists(
                        &bitcountry_id,
                        |bitcountry| -> DispatchResultWithPostInfo{
                            let mut bitcountry_record = bitcountry.as_mut().ok_or(Error::<T>::NoPermission)?;
                            bitcountry_record.owner = to.clone();
                            Self::deposit_event(Event::<T>::TransferredBitCountry(bitcountry_id, who.clone(), to.clone()));

                            Ok(().into())
                        },
                    )
                })
        }

        #[pallet::weight(10_000)]
        pub(super) fn freeze_bitcountry(origin: OriginFor<T>, bitcountry_id: BitCountryId) -> DispatchResultWithPostInfo {
            // Only Council can free a bitcountry
            ensure_root(origin)?;

            FreezingCountries::<T>::insert(bitcountry_id, ());
            Self::deposit_event(Event::<T>::BitCountryFreezed(bitcountry_id));

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub(super) fn unfreeze_bitcountry(origin: OriginFor<T>, bitcountry_id: BitCountryId) -> DispatchResultWithPostInfo {
            // Only Council can free a bitcountry
            ensure_root(origin)?;

            FreezingCountries::<T>::try_mutate(bitcountry_id, |freeze_bitcountry| -> DispatchResultWithPostInfo{
                ensure!(freeze_bitcountry.take().is_some(), Error::<T>::BitCountryInfoNotFound);

                Self::deposit_event(Event::<T>::BitCountryUnfreezed(bitcountry_id));
                Ok(().into())
            })
        }

        #[pallet::weight(10_000)]
        pub(super) fn destroy_country(origin: OriginFor<T>, bitcountry_id: BitCountryId) -> DispatchResultWithPostInfo {
            // Only Council can destroy a bitcountry
            ensure_root(origin)?;

            Countries::<T>::try_mutate(bitcountry_id, |bitcountry_info| -> DispatchResultWithPostInfo{
                let t = bitcountry_info.take().ok_or(Error::<T>::BitCountryInfoNotFound)?;

                BitCountryOwner::<T>::remove(&bitcountry_id, t.owner.clone());
                Self::deposit_event(Event::<T>::BitCountryDestroyed(bitcountry_id));

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

    fn new_bitcountry(owner: &T::AccountId, metadata: Vec<u8>) -> Result<BitCountryId, DispatchError> {
        let bitcountry_id = NextBitCountryId::<T>::try_mutate(|id| -> Result<BitCountryId, DispatchError> {
            let current_id = *id;
            *id = id
                .checked_add(One::one())
                .ok_or(Error::<T>::NoAvailableBitCountryId)?;
            Ok(current_id)
        })?;

        let country_info = Country {
            owner: owner.clone(),
            currency_id: FungibleTokenId::NativeToken(0),
            metadata,
        };

        Countries::<T>::insert(bitcountry_id, country_info);

        Ok(bitcountry_id)
    }
}

impl<T: Config> BitCountry<T::AccountId> for Module<T>
{
    fn check_ownership(who: &T::AccountId, bitcountry_id: &BitCountryId) -> bool {
        Self::get_country_owner(bitcountry_id, who) == Some(())
    }

    fn get_bitcountry(bitcountry_id: BitCountryId) -> Option<Country<T::AccountId>> {
        Self::get_bitcountry(bitcountry_id)
    }

    fn get_bitcountry_token(bitcountry_id: BitCountryId) -> Option<FungibleTokenId> {
        if let Some(country) = Self::get_bitcountry(bitcountry_id) {
            return Some(country.currency_id);
        }
        None
    }

    fn update_bitcountry_token(bitcountry_id: BitCountryId, currency_id: FungibleTokenId) -> Result<(), DispatchError> {
        Countries::<T>::try_mutate_exists(
            &bitcountry_id,
            |bitcountry| {
                let mut bitcountry_record = bitcountry.as_mut().ok_or(Error::<T>::NoPermission)?;

                ensure!(bitcountry_record.currency_id == FungibleTokenId::SocialToken(0), Error::<T>::FungibleTokenAlreadyIssued);

                bitcountry_record.currency_id = currency_id.clone();
                Self::deposit_event(Event::<T>::BitCountryMintedNewCurrency(bitcountry_id, currency_id));
                Ok(())
            },
        )
    }
}