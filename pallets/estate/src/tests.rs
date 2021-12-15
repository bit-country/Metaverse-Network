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

use super::*;
use frame_support::{assert_err, assert_noop, assert_ok};
use mock::{Event, *};
use sp_runtime::traits::BadOrigin;

#[test]
fn set_max_bound_should_reject_non_root() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EstateModule::set_max_bounds(Origin::signed(ALICE), METAVERSE_ID, MAX_BOUND),
			BadOrigin
		);
	});
}

#[test]
fn set_max_bound_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::set_max_bounds(Origin::root(), METAVERSE_ID, MAX_BOUND));

		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::MaxBoundSet(METAVERSE_ID, MAX_BOUND))
		);

		assert_eq!(EstateModule::get_max_bounds(METAVERSE_ID), MAX_BOUND);
	});
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
fn mint_land_should_reject_no_max_bound_set() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EstateModule::mint_land(Origin::root(), BENEFICIARY_ID, METAVERSE_ID, COORDINATE_IN_1),
			Error::<Runtime>::NoMaxBoundSet
		);
	});
}

#[test]
fn mint_land_should_reject_out_bound() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::set_max_bounds(Origin::root(), METAVERSE_ID, MAX_BOUND));

		assert_noop!(
			EstateModule::mint_land(Origin::root(), BENEFICIARY_ID, METAVERSE_ID, COORDINATE_OUT),
			Error::<Runtime>::LandUnitIsOutOfBound
		);
	});
}

#[test]
fn mint_land_should_work_with_one_coordinate() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::set_max_bounds(Origin::root(), METAVERSE_ID, MAX_BOUND));

		assert_ok!(EstateModule::mint_land(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			COORDINATE_IN_1
		));

		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::NewLandUnitMinted(
				BENEFICIARY_ID,
				METAVERSE_ID,
				COORDINATE_IN_1,
			))
		);

		assert_eq!(EstateModule::all_land_units_count(), 1);
	});
}

#[test]
fn mint_land_should_work_have_correct_owner() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::set_max_bounds(Origin::root(), METAVERSE_ID, MAX_BOUND));

		assert_eq!(EstateModule::get_land_units(METAVERSE_ID, COORDINATE_IN_1), 0);

		assert_ok!(EstateModule::mint_land(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			COORDINATE_IN_1
		));

		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::NewLandUnitMinted(
				BENEFICIARY_ID,
				METAVERSE_ID,
				COORDINATE_IN_1,
			))
		);

		assert_eq!(EstateModule::all_land_units_count(), 1);

		assert_eq!(
			EstateModule::get_land_units(METAVERSE_ID, COORDINATE_IN_1),
			BENEFICIARY_ID
		);
	});
}

#[test]
fn mint_land_should_reject_with_duplicate_coordinates() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::set_max_bounds(Origin::root(), METAVERSE_ID, MAX_BOUND));

		assert_ok!(EstateModule::mint_land(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			COORDINATE_IN_1
		));

		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::NewLandUnitMinted(
				BENEFICIARY_ID,
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
		assert_ok!(EstateModule::set_max_bounds(Origin::root(), METAVERSE_ID, MAX_BOUND));

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
		assert_ok!(EstateModule::set_max_bounds(Origin::root(), METAVERSE_ID, MAX_BOUND));

		assert_ok!(EstateModule::mint_land(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			COORDINATE_IN_1
		));

		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::NewLandUnitMinted(
				BENEFICIARY_ID,
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
				BENEFICIARY_ID,
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
fn mint_lands_should_reject_no_max_bound_set() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EstateModule::mint_lands(
				Origin::root(),
				BENEFICIARY_ID,
				METAVERSE_ID,
				vec![COORDINATE_IN_1, COORDINATE_IN_2]
			),
			Error::<Runtime>::NoMaxBoundSet
		);
	});
}

