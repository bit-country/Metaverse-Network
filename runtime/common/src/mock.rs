use codec::{Decode, Encode};
use frame_support::traits::{Contains, InstanceFilter};
use frame_support::{
	construct_runtime,
	dispatch::DispatchResult,
	pallet_prelude::TypeInfo,
	parameter_types,
	traits::{Everything, Nothing},
	weights::Weight,
	PalletId, RuntimeDebug,
};
use frame_system::Call as SystemCall;
use frame_system::EnsureRoot;
use orml_traits::parameter_type_with_key;
use pallet_evm::{
	EnsureAddressNever, EnsureAddressRoot, HashedAddressMapping, IsPrecompileResult, Precompile, PrecompileSet,
};
use pallet_evm::{PrecompileHandle, PrecompileOutput};
use sp_core::{ConstU128, ConstU32, MaxEncodedLen, H160, H256, U256};
use sp_runtime::traits::{AccountIdConversion, BlakeTwo256, IdentityLookup, Verify};
use sp_runtime::{AccountId32, DispatchError, MultiSignature, Perbill};

use auction_manager::{Auction, AuctionInfo, AuctionItem, AuctionType, CheckAuctionItemHandler, ListingLevel};
use core_primitives::{NftAssetData, NftClassData};
use evm_mapping::AddressMapping as AddressMappingEvm;
use evm_mapping::EvmAddressMapping;
use precompile_utils::precompile_set::*;
use precompile_utils::EvmResult;
use primitives::evm::{
	CurrencyIdType, Erc20Mapping, EvmAddress, H160_POSITION_CURRENCY_ID_TYPE, H160_POSITION_TOKEN,
	H160_POSITION_TOKEN_NFT, H160_POSITION_TOKEN_NFT_CLASS_ID_END,
};
use primitives::{Amount, AuctionId, ClassId, FungibleTokenId, ItemId, TokenId};

use crate::currencies::MultiCurrencyPrecompile;
use crate::nft::NftPrecompile;

use super::*;

pub type AccountId = AccountId32;
//pub type ClassId = u32;
pub type AssetId = u128;
pub type Balance = u128;
pub type BlockNumber = u32;
pub type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
pub type Block = frame_system::mocking::MockBlock<Runtime>;
type Signature = MultiSignature;
type AccountPublic = <Signature as Verify>::Signer;

pub const COLLECTION_ID: u64 = 0;
pub const CLASS_ID: ClassId = 0u32;
pub const CLASS_ID_2: ClassId = 1u32;
pub const TOKEN_ID: TokenId = 0u64;
pub const TOKEN_ID_2: TokenId = 1u64;
pub const ALICE_ACCOUNT: AccountId = AccountId::new([1u8; 32]);
pub const BOB_ACCOUNT: AccountId = AccountId::new([2u8; 32]);
pub const CHARLIE_ACCOUNT: AccountId = AccountId::new([3u8; 32]);

parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Runtime {
	type BaseCallFilter = Everything;
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = sp_runtime::generic::Header<BlockNumber, BlakeTwo256>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type BlockWeights = ();
	type BlockLength = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Config for Runtime {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 1;
}

impl pallet_balances::Config for Runtime {
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type MaxLocks = ();
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type HoldIdentifier = ();
	type FreezeIdentifier = ();
	type MaxHolds = frame_support::traits::ConstU32<1>;
	type MaxFreezes = frame_support::traits::ConstU32<1>;
}

/// The asset precompile address prefix. Addresses that match against this prefix will be routed
/// to MultiCurrencyPrecompile
pub const ASSET_PRECOMPILE_ADDRESS_PREFIX: &[u8] = &[0u8; 9];
/// The NFT precompile address prefix. Addresses that match against this prefix will be routed
/// to NftPrecompile
pub const NFT_PRECOMPILE_ADDRESS_PREFIX: &[u8] = &[2u8; 9];

#[derive(Default)]
pub struct Precompiles<R>(PhantomData<R>);

impl<R> PrecompileSet for Precompiles<R>
where
	MultiCurrencyPrecompile<R>: PrecompileSet,
	NftPrecompile<R>: PrecompileSet,
{
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<EvmResult<PrecompileOutput>> {
		match handle.code_address() {
			a if &a.to_fixed_bytes()[0..9] == ASSET_PRECOMPILE_ADDRESS_PREFIX => {
				MultiCurrencyPrecompile::<R>::default().execute(handle)
			}
			a if &a.to_fixed_bytes()[0..9] == NFT_PRECOMPILE_ADDRESS_PREFIX => {
				NftPrecompile::<R>::default().execute(handle)
			}
			_ => None,
		}
	}

	fn is_precompile(&self, _address: H160, remaining_gas: u64) -> IsPrecompileResult {
		IsPrecompileResult::Answer {
			is_precompile: true,
			extra_cost: 0,
		}
	}
}

/// Current approximation of the gas/s consumption considering
/// EVM execution over compiled WASM (on 4.4Ghz CPU).
/// Given the 500ms Weight, from which 75% only are used for transactions,
/// the total EVM execution gas limit is: GAS_PER_SECOND * 0.500 * 0.75 ~= 15_000_000.
pub const GAS_PER_SECOND: u64 = 40_000_000;

/// Approximate ratio of the amount of Weight per Gas.
/// u64 works for approximations because Weight is a very small unit compared to gas.
pub const WEIGHT_PER_GAS: u64 = WEIGHT_REF_TIME_PER_SECOND.saturating_div(GAS_PER_SECOND);

parameter_types! {
	pub BlockGasLimit: U256 = U256::max_value();
	pub PrecompilesValue: Precompiles<Runtime> = Precompiles(PhantomData);
	pub WeightPerGas: Weight = Weight::from_ref_time(WEIGHT_PER_GAS);
	pub const GasLimitPovSizeRatio: u64 = 4;
}

impl pallet_evm::Config for Runtime {
	type FeeCalculator = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type ChainId = ();
	type OnChargeTransaction = ();
	type FindAuthor = ();
	type CallOrigin = EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = EnsureAddressNever<AccountId>;
	type AddressMapping = HashedAddressMapping<BlakeTwo256>;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = Precompiles<Self>;
	type PrecompilesValue = PrecompilesValue;
	type BlockGasLimit = BlockGasLimit;
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type Timestamp = Timestamp;
	type OnCreate = ();
	type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
	type WeightInfo = ();
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: FungibleTokenId| -> Balance {
		Default::default()
	};
}

parameter_types! {
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
	type CurrencyHooks = CurrencyHooks<Runtime, TreasuryModuleAccount>;
	type MaxLocks = ();
	type ReserveIdentifier = [u8; 8];
	type MaxReserves = ();
	type DustRemovalWhitelist = Nothing;
}

pub type AdaptedBasicCurrency = orml_currencies::BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;

parameter_types! {
	pub const NativeCurrencyId: FungibleTokenId = FungibleTokenId::NativeToken(0);
	pub const MiningCurrencyId: FungibleTokenId = FungibleTokenId::MiningResource(0);
}

impl orml_currencies::Config for Runtime {
	type MultiCurrency = Tokens;
	type NativeCurrency = AdaptedBasicCurrency;
	type GetNativeCurrencyId = NativeCurrencyId;
	type WeightInfo = ();
}

impl currencies_pallet::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MultiSocialCurrency = Tokens;
	type NativeCurrency = AdaptedBasicCurrency;
	type GetNativeCurrencyId = NativeCurrencyId;
	type WeightInfo = ();
}

/// Evm address mapping
impl Erc20Mapping for Runtime {
	fn encode_evm_address(t: FungibleTokenId) -> Option<EvmAddress> {
		EvmAddress::try_from(t).ok()
	}

	fn encode_nft_evm_address(t: (ClassId, TokenId)) -> Option<EvmAddress> {
		let mut address = [2u8; 20];
		let mut asset_bytes: Vec<u8> = t.0.to_be_bytes().to_vec();
		asset_bytes.append(&mut t.1.to_be_bytes().to_vec());

		for byte_index in 0..asset_bytes.len() {
			address[byte_index + H160_POSITION_TOKEN_NFT.start] = asset_bytes.as_slice()[byte_index];
		}

		Some(EvmAddress::from_slice(&address))
	}

