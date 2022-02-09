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

#[test]
fn authorize_power_generator_collection_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EconomyModule::authorize_power_generator_collection(
			Origin::root(),
			CLASS_ID
		));

		let event = Event::Metaverse(crate::Event::PowerGeneratorCollectionAuthorized(CLASS_ID));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn authorize_power_generator_collection_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EconomyModule::authorize_power_generator_collection(Origin::signed(BOB), CLASS_ID),
			BadOrigin
		);
	});
}

#[test]
fn authorize_power_distributor_collection_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EconomyModule::authorize_power_distributor_collection(
			Origin::root(),
			CLASS_ID
		));

		let event = Event::Metaverse(crate::Event::PowerDistributorCollectionAuthorized(CLASS_ID));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn authorize_power_distributor_collection_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EconomyModule::authorize_power_distributor_collection(Origin::signed(BOB), CLASS_ID),
			BadOrigin
		);
	});
}
