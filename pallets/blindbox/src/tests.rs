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
fn set_blindbox_caller_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(BlindBoxModule::set_blindbox_caller(
            Origin::root(),
            ALICE
        ));
    });
}

#[test]
fn set_blindbox_caller_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(BlindBoxModule::set_blindbox_caller(
            Origin::signed(ALICE),
            ALICE
        ), BadOrigin);
    });
}

#[test]
fn generate_blindbox_ids_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(BlindBoxModule::set_blindbox_caller(
            Origin::root(),
            ALICE
        ));

        assert_ok!(BlindBoxModule::generate_blindbox_ids(
            Origin::signed(ALICE),
            10
        ));

        assert_eq!(
            BlindBoxModule::all_blindboxes_count(),
            10
        );
    });
}

#[test]
fn generate_blindbox_ids_no_permission() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(BlindBoxModule::set_blindbox_caller(
            Origin::root(),
            ALICE
        ));

        assert_noop!(
            BlindBoxModule::generate_blindbox_ids(Origin::signed(BOB), 10),
            Error::<Runtime>::NoPermission
        );
    });
}

#[test]
fn generate_blindbox_ids_blindbox_still_available() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(BlindBoxModule::set_blindbox_caller(
            Origin::root(),
            ALICE
        ));

        assert_ok!(BlindBoxModule::generate_blindbox_ids(
            Origin::signed(ALICE),
            10
        ));

        assert_noop!(
            BlindBoxModule::generate_blindbox_ids(Origin::signed(ALICE), 10),
            Error::<Runtime>::BlindBoxesStillAvailable
        );
    });
}

#[test]
fn open_blind_box_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(BlindBoxModule::set_blindbox_caller(
            Origin::root(),
            ALICE
        ));

        assert_ok!(BlindBoxModule::generate_blindbox_ids(
            Origin::signed(ALICE),
            10
        ));

        // TODO: should retrieve a newly generate blindboxId
        let lst_event = last_event();

        let blindBoxes = BlindBoxModule::get_blindboxes(0);

        let event = Event::blindbox(crate::Event::BlindBoxOpened(BLINDBOX_ID));
        assert_eq!(last_event(), event);
    });
}

#[test]
fn open_blind_box_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(BlindBoxModule::set_blindbox_caller(
            Origin::root(),
            ALICE
        ));

        assert_ok!(BlindBoxModule::generate_blindbox_ids(
            Origin::signed(ALICE),
            10
        ));

        assert_noop!(
            BlindBoxModule::open_blind_box(Origin::signed(ALICE), SUCCESS_BLINDBOX_ID),
            Error::<Runtime>::BlindBoxDoesNotExist
        );
    });
}