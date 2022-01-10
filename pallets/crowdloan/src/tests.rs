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

use mock::{Event, *};

use super::*;

#[test]
fn set_distribution_origin_reject_non_root() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			CrowdloanModule::set_distributor_origin(Origin::signed(ALICE), BOB),
			BadOrigin
		);
	});
}

#[test]
fn set_distribution_origin_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CrowdloanModule::set_distributor_origin(Origin::root(), ALICE));

		assert_eq!(
			last_event(),
			Event::Crowdloan(crate::Event::AddedDistributorOrigin(ALICE))
		);

		assert_eq!(CrowdloanModule::is_accepted_origin(&ALICE), true);
	});
}

#[test]
fn remove_distribution_origin_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CrowdloanModule::set_distributor_origin(Origin::root(), ALICE));

		assert_eq!(
			last_event(),
			Event::Crowdloan(crate::Event::AddedDistributorOrigin(ALICE))
		);

		assert_ok!(CrowdloanModule::remove_distributor_origin(Origin::root(), ALICE));
		assert_eq!(
			last_event(),
			Event::Crowdloan(crate::Event::RemovedDistributorOrigin(ALICE))
		);

		assert_eq!(CrowdloanModule::is_accepted_origin(&ALICE), false);
	});
}

#[test]
fn remove_distribution_origin_not_exist_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			CrowdloanModule::remove_distributor_origin(Origin::root(), ALICE),
			Error::<Runtime>::DistributorOriginDoesNotExist
		);
	});
}

#[test]
fn transfer_unlocked_reward_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CrowdloanModule::set_distributor_origin(Origin::root(), ALICE));

		assert_ok!(CrowdloanModule::transfer_unlocked_reward(
			Origin::signed(ALICE),
			BOB,
			100
		));
		assert_eq!(Balances::free_balance(&BOB), 100100);
		assert_eq!(Balances::free_balance(&ALICE), 99900);
	});
}

#[test]
fn transfer_unlocked_reward_non_accepted_origin_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			CrowdloanModule::transfer_unlocked_reward(Origin::signed(ALICE), BOB, 100),
			Error::<Runtime>::NoPermission
		);
	});
}

#[test]
fn transfer_vested_reward_non_accepted_origin_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let vested_schedule = VestingInfo::new(100, 10, 1);

		assert_noop!(
			CrowdloanModule::transfer_vested_reward(Origin::signed(ALICE), BOB, vested_schedule),
			Error::<Runtime>::NoPermission
		);
	});
}

#[test]
fn transfer_vested_reward_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CrowdloanModule::set_distributor_origin(Origin::root(), ALICE));

		let vested_schedule = VestingInfo::new(100, 10, 1);

		assert_ok!(CrowdloanModule::transfer_vested_reward(
			Origin::signed(ALICE),
			BOB,
			vested_schedule
		));
		assert_eq!(Vesting::vesting_balance(&BOB), Some(100));
	});
}

#[test]
fn remove_vested_reward_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CrowdloanModule::set_distributor_origin(Origin::root(), ALICE));

		let vested_schedule = VestingInfo::new(100, 10, 1);

		assert_ok!(CrowdloanModule::transfer_vested_reward(
			Origin::signed(ALICE),
			BOB,
			vested_schedule
		));
		assert_eq!(Vesting::vesting_balance(&BOB), Some(100));

		assert_ok!(CrowdloanModule::remove_vested_reward(Origin::root(), BOB, 0));
		// remove vesting balance
		assert_eq!(Vesting::vesting_balance(&BOB), None);
	});
}

#[test]
fn remove_vested_reward_should_fail_for_non_root() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CrowdloanModule::set_distributor_origin(Origin::root(), ALICE));

		let vested_schedule = VestingInfo::new(100, 10, 1);

		assert_ok!(CrowdloanModule::transfer_vested_reward(
			Origin::signed(ALICE),
			BOB,
			vested_schedule
		));
		assert_eq!(Vesting::vesting_balance(&BOB), Some(100));

		assert_noop!(
			CrowdloanModule::remove_vested_reward(Origin::signed(ALICE), BOB, 0),
			BadOrigin
		);
	});
}
