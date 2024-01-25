#![cfg(test)]

use frame_support::traits::{Contains, EqualPrivilegeOnly, InstanceFilter, Nothing};
use frame_support::{construct_runtime, pallet_prelude::Hooks, parameter_types, PalletId};
use frame_system::Call as SystemCall;
use frame_system::EnsureRoot;
use orml_traits::parameter_type_with_key;
use sp_core::crypto::AccountId32;
use sp_core::{ConstU128, H256};
use sp_runtime::traits::{AccountIdConversion, BlakeTwo256, IdentifyAccount, Verify};
use sp_runtime::{testing::Header, traits::IdentityLookup, MultiSignature, Perbill};

use auction_manager::{CheckAuctionItemHandler, ListingLevel};
use core_primitives::{MetaverseInfo, MetaverseMetadata, MetaverseTrait, NftAssetData, NftClassData};
use primitives::{
	continuum::MapTrait, estate::Estate, Amount, AuctionId, ClassId, EstateId, FungibleTokenId, MapSpotId,
	UndeployedLandBlockId,
};

use crate as auction;

use super::*;

parameter_types! {
	pub const BlockHashCount: u32 = 256;
}

pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;
pub type Balance = u128;
pub type BlockNumber = u64;
pub type MetaverseId = u64;
type Signature = MultiSignature;
type AccountPublic = <Signature as Verify>::Signer;

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

pub const UNDEPLOYED_LAND_BLOCK_ID_EXIST: UndeployedLandBlockId = 4;
pub const UNDEPLOYED_LAND_BLOCK_ID_NOT_EXIST: UndeployedLandBlockId = 5;

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
	type MaxReserves = ();
	type ReserveIdentifier = ();
	type WeightInfo = ();
	type HoldIdentifier = ();
	type FreezeIdentifier = ();
	type MaxHolds = frame_support::traits::ConstU32<0>;
	type MaxFreezes = frame_support::traits::ConstU32<0>;
}

pub struct Continuumm;

impl MapTrait<AccountId32> for Continuumm {
	fn transfer_spot(
		_spot_id: MapSpotId,
		_from: AccountId,
		_to: (AccountId, MetaverseId),
	) -> Result<MapSpotId, DispatchError> {
		Ok((0, 0))
	}
}

pub struct EstateHandler;

impl Estate<AccountId32> for EstateHandler {
	fn transfer_estate(_estate_id: EstateId, _from: &AccountId, _to: &AccountId) -> Result<EstateId, DispatchError> {
		Ok(1)
	}

	fn transfer_landunit(
		_coordinate: (i32, i32),
		_from: &AccountId,
		_to: &(AccountId, MetaverseId),
	) -> Result<(i32, i32), DispatchError> {
		Ok((0, 0))
	}

	fn transfer_undeployed_land_block(
		_who: &AccountId,
		_to: &AccountId,
		_undeployed_land_block_id: UndeployedLandBlockId,
	) -> Result<UndeployedLandBlockId, DispatchError> {
		Ok(2)
	}

	fn check_estate(estate_id: EstateId) -> Result<bool, DispatchError> {
		match estate_id {
			ESTATE_ID_EXIST | ESTATE_ID_EXIST_1 => Ok(true),
			ESTATE_ID_NOT_EXIST => Ok(false),
			_ => Ok(false),
		}
	}

	fn check_landunit(_metaverse_id: MetaverseId, coordinate: (i32, i32)) -> Result<bool, DispatchError> {
		match coordinate {
			LAND_UNIT_EXIST | LAND_UNIT_EXIST_1 => Ok(true),
			LAND_UNIT_NOT_EXIST => Ok(false),
			_ => Ok(false),
		}
	}

	fn check_undeployed_land_block(
		_owner: &AccountId,
		undeployed_land_block_id: UndeployedLandBlockId,
	) -> Result<bool, DispatchError> {
		match undeployed_land_block_id {
			UNDEPLOYED_LAND_BLOCK_ID_EXIST => Ok(true),
			UNDEPLOYED_LAND_BLOCK_ID_NOT_EXIST => Ok(false),
			_ => Ok(false),
		}
	}

	fn get_total_land_units(_estate_id: Option<EstateId>) -> u64 {
		100
	}

	fn get_total_undeploy_land_units() -> u64 {
		100
	}

	fn check_estate_ownership(_owner: AccountId, _estate_id: EstateId) -> Result<bool, DispatchError> {
		Ok(false)
	}

	fn is_estate_leasor(_leasor: AccountId, _estate_id: EstateId) -> Result<bool, DispatchError> {
		Ok(false)
	}

