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

use crate as continuum;
use super::*;
use frame_support::{
    construct_runtime, parameter_types, ord_parameter_types, weights::Weight,
};
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup, ModuleId};
use primitives::{CurrencyId, Amount, AssetId};
use frame_system::{EnsureSignedBy, EnsureRoot};
use auction_manager::{AuctionHandler, OnNewBidResult, Change, AuctionInfo, Auction};
use frame_support::pallet_prelude::{MaybeSerializeDeserialize, Hooks, GenesisBuild};
use frame_support::sp_runtime::traits::AtLeast32Bit;

parameter_types! {
    pub const BlockHashCount: u32 = 256;
    pub const MaximumBlockWeight: u32 = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
}

// Configure a mock runtime to test the pallet.

pub type AccountId = u128;
pub type AuctionId = u64;
pub type Balance = u64;
pub type CountryId = u64;
pub type BlockNumber = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CHARLIE: AccountId = 3;
pub const CLASS_ID: u32 = 0;
pub const COLLECTION_ID: u64 = 0;
pub const ALICE_COUNTRY_ID: CountryId = 1;
pub const BOB_COUNTRY_ID: CountryId = 2;

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
    type BaseCallFilter = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
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
}

pub struct MockAuctionManager;

impl Auction<AccountId, BlockNumber> for MockAuctionManager {
    type Balance = Balance;

    fn auction_info(id: u64) -> Option<AuctionInfo<u128, Self::Balance, u64>> {
        todo!()
    }

    fn update_auction(id: u64, info: AuctionInfo<u128, Self::Balance, u64>) -> DispatchResult {
        todo!()
    }

    fn new_auction(recipient: u128, initial_amount: Self::Balance, start: u64, end: Option<u64>) -> Result<u64, DispatchError> {
        todo!()
    }

    fn create_auction(auction_type: AuctionType, item_id: ItemId, end: Option<u64>, recipient: u128, initial_amount: Self::Balance, start: u64) -> Result<u64, DispatchError> {
        todo!()
    }

    fn remove_auction(id: u64, item_id: ItemId) {
        todo!()
    }

    fn auction_bid_handler(_now: u64, id: u64, new_bid: (u128, Self::Balance), last_bid: Option<(u128, Self::Balance)>) -> DispatchResult {
        todo!()
    }
    
    // fn swap_bidders(new_bidder: &u128, last_bidder: Option<&u128>) {
    //     todo!()
    // }

    fn check_item_in_auction(asset_id: AssetId) -> bool {
        todo!()
    }
}

parameter_types! {
    pub const ContinuumTreasuryModuleId: ModuleId = ModuleId(*b"bit/ctmu");
    pub const AuctionTimeToClose: u32 = 10; //Default 100800 Blocks
    pub const SessionDuration: BlockNumber = 10; //Default 43200 Blocks
    pub const SpotAuctionChillingDuration: BlockNumber = 10; //Default 43200 Blocks
}

pub struct CountryInfoSource {}

impl BCCountry<AccountId> for CountryInfoSource {
    fn check_ownership(who: &AccountId, country_id: &CountryId) -> bool {
        match *who {
            ALICE => *country_id == ALICE_COUNTRY_ID,
            BOB => *country_id == BOB_COUNTRY_ID,
            _ => false,
        }
    }

    fn get_country(country_id: CountryId) -> Option<Country<AccountId>> {
        None
    }

    fn get_country_token(country_id: CountryId) -> Option<CurrencyId> {
        None
    }
}

impl Config for Runtime {
    type Event = Event;
    type SessionDuration = SessionDuration;
    type SpotAuctionChillingDuration = SpotAuctionChillingDuration;
    type EmergencyOrigin = EnsureSignedBy<One, AccountId>;
    type AuctionHandler = MockAuctionManager;
    type AuctionDuration = SpotAuctionChillingDuration;
    type ContinuumTreasury = ContinuumTreasuryModuleId;
    type Currency = Balances;
    type CountryInfoSource = CountryInfoSource;
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
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
        Continuum: continuum::{Module, Call ,Storage, Event<T>},
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
            balances: vec![(ALICE, 100000), (BOB, 500)],
        }
            .assimilate_storage(&mut t)
            .unwrap();

        continuum::GenesisConfig::<Runtime> {
            initial_active_session: 0,
            initial_auction_rate: 5,
            initial_max_bound: (-100, 100),
            spot_price: 10000,
        }
            .assimilate_storage((&mut t))
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