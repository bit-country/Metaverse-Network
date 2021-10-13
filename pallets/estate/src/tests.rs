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
use sp_core::blake2_256;
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
			Event::Estate(crate::Event::NewLandUnitMinted(METAVERSE_ID, COORDINATE_IN_1))
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
			Event::Estate(crate::Event::NewLandUnitMinted(METAVERSE_ID, COORDINATE_IN_1))
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
			Event::Estate(crate::Event::NewLandUnitMinted(METAVERSE_ID, COORDINATE_IN_1))
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
				METAVERSE_ID,
				vec![COORDINATE_IN_1, COORDINATE_IN_2]
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
			Event::Estate(crate::Event::NewLandUnitMinted(METAVERSE_ID, COORDINATE_IN_1))
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
			Event::Estate(crate::Event::NewLandUnitMinted(METAVERSE_ID, COORDINATE_IN_2))
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
			Event::Estate(crate::Event::NewLandsMinted(METAVERSE_ID, vec![COORDINATE_IN_1]))
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
				METAVERSE_ID,
				vec![COORDINATE_IN_1, COORDINATE_IN_2]
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
				ALICE
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
fn mint_estate_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EstateModule::set_max_bounds(Origin::root(), METAVERSE_ID, MAX_BOUND));
		assert_ok!(EstateModule::mint_estate(
			Origin::root(),
			BENEFICIARY_ID,
			METAVERSE_ID,
			vec![COORDINATE_IN_1, COORDINATE_IN_2]
		));

		let mut estate_id: u64 = 0;
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

		let mut estate_id: u64 = 0;
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

		let mut estate_id: u64 = 0;
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

		let mut estate_id: u64 = 0;
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

		let mut estate_id: u64 = 0;
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

		let mut estate_id: u64 = 0;
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

		let mut estate_id: u64 = 0;
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

		let mut estate_id: u64 = 0;
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
