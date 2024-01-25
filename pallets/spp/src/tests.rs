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
use pallet_balances::BalanceLock;
use sp_runtime::traits::BadOrigin;
use sp_runtime::{Perbill, Permill};

use mock::{RuntimeEvent, *};
use primitives::bounded::FractionalRate;

use crate::utils::{BoostInfo, BoostingConviction, BoostingRecord, PoolInfo, PriorLock};

use super::*;

#[test]
fn test_one() {
	ExtBuilder::default().build().execute_with(|| assert_eq!(1, 1));
}

fn the_lock(amount: Balance) -> BalanceLock<Balance> {
	BalanceLock {
		id: BOOSTING_ID,
		amount,
		reasons: pallet_balances::Reasons::Misc,
	}
}

#[test]
fn create_ksm_pool_works() {
	ExtBuilder::default()
		.ksm_setup_for_alice_and_bob()
		.build()
		.execute_with(|| {
			// Create the first pool
			assert_ok!(SppModule::create_pool(
				RuntimeOrigin::signed(ALICE),
				FungibleTokenId::NativeToken(1),
				50,
				Rate::saturating_from_rational(5, 100)
			));

			// Check the next pool id will increment
			let next_pool_id: u32 = NextPoolId::<Runtime>::get();
			assert_eq!(next_pool_id, 2);

			// Check if the pool details as expected.
			assert_eq!(
				Pool::<Runtime>::get(next_pool_id - 1).unwrap(),
				PoolInfo::<AccountId> {
					creator: ALICE,
					commission: Rate::saturating_from_rational(5, 100),
					currency_id: FungibleTokenId::NativeToken(1),
					max: 50
				}
			);

			// Create a second pool
			assert_ok!(SppModule::create_pool(
				RuntimeOrigin::signed(BOB),
				FungibleTokenId::NativeToken(1),
				10,
				Rate::saturating_from_rational(1, 100),
			));

			// Check Id will increment
			let next_pool_id: u32 = NextPoolId::<Runtime>::get();
			assert_eq!(next_pool_id, 3);
			// Check the second pool has the right information as expected
			assert_eq!(
				Pool::<Runtime>::get(next_pool_id - 1).unwrap(),
				PoolInfo::<AccountId> {
					creator: BOB,
					commission: Rate::saturating_from_rational(1, 100),
					currency_id: FungibleTokenId::NativeToken(1),
					max: 10
				}
			);
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
				Rate::saturating_from_rational(5, 100)
			));

			let next_pool_id = NextPoolId::<Runtime>::get();
			assert_eq!(next_pool_id, 2);
			assert_eq!(
				Pool::<Runtime>::get(next_pool_id - 1).unwrap(),
				PoolInfo::<AccountId> {
					creator: ALICE,
					commission: Rate::saturating_from_rational(5, 100),
					currency_id: FungibleTokenId::NativeToken(1),
					max: 50
				}
			);

			assert_ok!(SppModule::deposit(RuntimeOrigin::signed(BOB), 1, 10000));
			// This is true because fee hasn't been set up.
			assert_eq!(Tokens::accounts(BOB, FungibleTokenId::FungibleToken(1)).free, 10000);

			assert_eq!(PoolLedger::<Runtime>::get(1), 10000);
			assert_eq!(NetworkLedger::<Runtime>::get(FungibleTokenId::NativeToken(1)), 10000);

			// Deposit another 10000 KSM
			assert_ok!(SppModule::deposit(RuntimeOrigin::signed(BOB), 1, 10000));
			assert_eq!(Tokens::accounts(BOB, FungibleTokenId::FungibleToken(1)).free, 20000);

			assert_eq!(PoolLedger::<Runtime>::get(1), 20000);
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
				Rate::saturating_from_rational(5, 100)
			));

			let next_pool_id = NextPoolId::<Runtime>::get();
			assert_eq!(next_pool_id, 2);
			assert_eq!(
				Pool::<Runtime>::get(next_pool_id - 1).unwrap(),
				PoolInfo::<AccountId> {
					creator: ALICE,
					commission: Rate::saturating_from_rational(5, 100),
					currency_id: FungibleTokenId::NativeToken(1),
					max: 50
				}
			);

			assert_ok!(SppModule::deposit(RuntimeOrigin::signed(BOB), 1, 10000));
			// This is true because fee hasn't been set up.
			assert_eq!(Tokens::accounts(BOB, FungibleTokenId::FungibleToken(1)).free, 10000);

			assert_eq!(PoolLedger::<Runtime>::get(1), 10000);
			assert_eq!(NetworkLedger::<Runtime>::get(FungibleTokenId::NativeToken(1)), 10000);

			// Deposit another 10000 KSM
			assert_ok!(SppModule::deposit(RuntimeOrigin::signed(BOB), 1, 10000));
			assert_eq!(Tokens::accounts(BOB, FungibleTokenId::FungibleToken(1)).free, 20000);

			assert_eq!(PoolLedger::<Runtime>::get(1), 20000);
			assert_eq!(NetworkLedger::<Runtime>::get(FungibleTokenId::NativeToken(1)), 20000);

			assert_noop!(
				SppModule::redeem(RuntimeOrigin::signed(BOB), 2, FungibleTokenId::FungibleToken(1), 10000),
				Error::<Runtime>::PoolDoesNotExist
			);

			assert_noop!(
				SppModule::redeem(RuntimeOrigin::signed(BOB), 1, FungibleTokenId::FungibleToken(0), 10000),
				Error::<Runtime>::CurrencyIsNotSupported
			);

			assert_noop!(
				SppModule::redeem(RuntimeOrigin::signed(BOB), 1, FungibleTokenId::FungibleToken(1), 10000),
				Error::<Runtime>::NoCurrentStakingRound
			);

			UnlockDuration::<Runtime>::insert(FungibleTokenId::NativeToken(1), StakingRound::Era(1));
			// Bump current staking round to 1
			CurrentStakingRound::<Runtime>::insert(FungibleTokenId::NativeToken(1), StakingRound::Era(1));
			assert_ok!(SppModule::redeem(
				RuntimeOrigin::signed(BOB),
				1,
				FungibleTokenId::FungibleToken(1),
				10000
			));

			// After Bob redeems, pool ledger 1 should have only 10000
			assert_eq!(PoolLedger::<Runtime>::get(1), 10000);

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

