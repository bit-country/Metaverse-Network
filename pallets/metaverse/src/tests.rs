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
use sp_runtime::Perbill;

use mock::{Event, *};
use primitives::staking::RoundInfo;

#[cfg(test)]
use super::*;

fn free_native_balance(who: AccountId) -> Balance {
	<Runtime as Config>::Currency::free_balance(who)
}

#[test]
fn create_metaverse_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(ALICE), vec![1]));
		assert_eq!(
			MetaverseModule::get_metaverse(&METAVERSE_ID),
			Some(MetaverseInfo {
				owner: ALICE,
				metadata: vec![1],
				currency_id: FungibleTokenId::NativeToken(0),
				is_frozen: false,
				land_class_id: 0u32,
				estate_class_id: 0u32,
				listing_fee: Perbill::from_percent(0u32)
			})
		);
		let event = Event::Metaverse(crate::Event::NewMetaverseCreated(METAVERSE_ID, ALICE));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn verify_is_metaverse_owner_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(ALICE), vec![1]));
		assert_eq!(
			MetaverseModule::get_metaverse(&METAVERSE_ID),
			Some(MetaverseInfo {
				owner: ALICE,
				metadata: vec![1],
				currency_id: FungibleTokenId::NativeToken(0),
				is_frozen: false,
				land_class_id: 0u32,
				estate_class_id: 0u32,
				listing_fee: Perbill::from_percent(0u32)
			})
		);
		let event = Event::Metaverse(crate::Event::NewMetaverseCreated(METAVERSE_ID, ALICE));
		assert_eq!(last_event(), event);

		assert_eq!(MetaverseModule::is_metaverse_owner(&ALICE), true);
		assert_eq!(MetaverseModule::is_metaverse_owner(&BOB), false);
	});
}

#[test]
fn transfer_metaverse_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(ALICE), vec![1]));
		assert_ok!(MetaverseModule::transfer_metaverse(
			Origin::signed(ALICE),
			BOB,
			METAVERSE_ID
		));
		let event = Event::Metaverse(crate::Event::TransferredMetaverse(METAVERSE_ID, ALICE, BOB));
		assert_eq!(last_event(), event);
		// Make sure 2 ways transfer works
		assert_ok!(MetaverseModule::transfer_metaverse(
			Origin::signed(BOB),
			ALICE,
			METAVERSE_ID
		));
		let event = Event::Metaverse(crate::Event::TransferredMetaverse(METAVERSE_ID, BOB, ALICE));
		assert_eq!(last_event(), event);
	})
}

#[test]
fn transfer_metaverse_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(ALICE), vec![1]));
		assert_noop!(
			MetaverseModule::transfer_metaverse(Origin::signed(BOB), ALICE, METAVERSE_ID),
			Error::<Runtime>::NoPermission
		);
	})
}

#[test]
fn freeze_metaverse_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(ALICE), vec![1]));
		assert_ok!(MetaverseModule::freeze_metaverse(Origin::signed(ALICE), METAVERSE_ID));
		let event = Event::Metaverse(crate::Event::MetaverseFreezed(METAVERSE_ID));
		assert_eq!(last_event(), event);
	})
}

#[test]
fn freeze_metaverse_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(ALICE), vec![1]));
		//Country owner tries to freeze their own metaverse
		assert_noop!(
			MetaverseModule::freeze_metaverse(Origin::signed(BOB), METAVERSE_ID),
			BadOrigin
		);
	})
}

#[test]
fn unfreeze_metaverse_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(ALICE), vec![1]));
		assert_ok!(MetaverseModule::freeze_metaverse(Origin::signed(ALICE), METAVERSE_ID));
		let event = Event::Metaverse(crate::Event::MetaverseFreezed(METAVERSE_ID));
		assert_eq!(last_event(), event);
		assert_ok!(MetaverseModule::unfreeze_metaverse(Origin::signed(ALICE), METAVERSE_ID));
		let event = Event::Metaverse(crate::Event::MetaverseUnfreezed(METAVERSE_ID));
		assert_eq!(last_event(), event);
	})
}

