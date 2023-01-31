#![cfg(test)]

use frame_support::traits::{EqualPrivilegeOnly, Nothing};
use frame_support::{construct_runtime, pallet_prelude::Hooks, parameter_types, PalletId};
use frame_system::{EnsureRoot, EnsureSignedBy};
use orml_traits::parameter_type_with_key;
use sp_core::H256;
use sp_runtime::traits::{AccountIdConversion, One};
use sp_runtime::{testing::Header, traits::IdentityLookup, Perbill};

use core_primitives::{MetaverseInfo, MetaverseMetadata, MetaverseTrait, NftAssetData, NftClassData};
use primitives::{
	continuum::MapTrait, estate::Estate, Amount, AuctionId, ClassId, EstateId, FungibleTokenId, MapSpotId, NftOffer,
	UndeployedLandBlockId,
};

use crate as bridge;

use super::*;

parameter_types! {
	pub const BlockHashCount: u32 = 256;
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
	type BridgeOrigin = EnsureSignedBy<One, AccountId>;
	type Currency = Tokens;
	type NFTHandler = NFTModule;
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

impl pallet_nft::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type Treasury = MetaverseNetworkTreasuryPalletId;
	type PalletId = NftPalletId;
	type AuctionHandler = AuctionModule;
	type WeightInfo = ();
	type MaxBatchTransfer = MaxBatchTransfer;
	type MaxBatchMinting = MaxBatchMinting;
	type MaxMetadata = MaxMetadata;
	type MultiCurrency = Currencies;
	type MiningResourceId = MiningCurrencyId;
	type AssetMintingFee = AssetMintingFee;
	type ClassMintingFee = ClassMintingFee;
}

parameter_types! {
	pub MaxClassMetadata: u32 = 1024;
	pub MaxTokenMetadata: u32 = 1024;
	pub AssetMintingFee: Balance = 1;
	pub ClassMintingFee: Balance = 2;
	pub const MetaverseNetworkTreasuryPalletId: PalletId = PalletId(*b"bit/trsy");
}

impl orml_nft::Config for Runtime {
	type ClassId = u32;
	type TokenId = u64;
	type Currency = Balances;
	type ClassData = NftClassData<Balance>;
	type TokenData = NftAssetData<Balance>;
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
		Currencies: currencies::{ Pallet, Storage, Call, Event<T>},
		Tokens: orml_tokens::{Pallet, Call, Storage, Config<T>, Event<T>},
		NFTModule: pallet_nft::{Pallet, Storage ,Call, Event<T>},
		OrmlNft: orml_nft::{Pallet, Storage, Config<T>},
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
