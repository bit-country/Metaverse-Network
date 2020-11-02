// Copyright 2018-2020 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;
use rand::prelude::*;

#[ink::contract]
pub mod luckydraw {
    #[ink(storage)]
    pub struct LuckyDraw {
        winning_number: u64,
    }

    impl LuckyDraw {
        /// Creates a new luckydraw smart contract initialized with the given value.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                //Can add inital winning prob here
                winning_number: 0,
            }
        }

        /// Creates a new luckydraw smart contract initialized to `false`.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new()
        }

        /// Generate random winning number.
        /// Later will accept the user number then compare the winning number
        #[ink(message)]
        pub fn open_lucky_draw(&mut self) {
            //random thread
            let x: u64 = rand::random();
            self.winning_number = x;
        }

        /// Returns the current value of the luckydraw's bool.
        #[ink(message)]
        pub fn get(&self) -> u64 {
            self.winning_number
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn default_works() {
            let luckydraw = LuckyDraw::default();
            assert_eq!(luckydraw.get(), 0);
        }

        #[test]
        fn it_works() {
            let mut luckydraw = LuckyDraw::new();
            assert_eq!(luckydraw.get(), 0);
            luckydraw.open_lucky_draw();
            let winning_number = luckydraw.winning_number;
            assert_eq!(luckydraw.get(), winning_number);
        }
    }
}