#[test]
fn mint_lands_should_reject_out_bound() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::set_max_bounds(Origin::root(), METAVERSE_ID, MAX_BOUND));

		assert_noop!(
			EstateModule::mint_lands(
				Origin::root(),
				BENEFICIARY_ID,
				METAVERSE_ID,
				vec![COORDINATE_OUT, COORDINATE_IN_1]
			),
			Error::<Runtime>::LandUnitIsOutOfBound
		);
	});
}

#[test]
fn mint_lands_should_work_with_one_coordinate() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::set_max_bounds(Origin::root(), METAVERSE_ID, MAX_BOUND));

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
		assert_ok!(EstateModule::set_max_bounds(Origin::root(), METAVERSE_ID, MAX_BOUND));

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
fn transfer_land_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::set_max_bounds(Origin::root(), METAVERSE_ID, MAX_BOUND));

		assert_ok!(EstateModule::mint_land(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			COORDINATE_IN_1
		));

		assert_eq!(
			EstateModule::get_land_units(METAVERSE_ID, COORDINATE_IN_1),
			BENEFICIARY_ID
		);

		assert_ok!(EstateModule::transfer_land(
			Origin::signed(BENEFICIARY_ID),
			ALICE,
			METAVERSE_ID,
			COORDINATE_IN_1
		));

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
		assert_ok!(EstateModule::set_max_bounds(Origin::root(), METAVERSE_ID, MAX_BOUND));

		assert_ok!(EstateModule::mint_land(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			COORDINATE_IN_1
		));

		assert_eq!(
			EstateModule::get_land_units(METAVERSE_ID, COORDINATE_IN_1),
			BENEFICIARY_ID
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
		assert_ok!(EstateModule::set_max_bounds(Origin::root(), METAVERSE_ID, MAX_BOUND));

		assert_ok!(EstateModule::mint_land(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			COORDINATE_IN_1
		));

		assert_eq!(
			EstateModule::get_land_units(METAVERSE_ID, COORDINATE_IN_1),
			BENEFICIARY_ID
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
			BENEFICIARY_ID
		);
	});
}

#[test]
fn transfer_land_should_do_fail_for_already_in_auction() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EstateModule::transfer_land(
				Origin::signed(BENEFICIARY_ID),
				BENEFICIARY_ID,
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
		assert_ok!(EstateModule::set_max_bounds(Origin::root(), METAVERSE_ID, MAX_BOUND));

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
		assert_ok!(EstateModule::set_max_bounds(Origin::root(), METAVERSE_ID, MAX_BOUND));

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
			Some(vec![COORDINATE_IN_1, COORDINATE_IN_2])
		); //vec![COORDINATE_IN_1, COORDINATE_IN_2]
		assert_eq!(EstateModule::get_estate_owner(BENEFICIARY_ID, estate_id), Some(()));
		assert_eq!(
			EstateModule::get_user_land_units(&BENEFICIARY_ID, &METAVERSE_ID).len(),
			2
		);

		// Destroy estate
		assert_ok!(EstateModule::dissolve_estate(
			Origin::signed(BENEFICIARY_ID),
			estate_id,
			METAVERSE_ID,
		));

		assert_eq!(EstateModule::all_estates_count(), 0);
		assert_eq!(EstateModule::get_estates(estate_id), None);
		assert_eq!(EstateModule::get_estate_owner(BENEFICIARY_ID, estate_id), None);
		assert_eq!(
			EstateModule::get_user_land_units(&BENEFICIARY_ID, &METAVERSE_ID).len(),
			2
		);
	});
}

#[test]
fn dissolve_estate_should_reject_non_owner() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::set_max_bounds(Origin::root(), METAVERSE_ID, MAX_BOUND));

		// Mint estate
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		assert_err!(
			EstateModule::dissolve_estate(Origin::signed(ALICE), 0, METAVERSE_ID),
			Error::<Runtime>::NoPermission
		);
	});
}

