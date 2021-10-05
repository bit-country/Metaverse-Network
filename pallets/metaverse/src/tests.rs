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
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};
use sp_core::blake2_256;
use sp_runtime::traits::BadOrigin;

#[test]
fn create_metaverse_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(1), vec![1], ALICE, 1));
		assert_eq!(
			MetaverseModule::get_metaverse(&METAVERSE_ID),
			Some(MetaverseInfo {
				owner: ALICE,
				metadata: vec![1],
				currency_id: FungibleTokenId::NativeToken(0),
			})
		);
		let event = Event::Metaverse(crate::Event::NewMetaverseCreated(METAVERSE_ID));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn create_metaverse_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			MetaverseModule::create_metaverse(Origin::none(), vec![1], ALICE, 1),
			BadOrigin
		);
	});
}

#[test]
fn transfer_metaverse_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(1), vec![1], ALICE, 1));
		assert_ok!(MetaverseModule::transfer_metaverse(
			Origin::signed(1),
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
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(1), vec![1], ALICE, 1));
		assert_noop!(
			MetaverseModule::transfer_metaverse(Origin::signed(BOB), ALICE, METAVERSE_ID),
			Error::<Runtime>::NoPermission
		);
	})
}

#[test]
fn freeze_metaverse_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(1), vec![1], ALICE, 1));
		assert_ok!(MetaverseModule::freeze_metaverse(Origin::signed(1), METAVERSE_ID));
		let event = Event::Metaverse(crate::Event::MetaverseFreezed(METAVERSE_ID));
		assert_eq!(last_event(), event);
	})
}

#[test]
fn freeze_metaverse_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(1), vec![1], ALICE, 1));
		//Country owner tries to freeze their own metaverse
		assert_noop!(
			MetaverseModule::freeze_metaverse(Origin::signed(2), METAVERSE_ID),
			BadOrigin
		);
	})
}

#[test]
fn unfreeze_metaverse_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(1), vec![1], ALICE, 1));
		assert_ok!(MetaverseModule::freeze_metaverse(Origin::signed(1), METAVERSE_ID));
		let event = Event::Metaverse(crate::Event::MetaverseFreezed(METAVERSE_ID));
		assert_eq!(last_event(), event);
		assert_ok!(MetaverseModule::unfreeze_metaverse(Origin::signed(1), METAVERSE_ID));
		let event = Event::Metaverse(crate::Event::MetaverseUnfreezed(METAVERSE_ID));
		assert_eq!(last_event(), event);
	})
}

#[test]
fn destroy_metaverse_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(1), vec![1], ALICE, 1));
		assert_ok!(MetaverseModule::destroy_metaverse(Origin::signed(1), METAVERSE_ID));
		let event = Event::Metaverse(crate::Event::MetaverseDestroyed(METAVERSE_ID));
		assert_eq!(last_event(), event);
	})
}

#[test]
fn destroy_metaverse_without_root_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(1), vec![1], ALICE, 1));
		assert_noop!(
			MetaverseModule::destroy_metaverse(Origin::signed(2), METAVERSE_ID),
			BadOrigin
		);
	})
}

#[test]
fn destroy_metaverse_with_no_id_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MetaverseModule::create_metaverse(Origin::signed(1), vec![1], ALICE, 1));
		assert_noop!(
			MetaverseModule::destroy_metaverse(Origin::signed(1), COUNTRY_ID_NOT_EXIST),
			Error::<Runtime>::MetaverseInfoNotFound
		);
	})
}
