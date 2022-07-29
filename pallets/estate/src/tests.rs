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

use mock::{Event, *};

use super::*;

fn estate_sub_account(estate_id: mock::EstateId) -> AccountId {
	<Runtime as Config>::LandTreasury::get().into_sub_account_truncating(estate_id)
}

#[test]
fn mint_land_should_reject_non_root() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EstateModule::mint_land(Origin::signed(ALICE), BENEFICIARY_ID, METAVERSE_ID, COORDINATE_IN_1),
			BadOrigin
		);
	});
}

#[test]
fn mint_land_should_work_with_one_coordinate() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::mint_land(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			COORDINATE_IN_1
		));

		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::NewLandUnitMinted(
				OWNER_LAND_ASSET_ID,
				METAVERSE_ID,
				COORDINATE_IN_1,
			))
		);

		assert_eq!(EstateModule::all_land_units_count(), 1);
	});
}

#[test]
fn mint_land_token_should_work_have_correct_owner() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(EstateModule::get_land_units(METAVERSE_ID, COORDINATE_IN_1), None);

		assert_ok!(EstateModule::mint_land(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			COORDINATE_IN_1
		));
		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::NewLandUnitMinted(
				OWNER_LAND_ASSET_ID,
				METAVERSE_ID,
				COORDINATE_IN_1,
			))
		);

		assert_eq!(EstateModule::all_land_units_count(), 1);

		assert_eq!(
			EstateModule::get_land_units(METAVERSE_ID, COORDINATE_IN_1),
			Some(OWNER_LAND_ASSET_ID)
		);
	});
}

#[test]
fn mint_land_should_reject_with_duplicate_coordinates() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::mint_land(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			COORDINATE_IN_1
		));

		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::NewLandUnitMinted(
				OWNER_LAND_ASSET_ID,
				METAVERSE_ID,
				COORDINATE_IN_1,
			))
		);

		assert_eq!(EstateModule::all_land_units_count(), 1);
		assert_noop!(
			EstateModule::mint_land(Origin::root(), BENEFICIARY_ID, METAVERSE_ID, COORDINATE_IN_1),
			Error::<Runtime>::LandUnitIsNotAvailable
		);
	});
}

#[test]
fn mint_lands_should_reject_with_duplicate_coordinates() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::mint_lands(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::NewLandsMinted(
				BENEFICIARY_ID,
				METAVERSE_ID,
				vec![COORDINATE_IN_1, COORDINATE_IN_2],
			))
		);

		assert_eq!(EstateModule::all_land_units_count(), 2);
		assert_noop!(
			EstateModule::mint_lands(Origin::root(), BENEFICIARY_ID, METAVERSE_ID, vec![COORDINATE_IN_1]),
			Error::<Runtime>::LandUnitIsNotAvailable
		);
	});
}

#[test]
fn mint_land_should_work_with_different_coordinate() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::mint_land(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			COORDINATE_IN_1
		));

		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::NewLandUnitMinted(
				OWNER_LAND_ASSET_ID,
				METAVERSE_ID,
				COORDINATE_IN_1,
			))
		);

		assert_eq!(EstateModule::all_land_units_count(), 1);

		assert_ok!(EstateModule::mint_land(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			COORDINATE_IN_2
		));

		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::NewLandUnitMinted(
				OWNER_LAND_ASSET_ID,
				METAVERSE_ID,
				COORDINATE_IN_2,
			))
		);

		assert_eq!(EstateModule::all_land_units_count(), 2);
	});
}

#[test]
fn mint_lands_should_reject_non_root() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EstateModule::mint_lands(
				Origin::signed(ALICE),
				BENEFICIARY_ID,
				METAVERSE_ID,
				vec![COORDINATE_IN_1, COORDINATE_IN_2]
			),
			BadOrigin
		);
	});
}

#[test]
fn mint_lands_should_work_with_one_coordinate() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::mint_lands(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1]
		));

		assert_eq!(
			EstateModule::get_user_land_units(&BENEFICIARY_ID, &METAVERSE_ID).len(),
			1
		);
		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::NewLandsMinted(
				BENEFICIARY_ID,
				METAVERSE_ID,
				vec![COORDINATE_IN_1],
			))
		);

		assert_eq!(EstateModule::all_land_units_count(), 1);
	});
}

#[test]
fn mint_lands_should_work_with_more_than_one_coordinate() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::mint_lands(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::NewLandsMinted(
				BENEFICIARY_ID,
				METAVERSE_ID,
				vec![COORDINATE_IN_1, COORDINATE_IN_2],
			))
		);

		assert_eq!(EstateModule::all_land_units_count(), 2);
	});
}

#[test]
fn transfer_land_token_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::mint_land(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			COORDINATE_IN_1
		));
		assert_eq!(
			EstateModule::get_land_units(METAVERSE_ID, COORDINATE_IN_1),
			Some(OWNER_LAND_ASSET_ID)
		);

		assert_ok!(EstateModule::transfer_land(
			Origin::signed(BENEFICIARY_ID),
			ALICE,
			METAVERSE_ID,
			COORDINATE_IN_1
		));

		assert_eq!(
			EstateModule::get_land_units(METAVERSE_ID, COORDINATE_IN_1),
			Some(OWNER_LAND_ASSET_ID)
		);

		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::TransferredLandUnit(
				METAVERSE_ID,
				COORDINATE_IN_1,
				BENEFICIARY_ID,
				ALICE,
			))
		);
	});
}

#[test]
fn transfer_land_should_reject_no_permission() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::mint_land(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			COORDINATE_IN_1
		));

		assert_eq!(
			EstateModule::get_land_units(METAVERSE_ID, COORDINATE_IN_1),
			Some(OWNER_LAND_ASSET_ID)
		);

		assert_noop!(
			EstateModule::transfer_land(Origin::signed(BOB), ALICE, METAVERSE_ID, COORDINATE_IN_1),
			Error::<Runtime>::NoPermission
		);
	});
}

#[test]
fn transfer_land_should_do_fail_for_same_account() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::mint_land(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			COORDINATE_IN_1
		));

		assert_eq!(
			EstateModule::get_land_units(METAVERSE_ID, COORDINATE_IN_1),
			Some(OWNER_LAND_ASSET_ID)
		);

		assert_noop!(
			EstateModule::transfer_land(
				Origin::signed(BENEFICIARY_ID),
				BENEFICIARY_ID,
				METAVERSE_ID,
				COORDINATE_IN_1
			),
			Error::<Runtime>::AlreadyOwnTheLandUnit
		);

		assert_eq!(
			EstateModule::get_land_units(METAVERSE_ID, COORDINATE_IN_1),
			Some(OWNER_LAND_ASSET_ID)
		);
	});
}

#[test]
fn transfer_land_should_do_fail_for_already_in_auction() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::mint_land(
			Origin::root(),
			AUCTION_BENEFICIARY_ID,
			METAVERSE_ID,
			COORDINATE_IN_AUCTION
		));
		assert_eq!(
			EstateModule::get_land_units(METAVERSE_ID, COORDINATE_IN_AUCTION),
			Some(OwnerId::Token(METAVERSE_LAND_CLASS, METAVERSE_LAND_IN_AUCTION_TOKEN))
		);

		assert_noop!(
			EstateModule::transfer_land(
				Origin::signed(AUCTION_BENEFICIARY_ID),
				BOB,
				METAVERSE_ID,
				COORDINATE_IN_AUCTION
			),
			Error::<Runtime>::LandUnitAlreadyInAuction
		);
	});
}

#[test]
fn mint_estate_should_reject_non_root() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EstateModule::mint_estate(
				Origin::signed(ALICE),
				BENEFICIARY_ID,
				METAVERSE_ID,
				vec![COORDINATE_IN_1, COORDINATE_IN_2]
			),
			BadOrigin
		);
	});
}

#[test]
fn mint_estate_should_fail_for_minted_land() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::mint_land(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			COORDINATE_IN_1
		));

		assert_err!(
			EstateModule::mint_estate(Origin::root(), BENEFICIARY_ID, METAVERSE_ID, vec![COORDINATE_IN_1]),
			Error::<Runtime>::LandUnitIsNotAvailable
		);
	});
}

