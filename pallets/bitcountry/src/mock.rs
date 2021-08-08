#![cfg(test)]

use crate as bitcountry;
use super::*;
use frame_support::{
    construct_runtime, parameter_types, ord_parameter_types, weights::Weight,
    impl_outer_event, impl_outer_origin, impl_outer_dispatch, traits::EnsureOrigin,
};
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup, ModuleId, Perbill};
use primitives::{CurrencyId, Amount, AssetId, ItemId};
use frame_system::{EnsureSignedBy, EnsureRoot};
use frame_support::pallet_prelude::{MaybeSerializeDeserialize, Hooks, GenesisBuild};
use frame_support::sp_runtime::traits::AtLeast32Bit;
use auction_manager::{AuctionHandler, AuctionType, OnNewBidResult, Change, AuctionInfo, Auction, ListingLevel};
use pallet_nft::AssetHandler;

pub type AccountId = u128;
pub type AuctionId = u64;
pub type Balance = u64;
pub type CountryId = u64;
pub type BlockNumber = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const COUNTRY_ID: CountryId = 0;
pub const COUNTRY_ID_NOT_EXIST: CountryId = 1;
pub const NUUM: CurrencyId = 0;

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

parameter_types! {
    pub CreateClassDeposit: Balance = 2;
    pub CreateAssetDeposit: Balance = 1;
    pub NftModuleId: ModuleId = ModuleId(*b"bit/bNFT");   
    pub OwnershipTokenClassId: u32 = 0;  
}

impl pallet_nft::Config for Runtime {
    type Event = Event;   
    type CreateClassDeposit = CreateClassDeposit;    
    type CreateAssetDeposit = CreateClassDeposit;    
    type Currency = Balances;        
    type ModuleId = NftModuleId;    
    type WeightInfo = ();
    type AuctionHandler = MockAuctionManager;
    type AssetsHandler = Handler;
    type OwnershipTokenClass = OwnershipTokenClassId;
    type CountryOwnershipSource = CountryModule;
}

parameter_types! {
	pub const CountryFundModuleId: ModuleId = ModuleId(*b"bit/fund");
}

impl Config for Runtime {
    type Event = Event;
    type ModuleId = CountryFundModuleId;
    type OwnershipTokenManager = Nft;
}

impl orml_nft::Config for Runtime {
    type ClassId = u32;
    type TokenId = u64;
    type ClassData = pallet_nft::NftClassData<Balance>;
    type TokenData = pallet_nft::NftAssetData<Balance>;
}

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
        CountryModule: bitcountry::{Module, Call ,Storage, Event<T>},    
        Nft: pallet_nft::{Module, Call, Event<T>},		
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

    fn create_auction(
        auction_type: AuctionType,
        item_id: ItemId,
        end: Option<BlockNumber>,
        recipient: AccountId,
        initial_amount: Self::Balance,
        start: BlockNumber,
        listing_level: ListingLevel,
    ) -> Result<AuctionId, DispatchError> {
        todo!()
    }

    fn local_auction_bid_handler(
        _now: BlockNumber,
        id: AuctionId,
        new_bid: (AccountId, Self::Balance),
        last_bid: Option<(AccountId, Self::Balance)>,
        social_currency_id: SocialTokenCurrencyId,
    ) -> DispatchResult {
        todo!()
    }

    fn remove_auction(id: u64, item_id: ItemId) {
        todo!()
    }

    fn auction_bid_handler(_now: u64, id: u64, new_bid: (u128, Self::Balance), last_bid: Option<(u128, Self::Balance)>) -> DispatchResult {
        todo!()
    }

    fn check_item_in_auction(asset_id: AssetId) -> bool {
        return false;
    }
}
pub struct Handler;

impl AssetHandler for Handler {
    fn check_item_in_auction(
        asset_id: AssetId,
    ) -> bool {
        return MockAuctionManager::check_item_in_auction(asset_id);
    }
}