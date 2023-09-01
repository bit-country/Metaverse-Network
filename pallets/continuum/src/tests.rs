// This file is part of Metaverse.Network & Bit.Country.

// Copyright (C) 2020-2022 Metaverse.Network & Bit.Country .
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

use frame_support::{assert_noop, assert_ok};
use sp_runtime::traits::BadOrigin;

use core_primitives::TokenType;
use mock::BlockNumber as MBlockNumber;
use mock::{RuntimeEvent, *};

use super::*;

#[test]
fn find_neighborhood_spot_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let continuum_spot = ContinuumSpot {
			x: 0,
			y: 0,
			metaverse_id: ALICE_METAVERSE_ID,
		};

		let correct_neighbors = vec![(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)];

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
fn issue_continuum_spot_should_fail_when_no_root() {
	ExtBuilder::default().build().execute_with(|| {
		let root = RuntimeOrigin::root();
		// Enable Allow BuyNow
		assert_ok!(ContinuumModule::set_allow_buy_now(root, true));

		assert_noop!(
			ContinuumModule::issue_map_slot(
				RuntimeOrigin::signed(ALICE),
				CONTINUUM_MAP_COORDINATE,
				TokenType::Transferable
			),
			BadOrigin
		);
	})
}

#[test]
fn issue_continuum_spot_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let root = RuntimeOrigin::root();
		let treasury = <Runtime as Config>::ContinuumTreasury::get().into_account_truncating();
		assert_ok!(ContinuumModule::issue_map_slot(
			root,
			CONTINUUM_MAP_COORDINATE,
			TokenType::Transferable
		));

		let map_spot = MapSpot {
			metaverse_id: None,
			owner: treasury,
			slot_type: TokenType::Transferable,
		};

		assert_eq!(ContinuumModule::get_map_spot(CONTINUUM_MAP_COORDINATE), Some(map_spot))
	})
}

#[test]
fn create_buy_now_for_continuum_spot_should_fail_when_no_root() {
	ExtBuilder::default().build().execute_with(|| {
		let root = RuntimeOrigin::root();
		// Enable Allow BuyNow
		assert_ok!(ContinuumModule::set_allow_buy_now(root.clone(), true));
		assert_ok!(ContinuumModule::issue_map_slot(
			root,
			CONTINUUM_MAP_COORDINATE,
			TokenType::Transferable
		));
		assert_noop!(
			ContinuumModule::create_new_auction(
				RuntimeOrigin::signed(ALICE),
				CONTINUUM_MAP_COORDINATE,
				AuctionType::BuyNow,
				100,
				10
			),
			BadOrigin
		);
	})
}

#[test]
fn create_buy_now_continuum_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let root = RuntimeOrigin::root();

		// Enable Allow BuyNow
		assert_ok!(ContinuumModule::set_allow_buy_now(root.clone(), true));
		assert_ok!(ContinuumModule::issue_map_slot(
			root.clone(),
			CONTINUUM_MAP_COORDINATE,
			TokenType::Transferable
		));

		assert_ok!(ContinuumModule::create_new_auction(
			root.clone(),
			CONTINUUM_MAP_COORDINATE,
			AuctionType::BuyNow,
			100,
			10
		));
	})
}

#[test]
fn create_auction_continuum_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let root = RuntimeOrigin::root();

		// Enable Allow BuyNow
		assert_ok!(ContinuumModule::set_allow_buy_now(root.clone(), true));
		assert_ok!(ContinuumModule::issue_map_slot(
			root.clone(),
			CONTINUUM_MAP_COORDINATE,
			TokenType::Transferable
		));

		assert_ok!(ContinuumModule::create_new_auction(
			root.clone(),
			CONTINUUM_MAP_COORDINATE,
			AuctionType::Auction,
			100,
			10
		));
	})
}