#[test]
fn dissolve_estate_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		// Mint estate
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		let estate_id: u64 = 0;
		assert_eq!(EstateModule::all_estates_count(), 1);
		assert_eq!(EstateModule::next_estate_id(), 1);

		assert_eq!(
			EstateModule::get_estates(estate_id),
			Some(EstateInfo {
				metaverse_id: METAVERSE_ID,
				land_units: vec![COORDINATE_IN_1, COORDINATE_IN_2]
			})
		);
		assert_eq!(EstateModule::get_estate_owner(estate_id), Some(OWNER_ESTATE_ASSET_ID));

		assert_eq!(
			EstateModule::get_user_land_units(&BENEFICIARY_ID, &METAVERSE_ID).len(),
			2
		);

		// Destroy estate
		assert_ok!(EstateModule::dissolve_estate(Origin::signed(BENEFICIARY_ID), estate_id,));

		assert_eq!(EstateModule::all_estates_count(), 0);
		assert_eq!(EstateModule::get_estates(estate_id), None);
		assert_eq!(EstateModule::get_estate_owner(estate_id), None);
		assert_eq!(
			EstateModule::get_user_land_units(&BENEFICIARY_ID, &METAVERSE_ID).len(),
			2
		);
	});
}

#[test]
fn dissolve_estate_should_reject_non_owner() {
	ExtBuilder::default().build().execute_with(|| {
		// Mint estate
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		assert_err!(
			EstateModule::dissolve_estate(Origin::signed(ALICE), 0),
			Error::<Runtime>::NoPermission
		);
	});
}

#[test]
fn add_land_unit_to_estate_should_reject_non_owner() {
	ExtBuilder::default().build().execute_with(|| {
		// Mint estate
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_2]
		));

		assert_err!(
			EstateModule::add_land_unit_to_estate(Origin::signed(ALICE), 0, vec![COORDINATE_IN_2]),
			Error::<Runtime>::NoPermission
		);
	});
}

#[test]
fn add_land_unit_to_estate_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		// Mint estate
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1]
		));

		let estate_id: u64 = 0;
		assert_eq!(EstateModule::all_estates_count(), 1);
		assert_eq!(EstateModule::next_estate_id(), 1);
		assert_eq!(
			EstateModule::get_estates(estate_id),
			Some(EstateInfo {
				metaverse_id: METAVERSE_ID,
				land_units: vec![COORDINATE_IN_1]
			})
		);

		assert_eq!(
			EstateModule::get_user_land_units(&BENEFICIARY_ID, &METAVERSE_ID).len(),
			1
		);
		assert_eq!(EstateModule::get_estate_owner(estate_id), Some(OWNER_ESTATE_ASSET_ID));

		assert_eq!(EstateModule::all_land_units_count(), 1);

		assert_ok!(EstateModule::mint_land(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			COORDINATE_IN_2
		));
		// Update estate
		assert_ok!(EstateModule::add_land_unit_to_estate(
			Origin::signed(BENEFICIARY_ID),
			estate_id,
			vec![COORDINATE_IN_2]
		));

		assert_eq!(
			EstateModule::get_estates(estate_id),
			Some(EstateInfo {
				metaverse_id: METAVERSE_ID,
				land_units: vec![COORDINATE_IN_1, COORDINATE_IN_2]
			})
		);

		assert_eq!(
			EstateModule::get_user_land_units(&BENEFICIARY_ID, &METAVERSE_ID).len(),
			2
		);
		assert_eq!(EstateModule::all_land_units_count(), 2);
	});
}

#[test]
fn remove_land_unit_from_estate_should_reject_non_owner() {
	ExtBuilder::default().build().execute_with(|| {
		// Mint estate
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		assert_err!(
			EstateModule::remove_land_unit_from_estate(Origin::signed(ALICE), 0, vec![COORDINATE_IN_2]),
			Error::<Runtime>::NoPermission
		);
	});
}

#[test]
fn remove_land_unit_from_estate_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		// Mint estate
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		let estate_id: u64 = 0;
		assert_eq!(EstateModule::all_estates_count(), 1);
		assert_eq!(EstateModule::next_estate_id(), 1);
		assert_eq!(
			EstateModule::get_estates(estate_id),
			Some(EstateInfo {
				metaverse_id: METAVERSE_ID,
				land_units: vec![COORDINATE_IN_1, COORDINATE_IN_2]
			})
		);
		assert_eq!(EstateModule::get_estate_owner(estate_id), Some(OWNER_ESTATE_ASSET_ID));
		assert_eq!(
			EstateModule::get_user_land_units(&BENEFICIARY_ID, &METAVERSE_ID).len(),
			2
		);
		assert_eq!(EstateModule::all_land_units_count(), 2);

		// Update estate
		assert_ok!(EstateModule::remove_land_unit_from_estate(
			Origin::signed(BENEFICIARY_ID),
			estate_id,
			vec![COORDINATE_IN_2]
		));

		assert_eq!(
			EstateModule::get_estates(estate_id),
			Some(EstateInfo {
				metaverse_id: METAVERSE_ID,
				land_units: vec![COORDINATE_IN_1]
			})
		);
		assert_eq!(
			EstateModule::get_user_land_units(&BENEFICIARY_ID, &METAVERSE_ID).len(),
			2
		);
		assert_eq!(EstateModule::all_land_units_count(), 2);
	});
}

#[test]
fn mint_estate_and_land_should_return_correct_total_land_unit() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		let estate_id: u64 = 0;
		assert_eq!(EstateModule::all_estates_count(), 1);
		assert_eq!(EstateModule::next_estate_id(), 1);
		assert_eq!(
			EstateModule::get_estates(estate_id),
			Some(EstateInfo {
				metaverse_id: METAVERSE_ID,
				land_units: vec![COORDINATE_IN_1, COORDINATE_IN_2]
			})
		);
		assert_eq!(EstateModule::get_estate_owner(estate_id), Some(OWNER_ESTATE_ASSET_ID));

		assert_eq!(
			EstateModule::get_user_land_units(&BENEFICIARY_ID, &METAVERSE_ID).len(),
			2
		);

		assert_ok!(EstateModule::mint_land(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			(-6, 6)
		));
		assert_eq!(
			EstateModule::get_user_land_units(&BENEFICIARY_ID, &METAVERSE_ID).len(),
			3
		);
	});
}

#[test]
fn mint_estate_should_return_none_for_non_exist_estate() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		let estate_id: u64 = 0;
		assert_eq!(EstateModule::all_estates_count(), 1);
		assert_eq!(EstateModule::next_estate_id(), 1);
		assert_eq!(
			EstateModule::get_estates(estate_id),
			Some(EstateInfo {
				metaverse_id: METAVERSE_ID,
				land_units: vec![COORDINATE_IN_1, COORDINATE_IN_2]
			})
		);
		assert_eq!(EstateModule::get_estate_owner(estate_id), Some(OWNER_ESTATE_ASSET_ID));

		let estate_id_non_exists: u64 = 999;
		assert_eq!(EstateModule::get_estates(estate_id_non_exists), None);
		assert_eq!(EstateModule::get_estate_owner(estate_id_non_exists), None);
	});
}

#[test]
fn transfer_estate_token_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		let estate_id: u64 = 0;
		assert_eq!(EstateModule::get_estate_owner(estate_id), Some(OWNER_ESTATE_ASSET_ID));

		assert_ok!(EstateModule::transfer_estate(
			Origin::signed(BENEFICIARY_ID),
			ALICE,
			estate_id
		));
		assert_eq!(EstateModule::get_estate_owner(estate_id), Some(OWNER_ESTATE_ASSET_ID));

		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::TransferredEstate(estate_id, BENEFICIARY_ID, ALICE))
		);
	});
}

