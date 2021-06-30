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
use frame_support::{ensure, decl_storage};
use frame_system::{ensure_root, ensure_signed};
use primitives::{Balance, CountryId, CurrencyId, BlindBoxId};
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
use sp_core::H256;
use frame_support::traits::Randomness;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::traits::Randomness;
    use sp_core::H256;
    use primitives::BlindBoxId;

    #[pallet::pallet]
    #[pallet::generate_store(trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        #[pallet::constant]
        type ModuleId: Get<ModuleId>;

        /// Something that provides randomness in the runtime.
        type Randomness: Randomness<Self::Hash>;

        type MaxGenerateRandom: Get<u32>;
    }

    #[pallet::storage]
    #[pallet::getter(fn get_blindbox_rewards)]
    pub type BlindBoxRewards<T: Config> =
    StorageDoubleMap<_, Twox64Concat, BlindBoxId, Twox64Concat, T::AccountId, (), OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_blindboxes)]
    pub type BlindBoxes<T: Config> = StorageMap<_, Twox64Concat, BlindBoxId, (), OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_blindboxescreators)]
    pub type BlindBoxesCreators<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, (), OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn all_blindboxes_count)]
    pub(super) type AvailableBlindBoxesCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn all_unique_blindboxes_count)]
    pub(super) type UniqueBlindBoxesCount<T: Config> = StorageValue<_, u32, ValueQuery>;

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
        RandomnessConsumed(H256, H256),
        BlindBoxIdGenerated(Vec<u32>),
        BlindBoxOpened(u32),
        BlindBoxGoodLuckNextTime(u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        //No permission
        NoPermission,
        // BlindBox does not exist
        BlindBoxDoesNotExist,
        // BlindBoxes still available, only allow to create once all the blindboxes have been used
        BlindBoxesStillAvailable
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub(super) fn set_blindbox_caller(origin: OriginFor<T>, account_id: T::AccountId ) -> DispatchResultWithPostInfo {
            let _ = ensure_root(origin)?;

            BlindBoxesCreators::<T>::insert(account_id, {});

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub(super) fn generate_blindbox_ids(origin: OriginFor<T>, number_blindboxes: u32) -> DispatchResultWithPostInfo {
            let caller = ensure_signed(origin)?;

            // Ensure the authorized caller can call this func
            ensure!(
                BlindBoxesCreators::<T>::contains_key(&caller),
                Error::<T>::NoPermission
            );

            // Ensure caller can only generate blindboxes once all the available blindboxes have been used
            ensure!(
                AvailableBlindBoxesCount::<T>::get() <= 0,
                Error::<T>::BlindBoxesStillAvailable
            );

            let mut blindbox_vec = Vec::new();

            // Generate random blindbox id and store
            let mut n= 0;

            while n < number_blindboxes {
                let mut blindbox_id = Self::generate_random_number(n);

                if !BlindBoxes::<T>::contains_key(blindbox_id) {
                    // Push to Vec and save to storage
                    blindbox_vec.push(blindbox_id);
                    BlindBoxes::<T>::insert(blindbox_id, {});

                    n += 1;
                }
            }

            AvailableBlindBoxesCount::<T>::put(number_blindboxes);

            Self::deposit_event(Event::BlindBoxIdGenerated(blindbox_vec));

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub(super) fn open_blind_box(origin: OriginFor<T>, blindbox_id: BlindBoxId) -> DispatchResultWithPostInfo {
            let owner = ensure_signed(origin)?;

            // Ensure the specified blindbox id exist in storage
            ensure!(
                BlindBoxes::<T>::contains_key(blindbox_id),
                Error::<T>::BlindBoxDoesNotExist
            );

            let mut random_number = Self::generate_random_number(0);

            // Best effort attempt to remove bias from modulus operator.
            for i in 1 .. T::MaxGenerateRandom::get() {
                random_number = Self::generate_random_number(i);
            }

            // 10% chance of winning
            if random_number % 10 == 0 {
                Self::save_blindbox_reward(&owner, blindbox_id);
                Self::deposit_event(Event::<T>::BlindBoxOpened(blindbox_id.clone()));
            }
            // 50% chance of winning
            else if random_number % 2 == 0 {
                Self::save_blindbox_reward(&owner, blindbox_id);
                Self::deposit_event(Event::<T>::BlindBoxOpened(blindbox_id.clone()));
            } else{
                Self::deposit_event(Event::<T>::BlindBoxGoodLuckNextTime(blindbox_id.clone()));
            }

            Ok(().into())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}

impl<T: Config> Pallet<T> {
    fn check_blindbox_exist(blindbox_id: BlindBoxId) -> bool{
        Self::get_blindboxes(blindbox_id) == Some(())
    }

    fn save_blindbox_reward(owner: &T::AccountId, blindbox_id: BlindBoxId) -> Result<BlindBoxId, DispatchError> {
        // Remove from BlindBoxes
        BlindBoxes::<T>::remove(blindbox_id);

        // Add to BlindBoxRewards
        BlindBoxRewards::<T>::insert( blindbox_id, owner, ());

        // Update AvailableBlindBoxesCount
        let available_blindbox_count = Self::all_blindboxes_count();

        let new_available_blindbox_count = available_blindbox_count.checked_sub(One::one()).ok_or("Overflow subtracting new count to available blindboxes")?;
        AvailableBlindBoxesCount::<T>::put(new_available_blindbox_count);

        Ok(blindbox_id)
    }

    // Generate a random number from a given seed.
    // Note that there is potential bias introduced by using modulus operator.
    // You should call this function with different seed values until the random
    // number lies within `u32.Max`.
    // https://github.com/paritytech/substrate/issues/8311
    fn generate_random_number(seed :u32) -> u32 {
        let random_seed = T::Randomness::random(&("blindbox", seed).encode());

        let random_number = <u32>::decode(&mut random_seed.as_ref())
            .expect("secure hashes should always be bigger than u32; qed");
        random_number
    }
}