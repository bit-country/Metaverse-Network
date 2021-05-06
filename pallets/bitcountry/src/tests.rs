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
fn create_country_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(CountryModule::create_country(
            Origin::signed(ALICE),
            vec![1]
        ));
        assert_eq!(
            CountryModule::get_country(&COUNTRY_ID),
            Some(Country {
                owner: ALICE,
                metadata: vec![1],
                currency_id: Default::default(),
            })
        );
        let event = Event::bitcountry(RawEvent::NewCountryCreated(COUNTRY_ID));
        assert_eq!(last_event(), event);
    });
}

#[test]
fn create_country_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            CountryModule::create_country(Origin::none(), vec![1],),
            BadOrigin
        );
    });
}

#[test]
fn transfer_country_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(CountryModule::create_country(
            Origin::signed(ALICE),
            vec![1]
        ));
        assert_ok!(CountryModule::transfer_country(
            Origin::signed(ALICE),
            BOB,
            COUNTRY_ID
        ));
        let event = Event::bitcountry(RawEvent::TransferredCountry(COUNTRY_ID, ALICE, BOB));
        assert_eq!(last_event(), event);
        //Make sure 2 ways transfer works
        assert_ok!(CountryModule::transfer_country(
            Origin::signed(BOB),
            ALICE,
            COUNTRY_ID
        ));
        let event = Event::bitcountry(RawEvent::TransferredCountry(COUNTRY_ID, BOB, ALICE));
        assert_eq!(last_event(), event);
    })
}

#[test]
fn transfer_country_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(CountryModule::create_country(
            Origin::signed(ALICE),
            vec![1]
        ));
        assert_noop!(
            CountryModule::transfer_country(Origin::signed(BOB), ALICE, COUNTRY_ID),
            Error::<Runtime>::NoPermission
        );
    })
}

#[test]
fn freeze_country_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(CountryModule::create_country(
            Origin::signed(ALICE),
            vec![1]
        ));
        assert_ok!(CountryModule::freeze_country(Origin::root(), COUNTRY_ID));
        let event = Event::bitcountry(RawEvent::CountryFreezed(COUNTRY_ID));
        assert_eq!(last_event(), event);
    })
}

#[test]
fn freeze_country_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(CountryModule::create_country(
            Origin::signed(ALICE),
            vec![1]
        ));
        //Country owner tries to freeze their own bitcountry
        assert_noop!(
            CountryModule::freeze_country(Origin::signed(ALICE), COUNTRY_ID),
            BadOrigin
        );
    })
}

#[test]
fn unfreeze_country_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(CountryModule::create_country(
            Origin::signed(ALICE),
            vec![1]
        ));
        assert_ok!(CountryModule::freeze_country(Origin::root(), COUNTRY_ID));
        let event = Event::bitcountry(RawEvent::CountryFreezed(COUNTRY_ID));
        assert_eq!(last_event(), event);
        assert_ok!(CountryModule::unfreeze_country(Origin::root(), COUNTRY_ID));
        let event = Event::bitcountry(RawEvent::CountryUnFreezed(COUNTRY_ID));
        assert_eq!(last_event(), event);
    })
}

#[test]
fn destroy_country_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(CountryModule::create_country(
            Origin::signed(ALICE),
            vec![1]
        ));
        assert_ok!(CountryModule::destroy_country(Origin::root(), COUNTRY_ID));
        let event = Event::bitcountry(RawEvent::CountryDestroyed(COUNTRY_ID));
        assert_eq!(last_event(), event);
    })
}

#[test]
fn destroy_country_without_root_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(CountryModule::create_country(
            Origin::signed(ALICE),
            vec![1]
        ));
        assert_noop!(
            CountryModule::destroy_country(Origin::signed(ALICE), COUNTRY_ID),
            BadOrigin
        );
    })
}

#[test]
fn destroy_country_with_no_id_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(CountryModule::create_country(
            Origin::signed(ALICE),
            vec![1]
        ));
        assert_noop!(
            CountryModule::destroy_country(Origin::root(), COUNTRY_ID_NOT_EXIST),
            Error::<Runtime>::CountryInfoNotFound
        );
    })
}