#[test]
fn add_land_unit_should_reject_non_owner() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::set_max_bounds(Origin::root(), METAVERSE_ID, MAX_BOUND));

		// Mint estate
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_2]
		));

		assert_err!(
			EstateModule::add_land_unit(Origin::signed(ALICE), 0, METAVERSE_ID, vec![COORDINATE_IN_2]),
			Error::<Runtime>::NoPermission
		);
	});
}

#[test]
fn add_land_unit_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::set_max_bounds(Origin::root(), METAVERSE_ID, MAX_BOUND));

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
		assert_eq!(EstateModule::get_estates(estate_id), Some(vec![COORDINATE_IN_1])); //vec![COORDINATE_IN_1, COORDINATE_IN_2]
		assert_eq!(EstateModule::get_estate_owner(BENEFICIARY_ID, estate_id), Some(()));
		assert_eq!(
			EstateModule::get_user_land_units(&BENEFICIARY_ID, &METAVERSE_ID).len(),
			1
		);
		assert_eq!(EstateModule::all_land_units_count(), 1);

		assert_ok!(EstateModule::mint_land(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			COORDINATE_IN_2
		));
		// Update estate
		assert_ok!(EstateModule::add_land_unit(
			Origin::signed(BENEFICIARY_ID),
			estate_id,
			METAVERSE_ID,
			vec![COORDINATE_IN_2]
		));

		assert_eq!(
			EstateModule::get_estates(estate_id),
			Some(vec![COORDINATE_IN_1, COORDINATE_IN_2])
		);

		assert_eq!(
			EstateModule::get_user_land_units(&BENEFICIARY_ID, &METAVERSE_ID).len(),
			2
		);
		assert_eq!(EstateModule::all_land_units_count(), 2);
	});
}

#[test]
fn remove_land_unit_should_reject_non_owner() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::set_max_bounds(Origin::root(), METAVERSE_ID, MAX_BOUND));

		// Mint estate
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		assert_err!(
			EstateModule::remove_land_unit(Origin::signed(ALICE), 0, METAVERSE_ID, vec![COORDINATE_IN_2]),
			Error::<Runtime>::NoPermission
		);
	});
}

#[test]
fn remove_land_unit_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::set_max_bounds(Origin::root(), METAVERSE_ID, MAX_BOUND));

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
			Some(vec![COORDINATE_IN_1, COORDINATE_IN_2])
		);
		assert_eq!(EstateModule::get_estate_owner(BENEFICIARY_ID, estate_id), Some(()));
		assert_eq!(
			EstateModule::get_user_land_units(&BENEFICIARY_ID, &METAVERSE_ID).len(),
			2
		);
		assert_eq!(EstateModule::all_land_units_count(), 2);

		// Update estate
		assert_ok!(EstateModule::remove_land_unit(
			Origin::signed(BENEFICIARY_ID),
			estate_id,
			METAVERSE_ID,
			vec![COORDINATE_IN_2]
		));

		assert_eq!(EstateModule::get_estates(estate_id), Some(vec![COORDINATE_IN_1]));

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
		assert_ok!(EstateModule::set_max_bounds(Origin::root(), METAVERSE_ID, MAX_BOUND));
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
			Some(vec![COORDINATE_IN_1, COORDINATE_IN_2])
		); //vec![COORDINATE_IN_1, COORDINATE_IN_2]
		assert_eq!(EstateModule::get_estate_owner(BENEFICIARY_ID, estate_id), Some(()));
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
		assert_ok!(EstateModule::set_max_bounds(Origin::root(), METAVERSE_ID, MAX_BOUND));
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
			Some(vec![COORDINATE_IN_1, COORDINATE_IN_2])
		);
		assert_eq!(EstateModule::get_estate_owner(BENEFICIARY_ID, estate_id), Some(()));

		let estate_id_non_exists: u64 = 999;
		assert_eq!(EstateModule::get_estates(estate_id_non_exists), None);
		assert_eq!(
			EstateModule::get_estate_owner(BENEFICIARY_ID, estate_id_non_exists),
			None
		);
	});
}

