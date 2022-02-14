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

#![cfg(test)]

use frame_support::{assert_err, assert_noop, assert_ok};
use sp_runtime::traits::BadOrigin;

use super::*;
use mock::{Event, *};

use auction_manager::ListingLevel;
use pallet_nft::{Attributes, CollectionType, TokenType};

fn init_test_nft(owner: Origin, classId: ClassId) {
	//Create group collection before class
	assert_ok!(NFTModule::<Runtime>::create_group(Origin::root(), vec![1], vec![1]));

	assert_ok!(NFTModule::<Runtime>::create_class(
		owner.clone(),
		vec![1],
		test_attributes(1),
		COLLECTION_ID,
		TokenType::Transferable,
		CollectionType::Collectable,
	));

	assert_ok!(NFTModule::<Runtime>::mint(
		owner.clone(),
		classId,
		vec![1],
		test_attributes(1),
		1
	));
}

fn test_attributes(x: u8) -> Attributes {
	let mut attr: Attributes = BTreeMap::new();
	attr.insert(vec![x, x + 5], vec![x, x + 10]);
	attr
}

fn sub_account(nft_id: AssetId) -> AccountId {
	<Runtime as Config>::EconomyTreasury::get().into_sub_account(nft_id)
}

#[test]
fn authorize_power_generator_collection_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EconomyModule::authorize_power_generator_collection(
			Origin::root(),
			DISTRIBUTOR_CLASS_ID
		));

		let event = Event::Economy(crate::Event::PowerGeneratorCollectionAuthorized(DISTRIBUTOR_CLASS_ID));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn authorize_power_generator_collection_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EconomyModule::authorize_power_generator_collection(Origin::signed(BOB), DISTRIBUTOR_CLASS_ID),
			BadOrigin
		);
	});
}

#[test]
fn authorize_power_distributor_collection_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EconomyModule::authorize_power_distributor_collection(
			Origin::root(),
			DISTRIBUTOR_CLASS_ID
		));

		let event = Event::Economy(crate::Event::PowerDistributorCollectionAuthorized(DISTRIBUTOR_CLASS_ID));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn authorize_power_distributor_collection_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EconomyModule::authorize_power_distributor_collection(Origin::signed(BOB), DISTRIBUTOR_CLASS_ID),
			BadOrigin
		);
	});
}

#[test]
// Creating auction should work
fn buy_power_by_user_should_fail_nft_does_not_exist() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);

		assert_noop!(
			EconomyModule::buy_power_by_user(Origin::signed(ALICE), 100u64.into(), 0,),
			Error::<Runtime>::NFTAssetDoesNotExist
		);
	});
}

#[test]
fn buy_power_by_user_should_fail_NFT_not_authorized() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		init_test_nft(origin.clone(), DISTRIBUTOR_CLASS_ID);

		assert_noop!(
			EconomyModule::buy_power_by_user(origin, 100u64.into(), DISTRIBUTOR_NFT_ASSET_ID,),
			Error::<Runtime>::NoPermissionToBuyMiningPower
		);
	});
}

#[test]
fn buy_power_by_user_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		init_test_nft(origin.clone(), DISTRIBUTOR_CLASS_ID);

		assert_ok!(EconomyModule::authorize_power_distributor_collection(
			Origin::root(),
			DISTRIBUTOR_CLASS_ID
		));

		assert_ok!(EconomyModule::buy_power_by_user(
			origin,
			USER_BUY_POWER_AMOUNT,
			DISTRIBUTOR_NFT_ASSET_ID,
		));

		let event = Event::Economy(crate::Event::BuyPowerOrderByUserHasAddedToQueue(
			ALICE,
			USER_BUY_POWER_AMOUNT,
			DISTRIBUTOR_NFT_ASSET_ID,
		));
		assert_eq!(last_event(), event);

		assert_eq!(
			EconomyModule::get_buy_power_by_user_request_queue(DISTRIBUTOR_NFT_ASSET_ID),
			Some(vec![(ALICE, USER_BUY_POWER_AMOUNT)])
		);
	});
}

#[test]
// Creating auction should work
fn buy_power_by_distributor_should_fail_nft_does_not_exist() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);

		assert_noop!(
			EconomyModule::buy_power_by_distributor(
				Origin::signed(ALICE),
				GENERATOR_NFT_ASSET_ID,
				DISTRIBUTOR_NFT_ASSET_ID,
				GENERATE_POWER_AMOUNT,
			),
			Error::<Runtime>::NFTAssetDoesNotExist
		);
	});
}

#[test]
fn buy_power_by_distributor_should_fail_NFT_not_authorized() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		init_test_nft(origin.clone(), DISTRIBUTOR_CLASS_ID);
		init_test_nft(origin.clone(), GENERATOR_CLASS_ID);

		assert_noop!(
			EconomyModule::buy_power_by_distributor(
				origin,
				GENERATOR_NFT_ASSET_ID,
				DISTRIBUTOR_NFT_ASSET_ID,
				GENERATE_POWER_AMOUNT,
			),
			Error::<Runtime>::NoPermissionToBuyMiningPower
		);
	});
}

#[test]
fn buy_power_by_distributor_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		init_test_nft(origin.clone(), DISTRIBUTOR_CLASS_ID);
		init_test_nft(origin.clone(), GENERATOR_CLASS_ID);

		assert_ok!(EconomyModule::authorize_power_generator_collection(
			Origin::root(),
			GENERATOR_CLASS_ID
		));

		assert_ok!(EconomyModule::buy_power_by_distributor(
			origin,
			GENERATOR_NFT_ASSET_ID,
			DISTRIBUTOR_NFT_ASSET_ID,
			GENERATE_POWER_AMOUNT,
		));

		let distributor_account_id = sub_account(DISTRIBUTOR_NFT_ASSET_ID);

		let event = Event::Economy(crate::Event::BuyPowerOrderByDistributorHasAddedToQueue(
			distributor_account_id,
			GENERATE_POWER_AMOUNT,
			GENERATOR_NFT_ASSET_ID,
		));
		assert_eq!(last_event(), event);

		assert_eq!(
			EconomyModule::get_buy_power_by_distributor_request_queue(GENERATOR_NFT_ASSET_ID),
			Some(vec![(distributor_account_id, GENERATE_POWER_AMOUNT)])
		);
	});
}
