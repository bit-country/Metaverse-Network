#![cfg(test)]

use crate::{Config, Pallet};
use frame_support::{
    impl_outer_event, impl_outer_origin, impl_outer_dispatch, parameter_types, traits::EnsureOrigin, weights::Weight,
};
use frame_system as system;
use frame_system::RawOrigin;
use sp_core::{sr25519, Pair, H256};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
    ModuleId,
};
use primitives::{CountryId, CurrencyId, AuctionId};

pub type AccountId = u128;
pub type BlockNumber = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const COUNTRY_ID: CountryId = 0;
pub const COUNTRY_ID_NOT_EXIST: CountryId = 1;
pub const BCG: CurrencyId = 0;

#[derive(Clone, Eq, PartialEq)]
pub struct Runtime;

use crate as continuum;

impl_outer_origin! {
	pub enum Origin for Runtime {}
}

impl_outer_event! {
	pub enum TestEvent for Runtime {
		frame_system<T>,
		country<T>,
	}
}

impl_outer_dispatch! {
	pub enum Call for Runtime where origin: Origin {
		frame_system::System,
	}
}


// Configure a mock runtime to test the pallet.

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: u32 = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}


impl frame_system::Config for Runtime {
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Call = Call;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = TestEvent;
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type PalletInfo = ();
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
    type BlockExecutionWeight = ();
    type ExtrinsicBaseWeight = ();
    type MaximumExtrinsicWeight = ();
    type BaseCallFilter = ();
    type SystemWeightInfo = ();
}

pub type System = frame_system::Module<Runtime>;

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

pub struct Handler;

impl AuctionHandler<AccountId, Balance, BlockNumber, AuctionId> for Handler {
    fn on_new_bid(now: BlockNumber, id: AuctionId, new_bid: (AccountId, Balance), last_bid: Option<(AccountId, Balance)>) -> OnNewBidResult<BlockNumber> {
        //Test with Alice bid
        if new_bid.0 == ALICE {
            OnNewBidResult {
                accept_bid: true,
                auction_end_change: Change::NoChange,
            }
        } else {
            OnNewBidResult {
                accept_bid: false,
                auction_end_change: Change::NoChange,
            }
        }
    }

    fn on_auction_ended(_id: AuctionId, _winner: Option<(AccountId, Balance)>) {}
}

parameter_types! {
    pub const ContinuumTreasuryModuleId: ModuleId = ModuleId(*b"bit/ctmu");
    pub const AuctionTimeToClose: u32 = 100800; //Default 100800 Blocks
    pub const SessionDuration: BlockNumber = 43200; //Default 43200 Blocks
    pub const SpotAuctionChillingDuration: BlockNumber = 43200; //Default 43200 Blocks
}

impl pallet_auction::Config for Runtime {
    type Event = TestEvent;
    type AuctionTimeToClose = AuctionTimeToClose;
    type AuctionId = AuctionId;
    type Handler = Handler;
    type Currency = Balances;
}

impl Config for Runtime {
    type Event = TestEvent;
    type SessionDuration = SessionDuration;
    type SpotAuctionChillingDuration = SpotAuctionChillingDuration;
    type EmergencyOrigin = EnsureRootOrHalfGeneralCouncil;
    type AuctionHandler = Auction;
    type AuctionDuration = SpotAuctionChillingDuration;
    type ContinuumTreasury = ContinuumTreasuryModuleId;
    type Currency = Balances;
}

pub type ContinuumModule = Pallet<Runtime>;

use frame_system::Call as SystemCall;
use auction_manager::{AuctionHandler, OnNewBidResult, Change};

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
        Auction: pallet_auction::{Module, Call ,Storage, Event<T>},
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
        let t = frame_system::GenesisConfig::default()
            .build_storage::<Runtime>()
            .unwrap();

        pallet_balances::GenesisConfig::<Runtime> {
            balances: vec![(ALICE, 100000)],
        }
            .assimilate_storage(&mut t)
            .unwrap();

        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}

pub fn last_event() -> TestEvent {
    frame_system::Module::<Runtime>::events()
        .pop()
        .expect("Event expected")
        .event
}