#[test]
fn transfer_estate_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::set_max_bounds(Origin::root(), METAVERSE_ID, MAX_BOUND));
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		let estate_id: u64 = 0;
		assert_eq!(EstateModule::get_estate_owner(BENEFICIARY_ID, estate_id), Some(()));

		assert_ok!(EstateModule::transfer_estate(
			Origin::signed(BENEFICIARY_ID),
			ALICE,
			estate_id
		));

		assert_eq!(EstateModule::get_estate_owner(BENEFICIARY_ID, estate_id), None);
		assert_eq!(EstateModule::get_estate_owner(ALICE, estate_id), Some(()));

		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::TransferredEstate(estate_id, BENEFICIARY_ID, ALICE))
		);
	});
}

#[test]
fn transfer_estate_should_reject_no_permission() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::set_max_bounds(Origin::root(), METAVERSE_ID, MAX_BOUND));
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		let estate_id: u64 = 0;
		assert_eq!(EstateModule::get_estate_owner(BENEFICIARY_ID, estate_id), Some(()));

		assert_noop!(
			EstateModule::transfer_estate(Origin::signed(BOB), ALICE, estate_id),
			Error::<Runtime>::NoPermission
		);
	});
}

#[test]
fn transfer_estate_should_reject_already_in_auction() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EstateModule::transfer_estate(Origin::signed(BOB), ALICE, ESTATE_IN_AUCTION),
			Error::<Runtime>::EstateAlreadyInAuction
		);
	});
}

#[test]
fn transfer_estate_should_fail_with_same_account() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::set_max_bounds(Origin::root(), METAVERSE_ID, MAX_BOUND));
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		let estate_id: u64 = 0;
		assert_eq!(EstateModule::get_estate_owner(BENEFICIARY_ID, estate_id), Some(()));

		assert_noop!(
			EstateModule::transfer_estate(Origin::signed(BENEFICIARY_ID), BENEFICIARY_ID, estate_id),
			Error::<Runtime>::AlreadyOwnTheEstate
		);

		assert_eq!(EstateModule::get_estate_owner(BENEFICIARY_ID, estate_id), Some(()));
	});
}

#[test]
fn create_estate_should_reject_non_root() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EstateModule::create_estate(
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
fn create_estate_should_fail_for_not_minted_land() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::set_max_bounds(Origin::root(), METAVERSE_ID, MAX_BOUND));

		assert_err!(
			EstateModule::create_estate(
				Origin::root(),
				BENEFICIARY_ID,
				METAVERSE_ID,
				vec![COORDINATE_IN_1, COORDINATE_IN_2]
			),
			Error::<Runtime>::LandUnitIsNotAvailable
		);
	});
}

#[test]
fn create_estate_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::set_max_bounds(Origin::root(), METAVERSE_ID, MAX_BOUND));

		assert_ok!(EstateModule::mint_lands(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		assert_ok!(EstateModule::create_estate(
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
			Some(vec![COORDINATE_IN_1, COORDINATE_IN_2])
		); //vec![COORDINATE_IN_1, COORDINATE_IN_2]
		assert_eq!(EstateModule::get_estate_owner(BENEFICIARY_ID, estate_id), Some(()));
	});
}

