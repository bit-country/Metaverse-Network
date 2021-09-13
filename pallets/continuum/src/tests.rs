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

#[test]
// Private test continuum pallet to ensure mocking and unit tests working
fn test() {
    ExtBuilder::default()
        .build()
        .execute_with(|| assert_eq!(0, 0));
}

#[test]
fn find_neighborhood_spot_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        let continuum_spot = ContinuumSpot {
            x: 0,
            y: 0,
            country: ALICE_COUNTRY_ID,
        };

        let correct_neighbors = vec![
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];

        let neighbors = continuum_spot.find_neighbour();
        let total_matching = neighbors
            .iter()
            .zip(&correct_neighbors)
            .filter(|&(a, b)| a.0 == b.0 && a.1 == b.1)
            .count();

        assert_eq!(8, total_matching)
    })
}

#[test]
fn register_interest_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);

        System::set_block_number(1);
        assert_ok!(ContinuumModule::register_interest(
            origin,
            ALICE_COUNTRY_ID,
            (0, 0)
        ));
        assert_eq!(
            last_event(),
            Event::continuum(crate::Event::NewExpressOfInterestAdded(ALICE, 0))
        )
    })
}

#[test]
fn register_interest_should_not_work_for_non_owner() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);
        System::set_block_number(1);
        assert_noop!(
            ContinuumModule::register_interest(origin, BOB_COUNTRY_ID, (0, 0)),
            Error::<Runtime>::NoPermission
        );
    })
}

#[test]
fn rotate_session_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = Origin::signed(ALICE);
        let bob = Origin::signed(BOB);

        System::set_block_number(1);
        assert_ok!(ContinuumModule::register_interest(
            alice,
            ALICE_COUNTRY_ID,
            (0, 0)
        ));
        assert_ok!(ContinuumModule::register_interest(
            bob,
            BOB_COUNTRY_ID,
            (0, 0)
        ));

        run_to_block(10);

        let auction_slot = AuctionSlot {
            spot_id: 0,
            participants: vec![ALICE, BOB],
            active_session_index: 1,
            status: ContinuumAuctionSlotStatus::AcceptParticipates,
        };

        let auction_slots: Vec<AuctionSlot<BlockNumber, AccountId>> = vec![auction_slot];

        let active_auctions: Option<Vec<AuctionSlot<BlockNumber, AccountId>>> =
            ContinuumModule::get_active_auction_slots(10);
        match active_auctions {
            Some(a) => {
                // Verify EOI move to Auction Slots
                assert_eq!(a[0].spot_id, auction_slots[0].spot_id);
            }
            _ => {
                // Should fail test
                assert_eq!(0, 1);
            }
        }

        //Test starting GNP should work
        run_to_block(20);
        let gnp_started_auctions: Option<Vec<AuctionSlot<BlockNumber, AccountId>>> =
            ContinuumModule::get_active_gnp_slots(20);
        match gnp_started_auctions {
            Some(a) => {
                // Verify Auction slots move to good neighborhood protocol Slots
                assert_eq!(a[0].spot_id, a[0].spot_id);
                assert_eq!(a[0].status, ContinuumAuctionSlotStatus::GNPStarted);

                // Test start referendum should work
                let status = ContinuumModule::referendum_status(0);
                match status {
                    Ok(r) => {
                        assert_eq!(r.end, 30);
                    }
                    _ => {
                        assert_eq!(0, 1);
                    }
                }
            }
            _ => {
                // Should fail test
                assert_eq!(0, 1);
            }
        }

        // Try vote while referendum is active
        assert_ok!(ContinuumModule::try_vote(
            &CHARLIE,
            0,
            AccountVote::Standard {
                vote: Vote {
                    nay: true,
                    who: ALICE
                }
            }
        ));

        // ALICE should be removed from participants list
        // Conduct the referendum and finalise vote
        run_to_block(30);

        let finalised_votes: Option<Vec<AuctionSlot<BlockNumber, AccountId>>> =
            ContinuumModule::get_active_auction_slots(20);
        match finalised_votes {
            Some(v) => {
                // Only BOB is eligible
                assert_eq!(v[0].participants.len(), 1);
                // Confirm if it's BOB
                assert_eq!(v[0].participants[0], BOB);
            }
            _ => {}
        }
    })
}

#[test]
fn buy_now_continuum_should_fail_when_not_owner() {
    ExtBuilder::default().build().execute_with(|| {
        let root = Origin::root();

        // Enable Allow BuyNow
        assert_ok!(ContinuumModule::set_allow_buy_now(root, true));
        assert_noop!(
            ContinuumModule::buy_continuum_spot(Origin::signed(ALICE), (0, 1), BOB_COUNTRY_ID),
            Error::<Runtime>::NoPermission
        );
    })
}

#[test]
fn buy_now_continuum_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        let root = Origin::root();

        // Enable Allow BuyNow
        assert_ok!(ContinuumModule::set_allow_buy_now(root, true));
        assert_ok!(ContinuumModule::buy_continuum_spot(
            Origin::signed(ALICE),
            (0, 1),
            ALICE_COUNTRY_ID
        ));
    })
}

#[test]
fn buy_now_continuum_should_fail_if_buy_now_setting_is_disabled() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            ContinuumModule::buy_continuum_spot(Origin::signed(ALICE), (0, 1), ALICE_COUNTRY_ID),
            Error::<Runtime>::ContinuumBuyNowIsDisabled
        );
    })
}
