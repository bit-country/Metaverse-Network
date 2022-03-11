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
use sp_std::default::Default;

use auction_manager::ListingLevel;
use mock::{Event, *};
use pallet_nft::{Attributes, CollectionType, TokenType};
use primitives::GroupCollectionId;

use super::*;

fn init_test_nft(owner: Origin, collection_id: GroupCollectionId, classId: ClassId) {
	//Create group collection before class
	assert_ok!(NFTModule::create_group(Origin::root(), vec![1], vec![1]));

	assert_ok!(NFTModule::create_class(
		owner.clone(),
		vec![1],
		test_attributes(1),
		collection_id,
		TokenType::Transferable,
		CollectionType::Collectable,
	));

	assert_ok!(NFTModule::mint(owner.clone(), classId, vec![1], test_attributes(1), 1));
}

fn test_attributes(x: u8) -> Attributes {
	let mut attr: Attributes = BTreeMap::new();
	attr.insert(vec![x, x + 5], vec![x, x + 10]);
	attr
}

fn sub_account(nft_id: (ClassId, TokenId)) -> AccountId {
	<Runtime as Config>::EconomyTreasury::get().into_sub_account(nft_id)
}

fn get_mining_currency() -> FungibleTokenId {
	<Runtime as Config>::MiningCurrencyId::get()
}

#[test]
fn authorize_power_generator_collection_should_fail_class_id_does_not_exists() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EconomyModule::authorize_power_generator_collection(
				Origin::root(),
				GENERATOR_COLLECTION_ID,
				GENERATOR_CLASS_ID,
				Default::default()
			),
			pallet_nft::Error::<Runtime>::ClassIdNotFound
		);
	});
}

#[test]
fn authorize_power_generator_collection_should_fail_collection_id_does_not_match() {
	ExtBuilder::default().build().execute_with(|| {
		init_test_nft(Origin::signed(ALICE), DISTRIBUTOR_COLLECTION_ID, DISTRIBUTOR_CLASS_ID);
		init_test_nft(Origin::signed(ALICE), GENERATOR_COLLECTION_ID, GENERATOR_CLASS_ID);

		assert_noop!(
			EconomyModule::authorize_power_generator_collection(
				Origin::root(),
				COLLECTION_ID_NOT_EXIST,
				GENERATOR_CLASS_ID,
				Default::default()
			),
			Error::<Runtime>::CollectionIdDoesNotMatchNFTCollectionId
		);
	});
}

#[test]
fn authorize_power_generator_collection_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		init_test_nft(Origin::signed(ALICE), DISTRIBUTOR_COLLECTION_ID, DISTRIBUTOR_CLASS_ID);
		init_test_nft(Origin::signed(ALICE), GENERATOR_COLLECTION_ID, GENERATOR_CLASS_ID);

		assert_ok!(EconomyModule::authorize_power_generator_collection(
			Origin::root(),
			GENERATOR_COLLECTION_ID,
			GENERATOR_CLASS_ID,
			Default::default()
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
				GENERATOR_CLASS_ID,
				Default::default()
			),
			BadOrigin
		);
	});
}

#[test]
fn authorize_power_distributor_collection_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		init_test_nft(Origin::signed(ALICE), DISTRIBUTOR_COLLECTION_ID, DISTRIBUTOR_CLASS_ID);

		assert_ok!(EconomyModule::authorize_power_distributor_collection(
			Origin::root(),
			DISTRIBUTOR_COLLECTION_ID,
			DISTRIBUTOR_CLASS_ID,
			Default::default()
		));

		let event = Event::Economy(crate::Event::PowerDistributorCollectionAuthorized(
			DISTRIBUTOR_COLLECTION_ID,
			DISTRIBUTOR_CLASS_ID,
		));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn authorize_power_distributor_collection_should_fail_class_id_does_not_exists() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EconomyModule::authorize_power_distributor_collection(
				Origin::root(),
				DISTRIBUTOR_COLLECTION_ID,
				DISTRIBUTOR_CLASS_ID,
				Default::default()
			),
			pallet_nft::Error::<Runtime>::ClassIdNotFound
		);
	});
}

