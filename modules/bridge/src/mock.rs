#![cfg(test)]

use std::collections::BTreeMap;
use std::vec;

use frame_support::traits::{EqualPrivilegeOnly, Nothing};
use frame_support::{construct_runtime, ord_parameter_types, pallet_prelude::Hooks, parameter_types, PalletId};
use frame_system::{EnsureRoot, EnsureSignedBy};
use orml_traits::parameter_type_with_key;
use sp_core::H256;
use sp_runtime::traits::AccountIdConversion;
use sp_runtime::{testing::Header, traits::IdentityLookup, ModuleError, Perbill};

use auction_manager::{Auction, AuctionInfo, AuctionItem, AuctionType, CheckAuctionItemHandler, ListingLevel};
use core_primitives::{
	Attributes, CollectionType, MetaverseInfo, MetaverseMetadata, MetaverseTrait, NFTTrait, NftAssetData, NftClassData,
	NftMetadata, TokenType,
};
use primitives::{
	continuum::MapTrait, estate::Estate, Amount, AuctionId, ClassId, EstateId, FungibleTokenId, GroupCollectionId,
	ItemId, MapSpotId, NftOffer, TokenId, UndeployedLandBlockId,
};

use crate as bridge;

use super::*;

parameter_types! {
	pub const BlockHashCount: u32 = 256;
}

ord_parameter_types! {
	pub const One: AccountId = ALICE;
}

pub type AccountId = u128;
pub type Balance = u128;
pub type BlockNumber = u64;
pub type MetaverseId = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const NO_METAVERSE_OWNER: AccountId = 3;
pub const CLASS_ID: u32 = 0;
pub const COLLECTION_ID: u64 = 0;
pub const ALICE_METAVERSE_ID: MetaverseId = 1;
pub const BOB_METAVERSE_ID: MetaverseId = 2;

pub const CLASS_FUND_ID: AccountId = 123;
pub const BENEFICIARY_ID: AccountId = 99;
pub const ASSET_ID_1: TokenId = 101;
pub const ASSET_ID_2: TokenId = 100;
pub const ASSET_CLASS_ID: ClassId = 5;
pub const ASSET_TOKEN_ID: TokenId = 6;
pub const ASSET_COLLECTION_ID: GroupCollectionId = 7;

pub const ESTATE_ID_EXIST: EstateId = 0;
pub const ESTATE_ID_EXIST_1: EstateId = 1;
pub const ESTATE_ID_NOT_EXIST: EstateId = 99;
pub const LAND_UNIT_EXIST: (i32, i32) = (0, 0);
pub const LAND_UNIT_EXIST_1: (i32, i32) = (1, 1);
pub const LAND_UNIT_NOT_EXIST: (i32, i32) = (99, 99);

pub const GENERAL_METAVERSE_FUND: AccountId = 102;

pub const UNDEPLOYED_LAND_BLOCK_ID_EXIST: UndeployedLandBlockId = 4;
pub const UNDEPLOYED_LAND_BLOCK_ID_NOT_EXIST: UndeployedLandBlockId = 5;

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

pub struct Continuumm;

impl MapTrait<u128> for Continuumm {
	fn transfer_spot(
		_spot_id: MapSpotId,
		_from: AccountId,
		_to: (AccountId, MetaverseId),
	) -> Result<MapSpotId, DispatchError> {
		Ok((0, 0))
	}
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: FungibleTokenId| -> Balance {
		Default::default()
	};
}

parameter_types! {
	pub const MetaverseTreasuryPalletId: PalletId = PalletId(*b"bit/trsy");
	pub TreasuryModuleAccount: AccountId = MetaverseTreasuryPalletId::get().into_account_truncating();
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
	type ReserveIdentifier = [u8; 8];
	type MaxReserves = ();
	type DustRemovalWhitelist = Nothing;
	type OnNewTokenAccount = ();
	type OnKilledTokenAccount = ();
}

parameter_types! {
	pub const BridgeSovereignPalletId: PalletId = PalletId(*b"bit/brgd");
}