	fn decode_evm_address(addr: EvmAddress) -> Option<FungibleTokenId> {
		let address = addr.as_bytes();
		let currency_id = match CurrencyIdType::try_from(address[H160_POSITION_CURRENCY_ID_TYPE]).ok()? {
			CurrencyIdType::NativeToken => address[H160_POSITION_TOKEN]
				.try_into()
				.map(FungibleTokenId::NativeToken)
				.ok(),
			CurrencyIdType::MiningResource => address[H160_POSITION_TOKEN]
				.try_into()
				.map(FungibleTokenId::MiningResource)
				.ok(),
			CurrencyIdType::FungibleToken => address[H160_POSITION_TOKEN]
				.try_into()
				.map(FungibleTokenId::FungibleToken)
				.ok(),
		};

		// Encode again to ensure encoded address is matched
		Self::encode_evm_address(currency_id?).and_then(|encoded| if encoded == addr { currency_id } else { None })
	}

	fn decode_nft_evm_address(addr: EvmAddress) -> Option<(ClassId, TokenId)> {
		let address = addr.as_bytes();

		let mut class_id_bytes = [2u8; 4];
		let mut token_id_bytes = [2u8; 8];
		for byte_index in H160_POSITION_TOKEN_NFT {
			if byte_index < H160_POSITION_TOKEN_NFT_CLASS_ID_END {
				class_id_bytes[byte_index - H160_POSITION_TOKEN_NFT.start] = address[byte_index];
			} else {
				token_id_bytes[byte_index - H160_POSITION_TOKEN_NFT_CLASS_ID_END] = address[byte_index];
			}
		}

		let class_id = u32::from_be_bytes(class_id_bytes);
		let token_id = u64::from_be_bytes(token_id_bytes);

		// Encode again to ensure encoded address is matched
		Self::encode_nft_evm_address((class_id, token_id)).and_then(|encoded| {
			if encoded == addr {
				Some((class_id, token_id))
			} else {
				None
			}
		})
	}
}

impl asset_manager::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type RegisterOrigin = EnsureRoot<AccountId>;
}

// NFT - related
pub struct MockAuctionManager;

impl Auction<AccountId, BlockNumber> for MockAuctionManager {
	type Balance = Balance;

	fn auction_info(_id: u64) -> Option<AuctionInfo<AccountId32, Self::Balance, u32>> {
		None
	}

	fn auction_item(_id: AuctionId) -> Option<AuctionItem<AccountId, BlockNumber, Self::Balance>> {
		None
	}

	fn update_auction(_id: u64, _info: AuctionInfo<AccountId32, Self::Balance, u32>) -> DispatchResult {
		Ok(())
	}

	fn update_auction_item(_id: AuctionId, _item_id: ItemId<Self::Balance>) -> DispatchResult {
		Ok(())
	}

	fn new_auction(
		_recipient: AccountId32,
		_initial_amount: Self::Balance,
		_start: u32,
		_end: Option<u32>,
	) -> Result<u64, DispatchError> {
		Ok(0)
	}

	fn create_auction(
		_auction_type: AuctionType,
		_item_id: ItemId<Balance>,
		_end: Option<u32>,
		_recipient: AccountId32,
		_initial_amount: Self::Balance,
		_start: u32,
		_listing_level: ListingLevel<AccountId>,
		_listing_fee: Perbill,
		_currency_id: FungibleTokenId,
	) -> Result<u64, DispatchError> {
		Ok(0)
	}

	fn remove_auction(_id: u64, _item_id: ItemId<Balance>) {}

	fn auction_bid_handler(_from: AccountId, _id: AuctionId, _value: Self::Balance) -> DispatchResult {
		Ok(())
	}

