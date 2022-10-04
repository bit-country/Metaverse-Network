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

use frame_support::{assert_err, assert_noop, assert_ok, sp_runtime::runtime_logger};
use sp_std::default::Default;

use mock::{Event, *};
use primitives::{CampaignInfo, FungibleTokenId};

use super::*;

#[test]
fn basic_setup_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(System::block_number(), 1);
		assert_eq!(Reward::campaigns(0), None);
	});
}

#[test]
fn create_campaign_works() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;
		assert_ok!(Reward::create_campaign(
			Origin::signed(ALICE),
			ALICE,
			10,
			10,
			10,
			vec![1],
			FungibleTokenId::NativeToken(0),
		));

		let campaign_info = CampaignInfo {
			creator: ALICE,
			properties: vec![1],
			cooling_off_duration: 10,
			trie_index: 0,
			end: 10,
			reward: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 10),
			claimed: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 0),
			cap: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 10),
		};
		assert_eq!(Reward::campaigns(campaign_id), Some(campaign_info));
		assert_eq!(Balances::free_balance(ALICE), 9989);

		let event = mock::Event::Reward(crate::Event::NewRewardCampaignCreated(campaign_id, ALICE));
		assert_eq!(last_event(), event)
	});
}

#[test]
fn create_multicurrency_campaign_works() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;
		assert_ok!(Reward::create_campaign(
			Origin::signed(ALICE),
			ALICE,
			10,
			10,
			10,
			vec![1],
			FungibleTokenId::MiningResource(0),
		));

		let campaign_info = CampaignInfo {
			creator: ALICE,
			properties: vec![1],
			cooling_off_duration: 10,
			trie_index: 0,
			end: 10,
			reward: RewardType::FungibleTokens(FungibleTokenId::MiningResource(0), 10),
			claimed: RewardType::FungibleTokens(FungibleTokenId::MiningResource(0), 0),
			cap: RewardType::FungibleTokens(FungibleTokenId::MiningResource(0), 10),
		};
		assert_eq!(Reward::campaigns(campaign_id), Some(campaign_info));
		assert_eq!(Balances::free_balance(ALICE), 9999);
		assert_eq!(Tokens::accounts(ALICE, FungibleTokenId::MiningResource(0)).free, 9990);

		let event = mock::Event::Reward(crate::Event::NewRewardCampaignCreated(campaign_id, ALICE));
		assert_eq!(last_event(), event)
	});
}

#[test]
fn create_campaign_fails() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;

		assert_noop!(
			Reward::create_campaign(
				Origin::signed(ALICE),
				ALICE,
				10,
				2,
				10,
				vec![1],
				FungibleTokenId::NativeToken(0)
			),
			Error::<Runtime>::CampaignDurationBelowMinimum
		);

		assert_noop!(
			Reward::create_campaign(
				Origin::signed(ALICE),
				ALICE,
				0,
				10,
				10,
				vec![1],
				FungibleTokenId::NativeToken(0)
			),
			Error::<Runtime>::RewardPoolBelowMinimum
		);

		assert_noop!(
			Reward::create_campaign(
				Origin::signed(ALICE),
				ALICE,
				10,
				10,
				1,
				vec![1],
				FungibleTokenId::NativeToken(0)
			),
			Error::<Runtime>::CoolingOffPeriodBelowMinimum
		);
	});
}

#[test]
fn set_reward_works() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;
		assert_ok!(Reward::add_set_reward_origin(Origin::signed(ALICE), ALICE));

		assert_ok!(Reward::create_campaign(
			Origin::signed(ALICE),
			ALICE,
			10,
			10,
			10,
			vec![1],
			FungibleTokenId::NativeToken(0),
		));

		let campaign_info = CampaignInfo {
			creator: ALICE,
			properties: vec![1],
			cooling_off_duration: 10,
			trie_index: 0,
			end: 10,
			reward: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 10),
			claimed: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 0),
			cap: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 10),
		};
		assert_eq!(Reward::campaigns(campaign_id), Some(campaign_info));

		assert_ok!(Reward::set_reward(Origin::signed(ALICE), 0, BOB, 5));

		let campaign_info = CampaignInfo {
			creator: ALICE,
			properties: vec![1],
			reward: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 10),
			claimed: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 0),
			end: 10,
			cap: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 5),
			cooling_off_duration: 10,
			trie_index: 0,
		};
		assert_eq!(Reward::campaigns(campaign_id), Some(campaign_info));

		let event = mock::Event::Reward(crate::Event::SetReward(campaign_id, BOB, 5u32.into()));
		assert_eq!(last_event(), event)
	});
}