	fn is_estate_leased(_estate_id: EstateId) -> Result<bool, DispatchError> {
		Ok(false)
	}
}

pub struct Handler;

impl AuctionHandler<AccountId, Balance, BlockNumber, AuctionId> for Handler {
	fn on_new_bid(
		_now: BlockNumber,
		_id: AuctionId,
		new_bid: (AccountId, Balance),
		_last_bid: Option<(AccountId, Balance)>,
	) -> OnNewBidResult<BlockNumber> {
		// Test with [1,32].into() bid
		if new_bid.0 == [1; 32].into() || new_bid.0 == [2; 32].into() {
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
	pub TreasuryModuleAccount: AccountId = MetaverseTreasuryPalletId::get().into_account_truncating();
	pub const MetaverseFundPalletId: PalletId = PalletId(*b"bit/fund");
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

parameter_types! {
	// Test auction end within 100 blocks
	pub const AuctionTimeToClose: u64 = 100;
	// Test auction end within 100 blocks
	pub const MinimumAuctionDuration: u64 = 10;
	pub const MaxFinality: u32 = 3;
	pub const MaxBundleItem: u32 = 5;
	pub const NetworkFeeReserve: Balance = 1; // Network fee reserved when item is listed for auction
	pub const NetworkFeeCommission: Perbill = Perbill::from_percent(1); // Network fee collected after an auction is over
	pub const OfferDuration: BlockNumber = 10; // Default 10
	pub const MinimumListingPrice: Balance = 1;
	pub const AntiSnipeDuration: BlockNumber = 5; // Default 5
}

pub struct MetaverseInfoSource {}

impl MetaverseTrait<AccountId> for MetaverseInfoSource {
	fn create_metaverse(_who: &AccountId, _metadata: MetaverseMetadata) -> MetaverseId {
		1u64
	}

	fn check_ownership(who: &AccountId, metaverse_id: &MetaverseId) -> bool {
		if who == &AccountId32::new([1; 32]) {
			*metaverse_id == ALICE_METAVERSE_ID
		} else if who == &AccountId32::new([2; 32]) {
			*metaverse_id == BOB_METAVERSE_ID
		} else {
			false
		}
	}

	fn get_metaverse(_metaverse_id: u64) -> Option<MetaverseInfo<AccountId>> {
		None
	}

	fn get_metaverse_token(_metaverse_id: u64) -> Option<FungibleTokenId> {
		None
	}

	fn update_metaverse_token(_metaverse_id: u64, _currency_id: FungibleTokenId) -> Result<(), DispatchError> {
		Ok(())
	}

	fn get_metaverse_land_class(_metaverse_id: MetaverseId) -> Result<ClassId, DispatchError> {
		Ok(15u32)
	}

	fn get_metaverse_estate_class(_metaverse_id: MetaverseId) -> Result<ClassId, DispatchError> {
		Ok(16u32)
	}

	fn get_metaverse_marketplace_listing_fee(_metaverse_id: MetaverseId) -> Result<Perbill, DispatchError> {
		Ok(Perbill::from_percent(1u32))
	}

	fn get_metaverse_treasury(_metaverse_id: MetaverseId) -> AccountId {
		[102; 32].into()
	}

	fn get_network_treasury() -> AccountId {
		[102; 32].into()
	}

	fn check_if_metaverse_estate(
		_metaverse_id: primitives::MetaverseId,
		class_id: &ClassId,
	) -> Result<bool, DispatchError> {
		if class_id == &15u32 || class_id == &16u32 {
			return Ok(true);
		}
		return Ok(false);
	}

	fn check_if_metaverse_has_any_land(_metaverse_id: primitives::MetaverseId) -> Result<bool, DispatchError> {
		Ok(true)
	}

	fn is_metaverse_owner(who: &AccountId) -> bool {
		who != &[3; 32].into()
	}
}

impl Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AuctionTimeToClose = AuctionTimeToClose;
	type Handler = AuctionModule;
	type Currency = Balances;
	type ContinuumHandler = Continuumm;
	type FungibleTokenCurrency = Tokens;
	type MetaverseInfoSource = MetaverseInfoSource;
	type MinimumAuctionDuration = MinimumAuctionDuration;
	type EstateHandler = EstateHandler;
	type MaxFinality = MaxFinality;
	type NFTHandler = NFTModule;
	type MaxBundleItem = MaxBundleItem;
	type NetworkFeeReserve = NetworkFeeReserve;
	type NetworkFeeCommission = NetworkFeeCommission;
	type WeightInfo = ();
	type OfferDuration = OfferDuration;
	type MinimumListingPrice = MinimumListingPrice;
	type AntiSnipeDuration = AntiSnipeDuration;
	type StorageDepositFee = StorageDepositFee;
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
	pub CreateClassDeposit: Balance = 2;
	pub CreateAssetDeposit: Balance = 1;
	pub NftPalletId: PalletId = PalletId(*b"bit/bNFT");
	pub MaxBatchTransfer: u32 = 3;
	pub MaxBatchMinting: u32 = 2000;
	pub MaxMetadata: u32 = 10;
	pub StorageDepositFee: Balance = 1;
}

impl pallet_nft::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
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
	type StorageDepositFee = StorageDepositFee;
	type OffchainSignature = Signature;
	type OffchainPublic = AccountPublic;
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

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum ProxyType {
	Any,
	JustTransfer,
}
impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}
impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::JustTransfer => matches!(c, RuntimeCall::Balances(pallet_balances::Call::transfer { .. })),
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		self == &ProxyType::Any || self == o
	}
}
pub struct BaseFilter;
impl Contains<RuntimeCall> for BaseFilter {
	fn contains(c: &RuntimeCall) -> bool {
		match *c {
			// Remark is used as a no-op call in the benchmarking
			RuntimeCall::System(SystemCall::remark { .. }) => true,
			RuntimeCall::System(_) => false,
			_ => true,
		}
	}
}

