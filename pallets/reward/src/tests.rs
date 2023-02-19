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
use sp_runtime::app_crypto::Ss58Codec;
use sp_std::collections::btree_map::BTreeMap;
use sp_std::default::Default;
use sp_core::crypto::AccountId32;

use super::*;
use core_primitives::Attributes;
use hex_literal::hex;
use mock::{Balance, Event, *};
use primitives::{CampaignInfo, FungibleTokenId, Hash};

fn init_test_nft(owner: Origin) {
	//Create group collection before class
	assert_ok!(NFTModule::create_group(Origin::root(), vec![1], vec![1]));

	assert_ok!(NFTModule::create_class(
		owner.clone(),
		vec![1],
		test_attributes(1),
		COLLECTION_ID,
		TokenType::Transferable,
		CollectionType::Collectable,
		Perbill::from_percent(1u32),
		None
	));

	assert_ok!(NFTModule::mint(owner.clone(), CLASS_ID, vec![1], test_attributes(1), 1));
}

fn test_attributes(x: u8) -> Attributes {
	let mut attr: Attributes = BTreeMap::new();
	attr.insert(vec![x, x + 5], vec![x, x + 10]);
	attr
}

fn test_hash(value: u64) -> Hash {
	Hash::from_low_u64_be(value)
}

fn test_js_root_hash() -> Hash {
	let root_bytes_hash = hex!("b0885a06a169891b562c9ed1b43510422328d6975923d72b661e084e119dde80");
	Hash::from_slice(&root_bytes_hash)
}

/*
	Hash values generated using: https://github.com/OpenZeppelin/merkle-tree
	Test Data:
	2 (BOB) - 10;
	3 (CHARLIE) - 25;
	4 (DONNA) - 50;
	5 (EVA) - 75;
*/
fn test_js_leaf_hashes(who: AccountId) -> Vec<Hash> {
	match who {
		BOB => {
			let charlie_bytes_hash = hex!("d90b5864238131f03c065e80a5e0c04aadb2493984702ef3bb279dcd3cb8ac7d");
			let branch_bytes_hash = hex!("addd535f444323fab87b3350449e85e8ca478541d55a3697caa567d06b45ec3a");
			return vec![
				Hash::from_slice(&charlie_bytes_hash),
				Hash::from_slice(&branch_bytes_hash),
			];
		}
		CHARLIE => {
			let bob_bytes_hash = hex!("64b39d59f54b02b6d862584c58735a0d3ff7c8d1ee46250809f4c244ca13d5ca");
			let branch_bytes_hash = hex!("addd535f444323fab87b3350449e85e8ca478541d55a3697caa567d06b45ec3a");
			return vec![Hash::from_slice(&bob_bytes_hash), Hash::from_slice(&branch_bytes_hash)];
		}
		DONNA => {
			let eva_bytes_hash = hex!("7ecf6a4f9809680533d36217de280ae07964f4c65595308405e2c860bc52d4bf");
			let branch_bytes_hash = hex!("8903505f09ba64010935c4ff9155d37f572578ffdf205dbfdf4381d41a9a83cd");
			return vec![Hash::from_slice(&eva_bytes_hash), Hash::from_slice(&branch_bytes_hash)];
		}
		EVA => {
			let donna_bytes_hash = hex!("77ead2ce9a216ed6ac05f5d8a2c7d12373428794b33d56f65163073769976208");
			let branch_bytes_hash = hex!("8903505f09ba64010935c4ff9155d37f572578ffdf205dbfdf4381d41a9a83cd");
			return vec![
				Hash::from_slice(&donna_bytes_hash),
				Hash::from_slice(&branch_bytes_hash),
			];
		}
		_ => {
			vec![]
		}
	}
}

fn test_claim_hash(who: AccountId, balance: Balance) -> Hash {
	let mut leaf: Vec<u8> = who.encode();
	leaf.extend(balance.encode());

	keccak_256(&keccak_256(&leaf)).into()
}

fn test_claim_nft_hash(who: AccountId, token: (ClassId, TokenId)) -> Hash {
	let mut leaf: Vec<u8> = who.encode();
	leaf.extend(token.0.encode());
	leaf.extend(token.1.encode());
	keccak_256(&keccak_256(&leaf)).into()
}

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
fn create_nft_campaign_works() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;
		init_test_nft(Origin::signed(ALICE));
		init_test_nft(Origin::signed(ALICE));

		assert_ok!(Reward::create_nft_campaign(
			Origin::signed(ALICE),
			ALICE,
			vec![(0u32, 1u64)],
			10,
			10,
			vec![1],
		));

		let campaign_info = CampaignInfo {
			creator: ALICE,
			properties: vec![1],
			cooling_off_duration: 10,
			trie_index: 0,
			end: 10,
			reward: RewardType::NftAssets(vec![(0u32, 1u64)]),
			claimed: RewardType::NftAssets(vec![]),
			cap: RewardType::NftAssets(vec![(0u32, 1u64)]),
		};
		assert_eq!(Reward::campaigns(campaign_id), Some(campaign_info));
		assert_eq!(Balances::free_balance(ALICE), 9993);
		assert_eq!(OrmlNft::tokens(0u32, 1u64).unwrap().data.is_locked, true);

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
fn create_nft_campaign_fails() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;
		init_test_nft(Origin::signed(ALICE));
		init_test_nft(Origin::signed(ALICE));
		assert_noop!(
			Reward::create_nft_campaign(Origin::signed(ALICE), ALICE, vec![(0u32, 1u64)], 2, 10, vec![1],),
			Error::<Runtime>::CampaignDurationBelowMinimum
		);

		assert_noop!(
			Reward::create_nft_campaign(Origin::signed(ALICE), ALICE, vec![], 10, 10, vec![1],),
			Error::<Runtime>::RewardPoolBelowMinimum
		);

		assert_noop!(
			Reward::create_nft_campaign(Origin::signed(ALICE), ALICE, vec![(0u32, 1u64)], 10, 1, vec![1],),
			Error::<Runtime>::CoolingOffPeriodBelowMinimum
		);

		assert_noop!(
			Reward::create_nft_campaign(Origin::signed(ALICE), BOB, vec![(0u32, 1u64)], 10, 10, vec![1],),
			Error::<Runtime>::NoPermissionToUseNftInRewardPool
		);

		NFTModule::set_lock_nft((0u32, 1u64), true);

		assert_noop!(
			Reward::create_nft_campaign(Origin::signed(ALICE), ALICE, vec![(0u32, 1u64)], 10, 10, vec![1],),
			Error::<Runtime>::NoPermissionToUseNftInRewardPool
		);
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

		run_to_block(11);
		assert_noop!(
			Reward::create_campaign(
				Origin::signed(ALICE),
				ALICE,
				10,
				10,
				10,
				vec![1],
				FungibleTokenId::NativeToken(0)
			),
			Error::<Runtime>::CampaignDurationBelowMinimum
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

		assert_ok!(Reward::set_reward(Origin::signed(ALICE), 0, vec![(BOB, 5)]));

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

		let event = mock::Event::Reward(crate::Event::SetReward(campaign_id, vec![(BOB, 5u32.into())]));
		assert_eq!(last_event(), event)
	});
}