#[test]
fn authorize_power_distributor_collection_should_fail_collection_id_does_not_match() {
	ExtBuilder::default().build().execute_with(|| {
		init_test_nft(Origin::signed(ALICE), DISTRIBUTOR_COLLECTION_ID, DISTRIBUTOR_CLASS_ID);

		assert_noop!(
			EconomyModule::authorize_power_distributor_collection(
				Origin::root(),
				COLLECTION_ID_NOT_EXIST,
				DISTRIBUTOR_CLASS_ID,
				Default::default()
			),
			Error::<Runtime>::CollectionIdDoesNotMatchNFTCollectionId
		);
	});
}

#[test]
fn authorize_power_distributor_collection_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EconomyModule::authorize_power_distributor_collection(
				Origin::signed(BOB),
				DISTRIBUTOR_COLLECTION_ID,
				DISTRIBUTOR_CLASS_ID,
				Default::default()
			),
			BadOrigin
		);
	});
}

#[test]
fn buy_power_by_user_should_fail_nft_does_not_exist() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EconomyModule::buy_power_by_user(Origin::signed(ALICE), 100u64.into(), (0, 0)),
			Error::<Runtime>::NoPermissionToBuyPower
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
			Error::<Runtime>::NoPermissionToBuyPower
		);
	});
}

#[test]
fn buy_power_by_user_should_fail_insufficient_balance() {
	ExtBuilder::default()
		.balances(vec![(ALICE, get_mining_currency(), ALICE_MINING_LOW_BALANCE.into())])
		.build()
		.execute_with(|| {
			let origin = Origin::signed(ALICE);
			init_test_nft(origin.clone(), DISTRIBUTOR_COLLECTION_ID, DISTRIBUTOR_CLASS_ID);

			assert_ok!(EconomyModule::authorize_power_distributor_collection(
				Origin::root(),
				DISTRIBUTOR_COLLECTION_ID,
				DISTRIBUTOR_CLASS_ID,
				Default::default()
			));

			assert_ok!(EconomyModule::set_bit_power_exchange_rate(
				Origin::root(),
				EXCHANGE_RATE
			));

			assert_noop!(
				EconomyModule::buy_power_by_user(origin, USER_BUY_POWER_AMOUNT, DISTRIBUTOR_NFT_ASSET_ID,),
				Error::<Runtime>::InsufficientBalanceToBuyPower
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
				DISTRIBUTOR_CLASS_ID,
				Default::default()
			));

			assert_ok!(EconomyModule::set_bit_power_exchange_rate(
				Origin::root(),
				EXCHANGE_RATE
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

			let bit_amount: mock::Balance = EXCHANGE_RATE * u128::try_from(USER_BUY_POWER_AMOUNT).unwrap();
			assert_eq!(
				EconomyModule::get_buy_power_by_user_request_queue(DISTRIBUTOR_NFT_ASSET_ID, ALICE),
				Some(OrderInfo {
					power_amount: USER_BUY_POWER_AMOUNT,
					bit_amount: bit_amount.into()
				})
			);

			// Check reserved balance
			let mining_currency_id = get_mining_currency();
			assert_eq!(
				OrmlTokens::reserved_balance(mining_currency_id, &ALICE),
				bit_amount.into()
			);

			let remaining_amount: u128 = ALICE_MINING_BALANCE - u128::try_from(bit_amount).unwrap();
			assert_eq!(OrmlTokens::free_balance(mining_currency_id, &ALICE), remaining_amount);
		});
}

#[test]
fn buy_power_by_distributor_should_fail_nft_does_not_exist() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EconomyModule::buy_power_by_distributor(
				Origin::signed(ALICE),
				GENERATOR_NFT_ASSET_ID,
				DISTRIBUTOR_NFT_ASSET_ID,
				GENERATE_POWER_AMOUNT,
			),
			Error::<Runtime>::NoPermissionToBuyPower
		);
	});
}

