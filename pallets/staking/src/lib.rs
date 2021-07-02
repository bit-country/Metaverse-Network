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

use support::{decl_module, decl_event, decl_storage, StorageValue, StorageMap};
use system::ensure_signed;

pub trait Config: system::Config {
    // The traits the `Event` type used in this pallet has.
    type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;
}

decl_event!{
    pub enum Event<T> where
        AccountId = <T as system::Config>::AccountId,
    {
        RewardGet(AccountId, u64),
    }
}

decl_storage! {
    trait Store for Module<T: Config> as Example {
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        // A default function for depositing events in our runtime
        fn deposit_event() = default;

        /// Reward a validator
        #[weight = 10_000]
        pub fn reward_myself(origin) -> dispatch::DispatchResult {
            let reported = ensure_signed(origin)?;
            <staking::Pallet<T>>::reward_by_ids(vec![(reported, 10)]);
            Ok(())
        }
    }
}
