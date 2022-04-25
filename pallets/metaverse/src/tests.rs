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
use primitives::staking::RoundInfo;

use super::*;

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
				estate_class_id: 1u32,
			})
		);
		let event = Event::Metaverse(crate::Event::NewMetaverseCreated(METAVERSE_ID, ALICE));
		assert_eq!(last_event(), event);
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
fn register_metaverse_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(ALICE), vec![1]));
		assert_ok!(MetaverseModule::register_metaverse(Origin::signed(ALICE), METAVERSE_ID));
		let event = Event::Metaverse(crate::Event::NewMetaverseRegisteredForStaking(METAVERSE_ID, ALICE));
		assert_eq!(last_event(), event);

		assert_eq!(MetaverseModule::get_registered_metaverse(METAVERSE_ID), Some(ALICE));
	})
}

#[test]
fn register_metaverse_should_fail_no_permission() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(ALICE), vec![1]));

		assert_noop!(
			MetaverseModule::register_metaverse(Origin::signed(BOB), METAVERSE_ID),
			Error::<Runtime>::NoPermission
		);
	})
}

#[test]
fn register_metaverse_should_fail_already_registered() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(ALICE), vec![1]));

		assert_ok!(MetaverseModule::register_metaverse(Origin::signed(ALICE), METAVERSE_ID));

		let event = Event::Metaverse(crate::Event::NewMetaverseRegisteredForStaking(METAVERSE_ID, ALICE));
		assert_eq!(last_event(), event);

		assert_noop!(
			MetaverseModule::register_metaverse(Origin::signed(ALICE), METAVERSE_ID),
			Error::<Runtime>::AlreadyRegisteredForStaking
		);
	})
}

#[test]
fn stake_should_fail_not_registered() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(ALICE), vec![1]));
		assert_noop!(
			MetaverseModule::stake(Origin::signed(ALICE), METAVERSE_ID, 100),
			Error::<Runtime>::NotRegisteredForStaking
		);
	})
}

#[test]
fn stake_should_fail_not_enough_balance() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(ALICE), vec![1]));
		assert_ok!(MetaverseModule::register_metaverse(Origin::signed(ALICE), METAVERSE_ID));
		assert_noop!(
			MetaverseModule::stake(Origin::signed(FREEDY), METAVERSE_ID, 10000),
			Error::<Runtime>::NotEnoughBalanceToStake
		);
	})
}

#[test]
fn stake_should_fail_min_staking_required() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(ALICE), vec![1]));
		assert_ok!(MetaverseModule::register_metaverse(Origin::signed(ALICE), METAVERSE_ID));

		assert_err!(
			MetaverseModule::stake(Origin::signed(ALICE), METAVERSE_ID, 10),
			Error::<Runtime>::MinimumStakingAmountRequired
		);
	})
}

#[test]
fn stake_should_fail_max_stakers() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(ALICE), vec![1]));
		assert_ok!(MetaverseModule::register_metaverse(Origin::signed(ALICE), METAVERSE_ID));
		assert_ok!(MetaverseModule::stake(Origin::signed(ALICE), METAVERSE_ID, 10000));

		assert_noop!(
			MetaverseModule::stake(Origin::signed(BOB), METAVERSE_ID, 10000),
			Error::<Runtime>::MaximumAmountOfStakersPerMetaverse
		);
	})
}

#[test]
fn stake_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(ALICE), vec![1]));
		assert_ok!(MetaverseModule::register_metaverse(Origin::signed(ALICE), METAVERSE_ID));
		assert_ok!(MetaverseModule::stake(Origin::signed(ALICE), METAVERSE_ID, 100000));

		let event = Event::Metaverse(crate::Event::MetaverseStaked(ALICE, METAVERSE_ID, 100000));
		assert_eq!(last_event(), event);

		assert_eq!(MetaverseModule::staking_info(ALICE), 100000);

		let current_staking_round: RoundInfo<BlockNumber> = MetaverseModule::staking_round();
		let mut metaverse_stake_per_round: MetaverseStakingPoints<AccountId, Balance> =
			MetaverseModule::get_metaverse_stake_per_round(METAVERSE_ID, current_staking_round.current).unwrap();

		assert_eq!(
			*(metaverse_stake_per_round.stakers.entry(ALICE).or_default()),
			100000u64
		);
		assert_eq!(metaverse_stake_per_round.stakers.len(), 1);
	})
}

#[test]
fn stake_should_work_with_min_value() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(ALICE), vec![1]));
		assert_ok!(MetaverseModule::register_metaverse(Origin::signed(ALICE), METAVERSE_ID));
		assert_ok!(MetaverseModule::stake(Origin::signed(BOB), METAVERSE_ID, 50000));

		let event = Event::Metaverse(crate::Event::MetaverseStaked(BOB, METAVERSE_ID, 19900));
		assert_eq!(last_event(), event);

		assert_eq!(MetaverseModule::staking_info(BOB), 19900);
	})
}

#[test]
fn unstake_should_fail_not_registered() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(ALICE), vec![1]));
		assert_noop!(
			MetaverseModule::unstake_and_withdraw(Origin::signed(ALICE), METAVERSE_ID, 100),
			Error::<Runtime>::NotRegisteredForStaking
		);
	})
}

#[test]
fn unstake_should_fail_no_staking_info() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(ALICE), vec![1]));
		assert_ok!(MetaverseModule::register_metaverse(Origin::signed(ALICE), METAVERSE_ID));

		assert_noop!(
			MetaverseModule::unstake_and_withdraw(Origin::signed(ALICE), METAVERSE_ID, 100),
			Error::<Runtime>::MetaverseStakingInfoNotFound
		);
	})
}

#[test]
fn unstake_should_fail_no_permission() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(ALICE), vec![1]));
		assert_ok!(MetaverseModule::register_metaverse(Origin::signed(ALICE), METAVERSE_ID));
		assert_ok!(MetaverseModule::stake(Origin::signed(ALICE), METAVERSE_ID, 10000));

		assert_noop!(
			MetaverseModule::unstake_and_withdraw(Origin::signed(BOB), METAVERSE_ID, 100),
			Error::<Runtime>::NoPermission
		);
	})
}

#[test]
fn unstake_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(ALICE), vec![1]));
		assert_ok!(MetaverseModule::register_metaverse(Origin::signed(ALICE), METAVERSE_ID));
		assert_ok!(MetaverseModule::stake(Origin::signed(ALICE), METAVERSE_ID, 10000));

		assert_ok!(MetaverseModule::unstake_and_withdraw(
			Origin::signed(ALICE),
			METAVERSE_ID,
			100
		));

		let event = Event::Metaverse(crate::Event::MetaverseUnstaked(ALICE, METAVERSE_ID, 100));
		assert_eq!(last_event(), event);

		assert_eq!(MetaverseModule::staking_info(ALICE), 9900);

		let current_staking_round: RoundInfo<BlockNumber> = MetaverseModule::staking_round();
		let mut metaverse_stake_per_round: MetaverseStakingPoints<AccountId, Balance> =
			MetaverseModule::get_metaverse_stake_per_round(METAVERSE_ID, current_staking_round.current).unwrap();

		assert_eq!(*(metaverse_stake_per_round.stakers.entry(ALICE).or_default()), 9900u64);
	})
}
