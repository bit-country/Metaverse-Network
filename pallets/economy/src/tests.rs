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
use orml_nft::Tokens;
use sp_runtime::traits::BadOrigin;

use super::*;
use mock::{Event, *};

use auction_manager::ListingLevel;
use pallet_nft::{Attributes, CollectionType, TokenType};
use primitives::GroupCollectionId;

fn init_test_nft(owner: Origin, collection_id: GroupCollectionId, classId: ClassId) {
	//Create group collection before class
	assert_ok!(NFTModule::<Runtime>::create_group(Origin::root(), vec![1], vec![1]));

	assert_ok!(NFTModule::<Runtime>::create_class(
		owner.clone(),
		vec![1],
		test_attributes(1),
		collection_id,
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

fn get_mining_currency() -> FungibleTokenId {
	<Runtime as Config>::MiningCurrencyId::get()
}

#[test]
fn authorize_power_generator_collection_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EconomyModule::authorize_power_generator_collection(
			Origin::root(),
			GENERATOR_COLLECTION_ID,
			GENERATOR_CLASS_ID
		));

		let event = Event::Economy(crate::Event::PowerGeneratorCollectionAuthorized(
			GENERATOR_COLLECTION_ID,
			GENERATOR_CLASS_ID,
		));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn authorize_power_generator_collection_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EconomyModule::authorize_power_generator_collection(
				Origin::signed(BOB),
				GENERATOR_COLLECTION_ID,
				GENERATOR_CLASS_ID
			),
			BadOrigin
		);
	});
}

#[test]
fn authorize_power_distributor_collection_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EconomyModule::authorize_power_distributor_collection(
			Origin::root(),
			DISTRIBUTOR_COLLECTION_ID,
			DISTRIBUTOR_CLASS_ID
		));

		let event = Event::Economy(crate::Event::PowerDistributorCollectionAuthorized(
			DISTRIBUTOR_COLLECTION_ID,
			DISTRIBUTOR_CLASS_ID,
		));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn authorize_power_distributor_collection_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EconomyModule::authorize_power_distributor_collection(
				Origin::signed(BOB),
				DISTRIBUTOR_COLLECTION_ID,
				DISTRIBUTOR_CLASS_ID
			),
			BadOrigin
		);
	});
}

#[test]
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
		init_test_nft(origin.clone(), DISTRIBUTOR_COLLECTION_ID, DISTRIBUTOR_CLASS_ID);

		assert_noop!(
			EconomyModule::buy_power_by_user(origin, 100u64.into(), DISTRIBUTOR_NFT_ASSET_ID,),
			Error::<Runtime>::NoPermissionToBuyMiningPower
		);
	});
}

#[test]
fn buy_power_by_user_should_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, get_mining_currency(), ALICE_MINING_BALANCE.into())])
		.build()
		.execute_with(|| {
			let origin = Origin::signed(ALICE);
			init_test_nft(origin.clone(), DISTRIBUTOR_COLLECTION_ID, DISTRIBUTOR_CLASS_ID);

			assert_ok!(EconomyModule::authorize_power_distributor_collection(
				Origin::root(),
				DISTRIBUTOR_COLLECTION_ID,
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

			// Check reserved balance
			let mining_currency_id = get_mining_currency();
			let bit_amount = USER_BUY_POWER_AMOUNT + 100;
			assert_eq!(
				OrmlTokens::reserved_balance(mining_currency_id, &ALICE),
				bit_amount.into()
			);

			let remaining_amount: u64 = ALICE_MINING_BALANCE - USER_BUY_POWER_AMOUNT;
			assert_eq!(
				OrmlTokens::free_balance(mining_currency_id, &ALICE),
				(remaining_amount - 100).into()
			);
		});
}

#[test]
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
		init_test_nft(origin.clone(), DISTRIBUTOR_COLLECTION_ID, DISTRIBUTOR_CLASS_ID);
		init_test_nft(origin.clone(), GENERATOR_COLLECTION_ID, GENERATOR_CLASS_ID);

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
	ExtBuilder::default()
		.balances(vec![(
			sub_account(DISTRIBUTOR_NFT_ASSET_ID),
			get_mining_currency(),
			DISTRIBUTOR_MINING_BALANCE.into(),
		)])
		.build()
		.execute_with(|| {
			let origin = Origin::signed(ALICE);
			init_test_nft(origin.clone(), DISTRIBUTOR_COLLECTION_ID, DISTRIBUTOR_CLASS_ID);
			init_test_nft(origin.clone(), GENERATOR_COLLECTION_ID, GENERATOR_CLASS_ID);

			assert_ok!(EconomyModule::authorize_power_generator_collection(
				Origin::root(),
				GENERATOR_COLLECTION_ID,
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

			// Check reserved balance
			let mining_currency_id = get_mining_currency();
			let bit_amount = GENERATE_POWER_AMOUNT + 100;
			assert_eq!(
				OrmlTokens::reserved_balance(mining_currency_id, &distributor_account_id),
				bit_amount.into()
			);

			let remaining_amount: u64 = DISTRIBUTOR_MINING_BALANCE - GENERATE_POWER_AMOUNT;
			assert_eq!(
				OrmlTokens::free_balance(mining_currency_id, &distributor_account_id),
				(remaining_amount - 100).into()
			);
		});
}

#[test]
fn execute_buy_power_order_should_fail_nft_does_not_exist() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		init_test_nft(origin.clone(), DISTRIBUTOR_COLLECTION_ID, DISTRIBUTOR_CLASS_ID);

		assert_ok!(EconomyModule::authorize_power_distributor_collection(
			Origin::root(),
			DISTRIBUTOR_COLLECTION_ID,
			DISTRIBUTOR_CLASS_ID
		));

		assert_noop!(
			EconomyModule::execute_buy_power_order(origin, NFT_ASSET_ID_NOT_EXIST, ALICE),
			Error::<Runtime>::NFTAssetDoesNotExist
		);
	});
}

