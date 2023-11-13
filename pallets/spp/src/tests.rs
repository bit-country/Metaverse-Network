// This file is part of Metaverse.Network & Bit.Country.

// Copyright (C) 2020-2022 Metaverse.Network & Bit.Country .
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

#![cfg(test)]

use frame_support::{assert_err, assert_noop, assert_ok};
use sp_runtime::traits::BadOrigin;
use sp_runtime::{Perbill, Permill};

use mock::{RuntimeEvent, *};

use crate::utils::PoolInfo;

use super::*;

#[test]
fn test_one() {
	ExtBuilder::default().build().execute_with(|| assert_eq!(1, 1));
}

#[test]
fn create_ksm_pool_works() {
	ExtBuilder::default()
		.ksm_setup_for_alice_and_bob()
		.build()
		.execute_with(|| {
			assert_ok!(SppModule::create_pool(
				RuntimeOrigin::signed(ALICE),
				FungibleTokenId::NativeToken(1),
				50,
				Permill::from_percent(5)
			));

			let next_pool_id = NextPoolId::<Runtime>::get();
			assert_eq!(next_pool_id, 1);
			assert_eq!(
				Pool::<Runtime>::get(next_pool_id - 1).unwrap(),
				PoolInfo::<AccountId> {
					creator: ALICE,
					commission: Permill::from_percent(5),
					currency_id: FungibleTokenId::NativeToken(1),
					max: 50
				}
			)
		});
}

#[test]
fn deposit_ksm_works() {
	ExtBuilder::default()
		.ksm_setup_for_alice_and_bob()
		.build()
		.execute_with(|| {
			assert_ok!(SppModule::create_pool(
				RuntimeOrigin::signed(ALICE),
				FungibleTokenId::NativeToken(1),
				50,
				Permill::from_percent(5)
			));

			let next_pool_id = NextPoolId::<Runtime>::get();
			assert_eq!(next_pool_id, 1);
			assert_eq!(
				Pool::<Runtime>::get(next_pool_id - 1).unwrap(),
				PoolInfo::<AccountId> {
					creator: ALICE,
					commission: Permill::from_percent(5),
					currency_id: FungibleTokenId::NativeToken(1),
					max: 50
				}
			);

			assert_ok!(SppModule::deposit(RuntimeOrigin::signed(BOB), 0, 10000));
			// This is true because fee hasn't been set up.
			assert_eq!(Tokens::accounts(BOB, FungibleTokenId::FungibleToken(1)).free, 10000);

			assert_eq!(PoolLedger::<Runtime>::get(0), 10000);
			assert_eq!(NetworkLedger::<Runtime>::get(FungibleTokenId::NativeToken(1)), 10000);

			// Deposit another 10000 KSM
			assert_ok!(SppModule::deposit(RuntimeOrigin::signed(BOB), 0, 10000));
			assert_eq!(Tokens::accounts(BOB, FungibleTokenId::FungibleToken(1)).free, 20000);

			assert_eq!(PoolLedger::<Runtime>::get(0), 20000);
			assert_eq!(NetworkLedger::<Runtime>::get(FungibleTokenId::NativeToken(1)), 20000);
		});
}