#[test]
fn buy_now_continuum_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let root = RuntimeOrigin::root();

		// Enable Allow BuyNow
		assert_ok!(ContinuumModule::set_allow_buy_now(root.clone(), true));
		assert_ok!(ContinuumModule::issue_map_slot(
			root.clone(),
			CONTINUUM_MAP_COORDINATE,
			TokenType::Transferable
		));

		assert_ok!(ContinuumModule::create_new_auction(
			root.clone(),
			CONTINUUM_MAP_COORDINATE,
			AuctionType::BuyNow,
			100,
			10
		));

		assert_ok!(ContinuumModule::buy_map_spot(
			RuntimeOrigin::signed(ALICE),
			1,
			100,
			ALICE_METAVERSE_ID
		));

		let treasury = <Runtime as Config>::ContinuumTreasury::get().into_account_truncating();
		ContinuumModule::transfer_spot(CONTINUUM_MAP_COORDINATE, treasury, (ALICE, ALICE_METAVERSE_ID));

		// Ensure Metaverse leading bid has no record of this spot
		assert_eq!(
			MetaverseLeadingBid::<Runtime>::iter_prefix(CONTINUUM_MAP_COORDINATE).count(),
			0
		)
	})
}

#[test]
fn bid_auction_continuum_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let root = RuntimeOrigin::root();

		assert_ok!(ContinuumModule::issue_map_slot(
			root.clone(),
			CONTINUUM_MAP_COORDINATE,
			TokenType::Transferable
		));

		assert_ok!(ContinuumModule::create_new_auction(
			root.clone(),
			CONTINUUM_MAP_COORDINATE,
			AuctionType::Auction,
			100,
			10
		));

		assert_ok!(ContinuumModule::bid_map_spot(
			RuntimeOrigin::signed(ALICE),
			1,
			200,
			ALICE_METAVERSE_ID
		));

		assert!(MetaverseLeadingBid::<Runtime>::contains_key(
			CONTINUUM_MAP_COORDINATE,
			ALICE_METAVERSE_ID
		));

		let treasury = <Runtime as Config>::ContinuumTreasury::get().into_account_truncating();
		ContinuumModule::transfer_spot(CONTINUUM_MAP_COORDINATE, treasury, (ALICE, ALICE_METAVERSE_ID));

		// Ensure Metaverse leading bid has no record of this spot
		assert_eq!(
			MetaverseLeadingBid::<Runtime>::iter_prefix(CONTINUUM_MAP_COORDINATE).count(),
			0
		)
	})
}

#[test]
fn buy_now_continuum_should_fail_if_already_got_spot() {
	ExtBuilder::default().build().execute_with(|| {
		let root = RuntimeOrigin::root();

		MetaverseMap::<Runtime>::insert(ALICE_METAVERSE_ID, (0, 1));

		// Enable Allow BuyNow
		assert_ok!(ContinuumModule::set_allow_buy_now(root.clone(), true));
		assert_ok!(ContinuumModule::issue_map_slot(
			root.clone(),
			CONTINUUM_MAP_COORDINATE,
			TokenType::Transferable
		));

		assert_ok!(ContinuumModule::create_new_auction(
			root.clone(),
			CONTINUUM_MAP_COORDINATE,
			AuctionType::BuyNow,
			100,
			10
		));

		assert_noop!(
			ContinuumModule::buy_map_spot(RuntimeOrigin::signed(ALICE), 1, 100, ALICE_METAVERSE_ID),
			Error::<Runtime>::MetaverseAlreadyGotSpot
		);
	})
}

#[test]
fn bid_auction_continuum_should_fail_if_already_got_spot() {
	ExtBuilder::default().build().execute_with(|| {
		let root = RuntimeOrigin::root();

		MetaverseMap::<Runtime>::insert(ALICE_METAVERSE_ID, (0, 1));

		// Enable Allow BuyNow
		assert_ok!(ContinuumModule::set_allow_buy_now(root.clone(), true));
		assert_ok!(ContinuumModule::issue_map_slot(
			root.clone(),
			CONTINUUM_MAP_COORDINATE,
			TokenType::Transferable
		));

		assert_ok!(ContinuumModule::create_new_auction(
			root.clone(),
			CONTINUUM_MAP_COORDINATE,
			AuctionType::Auction,
			100,
			10
		));

		assert_noop!(
			ContinuumModule::bid_map_spot(RuntimeOrigin::signed(ALICE), 1, 100, ALICE_METAVERSE_ID),
			Error::<Runtime>::MetaverseAlreadyGotSpot
		);
	})
}