#[test]
fn transfer_estate_should_reject_no_permission() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		let estate_id: u64 = 0;
		assert_eq!(EstateModule::get_estate_owner(estate_id), Some(OWNER_ESTATE_ASSET_ID));

		assert_noop!(
			EstateModule::transfer_estate(Origin::signed(BOB), ALICE, estate_id),
			Error::<Runtime>::NoPermission
		);
	});
}

#[test]
fn transfer_estate_should_reject_already_in_auction() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1]
		));
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_2]
		));
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_3]
		));
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			AUCTION_BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_AUCTION]
		));
		assert_noop!(
			EstateModule::transfer_estate(Origin::signed(AUCTION_BENEFICIARY_ID), ALICE, ESTATE_IN_AUCTION),
			Error::<Runtime>::EstateAlreadyInAuction
		);
	});
}

#[test]
fn transfer_estate_should_fail_with_same_account() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		let estate_id: u64 = 0;
		assert_eq!(EstateModule::get_estate_owner(estate_id), Some(OWNER_ESTATE_ASSET_ID));

		assert_noop!(
			EstateModule::transfer_estate(Origin::signed(BENEFICIARY_ID), BENEFICIARY_ID, estate_id),
			Error::<Runtime>::AlreadyOwnTheEstate
		);

		assert_eq!(EstateModule::get_estate_owner(estate_id), Some(OWNER_ESTATE_ASSET_ID));
	});
}

#[test]
fn create_estate_token_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::mint_lands(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		assert_ok!(EstateModule::create_estate(
			Origin::signed(BENEFICIARY_ID),
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		let estate_id: u64 = 0;
		assert_eq!(EstateModule::all_estates_count(), 1);
		assert_eq!(EstateModule::next_estate_id(), 1);
		assert_eq!(
			EstateModule::get_estates(estate_id),
			Some(EstateInfo {
				metaverse_id: METAVERSE_ID,
				land_units: vec![COORDINATE_IN_1, COORDINATE_IN_2]
			})
		);
		assert_eq!(EstateModule::get_estate_owner(estate_id), Some(OWNER_ESTATE_ASSET_ID));
		assert_eq!(Balances::free_balance(BENEFICIARY_ID), 999999);
	});
}

#[test]
fn create_estate_token_after_minting_account_and_token_based_lands_should_give_correct_total_user_land_units() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::mint_land(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			COORDINATE_IN_1
		));

		assert_ok!(EstateModule::mint_land(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			COORDINATE_IN_2
		));

		assert_ok!(EstateModule::create_estate(
			Origin::signed(BENEFICIARY_ID),
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		let estate_id: u64 = 0;
		assert_eq!(EstateModule::all_estates_count(), 1);
		assert_eq!(EstateModule::next_estate_id(), 1);
		assert_eq!(
			EstateModule::get_estates(estate_id),
			Some(EstateInfo {
				metaverse_id: METAVERSE_ID,
				land_units: vec![COORDINATE_IN_1, COORDINATE_IN_2]
			})
		);
		assert_eq!(EstateModule::get_estate_owner(estate_id), Some(OWNER_ESTATE_ASSET_ID));
		assert_eq!(
			EstateModule::get_user_land_units(&BENEFICIARY_ID, &METAVERSE_ID).len(),
			2
		);
		assert_eq!(EstateModule::all_land_units_count(), 2);
		assert_eq!(Balances::free_balance(BENEFICIARY_ID), 999999);
	});
}

#[test]
fn create_estate_should_return_none_for_non_exist_estate() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::mint_lands(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		assert_ok!(EstateModule::create_estate(
			Origin::signed(BENEFICIARY_ID),
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));
		assert_eq!(Balances::free_balance(BENEFICIARY_ID), 999999);

		let estate_id: u64 = 0;
		assert_eq!(EstateModule::all_estates_count(), 1);
		assert_eq!(EstateModule::next_estate_id(), 1);
		assert_eq!(
			EstateModule::get_estates(estate_id),
			Some(EstateInfo {
				metaverse_id: METAVERSE_ID,
				land_units: vec![COORDINATE_IN_1, COORDINATE_IN_2]
			})
		);
		assert_eq!(EstateModule::get_estate_owner(estate_id), Some(OWNER_ESTATE_ASSET_ID));

		let estate_id_non_exists: u64 = 999;
		assert_eq!(EstateModule::get_estates(estate_id_non_exists), None);
		assert_eq!(EstateModule::get_estate_owner(estate_id_non_exists), None);
	});
}

#[test]
fn issue_land_block_should_fail_if_not_root() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EstateModule::issue_undeployed_land_blocks(
				Origin::signed(ALICE),
				BOB,
				1,
				20,
				UndeployedLandBlockType::BoundToAddress
			),
			BadOrigin
		);
	});
}

#[test]
fn issue_land_block_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			20,
			UndeployedLandBlockType::BoundToAddress
		));

		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::UndeployedLandBlockIssued(BOB, 0))
		);

		assert_eq!(EstateModule::get_undeployed_land_block_owner(BOB, 0), Some(()));

		let issued_undeployed_land_block = EstateModule::get_undeployed_land_block(0);
		match issued_undeployed_land_block {
			Some(a) => {
				// Verify details of UndeployedLandBlock
				assert_eq!(a.owner, BOB);
				assert_eq!(a.number_land_units, 20);
				assert_eq!(a.undeployed_land_block_type, UndeployedLandBlockType::BoundToAddress);
				assert_eq!(a.is_locked, false);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	});
}

#[test]
fn issue_two_land_block_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			20,
			UndeployedLandBlockType::BoundToAddress
		));

		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::UndeployedLandBlockIssued(BOB, 0))
		);

		assert_eq!(EstateModule::get_undeployed_land_block_owner(BOB, 0), Some(()));

		let first_issued_undeployed_land_block = EstateModule::get_undeployed_land_block(0);
		match first_issued_undeployed_land_block {
			Some(a) => {
				// Verify details of UndeployedLandBlock
				assert_eq!(a.owner, BOB);
				assert_eq!(a.number_land_units, 20);
				assert_eq!(a.undeployed_land_block_type, UndeployedLandBlockType::BoundToAddress);
				assert_eq!(a.is_locked, false);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}

		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			ALICE,
			1,
			30,
			UndeployedLandBlockType::Transferable
		));

		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::UndeployedLandBlockIssued(ALICE, 1))
		);

		assert_eq!(EstateModule::get_undeployed_land_block_owner(ALICE, 1), Some(()));

		let second_issued_undeployed_land_block = EstateModule::get_undeployed_land_block(1);
		match second_issued_undeployed_land_block {
			Some(a) => {
				// Verify details of UndeployedLandBlock
				assert_eq!(a.owner, ALICE);
				assert_eq!(a.number_land_units, 30);
				assert_eq!(a.undeployed_land_block_type, UndeployedLandBlockType::Transferable);
				assert_eq!(a.is_locked, false);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	});
}

#[test]
fn freeze_undeployed_land_block_should_fail_if_not_root() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EstateModule::freeze_undeployed_land_blocks(Origin::signed(ALICE), 0),
			BadOrigin
		);
	});
}

#[test]
fn freeze_undeployed_land_block_should_fail_not_found() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EstateModule::freeze_undeployed_land_blocks(Origin::root(), 0),
			Error::<Runtime>::UndeployedLandBlockNotFound
		);
	});
}

#[test]
fn freeze_undeployed_land_block_should_fail_if_already_in_auction() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			1,
			UndeployedLandBlockType::Transferable,
		));

		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			21,
			UndeployedLandBlockType::Transferable,
		));

		let issued_undeployed_land_block = EstateModule::get_undeployed_land_block(UNDEPLOYED_LAND_BLOCK_IN_AUCTION);
		match issued_undeployed_land_block {
			Some(a) => {
				// Verify details of UndeployedLandBlock
				assert_eq!(a.owner, BOB);
				assert_eq!(a.number_land_units, 21);
				assert_eq!(a.undeployed_land_block_type, UndeployedLandBlockType::Transferable);
				assert_eq!(a.is_locked, false);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
		assert_noop!(
			EstateModule::freeze_undeployed_land_blocks(Origin::root(), UNDEPLOYED_LAND_BLOCK_IN_AUCTION),
			Error::<Runtime>::UndeployedLandBlockAlreadyInAuction
		);
	});
}