#[test]
fn buy_power_by_distributor_should_fail_nft_not_authorized() {
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
			Error::<Runtime>::NoPermissionToBuyPower
		);
	});
}

#[test]
fn buy_power_by_distributor_should_fail_insufficient_balance() {
	ExtBuilder::default()
		.balances(vec![(
			sub_account(DISTRIBUTOR_NFT_ASSET_ID),
			get_mining_currency(),
			DISTRIBUTOR_MINING_LOW_BALANCE.into(),
		)])
		.build()
		.execute_with(|| {
			let origin = Origin::signed(ALICE);

			let sub_account = sub_account(DISTRIBUTOR_NFT_ASSET_ID);

			init_test_nft(origin.clone(), DISTRIBUTOR_COLLECTION_ID, DISTRIBUTOR_CLASS_ID);
			init_test_nft(origin.clone(), GENERATOR_COLLECTION_ID, GENERATOR_CLASS_ID);

			assert_ok!(EconomyModule::authorize_power_generator_collection(
				Origin::root(),
				GENERATOR_COLLECTION_ID,
				GENERATOR_CLASS_ID,
				Default::default()
			));

			assert_ok!(EconomyModule::authorize_power_distributor_collection(
				Origin::root(),
				DISTRIBUTOR_COLLECTION_ID,
				DISTRIBUTOR_CLASS_ID,
				Default::default()
			));

			assert_ok!(EconomyModule::set_bit_power_exchange_rate(
				Origin::root(),
				EXCHANGE_RATE
			));

			assert_noop!(
				EconomyModule::buy_power_by_distributor(
					origin,
					GENERATOR_NFT_ASSET_ID,
					DISTRIBUTOR_NFT_ASSET_ID,
					GENERATE_POWER_AMOUNT,
				),
				Error::<Runtime>::InsufficientBalanceToBuyPower
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
				GENERATOR_CLASS_ID,
				Default::default()
			));

			assert_ok!(EconomyModule::authorize_power_distributor_collection(
				Origin::root(),
				DISTRIBUTOR_COLLECTION_ID,
				DISTRIBUTOR_CLASS_ID,
				Default::default()
			));

			assert_ok!(EconomyModule::set_bit_power_exchange_rate(
				Origin::root(),
				EXCHANGE_RATE
			));

			let distributor_account_id = sub_account(DISTRIBUTOR_NFT_ASSET_ID);
			let mining_currency_id = get_mining_currency();

			assert_ok!(EconomyModule::buy_power_by_distributor(
				origin,
				GENERATOR_NFT_ASSET_ID,
				DISTRIBUTOR_NFT_ASSET_ID,
				GENERATE_POWER_AMOUNT,
			));

			let event = Event::Economy(crate::Event::BuyPowerOrderByDistributorHasAddedToQueue(
				distributor_account_id,
				GENERATE_POWER_AMOUNT,
				GENERATOR_NFT_ASSET_ID,
			));
			assert_eq!(last_event(), event);

			let bit_amount: mock::Balance = EXCHANGE_RATE * u128::try_from(GENERATE_POWER_AMOUNT).unwrap();
			assert_eq!(
				EconomyModule::get_buy_power_by_distributor_request_queue(
					GENERATOR_NFT_ASSET_ID,
					distributor_account_id
				),
				Some(OrderInfo {
					power_amount: GENERATE_POWER_AMOUNT,
					bit_amount: bit_amount.into()
				})
			);

			// Check reserved balance
			assert_eq!(
				OrmlTokens::reserved_balance(mining_currency_id, &distributor_account_id),
				bit_amount.into()
			);

			let remaining_amount: u128 = DISTRIBUTOR_MINING_BALANCE - u128::try_from(bit_amount).unwrap();
			assert_eq!(
				OrmlTokens::free_balance(mining_currency_id, &distributor_account_id),
				remaining_amount
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
			DISTRIBUTOR_CLASS_ID,
			Default::default()
		));

		assert_noop!(
			EconomyModule::execute_buy_power_order(origin, NFT_ASSET_ID_NOT_EXIST, ALICE),
			pallet_nft::Error::<Runtime>::AssetInfoNotFound
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
			DISTRIBUTOR_CLASS_ID,
			Default::default()
		));

		assert_noop!(
			EconomyModule::execute_buy_power_order(origin, DISTRIBUTOR_NFT_ASSET_ID, ALICE),
			Error::<Runtime>::PowerDistributionQueueDoesNotExist
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
				DISTRIBUTOR_CLASS_ID,
				Default::default()
			));

			assert_ok!(EconomyModule::buy_power_by_user(
				origin.clone(),
				USER_BUY_POWER_AMOUNT,
				DISTRIBUTOR_NFT_ASSET_ID,
			));

			assert_noop!(
				EconomyModule::execute_buy_power_order(origin, DISTRIBUTOR_NFT_ASSET_ID, BOB),
				Error::<Runtime>::PowerDistributionQueueDoesNotExist
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
				DISTRIBUTOR_CLASS_ID,
				Default::default()
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
				DISTRIBUTOR_CLASS_ID,
				Default::default()
			));

			assert_ok!(EconomyModule::buy_power_by_user(
				origin.clone(),
				USER_BUY_POWER_AMOUNT,
				DISTRIBUTOR_NFT_ASSET_ID,
			));

			let order_info =
				EconomyModule::get_buy_power_by_user_request_queue(DISTRIBUTOR_NFT_ASSET_ID, ALICE).unwrap();

			let bit_amount = order_info.bit_amount;
			assert_eq!(OrmlTokens::reserved_balance(mining_currency_id, &ALICE), bit_amount);

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
				EconomyModule::get_buy_power_by_user_request_queue(DISTRIBUTOR_NFT_ASSET_ID, ALICE),
				None
			);

			let remaining_balance: PowerAmount = DISTRIBUTOR_POWER_BALANCE - USER_BUY_POWER_AMOUNT;
			assert_eq!(
				EconomyModule::get_power_balance(distributor_account_id),
				remaining_balance
			);

			assert_eq!(EconomyModule::get_power_balance(ALICE), USER_BUY_POWER_AMOUNT);

			// Check reserved balance
			assert_eq!(OrmlTokens::reserved_balance(mining_currency_id, &ALICE), 0u8.into());

			let remaining_amount: mock::Balance = (ALICE_MINING_BALANCE - bit_amount).into();
			assert_eq!(OrmlTokens::free_balance(mining_currency_id, &ALICE), remaining_amount);
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
			GENERATOR_CLASS_ID,
			Default::default()
		));

		assert_noop!(
			EconomyModule::execute_generate_power_order(origin, NFT_ASSET_ID_NOT_EXIST, ALICE),
			Error::<Runtime>::PowerGenerationIsNotAuthorized
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
			GENERATOR_CLASS_ID,
			Default::default()
		));

		assert_noop!(
			EconomyModule::execute_generate_power_order(origin, GENERATOR_NFT_ASSET_ID, ALICE),
			Error::<Runtime>::PowerGenerationQueueDoesNotExist
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
				GENERATOR_CLASS_ID,
				Default::default()
			));

			assert_ok!(EconomyModule::authorize_power_distributor_collection(
				Origin::root(),
				DISTRIBUTOR_COLLECTION_ID,
				DISTRIBUTOR_CLASS_ID,
				Default::default()
			));

			assert_ok!(EconomyModule::buy_power_by_distributor(
				origin.clone(),
				GENERATOR_NFT_ASSET_ID,
				DISTRIBUTOR_NFT_ASSET_ID,
				GENERATE_POWER_AMOUNT,
			));

			assert_noop!(
				EconomyModule::execute_generate_power_order(origin, GENERATOR_NFT_ASSET_ID, BOB),
				Error::<Runtime>::PowerGenerationQueueDoesNotExist
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
				GENERATOR_CLASS_ID,
				Default::default()
			));

			assert_ok!(EconomyModule::authorize_power_distributor_collection(
				Origin::root(),
				DISTRIBUTOR_COLLECTION_ID,
				DISTRIBUTOR_CLASS_ID,
				Default::default()
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
				GENERATOR_CLASS_ID,
				Default::default()
			));

			assert_ok!(EconomyModule::authorize_power_distributor_collection(
				Origin::root(),
				DISTRIBUTOR_COLLECTION_ID,
				DISTRIBUTOR_CLASS_ID,
				Default::default()
			));

			assert_ok!(EconomyModule::buy_power_by_distributor(
				origin.clone(),
				GENERATOR_NFT_ASSET_ID,
				DISTRIBUTOR_NFT_ASSET_ID,
				GENERATE_POWER_AMOUNT,
			));

			let order_info = EconomyModule::get_buy_power_by_distributor_request_queue(
				GENERATOR_NFT_ASSET_ID,
				distributor_account_id,
			)
			.unwrap();

			let bit_amount = order_info.bit_amount;
			assert_eq!(
				OrmlTokens::reserved_balance(mining_currency_id, &distributor_account_id),
				bit_amount
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
				EconomyModule::get_buy_power_by_distributor_request_queue(
					GENERATOR_NFT_ASSET_ID,
					distributor_account_id
				),
				None
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

			let remaining_amount: mock::Balance = (DISTRIBUTOR_MINING_BALANCE - bit_amount).into();
			assert_eq!(
				OrmlTokens::free_balance(mining_currency_id, &distributor_account_id),
				remaining_amount
			);
		});
}

