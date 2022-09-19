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

use frame_support::{assert_err, assert_noop, assert_ok};
use orml_nft::Tokens;
use sp_runtime::traits::BadOrigin;
use sp_std::default::Default;

use auction_manager::ListingLevel;
use core_primitives::{Attributes, CollectionType, TokenType};
use mock::{Event, *};
use primitives::{CampaignInfo, GroupCollectionId};

use super::*;

#[test]
fn basic_setup_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(System::block_number(), 1);
		assert_eq!(Reward::campaigns(0), None);
	});
}

#[test]
fn create_campaign_works() {
	ExtBuilder::default().build().execute_with(|| {
		let campaign_id = 0;
		assert_ok!(Reward::create_campaign(Origin::signed(ALICE), ALICE, 10, 10));

		let campaign_info = CampaignInfo {
			creator: ALICE,
			reward: 10,
			claimed: 0,
			end: 10,
			cap: 10,
			trie_index: 0,
		};
		assert_eq!(Reward::campaigns(campaign_id), Some(campaign_info))
	});
}