#[test]
fn set_reward_root_works() {
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

		assert_ok!(Reward::set_reward_root(Origin::signed(ALICE), 0, 5, test_hash(1u64), vec![(BOB, 0u64)]));

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
		assert_eq!(Reward::campaign_claim_indexes(campaign_id), vec![(BOB, 0u64)]);

		let event = mock::Event::Reward(crate::Event::SetRewardRoot(campaign_id, 5u32.into(), test_hash(1u64)));
		assert_eq!(last_event(), event)
	});
}

#[test]
fn set_nft_reward_works() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;
		assert_ok!(Reward::add_set_reward_origin(Origin::signed(ALICE), ALICE));
		init_test_nft(Origin::signed(ALICE));
		init_test_nft(Origin::signed(ALICE));

		assert_ok!(Reward::create_nft_campaign(
			Origin::signed(ALICE),
			ALICE,
			vec![(0u32, 1u64)],
			10,
			10,
			vec![1],
		));

		let campaign_info = CampaignInfo {
			creator: ALICE,
			properties: vec![1],
			cooling_off_duration: 10,
			trie_index: 0,
			end: 10,
			reward: RewardType::NftAssets(vec![(0u32, 1u64)]),
			claimed: RewardType::NftAssets(vec![]),
			cap: RewardType::NftAssets(vec![(0u32, 1u64)]),
		};
		assert_eq!(Reward::campaigns(campaign_id), Some(campaign_info));
		assert_eq!(Balances::free_balance(ALICE), 9993);
		assert_eq!(OrmlNft::tokens(0u32, 1u64).unwrap().data.is_locked, true);

		assert_ok!(Reward::set_nft_reward(Origin::signed(ALICE), 0, vec![(BOB, 1)], 1));

		let campaign_info_2 = CampaignInfo {
			creator: ALICE,
			properties: vec![1],
			cooling_off_duration: 10,
			trie_index: 0,
			end: 10,
			reward: RewardType::NftAssets(vec![(0u32, 1u64)]),
			claimed: RewardType::NftAssets(vec![]),
			cap: RewardType::NftAssets(vec![]),
		};

		assert_eq!(Reward::campaigns(campaign_id), Some(campaign_info_2));

		let event = mock::Event::Reward(crate::Event::SetNftReward(campaign_id, vec![(BOB, vec![(0u32, 1u64)])]));
		assert_eq!(last_event(), event)
	});
}

#[test]
fn set_nft_reward_root_works() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;
		assert_ok!(Reward::add_set_reward_origin(Origin::signed(ALICE), ALICE));
		init_test_nft(Origin::signed(ALICE));
		init_test_nft(Origin::signed(ALICE));
		init_test_nft(Origin::signed(ALICE));

		assert_ok!(Reward::create_nft_campaign(
			Origin::signed(ALICE),
			ALICE,
			vec![(0u32, 2u64), (0u32, 1u64)],
			10,
			10,
			vec![1],
		));

		let campaign_info = CampaignInfo {
			creator: ALICE,
			properties: vec![1],
			cooling_off_duration: 10,
			trie_index: 0,
			end: 10,
			reward: RewardType::NftAssets(vec![(0u32, 2u64), (0u32, 1u64)]),
			claimed: RewardType::NftAssets(vec![]),
			cap: RewardType::NftAssets(vec![(0u32, 2u64), (0u32, 1u64)]),
		};
		assert_eq!(Reward::campaigns(campaign_id), Some(campaign_info));
		assert_eq!(Balances::free_balance(ALICE), 9990);
		assert_eq!(OrmlNft::tokens(0u32, 1u64).unwrap().data.is_locked, true);

		assert_ok!(Reward::set_nft_reward_root(Origin::signed(ALICE), 0, test_hash(1u64), vec![(BOB, 1u64)]));

		let campaign_info_2 = CampaignInfo {
			creator: ALICE,
			properties: vec![1],
			cooling_off_duration: 10,
			trie_index: 0,
			end: 10,
			reward: RewardType::NftAssets(vec![(0u32, 2u64), (0u32, 1u64)]),
			claimed: RewardType::NftAssets(vec![]),
			cap: RewardType::NftAssets(vec![]),
		};

		assert_eq!(Reward::campaigns(campaign_id), Some(campaign_info_2));
		assert_eq!(Reward::campaign_claim_indexes(campaign_id), vec![(BOB, 1u64)]);

		let event = mock::Event::Reward(crate::Event::SetNftRewardRoot(campaign_id, test_hash(1u64)));
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
			Reward::set_reward(Origin::signed(ALICE), 1, vec![(BOB, 10)]),
			Error::<Runtime>::CampaignIsNotFound
		);

		assert_noop!(
			Reward::set_reward(Origin::signed(ALICE), 0, vec![(BOB, 11)]),
			Error::<Runtime>::RewardExceedCap
		);

		assert_noop!(
			Reward::set_reward(Origin::signed(ALICE), 0, vec![(BOB, 3), (100, 3), (102, 3)]),
			Error::<Runtime>::RewardsListSizeAboveMaximum
		);

		assert_ok!(Reward::set_reward(Origin::signed(ALICE), 0, vec![(BOB, 5)]));

		assert_noop!(
			Reward::set_reward(Origin::signed(ALICE), 0, vec![(ALICE, 6)]),
			Error::<Runtime>::RewardExceedCap
		);

		assert_noop!(
			Reward::set_reward(Origin::signed(3), 0, vec![(BOB, 5)]),
			Error::<Runtime>::InvalidSetRewardOrigin
		);

		run_to_block(2);
		assert_noop!(
			Reward::set_reward(Origin::signed(ALICE), 0, vec![(BOB, 5)]),
			Error::<Runtime>::AccountAlreadyRewarded
		);

		run_to_block(21);

		assert_noop!(
			Reward::set_reward(Origin::signed(ALICE), 0, vec![(BOB, 5)]),
			Error::<Runtime>::CampaignExpired
		);

		init_test_nft(Origin::signed(ALICE));
		init_test_nft(Origin::signed(ALICE));
		assert_ok!(Reward::create_nft_campaign(
			Origin::signed(ALICE),
			ALICE,
			vec![(0u32, 1u64)],
			31,
			10,
			vec![1],
		));

		assert_noop!(
			Reward::set_reward(Origin::signed(ALICE), 1, vec![(BOB, 5)]),
			Error::<Runtime>::InvalidCampaignType
		);
	});
}

#[test]
fn set_reward_root_fails() {
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
			Reward::set_reward_root(Origin::signed(ALICE), 1, 10, test_hash(1u64), vec![(BOB, 0u64)]),
			Error::<Runtime>::CampaignIsNotFound
		);

		assert_noop!(
			Reward::set_reward_root(Origin::signed(ALICE), 0, 11, test_hash(1u64), vec![(BOB, 0u64)]),
			Error::<Runtime>::RewardExceedCap
		);

		assert_noop!(
			Reward::set_reward_root(Origin::signed(ALICE), 0, 5, test_hash(1u64), vec![]),
			Error::<Runtime>::InvalidClaimIndex
		);

		assert_ok!(Reward::set_reward_root(Origin::signed(ALICE), 0, 5, test_hash(1u64), vec![(BOB, 0u64)]));

		assert_noop!(
			Reward::set_reward_root(Origin::signed(ALICE), 0, 5, test_hash(1u64), vec![(BOB, 0u64)]),
			Error::<Runtime>::RewardAlreadySet
		);

		assert_noop!(
			Reward::set_reward_root(Origin::signed(ALICE), 0, 6, test_hash(2u64), vec![(BOB, 0u64)]),
			Error::<Runtime>::RewardExceedCap
		);

		assert_noop!(
			Reward::set_reward_root(Origin::signed(3), 0, 5, test_hash(2u64), vec![(BOB, 0u64)]),
			Error::<Runtime>::InvalidSetRewardOrigin
		);

		run_to_block(21);

		assert_noop!(
			Reward::set_reward_root(Origin::signed(ALICE), 0, 5, test_hash(2u64), vec![(BOB, 0u64)]),
			Error::<Runtime>::CampaignExpired
		);

		init_test_nft(Origin::signed(ALICE));
		init_test_nft(Origin::signed(ALICE));
		assert_ok!(Reward::create_nft_campaign(
			Origin::signed(ALICE),
			ALICE,
			vec![(0u32, 1u64)],
			31,
			10,
			vec![1],
		));

		assert_noop!(
			Reward::set_reward_root(Origin::signed(ALICE), 1, 5, test_hash(1u64), vec![(BOB, 0u64)]),
			Error::<Runtime>::InvalidCampaignType
		);
	});
}