#[test]
fn execute_generate_order_with_commission_should_work() {
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

			assert_ok!(EconomyModule::set_bit_power_exchange_rate(
				Origin::root(),
				EXCHANGE_RATE
			));

			let mining_currency_id = get_mining_currency();

			let distributor_account_id = sub_account(DISTRIBUTOR_NFT_ASSET_ID);
			let generator_account_id = sub_account(GENERATOR_NFT_ASSET_ID);

			assert_ok!(EconomyModule::authorize_power_generator_collection(
				Origin::root(),
				GENERATOR_COLLECTION_ID,
				GENERATOR_CLASS_ID,
				Perbill::from_percent(10)
			));

			assert_ok!(EconomyModule::authorize_power_distributor_collection(
				Origin::root(),
				DISTRIBUTOR_COLLECTION_ID,
				DISTRIBUTOR_CLASS_ID,
				Perbill::from_percent(10)
			));

			assert_ok!(EconomyModule::buy_power_by_distributor(
				origin.clone(),
				GENERATOR_NFT_ASSET_ID,
				DISTRIBUTOR_NFT_ASSET_ID,
				GENERATE_POWER_AMOUNT,
			));

			let rate = EconomyModule::get_bit_power_exchange_rate();
			let base_bit_required = EconomyModule::convert_power_to_bit(200u128, Perbill::from_percent(0));
			let bit_with_commission_required = EconomyModule::convert_power_to_bit(200u128, Perbill::from_percent(10));

			assert_eq!(base_bit_required, BIT_REQUIRED);
			assert_eq!(bit_with_commission_required, BIT_REQUIRED_WITH_10_PERCENT_COMMISSION);

			let order_info = EconomyModule::get_buy_power_by_distributor_request_queue(
				GENERATOR_NFT_ASSET_ID,
				distributor_account_id,
			)
			.unwrap();

			let bit_amount = order_info.bit_amount;
			assert_eq!(
				OrmlTokens::reserved_balance(mining_currency_id, &distributor_account_id),
				bit_amount
			);
			assert_eq!(bit_amount, BIT_REQUIRED_WITH_10_PERCENT_COMMISSION);

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
				EconomyModule::get_buy_power_by_distributor_request_queue(
					GENERATOR_NFT_ASSET_ID,
					distributor_account_id
				),
				None
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

			let remaining_amount: mock::Balance = (DISTRIBUTOR_MINING_BALANCE - bit_amount).into();
			assert_eq!(
				OrmlTokens::free_balance(mining_currency_id, &distributor_account_id),
				remaining_amount
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

#[test]
fn set_bit_power_exchange_rate_should_fail_bad_origin() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EconomyModule::set_bit_power_exchange_rate(Origin::signed(BOB), EXCHANGE_RATE),
			BadOrigin
		);
	});
}