#[test]
fn execute_buy_power_order_should_fail_distributor_does_not_exist() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		init_test_nft(origin.clone(), DISTRIBUTOR_COLLECTION_ID, DISTRIBUTOR_CLASS_ID);

		assert_ok!(EconomyModule::authorize_power_distributor_collection(
			Origin::root(),
			DISTRIBUTOR_COLLECTION_ID,
			DISTRIBUTOR_CLASS_ID
		));

		assert_noop!(
			EconomyModule::execute_buy_power_order(origin, DISTRIBUTOR_NFT_ASSET_ID, ALICE),
			Error::<Runtime>::DistributorNftDoesNotExist
		);
	});
}

#[test]
fn execute_buy_power_order_should_fail_account_does_not_exist() {
	ExtBuilder::default()
		.balances(vec![(ALICE, get_mining_currency(), ALICE_MINING_BALANCE.into())])
		.build()
		.execute_with(|| {
			let origin = Origin::signed(ALICE);
			init_test_nft(origin.clone(), DISTRIBUTOR_COLLECTION_ID, DISTRIBUTOR_CLASS_ID);

			assert_ok!(EconomyModule::authorize_power_distributor_collection(
				Origin::root(),
				DISTRIBUTOR_COLLECTION_ID,
				DISTRIBUTOR_CLASS_ID
			));

			assert_ok!(EconomyModule::buy_power_by_user(
				origin.clone(),
				USER_BUY_POWER_AMOUNT,
				DISTRIBUTOR_NFT_ASSET_ID,
			));

			assert_noop!(
				EconomyModule::execute_buy_power_order(origin, DISTRIBUTOR_NFT_ASSET_ID, BOB),
				Error::<Runtime>::AccountIdDoesNotExistInBuyOrderQueue
			);
		});
}

#[test]
fn execute_buy_power_order_should_fail_insufficient_balance() {
	ExtBuilder::default()
		.balances(vec![(ALICE, get_mining_currency(), ALICE_MINING_BALANCE.into())])
		.build()
		.execute_with(|| {
			let origin = Origin::signed(ALICE);
			init_test_nft(origin.clone(), DISTRIBUTOR_COLLECTION_ID, DISTRIBUTOR_CLASS_ID);

			assert_ok!(EconomyModule::authorize_power_distributor_collection(
				Origin::root(),
				DISTRIBUTOR_COLLECTION_ID,
				DISTRIBUTOR_CLASS_ID
			));

			assert_ok!(EconomyModule::buy_power_by_user(
				origin.clone(),
				USER_BUY_POWER_AMOUNT,
				DISTRIBUTOR_NFT_ASSET_ID,
			));

			assert_err!(
				EconomyModule::execute_buy_power_order(origin, DISTRIBUTOR_NFT_ASSET_ID, ALICE),
				Error::<Runtime>::InsufficientBalanceToDistributePower
			);
		});
}