#[test]
fn set_nft_reward_fails() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;
		assert_ok!(Reward::add_set_reward_origin(Origin::signed(ALICE), ALICE));
		init_test_nft(Origin::signed(ALICE));
		init_test_nft(Origin::signed(ALICE));
		init_test_nft(Origin::signed(ALICE));

		assert_ok!(Reward::create_nft_campaign(
			Origin::signed(ALICE),
			ALICE,
			vec![(0u32, 0u64), (0u32, 1u64), (0u32, 2u64)],
			10,
			10,
			vec![1],
		));

		assert_noop!(
			Reward::set_nft_reward(Origin::signed(3), 0, vec![(BOB, 1)], 1),
			Error::<Runtime>::InvalidSetRewardOrigin
		);

		assert_noop!(
			Reward::set_nft_reward(Origin::signed(ALICE), 0, vec![(BOB, 1), (102, 1), (100, 1)], 3),
			Error::<Runtime>::RewardsListSizeAboveMaximum
		);

		assert_noop!(
			Reward::set_nft_reward(Origin::signed(ALICE), 1, vec![(BOB, 1)], 1),
			Error::<Runtime>::CampaignIsNotFound
		);

		assert_noop!(
			Reward::set_nft_reward(Origin::signed(ALICE), 0, vec![(BOB, 4)], 4),
			Error::<Runtime>::RewardExceedCap
		);

		assert_noop!(
			Reward::set_nft_reward(Origin::signed(ALICE), 0, vec![(BOB, 1)], 0),
			Error::<Runtime>::InvalidTotalNftRewardAmountParameter
		);

		assert_ok!(Reward::set_nft_reward(Origin::signed(ALICE), 0, vec![(BOB, 1)], 2));

		assert_noop!(
			Reward::set_nft_reward(Origin::signed(ALICE), 0, vec![(BOB, 1)], 1),
			Error::<Runtime>::AccountAlreadyRewarded
		);

		assert_ok!(Reward::set_nft_reward(Origin::signed(ALICE), 0, vec![(106, 2)], 2));

		assert_noop!(
			Reward::set_nft_reward(Origin::signed(ALICE), 0, vec![(100, 1)], 1),
			Error::<Runtime>::RewardExceedCap
		);

		run_to_block(21);

		assert_noop!(
			Reward::set_nft_reward(Origin::signed(ALICE), 0, vec![(BOB, 1)], 1),
			Error::<Runtime>::CampaignExpired
		);

		assert_ok!(Reward::create_campaign(
			Origin::signed(ALICE),
			ALICE,
			10,
			31,
			10,
			vec![1],
			FungibleTokenId::NativeToken(0),
		));

		assert_noop!(
			Reward::set_nft_reward(Origin::signed(ALICE), 1, vec![(BOB, 1)], 1),
			Error::<Runtime>::InvalidCampaignType
		);
	});
}

#[test]
fn set_nft_reward_root_fails() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;
		assert_ok!(Reward::add_set_reward_origin(Origin::signed(ALICE), ALICE));
		init_test_nft(Origin::signed(ALICE));
		init_test_nft(Origin::signed(ALICE));
		assert_ok!(Reward::create_nft_campaign(
			Origin::signed(ALICE),
			ALICE,
			vec![(0u32, 1u64)],
			10,
			10,
			vec![1],
		));

		assert_noop!(
			Reward::set_nft_reward_root(Origin::signed(ALICE), 1, test_hash(1u64), vec![(BOB, 1u64)]),
			Error::<Runtime>::CampaignIsNotFound
		);

		assert_noop!(
			Reward::set_nft_reward_root(Origin::signed(ALICE), 0, test_hash(1u64), vec![]),
			Error::<Runtime>::InvalidClaimIndex
		);

		assert_ok!(Reward::set_nft_reward_root(Origin::signed(ALICE), 0, test_hash(1u64), vec![(BOB, 1u64)]));

		assert_noop!(
			Reward::set_nft_reward_root(Origin::signed(ALICE), 0, test_hash(1u64), vec![(BOB, 1u64)]),
			Error::<Runtime>::RewardAlreadySet
		);

		init_test_nft(Origin::signed(ALICE));
		assert_ok!(Reward::create_nft_campaign(
			Origin::signed(ALICE),
			ALICE,
			vec![(0u32, 2u64)],
			10,
			10,
			vec![1],
		));

		assert_noop!(
			Reward::set_nft_reward_root(Origin::signed(3), 1, test_hash(2u64), vec![(CHARLIE, 2u64)]),
			Error::<Runtime>::InvalidSetRewardOrigin
		);

		run_to_block(21);

		assert_noop!(
			Reward::set_nft_reward_root(Origin::signed(ALICE), 1, test_hash(2u64), vec![(CHARLIE, 2u64)]),
			Error::<Runtime>::CampaignExpired
		);

		assert_ok!(Reward::create_campaign(
			Origin::signed(ALICE),
			ALICE,
			10,
			31,
			10,
			vec![1],
			FungibleTokenId::NativeToken(0),
		));

		assert_noop!(
			Reward::set_nft_reward_root(Origin::signed(ALICE), 2, test_hash(1u64), vec![(BOB, 1u64)]),
			Error::<Runtime>::InvalidCampaignType
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
		assert_ok!(Reward::set_reward(Origin::signed(ALICE), 0, vec![(BOB, 5)]));

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
fn claim_reward_root_works() {
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
		assert_ok!(Reward::set_reward_root(
			Origin::signed(ALICE),
			0,
			5,
			test_claim_hash(BOB, 5),
			vec![(BOB, 0u64)]
		));

		run_to_block(17);
		//assert_eq!(last_event(), mock::Event::Reward(crate::Event::RewardCampaignEnded(0)));

		assert_ok!(Reward::claim_reward_root(Origin::signed(BOB), 0, 0, 5, vec![]));
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
		assert_eq!(Reward::campaign_claim_indexes(campaign_id), vec![]);

		let event = mock::Event::Reward(crate::Event::RewardClaimed(campaign_id, BOB, 5u32.into()));
		assert_eq!(last_event(), event)
	});
}