#[test]
fn destroy_metaverse_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(ALICE), vec![1]));
		assert_ok!(MetaverseModule::freeze_metaverse(Origin::signed(ALICE), METAVERSE_ID));
		assert_ok!(MetaverseModule::destroy_metaverse(Origin::signed(ALICE), METAVERSE_ID));
		let event = Event::Metaverse(crate::Event::MetaverseDestroyed(METAVERSE_ID));
		assert_eq!(MetaverseModule::get_metaverse(&METAVERSE_ID), None);
		assert_eq!(last_event(), event);
	})
}

#[test]
fn destroy_metaverse_without_root_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(ALICE), vec![1]));
		assert_noop!(
			MetaverseModule::destroy_metaverse(Origin::signed(2), METAVERSE_ID),
			BadOrigin
		);
	})
}

#[test]
fn destroy_metaverse_with_no_id_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(ALICE), vec![1]));
		assert_noop!(
			MetaverseModule::destroy_metaverse(Origin::signed(ALICE), COUNTRY_ID_NOT_EXIST),
			Error::<Runtime>::MetaverseInfoNotFound
		);
	})
}

#[test]
fn update_metaverse_listing_fee_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(ALICE), vec![1]));
		assert_ok!(MetaverseModule::update_metaverse_listing_fee(
			Origin::signed(ALICE),
			METAVERSE_ID,
			Perbill::from_percent(10u32)
		));
		assert_eq!(
			MetaverseModule::get_metaverse_marketplace_listing_fee(METAVERSE_ID),
			Ok(Perbill::from_percent(10u32))
		);
	})
}

#[test]
fn update_metaverse_listing_fee_should_fail_if_exceed_25_percents() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(ALICE), vec![1]));
		assert_noop!(
			MetaverseModule::update_metaverse_listing_fee(
				Origin::signed(ALICE),
				METAVERSE_ID,
				Perbill::from_percent(26u32)
			),
			Error::<Runtime>::MetaverseListingFeeExceedThreshold
		);
	})
}

#[test]
fn update_metaverse_listing_fee_should_fail_if_not_metaverse_owner() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(ALICE), vec![1]));
		assert_noop!(
			MetaverseModule::update_metaverse_listing_fee(
				Origin::signed(BOB),
				METAVERSE_ID,
				Perbill::from_percent(10u32)
			),
			Error::<Runtime>::NoPermission
		);
	})
}

#[test]
fn do_withdraw_funds_from_metaverse_treasury_fund_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			MetaverseModule::withdraw_from_metaverse_fund(Origin::signed(ALICE), METAVERSE_ID),
			Error::<Runtime>::NoPermission
		);
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(ALICE), vec![1]));
		assert_noop!(
			MetaverseModule::withdraw_from_metaverse_fund(Origin::signed(BOB), METAVERSE_ID),
			Error::<Runtime>::NoPermission
		);
	})
}

#[test]
fn do_withdraw_from_metaverse_fund_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(ALICE), vec![1]));
		let metaverse_fund: AccountId =
			<Runtime as Config>::MetaverseTreasury::get().into_sub_account_truncating(METAVERSE_ID);
		assert_ok!(<Runtime as Config>::Currency::transfer(
			origin.clone(),
			metaverse_fund,
			100
		));
		assert_eq!(free_native_balance(ALICE), 9999999999999999899);
		assert_eq!(free_native_balance(metaverse_fund), 101);
		assert_ok!(MetaverseModule::withdraw_from_metaverse_fund(
			origin.clone(),
			METAVERSE_ID
		));
		assert_eq!(free_native_balance(ALICE), 9999999999999999999);
		assert_eq!(free_native_balance(metaverse_fund), 1);
	})
}