#[test]
fn execute_buy_power_order_should_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, get_mining_currency(), ALICE_MINING_BALANCE.into())])
		.build()
		.execute_with(|| {
			let origin = Origin::signed(ALICE);
			let mining_currency_id = get_mining_currency();

			init_test_nft(origin.clone(), DISTRIBUTOR_COLLECTION_ID, DISTRIBUTOR_CLASS_ID);

			assert_ok!(EconomyModule::authorize_power_distributor_collection(
				Origin::root(),
				DISTRIBUTOR_COLLECTION_ID,
				DISTRIBUTOR_CLASS_ID
			));

			assert_ok!(EconomyModule::buy_power_by_user(
				origin.clone(),
				USER_BUY_POWER_AMOUNT,
				DISTRIBUTOR_NFT_ASSET_ID,
			));

			let bit_amount = USER_BUY_POWER_AMOUNT + 100;
			assert_eq!(
				OrmlTokens::reserved_balance(mining_currency_id, &ALICE),
				bit_amount.into()
			);

			let distributor_account_id = sub_account(DISTRIBUTOR_NFT_ASSET_ID);
			PowerBalance::<Runtime>::insert(distributor_account_id, DISTRIBUTOR_POWER_BALANCE);

			assert_ok!(EconomyModule::execute_buy_power_order(
				origin,
				DISTRIBUTOR_NFT_ASSET_ID,
				ALICE
			));

			let event = Event::Economy(crate::Event::BuyPowerOrderByUserExecuted(
				ALICE,
				USER_BUY_POWER_AMOUNT,
				DISTRIBUTOR_NFT_ASSET_ID,
			));
			assert_eq!(last_event(), event);

			assert_eq!(
				EconomyModule::get_buy_power_by_user_request_queue(DISTRIBUTOR_NFT_ASSET_ID),
				Some(vec![])
			);

			let remaining_balance: PowerAmount = DISTRIBUTOR_POWER_BALANCE - USER_BUY_POWER_AMOUNT;
			assert_eq!(
				EconomyModule::get_power_balance(distributor_account_id),
				remaining_balance
			);

			assert_eq!(EconomyModule::get_power_balance(ALICE), USER_BUY_POWER_AMOUNT);

			// Check reserved balance
			assert_eq!(OrmlTokens::reserved_balance(mining_currency_id, &ALICE), 0u8.into());

			let remaining_amount: u64 = ALICE_MINING_BALANCE - USER_BUY_POWER_AMOUNT;
			assert_eq!(
				OrmlTokens::free_balance(mining_currency_id, &ALICE),
				(remaining_amount - 100).into()
			);
		});
}

#[test]
fn execute_generate_power_order_should_fail_nft_does_not_exist() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		init_test_nft(origin.clone(), DISTRIBUTOR_COLLECTION_ID, DISTRIBUTOR_CLASS_ID);
		init_test_nft(origin.clone(), GENERATOR_COLLECTION_ID, GENERATOR_CLASS_ID);

		assert_ok!(EconomyModule::authorize_power_generator_collection(
			Origin::root(),
			GENERATOR_COLLECTION_ID,
			GENERATOR_CLASS_ID
		));

		assert_noop!(
			EconomyModule::execute_generate_power_order(origin, NFT_ASSET_ID_NOT_EXIST, ALICE),
			Error::<Runtime>::NFTAssetDoesNotExist
		);
	});
}

#[test]
fn execute_generate_power_order_should_fail_distributor_does_not_exist() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		init_test_nft(origin.clone(), DISTRIBUTOR_COLLECTION_ID, DISTRIBUTOR_CLASS_ID);
		init_test_nft(origin.clone(), GENERATOR_COLLECTION_ID, GENERATOR_CLASS_ID);

		assert_ok!(EconomyModule::authorize_power_generator_collection(
			Origin::root(),
			GENERATOR_COLLECTION_ID,
			GENERATOR_CLASS_ID
		));

		assert_noop!(
			EconomyModule::execute_generate_power_order(origin, GENERATOR_NFT_ASSET_ID, ALICE),
			Error::<Runtime>::GeneratorNftDoesNotExist
		);
	});
}