#[test]
fn claim_nft_reward_works() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;
		assert_ok!(Reward::add_set_reward_origin(Origin::signed(ALICE), ALICE));
		init_test_nft(Origin::signed(ALICE));
		init_test_nft(Origin::signed(ALICE));

		assert_ok!(Reward::create_nft_campaign(
			Origin::signed(ALICE),
			ALICE,
			vec![(0u32, 1u64)],
			10,
			10,
			vec![1],
		));

		let campaign_info = CampaignInfo {
			creator: ALICE,
			properties: vec![1],
			cooling_off_duration: 10,
			trie_index: 0,
			end: 10,
			reward: RewardType::NftAssets(vec![(0u32, 1u64)]),
			claimed: RewardType::NftAssets(vec![]),
			cap: RewardType::NftAssets(vec![(0u32, 1u64)]),
		};
		assert_eq!(Reward::campaigns(campaign_id), Some(campaign_info));
		assert_eq!(Balances::free_balance(ALICE), 9993);
		assert_eq!(OrmlNft::tokens(0u32, 1u64).unwrap().data.is_locked, true);

		assert_ok!(Reward::set_nft_reward(Origin::signed(ALICE), 0, vec![(BOB, 1)], 1));

		run_to_block(17);

		assert_ok!(Reward::claim_nft_reward(Origin::signed(BOB), 0, 1));

		let campaign_info_after_claim = CampaignInfo {
			creator: ALICE,
			properties: vec![1],
			cooling_off_duration: 10,
			trie_index: 0,
			end: 10,
			reward: RewardType::NftAssets(vec![(0u32, 1u64)]),
			claimed: RewardType::NftAssets(vec![(0u32, 1u64)]),
			cap: RewardType::NftAssets(vec![]),
		};
		assert_eq!(Reward::campaigns(campaign_id), Some(campaign_info_after_claim));
		assert_eq!(OrmlNft::tokens(0u32, 1u64).unwrap().data.is_locked, false);
		assert_eq!(OrmlNft::tokens(0u32, 1u64).unwrap().owner, BOB);

		let event = mock::Event::Reward(crate::Event::NftRewardClaimed(campaign_id, BOB, vec![(0u32, 1u64)]));
		assert_eq!(last_event(), event)
	});
}

#[test]
fn claim_nft_reward_root_works() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;
		assert_ok!(Reward::add_set_reward_origin(Origin::signed(ALICE), ALICE));
		init_test_nft(Origin::signed(ALICE));
		init_test_nft(Origin::signed(ALICE));

		assert_ok!(Reward::create_nft_campaign(
			Origin::signed(ALICE),
			ALICE,
			vec![(0u32, 1u64)],
			10,
			10,
			vec![1],
		));

		let campaign_info = CampaignInfo {
			creator: ALICE,
			properties: vec![1],
			cooling_off_duration: 10,
			trie_index: 0,
			end: 10,
			reward: RewardType::NftAssets(vec![(0u32, 1u64)]),
			claimed: RewardType::NftAssets(vec![]),
			cap: RewardType::NftAssets(vec![(0u32, 1u64)]),
		};
		assert_eq!(Reward::campaigns(campaign_id), Some(campaign_info));
		assert_eq!(Balances::free_balance(ALICE), 9993);
		assert_eq!(OrmlNft::tokens(0u32, 1u64).unwrap().data.is_locked, true);

		assert_ok!(Reward::set_nft_reward_root(
			Origin::signed(ALICE),
			0,
			test_claim_nft_hash(BOB, (0u32, 1u64)),
			vec![(BOB, 0u64)]
		));

		run_to_block(17);

		assert_ok!(Reward::claim_nft_reward_root(
			Origin::signed(BOB),
			0,
			0,
			vec![(0u32, 1u64)],
			vec![]
		));

		let campaign_info_after_claim = CampaignInfo {
			creator: ALICE,
			properties: vec![1],
			cooling_off_duration: 10,
			trie_index: 0,
			end: 10,
			reward: RewardType::NftAssets(vec![(0u32, 1u64)]),
			claimed: RewardType::NftAssets(vec![(0u32, 1u64)]),
			cap: RewardType::NftAssets(vec![]),
		};
		assert_eq!(Reward::campaigns(campaign_id), Some(campaign_info_after_claim));
		assert_eq!(OrmlNft::tokens(0u32, 1u64).unwrap().data.is_locked, false);
		assert_eq!(OrmlNft::tokens(0u32, 1u64).unwrap().owner, BOB);
		assert_eq!(Reward::campaign_claim_indexes(0), vec![(BOB, 0)]);

		let event = mock::Event::Reward(crate::Event::NftRewardClaimed(campaign_id, BOB, vec![(0u32, 1u64)]));
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
		assert_ok!(Reward::set_reward(Origin::signed(ALICE), 0, vec![(BOB, 5)]));

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
		assert_ok!(Reward::set_reward(Origin::signed(ALICE), 0, vec![(BOB, 5)]));

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

		assert_ok!(Reward::claim_reward(Origin::signed(BOB), 0));

		assert_noop!(
			Reward::claim_reward(Origin::signed(BOB), 0),
			Error::<Runtime>::NoRewardFound
		);

		run_to_block(23);

		assert_noop!(
			Reward::claim_reward(Origin::signed(BOB), 0),
			Error::<Runtime>::CampaignExpired
		);

		init_test_nft(Origin::signed(ALICE));
		init_test_nft(Origin::signed(ALICE));

		assert_ok!(Reward::create_nft_campaign(
			Origin::signed(ALICE),
			ALICE,
			vec![(0u32, 1u64)],
			33,
			10,
			vec![1],
		));

		assert_ok!(Reward::set_nft_reward(Origin::signed(ALICE), 1, vec![(BOB, 1)], 1));

		run_to_block(37);

		assert_noop!(
			Reward::claim_reward(Origin::signed(BOB), 1),
			Error::<Runtime>::InvalidCampaignType
		);
	});
}