#[test]
fn current_era_update_works() {
	ExtBuilder::default()
		.ksm_setup_for_alice_and_bob()
		.build()
		.execute_with(|| {
			assert_eq!(SppModule::last_era_updated_block(), 0);
			assert_eq!(SppModule::update_era_frequency(), 0);
			assert_eq!(MockRelayBlockNumberProvider::current_block_number(), 0);
			// Current relaychain block is 102.
			//			MockRelayBlockNumberProvider::set(102);
			run_to_block(102);
			RelayChainCurrentEra::<Runtime>::put(1);
			IterationLimit::<Runtime>::put(50);
			// The correct set up era config is the last era block records is 101 with duration is 100 blocks
			assert_ok!(SppModule::update_era_config(
				RuntimeOrigin::signed(Admin::get()),
				Some(101),
				Some(100),
				StakingRound::Era(1),
				Some(Rate::saturating_from_rational(35, 100000)),
				Some((FungibleTokenId::NativeToken(1), StakingRound::Era(1))),
				Some(50),
				Some((FungibleTokenId::NativeToken(1), 0)),
				Some(100),
				Some((FungibleTokenId::NativeToken(1), StakingRound::Era(1)))
			));

			assert_ok!(SppModule::create_pool(
				RuntimeOrigin::signed(ALICE),
				FungibleTokenId::NativeToken(1),
				50,
				Rate::saturating_from_rational(5, 100)
			));

			let next_pool_id = NextPoolId::<Runtime>::get();
			assert_eq!(next_pool_id, 2);
			assert_eq!(
				Pool::<Runtime>::get(next_pool_id - 1).unwrap(),
				PoolInfo::<AccountId> {
					creator: ALICE,
					commission: Rate::saturating_from_rational(5, 100),
					currency_id: FungibleTokenId::NativeToken(1),
					max: 50,
				}
			);
			// Verify BOB account with 20000 KSM
			assert_eq!(Tokens::accounts(BOB, FungibleTokenId::NativeToken(1)).free, 20000);
			assert_ok!(SppModule::deposit(RuntimeOrigin::signed(BOB), 1, 10000));
			// This is true because fee hasn't been set up.
			assert_eq!(Tokens::accounts(BOB, FungibleTokenId::FungibleToken(1)).free, 10000);
			// Bob KSM balance become 10000
			assert_eq!(Tokens::accounts(BOB, FungibleTokenId::NativeToken(1)).free, 10000);

			assert_eq!(PoolLedger::<Runtime>::get(1), 10000);
			assert_eq!(NetworkLedger::<Runtime>::get(FungibleTokenId::NativeToken(1)), 10000);

			// Deposit another 10000 KSM
			assert_ok!(SppModule::deposit(RuntimeOrigin::signed(BOB), 1, 10000));
			assert_eq!(Tokens::accounts(BOB, FungibleTokenId::FungibleToken(1)).free, 20000);
			// Bob KSM now is 0
			assert_eq!(Tokens::accounts(BOB, FungibleTokenId::NativeToken(1)).free, 0);

			assert_eq!(PoolLedger::<Runtime>::get(1), 20000);
			assert_eq!(NetworkLedger::<Runtime>::get(FungibleTokenId::NativeToken(1)), 20000);

			// Pool summary
			// Pool Total deposited: 20000
			// Network deposited: 20000, NativeToken(1)

			// Bob summary
			// Holding: 20000 FungibleToken(1) reciept token of NativeToken(1)

			assert_noop!(
				SppModule::redeem(RuntimeOrigin::signed(BOB), 2, FungibleTokenId::FungibleToken(1), 10000),
				Error::<Runtime>::PoolDoesNotExist
			);

			assert_noop!(
				SppModule::redeem(RuntimeOrigin::signed(BOB), 1, FungibleTokenId::FungibleToken(0), 10000),
				Error::<Runtime>::CurrencyIsNotSupported
			);

			UnlockDuration::<Runtime>::insert(FungibleTokenId::NativeToken(1), StakingRound::Era(1)); // Bump current staking round to 1
			CurrentStakingRound::<Runtime>::insert(FungibleTokenId::NativeToken(1), StakingRound::Era(1));

			// Bob successfully redeemed
			assert_ok!(SppModule::redeem(
				RuntimeOrigin::signed(BOB),
				1,
				FungibleTokenId::FungibleToken(1),
				10000
			));

			// After Bob redeems, pool ledger 0 should have only 10000
			assert_eq!(PoolLedger::<Runtime>::get(1), 10000);

			// After Bob redeem, make sure BOB KSM balance remains the same as it will only released next era
			assert_eq!(Tokens::accounts(BOB, FungibleTokenId::NativeToken(1)).free, 0);

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

			// Move to era 2 to allow user redeem token successfully
			MockRelayBlockNumberProvider::set(202);
			run_to_block(202);
			SppModule::on_initialize(202);

			let pool_account = SppModule::get_pool_account();
			assert_eq!(
				Tokens::accounts(pool_account, FungibleTokenId::NativeToken(1)).free,
				10000
			);

			// After KSM released, BOB balance now is
			assert_eq!(Tokens::accounts(BOB, FungibleTokenId::NativeToken(1)).free, 10000);
			assert_eq!(
				CurrencyRedeemQueue::<Runtime>::get(FungibleTokenId::NativeToken(1), 0),
				None
			);
			assert_eq!(
				UserCurrencyRedeemQueue::<Runtime>::get(BOB, FungibleTokenId::NativeToken(1)),
				None
			);
			assert_eq!(
				StakingRoundRedeemQueue::<Runtime>::get(StakingRound::Era(2), FungibleTokenId::NativeToken(1)),
				None
			);

			// Move to era 3, make sure no double redeem process
			MockRelayBlockNumberProvider::set(302);
			SppModule::on_initialize(300);

			// Pool account remain the same
			assert_eq!(
				Tokens::accounts(pool_account, FungibleTokenId::NativeToken(1)).free,
				10000
			);

			// BOB balance remain the same
			assert_eq!(Tokens::accounts(BOB, FungibleTokenId::NativeToken(1)).free, 10000);
			assert_eq!(
				CurrencyRedeemQueue::<Runtime>::get(FungibleTokenId::NativeToken(1), 0),
				None
			);
			assert_eq!(
				UserCurrencyRedeemQueue::<Runtime>::get(BOB, FungibleTokenId::NativeToken(1)),
				None
			);
			assert_eq!(
				StakingRoundRedeemQueue::<Runtime>::get(StakingRound::Era(2), FungibleTokenId::NativeToken(1)),
				None
			);
			assert_eq!(
				StakingRoundRedeemQueue::<Runtime>::get(StakingRound::Era(3), FungibleTokenId::NativeToken(1)),
				None
			);
		});
}