#[test]
fn execute_generate_power_order_should_fail_account_does_not_exist() {
	ExtBuilder::default()
		.balances(vec![(
			sub_account(DISTRIBUTOR_NFT_ASSET_ID),
			get_mining_currency(),
			DISTRIBUTOR_MINING_BALANCE.into(),
		)])
		.build()
		.execute_with(|| {
			let origin = Origin::signed(ALICE);
			init_test_nft(origin.clone(), DISTRIBUTOR_COLLECTION_ID, DISTRIBUTOR_CLASS_ID);
			init_test_nft(origin.clone(), GENERATOR_COLLECTION_ID, GENERATOR_CLASS_ID);

			assert_ok!(EconomyModule::authorize_power_generator_collection(
				Origin::root(),
				GENERATOR_COLLECTION_ID,
				GENERATOR_CLASS_ID
			));

			assert_ok!(EconomyModule::buy_power_by_distributor(
				origin.clone(),
				GENERATOR_NFT_ASSET_ID,
				DISTRIBUTOR_NFT_ASSET_ID,
				GENERATE_POWER_AMOUNT,
			));

			assert_noop!(
				EconomyModule::execute_generate_power_order(origin, GENERATOR_NFT_ASSET_ID, BOB),
				Error::<Runtime>::DistributorAccountIdDoesNotExistInBuyOrderQueue
			);
		});
}

#[test]
fn execute_generate_power_order_should_fail_insufficient_balance() {
	ExtBuilder::default()
		.balances(vec![(
			sub_account(DISTRIBUTOR_NFT_ASSET_ID),
			get_mining_currency(),
			DISTRIBUTOR_MINING_BALANCE.into(),
		)])
		.build()
		.execute_with(|| {
			let origin = Origin::signed(ALICE);
			init_test_nft(origin.clone(), DISTRIBUTOR_COLLECTION_ID, DISTRIBUTOR_CLASS_ID);
			init_test_nft(origin.clone(), GENERATOR_COLLECTION_ID, GENERATOR_CLASS_ID);

			assert_ok!(EconomyModule::authorize_power_generator_collection(
				Origin::root(),
				GENERATOR_COLLECTION_ID,
				GENERATOR_CLASS_ID
			));

			assert_ok!(EconomyModule::buy_power_by_distributor(
				origin.clone(),
				GENERATOR_NFT_ASSET_ID,
				DISTRIBUTOR_NFT_ASSET_ID,
				GENERATE_POWER_AMOUNT,
			));

			let distributor_account_id = sub_account(DISTRIBUTOR_NFT_ASSET_ID);

			assert_err!(
				EconomyModule::execute_generate_power_order(origin, GENERATOR_NFT_ASSET_ID, distributor_account_id),
				Error::<Runtime>::InsufficientBalanceToGeneratePower
			);
		});
}

#[test]
fn execute_generate_power_order_should_work() {
	ExtBuilder::default()
		.balances(vec![(
			sub_account(DISTRIBUTOR_NFT_ASSET_ID),
			get_mining_currency(),
			DISTRIBUTOR_MINING_BALANCE.into(),
		)])
		.build()
		.execute_with(|| {
			let origin = Origin::signed(ALICE);

			init_test_nft(origin.clone(), DISTRIBUTOR_COLLECTION_ID, DISTRIBUTOR_CLASS_ID);
			init_test_nft(origin.clone(), GENERATOR_COLLECTION_ID, GENERATOR_CLASS_ID);

			let mining_currency_id = get_mining_currency();

			let distributor_account_id = sub_account(DISTRIBUTOR_NFT_ASSET_ID);
			let generator_account_id = sub_account(GENERATOR_NFT_ASSET_ID);

			assert_ok!(EconomyModule::authorize_power_generator_collection(
				Origin::root(),
				GENERATOR_COLLECTION_ID,
				GENERATOR_CLASS_ID
			));

			assert_ok!(EconomyModule::buy_power_by_distributor(
				origin.clone(),
				GENERATOR_NFT_ASSET_ID,
				DISTRIBUTOR_NFT_ASSET_ID,
				GENERATE_POWER_AMOUNT,
			));

			let bit_amount = GENERATE_POWER_AMOUNT + 100;
			assert_eq!(
				OrmlTokens::reserved_balance(mining_currency_id, &distributor_account_id),
				bit_amount.into()
			);

			PowerBalance::<Runtime>::insert(generator_account_id, GENERATOR_POWER_BALANCE);

			assert_ok!(EconomyModule::execute_generate_power_order(
				origin,
				GENERATOR_NFT_ASSET_ID,
				distributor_account_id
			));

			let event = Event::Economy(crate::Event::BuyPowerOrderByDistributorExecuted(
				distributor_account_id,
				GENERATE_POWER_AMOUNT,
				GENERATOR_NFT_ASSET_ID,
			));
			assert_eq!(last_event(), event);

			assert_eq!(
				EconomyModule::get_buy_power_by_distributor_request_queue(GENERATOR_NFT_ASSET_ID),
				Some(vec![])
			);

			let remaining_balance: PowerAmount = GENERATOR_POWER_BALANCE - GENERATE_POWER_AMOUNT;
			assert_eq!(
				EconomyModule::get_power_balance(generator_account_id),
				remaining_balance
			);

			assert_eq!(
				EconomyModule::get_power_balance(distributor_account_id),
				GENERATE_POWER_AMOUNT
			);

			// Check reserved balance
			assert_eq!(OrmlTokens::reserved_balance(mining_currency_id, &ALICE), 0u8.into());

			let remaining_amount: u64 = DISTRIBUTOR_MINING_BALANCE - GENERATE_POWER_AMOUNT;
			assert_eq!(
				OrmlTokens::free_balance(mining_currency_id, &distributor_account_id),
				(remaining_amount - 100).into()
			);
		});
}

