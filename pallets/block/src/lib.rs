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
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
    traits::Randomness, StorageMap, StorageValue,
};
use frame_system::{self as system, ensure_signed};
use sp_core::H256;
use sp_runtime::traits::Hash;
use sp_std::vec::Vec;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Block<Hash> {
    id: Hash,
    country_id: Hash,
}

mod mock;
mod tests;

pub trait Config: system::Config {
    type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;
    type RandomnessSource: Randomness<H256>;
}

decl_storage! {
    trait Store for Module<T: Config> as BlockModule {

        pub BlockOwner get(fn owner_of): map hasher(blake2_128_concat) T::Hash => Option<T::AccountId>;
        pub Blocks get(fn get_block): map hasher(blake2_128_concat) T::Hash => Block<T::Hash>;
        pub AllBlocksCount get(fn all_blocks_count): u64;

        Init get(fn is_init): bool;

        Nonce get(fn nonce): u32;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Config>::AccountId,
    {
        Initialized(AccountId),
        RandomnessConsumed(H256, H256),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        /// Attempted to initialize the token after it had already been initialized.
        AlreadyInitialized,
        /// Attempted to transfer more funds than were available
        InsufficientFunds,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        #[weight = 10_000]
        fn create_block(origin, country_id: T::Hash) -> DispatchResult {

            let sender = ensure_signed(origin)?;

            let random_str = Self::encode_and_update_nonce();

            let random_seed = T::RandomnessSource::random_seed();
            let random_result = T::RandomnessSource::random(&random_str);
            let random_hash = (random_seed, &sender, random_result).using_encoded(<T as system::Config>::Hashing::hash);

            //Check only country owner can add new block

            let new_block = Block{
                id: random_hash,
                country_id: country_id,
            };
            ensure!(!<Blocks<T>>::contains_key(random_hash), "Block already exists");

            <BlockOwner<T>>::insert(random_hash, &sender);
            <Blocks<T>>::insert(random_hash, new_block);

            let all_blocks_count = Self::all_blocks_count();

            let new_all_blocks_count = all_blocks_count.checked_add(1)
                .ok_or("Overflow adding a new block to total supply")?;

            AllBlocksCount::put(new_all_blocks_count);

            Self::deposit_event(RawEvent::RandomnessConsumed(random_seed, random_result));

            Ok(())
        }
    }
}

impl<T: Config> Module<T> {
    /// Reads the nonce from storage, increments the stored nonce, and returns
    /// the encoded nonce to the caller.
    fn encode_and_update_nonce() -> Vec<u8> {
        let nonce = Nonce::get();
        Nonce::put(nonce.wrapping_add(1));
        nonce.encode()
    }
}