#[test]
fn create_estate_should_return_none_for_non_exist_estate() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::set_max_bounds(Origin::root(), METAVERSE_ID, MAX_BOUND));

		assert_ok!(EstateModule::mint_lands(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		assert_ok!(EstateModule::create_estate(
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
			Some(vec![COORDINATE_IN_1, COORDINATE_IN_2])
		);
		assert_eq!(EstateModule::get_estate_owner(BENEFICIARY_ID, estate_id), Some(()));

		let estate_id_non_exists: u64 = 999;
		assert_eq!(EstateModule::get_estates(estate_id_non_exists), None);
		assert_eq!(
			EstateModule::get_estate_owner(BENEFICIARY_ID, estate_id_non_exists),
			None
		);
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
				assert_eq!(a.is_frozen, false);
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
				assert_eq!(a.is_frozen, false);
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
				assert_eq!(a.is_frozen, false);
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
				assert_eq!(a.is_frozen, false);
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
				assert_eq!(a.is_frozen, true);
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
				assert_eq!(a.is_frozen, true);
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
				assert_eq!(a.is_frozen, false);
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
fn deploye_undeployed_land_block_should_fail_if_not_found() {
	ExtBuilder::default().build().execute_with(|| {
		let undeployed_land_block_id: UndeployedLandBlockId = 0;

		assert_noop!(
			EstateModule::deploy_land_block(
				Origin::signed(ALICE),
				undeployed_land_block_id,
				METAVERSE_ID,
				vec![COORDINATE_IN_1]
			),
			Error::<Runtime>::UndeployedLandBlockNotFound
		);
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
				vec![COORDINATE_IN_1]
			),
			Error::<Runtime>::NoPermission
		);
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
				METAVERSE_ID,
				vec![COORDINATE_IN_1]
			),
			Error::<Runtime>::UndeployedLandBlockFreezed
		);
	});
}

#[test]
fn deploy_undeployed_land_block_should_fail_not_enough_land_units() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			1,
			UndeployedLandBlockType::BoundToAddress
		));

		let undeployed_land_block_id: UndeployedLandBlockId = 0;

		assert_noop!(
			EstateModule::deploy_land_block(
				Origin::signed(BOB),
				undeployed_land_block_id,
				METAVERSE_ID,
				vec![COORDINATE_IN_1, COORDINATE_IN_2]
			),
			Error::<Runtime>::UndeployedLandBlockDoesNotHaveEnoughLandUnits
		);
	});
}

#[test]
fn deploy_undeployed_land_block_should_fail_if_no_maxbound() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			100,
			UndeployedLandBlockType::BoundToAddress
		));

		let undeployed_land_block_id: UndeployedLandBlockId = 0;

		assert_noop!(
			EstateModule::deploy_land_block(
				Origin::signed(BOB),
				undeployed_land_block_id,
				METAVERSE_ID,
				vec![COORDINATE_IN_1, COORDINATE_IN_2]
			),
			Error::<Runtime>::NoMaxBoundSet
		);
	});
}

#[test]
fn deploy_undeployed_land_block_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::set_max_bounds(Origin::root(), METAVERSE_ID, MAX_BOUND));

		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			100,
			UndeployedLandBlockType::BoundToAddress
		));

		let undeployed_land_block_id: UndeployedLandBlockId = 0;

		let undeployed_land_block = EstateModule::get_undeployed_land_block(undeployed_land_block_id);
		match undeployed_land_block {
			Some(a) => {
				assert_eq!(a.number_land_units, 100);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}

		assert_ok!(EstateModule::deploy_land_block(
			Origin::signed(BOB),
			undeployed_land_block_id,
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		assert_eq!(
			last_event(),
			Event::Estate(crate::Event::LandBlockDeployed(
				BOB,
				METAVERSE_ID,
				undeployed_land_block_id,
				vec![COORDINATE_IN_1, COORDINATE_IN_2],
			))
		);

		let updated_undeployed_land_block = EstateModule::get_undeployed_land_block(undeployed_land_block_id);
		match updated_undeployed_land_block {
			Some(a) => {
				assert_eq!(a.number_land_units, 98);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}

		assert_eq!(EstateModule::all_land_units_count(), 2);
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
fn burn_undeployed_land_block_should_fail_if_not_frozon() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::issue_undeployed_land_blocks(
			Origin::root(),
			BOB,
			1,
			20,
			UndeployedLandBlockType::BoundToAddress
		));

		assert_noop!(
			EstateModule::burn_undeployed_land_blocks(Origin::root(), 0),
			Error::<Runtime>::OnlyFrozenUndeployedLandBlockCanBeDestroyed
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

		assert_ok!(EstateModule::freeze_undeployed_land_blocks(Origin::root(), 0));

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
