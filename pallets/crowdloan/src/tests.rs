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
