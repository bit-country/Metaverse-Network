// This file is part of Bit.Country.

// The evm-mapping pallet is inspired by evm mapping designed by AcalaNetwork

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

use std::str::FromStr;

use frame_support::{assert_noop, assert_ok};

use mock::{alice, bob, secp_utils::*, EVMMapping, Event, ExtBuilder, Origin, Runtime, System, ALICE, BOB};

use super::*;

#[test]
fn claim_account_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EVMMapping::claim_eth_account(
			Origin::signed(ALICE),
			eth(&alice()),
			sig::<Runtime>(&alice(), &eth(&alice()).encode(), &[][..])
		));
		System::assert_last_event(Event::EVMMapping(crate::Event::ClaimAccount {
			account_id: ALICE,
			evm_address: eth(&alice()),
		}));
		assert!(Accounts::<Runtime>::contains_key(eth(&alice())) && EvmAddresses::<Runtime>::contains_key(ALICE));
	});
}
