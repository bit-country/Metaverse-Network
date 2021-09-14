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
fn create_bitcountry_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(BitCountryModule::create_bitcountry(
            Origin::signed(ALICE),
            vec![1]
        ));
        assert_eq!(
            BitCountryModule::get_bitcountry(&BITCOUNTRY_ID),
            Some(BitCountryStruct {
                owner: ALICE,
                metadata: vec![1],
                currency_id: FungibleTokenId::NativeToken(0),
            })
        );
        let event = Event::bitcountry(crate::Event::NewBitCountryCreated(BITCOUNTRY_ID));
        assert_eq!(last_event(), event);
    });
}

#[test]
fn create_bitcountry_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            BitCountryModule::create_bitcountry(Origin::none(), vec![1],),
            BadOrigin
        );
    });
}

#[test]
fn transfer_bitcountry_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(BitCountryModule::create_bitcountry(
            Origin::signed(ALICE),
            vec![1]
        ));
        assert_ok!(BitCountryModule::transfer_bitcountry(
            Origin::signed(ALICE),
            BOB,
            BITCOUNTRY_ID
        ));
        let event = Event::bitcountry(crate::Event::TransferredBitCountry(
            BITCOUNTRY_ID,
            ALICE,
            BOB,
        ));
        assert_eq!(last_event(), event);
        // Make sure 2 ways transfer works
        assert_ok!(BitCountryModule::transfer_bitcountry(
            Origin::signed(BOB),
            ALICE,
            BITCOUNTRY_ID
        ));
        let event = Event::bitcountry(crate::Event::TransferredBitCountry(
            BITCOUNTRY_ID,
            BOB,
            ALICE,
        ));
        assert_eq!(last_event(), event);
    })
}

#[test]
fn transfer_bitcountry_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(BitCountryModule::create_bitcountry(
            Origin::signed(ALICE),
            vec![1]
        ));
        assert_noop!(
            BitCountryModule::transfer_bitcountry(Origin::signed(BOB), ALICE, BITCOUNTRY_ID),
            Error::<Runtime>::NoPermission
        );
    })
}

#[test]
fn freeze_bitcountry_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(BitCountryModule::create_bitcountry(
            Origin::signed(ALICE),
            vec![1]
        ));
        assert_ok!(BitCountryModule::freeze_bitcountry(
            Origin::root(),
            BITCOUNTRY_ID
        ));
        let event = Event::bitcountry(crate::Event::BitCountryFreezed(BITCOUNTRY_ID));
        assert_eq!(last_event(), event);
    })
}

#[test]
fn freeze_bitcountry_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(BitCountryModule::create_bitcountry(
            Origin::signed(ALICE),
            vec![1]
        ));
        //Country owner tries to freeze their own bitcountry
        assert_noop!(
            BitCountryModule::freeze_bitcountry(Origin::signed(ALICE), BITCOUNTRY_ID),
            BadOrigin
        );
    })
}

#[test]
fn unfreeze_bitcountry_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(BitCountryModule::create_bitcountry(
            Origin::signed(ALICE),
            vec![1]
        ));
        assert_ok!(BitCountryModule::freeze_bitcountry(
            Origin::root(),
            BITCOUNTRY_ID
        ));
        let event = Event::bitcountry(crate::Event::BitCountryFreezed(BITCOUNTRY_ID));
        assert_eq!(last_event(), event);
        assert_ok!(BitCountryModule::unfreeze_bitcountry(
            Origin::root(),
            BITCOUNTRY_ID
        ));
        let event = Event::bitcountry(crate::Event::BitCountryUnfreezed(BITCOUNTRY_ID));
        assert_eq!(last_event(), event);
    })
}

#[test]
fn destroy_bitcountry_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(BitCountryModule::create_bitcountry(
            Origin::signed(ALICE),
            vec![1]
        ));
        assert_ok!(BitCountryModule::destroy_bitcountry(
            Origin::root(),
            BITCOUNTRY_ID
        ));
        let event = Event::bitcountry(crate::Event::BitCountryDestroyed(BITCOUNTRY_ID));
        assert_eq!(last_event(), event);
    })
}

#[test]
fn destroy_bitcountry_without_root_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(BitCountryModule::create_bitcountry(
            Origin::signed(ALICE),
            vec![1]
        ));
        assert_noop!(
            BitCountryModule::destroy_bitcountry(Origin::signed(ALICE), BITCOUNTRY_ID),
            BadOrigin
        );
    })
}

#[test]
fn destroy_bitcountry_with_no_id_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(BitCountryModule::create_bitcountry(
            Origin::signed(ALICE),
            vec![1]
        ));
        assert_noop!(
            BitCountryModule::destroy_bitcountry(Origin::root(), COUNTRY_ID_NOT_EXIST),
            Error::<Runtime>::BitCountryInfoNotFound
        );
    })
}
