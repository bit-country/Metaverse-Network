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
use orml_nft::Tokens;
use sp_runtime::traits::BadOrigin;
use sp_std::default::Default;

use auction_manager::ListingLevel;
use core_primitives::{Attributes, CollectionType, TokenType};
use mock::{Event, *};
use primitives::{CampaignInfo, GroupCollectionId};

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
			vec![1]
		));

		let campaign_info = CampaignInfo {
			creator: ALICE,
			properties: vec![1],
			reward: 10,
			claimed: 0,
			end: 10,
			cap: 10,
			cooling_off_duration: 10,
			trie_index: 0,
		};
		assert_eq!(Reward::campaigns(campaign_id), Some(campaign_info));
		assert_eq!(Balances::free_balance(ALICE), 9989);

		let event = mock::Event::Reward(crate::Event::NewRewardCampaignCreated(campaign_id, ALICE));
		assert_eq!(last_event(), event)
	});
}

#[test]
fn create_campaign_fails() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;

		assert_noop!(
			Reward::create_campaign(Origin::signed(ALICE), ALICE, 10, 2, 10, vec![1]),
			Error::<Runtime>::CampaignDurationBelowMinimum
		);

		run_to_block(11);
		assert_noop!(
			Reward::create_campaign(Origin::signed(ALICE), ALICE, 10, 10, 10, vec![1]),
			Error::<Runtime>::CampaignDurationBelowMinimum
		);

		assert_noop!(
			Reward::create_campaign(Origin::signed(ALICE), ALICE, 0, 10, 10, vec![1]),
			Error::<Runtime>::RewardPoolBelowMinimum
		);

		assert_noop!(
			Reward::create_campaign(Origin::signed(ALICE), ALICE, 10, 10, 1, vec![1]),
			Error::<Runtime>::CoolingOffPeriodBelowMinimum
		);
	});
}

#[test]
fn set_reward_works() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;
		assert_ok!(Reward::create_campaign(
			Origin::signed(ALICE),
			ALICE,
			10,
			10,
			10,
			vec![1]
		));

		let campaign_info = CampaignInfo {
			creator: ALICE,
			properties: vec![1],
			reward: 10,
			claimed: 0,
			end: 10,
			cap: 10,
			cooling_off_duration: 10,
			trie_index: 0,
		};
		assert_eq!(Reward::campaigns(campaign_id), Some(campaign_info));

		assert_ok!(Reward::set_reward(Origin::signed(ALICE), 0, BOB, 5));

		let campaign_info = CampaignInfo {
			creator: ALICE,
			properties: vec![1],
			reward: 10,
			claimed: 0,
			end: 10,
			cap: 5,
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
		assert_ok!(Reward::create_campaign(
			Origin::signed(ALICE),
			ALICE,
			10,
			10,
			10,
			vec![1]
		));

		let campaign_info = CampaignInfo {
			creator: ALICE,
			properties: vec![1],
			reward: 10,
			claimed: 0,
			end: 10,
			cap: 10,
			cooling_off_duration: 10,
			trie_index: 0,
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
		assert_ok!(Reward::create_campaign(
			Origin::signed(ALICE),
			ALICE,
			10,
			10,
			10,
			vec![1]
		));

		let campaign_info = CampaignInfo {
			creator: ALICE,
			properties: vec![1],
			reward: 10,
			claimed: 0,
			end: 10,
			cap: 10,
			cooling_off_duration: 10,
			trie_index: 0,
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
			reward: 10,
			claimed: 5,
			end: 10,
			cap: 5,
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
		assert_ok!(Reward::create_campaign(
			Origin::signed(ALICE),
			ALICE,
			10,
			10,
			10,
			vec![1]
		));

		let campaign_info = CampaignInfo {
			creator: ALICE,
			properties: vec![1],
			reward: 10,
			claimed: 0,
			end: 10,
			cap: 10,
			cooling_off_duration: 10,
			trie_index: 0,
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
		assert_ok!(Reward::create_campaign(Origin::signed(ALICE), BOB, 10, 10, 10, vec![1]));

		assert_eq!(Balances::free_balance(ALICE), 9989);

		run_to_block(100);

		assert_ok!(Reward::close_campaign(Origin::signed(BOB), 0));

		assert_eq!(Balances::free_balance(ALICE), 9989);
		assert_eq!(Balances::free_balance(BOB), 20010);

		assert_eq!(Campaigns::<Runtime>::get(campaign_id), None);

		let event = mock::Event::Reward(crate::Event::RewardCampaignClosed(campaign_id));
		assert_eq!(last_event(), event)
	});
}

#[test]
fn close_campaign_fails() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;
		assert_ok!(Reward::create_campaign(Origin::signed(ALICE), BOB, 10, 10, 10, vec![1]));

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
