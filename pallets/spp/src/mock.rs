#![cfg(test)]

use frame_support::traits::Nothing;
use frame_support::{construct_runtime, ord_parameter_types, parameter_types, PalletId};
use frame_system::EnsureSignedBy;
use orml_traits::parameter_type_with_key;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{ConvertInto, IdentityLookup},
	DispatchError, Perbill,
};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::default::Default;
use sp_std::vec::Vec;

use asset_manager::ForeignAssetMapping;
use auction_manager::{Auction, AuctionInfo, AuctionItem, AuctionType, CheckAuctionItemHandler, ListingLevel};
use core_primitives::{CollectionType, NftClassData, TokenType};
use primitives::{
	Amount, AssetId, Attributes, AuctionId, ClassId, FungibleTokenId, GroupCollectionId, NftMetadata, PoolId, TokenId,
	LAND_CLASS_ID,
};

use crate as spp;

use super::*;

pub type AccountId = u128;
pub type Balance = u128;
pub type MetaverseId = u64;
pub type BlockNumber = u64;
pub type EstateId = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 5;
pub const CHARLIE: AccountId = 6;
pub const DOM: AccountId = 7;
pub const BENEFICIARY_ID: AccountId = 99;
pub const AUCTION_BENEFICIARY_ID: AccountId = 100;
pub const CLASS_FUND_ID: AccountId = 123;
pub const METAVERSE_ID: MetaverseId = 0;
pub const DOLLARS: Balance = 1_000_000_000_000_000_000;
pub const ALICE_METAVERSE_ID: MetaverseId = 1;
pub const BOB_METAVERSE_ID: MetaverseId = 2;
pub const MAX_BOUND: (i32, i32) = (-100, 100);
pub const LANDBLOCK_COORDINATE: (i32, i32) = (0, 0);
pub const COORDINATE_IN_1: (i32, i32) = (-4, 4);
pub const COORDINATE_IN_2: (i32, i32) = (-4, 5);
pub const COORDINATE_IN_3: (i32, i32) = (-4, 6);
pub const COORDINATE_IN_4: (i32, i32) = (-4, 8);
pub const COORDINATE_OUT: (i32, i32) = (0, 101);
pub const COORDINATE_IN_AUCTION: (i32, i32) = (-4, 7);
pub const ESTATE_IN_AUCTION: EstateId = 3;

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
pub const METAVERSE_LAND_CLASS: ClassId = 15;
pub const METAVERSE_LAND_IN_AUCTION_TOKEN: TokenId = 4;
pub const METAVERSE_ESTATE_CLASS: ClassId = 16;
pub const METAVERSE_ESTATE_IN_AUCTION_TOKEN: TokenId = 3;

pub const GENERAL_METAVERSE_FUND: AccountId = 102;

ord_parameter_types! {
	pub const One: AccountId = ALICE;
	pub const Admin: AccountId = ALICE;
}

// Configure a mock runtime to test the pallet.

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: u32 = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}

impl frame_system::Config for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
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
	type RuntimeEvent = RuntimeEvent;
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
	pub const LandTreasuryPalletId: PalletId = PalletId(*b"bit/land");
	pub const PoolAccountPalletId: PalletId = PalletId(*b"bit/pool");
	pub const MinimumLandPrice: Balance = 10 * DOLLARS;
}

fn test_attributes(x: u8) -> Attributes {
	let mut attr: Attributes = BTreeMap::new();
	attr.insert(vec![x, x + 5], vec![x, x + 10]);
	attr
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: FungibleTokenId| -> Balance {
		Default::default()
	};
}

impl orml_tokens::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = FungibleTokenId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type CurrencyHooks = ();
	type MaxLocks = ();
	type ReserveIdentifier = [u8; 8];
	type MaxReserves = ();
	type DustRemovalWhitelist = Nothing;
}

pub type AdaptedBasicCurrency = currencies::BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;