#[test]
fn claim_reward_root_fails() {
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
		assert_ok!(Reward::set_reward_root(
			Origin::signed(ALICE),
			0,
			5,
			test_claim_hash(BOB, 5), vec![(BOB, 0u64)]
		));

		run_to_block(9);

		assert_noop!(
			Reward::claim_reward_root(Origin::signed(BOB), 0, 0, 5, vec![]),
			Error::<Runtime>::CampaignStillActive
		);

		run_to_block(17);

		assert_noop!(
			Reward::claim_reward_root(Origin::signed(BOB), 1, 0, 5, vec![]),
			Error::<Runtime>::CampaignIsNotFound
		);

		assert_noop!(
			Reward::claim_reward_root(Origin::signed(BOB), 0, 1, 5, vec![]),
			Error::<Runtime>::NoClaimIndexEntry
		);

		assert_ok!(Reward::claim_reward_root(Origin::signed(BOB), 0, 5, vec![]));

		assert_noop!(
			Reward::claim_reward_root(Origin::signed(BOB), 0, 0, 5, vec![]),
			Error::<Runtime>::NoClaimIndexEntry
		);

		//assert_noop!(
		//	Reward::claim_reward_root(Origin::signed(BOB), 0, 5, test_hash(1u64)),
		//	Error::<Runtime>::NoRewardFound
		//);

		run_to_block(23);

		assert_noop!(
			Reward::claim_reward_root(Origin::signed(BOB), 0, 0, 5, vec![]),
			Error::<Runtime>::CampaignExpired
		);

		init_test_nft(Origin::signed(ALICE));
		init_test_nft(Origin::signed(ALICE));

		assert_ok!(Reward::create_nft_campaign(
			Origin::signed(ALICE),
			ALICE,
			vec![(0u32, 1u64)],
			33,
			10,
			vec![1],
		));

		assert_ok!(Reward::set_nft_reward(Origin::signed(ALICE), 1, vec![(BOB, 1)], 1));

		run_to_block(37);

		assert_noop!(
			Reward::claim_reward_root(Origin::signed(BOB), 1, 1, 5, vec![]),
			Error::<Runtime>::InvalidCampaignType
		);

		assert_ok!(Reward::create_campaign(
			Origin::signed(ALICE),
			ALICE,
			20,
			50,
			10,
			vec![1],
			FungibleTokenId::NativeToken(0),
		));

		run_to_block(51);

		assert_noop!(
			Reward::claim_reward_root(Origin::signed(BOB), 2, 2, 5, vec![]),
			Error::<Runtime>::NoClaimIndexEntry
		);

		assert_ok!(Reward::set_reward_root(
			Origin::signed(ALICE),
			2,
			10,
			test_claim_hash(BOB, 10), vec![(BOB, 2u64)]
		));

		assert_noop!(
			Reward::claim_reward_root(Origin::signed(BOB), 2, 2, 5, vec![]),
			Error::<Runtime>::MerkleRootNotRelatedToCampaign
		);

		assert_noop!(
			Reward::claim_reward_root(Origin::signed(BOB), 2, 2, 10, vec![test_hash(2)]),
			Error::<Runtime>::MerkleRootNotRelatedToCampaign
		);

		assert_noop!(
			Reward::claim_reward_root(Origin::signed(ALICE), 2, 2, 10, vec![]),
			Error::<Runtime>::NoClaimIndexEntry
		);
	});
}

#[test]
fn claim_nft_reward_fails() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;
		assert_ok!(Reward::add_set_reward_origin(Origin::signed(ALICE), ALICE));
		init_test_nft(Origin::signed(ALICE));
		init_test_nft(Origin::signed(ALICE));

		assert_ok!(Reward::create_nft_campaign(
			Origin::signed(ALICE),
			ALICE,
			vec![(0u32, 1u64)],
			10,
			10,
			vec![1],
		));

		assert_ok!(Reward::set_nft_reward(Origin::signed(ALICE), 0, vec![(BOB, 1)], 1));

		run_to_block(9);

		assert_noop!(
			Reward::claim_nft_reward(Origin::signed(BOB), 0, 1),
			Error::<Runtime>::CampaignStillActive
		);

		run_to_block(17);

		assert_noop!(
			Reward::claim_nft_reward(Origin::signed(ALICE), 1, 1),
			Error::<Runtime>::CampaignIsNotFound
		);

		assert_noop!(
			Reward::claim_nft_reward(Origin::signed(ALICE), 0, 1),
			Error::<Runtime>::NoRewardFound
		);

		assert_noop!(
			Reward::claim_nft_reward(Origin::signed(BOB), 0, 2),
			Error::<Runtime>::InvalidNftQuantity
		);

		assert_ok!(Reward::claim_nft_reward(Origin::signed(BOB), 0, 1));

		assert_noop!(
			Reward::claim_nft_reward(Origin::signed(BOB), 0, 1),
			Error::<Runtime>::NoRewardFound
		);

		run_to_block(23);

		assert_noop!(
			Reward::claim_nft_reward(Origin::signed(BOB), 0, 1),
			Error::<Runtime>::CampaignExpired
		);

		assert_ok!(Reward::create_campaign(
			Origin::signed(ALICE),
			ALICE,
			10,
			33,
			10,
			vec![1],
			FungibleTokenId::MiningResource(0),
		));

		assert_ok!(Reward::set_reward(Origin::signed(ALICE), 1, vec![(BOB, 5)]));

		run_to_block(37);

		assert_noop!(
			Reward::claim_nft_reward(Origin::signed(BOB), 1, 1),
			Error::<Runtime>::InvalidCampaignType
		);
	});
}

#[test]
fn claim_nft_reward_root_fails() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;
		assert_ok!(Reward::add_set_reward_origin(Origin::signed(ALICE), ALICE));
		init_test_nft(Origin::signed(ALICE));
		init_test_nft(Origin::signed(ALICE));

		assert_ok!(Reward::create_nft_campaign(
			Origin::signed(ALICE),
			ALICE,
			vec![(0u32, 1u64)],
			10,
			10,
			vec![1],
		));

		assert_ok!(Reward::set_nft_reward_root(
			Origin::signed(ALICE),
			0,
			test_claim_nft_hash(BOB, (0u32, 1u64)),
			vec![(BOB, 0u64)]
		));

		run_to_block(9);

		assert_noop!(
			Reward::claim_nft_reward_root(Origin::signed(BOB), 0, 0, vec![(0u32, 1u64)], vec![]),
			Error::<Runtime>::CampaignStillActive
		);

		run_to_block(17);

		assert_noop!(
			Reward::claim_nft_reward_root(Origin::signed(BOB), 1, 0, vec![(0u32, 1u64)], vec![]),
			Error::<Runtime>::CampaignIsNotFound
		);

		assert_noop!(
			Reward::claim_nft_reward_root(Origin::signed(BOB), 0, 0, vec![(0u32, 2u64)], vec![]),
			Error::<Runtime>::MerkleRootNotRelatedToCampaign
		);

		assert_noop!(
			Reward::claim_nft_reward_root(
				Origin::signed(BOB),
				0,
				1,
				vec![(0u32, 1u64)],
				vec![]
			),
			Error::<Runtime>::NoClaimIndexEntry
		);

		assert_ok!(Reward::claim_nft_reward_root(
			Origin::signed(BOB),
			0,
			0,
			vec![(0u32, 1u64)],
			vec![]
		));

		assert_noop!(
			Reward::claim_nft_reward_root(Origin::signed(BOB), 0, 0, vec![(0u32, 1u64)], vec![]),
			Error::<Runtime>::NoRewardFound
		);

		init_test_nft(Origin::signed(ALICE));

		assert_ok!(Reward::create_nft_campaign(
			Origin::signed(ALICE),
			ALICE,
			vec![(0u32, 2u64)],
			27,
			10,
			vec![1],
		));
		assert_ok!(Reward::set_nft_reward_root(
			Origin::signed(ALICE),
			1,
			test_claim_nft_hash(BOB, (0u32, 2u64)),
			vec![(BOB, 1u64)]
		));

		run_to_block(38);

		assert_noop!(
			Reward::claim_nft_reward_root(Origin::signed(BOB), 1, 1, vec![(0u32, 2u64)], vec![]),
			Error::<Runtime>::CampaignExpired
		);

		assert_ok!(Reward::create_campaign(
			Origin::signed(ALICE),
			ALICE,
			10,
			50,
			10,
			vec![1],
			FungibleTokenId::NativeToken(0),
		));

		assert_ok!(Reward::set_reward_root(
			Origin::signed(ALICE),
			2,
			5,
			test_claim_hash(BOB, 5),
			vec![(BOB, 2u64)]
		));

		run_to_block(51);

		assert_noop!(
			Reward::claim_nft_reward_root(Origin::signed(BOB), 2, 2, vec![(0u32, 2u64)], vec![]),
			Error::<Runtime>::InvalidCampaignType
		);

		init_test_nft(Origin::signed(ALICE));

		assert_ok!(Reward::create_nft_campaign(
			Origin::signed(ALICE),
			ALICE,
			vec![(0u32, 3u64)],
			80,
			10,
			vec![1],
		));

		run_to_block(81);

		assert_noop!(
			Reward::claim_nft_reward_root(Origin::signed(BOB), 3, 0, vec![(0u32, 3u64)], vec![]),
			Error::<Runtime>::NoClaimIndexEntry
		);

		assert_ok!(Reward::set_nft_reward_root(
			Origin::signed(ALICE),
			3,
			test_claim_nft_hash(BOB, (0u32, 3u64)),
			vec![(BOB, 3u64)]
		));

		assert_noop!(
			Reward::claim_nft_reward_root(Origin::signed(BOB), 3, 3, vec![(0u32, 2u64)], vec![]),
			Error::<Runtime>::MerkleRootNotRelatedToCampaign
		);

		assert_noop!(
			Reward::claim_nft_reward_root(
				Origin::signed(BOB),
				3,
				3,
				vec![(0u32, 3u64)],
				vec![test_claim_nft_hash(ALICE, (0u32, 3u64))]
			),
			Error::<Runtime>::MerkleRootNotRelatedToCampaign
		);
		assert_noop!(
			Reward::claim_nft_reward_root(Origin::signed(ALICE), 3, 3, vec![(0u32, 3u64)], vec![]),
			Error::<Runtime>::NoClaimIndexEntry
		);
	});
}

