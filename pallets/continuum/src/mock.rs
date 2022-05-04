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

use frame_support::pallet_prelude::{GenesisBuild, Hooks};
use frame_support::{construct_runtime, ord_parameter_types, parameter_types, PalletId};
use frame_system::EnsureSignedBy;
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup, Perbill};

use auction_manager::{Auction, AuctionInfo, CheckAuctionItemHandler, ListingLevel};
use core_primitives::{MetaverseInfo, MetaverseTrait};
use primitives::{ClassId, FungibleTokenId};

use crate as continuum;

use super::*;

parameter_types! {
	pub const BlockHashCount: u32 = 256;
	pub const MaximumBlockWeight: u32 = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
}

// Configure a mock runtime to test the pallet.

pub type AccountId = u128;
pub type Balance = u64;
pub type MetaverseId = u64;
pub type BlockNumber = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CHARLIE: AccountId = 3;
pub const ALICE_METAVERSE_ID: MetaverseId = 1;
pub const BOB_METAVERSE_ID: MetaverseId = 2;
pub const CHARLIE_METAVERSE_ID: MetaverseId = 3;

pub const ALICE_METAVERSE_FUND: AccountId = 100;
pub const BOB_METAVERSE_FUND: AccountId = 101;
pub const GENERAL_METAVERSE_FUND: AccountId = 102;

ord_parameter_types! {
	pub const One: AccountId = ALICE;
}

impl frame_system::Config for Runtime {
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Call = Call;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type BlockWeights = ();
	type BlockLength = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = frame_support::traits::Everything;
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type MaxLocks = ();
	type WeightInfo = ();
	type MaxReserves = ();
	type ReserveIdentifier = ();
}

pub struct MockAuctionManager;

impl Auction<AccountId, BlockNumber> for MockAuctionManager {
	type Balance = Balance;

	fn auction_info(_id: u64) -> Option<AuctionInfo<u128, Self::Balance, u64>> {
		None
	}

	fn update_auction(_id: u64, _info: AuctionInfo<u128, Self::Balance, u64>) -> DispatchResult {
		Ok(())
	}

	fn new_auction(
		_recipient: u128,
		_initial_amount: Self::Balance,
		_start: u64,
		_end: Option<u64>,
	) -> Result<u64, DispatchError> {
		Ok(1)
	}

	fn create_auction(
		_auction_type: AuctionType,
		_item_id: ItemId,
		_end: Option<u64>,
		_recipient: u128,
		_initial_amount: Self::Balance,
		_start: u64,
		_listing_level: ListingLevel<AccountId>,
		_listing_fee: Perbill,
	) -> Result<u64, DispatchError> {
		Ok(1)
	}

	fn remove_auction(_id: u64, _item_id: ItemId) {}

	fn auction_bid_handler(
		_now: u64,
		_id: u64,
		_new_bid: (u128, Self::Balance),
		_last_bid: Option<(u128, Self::Balance)>,
	) -> DispatchResult {
		Ok(())
	}

	fn local_auction_bid_handler(
		_now: u64,
		_id: u64,
		_new_bid: (u128, Self::Balance),
		_last_bid: Option<(u128, Self::Balance)>,
		_social_currency_id: FungibleTokenId,
	) -> DispatchResult {
		Ok(())
	}

	fn collect_royalty_fee(
		_high_bid_price: &Self::Balance,
		_high_bidder: &u128,
		_asset_id: &(u32, u64),
		_social_currency_id: FungibleTokenId,
		_listing_level: ListingLevel<AccountId>,
		_listing_fee: Perbill,
	) -> DispatchResult {
		Ok(())
	}
}

impl CheckAuctionItemHandler for MockAuctionManager {
	fn check_item_in_auction(_item_id: ItemId) -> bool {
		return false;
	}
}

parameter_types! {
	pub const ContinuumTreasuryPalletId: PalletId = PalletId(*b"bit/ctmu");
	// Default 100800 Blocks
	pub const AuctionTimeToClose: u32 = 10;
	// Default 43200 Blocks
	pub const SessionDuration: BlockNumber = 10;
	// Default 43200 Blocks
	pub const SpotAuctionChillingDuration: BlockNumber = 10;
}

pub struct MetaverseInfoSource {}

impl MetaverseTrait<AccountId> for MetaverseInfoSource {
	fn check_ownership(who: &AccountId, metaverse_id: &MetaverseId) -> bool {
		match *who {
			ALICE => *metaverse_id == ALICE_METAVERSE_ID,
			BOB => *metaverse_id == BOB_METAVERSE_ID,
			CHARLIE => *metaverse_id == CHARLIE_METAVERSE_ID,
			_ => false,
		}
	}

	fn get_metaverse(_metaverse_id: u64) -> Option<MetaverseInfo<u128>> {
		None
	}

	fn get_metaverse_token(_metaverse_id: u64) -> Option<FungibleTokenId> {
		None
	}

	fn update_metaverse_token(_metaverse_id: u64, _currency_id: FungibleTokenId) -> Result<(), DispatchError> {
		Ok(())
	}

	fn get_metaverse_land_class(metaverse_id: MetaverseId) -> ClassId {
		15u32
	}

	fn get_metaverse_estate_class(metaverse_id: MetaverseId) -> ClassId {
		16u32
	}

	fn get_metaverse_marketplace_listing_fee(metaverse_id: MetaverseId) -> Perbill {
		Perbill::from_percent(1u32)
	}

	fn get_metaverse_treasury(metaverse_id: MetaverseId) -> AccountId {
		match metaverse_id {
			ALICE_METAVERSE_ID => return ALICE_METAVERSE_FUND,
			BOB_METAVERSE_ID => return BOB_METAVERSE_FUND,
			_ => GENERAL_METAVERSE_FUND,
		}
	}
}

impl Config for Runtime {
	type Event = Event;
	type SessionDuration = SessionDuration;
	type SpotAuctionChillingDuration = SpotAuctionChillingDuration;
	type EmergencyOrigin = EnsureSignedBy<One, AccountId>;
	type AuctionHandler = MockAuctionManager;
	type AuctionDuration = SpotAuctionChillingDuration;
	type ContinuumTreasury = ContinuumTreasuryPalletId;
	type Currency = Balances;
	type MetaverseInfoSource = MetaverseInfoSource;
}

pub type ContinuumModule = Pallet<Runtime>;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Continuum: continuum::{Pallet, Call ,Storage, Event<T>},
	}
);

pub struct ExtBuilder;

impl Default for ExtBuilder {
	fn default() -> Self {
		ExtBuilder
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		self.build_with_block_number(1)
	}

	pub fn build_with_block_number(self, block_number: u64) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: vec![(ALICE, 100000), (BOB, 500), (CHARLIE, 100000)],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		continuum::GenesisConfig::<Runtime> {
			initial_active_session: 0,
			initial_auction_rate: 5,
			initial_max_bound: (-100, 100),
			spot_price: 100,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(block_number));
		ext
	}
}

pub fn last_event() -> Event {
	frame_system::Pallet::<Runtime>::events()
		.pop()
		.expect("Event expected")
		.event
}

fn next_block() {
	System::set_block_number(System::block_number() + 1);
	ContinuumModule::on_initialize(System::block_number());
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		next_block();
	}
}