impl pallet_proxy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ConstU128<1>;
	type ProxyDepositFactor = ConstU128<1>;
	type MaxProxies = ConstU32<4>;
	type WeightInfo = ();
	type CallHasher = BlakeTwo256;
	type MaxPending = ConstU32<2>;
	type AnnouncementDepositBase = ConstU128<1>;
	type AnnouncementDepositFactor = ConstU128<1>;
}

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Currencies: currencies::{ Pallet, Storage, Call, Event<T>},
		Tokens: orml_tokens::{Pallet, Call, Storage, Config<T>, Event<T>},
		NFTModule: pallet_nft::{Pallet, Storage ,Call, Event<T>},
		OrmlNft: orml_nft::{Pallet, Storage, Config<T>},
		AuctionModule: auction::{Pallet, Call, Storage, Event<T>},
		Proxy: pallet_proxy
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
			balances: vec![([1; 32].into(), 100000), ([2; 32].into(), 500), ([3; 32].into(), 500)],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		orml_tokens::GenesisConfig::<Runtime> {
			balances: vec![
				([1; 32].into(), FungibleTokenId::MiningResource(0), 10000),
				([2; 32].into(), FungibleTokenId::MiningResource(0), 5000),
			],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(block_number));
		ext
	}
}

pub fn last_event() -> RuntimeEvent {
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

	fn auction_info(_id: u64) -> Option<AuctionInfo<AccountId, Self::Balance, u64>> {
		None
	}

	fn auction_item(_id: AuctionId) -> Option<AuctionItem<AccountId, BlockNumber, Self::Balance>> {
		None
	}

	fn update_auction(_id: u64, _info: AuctionInfo<AccountId, Self::Balance, u64>) -> DispatchResult {
		Ok(())
	}

	fn update_auction_item(_id: AuctionId, _item_id: ItemId<Self::Balance>) -> frame_support::dispatch::DispatchResult {
		Ok(())
	}

	fn new_auction(
		_recipient: AccountId,
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
		_recipient: AccountId,
		_initial_amount: Self::Balance,
		_start: u64,
		_listing_level: ListingLevel<AccountId>,
		_listing_fee: Perbill,
		_currency_id: FungibleTokenId,
	) -> Result<u64, DispatchError> {
		Ok(1)
	}

	fn remove_auction(_id: u64, _item_id: ItemId<Balance>) {}

	fn auction_bid_handler(
		_from: AccountId,
		_id: AuctionId,
		_value: Self::Balance,
	) -> frame_support::dispatch::DispatchResult {
		Ok(())
	}

	fn buy_now_handler(
		_from: AccountId,
		_auction_id: AuctionId,
		_value: Self::Balance,
	) -> frame_support::dispatch::DispatchResult {
		Ok(())
	}

	fn local_auction_bid_handler(
		_now: u64,
		_id: u64,
		_new_bid: (AccountId, Self::Balance),
		_last_bid: Option<(AccountId, Self::Balance)>,
		_social_currency_id: FungibleTokenId,
	) -> DispatchResult {
		Ok(())
	}

	fn collect_royalty_fee(
		_high_bid_price: &Self::Balance,
		_high_bidder: &AccountId,
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