#[test]
fn buy_now_continuum_should_fail_if_has_not_deploy_land() {
	ExtBuilder::default().build().execute_with(|| {
		let root = RuntimeOrigin::root();

		// Enable Allow BuyNow
		assert_ok!(ContinuumModule::set_allow_buy_now(root.clone(), true));
		assert_ok!(ContinuumModule::issue_map_slot(
			root.clone(),
			CONTINUUM_MAP_COORDINATE,
			TokenType::Transferable
		));

		assert_ok!(ContinuumModule::create_new_auction(
			root.clone(),
			CONTINUUM_MAP_COORDINATE,
			AuctionType::BuyNow,
			100,
			10
		));

		assert_noop!(
			ContinuumModule::buy_map_spot(RuntimeOrigin::signed(CHARLIE), 1, 100, CHARLIE_METAVERSE_ID),
			Error::<Runtime>::MetaverseHasNotDeployedAnyLand
		);
	})
}

#[test]
fn bid_auction_continuum_should_fail_if_has_not_deployed_land() {
	ExtBuilder::default().build().execute_with(|| {
		let root = RuntimeOrigin::root();

		assert_ok!(ContinuumModule::issue_map_slot(
			root.clone(),
			CONTINUUM_MAP_COORDINATE,
			TokenType::Transferable
		));

		assert_ok!(ContinuumModule::create_new_auction(
			root.clone(),
			CONTINUUM_MAP_COORDINATE,
			AuctionType::Auction,
			100,
			10
		));

		assert_noop!(
			ContinuumModule::bid_map_spot(RuntimeOrigin::signed(CHARLIE), 1, 100, CHARLIE_METAVERSE_ID),
			Error::<Runtime>::MetaverseHasNotDeployedAnyLand
		);
	})
}

#[test]
fn bid_auction_continuum_should_fail_if_has_leading_bid() {
	ExtBuilder::default().build().execute_with(|| {
		let root = RuntimeOrigin::root();

		MetaverseLeadingBid::<Runtime>::insert((0, 1), ALICE_METAVERSE_ID, ());

		// Enable Allow BuyNow
		assert_ok!(ContinuumModule::set_allow_buy_now(root.clone(), true));
		assert_ok!(ContinuumModule::issue_map_slot(
			root.clone(),
			CONTINUUM_MAP_COORDINATE,
			TokenType::Transferable
		));

		assert_ok!(ContinuumModule::create_new_auction(
			root.clone(),
			CONTINUUM_MAP_COORDINATE,
			AuctionType::Auction,
			100,
			10
		));

		assert_noop!(
			ContinuumModule::bid_map_spot(RuntimeOrigin::signed(ALICE), 1, 100, ALICE_METAVERSE_ID),
			Error::<Runtime>::MetaverseHasBidLeading
		);
	})
}

#[test]
fn buy_now_continuum_should_fail_if_has_any_leading_bid() {
	ExtBuilder::default().build().execute_with(|| {
		let root = RuntimeOrigin::root();

		MetaverseLeadingBid::<Runtime>::insert((0, 1), ALICE_METAVERSE_ID, ());

		// Enable Allow BuyNow
		assert_ok!(ContinuumModule::set_allow_buy_now(root.clone(), true));
		assert_ok!(ContinuumModule::issue_map_slot(
			root.clone(),
			CONTINUUM_MAP_COORDINATE,
			TokenType::Transferable
		));

		assert_ok!(ContinuumModule::create_new_auction(
			root.clone(),
			CONTINUUM_MAP_COORDINATE,
			AuctionType::BuyNow,
			100,
			10
		));

		assert_noop!(
			ContinuumModule::buy_map_spot(RuntimeOrigin::signed(ALICE), 1, 100, ALICE_METAVERSE_ID),
			Error::<Runtime>::MetaverseHasBidLeading
		);
	})
}