#[test]
fn freeze_undeployed_land_block_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			20,
			UndeployedLandBlockType::BoundToAddress
		));

		let issued_undeployed_land_block = EstateModule::get_undeployed_land_block(0);
		match issued_undeployed_land_block {
			Some(a) => {
				// Verify details of UndeployedLandBlock
				assert_eq!(a.owner, BOB);
				assert_eq!(a.number_land_units, 20);
				assert_eq!(a.undeployed_land_block_type, UndeployedLandBlockType::BoundToAddress);
				assert_eq!(a.is_locked, false);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}

		assert_ok!(EstateModule::freeze_undeployed_land_blocks(Origin::root(), 0));

		assert_eq!(last_event(), Event::Estate(crate::Event::UndeployedLandBlockFreezed(0)));

		assert_eq!(EstateModule::get_undeployed_land_block_owner(BOB, 0), Some(()));

		let frozen_undeployed_land_block = EstateModule::get_undeployed_land_block(0);
		match frozen_undeployed_land_block {
			Some(a) => {
				assert_eq!(a.is_locked, true);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	});
}

#[test]
fn freeze_undeployed_land_block_should_fail_already_freezed() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			20,
			UndeployedLandBlockType::BoundToAddress
		));

		assert_ok!(EstateModule::freeze_undeployed_land_blocks(Origin::root(), 0));

		assert_eq!(last_event(), Event::Estate(crate::Event::UndeployedLandBlockFreezed(0)));

		assert_noop!(
			EstateModule::freeze_undeployed_land_blocks(Origin::root(), 0),
			Error::<Runtime>::UndeployedLandBlockAlreadyFreezed
		);
	});
}

#[test]
fn unfreeze_undeployed_land_block_should_fail_if_not_root() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EstateModule::unfreeze_undeployed_land_blocks(Origin::signed(ALICE), 0),
			BadOrigin
		);
	});
}

#[test]
fn unfreeze_undeployed_land_block_should_fail_not_found() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EstateModule::unfreeze_undeployed_land_blocks(Origin::root(), 0),
			Error::<Runtime>::UndeployedLandBlockNotFound
		);
	});
}

#[test]
fn unfreeze_undeployed_land_block_should_fail_if_already_in_auction() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			1,
			UndeployedLandBlockType::Transferable,
		));

		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			21,
			UndeployedLandBlockType::Transferable,
		));

		let issued_undeployed_land_block = EstateModule::get_undeployed_land_block(1);
		match issued_undeployed_land_block {
			Some(a) => {
				// Verify details of UndeployedLandBlock
				assert_eq!(a.owner, BOB);
				assert_eq!(a.number_land_units, 21);
				assert_eq!(a.undeployed_land_block_type, UndeployedLandBlockType::Transferable);
				assert_eq!(a.is_locked, false);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}

		assert_noop!(
			EstateModule::unfreeze_undeployed_land_blocks(Origin::root(), UNDEPLOYED_LAND_BLOCK_IN_AUCTION),
			Error::<Runtime>::UndeployedLandBlockAlreadyInAuction
		);
	});
}

#[test]
fn unfreeze_undeployed_land_block_should_fail_not_frozen() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			20,
			UndeployedLandBlockType::BoundToAddress
		));

		assert_noop!(
			EstateModule::unfreeze_undeployed_land_blocks(Origin::root(), 0),
			Error::<Runtime>::UndeployedLandBlockNotFrozen
		);
	});
}

#[test]
fn unfreeze_undeployed_land_block_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			20,
			UndeployedLandBlockType::BoundToAddress
		));

		assert_ok!(EstateModule::freeze_undeployed_land_blocks(Origin::root(), 0));

		let freezed_undeployed_land_block = EstateModule::get_undeployed_land_block(0);
		match freezed_undeployed_land_block {
			Some(a) => {
				assert_eq!(a.is_locked, true);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}

		assert_ok!(EstateModule::unfreeze_undeployed_land_blocks(Origin::root(), 0));

		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::UndeployedLandBlockUnfreezed(0))
		);

		let unfreezed_undeployed_land_block = EstateModule::get_undeployed_land_block(0);
		match unfreezed_undeployed_land_block {
			Some(a) => {
				assert_eq!(a.is_locked, false);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	});
}

#[test]
fn transfer_undeployed_land_block_should_fail_if_not_found() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EstateModule::transfer_undeployed_land_blocks(Origin::signed(ALICE), BOB, 0),
			Error::<Runtime>::UndeployedLandBlockNotFound
		);
	});
}

#[test]
fn transfer_undeployed_land_block_should_fail_if_not_owner() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			20,
			UndeployedLandBlockType::BoundToAddress
		));

		let undeployed_land_block_id: UndeployedLandBlockId = 0;

		assert_noop!(
			EstateModule::transfer_undeployed_land_blocks(Origin::signed(ALICE), BOB, undeployed_land_block_id),
			Error::<Runtime>::NoPermission
		);
	});
}

#[test]
fn transfer_undeployed_land_block_should_fail_if_freezed() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			20,
			UndeployedLandBlockType::BoundToAddress
		));

		let undeployed_land_block_id: UndeployedLandBlockId = 0;

		assert_ok!(EstateModule::freeze_undeployed_land_blocks(
			Origin::root(),
			undeployed_land_block_id
		));

		assert_noop!(
			EstateModule::transfer_undeployed_land_blocks(Origin::signed(BOB), ALICE, undeployed_land_block_id),
			Error::<Runtime>::UndeployedLandBlockAlreadyFreezed
		);
	});
}

#[test]
fn transfer_undeployed_land_block_should_fail_if_not_transferable() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			20,
			UndeployedLandBlockType::BoundToAddress
		));

		let undeployed_land_block_id: UndeployedLandBlockId = 0;

		assert_noop!(
			EstateModule::transfer_undeployed_land_blocks(Origin::signed(BOB), ALICE, undeployed_land_block_id),
			Error::<Runtime>::UndeployedLandBlockIsNotTransferable
		);
	});
}

#[test]
fn transfer_undeployed_land_block_should_fail_if_already_in_auction() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			1,
			UndeployedLandBlockType::Transferable,
		));

		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			21,
			UndeployedLandBlockType::Transferable,
		));

		let issued_undeployed_land_block = EstateModule::get_undeployed_land_block(1);
		match issued_undeployed_land_block {
			Some(a) => {
				// Verify details of UndeployedLandBlock
				assert_eq!(a.owner, BOB);
				assert_eq!(a.number_land_units, 21);
				assert_eq!(a.undeployed_land_block_type, UndeployedLandBlockType::Transferable);
				assert_eq!(a.is_locked, false);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}

		assert_noop!(
			EstateModule::transfer_undeployed_land_blocks(Origin::signed(BOB), ALICE, UNDEPLOYED_LAND_BLOCK_IN_AUCTION),
			Error::<Runtime>::UndeployedLandBlockAlreadyInAuction
		);
	});
}

#[test]
fn transfer_undeployed_land_block_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			20,
			UndeployedLandBlockType::Transferable
		));

		let undeployed_land_block_id: UndeployedLandBlockId = 0;

		let issued_undeployed_land_block = EstateModule::get_undeployed_land_block(undeployed_land_block_id);
		match issued_undeployed_land_block {
			Some(a) => {
				assert_eq!(a.owner, BOB);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}

		assert_eq!(
			EstateModule::get_undeployed_land_block_owner(BOB, undeployed_land_block_id),
			Some(())
		);

		assert_ok!(EstateModule::transfer_undeployed_land_blocks(
			Origin::signed(BOB),
			ALICE,
			undeployed_land_block_id
		));

		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::UndeployedLandBlockTransferred(
				BOB,
				ALICE,
				undeployed_land_block_id,
			))
		);

		let transferred_issued_undeployed_land_block =
			EstateModule::get_undeployed_land_block(undeployed_land_block_id);
		match transferred_issued_undeployed_land_block {
			Some(a) => {
				assert_eq!(a.owner, ALICE);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}

		assert_eq!(
			EstateModule::get_undeployed_land_block_owner(BOB, undeployed_land_block_id),
			None
		);
		assert_eq!(
			EstateModule::get_undeployed_land_block_owner(ALICE, undeployed_land_block_id),
			Some(())
		);
	});
}

