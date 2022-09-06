#![cfg_attr(not(feature = "std"), no_std)]
use codec::{Decode, Encode};
use frame_support::{
	ord_parameter_types, parameter_types,
	traits::{ConstU128, ConstU32, ConstU64, FindAuthor, Nothing},
	weights::Weight,
	ConsensusEngineId, RuntimeDebug,
};
use pallet_evm::{EnsureAddressNever, EnsureAddressRoot, HashedAddressMapping};
use pallet_ethereum::EthereumBlockHashMapping;
use evm_mapping::EvmAddressMapping;
use orml_traits::parameter_type_with_key;
use primitives::{
	evm::EvmAddress, Amount, BlockNumber, ClassId, FungibleTokenId, Header, MetaverseId, Nonce, TokenId, AccountId
};
use scale_info::TypeInfo;
use sp_core::{H160, H256, U256};
use sp_runtime::traits::Convert;
pub use sp_runtime::AccountId32;
use sp_runtime::{
	traits::{AccountIdConversion, BlakeTwo256, BlockNumberProvider, IdentityLookup, Zero},
};
use frame_support::{PalletId, traits::Everything};
use sp_std::str::FromStr;
use sp_std::prelude::*;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
type Block = frame_system::mocking::MockBlock<TestRuntime>;
type Balance = u128;

impl frame_system::Config for TestRuntime {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type Origin = Origin;
	type Index = Nonce;
	type BlockNumber = BlockNumber;
	type Call = Call;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU32<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = frame_support::weights::constants::RocksDbWeight;
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: FungibleTokenId| -> Balance {
		Default::default()
	};
}

parameter_types! {
	pub const MetaverseTreasuryPalletId: PalletId = PalletId(*b"bit/trsy");
	pub TreasuryModuleAccount: AccountId = MetaverseTreasuryPalletId::get().into_account_truncating();
	pub const CountryFundPalletId: PalletId = PalletId(*b"bit/fund");
}
impl pallet_balances::Config for TestRuntime {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ConstU128<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
}

impl pallet_timestamp::Config for TestRuntime {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<1000>;
	type WeightInfo = ();
}

impl orml_tokens::Config for TestRuntime {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = FungibleTokenId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = orml_tokens::TransferDust<TestRuntime, TreasuryModuleAccount>;
	type MaxLocks = ();
	type ReserveIdentifier = [u8; 8];
	type MaxReserves = ();
	type DustRemovalWhitelist = Nothing;
	type OnNewTokenAccount = ();
	type OnKilledTokenAccount = ();
}

impl pallet_ethereum::Config for TestRuntime {
	type Event = Event;
	type StateRoot = pallet_ethereum::IntermediateStateRoot<Self>;
}


pub const NEER_TOKEN_ID: TokenId = 0;
pub const NEER: FungibleTokenId = FungibleTokenId::NativeToken(NEER_TOKEN_ID);

//pub const NUUM_TOKEN_ID: TokenId = 1;
//pub const BIT_TOKEN_ID: TokenId = 2;
//pub const NUUM: FungibleTokenId = FungibleTokenId::NativeToken(NUUM_TOKEN_ID);
//pub const BIT: FungibleTokenId = FungibleTokenId::MiningResource(BIT_TOKEN_ID);

pub type AdaptedBasicCurrency = orml_currencies::BasicCurrencyAdapter<TestRuntime, Balances, Amount, BlockNumber>;

parameter_types! {
	pub const GetNativeCurrencyId: FungibleTokenId = NEER;
}

impl orml_currencies::Config for TestRuntime {
	type MultiCurrency = Tokens;
	type NativeCurrency = AdaptedBasicCurrency;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type WeightInfo = ();
}
pub struct MockBlockNumberProvider;

impl BlockNumberProvider for MockBlockNumberProvider {
	type BlockNumber = u32;

	fn current_block_number() -> Self::BlockNumber {
		Zero::zero()
	}
}

pub struct GasToWeight;

impl Convert<u64, u64> for GasToWeight {
	fn convert(a: u64) -> u64 {
		a
	}
}

/* 
pub struct AuthorGiven;
impl FindAuthor<AccountId32> for AuthorGiven {
	fn find_author<'a, I>(_digests: I) -> Option<AccountId32>
	where
		I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
	{
		Some(AccountId32::from_str("1234500000000000000000000000000000000000").unwrap())
	}
}
*/

parameter_types! {
	pub NetworkContractSource: H160 = H160::from_low_u64_be(1);
	pub const ChainId: u64 = 2042;
	pub BlockGasLimit: U256 = U256::from(u32::max_value());
}

impl evm_mapping::Config for TestRuntime {
	type Event = Event;
	type Currency = Balances;
	type AddressMapping = EvmAddressMapping<TestRuntime>;
	type ChainId = ChainId;
	type TransferAll = ();
	type WeightInfo = ();
}

impl pallet_evm::Config for TestRuntime {
	type Event = Event;
	type Currency = Balances;

	type BlockGasLimit = BlockGasLimit;
	type ChainId = ChainId;
	type BlockHashMapping = EthereumBlockHashMapping<Self>;
	type Runner = pallet_evm::runner::stack::Runner<Self>;

	type CallOrigin = EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = EnsureAddressNever<AccountId>;
	type AddressMapping = HashedAddressMapping<BlakeTwo256>;

	type FeeCalculator = ();
	type GasWeightMapping = ();
	type OnChargeTransaction = ();
	type FindAuthor = ();
	type PrecompilesType = ();
	type PrecompilesValue = ();
	//type WeightInfo = pallet_evm::weights::SubstrateWeight<Self>;
}

frame_support::construct_runtime!(
	pub enum TestRuntime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		Ethereum: pallet_ethereum,
		EVM: pallet_evm,
		EvmAccounts: evm_mapping,
		Tokens: orml_tokens exclude_parts { Call },
		Balances: pallet_balances,
		Currencies: orml_currencies,
	}
);

pub fn new_test_ext() -> sp_io::TestExternalities {
	sp_io::TestExternalities::new_empty()
}