#[test]
fn close_nft_campaign_works() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;

		init_test_nft(Origin::signed(ALICE));
		init_test_nft(Origin::signed(ALICE));

		assert_ok!(Reward::create_nft_campaign(
			Origin::signed(ALICE),
			ALICE,
			vec![(0u32, 1u64)],
			10,
			10,
			vec![1],
		));

		assert_eq!(Balances::free_balance(ALICE), 9993);
		assert_eq!(OrmlNft::tokens(0u32, 1u64).unwrap().data.is_locked, true);

		run_to_block(100);

		assert_ok!(Reward::close_nft_campaign(Origin::signed(ALICE), 0, 1));

		assert_eq!(Balances::free_balance(ALICE), 9994);
		assert_eq!(OrmlNft::tokens(0u32, 1u64).unwrap().data.is_locked, false);

		assert_eq!(Campaigns::<Runtime>::get(campaign_id), None);

		let event = mock::Event::Reward(crate::Event::RewardCampaignClosed(campaign_id));
		assert_eq!(last_event(), event)
	});
}

#[test]
fn close_nft_campaign_with_merkle_root_works() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;

		init_test_nft(Origin::signed(ALICE));
		init_test_nft(Origin::signed(ALICE));
		assert_ok!(Reward::add_set_reward_origin(Origin::signed(ALICE), ALICE));

		assert_ok!(Reward::create_nft_campaign(
			Origin::signed(ALICE),
			ALICE,
			vec![(0u32, 1u64)],
			10,
			10,
			vec![1],
		));

		assert_eq!(Balances::free_balance(ALICE), 9993);
		assert_eq!(OrmlNft::tokens(0u32, 1u64).unwrap().data.is_locked, true);
		assert_ok!(Reward::set_nft_reward_root(Origin::signed(ALICE), 0, test_hash(1u64), vec![(BOB, 0u64)]));
		assert_eq!(CampaignMerkleRoots::<Runtime>::get(campaign_id), vec![test_hash(1u64)]);

		run_to_block(100);

		assert_ok!(Reward::close_nft_campaign(Origin::signed(ALICE), 0, 1));

		assert_eq!(Balances::free_balance(ALICE), 9994);
		assert_eq!(OrmlNft::tokens(0u32, 1u64).unwrap().data.is_locked, false);

		assert_eq!(Campaigns::<Runtime>::get(campaign_id), None);
		assert_eq!(CampaignMerkleRoots::<Runtime>::get(campaign_id), vec![]);
		assert_eq!(CampaignClaimIndexes::<Runtime>::get(campaign_id), vec![]);

		let event = mock::Event::Reward(crate::Event::RewardCampaignRootClosed(campaign_id));
		assert_eq!(last_event(), event)
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

		assert_ok!(Reward::close_campaign(Origin::signed(BOB), 0, 0));

		assert_eq!(Balances::free_balance(ALICE), 9989);
		assert_eq!(Balances::free_balance(BOB), 20011);

		assert_eq!(Campaigns::<Runtime>::get(campaign_id), None);

		let event = mock::Event::Reward(crate::Event::RewardCampaignClosed(campaign_id));
		assert_eq!(last_event(), event)
	});
}

#[test]
fn close_campaign_using_merkle_root_works() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;
		assert_ok!(Reward::add_set_reward_origin(Origin::signed(ALICE), ALICE));
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
		assert_ok!(Reward::set_reward_root(
			Origin::signed(ALICE),
			0,
			5,
			test_claim_hash(BOB, 5),
			vec![(BOB, 0u64)]
		));

		assert_ok!(Reward::set_reward_root(
			Origin::signed(ALICE),
			0,
			4,
			test_claim_hash(3, 3),
			vec![(3, 1u64)]
		));

		run_to_block(11);
		assert_ok!(Reward::claim_reward_root(Origin::signed(BOB), 0, 0, 5, vec![]));
		run_to_block(100);

		assert_ok!(Reward::close_campaign(Origin::signed(BOB), 0, 2));

		assert_eq!(Balances::free_balance(ALICE), 9989);
		assert_eq!(Balances::free_balance(BOB), 20011);

		assert_eq!(Campaigns::<Runtime>::get(campaign_id), None);
		assert_eq!(CampaignMerkleRoots::<Runtime>::get(campaign_id), vec![]);
		assert_eq!(CampaignClaimedAccounts::<Runtime>::get(campaign_id), vec![]);
		assert_eq!(CampaignClaimIndexes::<Runtime>::get(campaign_id), vec![]);

		let event = mock::Event::Reward(crate::Event::RewardCampaignRootClosed(campaign_id));
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

		assert_ok!(Reward::close_campaign(Origin::signed(BOB), 0, 0));

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
			Reward::close_campaign(Origin::signed(ALICE), 1, 0),
			Error::<Runtime>::CampaignIsNotFound
		);

		assert_noop!(
			Reward::close_campaign(Origin::signed(ALICE), 0, 0),
			Error::<Runtime>::NotCampaignCreator
		);

		assert_noop!(
			Reward::close_campaign(Origin::signed(BOB), 0, 0),
			Error::<Runtime>::CampaignStillActive
		);

		init_test_nft(Origin::signed(ALICE));
		init_test_nft(Origin::signed(ALICE));
		init_test_nft(Origin::signed(ALICE));

		assert_ok!(Reward::create_nft_campaign(
			Origin::signed(ALICE),
			ALICE,
			vec![(0u32, 1u64), (0u32, 2u64)],
			27,
			10,
			vec![1],
		));

		run_to_block(117);

		assert_noop!(
			Reward::close_campaign(Origin::signed(ALICE), 1, 0),
			Error::<Runtime>::InvalidCampaignType
		);
	});
}