#[test]
fn deploy_undeployed_land_block_should_fail_if_not_found() {
	ExtBuilder::default().build().execute_with(|| {
		let undeployed_land_block_id: UndeployedLandBlockId = 0;

		assert_noop!(
			EstateModule::deploy_land_block(
				Origin::signed(ALICE),
				undeployed_land_block_id,
				ALICE_METAVERSE_ID,
				LANDBLOCK_COORDINATE,
				vec![COORDINATE_IN_1]
			),
			Error::<Runtime>::UndeployedLandBlockNotFound
		);
		assert_eq!(Balances::free_balance(BOB), 100000);
	});
}

#[test]
fn deploy_undeployed_land_block_should_fail_if_not_owner() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			20,
			UndeployedLandBlockType::BoundToAddress
		));

		let undeployed_land_block_id: UndeployedLandBlockId = 0;

		assert_noop!(
			EstateModule::deploy_land_block(
				Origin::signed(ALICE),
				undeployed_land_block_id,
				METAVERSE_ID,
				LANDBLOCK_COORDINATE,
				vec![COORDINATE_IN_1]
			),
			Error::<Runtime>::NoPermission
		);
		assert_eq!(Balances::free_balance(ALICE), 100000);
	});
}

#[test]
fn deploy_undeployed_land_block_should_fail_if_freezed() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			20,
			UndeployedLandBlockType::BoundToAddress
		));

		let undeployed_land_block_id: UndeployedLandBlockId = 0;

		assert_ok!(EstateModule::freeze_undeployed_land_blocks(
			Origin::root(),
			undeployed_land_block_id
		));

		assert_noop!(
			EstateModule::deploy_land_block(
				Origin::signed(BOB),
				undeployed_land_block_id,
				BOB_METAVERSE_ID,
				LANDBLOCK_COORDINATE,
				vec![COORDINATE_IN_1]
			),
			Error::<Runtime>::UndeployedLandBlockFreezed
		);
		assert_eq!(Balances::free_balance(BOB), 100000);
	});
}

#[test]
fn deploy_undeployed_land_block_should_fail_if_already_in_auction() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			1,
			UndeployedLandBlockType::Transferable,
		));

		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			21,
			UndeployedLandBlockType::Transferable,
		));

		let issued_undeployed_land_block = EstateModule::get_undeployed_land_block(1);
		match issued_undeployed_land_block {
			Some(a) => {
				// Verify details of UndeployedLandBlock
				assert_eq!(a.owner, BOB);
				assert_eq!(a.number_land_units, 21);
				assert_eq!(a.undeployed_land_block_type, UndeployedLandBlockType::Transferable);
				assert_eq!(a.is_locked, false);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}

		assert_noop!(
			EstateModule::deploy_land_block(
				Origin::signed(BOB),
				UNDEPLOYED_LAND_BLOCK_IN_AUCTION,
				METAVERSE_ID,
				LANDBLOCK_COORDINATE,
				vec![COORDINATE_IN_1, COORDINATE_IN_2]
			),
			Error::<Runtime>::UndeployedLandBlockAlreadyInAuction
		);
		assert_eq!(Balances::free_balance(BOB), 100000);
	});
}

#[test]
fn deploy_undeployed_land_block_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			2,
			UndeployedLandBlockType::BoundToAddress
		));

		let undeployed_land_block_id: UndeployedLandBlockId = 0;

		let undeployed_land_block = EstateModule::get_undeployed_land_block(undeployed_land_block_id);
		match undeployed_land_block {
			Some(a) => {
				assert_eq!(a.number_land_units, 2);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}

		assert_ok!(EstateModule::deploy_land_block(
			Origin::signed(BOB),
			undeployed_land_block_id,
			BOB_METAVERSE_ID,
			LANDBLOCK_COORDINATE,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::LandBlockDeployed(
				BOB,
				BOB_METAVERSE_ID,
				undeployed_land_block_id,
				vec![COORDINATE_IN_1, COORDINATE_IN_2],
			))
		);

		let updated_undeployed_land_block = EstateModule::get_undeployed_land_block(undeployed_land_block_id);

		assert_eq!(updated_undeployed_land_block, None);

		assert_eq!(EstateModule::all_land_units_count(), 2);
		assert_eq!(Balances::free_balance(BOB), 99999);
	});
}

#[test]
fn approve_undeployed_land_block_should_fail_if_not_found() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EstateModule::approve_undeployed_land_blocks(Origin::signed(ALICE), BOB, 0),
			Error::<Runtime>::UndeployedLandBlockNotFound
		);
	});
}

#[test]
fn approve_undeployed_land_block_should_fail_if_not_owner() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			20,
			UndeployedLandBlockType::BoundToAddress
		));

		let undeployed_land_block_id: UndeployedLandBlockId = 0;

		assert_noop!(
			EstateModule::approve_undeployed_land_blocks(Origin::signed(ALICE), BOB, undeployed_land_block_id),
			Error::<Runtime>::NoPermission
		);
	});
}

#[test]
fn approve_undeployed_land_block_should_fail_if_freezed() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			20,
			UndeployedLandBlockType::BoundToAddress
		));

		let undeployed_land_block_id: UndeployedLandBlockId = 0;

		assert_ok!(EstateModule::freeze_undeployed_land_blocks(
			Origin::root(),
			undeployed_land_block_id
		));

		assert_noop!(
			EstateModule::approve_undeployed_land_blocks(Origin::signed(BOB), ALICE, undeployed_land_block_id),
			Error::<Runtime>::UndeployedLandBlockAlreadyFreezed
		);
	});
}

#[test]
fn approve_undeployed_land_block_should_fail_if_already_in_auction() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			1,
			UndeployedLandBlockType::Transferable,
		));

		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			21,
			UndeployedLandBlockType::Transferable,
		));

		let issued_undeployed_land_block = EstateModule::get_undeployed_land_block(1);
		match issued_undeployed_land_block {
			Some(a) => {
				// Verify details of UndeployedLandBlock
				assert_eq!(a.owner, BOB);
				assert_eq!(a.number_land_units, 21);
				assert_eq!(a.undeployed_land_block_type, UndeployedLandBlockType::Transferable);
				assert_eq!(a.is_locked, false);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}

		assert_noop!(
			EstateModule::approve_undeployed_land_blocks(Origin::signed(BOB), ALICE, UNDEPLOYED_LAND_BLOCK_IN_AUCTION),
			Error::<Runtime>::UndeployedLandBlockAlreadyInAuction
		);
	});
}

#[test]
fn approve_undeployed_land_block_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			20,
			UndeployedLandBlockType::Transferable
		));

		let undeployed_land_block_id: UndeployedLandBlockId = 0;

		let issued_undeployed_land_block = EstateModule::get_undeployed_land_block(undeployed_land_block_id);
		match issued_undeployed_land_block {
			Some(a) => {
				assert_eq!(a.owner, BOB);
				assert_eq!(a.approved, None);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}

		assert_eq!(
			EstateModule::get_undeployed_land_block_owner(BOB, undeployed_land_block_id),
			Some(())
		);

		assert_ok!(EstateModule::approve_undeployed_land_blocks(
			Origin::signed(BOB),
			ALICE,
			undeployed_land_block_id
		));

		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::UndeployedLandBlockApproved(
				BOB,
				ALICE,
				undeployed_land_block_id,
			))
		);

		let transferred_issued_undeployed_land_block =
			EstateModule::get_undeployed_land_block(undeployed_land_block_id);
		match transferred_issued_undeployed_land_block {
			Some(a) => {
				assert_eq!(a.owner, BOB);
				assert_eq!(a.approved, Some(ALICE));
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}

		assert_eq!(
			EstateModule::get_undeployed_land_block_owner(BOB, undeployed_land_block_id),
			Some(())
		);
	});
}

