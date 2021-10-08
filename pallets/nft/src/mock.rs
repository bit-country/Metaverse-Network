#![cfg(test)]

use super::*;

use crate as nft;
use auction_manager::{Auction, AuctionHandler, AuctionInfo, AuctionType, Change, ListingLevel, OnNewBidResult};
use frame_support::{construct_runtime, parameter_types, traits::EnsureOrigin, weights::Weight};
use orml_traits::parameter_type_with_key;
use primitives::{Amount, CurrencyId, FungibleTokenId, ItemId};
use sp_core::H256;
use sp_runtime::testing::Header;
use sp_runtime::traits::IdentityLookup;

parameter_types! {
	pub const BlockHashCount: u32 = 256;
}

pub type AccountId = u128;
pub type Balance = u128;
pub type BlockNumber = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CLASS_ID: <Runtime as orml_nft::Config>::ClassId = 0;
pub const CLASS_ID_NOT_EXIST: <Runtime as orml_nft::Config>::ClassId = 1;
pub const TOKEN_ID: <Runtime as orml_nft::Config>::TokenId = 0;
pub const TOKEN_ID_NOT_EXIST: <Runtime as orml_nft::Config>::TokenId = 1;
pub const COLLECTION_ID: u64 = 0;

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

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
		Default::default()
	};
}

parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = 0;
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

	fn new_auction(
		recipient: u128,
		initial_amount: Self::Balance,
		start: u64,
		end: Option<u64>,
	) -> Result<u64, DispatchError> {
		todo!()
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
		todo!()
	}

	fn remove_auction(id: u64, item_id: ItemId) {
		todo!()
	}

	fn auction_bid_handler(
		_now: u64,
		id: u64,
		new_bid: (u128, Self::Balance),
		last_bid: Option<(u128, Self::Balance)>,
	) -> DispatchResult {
		todo!()
	}

	fn local_auction_bid_handler(
		_: BlockNumber,
		_: u64,
		_: (
			AccountId,
			<Self as auction_manager::Auction<AccountId, BlockNumber>>::Balance,
		),
		_: std::option::Option<(
			AccountId,
			<Self as auction_manager::Auction<AccountId, BlockNumber>>::Balance,
		)>,
		_: FungibleTokenId,
	) -> Result<(), sp_runtime::DispatchError> {
		todo!()
	}
}

impl CheckAuctionItemHandler for MockAuctionManager {
	fn check_item_in_auction(item_id: ItemId) -> bool {
		return false;
	}
}

parameter_types! {
	pub CreateClassDeposit: Balance = 2;
	pub CreateAssetDeposit: Balance = 1;
	pub NftPalletId: PalletId = PalletId(*b"bit/bNFT");
}

impl Config for Runtime {
	type Event = Event;
	type CreateClassDeposit = CreateClassDeposit;
	type CreateAssetDeposit = CreateAssetDeposit;
	type Currency = Balances;
	type PalletId = NftPalletId;
	type AuctionHandler = MockAuctionManager;
	type WeightInfo = ();
}

parameter_types! {
	pub MaxClassMetadata: u32 = 1024;
	pub MaxTokenMetadata: u32 = 1024;
}

impl orml_nft::Config for Runtime {
	type ClassId = u32;
	type TokenId = u64;
	type ClassData = nft::NftClassData<Balance>;
	type TokenData = nft::NftAssetData<Balance>;
	type MaxClassMetadata = MaxClassMetadata;
	type MaxTokenMetadata = MaxTokenMetadata;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Nft: nft::{Pallet, Call, Event<T>},
		OrmlNft: orml_nft::{Pallet, Storage, Config<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
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
		let mut t = frame_system::GenesisConfig::default()
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

pub fn last_event() -> Event {
	frame_system::Module::<Runtime>::events()
		.pop()
		.expect("Event expected")
		.event
}