parameter_types! {
	pub const NativeCurrencyId: FungibleTokenId = FungibleTokenId::NativeToken(0);
	pub const MiningCurrencyId: FungibleTokenId = FungibleTokenId::MiningResource(0);
}

impl currencies::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MultiSocialCurrency = Tokens;
	type NativeCurrency = AdaptedBasicCurrency;
	type GetNativeCurrencyId = NativeCurrencyId;
	type WeightInfo = ();
}

impl asset_manager::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type RegisterOrigin = EnsureSignedBy<One, AccountId>;
}

impl BlockNumberProvider for MockRelayBlockNumberProvider {
	type BlockNumber = BlockNumber;

	fn current_block_number() -> Self::BlockNumber {
		Self::get()
	}
}

impl orml_rewards::Config for Runtime {
	type Share = Balance;
	type Balance = Balance;
	type PoolId = PoolId;
	type CurrencyId = FungibleTokenId;
	type Handler = SppModule;
}

parameter_types! {
	pub const MinBlocksPerRound: u32 = 10;
	pub const MinimumStake: Balance = 200;
	/// Reward payments are delayed by 2 hours (2 * 300 * block_time)
	pub const RewardPaymentDelay: u32 = 2;
	pub const DefaultMaxBound: (i32,i32) = MAX_BOUND;
	pub const NetworkFee: Balance = 1; // Network fee
	pub const MaxOffersPerEstate: u32 = 2;
	pub const MinLeasePricePerBlock: Balance = 1u128;
	pub const MaxLeasePeriod: u32 = 9;
	pub const LeaseOfferExpiryPeriod: u32 = 6;
	pub StorageDepositFee: Balance = 1;
	pub const MaximumQueue: u32 = 50;
	pub static MockRelayBlockNumberProvider: BlockNumber = 0;
}

impl Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type WeightInfo = ();
	type MinimumStake = MinimumStake;
	type RewardPaymentDelay = RewardPaymentDelay;
	type DefaultMaxBound = DefaultMaxBound;
	type NetworkFee = NetworkFee;
	type BlockNumberToBalance = ConvertInto;
	type StorageDepositFee = StorageDepositFee;
	type MultiCurrency = Currencies;
	type PoolAccount = PoolAccountPalletId;
	type MaximumQueue = MaximumQueue;
	type CurrencyIdConversion = ForeignAssetMapping<Runtime>;
	type RelayChainBlockNumber = MockRelayBlockNumberProvider;
	type GovernanceOrigin = EnsureSignedBy<Admin, AccountId>;
}

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		AssetManager: asset_manager::{ Pallet, Storage, Call, Event<T>},
		Currencies: currencies::{ Pallet, Storage, Call, Event<T>},
		Tokens: orml_tokens::{Pallet, Call, Storage, Config<T>, Event<T>},
		Spp: spp:: {Pallet, Call, Storage, Event<T>},
		RewardsModule: orml_rewards
	}
);

pub type SppModule = Pallet<Runtime>;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, FungibleTokenId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			endowed_accounts: vec![],
		}
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: vec![(ALICE, 1000000000), (BOB, 100000), (BENEFICIARY_ID, 1000000)],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		orml_tokens::GenesisConfig::<Runtime> {
			balances: self.endowed_accounts.into_iter().collect::<Vec<_>>(),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}

	pub fn balances(mut self, endowed_accounts: Vec<(AccountId, FungibleTokenId, Balance)>) -> Self {
		self.endowed_accounts = endowed_accounts;
		self
	}

	pub fn ksm_setup_for_alice_and_bob(self) -> Self {
		self.balances(vec![
			(ALICE, FungibleTokenId::NativeToken(1), 20000), //KSM
			(BOB, FungibleTokenId::NativeToken(1), 20000),   //KSM
		])
	}
}

pub fn last_event() -> RuntimeEvent {
	frame_system::Pallet::<Runtime>::events()
		.pop()
		.expect("Event expected")
		.event
}

fn next_block() {
	SppModule::on_finalize(System::block_number());
	System::set_block_number(System::block_number() + 1);
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		next_block();
	}
}