#[test]
fn set_bit_power_exchange_rate_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EconomyModule::set_bit_power_exchange_rate(
			Origin::root(),
			EXCHANGE_RATE
		));

		assert_eq!(EconomyModule::get_bit_power_exchange_rate(), EXCHANGE_RATE);
	});
}

#[test]
fn stake_should_fail_insufficient_balance() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EconomyModule::stake(Origin::signed(ALICE), STAKE_EXCESS_BALANCE),
			Error::<Runtime>::InsufficientBalanceForStaking
		);
	});
}

#[test]
fn stake_should_fail_exit_queue_scheduled() {
	ExtBuilder::default().build().execute_with(|| {
		// Add account entry to ExitQueue
		ExitQueue::<Runtime>::insert(ALICE, CURRENT_ROUND, STAKE_BALANCE);

		assert_noop!(
			EconomyModule::stake(Origin::signed(ALICE), STAKE_BELOW_MINIMUM_BALANCE),
			Error::<Runtime>::ExitQueueAlreadyScheduled
		);
	});
}

#[test]
fn stake_should_fail_below_minimum() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EconomyModule::stake(Origin::signed(ALICE), STAKE_BELOW_MINIMUM_BALANCE),
			Error::<Runtime>::StakeBelowMinimum
		);
	});
}

