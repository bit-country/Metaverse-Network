#![cfg(test)]

use frame_support::{construct_runtime, ord_parameter_types, parameter_types, PalletId};
use frame_system::EnsureSignedBy;
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup, DispatchError, Perbill};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::default::Default;

use auction_manager::{Auction, AuctionInfo, AuctionType, CheckAuctionItemHandler, ListingLevel};
use core_primitives::{CollectionType, NftClassData, TokenType};
use primitives::{
	AssetId, Attributes, ClassId, FungibleTokenId, GroupCollectionId, NftMetadata, TokenId, LAND_CLASS_ID,
};

use crate as estate;

use super::*;

pub type AccountId = u128;
pub type Balance = u128;
pub type MetaverseId = u64;
pub type BlockNumber = u64;
pub type EstateId = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 5;
pub const BENEFICIARY_ID: AccountId = 99;
pub const CLASS_FUND_ID: AccountId = 123;
pub const METAVERSE_ID: MetaverseId = 0;
pub const DOLLARS: Balance = 1_000_000_000_000_000_000;
pub const ALICE_METAVERSE_ID: MetaverseId = 1;
pub const BOB_METAVERSE_ID: MetaverseId = 2;
pub const MAX_BOUND: (i32, i32) = (-100, 100);
pub const LANDBLOCK_COORDINATE: (i32, i32) = (0, 0);
pub const COORDINATE_IN_1: (i32, i32) = (-10, 10);
pub const COORDINATE_IN_2: (i32, i32) = (-5, 5);
pub const COORDINATE_OUT: (i32, i32) = (0, 101);
pub const COORDINATE_IN_AUCTION: (i32, i32) = (99, 99);
pub const ESTATE_IN_AUCTION: EstateId = 99;

pub const BOND_AMOUNT_1: Balance = 1000;
pub const BOND_AMOUNT_2: Balance = 2000;
pub const BOND_AMOUNT_BELOW_MINIMUM: Balance = 100;
pub const BOND_LESS_AMOUNT_1: Balance = 100;

pub const ESTATE_ID: EstateId = 0;

pub const ASSET_ID_1: TokenId = 101;
pub const ASSET_ID_2: TokenId = 100;
pub const ASSET_CLASS_ID: ClassId = 5;
pub const ASSET_TOKEN_ID: TokenId = 6;
pub const ASSET_COLLECTION_ID: GroupCollectionId = 7;

pub const OWNER_ACCOUNT_ID: OwnerId<AccountId, TokenId> = OwnerId::Account(BENEFICIARY_ID);
pub const OWNER_ID_ALICE: OwnerId<AccountId, TokenId> = OwnerId::Account(ALICE);
pub const OWNER_LAND_ASSET_ID: OwnerId<AccountId, TokenId> = OwnerId::Token(ASSET_ID_1);
pub const OWNER_ESTATE_ASSET_ID: OwnerId<AccountId, TokenId> = OwnerId::Token(ASSET_ID_2);

pub const ALICE_METAVERSE_FUND: AccountId = 100;
pub const BOB_METAVERSE_FUND: AccountId = 101;
pub const GENERAL_METAVERSE_FUND: AccountId = 102;

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
	fn create_metaverse(who: &AccountId, metadata: MetaverseMetadata) -> MetaverseId {
		1u64
	}

	fn check_ownership(who: &AccountId, metaverse_id: &MetaverseId) -> bool {
		match *who {
			ALICE => *metaverse_id == ALICE_METAVERSE_ID,
			BOB => *metaverse_id == BOB_METAVERSE_ID,
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

	fn get_metaverse_land_class(metaverse_id: MetaverseId) -> Result<ClassId, DispatchError> {
		Ok(15u32)
	}

	fn get_metaverse_estate_class(metaverse_id: MetaverseId) -> Result<ClassId, DispatchError> {
		Ok(16u32)
	}

	fn get_metaverse_marketplace_listing_fee(metaverse_id: MetaverseId) -> Result<Perbill, DispatchError> {
		Ok(Perbill::from_percent(1u32))
	}

	fn get_metaverse_treasury(metaverse_id: MetaverseId) -> AccountId {
		match metaverse_id {
			ALICE_METAVERSE_ID => return ALICE_METAVERSE_FUND,
			BOB_METAVERSE_ID => return BOB_METAVERSE_FUND,
			_ => GENERAL_METAVERSE_FUND,
		}
	}
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
		_item_id: ItemId<Balance>,
		_end: Option<u64>,
		_recipient: u128,
		_initial_amount: Self::Balance,
		_start: u64,
		_listing_level: ListingLevel<AccountId>,
		_listing_fee: Perbill,
	) -> Result<u64, DispatchError> {
		Ok(1)
	}

	fn remove_auction(_id: u64, _item_id: ItemId<Balance>) {}

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
	) -> DispatchResult {
		Ok(())
	}
}