#[test]
fn boosting_works() {
	ExtBuilder::default()
		.ksm_setup_for_alice_and_bob()
		.build()
		.execute_with(|| {
			assert_ok!(SppModule::create_pool(
				RuntimeOrigin::signed(ALICE),
				FungibleTokenId::NativeToken(1),
				50,
				Rate::saturating_from_rational(5, 100)
			));

			let next_pool_id = NextPoolId::<Runtime>::get();
			assert_eq!(next_pool_id, 2);
			assert_eq!(
				Pool::<Runtime>::get(next_pool_id - 1).unwrap(),
				PoolInfo::<AccountId> {
					creator: ALICE,
					commission: Rate::saturating_from_rational(5, 100),
					currency_id: FungibleTokenId::NativeToken(1),
					max: 50
				}
			);

			assert_ok!(SppModule::deposit(RuntimeOrigin::signed(BOB), 1, 10000));
			// This is true because fee hasn't been set up.
			assert_eq!(Tokens::accounts(BOB, FungibleTokenId::FungibleToken(1)).free, 10000);

			assert_eq!(PoolLedger::<Runtime>::get(1), 10000);
			assert_eq!(NetworkLedger::<Runtime>::get(FungibleTokenId::NativeToken(1)), 10000);

			// Deposit another 10000 KSM
			assert_ok!(SppModule::deposit(RuntimeOrigin::signed(BOB), 1, 10000));
			assert_eq!(Tokens::accounts(BOB, FungibleTokenId::FungibleToken(1)).free, 20000);

			assert_eq!(PoolLedger::<Runtime>::get(1), 20000);
			assert_eq!(NetworkLedger::<Runtime>::get(FungibleTokenId::NativeToken(1)), 20000);

			// Boosting works
			let bob_boost_balance = 1000;
			assert_ok!(SppModule::boost(
				RuntimeOrigin::signed(BOB),
				1,
				BoostInfo {
					balance: bob_boost_balance,
					conviction: BoostingConviction::None
				}
			));
			let boosting_of = BoostingOf::<Runtime>::get(BOB);
			let some_record = BoostingRecord {
				votes: vec![(
					1,
					BoostInfo {
						balance: bob_boost_balance,
						conviction: BoostingConviction::None,
					},
				)],
				prior: PriorLock(1, bob_boost_balance),
			};
			assert_eq!(boosting_of, some_record);
			assert_eq!(Balances::usable_balance(&BOB), 99000);
			let pool_1_shared_rewards = RewardsModule::shares_and_withdrawn_rewards(1, BOB);
			let network_shared_rewards = RewardsModule::shares_and_withdrawn_rewards(0, BOB);
			assert_eq!(pool_1_shared_rewards, (bob_boost_balance, Default::default()));
			assert_eq!(network_shared_rewards, (bob_boost_balance, Default::default()));

			// Second boost that will make total lock 11000
			assert_ok!(SppModule::boost(
				RuntimeOrigin::signed(BOB),
				1,
				BoostInfo {
					balance: 10000,
					conviction: BoostingConviction::None
				}
			));
			let second_boosting_of = BoostingOf::<Runtime>::get(BOB);
			let second_boosting_record = BoostingRecord {
				votes: vec![(
					1,
					BoostInfo {
						balance: 11000,
						conviction: BoostingConviction::None,
					},
				)],
				prior: PriorLock(1, 11000),
			};
			let view_votes = &second_boosting_of.votes;
			let debug_votes = &second_boosting_of.votes[0];
			assert_eq!(second_boosting_of, second_boosting_record);

			// Third boosting with lower balance than previous boost
			assert_ok!(SppModule::boost(
				RuntimeOrigin::signed(BOB),
				1,
				BoostInfo {
					balance: 500,
					conviction: BoostingConviction::None
				}
			));
			let third_boosting_of = BoostingOf::<Runtime>::get(BOB);
			let third_boosting_record = BoostingRecord {
				votes: vec![(
					1,
					BoostInfo {
						balance: 11500,
						conviction: BoostingConviction::None,
					},
				)],
				prior: PriorLock(1, 11500),
			};

			assert_eq!(third_boosting_of, third_boosting_record);
		});
}