#[test]
fn unapprove_undeployed_land_block_should_fail_if_not_found() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EstateModule::unapprove_undeployed_land_blocks(Origin::signed(ALICE), 0),
			Error::<Runtime>::UndeployedLandBlockNotFound
		);
	});
}

#[test]
fn unapprove_undeployed_land_block_should_fail_if_not_owner() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			20,
			UndeployedLandBlockType::BoundToAddress
		));

		let undeployed_land_block_id: UndeployedLandBlockId = 0;

		assert_noop!(
			EstateModule::unapprove_undeployed_land_blocks(Origin::signed(ALICE), undeployed_land_block_id),
			Error::<Runtime>::NoPermission
		);
	});
}

#[test]
fn unapprove_undeployed_land_block_should_fail_if_freezed() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			20,
			UndeployedLandBlockType::BoundToAddress
		));

		let undeployed_land_block_id: UndeployedLandBlockId = 0;

		assert_ok!(EstateModule::freeze_undeployed_land_blocks(
			Origin::root(),
			undeployed_land_block_id
		));

		assert_noop!(
			EstateModule::unapprove_undeployed_land_blocks(Origin::signed(BOB), undeployed_land_block_id),
			Error::<Runtime>::UndeployedLandBlockAlreadyFreezed
		);
	});
}

#[test]
fn unapprove_undeployed_land_block_should_fail_if_already_in_auction() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			1,
			UndeployedLandBlockType::Transferable,
		));

		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			21,
			UndeployedLandBlockType::Transferable,
		));

		let issued_undeployed_land_block = EstateModule::get_undeployed_land_block(1);
		match issued_undeployed_land_block {
			Some(a) => {
				// Verify details of UndeployedLandBlock
				assert_eq!(a.owner, BOB);
				assert_eq!(a.number_land_units, 21);
				assert_eq!(a.undeployed_land_block_type, UndeployedLandBlockType::Transferable);
				assert_eq!(a.is_locked, false);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}

		assert_noop!(
			EstateModule::unapprove_undeployed_land_blocks(Origin::signed(BOB), UNDEPLOYED_LAND_BLOCK_IN_AUCTION),
			Error::<Runtime>::UndeployedLandBlockAlreadyInAuction
		);
	});
}

#[test]
fn unapprove_undeployed_land_block_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			20,
			UndeployedLandBlockType::Transferable
		));

		let undeployed_land_block_id: UndeployedLandBlockId = 0;

		let issued_undeployed_land_block = EstateModule::get_undeployed_land_block(undeployed_land_block_id);
		match issued_undeployed_land_block {
			Some(a) => {
				assert_eq!(a.owner, BOB);
				assert_eq!(a.approved, None);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}

		assert_eq!(
			EstateModule::get_undeployed_land_block_owner(BOB, undeployed_land_block_id),
			Some(())
		);
		assert_ok!(EstateModule::approve_undeployed_land_blocks(
			Origin::signed(BOB),
			ALICE,
			undeployed_land_block_id
		));

		let approved_issued_undeployed_land_block = EstateModule::get_undeployed_land_block(undeployed_land_block_id);
		match approved_issued_undeployed_land_block {
			Some(a) => {
				assert_eq!(a.owner, BOB);
				assert_eq!(a.approved, Some(ALICE));
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}

		assert_ok!(EstateModule::unapprove_undeployed_land_blocks(
			Origin::signed(BOB),
			undeployed_land_block_id
		));

		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::UndeployedLandBlockUnapproved(undeployed_land_block_id))
		);

		let unapproved_issued_undeployed_land_block = EstateModule::get_undeployed_land_block(undeployed_land_block_id);
		match unapproved_issued_undeployed_land_block {
			Some(a) => {
				assert_eq!(a.owner, BOB);
				assert_eq!(a.approved, None);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	});
}

#[test]
fn burn_undeployed_land_block_should_fail_if_not_root() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EstateModule::burn_undeployed_land_blocks(Origin::signed(ALICE), 0),
			BadOrigin
		);
	});
}

#[test]
fn burn_undeployed_land_block_should_fail_not_found() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EstateModule::burn_undeployed_land_blocks(Origin::root(), 0),
			Error::<Runtime>::UndeployedLandBlockNotFound
		);
	});
}

#[test]
fn burn_undeployed_land_block_should_fail_if_already_in_auction() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			1,
			UndeployedLandBlockType::Transferable,
		));

		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			21,
			UndeployedLandBlockType::Transferable,
		));

		let issued_undeployed_land_block = EstateModule::get_undeployed_land_block(1);
		match issued_undeployed_land_block {
			Some(a) => {
				// Verify details of UndeployedLandBlock
				assert_eq!(a.owner, BOB);
				assert_eq!(a.number_land_units, 21);
				assert_eq!(a.undeployed_land_block_type, UndeployedLandBlockType::Transferable);
				assert_eq!(a.is_locked, false);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}

		assert_noop!(
			EstateModule::burn_undeployed_land_blocks(Origin::root(), UNDEPLOYED_LAND_BLOCK_IN_AUCTION),
			Error::<Runtime>::UndeployedLandBlockAlreadyInAuction
		);
	});
}

#[test]
fn burn_undeployed_land_block_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			20,
			UndeployedLandBlockType::BoundToAddress
		));

		let undeployed_land_block_id: UndeployedLandBlockId = 0;

		let issued_undeployed_land_block = EstateModule::get_undeployed_land_block(undeployed_land_block_id);
		match issued_undeployed_land_block {
			Some(a) => {
				assert_eq!(a.owner, BOB);
				assert_eq!(a.number_land_units, 20);
				assert_eq!(a.undeployed_land_block_type, UndeployedLandBlockType::BoundToAddress);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}

		assert_eq!(
			EstateModule::get_undeployed_land_block_owner(BOB, undeployed_land_block_id),
			Some(())
		);

		assert_ok!(EstateModule::burn_undeployed_land_blocks(
			Origin::root(),
			undeployed_land_block_id
		));

		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::UndeployedLandBlockBurnt(undeployed_land_block_id))
		);

		assert_eq!(
			EstateModule::get_undeployed_land_block_owner(BOB, undeployed_land_block_id),
			None
		);

		assert_eq!(EstateModule::get_undeployed_land_block(undeployed_land_block_id), None)
	});
}

#[test]
fn ensure_land_unit_within_land_block_bound_should_work() {
	//	let coordinates: Vec<(i32, i32)> = vec![(-4, 0), (-3, 0), (-3, 0), (0, 5)];
	//	assert_eq!(EstateModule::verify_land_unit_in_bound(&(0, 0), &coordinates), true);

	let second_coordinates: Vec<(i32, i32)> = vec![(-204, 25), (-203, 24), (-195, 20), (-197, 16)];
	assert_eq!(
		EstateModule::verify_land_unit_in_bound(&(-20, 2), &second_coordinates),
		true
	);

	let third_coordinates: Vec<(i32, i32)> = vec![(-64, 5), (-64, 4), (-64, 4), (-55, -4)];
	assert_eq!(
		EstateModule::verify_land_unit_in_bound(&(-6, 0), &third_coordinates),
		true
	);

	// Combined in and out bound should fail
	let fourth_coordinates: Vec<(i32, i32)> = vec![(-5, 3), (-4, 6), (-5, 4)];
	assert_eq!(
		EstateModule::verify_land_unit_in_bound(&(0, 0), &fourth_coordinates),
		false
	);
}

#[test]
fn ensure_land_unit_out_of_land_block_bound_should_fail() {
	let coordinates: Vec<(i32, i32)> = vec![(-51, 0), (-48, 0), (-47, 0), (0, 51)];
	assert_eq!(EstateModule::verify_land_unit_in_bound(&(0, 0), &coordinates), false);

	let second_coordinates: Vec<(i32, i32)> = vec![(-250, 2), (-248, 2), (-150, 2), (-151, 6)];
	assert_eq!(
		EstateModule::verify_land_unit_in_bound(&(-200, 2), &second_coordinates),
		false
	);
}