#[test]
fn stake_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EconomyModule::stake(Origin::signed(ALICE), STAKE_BALANCE));

		assert_eq!(
			last_event(),
			Event::Economy(crate::Event::SelfStakedToEconomy101(ALICE, STAKE_BALANCE))
		);

		assert_eq!(Balances::reserved_balance(ALICE), STAKE_BALANCE);

		assert_eq!(EconomyModule::get_staking_info(ALICE), STAKE_BALANCE);

		assert_eq!(EconomyModule::total_stake(), STAKE_BALANCE);
	});
}

#[test]
fn stake_should_work_with_more_operations() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EconomyModule::stake(Origin::signed(ALICE), STAKE_BALANCE));

		assert_ok!(EconomyModule::stake(Origin::signed(ALICE), 100));

		let total_staked_balance = STAKE_BALANCE + 100u128;

		assert_eq!(Balances::reserved_balance(ALICE), total_staked_balance);

		assert_eq!(EconomyModule::get_staking_info(ALICE), total_staked_balance);

		assert_eq!(EconomyModule::total_stake(), total_staked_balance);
	});
}

#[test]
fn unstake_should_fail_exceeds_staked_amount() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EconomyModule::unstake(Origin::signed(ALICE), UNSTAKE_AMOUNT),
			Error::<Runtime>::UnstakeAmountExceedStakedAmount
		);
	});
}

#[test]
fn unstake_should_fail_unstake_zero() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EconomyModule::stake(Origin::signed(ALICE), STAKE_BALANCE));

		assert_noop!(
			EconomyModule::unstake(Origin::signed(ALICE), 0u128),
			Error::<Runtime>::UnstakeAmountExceedStakedAmountZero
		);
	});
}

#[test]
fn unstake_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EconomyModule::stake(Origin::signed(ALICE), STAKE_BALANCE));

		assert_ok!(EconomyModule::unstake(Origin::signed(ALICE), UNSTAKE_AMOUNT));

		assert_eq!(
			last_event(),
			Event::Economy(crate::Event::SelfStakingRemovedFromEconomy101(ALICE, UNSTAKE_AMOUNT))
		);

		let total_staked_balance = STAKE_BALANCE - UNSTAKE_AMOUNT;

		assert_eq!(EconomyModule::get_staking_info(ALICE), total_staked_balance);
		assert_eq!(EconomyModule::total_stake(), total_staked_balance);
		let next_round: RoundIndex = CURRENT_ROUND.saturating_add(1);
		assert_eq!(
			EconomyModule::staking_exit_queue(ALICE, next_round),
			Some(UNSTAKE_AMOUNT)
		);
	});
}

