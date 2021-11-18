#![cfg(test)]

use super::*;
use crate as auction;
use frame_support::{construct_runtime, pallet_prelude::Hooks, parameter_types, PalletId};
use orml_traits::parameter_type_with_key;
use primitives::{continuum::Continuum, estate::Estate, Amount, AuctionId, CurrencyId, EstateId, FungibleTokenId};
use sp_core::H256;
use sp_runtime::traits::AccountIdConversion;
use sp_runtime::{testing::Header, traits::IdentityLookup};

use auction_manager::{CheckAuctionItemHandler, ListingLevel};
use bc_primitives::{MetaverseInfo, MetaverseTrait};
use frame_support::traits::{Contains, Nothing};

parameter_types! {
	pub const BlockHashCount: u32 = 256;
}

pub type AccountId = u128;
pub type Balance = u128;
pub type BlockNumber = u64;
pub type MetaverseId = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CLASS_ID: u32 = 0;
pub const COLLECTION_ID: u64 = 0;
pub const ALICE_METAVERSE_ID: MetaverseId = 1;
pub const BOB_METAVERSE_ID: MetaverseId = 2;

pub const ESTATE_ID_EXIST: EstateId = 0;
pub const ESTATE_ID_EXIST_1: EstateId = 1;
pub const ESTATE_ID_NOT_EXIST: EstateId = 99;
pub const LAND_UNIT_EXIST: (i32, i32) = (0, 0);
pub const LAND_UNIT_EXIST_1: (i32, i32) = (1, 1);
pub const LAND_UNIT_NOT_EXIST: (i32, i32) = (99, 99);

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

pub struct Continuumm;

impl Continuum<u128> for Continuumm {
	fn transfer_spot(spot_id: u64, from: &AccountId, to: &(AccountId, u64)) -> Result<u64, DispatchError> {
		Ok(1)
	}
}

pub struct EstateHandler;

impl Estate<u128> for EstateHandler {
	fn transfer_estate(estate_id: EstateId, from: &AccountId, to: &AccountId) -> Result<EstateId, DispatchError> {
		Ok(1)
	}

	fn transfer_landunit(
		coordinate: (i32, i32),
		from: &AccountId,
		to: &(AccountId, MetaverseId),
	) -> Result<(i32, i32), DispatchError> {
		Ok((0, 0))
	}

	fn check_estate(estate_id: EstateId) -> Result<bool, DispatchError> {
		match estate_id {
			ESTATE_ID_EXIST | ESTATE_ID_EXIST_1 => Ok(true),
			ESTATE_ID_NOT_EXIST => Ok(false),
			_ => Ok(false),
		}
	}

	fn check_landunit(metaverse_id: MetaverseId, coordinate: (i32, i32)) -> Result<bool, DispatchError> {
		match coordinate {
			LAND_UNIT_EXIST | LAND_UNIT_EXIST_1 => Ok(true),
			LAND_UNIT_NOT_EXIST => Ok(false),
			_ => Ok(false),
		}
	}

	fn get_total_land_units() -> u64 {
		100
	}

	fn get_total_undeploy_land_units() -> u64 {
		100
	}
}

pub struct Handler;

impl AuctionHandler<AccountId, Balance, BlockNumber, AuctionId> for Handler {
	fn on_new_bid(
		now: BlockNumber,
		id: AuctionId,
		new_bid: (AccountId, Balance),
		last_bid: Option<(AccountId, Balance)>,
	) -> OnNewBidResult<BlockNumber> {
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

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: FungibleTokenId| -> Balance {
		Default::default()
	};
}

parameter_types! {
	pub const MetaverseTreasuryPalletId: PalletId = PalletId(*b"bit/trsy");
	pub TreasuryModuleAccount: AccountId = MetaverseTreasuryPalletId::get().into_account();
	pub const MetaverseFundPalletId: PalletId = PalletId(*b"bit/fund");
}

impl orml_tokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = FungibleTokenId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = orml_tokens::TransferDust<Runtime, TreasuryModuleAccount>;
	type MaxLocks = ();
	type DustRemovalWhitelist = Nothing;
}

parameter_types! {
	pub const AuctionTimeToClose: u64 = 100; //Test auction end within 100 blocks
	pub const MinimumAuctionDuration: u64 = 10; //Test auction end within 100 blocks
	pub const LoyaltyFee: u16 = 100; // Test 1% loyalty fee
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

impl Config for Runtime {
	type Event = Event;
	type AuctionTimeToClose = AuctionTimeToClose;
	type Handler = Handler;
	type Currency = Balances;
	type ContinuumHandler = Continuumm;
	type FungibleTokenCurrency = Tokens;
	type MetaverseInfoSource = MetaverseInfoSource;
	type MinimumAuctionDuration = MinimumAuctionDuration;
	type EstateHandler = EstateHandler;
	type LoyaltyFee = LoyaltyFee;
}

parameter_types! {
	pub CreateClassDeposit: Balance = 2;
	pub CreateAssetDeposit: Balance = 1;
	pub NftPalletId: PalletId = PalletId(*b"bit/bNFT");
	pub MaxBatchTransfer: u32 = 3;
	pub MaxBatchMinting: u32 = 10;
	pub MaxMetadata: u32 = 10;
}

impl pallet_nft::Config for Runtime {
	type Event = Event;
	type CreateClassDeposit = CreateClassDeposit;
	type CreateAssetDeposit = CreateAssetDeposit;
	type Currency = Balances;
	type PalletId = NftPalletId;
	type WeightInfo = ();
	type AuctionHandler = MockAuctionManager;
	type MaxBatchTransfer = MaxBatchTransfer;
	type MaxBatchMinting = MaxBatchMinting;
	type MaxMetadata = MaxMetadata;
}

parameter_types! {
	pub MaxClassMetadata: u32 = 1024;
	pub MaxTokenMetadata: u32 = 1024;
}

impl orml_nft::Config for Runtime {
	type ClassId = u32;
	type TokenId = u64;
	type ClassData = pallet_nft::NftClassData<Balance>;
	type TokenData = pallet_nft::NftAssetData<Balance>;
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
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Tokens: orml_tokens::{Pallet, Call, Storage, Config<T>, Event<T>},
		NFTModule: pallet_nft::{Pallet, Storage ,Call, Event<T>},
		OrmlNft: orml_nft::{Pallet, Storage, Config<T>},
		AuctionModule: auction::{Pallet, Call, Storage, Event<T>},
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

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		AuctionModule::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		AuctionModule::on_initialize(System::block_number());
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

	fn collect_loyalty_fee(
		high_bid_price: &Self::Balance,
		high_bidder: &u128,
		asset_id: &u64,
		social_currency_id: FungibleTokenId,
	) -> DispatchResult {
		todo!()
	}
}

impl CheckAuctionItemHandler for MockAuctionManager {
	fn check_item_in_auction(item_id: ItemId) -> bool {
		return false;
	}
}