#[test]
fn mint_element_should_fail_element_does_not_exist() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);

		assert_noop!(
			EconomyModule::mint_element(origin, ELEMENT_INDEX_ID, ELEMENT_AMOUNT),
			Error::<Runtime>::ElementDoesNotExist
		);
	});
}

#[test]
fn mint_element_should_fail_insufficient_balance() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);

		ElementIndex::<Runtime>::insert(
			ELEMENT_INDEX_ID,
			ElementInfo {
				power_price: 10,
				compositions: vec![],
			},
		);

		assert_noop!(
			EconomyModule::mint_element(origin, ELEMENT_INDEX_ID, ELEMENT_AMOUNT),
			Error::<Runtime>::InsufficientBalanceToMintElement
		);
	});
}

#[test]
fn mint_element_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);

		ElementIndex::<Runtime>::insert(
			ELEMENT_INDEX_ID,
			ElementInfo {
				power_price: 10,
				compositions: vec![],
			},
		);

		PowerBalance::<Runtime>::insert(ALICE, ALICE_POWER_AMOUNT);

		assert_ok!(EconomyModule::mint_element(origin, ELEMENT_INDEX_ID, ELEMENT_AMOUNT));

		let event = Event::Economy(crate::Event::ElementMinted(ALICE, ELEMENT_INDEX_ID, ELEMENT_AMOUNT));
		assert_eq!(last_event(), event);

		let remaining_balance: PowerAmount = ALICE_POWER_AMOUNT - 10 * ELEMENT_AMOUNT;
		assert_eq!(EconomyModule::get_power_balance(ALICE), remaining_balance);

		assert_eq!(
			EconomyModule::get_elements_by_account(ALICE, ELEMENT_INDEX_ID),
			ELEMENT_AMOUNT
		);
	});
}

#[test]
fn mint_element_should_work_with_more_one_operation() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);

		ElementIndex::<Runtime>::insert(
			ELEMENT_INDEX_ID,
			ElementInfo {
				power_price: 10,
				compositions: vec![],
			},
		);

		PowerBalance::<Runtime>::insert(ALICE, ALICE_POWER_AMOUNT);

		assert_ok!(EconomyModule::mint_element(
			origin.clone(),
			ELEMENT_INDEX_ID,
			ELEMENT_AMOUNT
		));

		assert_ok!(EconomyModule::mint_element(origin, ELEMENT_INDEX_ID, ELEMENT_AMOUNT));

		let remaining_balance: PowerAmount = ALICE_POWER_AMOUNT - 10 * ELEMENT_AMOUNT - 10 * ELEMENT_AMOUNT;
		assert_eq!(EconomyModule::get_power_balance(ALICE), remaining_balance);

		assert_eq!(
			EconomyModule::get_elements_by_account(ALICE, ELEMENT_INDEX_ID),
			ELEMENT_AMOUNT + ELEMENT_AMOUNT
		);
	});
}