#[test]
fn boosting_and_claim_reward_works() {
	ExtBuilder::default()
		.ksm_setup_for_alice_and_bob()
		.build()
		.execute_with(|| {
			// Era config set up
			// Current relaychain block is 102.
			// MockRelayBlockNumberProvider::set(102);
			run_to_block(102);
			RelayChainCurrentEra::<Runtime>::put(1);
			IterationLimit::<Runtime>::put(50);
			// The correct set up era config is the last era block records is 101 with duration is 100 blocks
			assert_ok!(SppModule::update_era_config(
				RuntimeOrigin::signed(Admin::get()),
				Some(101),
				Some(100),
				StakingRound::Era(1),
				Some(Rate::saturating_from_rational(35, 100000)),
				Some((FungibleTokenId::NativeToken(1), StakingRound::Era(1))),
				Some(50),
				Some((FungibleTokenId::NativeToken(1), 0)),
				Some(100),
				Some((FungibleTokenId::NativeToken(1), StakingRound::Era(1)))
			));

			assert_ok!(SppModule::create_pool(
				RuntimeOrigin::signed(ALICE),
				FungibleTokenId::NativeToken(1),
				50,
				Rate::saturating_from_rational(5, 100)
			));

			let next_pool_id = NextPoolId::<Runtime>::get();
			assert_eq!(next_pool_id, 2);
			assert_eq!(
				Pool::<Runtime>::get(next_pool_id - 1).unwrap(),
				PoolInfo::<AccountId> {
					creator: ALICE,
					commission: Rate::saturating_from_rational(5, 100),
					currency_id: FungibleTokenId::NativeToken(1),
					max: 50
				}
			);

			assert_ok!(SppModule::deposit(RuntimeOrigin::signed(BOB), 1, 10000));
			// This is true because fee hasn't been set up.
			assert_eq!(Tokens::accounts(BOB, FungibleTokenId::FungibleToken(1)).free, 10000);

			assert_eq!(PoolLedger::<Runtime>::get(1), 10000);
			assert_eq!(NetworkLedger::<Runtime>::get(FungibleTokenId::NativeToken(1)), 10000);

			// Deposit another 10000 KSM
			assert_ok!(SppModule::deposit(RuntimeOrigin::signed(BOB), 1, 10000));
			assert_eq!(Tokens::accounts(BOB, FungibleTokenId::FungibleToken(1)).free, 20000);

			assert_eq!(PoolLedger::<Runtime>::get(1), 20000);
			assert_eq!(NetworkLedger::<Runtime>::get(FungibleTokenId::NativeToken(1)), 20000);

			// Boosting works
			let bob_free_balance = Balances::free_balance(BOB);
			assert_ok!(SppModule::boost(
				RuntimeOrigin::signed(BOB),
				1,
				BoostInfo {
					balance: 15000,
					conviction: BoostingConviction::None
				}
			));
			let boosting_of = BoostingOf::<Runtime>::get(BOB);
			let some_record = BoostingRecord {
				votes: vec![(
					1,
					BoostInfo {
						balance: 15000,
						conviction: BoostingConviction::None,
					},
				)],
				prior: PriorLock(202, 15000),
			};
			assert_eq!(boosting_of, some_record);
			assert_eq!(Balances::usable_balance(&BOB), bob_free_balance - 15000);
			let pool_1_shared_rewards = RewardsModule::shares_and_withdrawn_rewards(1, BOB);
			let network_shared_rewards = RewardsModule::shares_and_withdrawn_rewards(0, BOB);
			assert_eq!(pool_1_shared_rewards, (15000, Default::default()));
			assert_eq!(network_shared_rewards, (15000, Default::default()));

			// Set reward per era. - 1000 NativeToken(0) per 100 blocks
			RewardEraFrequency::<Runtime>::put(1000);
			// Simulate Council transfer 10000 NativeToken to reward_payout_account so that account has
			// sufficient balance for reward distribution
			let reward_holding_account = SppModule::get_reward_holding_account_id();
			assert_ok!(Balances::transfer(
				RuntimeOrigin::signed(ALICE),
				reward_holding_account.clone(),
				10000
			));

			// Move to era 2, now protocol distribute 1000 NEER to incentivise boosters
			//			MockRelayBlockNumberProvider::set(202);
			run_to_block(202);
			SppModule::on_initialize(200);

			let network_reward_pool = RewardsModule::pool_infos(0u32);
			let reward_accumulated = RewardsModule::shares_and_withdrawn_rewards(0, BOB);

			// Verify after 1 era, total rewards should have 1000 NEER and 0 claimed
			assert_eq!(
				network_reward_pool,
				orml_rewards::PoolInfo {
					total_shares: 15000,
					rewards: vec![(FungibleTokenId::NativeToken(0), (1000, 0))].into_iter().collect()
				}
			);

			// Reward records of BOB holding 15000 shares and 0 claimed
			assert_eq!(reward_accumulated, (15000, Default::default()));
			// Reward distribution works, now claim rewards
			let bob_balance_before_claiming_boosting_reward = Balances::free_balance(BOB);
			// Bob claim rewards
			assert_ok!(SppModule::claim_rewards(RuntimeOrigin::signed(BOB), 0));
			assert_eq!(
				last_event(),
				mock::RuntimeEvent::Spp(crate::Event::ClaimRewards {
					who: BOB,
					pool: 0,
					reward_currency_id: FungibleTokenId::NativeToken(0),
					claimed_amount: 1000,
				})
			);

			// Bob free balance now will be bob_balance_before_claiming_boosting_reward + 1000 as claimed reward
			assert_eq!(
				Balances::free_balance(BOB),
				bob_balance_before_claiming_boosting_reward + 1000
			);

			// Bob try to claim again but getting no reward
			assert_ok!(SppModule::claim_rewards(RuntimeOrigin::signed(BOB), 0));
			// Bob balance doesn't increase
			assert_eq!(
				Balances::free_balance(BOB),
				bob_balance_before_claiming_boosting_reward + 1000
			);

			// Move to era 3, now protocol distribute another 1000 NEER to incentivise boosters
			//			MockRelayBlockNumberProvider::set(302);
			run_to_block(302);
			SppModule::on_initialize(300);

			// Bob try to claim reward for new era
			assert_ok!(SppModule::claim_rewards(RuntimeOrigin::signed(BOB), 0));
			// Bob balance should increase 2000
			assert_eq!(
				Balances::free_balance(BOB),
				bob_balance_before_claiming_boosting_reward + 2000
			);

			// Charlie now boost pool 1 with 15000 (share 50% of reward with Bob)
			assert_ok!(SppModule::boost(
				RuntimeOrigin::signed(CHARLIE),
				1,
				BoostInfo {
					balance: 15000,
					conviction: BoostingConviction::None
				}
			));
			// Charlie now should have 15000 shares in the pool
			assert_eq!(
				RewardsModule::shares_and_withdrawn_rewards(1, CHARLIE),
				(15000, Default::default())
			);

			// Network pool ledger should have total shares of 30,000 , 2000 total reward and claimed 2000 by
			// Bob. However, as Charlie boosted, network pool inflate 15,000 shares, added 50% reward and 50%
			// claimed reward to avoid dilution.
			assert_eq!(
				RewardsModule::pool_infos(0u32),
				orml_rewards::PoolInfo {
					total_shares: 30000,
					rewards: vec![(FungibleTokenId::NativeToken(0), (4000, 4000))]
						.into_iter()
						.collect()
				}
			);

			let charlie_balance_before_claiming_boosting_reward = Balances::free_balance(CHARLIE);

			// Move to era 4, now protocol distribute another 1000 NEER to incentivise boosters
			MockRelayBlockNumberProvider::set(402);
			run_to_block(402);
			SppModule::on_initialize(400);

			// Bob try to claim reward for new era
			assert_ok!(SppModule::claim_rewards(RuntimeOrigin::signed(BOB), 0));
			// Bob balance should increase 500 as Charlie shares 50% rewards
			assert_eq!(
				Balances::free_balance(BOB),
				bob_balance_before_claiming_boosting_reward + 2500
			);

			// Charlie try to claim reward for new era
			assert_ok!(SppModule::claim_rewards(RuntimeOrigin::signed(CHARLIE), 0));
			// Charlie balance should increase 500
			assert_eq!(
				Balances::free_balance(CHARLIE),
				charlie_balance_before_claiming_boosting_reward + 500
			);
		});
}