#[test]
fn set_reward_fails() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;
		assert_ok!(Reward::add_set_reward_origin(Origin::signed(ALICE), ALICE));
		assert_ok!(Reward::create_campaign(
			Origin::signed(ALICE),
			ALICE,
			10,
			10,
			10,
			vec![1],
			FungibleTokenId::NativeToken(0),
		));

		let campaign_info = CampaignInfo {
			creator: ALICE,
			properties: vec![1],
			cooling_off_duration: 10,
			trie_index: 0,
			end: 10,
			reward: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 10),
			claimed: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 0),
			cap: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 10),
		};
		assert_eq!(Reward::campaigns(campaign_id), Some(campaign_info));

		assert_noop!(
			Reward::set_reward(Origin::signed(ALICE), 1, BOB, 10),
			Error::<Runtime>::CampaignIsNotFound
		);

		assert_noop!(
			Reward::set_reward(Origin::signed(ALICE), 0, BOB, 11),
			Error::<Runtime>::RewardExceedCap
		);

		assert_ok!(Reward::set_reward(Origin::signed(ALICE), 0, BOB, 5));

		assert_noop!(
			Reward::set_reward(Origin::signed(ALICE), 0, ALICE, 6),
			Error::<Runtime>::RewardExceedCap
		);

		assert_noop!(
			Reward::set_reward(Origin::signed(3), 0, BOB, 5),
			Error::<Runtime>::InvalidSetRewardOrigin
		);

		run_to_block(21);

		assert_noop!(
			Reward::set_reward(Origin::signed(ALICE), 0, BOB, 5),
			Error::<Runtime>::CampaignExpired
		);
	});
}

#[test]
fn claim_reward_works() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;
		assert_ok!(Reward::add_set_reward_origin(Origin::signed(ALICE), ALICE));
		assert_ok!(Reward::create_campaign(
			Origin::signed(ALICE),
			ALICE,
			10,
			10,
			10,
			vec![1],
			FungibleTokenId::NativeToken(0),
		));

		let campaign_info = CampaignInfo {
			creator: ALICE,
			properties: vec![1],
			cooling_off_duration: 10,
			trie_index: 0,
			end: 10,
			reward: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 10),
			claimed: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 0),
			cap: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 10),
		};
		assert_eq!(Reward::campaigns(campaign_id), Some(campaign_info));
		assert_ok!(Reward::set_reward(Origin::signed(ALICE), 0, BOB, 5));

		run_to_block(17);
		//assert_eq!(last_event(), mock::Event::Reward(crate::Event::RewardCampaignEnded(0)));

		assert_ok!(Reward::claim_reward(Origin::signed(BOB), 0));
		assert_eq!(Balances::free_balance(BOB), 20005);

		let campaign_info_after_claim = CampaignInfo {
			creator: ALICE,
			properties: vec![1],
			reward: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 10),
			claimed: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 5),
			end: 10,
			cap: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 5),
			cooling_off_duration: 10,
			trie_index: 0,
		};
		assert_eq!(Reward::campaigns(campaign_id), Some(campaign_info_after_claim));

		let event = mock::Event::Reward(crate::Event::RewardClaimed(campaign_id, BOB, 5u32.into()));
		assert_eq!(last_event(), event)
	});
}

#[test]
fn claim_multicurrency_reward_works() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;
		assert_ok!(Reward::add_set_reward_origin(Origin::signed(ALICE), ALICE));
		assert_ok!(Reward::create_campaign(
			Origin::signed(ALICE),
			ALICE,
			10,
			10,
			10,
			vec![1],
			FungibleTokenId::MiningResource(0),
		));

		let campaign_info = CampaignInfo {
			creator: ALICE,
			properties: vec![1],
			cooling_off_duration: 10,
			trie_index: 0,
			end: 10,
			reward: RewardType::FungibleTokens(FungibleTokenId::MiningResource(0), 10),
			claimed: RewardType::FungibleTokens(FungibleTokenId::MiningResource(0), 0),
			cap: RewardType::FungibleTokens(FungibleTokenId::MiningResource(0), 10),
		};
		assert_eq!(Reward::campaigns(campaign_id), Some(campaign_info));
		assert_ok!(Reward::set_reward(Origin::signed(ALICE), 0, BOB, 5));

		run_to_block(17);
		//assert_eq!(last_event(), mock::Event::Reward(crate::Event::RewardCampaignEnded(0)));

		assert_ok!(Reward::claim_reward(Origin::signed(BOB), 0));
		assert_eq!(Tokens::accounts(BOB, FungibleTokenId::MiningResource(0)).free, 5005);

		let campaign_info_after_claim = CampaignInfo {
			creator: ALICE,
			properties: vec![1],
			reward: RewardType::FungibleTokens(FungibleTokenId::MiningResource(0), 10),
			claimed: RewardType::FungibleTokens(FungibleTokenId::MiningResource(0), 5),
			end: 10,
			cap: RewardType::FungibleTokens(FungibleTokenId::MiningResource(0), 5),
			cooling_off_duration: 10,
			trie_index: 0,
		};
		assert_eq!(Reward::campaigns(campaign_id), Some(campaign_info_after_claim));

		let event = mock::Event::Reward(crate::Event::RewardClaimed(campaign_id, BOB, 5u32.into()));
		assert_eq!(last_event(), event)
	});
}

