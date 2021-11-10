#![cfg(test)]

use super::*;
use crate as estate;
// use crate::{Config, Module};
use bc_primitives::*;
// use bit_country_primitives::*;
// // use sp_std::vec::Vec;
use auction_manager::{Auction, AuctionHandler, AuctionInfo, Change, CheckAuctionItemHandler, OnNewBidResult};
use frame_support::ensure;
use frame_support::pallet_prelude::{GenesisBuild, Hooks, MaybeSerializeDeserialize};
use frame_support::sp_runtime::traits::AtLeast32Bit;
use frame_support::{
	construct_runtime, ord_parameter_types, parameter_types, traits::EnsureOrigin, weights::Weight, PalletId,
};
use frame_system::{ensure_root, ensure_signed};
use frame_system::{EnsureRoot, EnsureSignedBy};
use primitives::{Amount, CurrencyId, FungibleTokenId};
use sp_core::{
	u32_trait::{_1, _2, _3, _4, _5},
	H256,
};
use sp_runtime::{testing::Header, traits::IdentityLookup, DispatchError, Perbill};

pub type AccountId = u128;
pub type AuctionId = u64;
pub type Balance = u128;
pub type MetaverseId = u64;
pub type BlockNumber = u64;
pub type LandId = u64;
pub type EstateId = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 5;
pub const BENEFICIARY_ID: AccountId = 99;
pub const METAVERSE_ID: MetaverseId = 0;
pub const DOLLARS: Balance = 1_000_000_000_000_000_000;
pub const ALICE_METAVERSE_ID: MetaverseId = 1;
pub const BOB_METAVERSE_ID: MetaverseId = 2;
pub const MAX_BOUND: (i32, i32) = (-100, 100);
pub const COORDINATE_IN_1: (i32, i32) = (-10, 10);
pub const COORDINATE_IN_2: (i32, i32) = (-5, 5);
pub const COORDINATE_OUT: (i32, i32) = (0, 101);
pub const COORDINATE_IN_AUCTION: (i32, i32) = (99, 99);
pub const ESTATE_IN_AUCTION: EstateId = 99;

ord_parameter_types! {
	pub const One: AccountId = ALICE;
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

// pub type AdaptedBasicCurrency =
// currencies::BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;

parameter_types! {
	pub const GetNativeCurrencyId: FungibleTokenId = FungibleTokenId::NativeToken(0);
	pub const MiningCurrencyId: FungibleTokenId = FungibleTokenId::MiningResource(0);
	pub const LandTreasuryPalletId: PalletId = PalletId(*b"bit/land");
	pub const MinimumLandPrice: Balance = 10 * DOLLARS;
}

pub struct MetaverseInfoSource {}

impl MetaverseTrait<AccountId> for MetaverseInfoSource {
	fn check_ownership(who: &AccountId, metaverse_id: &MetaverseId) -> bool {
		match *who {
			ALICE => *metaverse_id == ALICE_METAVERSE_ID,
			BOB => *metaverse_id == BOB_METAVERSE_ID,
			_ => false,
		}
	}

	fn get_metaverse(metaverse_id: u64) -> Option<MetaverseInfo<u128>> {
		None
	}

	fn get_metaverse_token(metaverse_id: u64) -> Option<FungibleTokenId> {
		None
	}

	fn update_metaverse_token(metaverse_id: u64, currency_id: FungibleTokenId) -> Result<(), DispatchError> {
		Ok(())
	}
}

pub struct MockAuctionManager;

impl Auction<AccountId, BlockNumber> for MockAuctionManager {
	type Balance = Balance;

	fn auction_info(id: u64) -> Option<AuctionInfo<u128, Self::Balance, u64>> {
		None
	}

	fn update_auction(id: u64, info: AuctionInfo<u128, Self::Balance, u64>) -> DispatchResult {
		Ok(())
	}

	fn new_auction(
		recipient: u128,
		initial_amount: Self::Balance,
		start: u64,
		end: Option<u64>,
	) -> Result<u64, DispatchError> {
		Ok(1)
	}

	fn create_auction(
		auction_type: AuctionType,
		item_id: ItemId,
		end: Option<u64>,
		recipient: u128,
		initial_amount: Self::Balance,
		start: u64,
		listing_level: ListingLevel,
	) -> Result<u64, DispatchError> {
		Ok(1)
	}

	fn remove_auction(id: u64, item_id: ItemId) {}

	fn auction_bid_handler(
		_now: u64,
		id: u64,
		new_bid: (u128, Self::Balance),
		last_bid: Option<(u128, Self::Balance)>,
	) -> DispatchResult {
		Ok(())
	}

	fn local_auction_bid_handler(
		_now: u64,
		id: u64,
		new_bid: (u128, Self::Balance),
		last_bid: Option<(u128, Self::Balance)>,
		social_currency_id: FungibleTokenId,
	) -> DispatchResult {
		Ok(())
	}
}

impl CheckAuctionItemHandler for MockAuctionManager {
	fn check_item_in_auction(item_id: ItemId) -> bool {
		match item_id {
			ItemId::Estate(ESTATE_IN_AUCTION) => {
				return true;
			}
			ItemId::LandUnit(COORDINATE_IN_AUCTION, METAVERSE_ID) => {
				return true;
			}
			_ => {
				return false;
			}
		}
	}
}

parameter_types! {
	pub const MinBlocksPerRound: u32 = 10;
}

impl Config for Runtime {
	type Event = Event;
	type LandTreasury = LandTreasuryPalletId;
	type MetaverseInfoSource = MetaverseInfoSource;
	type Currency = Balances;
	type MinimumLandPrice = MinimumLandPrice;
	type CouncilOrigin = EnsureSignedBy<One, AccountId>;
	type AuctionHandler = MockAuctionManager;
	type MinBlocksPerRound = MinBlocksPerRound;
	type WeightInfo = ();
}

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Estate: estate:: {Pallet, Call, Storage, Event<T>}
	}
);

pub type EstateModule = Pallet<Runtime>;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

pub struct ExtBuilder;

impl Default for ExtBuilder {
	fn default() -> Self {
		ExtBuilder
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: vec![(ALICE, 100000), (BOB, 100000), (BENEFICIARY_ID, 100000)],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

pub fn last_event() -> Event {
	frame_system::Module::<Runtime>::events()
		.pop()
		.expect("Event expected")
		.event
}