#[test]
fn issue_land_block_and_create_estate_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			2,
			UndeployedLandBlockType::BoundToAddress
		));

		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::UndeployedLandBlockIssued(BOB, 0))
		);

		assert_eq!(EstateModule::get_undeployed_land_block_owner(BOB, 0), Some(()));

		let issued_undeployed_land_block = EstateModule::get_undeployed_land_block(0);
		match issued_undeployed_land_block {
			Some(a) => {
				// Verify details of UndeployedLandBlock
				assert_eq!(a.owner, BOB);
				assert_eq!(a.number_land_units, 2);
				assert_eq!(a.undeployed_land_block_type, UndeployedLandBlockType::BoundToAddress);
				assert_eq!(a.is_locked, false);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}

		// Bob can deploy raw land block to his metaverse
		assert_ok!(EstateModule::deploy_land_block(
			Origin::signed(BOB),
			0,
			BOB_METAVERSE_ID,
			LANDBLOCK_COORDINATE,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));
		assert_eq!(Balances::free_balance(BOB), 99999);

		assert_eq!(
			EstateModule::get_land_units(BOB_METAVERSE_ID, COORDINATE_IN_1),
			Some(OwnerId::Token(METAVERSE_LAND_CLASS, 2))
		);

		assert_eq!(
			EstateModule::get_land_units(BOB_METAVERSE_ID, COORDINATE_IN_2),
			Some(OwnerId::Token(METAVERSE_LAND_CLASS, 2))
		);
	});
}

#[test]
fn create_estate_lease_offer_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		// Mint estate
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1]
		));

		assert_noop!(
			EstateModule::create_lease_offer(Origin::signed(ALICE), 1u64, 10u128, 8u32),
			Error::<Runtime>::EstateDoesNotExist
		);

		assert_noop!(
			EstateModule::create_lease_offer(Origin::signed(ALICE), 0u64, 0u128, 8u32),
			Error::<Runtime>::LeaseOfferPriceBelowMinimum
		);

		assert_noop!(
			EstateModule::create_lease_offer(Origin::signed(ALICE), 0u64, 2u128, 1000u32),
			Error::<Runtime>::LeaseOfferDurationAboveMaximum
		);

		assert_ok!(EstateModule::create_lease_offer(
			Origin::signed(ALICE),
			0u64,
			10u128,
			8u32
		));

		assert_ok!(EstateModule::create_lease_offer(
			Origin::signed(BOB),
			0u64,
			11u128,
			8u32
		));

		assert_noop!(
			EstateModule::create_lease_offer(Origin::signed(CHARLIE), 0u64, 12u128, 8u32),
			Error::<Runtime>::EstateLeaseOffersQueueLimitIsReached
		);

		assert_ok!(EstateModule::accept_lease_offer(
			Origin::signed(BENEFICIARY_ID),
			0u64,
			ALICE
		));

		assert_noop!(
			EstateModule::create_lease_offer(Origin::signed(CHARLIE), 0u64, 12u128, 8u32),
			Error::<Runtime>::EstateIsAlreadyLeased
		);

		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			AUCTION_BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_2]
		));

		assert_noop!(
			EstateModule::create_lease_offer(Origin::signed(ALICE), 1u64, 3u128, 8u32),
			Error::<Runtime>::EstateAlreadyInAuction
		);

		assert_noop!(
			EstateModule::create_lease_offer(Origin::signed(BENEFICIARY_ID), 0u64, 10u128, 8u32),
			Error::<Runtime>::NoPermission
		);
	});
}

#[test]
fn create_estate_lease_offer_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		assert_ok!(EstateModule::create_lease_offer(
			Origin::signed(ALICE),
			0u64,
			10u128,
			8u32
		));

		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::EstateLeaseOfferCreated(ALICE, 0, 80))
		);

		let lease_contract = LeaseContract {
			price_per_block: 10u128,
			duration: 8u32,
			end_block: 7,
			start_block: 8,
			unclaimed_rent: 80u128,
		};

		assert_eq!(EstateModule::lease_offers(0u64, ALICE), Some(lease_contract));

		assert_eq!(Balances::free_balance(ALICE), 99920);
	});
}

#[test]
fn accept_estate_lease_offer_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		// Mint estate
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1]
		));

		assert_ok!(EstateModule::create_lease_offer(
			Origin::signed(BOB),
			0u64,
			10u128,
			8u32
		));

		assert_noop!(
			EstateModule::accept_lease_offer(Origin::signed(ALICE), 0u64, BOB),
			Error::<Runtime>::NoPermission
		);
		//TO DO: Offer cannot be accepted after asset is listed on auction

		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_2]
		));

		assert_ok!(EstateModule::create_lease_offer(
			Origin::signed(BOB),
			1u64,
			10u128,
			8u32
		));

		assert_ok!(EstateModule::create_lease_offer(
			Origin::signed(ALICE),
			1u64,
			10u128,
			8u32
		));

		assert_ok!(EstateModule::accept_lease_offer(
			Origin::signed(BENEFICIARY_ID),
			1u64,
			ALICE
		));

		assert_noop!(
			EstateModule::accept_lease_offer(Origin::signed(BENEFICIARY_ID), 1u64, BOB),
			Error::<Runtime>::EstateIsAlreadyLeased
		);

		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_3]
		));

		assert_noop!(
			EstateModule::accept_lease_offer(Origin::signed(BENEFICIARY_ID), 2u64, BOB),
			Error::<Runtime>:::LeaseOfferDoesNotExist
		);
	});
}

#[test]
fn accept_estate_lease_offer_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		assert_ok!(EstateModule::create_lease_offer(
			Origin::signed(ALICE),
			0u64,
			10u128,
			8u32
		));

		assert_eq!(Balances::free_balance(ALICE), 99920);

		let lease_contract = LeaseContract {
			price_per_block: 10u128,
			duration: 8u32,
			end_block: 7,
			start_block: 8,
			unclaimed_rent: 80u128,
		};

		assert_eq!(EstateModule::lease_offers(0u64, ALICE), Some(lease_contract));

		assert_ok!(EstateModule::accept_lease_offer(
			Origin::signed(BENEFICIARY_ID),
			0u64,
			ALICE
		));

		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::EstateLeaseOfferAccepted(0, ALICE, 8))
		);

		let lease = LeaseContract {
			price_per_block: 10u128,
			duration: 8u32,
			end_block: 8,
			start_block: 0,
			unclaimed_rent: 80u128,
		};

		assert_eq!(EstateModule::leases(0u64), Some(lease));

		assert_eq!(EstateModule::leasors(ALICE, 0u64), Some(()));

		assert_eq!(EstateModule::lease_offers(0u64, ALICE), None);

		assert_eq!(Balances::free_balance(ALICE), 99920);
	});
}

#[test]
fn cancel_lease_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		// Mint estate
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1]
		));

		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_2]
		));

		assert_ok!(EstateModule::create_lease_offer(
			Origin::signed(ALICE),
			0u64,
			10u128,
			8u32
		));

		assert_noop!(
			EstateModule::cancel_lease(Origin::signed(ALICE), 0u64, ALICE),
			BadOrigin
		);

		assert_noop!(
			EstateModule::cancel_lease(Origin::root(), 1u64, ALICE),
			Error::<Runtime>::LeaseDoesNotExist
		);

		assert_ok!(EstateModule::accept_lease_offer(
			Origin::signed(BENEFICIARY_ID),
			0u64,
			ALICE
		));

		assert_noop!(
			EstateModule::cancel_lease(Origin::root(), 0u64, BOB),
			Error::<Runtime>::LeaseDoesNotExist
		);
	});
}