#[test]
fn close_campaign_using_merkle_root_fails() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;
		assert_ok!(Reward::add_set_reward_origin(Origin::signed(ALICE), ALICE));
		assert_ok!(Reward::create_campaign(
			Origin::signed(ALICE),
			BOB,
			10,
			10,
			10,
			vec![1],
			FungibleTokenId::NativeToken(0)
		));

		assert_ok!(Reward::set_reward_root(
			Origin::signed(ALICE),
			0,
			5,
			test_claim_hash(BOB, 5),
			vec![(BOB, 0u64)]
		));

		assert_ok!(Reward::set_reward_root(
			Origin::signed(ALICE),
			0,
			4,
			test_claim_hash(3, 3),
			vec![(BOB, 0u64)]
		));

		run_to_block(17);

		assert_noop!(
			Reward::close_campaign(Origin::signed(ALICE), 1, 2),
			Error::<Runtime>::CampaignIsNotFound
		);

		assert_noop!(
			Reward::close_campaign(Origin::signed(ALICE), 0, 2),
			Error::<Runtime>::NotCampaignCreator
		);

		assert_noop!(
			Reward::close_campaign(Origin::signed(BOB), 0, 2),
			Error::<Runtime>::CampaignStillActive
		);

		run_to_block(100);

		assert_noop!(
			Reward::close_campaign(Origin::signed(BOB), 0, 1),
			Error::<Runtime>::InvalidMerkleRootsQuantity
		);

		assert_ok!(Reward::close_campaign(Origin::signed(BOB), 0, 2));

		assert_noop!(
			Reward::close_campaign(Origin::signed(BOB), 0, 2),
			Error::<Runtime>::CampaignIsNotFound
		);

		init_test_nft(Origin::signed(ALICE));
		init_test_nft(Origin::signed(ALICE));
		init_test_nft(Origin::signed(ALICE));

		assert_ok!(Reward::create_nft_campaign(
			Origin::signed(ALICE),
			ALICE,
			vec![(0u32, 1u64), (0u32, 2u64)],
			110,
			10,
			vec![1],
		));

		run_to_block(150);

		assert_noop!(
			Reward::close_campaign(Origin::signed(ALICE), 1, 1),
			Error::<Runtime>::InvalidCampaignType
		);
	});
}

#[test]
fn close_nft_campaign_fails() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;

		init_test_nft(Origin::signed(ALICE));
		init_test_nft(Origin::signed(ALICE));
		init_test_nft(Origin::signed(ALICE));

		assert_ok!(Reward::create_nft_campaign(
			Origin::signed(ALICE),
			ALICE,
			vec![(0u32, 1u64), (0u32, 2u64)],
			10,
			10,
			vec![1],
		));

		run_to_block(17);

		assert_noop!(
			Reward::close_nft_campaign(Origin::signed(ALICE), 1, 2),
			Error::<Runtime>::CampaignIsNotFound
		);

		assert_noop!(
			Reward::close_nft_campaign(Origin::signed(BOB), 0, 2),
			Error::<Runtime>::NotCampaignCreator
		);

		assert_noop!(
			Reward::close_nft_campaign(Origin::signed(ALICE), 0, 2),
			Error::<Runtime>::CampaignStillActive
		);

		run_to_block(100);

		assert_noop!(
			Reward::close_nft_campaign(Origin::signed(ALICE), 0, 1),
			Error::<Runtime>::InvalidNftQuantity
		);

		assert_ok!(Reward::create_campaign(
			Origin::signed(ALICE),
			BOB,
			10,
			120,
			10,
			vec![1],
			FungibleTokenId::NativeToken(0)
		));
		run_to_block(200);

		assert_noop!(
			Reward::close_nft_campaign(Origin::signed(BOB), 1, 2),
			Error::<Runtime>::InvalidCampaignType
		);
	});
}

#[test]
fn cancel_nft_campaign_works() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;

		init_test_nft(Origin::signed(ALICE));
		init_test_nft(Origin::signed(ALICE));

		assert_ok!(Reward::create_nft_campaign(
			Origin::signed(ALICE),
			ALICE,
			vec![(0u32, 1u64)],
			10,
			10,
			vec![1],
		));

		assert_eq!(Balances::free_balance(ALICE), 9993);
		assert_eq!(OrmlNft::tokens(0u32, 1u64).unwrap().data.is_locked, true);

		run_to_block(5);

		assert_ok!(Reward::cancel_nft_campaign(Origin::signed(ALICE), 0, 1));

		assert_eq!(Balances::free_balance(ALICE), 9994);
		assert_eq!(OrmlNft::tokens(0u32, 1u64).unwrap().data.is_locked, false);

		assert_eq!(Campaigns::<Runtime>::get(campaign_id), None);

		let event = mock::Event::Reward(crate::Event::RewardCampaignCanceled(campaign_id));
		assert_eq!(last_event(), event)
	});
}