#[test]
fn withdraw_unstake_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EconomyModule::stake(Origin::signed(ALICE), STAKE_BALANCE));

		assert_ok!(EconomyModule::unstake(Origin::signed(ALICE), UNSTAKE_AMOUNT));

		assert_eq!(
			last_event(),
			Event::Economy(crate::Event::SelfStakingRemovedFromEconomy101(ALICE, UNSTAKE_AMOUNT))
		);

		let total_staked_balance = STAKE_BALANCE - UNSTAKE_AMOUNT;

		assert_eq!(EconomyModule::get_staking_info(ALICE), total_staked_balance);
		assert_eq!(EconomyModule::total_stake(), total_staked_balance);
		let next_round: RoundIndex = CURRENT_ROUND.saturating_add(1);
		assert_eq!(
			EconomyModule::staking_exit_queue(ALICE, next_round),
			Some(UNSTAKE_AMOUNT)
		);

		// Default round length is 20 blocks so moving 25 blocks will move to the next round
		run_to_block(25);
		assert_ok!(EconomyModule::withdraw_unreserved(Origin::signed(ALICE), next_round));
		// ALICE balance free_balance was 9000 and added 9010 after withdraw unreserved
		assert_eq!(Balances::free_balance(ALICE), FREE_BALANCE);
	});
}

#[test]
fn unstake_should_work_with_more_operation() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EconomyModule::stake(Origin::signed(ALICE), STAKE_BALANCE));

		assert_ok!(EconomyModule::unstake(Origin::signed(ALICE), UNSTAKE_AMOUNT));

		assert_ok!(EconomyModule::unstake(Origin::signed(ALICE), UNSTAKE_AMOUNT));

		assert_ok!(EconomyModule::stake(Origin::signed(BOB), 200));

		let alice_staked_balance = STAKE_BALANCE - UNSTAKE_AMOUNT - UNSTAKE_AMOUNT;

		assert_eq!(EconomyModule::get_staking_info(ALICE), alice_staked_balance);

		let total_staked_balance = alice_staked_balance + 200;
		assert_eq!(EconomyModule::total_stake(), total_staked_balance);
	});
}

#[test]
fn generate_power_from_generator_should_work() {
	ExtBuilder::default()
		.balances(vec![(
			sub_account(GENERATOR_NFT_ASSET_ID),
			get_mining_currency(),
			DISTRIBUTOR_MINING_BALANCE.into(),
		)])
		.build()
		.execute_with(|| {
			let origin = Origin::signed(ALICE);

			init_test_nft(origin.clone(), DISTRIBUTOR_COLLECTION_ID, DISTRIBUTOR_CLASS_ID);
			init_test_nft(origin.clone(), GENERATOR_COLLECTION_ID, GENERATOR_CLASS_ID);

			assert_ok!(EconomyModule::set_bit_power_exchange_rate(
				Origin::root(),
				EXCHANGE_RATE
			));

			let generator_account_id = sub_account(GENERATOR_NFT_ASSET_ID);

			assert_ok!(EconomyModule::authorize_power_generator_collection(
				Origin::root(),
				GENERATOR_COLLECTION_ID,
				GENERATOR_CLASS_ID,
				Perbill::from_percent(5)
			));

			assert_ok!(EconomyModule::get_more_power_by_generator(
				origin.clone(),
				GENERATOR_NFT_ASSET_ID,
				GENERATE_POWER_AMOUNT,
			));

			let event = Event::Economy(crate::Event::BuyPowerOrderByGeneratorToNetworkExecuted(
				generator_account_id,
				GENERATE_POWER_AMOUNT,
				GENERATOR_NFT_ASSET_ID,
			));
			assert_eq!(last_event(), event);

			assert_eq!(
				EconomyModule::get_power_balance(generator_account_id),
				GENERATE_POWER_AMOUNT
			);
		});
}
