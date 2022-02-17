#![cfg(test)]

use frame_support::traits::Nothing;
use frame_support::{construct_runtime, ord_parameter_types, parameter_types, PalletId};
use frame_system::EnsureSignedBy;
use orml_traits::parameter_type_with_key;
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup, Perbill};

use auction_manager::*;
use primitives::{Amount, FungibleTokenId, ItemId};

use crate as economy;

use super::*;

pub type AccountId = u128;
pub type Balance = u128;
pub type BlockNumber = u64;
pub type ClassId = u32;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const FREEDY: AccountId = 3;

pub const DISTRIBUTOR_COLLECTION_ID: u64 = 0;
pub const DISTRIBUTOR_CLASS_ID: ClassId = 0;
pub const DISTRIBUTOR_NFT_ASSET_ID: AssetId = 0;
pub const NFT_ASSET_ID_NOT_EXIST: AssetId = 99;
pub const USER_BUY_POWER_AMOUNT: PowerAmount = 100;
pub const DISTRIBUTOR_POWER_BALANCE: PowerAmount = 100000;
pub const DISTRIBUTOR_MINING_BALANCE: Balance = 1000 * DOLLARS;
pub const DISTRIBUTOR_MINING_LOW_BALANCE: Balance = 1000;

pub const GENERATOR_COLLECTION_ID: u64 = 1;
pub const GENERATOR_CLASS_ID: ClassId = 1;
pub const GENERATOR_NFT_ASSET_ID: AssetId = 1;
pub const GENERATE_POWER_AMOUNT: PowerAmount = 200;
pub const GENERATOR_POWER_BALANCE: PowerAmount = 200000;

pub const COLLECTION_ID_NOT_EXIST: u64 = 99;

pub const EXCHANGE_RATE: Balance = 66_000_000_000_000_000;
pub const DOLLARS: Balance = 1_000_000_000_000_000_000;

pub const ELEMENT_INDEX_ID: ElementId = 0;
pub const ELEMENT_AMOUNT: u64 = 3;
pub const ALICE_POWER_AMOUNT: PowerAmount = 20000;
pub const ALICE_MINING_BALANCE: Balance = 10 * DOLLARS;
pub const ALICE_MINING_LOW_BALANCE: Balance = 1000;

pub const STAKE_BALANCE: Balance = 1000;
pub const STAKE_BELOW_MINIMUM_BALANCE: Balance = 1;
pub const STAKE_EXCESS_BALANCE: Balance = 10 * DOLLARS;
pub const UNSTAKE_AMOUNT: Balance = 10;
pub const CURRENT_ROUND: RoundIndex = 1;
pub const FREE_BALANCE: Balance = 9010;

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
	pub const EconomyPalletId: PalletId = PalletId(*b"bit/fund");
	pub const MiningTreasuryPalletId: PalletId = PalletId(*b"bit/fund");
	pub const MaxTokenMetadata: u32 = 1024;
	pub const MinimumStake: Balance = 100;
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

impl pallet_mining::Config for Runtime {
	type Event = Event;
	type MiningCurrency = Currencies;
	type BitMiningTreasury = MiningTreasuryPalletId;
	type BitMiningResourceId = MiningCurrencyId;
	type AdminOrigin = EnsureSignedBy<One, AccountId>;
}

ord_parameter_types! {
	pub const One: AccountId = 1;
	pub const Two: AccountId = 2;
}
impl Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type FungibleTokenCurrency = Currencies;
	type NFTHandler = NFTModule;
	type RoundHandler = Mining;
	type EconomyTreasury = EconomyPalletId;
	type MiningCurrencyId = MiningCurrencyId;
	type MinimumStake = MinimumStake;
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: FungibleTokenId| -> Balance {
		Default::default()
	};
}

