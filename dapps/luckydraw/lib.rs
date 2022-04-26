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
#![allow(clippy::new_without_default)]

use ink_lang as ink;

#[ink::contract]
pub mod luckydraw {
    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_storage::collections::HashMap as StorageHashMap;

    #[ink(storage)]
    pub struct LuckyDraw {
        //Total balance of the pot
        total_balance: Balance,
        //Pot contract owner
        pot_owner: AccountId,
        //Player status
        player_status: StorageHashMap<AccountId, bool>,
        //The random winning number generated per play
        player_winning_numer: StorageHashMap<AccountId, u8>,
    }

    /// Errors that can occur upon calling this contract.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        /// The transfer has failed
        TransferFailed,
        /// Invalid Contract Owner
        InvalidOwner,
        BelowSubsistenceThreshold,
    }

    #[ink(event)]
    /// A winner has been picked
    pub struct WinnerPicked {
        #[ink(topic)]
        /// The winner account
        winner: AccountId,
        winning_number: u8,
    }

    #[ink(event)]
    /// A winner has been picked
    pub struct WinnerFail {
        #[ink(topic)]
        /// The winner account
        caller: AccountId,
        winning_number: u8,
    }

    impl LuckyDraw {
        /// Creates a new luckydraw smart contract initialized.
        #[ink(constructor)]
        pub fn default() -> Self {
            let caller = Self::env().caller();

            let lucky_draw_obj = Self {
                total_balance: 0,
                pot_owner: caller,
                player_status: StorageHashMap::default(),
                player_winning_numer: StorageHashMap::default(),
            };

            return lucky_draw_obj;
        }

        /// Generate random winning number.
        /// Later will accept the user number then compare the winning number
        #[ink(message, payable)]
        pub fn open_lucky_draw(&mut self, number: u8) -> Result<(), Error> {
            let caller = self.env().caller();
            let value = self.env().transferred_balance();

            //Check if player has played
            let player_status = self.player_status.contains_key(&caller);

            if player_status == true {
                self.player_status.take(&caller);
            }

            let player_winning_numer = self.player_winning_numer.contains_key(&caller);
            if player_winning_numer == true {
                self.player_winning_numer.take(&caller);
            }
            //random thread
            let x: u8 = Self::get_random();
            //Winning
            if x == number {
                self.player_winning_numer.insert(caller.clone(), x);
                self.player_status.insert(caller.clone(), true);

                self.env()
                    .transfer(caller, self.total_balance)
                    .map_err(|err| match err {
                        ink_env::Error::BelowSubsistenceThreshold => {
                            Error::BelowSubsistenceThreshold
                        }
                        _ => Error::TransferFailed,
                    });

                self.env().emit_event(WinnerPicked {
                    winner: caller,
                    winning_number: x,
                });
                return Ok(());
            }
            //Fail
            else {
                self.total_balance += value;
                self.env().emit_event(WinnerFail {
                    caller: caller,
                    winning_number: x,
                });

                return Ok(());
            }
        }

        /// Returns the current value of the luckydraw's bool.
        #[ink(message)]
        pub fn get_total_pot(&self) -> Balance {
            return self.total_balance;
        }

        #[ink(message, payable)]
        pub fn add_more_fund(&mut self) -> Result<(), Error> {
            let value = self.env().transferred_balance();

            self.total_balance += value;

            return Ok(());
        }

       /// fn get_simple_random() -> u8 {
       ///     let block_number = Self::env().block_number();
       /// }

        fn get_random() -> u8 {
            let seed: [u8; 1] = [1];
            let random_hash = Self::env().random(&seed);
            let random_number = Self::as_u8_be(&random_hash.as_ref());

            if (random_number % 2) == 0 {
                return 0 as u8;
            }
            return 1 as u8;
        }

        fn as_u8_be(array: &[u8]) -> u8 {
            (array[0] as u8) << 7
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn default_works() {
            let luckydraw = LuckyDraw::default();
           /// assert_eq!(luckydraw.get(), 0);
        }

        #[test]
        fn it_works() {
           /// let mut luckydraw = LuckyDraw::new();
           /// assert_eq!(luckydraw.get(), 0);
           /// luckydraw.open_lucky_draw();
           /// let winning_number = luckydraw.winning_number;
           /// assert_eq!(luckydraw.get(), winning_number);
        }
    }
}