#[test]
fn reward_distribution_works() {
	ExtBuilder::default()
		.ksm_setup_for_alice_and_bob()
		.build()
		.execute_with(|| {
			// Era config set up
			// Current relaychain block is 102.
			// MockRelayBlockNumberProvider::set(102);
			run_to_block(102);
			RelayChainCurrentEra::<Runtime>::put(1);
			IterationLimit::<Runtime>::put(50);
			UnlockDuration::<Runtime>::insert(FungibleTokenId::NativeToken(1), StakingRound::Era(1)); // Bump current staking round to 1
			CurrentStakingRound::<Runtime>::insert(FungibleTokenId::NativeToken(1), StakingRound::Era(1));
			// The correct set up era config is the last era block records is 101 with duration is 100 blocks
			assert_ok!(SppModule::update_era_config(
				RuntimeOrigin::signed(Admin::get()),
				Some(101),
				Some(100),
				StakingRound::Era(1),
				Some(Rate::saturating_from_rational(20, 100)), // Set reward rate per era is 20%.
				Some((FungibleTokenId::NativeToken(1), StakingRound::Era(1))),
				Some(50),
				Some((FungibleTokenId::NativeToken(1), 0)),
				Some(100),
				Some((FungibleTokenId::NativeToken(1), StakingRound::Era(1)))
			));

			assert_ok!(SppModule::create_pool(
				RuntimeOrigin::signed(ALICE),
				FungibleTokenId::NativeToken(1),
				50,
				Rate::saturating_from_rational(5, 100)
			));

			let next_pool_id = NextPoolId::<Runtime>::get();
			assert_eq!(next_pool_id, 2);
			assert_eq!(
				Pool::<Runtime>::get(next_pool_id - 1).unwrap(),
				PoolInfo::<AccountId> {
					creator: ALICE,
					commission: Rate::saturating_from_rational(5, 100),
					currency_id: FungibleTokenId::NativeToken(1),
					max: 50
				}
			);

			assert_ok!(SppModule::deposit(RuntimeOrigin::signed(BOB), 1, 10000));
			// This is true because fee hasn't been set up.
			assert_eq!(Tokens::accounts(BOB, FungibleTokenId::FungibleToken(1)).free, 10000);

			assert_eq!(PoolLedger::<Runtime>::get(1), 10000);
			assert_eq!(NetworkLedger::<Runtime>::get(FungibleTokenId::NativeToken(1)), 10000);

			// Boosting works
			let bob_free_balance = Balances::free_balance(BOB);
			assert_ok!(SppModule::boost(
				RuntimeOrigin::signed(BOB),
				1,
				BoostInfo {
					balance: 15000,
					conviction: BoostingConviction::None
				}
			));
			let boosting_of = BoostingOf::<Runtime>::get(BOB);
			let some_record = BoostingRecord {
				votes: vec![(
					1,
					BoostInfo {
						balance: 15000,
						conviction: BoostingConviction::None,
					},
				)],
				prior: PriorLock(202, 15000),
			};
			assert_eq!(boosting_of, some_record);
			assert_eq!(Balances::usable_balance(&BOB), bob_free_balance - 15000);
			let pool_1_shared_rewards = RewardsModule::shares_and_withdrawn_rewards(1, BOB);
			let network_shared_rewards = RewardsModule::shares_and_withdrawn_rewards(0, BOB);
			assert_eq!(pool_1_shared_rewards, (15000, Default::default()));
			assert_eq!(network_shared_rewards, (15000, Default::default()));

			// Charlie boosted with 15000 Native token
			assert_ok!(SppModule::boost(
				RuntimeOrigin::signed(CHARLIE),
				1,
				BoostInfo {
					balance: 15000,
					conviction: BoostingConviction::None
				}
			));
			// Charlie now should have 15000 shares in the pool
			assert_eq!(
				RewardsModule::shares_and_withdrawn_rewards(1, CHARLIE),
				(15000, Default::default())
			);

			// Move to era 2
			MockRelayBlockNumberProvider::set(202);
			SppModule::on_initialize(200);

			assert_ok!(SppModule::handle_reward_distribution_to_pool_treasury(1, 2));
			let pool_treasury = SppModule::get_pool_treasury(1);

			assert_eq!(
				Currencies::free_balance(FungibleTokenId::FungibleToken(1), &pool_treasury),
				20
			);

			assert_eq!(Currencies::total_issuance(FungibleTokenId::FungibleToken(1)), 10020);
			assert_eq!(PoolLedger::<Runtime>::get(1), 12000);
			assert_eq!(NetworkLedger::<Runtime>::get(FungibleTokenId::NativeToken(1)), 12000);
		});
}
