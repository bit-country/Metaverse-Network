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

use frame_support::{decl_module, dispatch};
use frame_system::ensure_signed;
use pallet_staking::{self as staking};

pub trait Config: staking::Config {}

#![cfg(test)]

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        /// Reward a validator
        #[weight = 10_000]
        pub fn reward_myself(origin) -> dispatch::DispatchResult {
            let reported = ensure_signed(origin)?;
            <staking::Pallet<T>>::reward_by_ids(vec![(reported, 10)]);
            Ok(())
        }
    }
}
