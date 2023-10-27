#![cfg(test)]

use codec::Encode;
use frame_support::traits::{EqualPrivilegeOnly, Nothing};
use frame_support::{construct_runtime, parameter_types};
use frame_system::EnsureRoot;
use orml_traits::parameter_type_with_key;
use sp_core::crypto::AccountId32;
use sp_core::H256;
use sp_runtime::testing::Header;
use sp_runtime::traits::{IdentifyAccount, IdentityLookup, Verify};
use sp_runtime::{MultiSignature, Perbill};

use auction_manager::{Auction, AuctionInfo, AuctionItem, AuctionType, ListingLevel};
pub use primitive_traits::{CollectionType, NftAssetData, NftClassData};
use primitives::{Amount, AuctionId, CurrencyId, FungibleTokenId, ItemId};

use crate as nft;

use super::*;

parameter_types! {
	pub const BlockHashCount: u32 = 256;
}

pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;
pub type Balance = u128;
pub type BlockNumber = u64;
pub type Signature = MultiSignature;
pub type AccountPublic = <Signature as Verify>::Signer;

pub const CLASS_ID: <Runtime as orml_nft::Config>::ClassId = 0;
pub const CLASS_ID_1: <Runtime as orml_nft::Config>::ClassId = 1;
pub const NON_EXISTING_CLASS_ID: <Runtime as orml_nft::Config>::ClassId = 1000;
pub const TOKEN_ID: <Runtime as orml_nft::Config>::TokenId = 0;
pub const TOKEN_ID_1: <Runtime as orml_nft::Config>::TokenId = 1;
pub const TOKEN_ID_2: <Runtime as orml_nft::Config>::TokenId = 2;
pub const COLLECTION_ID: u64 = 0;

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

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: FungibleTokenId| -> Balance {
		Default::default()
	};
}

parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = 0;
}

pub struct MockAuctionManager;

impl Auction<AccountId, BlockNumber> for MockAuctionManager {
	type Balance = Balance;

	fn auction_info(_id: u64) -> Option<AuctionInfo<AccountId32, Self::Balance, u64>> {
		None
	}

	fn auction_item(id: AuctionId) -> Option<AuctionItem<AccountId, BlockNumber, Self::Balance>> {
		None
	}

	fn update_auction(_id: u64, _info: AuctionInfo<AccountId32, Self::Balance, u64>) -> DispatchResult {
		Ok(())
	}

	fn update_auction_item(id: AuctionId, item_id: ItemId<Self::Balance>) -> DispatchResult {
		Ok(())
	}

	fn new_auction(
		_recipient: AccountId32,
		_initial_amount: Self::Balance,
		_start: u64,
		_end: Option<u64>,
	) -> Result<u64, DispatchError> {
		Ok(0)
	}

	fn create_auction(
		_auction_type: AuctionType,
		_item_id: ItemId<Balance>,
		_end: Option<u64>,
		_recipient: AccountId32,
		_initial_amount: Self::Balance,
		_start: u64,
		_listing_level: ListingLevel<AccountId>,
		_listing_fee: Perbill,
		_currency_id: FungibleTokenId,
	) -> Result<u64, DispatchError> {
		Ok(0)
	}

	fn remove_auction(_id: u64, _item_id: ItemId<Balance>) {}

	fn auction_bid_handler(from: AccountId, id: AuctionId, value: Self::Balance) -> DispatchResult {
		Ok(())
	}

	fn buy_now_handler(from: AccountId, auction_id: AuctionId, value: Self::Balance) -> DispatchResult {
		Ok(())
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
		Ok(())
	}

	fn collect_royalty_fee(
		_high_bid_price: &Self::Balance,
		_high_bidder: &AccountId32,
		_asset_id: &(u32, u64),
		_social_currency_id: FungibleTokenId,
	) -> DispatchResult {
		Ok(())
	}
}

impl CheckAuctionItemHandler<Balance> for MockAuctionManager {
	fn check_item_in_auction(_item_id: ItemId<Balance>) -> bool {
		return false;
	}
}

parameter_types! {
	pub MetadataDataDepositPerByte: Balance = 1;
	pub NftPalletId: PalletId = PalletId(*b"bit/bNFT");
	pub MaxBatchTransfer: u32 = 3;
	pub MaxBatchMinting: u32 = 12;
	pub MaxMetadata: u32 = 10;
	pub const MetaverseTreasuryPalletId: PalletId = PalletId(*b"bit/trsy");
	pub TreasuryModuleAccount: AccountId = MetaverseTreasuryPalletId::get().into_account_truncating();
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

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Weight::from_ref_time(128);
}
impl pallet_scheduler::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type MaxScheduledPerBlock = ();
	type WeightInfo = ();
	type Preimages = ();
}

parameter_types! {
	pub AssetMintingFee: Balance = 1;
	pub ClassMintingFee: Balance = 2;
	pub const MetaverseNetworkTreasuryPalletId: PalletId = PalletId(*b"bit/trsy");
	pub StorageDepositFee: Balance = 1;
}

impl Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type Treasury = MetaverseNetworkTreasuryPalletId;
	type PalletId = NftPalletId;
	type AuctionHandler = MockAuctionManager;
	type WeightInfo = ();
	type MaxBatchTransfer = MaxBatchTransfer;
	type MaxBatchMinting = MaxBatchMinting;
	type MaxMetadata = MaxMetadata;
	type MultiCurrency = Currencies;
	type MiningResourceId = MiningCurrencyId;
	type AssetMintingFee = AssetMintingFee;
	type ClassMintingFee = ClassMintingFee;
	type StorageDepositFee = StorageDepositFee;
	type OffchainSignature = Signature;
	type OffchainPublic = AccountPublic;
}

parameter_types! {
	pub MaxClassMetadata: u32 = 1024;
	pub MaxTokenMetadata: u32 = 1024;
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
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>},
		Currencies: currencies::{ Pallet, Storage, Call, Event<T>},
		Tokens: orml_tokens::{ Pallet, Storage, Call, Event<T>},
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
			balances: vec![([0; 32].into(), 100), ([1; 32].into(), 100000), ([2; 32].into(), 1000)],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

pub fn last_event() -> RuntimeEvent {
	frame_system::Pallet::<Runtime>::events()
		.pop()
		.expect("Event expected")
		.event
}

pub fn transfer_balance_encode(to: AccountId, value: u128) -> Vec<u8> {
	RuntimeCall::Balances(pallet_balances::Call::transfer { dest: to, value: value }).encode()
}