#[test]
fn claim_reward_fails() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;
		assert_ok!(Reward::add_set_reward_origin(Origin::signed(ALICE), ALICE));
		assert_ok!(Reward::create_campaign(
			Origin::signed(ALICE),
			ALICE,
			10,
			10,
			10,
			vec![1],
			FungibleTokenId::NativeToken(0),
		));

		let campaign_info = CampaignInfo {
			creator: ALICE,
			properties: vec![1],
			cooling_off_duration: 10,
			trie_index: 0,
			end: 10,
			reward: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 10),
			claimed: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 0),
			cap: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 10),
		};

		assert_eq!(Reward::campaigns(campaign_id), Some(campaign_info));
		assert_ok!(Reward::set_reward(Origin::signed(ALICE), 0, BOB, 5));

		run_to_block(9);

		assert_noop!(
			Reward::claim_reward(Origin::signed(BOB), 0),
			Error::<Runtime>::CampaignStillActive
		);

		run_to_block(17);

		assert_noop!(
			Reward::claim_reward(Origin::signed(ALICE), 1),
			Error::<Runtime>::CampaignIsNotFound
		);

		assert_noop!(
			Reward::claim_reward(Origin::signed(ALICE), 0),
			Error::<Runtime>::NoRewardFound
		);

		assert_noop!(
			Reward::claim_reward(Origin::signed(3), 0),
			Error::<Runtime>::NoRewardFound
		);

		run_to_block(23);

		assert_noop!(
			Reward::claim_reward(Origin::signed(BOB), 0),
			Error::<Runtime>::CampaignExpired
		);
	});
}

#[test]
fn close_campaign_works() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;
		assert_ok!(Reward::create_campaign(
			Origin::signed(ALICE),
			BOB,
			10,
			10,
			10,
			vec![1],
			FungibleTokenId::NativeToken(0)
		));

		assert_eq!(Balances::free_balance(ALICE), 9989);

		run_to_block(100);

		assert_ok!(Reward::close_campaign(Origin::signed(BOB), 0));

		assert_eq!(Balances::free_balance(ALICE), 9989);
		assert_eq!(Balances::free_balance(BOB), 20011);

		assert_eq!(Campaigns::<Runtime>::get(campaign_id), None);

		let event = mock::Event::Reward(crate::Event::RewardCampaignClosed(campaign_id));
		assert_eq!(last_event(), event)
	});
}

#[test]
fn close_multicurrency_campaign_works() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;
		assert_ok!(Reward::create_campaign(
			Origin::signed(ALICE),
			BOB,
			10,
			10,
			10,
			vec![1],
			FungibleTokenId::MiningResource(0)
		));

		assert_eq!(Balances::free_balance(ALICE), 9999);
		assert_eq!(Tokens::accounts(ALICE, FungibleTokenId::MiningResource(0)).free, 9990);

		run_to_block(100);

		assert_ok!(Reward::close_campaign(Origin::signed(BOB), 0));

		assert_eq!(Balances::free_balance(ALICE), 9999);
		assert_eq!(Tokens::accounts(ALICE, FungibleTokenId::MiningResource(0)).free, 9990);
		assert_eq!(Balances::free_balance(BOB), 20001);
		assert_eq!(Tokens::accounts(BOB, FungibleTokenId::MiningResource(0)).free, 5010);

		assert_eq!(Campaigns::<Runtime>::get(campaign_id), None);

		let event = mock::Event::Reward(crate::Event::RewardCampaignClosed(campaign_id));
		assert_eq!(last_event(), event)
	});
}