#[test]
fn cancel_mekrle_root_nft_campaign_works() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;

		init_test_nft(Origin::signed(ALICE));
		init_test_nft(Origin::signed(ALICE));

		assert_ok!(Reward::add_set_reward_origin(Origin::signed(ALICE), ALICE));

		assert_ok!(Reward::create_nft_campaign(
			Origin::signed(ALICE),
			ALICE,
			vec![(0u32, 1u64)],
			10,
			10,
			vec![1],
		));

		assert_eq!(Balances::free_balance(ALICE), 9993);
		assert_eq!(OrmlNft::tokens(0u32, 1u64).unwrap().data.is_locked, true);

		assert_ok!(Reward::set_nft_reward_root(Origin::signed(ALICE), 0, test_hash(1u64), vec![(BOB, 1u64)]));

		run_to_block(5);

		assert_ok!(Reward::cancel_nft_campaign(Origin::signed(ALICE), 0, 1));

		assert_eq!(Balances::free_balance(ALICE), 9994);
		assert_eq!(OrmlNft::tokens(0u32, 1u64).unwrap().data.is_locked, false);

		assert_eq!(Campaigns::<Runtime>::get(campaign_id), None);
		assert_eq!(CampaignClaimIndexes::<Runtime>::get(campaign_id), vec![]);

		let event = mock::Event::Reward(crate::Event::RewardCampaignCanceled(campaign_id));
		assert_eq!(last_event(), event)
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
fn cancel_mekrle_root_campaign_works() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;
		assert_ok!(Reward::add_set_reward_origin(Origin::signed(ALICE), ALICE));
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

		assert_ok!(Reward::set_reward_root(
			Origin::signed(ALICE),
			0,
			5,
			test_hash(1u64), // get root hash value from JS
			vec![(BOB, 0u64)] 
		));

		run_to_block(5);
		

		assert_ok!(Reward::cancel_campaign(Origin::signed(ALICE), 0));

		assert_eq!(Balances::free_balance(ALICE), 9989);
		assert_eq!(Balances::free_balance(BOB), 20011);

		assert_eq!(Campaigns::<Runtime>::get(campaign_id), None);
		assert_eq!(CampaignClaimIndexes::<Runtime>::get(campaign_id), vec![]);

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
fn cancel_nft_campaign_fails() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;
		init_test_nft(Origin::signed(ALICE));
		init_test_nft(Origin::signed(ALICE));
		init_test_nft(Origin::signed(ALICE));

		assert_ok!(Reward::create_nft_campaign(
			Origin::signed(ALICE),
			ALICE,
			vec![(0u32, 1u64), (0u32, 2u64)],
			10,
			10,
			vec![1],
		));

		assert_noop!(
			Reward::cancel_nft_campaign(Origin::signed(ALICE), 1, 2),
			Error::<Runtime>::CampaignIsNotFound
		);

		assert_noop!(
			Reward::cancel_nft_campaign(Origin::signed(ALICE), 0, 1),
			Error::<Runtime>::InvalidNftQuantity
		);

		run_to_block(11);

		assert_noop!(
			Reward::cancel_nft_campaign(Origin::signed(ALICE), 0, 2),
			Error::<Runtime>::CampaignEnded
		);

		assert_ok!(Reward::create_campaign(
			Origin::signed(ALICE),
			BOB,
			10,
			21,
			10,
			vec![1],
			FungibleTokenId::NativeToken(0)
		));

		assert_noop!(
			Reward::cancel_nft_campaign(Origin::signed(ALICE), 1, 2),
			Error::<Runtime>::InvalidCampaignType
		);
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

		init_test_nft(Origin::signed(ALICE));
		init_test_nft(Origin::signed(ALICE));
		init_test_nft(Origin::signed(ALICE));

		assert_ok!(Reward::create_nft_campaign(
			Origin::signed(ALICE),
			ALICE,
			vec![(0u32, 1u64), (0u32, 2u64)],
			21,
			10,
			vec![1],
		));

		assert_noop!(
			Reward::cancel_campaign(Origin::signed(ALICE), 1),
			Error::<Runtime>::InvalidCampaignType
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

#[test]
fn js_encoded_data_matches_blockchain_encoded_data() {
	ExtBuilder::default().build().execute_with(|| {
		let balance: Balance = 10;
		let mut leaf: Vec<u8> = BOB.encode();
		leaf.extend(balance.encode());

		assert_eq!(
			leaf,
			vec![2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
		); // SCALE encoding for [2u128, 10u128]
	});
}

#[test]
fn js_generated_leafs_match_blockchain_generated_leafs() {
	ExtBuilder::default().build().execute_with(|| {
		let bob_hash = Hash::from_slice(&hex!(
			"64b39d59f54b02b6d862584c58735a0d3ff7c8d1ee46250809f4c244ca13d5ca"
		));
		assert_eq!(test_claim_hash(BOB, 10), bob_hash);

		let charlie_hash = Hash::from_slice(&hex!(
			"d90b5864238131f03c065e80a5e0c04aadb2493984702ef3bb279dcd3cb8ac7d"
		));
		assert_eq!(test_claim_hash(CHARLIE, 25), charlie_hash);

		let donna_hash = Hash::from_slice(&hex!(
			"77ead2ce9a216ed6ac05f5d8a2c7d12373428794b33d56f65163073769976208"
		));
		assert_eq!(test_claim_hash(DONNA, 50), donna_hash);

		let eva_hash = Hash::from_slice(&hex!(
			"7ecf6a4f9809680533d36217de280ae07964f4c65595308405e2c860bc52d4bf"
		));
		assert_eq!(test_claim_hash(EVA, 75), eva_hash);
	});
}

#[test]
fn merkle_proof_based_cmapaing_works_with_js_generated_root() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;
		assert_ok!(Reward::add_set_reward_origin(Origin::signed(ALICE), ALICE));

		assert_ok!(Reward::create_campaign(
			Origin::signed(ALICE),
			ALICE,
			160,
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
			reward: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 160),
			claimed: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 0),
			cap: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 160),
		};
		assert_eq!(Reward::campaigns(campaign_id), Some(campaign_info));
		assert_ok!(Reward::set_reward_root(
			Origin::signed(ALICE),
			0,
			160,
			test_js_root_hash(), // get root hash value from JS
			vec![(BOB, 2u64), (CHARLIE, 3u64), (DONNA, 4u64), (EVA, 5u64)] 
		));

		run_to_block(17);

		// 10 reward winner
		assert_ok!(Reward::claim_reward_root(
			Origin::signed(BOB),
			0,
			2,
			10,
			test_js_leaf_hashes(BOB)
		));
		assert_eq!(Balances::free_balance(BOB), 20010);

		let campaign_info_after_claim_10 = CampaignInfo {
			creator: ALICE,
			properties: vec![1],
			reward: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 160),
			claimed: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 10),
			end: 10,
			cap: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 0),
			cooling_off_duration: 10,
			trie_index: 0,
		};

		assert_eq!(Reward::campaigns(campaign_id), Some(campaign_info_after_claim_10));
		assert_eq!(Reward::campaign_claim_indexes(campaign_id), vec![(CHARLIE, 3u64), (DONNA, 4u64), (EVA, 5u64)]);

		// 25 reward winner
		assert_ok!(Reward::claim_reward_root(
			Origin::signed(CHARLIE),
			0,
			3,
			25,
			test_js_leaf_hashes(CHARLIE)
		));
		assert_eq!(Balances::free_balance(CHARLIE), 2025);

		let campaign_info_after_claim_25 = CampaignInfo {
			creator: ALICE,
			properties: vec![1],
			reward: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 160),
			claimed: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 35),
			end: 10,
			cap: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 0),
			cooling_off_duration: 10,
			trie_index: 0,
		};

		assert_eq!(Reward::campaigns(campaign_id), Some(campaign_info_after_claim_25));
		assert_eq!(Reward::campaign_claim_indexes(campaign_id), vec![(DONNA, 4u64), (EVA, 5u64)]);

		// 50 reward winner
		assert_ok!(Reward::claim_reward_root(
			Origin::signed(DONNA),
			0,
			4,
			50,
			test_js_leaf_hashes(DONNA)
		));
		assert_eq!(Balances::free_balance(DONNA), 100050);

		let campaign_info_after_claim_50 = CampaignInfo {
			creator: ALICE,
			properties: vec![1],
			reward: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 160),
			claimed: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 85),
			end: 10,
			cap: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 0),
			cooling_off_duration: 10,
			trie_index: 0,
		};

		assert_eq!(Reward::campaigns(campaign_id), Some(campaign_info_after_claim_50));
		assert_eq!(Reward::campaign_claim_indexes(campaign_id), vec![(EVA, 5u64)]);

		// 75 reward winner
		assert_ok!(Reward::claim_reward_root(
			Origin::signed(EVA),
			0,
			5,
			75,
			test_js_leaf_hashes(EVA)
		));
		assert_eq!(Balances::free_balance(EVA), 1075);

		let campaign_info_after_claim_75 = CampaignInfo {
			creator: ALICE,
			properties: vec![1],
			reward: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 160),
			claimed: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 160),
			end: 10,
			cap: RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), 0),
			cooling_off_duration: 10,
			trie_index: 0,
		};

		assert_eq!(Reward::campaigns(campaign_id), Some(campaign_info_after_claim_75));
		assert_eq!(Reward::campaign_claim_indexes(campaign_id), vec![]);

	});
}
/* 
#[test]
fn account_conversion_works() {
	ExtBuilder::default().build().execute_with(|| {
		let account_id_32 = AccountId32::new([
			214, 117,  31,  71,  18, 107, 145, 170,
			200, 244,  39, 183,  98, 253,  41, 170,
			103, 204, 185, 127, 137,  60, 136, 184,
			184, 217,  82,  68,  61, 228,  63,  47
		]);
		assert_eq!(account_id_32.to_string(), "5GuttyuDTejF1p6fzv1ffzxNKEnTWWJ4jCMwqcFfiwMj1bYh");
		assert_eq!(account_id_32.to_ss58check(), "5GuttyuDTejF1p6fzv1ffzxNKEnTWWJ4jCMwqcFfiwMj1bYh");
	});
}
*/