#[test]
fn cancel_lease_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		assert_ok!(EstateModule::create_lease_offer(
			Origin::signed(ALICE),
			0u64,
			10u128,
			8u32
		));

		assert_eq!(Balances::free_balance(ALICE), 99920);

		let lease_contract = LeaseContract {
			price_per_block: 10u128,
			duration: 8u32,
			end_block: 7,
			start_block: 8,
			unclaimed_rent: 80u128,
		};

		assert_eq!(EstateModule::lease_offers(0u64, ALICE), Some(lease_contract));

		assert_ok!(EstateModule::accept_lease_offer(
			Origin::signed(BENEFICIARY_ID),
			0u64,
			ALICE
		));

		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::EstateLeaseOfferAccepted(0, ALICE, 8))
		);

		let lease = LeaseContract {
			price_per_block: 10u128,
			duration: 8u32,
			end_block: 8,
			start_block: 0,
			unclaimed_rent: 80u128,
		};

		assert_eq!(EstateModule::leases(0u64), Some(lease));

		assert_eq!(EstateModule::leasors(ALICE, 0u64), Some(()));

		assert_eq!(EstateModule::lease_offers(0u64, ALICE), None);

		assert_eq!(Balances::free_balance(ALICE), 99920);

		run_to_block(4);

		assert_ok!(EstateModule::cancel_lease(Origin::root(), 0u64, ALICE));

		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::EstateLeaseContractCancelled(0))
		);

		assert_eq!(EstateModule::leases(0u64), None);

		assert_eq!(EstateModule::leasors(ALICE, 0u64), None);

		assert_eq!(Balances::free_balance(ALICE), 99960);
		assert_eq!(Balances::free_balance(BENEFICIARY_ID), 1000040);
	});
}

#[test]
fn remove_expired_lease_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		// Mint estate
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		assert_noop!(
			EstateModule::remove_expired_lease(Origin::none(), 0u64, ALICE),
			Error::<Runtime>::LeaseDoesNotExist
		);

		assert_ok!(EstateModule::create_lease_offer(
			Origin::signed(ALICE),
			0u64,
			10u128,
			8u32
		));

		assert_ok!(EstateModule::accept_lease_offer(
			Origin::signed(BENEFICIARY_ID),
			0u64,
			ALICE
		));

		assert_noop!(
			EstateModule::remove_expired_lease(Origin::none(), 0u64, ALICE),
			Error::<Runtime>::LeaseIsNotExpired
		);
	});
}

#[test]
fn remove_expired_lease_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		assert_ok!(EstateModule::create_lease_offer(
			Origin::signed(ALICE),
			0u64,
			10u128,
			8u32
		));

		assert_eq!(Balances::free_balance(ALICE), 99920);

		let lease_contract = LeaseContract {
			price_per_block: 10u128,
			duration: 8u32,
			end_block: 7,
			start_block: 8,
			unclaimed_rent: 80u128,
		};

		assert_eq!(EstateModule::lease_offers(0u64, ALICE), Some(lease_contract));

		assert_ok!(EstateModule::accept_lease_offer(
			Origin::signed(BENEFICIARY_ID),
			0u64,
			ALICE
		));

		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::EstateLeaseOfferAccepted(0, ALICE, 8))
		);

		let lease = LeaseContract {
			price_per_block: 10u128,
			duration: 8u32,
			end_block: 8,
			start_block: 0,
			unclaimed_rent: 80u128,
		};

		assert_eq!(EstateModule::leases(0u64), Some(lease));

		assert_eq!(EstateModule::leasors(ALICE, 0u64), Some(()));

		assert_eq!(EstateModule::lease_offers(0u64, ALICE), None);

		assert_eq!(Balances::free_balance(ALICE), 99920);

		run_to_block(9);

		assert_ok!(EstateModule::remove_expired_lease(Origin::none(), 0u64, ALICE));

		assert_eq!(EstateModule::leases(0u64), None);

		assert_eq!(EstateModule::leasors(ALICE, 0u64), None);

		assert_eq!(Balances::free_balance(ALICE), 99920);
		assert_eq!(Balances::free_balance(BENEFICIARY_ID), 1000080);
	});
}

#[test]
fn remove_expired_lease_offer_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		assert_ok!(EstateModule::create_lease_offer(
			Origin::signed(ALICE),
			0u64,
			10u128,
			8u32
		));

		assert_noop!(
			EstateModule::remove_expired_lease_offer(Origin::none(), 1u64, ALICE),
			Error::<Runtime>::LeaseOfferDoesNotExist
		);

		assert_noop!(
			EstateModule::remove_expired_lease_offer(Origin::none(), 0u64, ALICE),
			Error::<Runtime>::LeaseOfferIsNotExpired
		);
	});
}

#[test]
fn remove_expired_lease_offer_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		assert_ok!(EstateModule::create_lease_offer(
			Origin::signed(ALICE),
			0u64,
			10u128,
			8u32
		));

		assert_eq!(Balances::free_balance(ALICE), 99920);

		let lease_contract = LeaseContract {
			price_per_block: 10u128,
			duration: 8u32,
			end_block: 7,
			start_block: 8,
			unclaimed_rent: 80u128,
		};

		assert_eq!(EstateModule::lease_offers(0u64, ALICE), Some(lease_contract));

		run_to_block(7);

		assert_ok!(EstateModule::remove_expired_lease_offer(Origin::none(), 0u64, ALICE));
		assert_eq!(EstateModule::lease_offers(0u64, ALICE), None);
		assert_eq!(Balances::free_balance(ALICE), 100000);
	});
}

#[test]
fn collect_rent_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		assert_noop!(
			EstateModule::collect_rent(Origin::signed(BENEFICIARY_ID), 0u64, ALICE),
			Error::<Runtime>::LeaseDoesNotExist
		);

		assert_noop!(
			EstateModule::collect_rent(Origin::signed(ALICE), 0u64, BENEFICIARY_ID),
			Error::<Runtime>::NoPermission
		);

		assert_ok!(EstateModule::create_lease_offer(
			Origin::signed(ALICE),
			0u64,
			10u128,
			8u32
		));
		assert_ok!(EstateModule::create_lease_offer(
			Origin::signed(ALICE),
			0u64,
			10u128,
			8u32
		));

		assert_noop!(
			EstateModule::collect_rent(Origin::signed(BENEFICIARY_ID), 0u64, BOB),
			Error::<Runtime>::LeaseDoesNotExist
		);
	});
}

#[test]
fn collect_rent_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		assert_ok!(EstateModule::create_lease_offer(
			Origin::signed(ALICE),
			0u64,
			10u128,
			8u32
		));

		assert_eq!(Balances::free_balance(ALICE), 99920);

		let lease_contract = LeaseContract {
			price_per_block: 10u128,
			duration: 8u32,
			end_block: 7,
			start_block: 8,
			unclaimed_rent: 80u128,
		};

		assert_eq!(EstateModule::lease_offers(0u64, ALICE), Some(lease_contract));

		assert_ok!(EstateModule::accept_lease_offer(
			Origin::signed(BENEFICIARY_ID),
			0u64,
			ALICE
		));

		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::EstateLeaseOfferAccepted(0, ALICE, 8))
		);

		let lease = LeaseContract {
			price_per_block: 10u128,
			duration: 8u32,
			end_block: 8,
			start_block: 0,
			unclaimed_rent: 80u128,
		};

		assert_eq!(EstateModule::leases(0u64), Some(lease.clone()));

		assert_eq!(EstateModule::leasors(ALICE, 0u64), Some(()));

		assert_eq!(EstateModule::lease_offers(0u64, ALICE), None);

		assert_eq!(Balances::free_balance(ALICE), 99920);

		run_to_block(4);

		assert_ok!(EstateModule::collect_rent(Origin::signed(BENEFICIARY_ID), 0u64, ALICE));

		assert_eq!(last_event(), Event::Estate(crate::Event::EstateRentCollected(0, 40)));

		assert_eq!(EstateModule::leases(0u64), Some(lease));

		assert_eq!(EstateModule::leasors(ALICE, 0u64), Some(()));

		assert_eq!(Balances::free_balance(ALICE), 99920);
		assert_eq!(Balances::free_balance(BENEFICIARY_ID), 1000040);
	});
}