#[test]
fn close_campaign_fails() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;
		assert_ok!(Reward::create_campaign(
			Origin::signed(ALICE),
			BOB,
			10,
			10,
			10,
			vec![1],
			FungibleTokenId::NativeToken(0)
		));

		run_to_block(17);

		assert_noop!(
			Reward::close_campaign(Origin::signed(ALICE), 1),
			Error::<Runtime>::CampaignIsNotFound
		);

		assert_noop!(
			Reward::close_campaign(Origin::signed(ALICE), 0),
			Error::<Runtime>::NotCampaignCreator
		);

		assert_noop!(
			Reward::close_campaign(Origin::signed(BOB), 0),
			Error::<Runtime>::CampaignStillActive
		);
	});
}

#[test]
fn cancel_campaign_works() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;
		assert_ok!(Reward::create_campaign(
			Origin::signed(ALICE),
			BOB,
			10,
			10,
			10,
			vec![1],
			FungibleTokenId::NativeToken(0)
		));

		assert_eq!(Balances::free_balance(ALICE), 9989);

		run_to_block(5);

		assert_ok!(Reward::cancel_campaign(Origin::signed(ALICE), 0));

		assert_eq!(Balances::free_balance(ALICE), 9989);
		assert_eq!(Balances::free_balance(BOB), 20011);

		assert_eq!(Campaigns::<Runtime>::get(campaign_id), None);

		let event = mock::Event::Reward(crate::Event::RewardCampaignCanceled(campaign_id));
		assert_eq!(last_event(), event)
	});
}

#[test]
fn cancel_multicurrency_campaign_works() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;
		assert_ok!(Reward::create_campaign(
			Origin::signed(ALICE),
			BOB,
			10,
			10,
			10,
			vec![1],
			FungibleTokenId::MiningResource(0)
		));

		assert_eq!(Balances::free_balance(ALICE), 9999);
		assert_eq!(Tokens::accounts(ALICE, FungibleTokenId::MiningResource(0)).free, 9990);

		run_to_block(5);

		assert_ok!(Reward::cancel_campaign(Origin::signed(ALICE), 0));

		assert_eq!(Balances::free_balance(ALICE), 9999);
		assert_eq!(Tokens::accounts(ALICE, FungibleTokenId::MiningResource(0)).free, 9990);
		assert_eq!(Balances::free_balance(BOB), 20001);
		assert_eq!(Tokens::accounts(BOB, FungibleTokenId::MiningResource(0)).free, 5010);

		assert_eq!(Campaigns::<Runtime>::get(campaign_id), None);

		let event = mock::Event::Reward(crate::Event::RewardCampaignCanceled(campaign_id));
		assert_eq!(last_event(), event)
	});
}

#[test]
fn cancel_campaign_fails() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;
		assert_ok!(Reward::create_campaign(
			Origin::signed(ALICE),
			BOB,
			10,
			10,
			10,
			vec![1],
			FungibleTokenId::NativeToken(0)
		));

		assert_noop!(
			Reward::cancel_campaign(Origin::signed(ALICE), 1),
			Error::<Runtime>::CampaignIsNotFound
		);

		run_to_block(11);

		assert_noop!(
			Reward::cancel_campaign(Origin::signed(ALICE), 0),
			Error::<Runtime>::CampaignEnded
		);
	});
}

#[test]
fn add_reward_origin_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Reward::add_set_reward_origin(Origin::signed(ALICE), ALICE));
		assert_eq!(Reward::is_set_reward_origin(&ALICE), true);
		let event = mock::Event::Reward(crate::Event::SetRewardOriginAdded(ALICE));
		assert_eq!(last_event(), event)
	});
}

#[test]
fn add_reward_origin_fails() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Reward::add_set_reward_origin(Origin::signed(ALICE), ALICE));
		assert_noop!(
			Reward::add_set_reward_origin(Origin::signed(ALICE), ALICE),
			Error::<Runtime>::SetRewardOriginAlreadyAdded
		);
	});
}

#[test]
fn remove_reward_origin_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Reward::add_set_reward_origin(Origin::signed(ALICE), ALICE));
		assert_eq!(Reward::is_set_reward_origin(&ALICE), true);
		assert_ok!(Reward::remove_set_reward_origin(Origin::signed(ALICE), ALICE));
		assert_eq!(Reward::is_set_reward_origin(&ALICE), false);
		let event = mock::Event::Reward(crate::Event::SetRewardOriginRemoved(ALICE));
		assert_eq!(last_event(), event)
	});
}

#[test]
fn remove_reward_origin_fails() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Reward::remove_set_reward_origin(Origin::signed(ALICE), ALICE),
			Error::<Runtime>::SetRewardOriginDoesNotExist
		);
	});
}