	fn buy_now_handler(_from: AccountId, _auction_id: AuctionId, _value: Self::Balance) -> DispatchResult {
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
	pub MaxClassMetadata: u32 = 1024;
	pub MaxTokenMetadata: u32 = 1024;
	pub StorageDepositFee: Balance = 1;
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

impl evm_mapping::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type AddressMapping = EvmAddressMapping<Runtime>;
	type ChainId = ();
	type TransferAll = OrmlCurrencies;
	type WeightInfo = ();
	type StorageDepositFee = StorageDepositFee;
	type NetworkTreasuryAccount = TreasuryModuleAccount;
}

parameter_types! {
	pub AssetMintingFee: Balance = 1;
	pub ClassMintingFee: Balance = 1;
	pub MaxBatchTransfer: u32 = 100;
	pub MaxBatchMinting: u32 = 1000;
	pub MaxNftMetadata: u32 = 1024;
	pub NftPalletId: PalletId = PalletId(*b"bit/bNFT");
	pub const MetaverseNetworkTreasuryPalletId: PalletId = PalletId(*b"bit/trsy");
}

impl nft_pallet::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type MultiCurrency = Currencies;
	type Treasury = MetaverseNetworkTreasuryPalletId;
	type WeightInfo = ();
	type PalletId = NftPalletId;
	type AuctionHandler = MockAuctionManager;
	type MaxBatchTransfer = MaxBatchTransfer;
	type MaxBatchMinting = MaxBatchMinting;
	type MaxMetadata = MaxNftMetadata;
	type MiningResourceId = MiningCurrencyId;
	type AssetMintingFee = AssetMintingFee;
	type ClassMintingFee = ClassMintingFee;
	type StorageDepositFee = StorageDepositFee;
	type OffchainSignature = Signature;
	type OffchainPublic = AccountPublic;
}

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

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},

		Evm: pallet_evm::{Pallet, Call, Storage, Event<T>},
		EvmMapping: evm_mapping::{Pallet, Call, Storage, Event<T>},

		Tokens: orml_tokens::{Pallet, Call, Storage, Config<T>, Event<T>},
		OrmlCurrencies: orml_currencies::{Pallet, Call},
		OrmlNft: orml_nft::{Pallet, Storage, Config<T>},

		Currencies: currencies_pallet::{ Pallet, Storage, Call, Event<T>},
		Nft: nft_pallet::{Pallet, Storage, Call, Event<T>},
		AssetManager: asset_manager::{Pallet, Call, Storage, Event<T>},

		Proxy: pallet_proxy,
	}
);

pub struct ExtBuilder {
	// endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder { balances: vec![] }
	}
}

impl ExtBuilder {
	pub(crate) fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.expect("Frame system builds valid default genesis config");

		pallet_balances::GenesisConfig::<Runtime> {
			balances: self.balances,
		}
		.assimilate_storage(&mut t)
		.expect("Pallet balances storage can be assimilated");

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

pub fn alice_evm_addr() -> H160 {
	//H160::from(hex_literal::hex!("1000000000000000000000000000000000000001"))
	EvmAddressMapping::<Runtime>::get_default_evm_address(&ALICE_ACCOUNT)
}

pub fn bob_evm_addr() -> H160 {
	//H160::from(hex_literal::hex!("1000000000000000000000000000000000000002"))
	EvmAddressMapping::<Runtime>::get_default_evm_address(&BOB_ACCOUNT)
}

pub fn neer_evm_address() -> H160 {
	H160::from(hex_literal::hex!("0000000000000000000100000000000000000000"))
}

pub fn nft_precompile_address() -> H160 {
	H160::from(hex_literal::hex!("0202020202020202020000000000000000000000"))
}

pub fn nft_address() -> H160 {
	H160::from(hex_literal::hex!("0202020202020200000000000000000000000002"))
}

pub fn bit_evm_address() -> H160 {
	H160::from(hex_literal::hex!("0000000000000000000300000000000000000000"))
}

pub fn alice_account_id() -> AccountId {
	//<Runtime as pallet_evm::Config>::AddressMapping::into_account_id(alice_evm_addr())
	ALICE_ACCOUNT
}

pub fn bob_account_id() -> AccountId {
	//<Runtime as pallet_evm::Config>::AddressMapping::into_account_id(bob_evm_addr())
	BOB_ACCOUNT
}

pub enum Account {
	Alice,
	Bob,
	Charlie,
	Bogus,
	Precompile,
}

impl Default for Account {
	fn default() -> Self {
		Self::Bogus
	}
}