impl Config for Runtime {
	type Event = Event;
	type BridgeOrigin = EnsureRoot<AccountId>;
	type Currency = Balances;
	type NFTHandler = MockNFTHandler;
	type NativeCurrencyId = NativeCurrencyId;
	type PalletId = BridgeSovereignPalletId;
}

pub type AdaptedBasicCurrency = currencies::BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;

parameter_types! {
	pub const NativeCurrencyId: FungibleTokenId = FungibleTokenId::NativeToken(0);
	pub const MiningCurrencyId: FungibleTokenId = FungibleTokenId::MiningResource(0);
}

impl currencies::Config for Runtime {
	type Event = Event;
	type MultiSocialCurrency = Tokens;
	type NativeCurrency = AdaptedBasicCurrency;
	type GetNativeCurrencyId = NativeCurrencyId;
	type WeightInfo = ();
}

parameter_types! {
	pub CreateClassDeposit: Balance = 2;
	pub CreateAssetDeposit: Balance = 1;
	pub NftPalletId: PalletId = PalletId(*b"bit/bNFT");
	pub MaxBatchTransfer: u32 = 3;
	pub MaxBatchMinting: u32 = 2000;
	pub MaxMetadata: u32 = 10;
}

parameter_types! {
	pub MaxClassMetadata: u32 = 1024;
	pub MaxTokenMetadata: u32 = 1024;
	pub AssetMintingFee: Balance = 1;
	pub ClassMintingFee: Balance = 2;
	pub const MetaverseNetworkTreasuryPalletId: PalletId = PalletId(*b"bit/trsy");
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
		Currencies: currencies::{ Pallet, Storage, Call, Event<T>},
		Tokens: orml_tokens::{Pallet, Call, Storage, Config<T>, Event<T>},
		BridgeModule: bridge::{Pallet, Call, Storage, Event<T>},
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
			balances: vec![(ALICE, 100000), (BOB, 500), (NO_METAVERSE_OWNER, 500)],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		orml_tokens::GenesisConfig::<Runtime> {
			balances: vec![
				(ALICE, FungibleTokenId::MiningResource(0), 10000),
				(BOB, FungibleTokenId::MiningResource(0), 5000),
			],
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
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
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
			|| (*who == ALICE && (nft_value.1 == 100 || nft_value.1 == 101))
		{
			return Ok(true);
		}
		if (nft_value.1 == 5) {
			return Err(DispatchError::Module(ModuleError {
				index: 5,
				error: [0, 0, 0, 0],
				message: Some("AssetInfoNotFound"),
			}));
		}
		Ok(false)
	}

	fn is_stackable(asset_id: (Self::ClassId, Self::TokenId)) -> Result<bool, DispatchError> {
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
		mint_limit: Option<u32>,
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
			mint_limit: None,
			total_minted_tokens: 0u32,
		};
		Ok(new_data)
	}

	fn set_lock_collection(class_id: Self::ClassId, is_locked: bool) -> sp_runtime::DispatchResult {
		todo!()
	}

	fn set_lock_nft(token_id: (Self::ClassId, Self::TokenId), is_locked: bool) -> sp_runtime::DispatchResult {
		todo!()
	}

	fn get_nft_class_detail(_class_id: Self::ClassId) -> Result<NftClassData<Balance>, DispatchError> {
		let new_data = NftClassData {
			deposit: 0,
			attributes: test_attributes(1),
			token_type: TokenType::Transferable,
			collection_type: CollectionType::Collectable,
			is_locked: false,
			royalty_fee: Perbill::from_percent(0u32),
			mint_limit: None,
			total_minted_tokens: 0u32,
		};
		Ok(new_data)
	}

	fn get_total_issuance(class_id: Self::ClassId) -> Result<Self::TokenId, DispatchError> {
		Ok(10u64)
	}

	fn get_asset_owner(asset_id: &(Self::ClassId, Self::TokenId)) -> Result<AccountId, DispatchError> {
		Ok(ALICE)
	}

	fn mint_token_with_id(
		sender: &AccountId,
		class_id: Self::ClassId,
		token_id: Self::TokenId,
		metadata: NftMetadata,
		attributes: Attributes,
	) -> Result<Self::TokenId, DispatchError> {
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
}