parameter_types! {
	pub TreasuryModuleAccount: AccountId = EconomyPalletId::get().into_account();
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

pub type AdaptedBasicCurrency = currencies::BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;

parameter_types! {
	pub const NativeCurrencyId: FungibleTokenId = FungibleTokenId::NativeToken(0);
	pub const MiningCurrencyId: FungibleTokenId = FungibleTokenId::MiningResource(0);
}

impl currencies::Config for Runtime {
	type Event = Event;
	type MultiSocialCurrency = OrmlTokens;
	type NativeCurrency = AdaptedBasicCurrency;
	type GetNativeCurrencyId = NativeCurrencyId;
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
		_item_id: ItemId,
		_end: Option<u64>,
		_recipient: u128,
		_initial_amount: Self::Balance,
		_start: u64,
		_listing_level: ListingLevel<AccountId>,
	) -> Result<u64, DispatchError> {
		Ok(1)
	}

	fn remove_auction(_id: u64, _item_id: ItemId) {}

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
		_asset_id: &u64,
		_social_currency_id: FungibleTokenId,
	) -> DispatchResult {
		Ok(())
	}
}

impl CheckAuctionItemHandler for MockAuctionManager {
	fn check_item_in_auction(_item_id: ItemId) -> bool {
		return false;
	}
}

parameter_types! {
	pub CreateClassDeposit: Balance = 2;
	pub CreateAssetDeposit: Balance = 1;
	pub NftPalletId: PalletId = PalletId(*b"bit/bNFT");
	pub MaxBatchTransfer: u32 = 3;
	pub MaxBatchMinting: u32 = 2000;
	pub MaxMetadata: u32 = 10;
	pub PromotionIncentive: Balance = 1;
}

impl pallet_nft::Config for Runtime {
	type Event = Event;
	type DataDepositPerByte = CreateAssetDeposit;
	type Currency = Balances;
	type PalletId = NftPalletId;
	type WeightInfo = ();
	type AuctionHandler = MockAuctionManager;
	type MaxBatchTransfer = MaxBatchTransfer;
	type MaxBatchMinting = MaxBatchMinting;
	type MaxMetadata = MaxMetadata;
	type MultiCurrency = Currencies;
	type MiningResourceId = MiningCurrencyId;
	type PromotionIncentive = PromotionIncentive;
}

parameter_types! {
	pub MaxClassMetadata: u32 = 1024;
}

impl orml_nft::Config for Runtime {
	type ClassId = u64;
	type TokenId = u64;
	type ClassData = pallet_nft::NftClassData<Balance>;
	type TokenData = pallet_nft::NftAssetData<Balance>;
	type MaxClassMetadata = MaxClassMetadata;
	type MaxTokenMetadata = MaxTokenMetadata;
}

pub type EconomyModule = Pallet<Runtime>;

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
		OrmlTokens: orml_tokens::{Pallet, Call, Storage, Config<T>, Event<T>},
		Mining: pallet_mining::{Pallet, Call, Storage, Event<T>},
		Economy: economy::{Pallet, Call ,Storage, Event<T>},
		OrmlNft: orml_nft::{Pallet, Storage, Config<T>},
		NFTModule: pallet_nft::{Pallet, Storage ,Call, Event<T>},
	}
);

pub struct ExtBuilder {
	balances: Vec<(AccountId, FungibleTokenId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self { balances: vec![] }
	}
}

impl ExtBuilder {
	pub fn balances(mut self, mut balances: Vec<(AccountId, FungibleTokenId, Balance)>) -> Self {
		self.balances.append(&mut balances);
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: vec![(ALICE, 10000)],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: vec![(BOB, 20000)],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		orml_tokens::GenesisConfig::<Runtime> {
			balances: self.balances, //vec![(ALICE, MiningCurrencyId, 1000000)],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Mining::on_initialize(System::block_number());
	}
}

pub fn last_event() -> Event {
	frame_system::Pallet::<Runtime>::events()
		.pop()
		.expect("Event expected")
		.event
}