impl CheckAuctionItemHandler<Balance> for MockAuctionManager {
	fn check_item_in_auction(item_id: ItemId<Balance>) -> bool {
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

fn test_attributes(x: u8) -> Attributes {
	let mut attr: Attributes = BTreeMap::new();
	attr.insert(vec![x, x + 5], vec![x, x + 10]);
	attr
}

pub struct MockNFTHandler;

impl NFTTrait<AccountId, Balance> for MockNFTHandler {
	type TokenId = TokenId;
	type ClassId = ClassId;

	fn check_ownership(who: &AccountId, asset_id: &(Self::ClassId, Self::TokenId)) -> Result<bool, DispatchError> {
		let nft_value = *asset_id;
		if (*who == ALICE && (nft_value.1 == 1 || nft_value.1 == 3))
			|| (*who == BOB && (nft_value.1 == 2 || nft_value.1 == 4))
			|| (*who == BENEFICIARY_ID && (nft_value.1 == 100 || nft_value.1 == 101))
		{
			return Ok(true);
		}
		Ok(false)
	}

	fn check_nft_ownership(who: &AccountId, nft: &(Self::ClassId, Self::TokenId)) -> Result<bool, DispatchError> {
		let nft_value = *nft;
		if *who == ALICE && nft_value.0 == ASSET_CLASS_ID && nft_value.1 == ASSET_TOKEN_ID {
			return Ok(true);
		}
		if *who == BENEFICIARY_ID && nft_value.0 == LAND_CLASS_ID && nft_value.1 == ASSET_ID_1 {
			return Ok(true);
		}
		Ok(false)
	}

	fn check_collection_and_class(
		collection_id: GroupCollectionId,
		class_id: Self::ClassId,
	) -> Result<bool, DispatchError> {
		if class_id == ASSET_CLASS_ID && collection_id == ASSET_COLLECTION_ID {
			return Ok(true);
		}
		Ok(false)
	}
	fn get_nft_group_collection(nft_collection: &Self::ClassId) -> Result<GroupCollectionId, DispatchError> {
		Ok(ASSET_COLLECTION_ID)
	}

	fn create_token_class(
		sender: &AccountId,
		metadata: NftMetadata,
		attributes: Attributes,
		collection_id: GroupCollectionId,
		token_type: TokenType,
		collection_type: CollectionType,
		royalty_fee: Perbill,
	) -> Result<ClassId, DispatchError> {
		match *sender {
			ALICE => {
				if collection_id == 0 {
					Ok(0)
				} else if collection_id == 1 {
					Ok(1)
				} else {
					Ok(2)
				}
			}
			BOB => Ok(3),
			BENEFICIARY_ID => Ok(ASSET_CLASS_ID),
			_ => Ok(100),
		}
	}

	fn mint_token(
		sender: &AccountId,
		class_id: ClassId,
		metadata: NftMetadata,
		attributes: Attributes,
	) -> Result<TokenId, DispatchError> {
		match *sender {
			ALICE => Ok(1),
			BOB => Ok(2),
			BENEFICIARY_ID => {
				if class_id == 15 {
					return Ok(ASSET_ID_1);
				} else if class_id == 16 {
					return Ok(ASSET_ID_2);
				} else {
					return Ok(200);
				}
			}
			_ => {
				if class_id == 0 {
					return Ok(1000);
				} else {
					return Ok(1001);
				}
			}
		}
	}

	fn transfer_nft(from: &AccountId, to: &AccountId, nft: &(Self::ClassId, Self::TokenId)) -> DispatchResult {
		Ok(())
	}

	fn check_item_on_listing(class_id: Self::ClassId, token_id: Self::TokenId) -> Result<bool, DispatchError> {
		Ok(true)
	}

	fn burn_nft(account: &AccountId, nft: &(Self::ClassId, Self::TokenId)) -> DispatchResult {
		Ok(())
	}
	fn is_transferable(nft: &(Self::ClassId, Self::TokenId)) -> Result<bool, DispatchError> {
		Ok(true)
	}

	fn get_class_fund(class_id: &Self::ClassId) -> AccountId {
		CLASS_FUND_ID
	}

	fn get_nft_detail(asset_id: (Self::ClassId, Self::TokenId)) -> Result<NftClassData<Balance>, DispatchError> {
		let new_data = NftClassData {
			deposit: 0,
			attributes: test_attributes(1),
			token_type: TokenType::Transferable,
			collection_type: CollectionType::Collectable,
			is_locked: false,
			royalty_fee: Perbill::from_percent(0u32),
		};
		Ok(new_data)
	}

	fn set_lock_collection(class_id: Self::ClassId, is_locked: bool) -> sp_runtime::DispatchResult {
		todo!()
	}

	fn set_lock_nft(token_id: (Self::ClassId, Self::TokenId), is_locked: bool) -> sp_runtime::DispatchResult {
		todo!()
	}
}

parameter_types! {
	pub const MinBlocksPerRound: u32 = 10;
	pub const MinimumStake: Balance = 200;
	/// Reward payments are delayed by 2 hours (2 * 300 * block_time)
	pub const RewardPaymentDelay: u32 = 2;
	pub const DefaultMaxBound: (i32,i32) = MAX_BOUND;
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
	type MinimumStake = MinimumStake;
	type RewardPaymentDelay = RewardPaymentDelay;
	type NFTTokenizationSource = MockNFTHandler;
	type DefaultMaxBound = DefaultMaxBound;
}

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Estate: estate:: {Pallet, Call, Storage, Event<T>},
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
			balances: vec![(ALICE, 100000), (BOB, 100000), (BENEFICIARY_ID, 1000000)],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

pub fn last_event() -> Event {
	frame_system::Pallet::<Runtime>::events()
		.pop()
		.expect("Event expected")
		.event
}
