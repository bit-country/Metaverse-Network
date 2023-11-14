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

#[test]
fn redeem_rksm_request_works() {
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

			assert_noop!(
				SppModule::redeem(RuntimeOrigin::signed(BOB), 1, FungibleTokenId::FungibleToken(1), 10000),
				Error::<Runtime>::PoolDoesNotExist
			);

			assert_noop!(
				SppModule::redeem(RuntimeOrigin::signed(BOB), 0, FungibleTokenId::FungibleToken(0), 10000),
				Error::<Runtime>::CurrencyIsNotSupported
			);

			assert_noop!(
				SppModule::redeem(RuntimeOrigin::signed(BOB), 0, FungibleTokenId::FungibleToken(1), 10000),
				Error::<Runtime>::NoCurrentStakingRound
			);

			UnlockDuration::<Runtime>::insert(FungibleTokenId::NativeToken(1), StakingRound::Era(1));
			// Bump current staking round to 1
			CurrentStakingRound::<Runtime>::insert(FungibleTokenId::NativeToken(1), StakingRound::Era(1));
			assert_ok!(SppModule::redeem(
				RuntimeOrigin::signed(BOB),
				0,
				FungibleTokenId::FungibleToken(1),
				10000
			));

			// After Bob redeems, pool ledger 0 should have only 10000
			assert_eq!(PoolLedger::<Runtime>::get(0), 10000);

			// Verify if redeem queue has requests

			let queue_id = QueueNextId::<Runtime>::get(FungibleTokenId::NativeToken(1));
			assert_eq!(queue_id, 1);
			let mut queue_items = BoundedVec::default();
			assert_ok!(queue_items.try_push(0));
			let user_redeem_queue = UserCurrencyRedeemQueue::<Runtime>::get(BOB, FungibleTokenId::NativeToken(1));
			let currency_redeem_queue = CurrencyRedeemQueue::<Runtime>::get(FungibleTokenId::NativeToken(1), 0);
			let staking_round_redeem_queue =
				StakingRoundRedeemQueue::<Runtime>::get(StakingRound::Era(2), FungibleTokenId::NativeToken(1));
			// Verify if user redeem queue has total unlocked and queue items
			assert_eq!(user_redeem_queue, Some((10000, queue_items.clone())));
			// If user redeem of Era 1, fund will be released at Era 2
			assert_eq!(currency_redeem_queue, Some((BOB, 10000, StakingRound::Era(2))));
			// Redeem added into staking round redeem queue for Era 2
			assert_eq!(
				staking_round_redeem_queue,
				Some((10000, queue_items.clone(), FungibleTokenId::NativeToken(1)))
			);
		});
}