#[test]
fn metaverse_leading_bid_inserted_on_new_bid() {
	ExtBuilder::default().build().execute_with(|| {
		let root = RuntimeOrigin::root();

		// Enable Allow BuyNow
		assert_ok!(ContinuumModule::set_allow_buy_now(root.clone(), true));
		assert_ok!(ContinuumModule::issue_map_slot(
			root.clone(),
			CONTINUUM_MAP_COORDINATE,
			TokenType::Transferable
		));

		assert_ok!(ContinuumModule::create_new_auction(
			root.clone(),
			CONTINUUM_MAP_COORDINATE,
			AuctionType::Auction,
			100,
			10
		));

		assert_ok!(ContinuumModule::bid_map_spot(
			RuntimeOrigin::signed(ALICE),
			1,
			100,
			ALICE_METAVERSE_ID
		));

		assert!(MetaverseLeadingBid::<Runtime>::contains_key(
			CONTINUUM_MAP_COORDINATE,
			ALICE_METAVERSE_ID
		));
	})
}

#[test]
fn bid_auction_continuum_should_replace_another_leading_bid() {
	ExtBuilder::default().build().execute_with(|| {
		let root = RuntimeOrigin::root();

		assert_ok!(ContinuumModule::issue_map_slot(
			root.clone(),
			CONTINUUM_MAP_COORDINATE,
			TokenType::Transferable
		));

		assert_ok!(ContinuumModule::create_new_auction(
			root.clone(),
			CONTINUUM_MAP_COORDINATE,
			AuctionType::Auction,
			100,
			10
		));

		assert_ok!(ContinuumModule::bid_map_spot(
			RuntimeOrigin::signed(ALICE),
			1,
			200,
			ALICE_METAVERSE_ID
		));

		assert!(MetaverseLeadingBid::<Runtime>::contains_key(
			CONTINUUM_MAP_COORDINATE,
			ALICE_METAVERSE_ID
		));

		assert_ok!(ContinuumModule::bid_map_spot(
			RuntimeOrigin::signed(BOB),
			1,
			300,
			BOB_METAVERSE_ID
		));

		assert!(MetaverseLeadingBid::<Runtime>::contains_key(
			CONTINUUM_MAP_COORDINATE,
			BOB_METAVERSE_ID
		));

		// Alice leading bid should be removed
		assert!(!MetaverseLeadingBid::<Runtime>::contains_key(
			CONTINUUM_MAP_COORDINATE,
			ALICE_METAVERSE_ID
		));

		// Ensure Alice can bid again
		assert_ok!(ContinuumModule::bid_map_spot(
			RuntimeOrigin::signed(ALICE),
			1,
			400,
			ALICE_METAVERSE_ID
		));

		assert!(MetaverseLeadingBid::<Runtime>::contains_key(
			CONTINUUM_MAP_COORDINATE,
			ALICE_METAVERSE_ID
		));

		// Bob leading bid should be removed
		assert!(!MetaverseLeadingBid::<Runtime>::contains_key(
			CONTINUUM_MAP_COORDINATE,
			BOB_METAVERSE_ID
		));

		let treasury = <Runtime as Config>::ContinuumTreasury::get().into_account_truncating();
		ContinuumModule::transfer_spot(CONTINUUM_MAP_COORDINATE, treasury, (BOB, BOB_METAVERSE_ID));

		// Ensure Metaverse leading bid has no record of this spot
		assert_eq!(
			MetaverseLeadingBid::<Runtime>::iter_prefix(CONTINUUM_MAP_COORDINATE).count(),
			0
		)
	})
}
