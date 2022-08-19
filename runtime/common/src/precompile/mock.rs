#![cfg(any(test, feature = "bench"))]

use crate::{AllPrecompiles, Ratio, RuntimeBlockWeights, Weight};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	ord_parameter_types, parameter_types,
	traits::{
		ConstU128, ConstU32, ConstU64, EqualPrivilegeOnly, Everything, InstanceFilter, Nothing, OnFinalize,
		OnInitialize, SortedMembers,
	},
	weights::IdentityFee,
	PalletId, RuntimeDebug,
};
use pallet_evm::{
	AddressMapping, ExitRevert, ExitSucceed, Precompile, PrecompileFailure, PrecompileHandle, PrecompileOutput,
	PrecompileResult, PrecompileSet,
};
use evm_mapping::EvmAddressMapping;
use frame_system::{offchain::SendTransactionTypes, EnsureRoot, EnsureSignedBy};
use scale_info::TypeInfo;
use sp_core::{H160, H256};
use sp_runtime::{
	traits::{AccountIdConversion, BlakeTwo256, BlockNumberProvider, Convert, IdentityLookup, One as OneT, Zero},
	AccountId32, DispatchResult, FixedPointNumber, FixedU128, Perbill, Percent, Permill,
};
use primitives::{Amount, ClassId, CurrencyId, BlockNumber, MetaverseId, evm::EvmAddress};
use sp_std::prelude::*;
use xcm::latest::prelude::*;

pub type AccountId = AccountId32;
type Key = CurrencyId;
pub type Price = FixedU128;
type Balance = u128;

impl frame_system::Config for Runtime {
	type BaseCallFilter = Everything;
    type BlockWeights = RuntimeBlockWeights;
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
	pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
		Default::default()
	};
}

parameter_types! {
	pub const MetaverseTreasuryPalletId: PalletId = PalletId(*b"bit/trsy");
	pub TreasuryModuleAccount: AccountId = MetaverseTreasuryPalletId::get().into_account_truncating();
	pub const CountryFundPalletId: PalletId = PalletId(*b"bit/fund");
}

impl orml_tokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = CurrencyId;
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

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistenceRequirement;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = ReserveIdentifier;
}

pub const NEER: CurrencyId = 0;
pub const NUUM: CurrencyId = 1;

parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = NEER;
}

pub type AdaptedBasicCurrency = orml_currencies::BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;

pub struct MetaverseInfoSource {}

impl MetaverseTrait<AccountId> for MetaverseInfoSource {
	fn create_metaverse(who: &AccountId, metadata: MetaverseMetadata) -> MetaverseId {
		1u64
	}

	fn check_ownership(who: &AccountId, metaverse_id: &MetaverseId) -> bool {
		match *who {
			ALICE => true,
			_ => false,
		}
	}

	fn get_country(metaverse_id: MetaverseId) -> Option<MetaverseInfo<AccountId>> {
		None
	}

	fn get_country_token(metaverse_id: MetaverseId) -> Option<CurrencyId> {
		None
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
		GENERAL_METAVERSE_FUND
	}

	fn get_network_treasury() -> AccountId {
		GENERAL_METAVERSE_FUND
	}

	fn check_if_metaverse_estate(
		metaverse_id: primitives::MetaverseId,
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
}

impl orml_currencies::Config for Runtime {
	type MultiCurrency = Tokens;
	type NativeCurrency = AdaptedBasicCurrency;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type WeightInfo = ();
}

impl pallet_currencies::Config for Test {
    type Event = Event;
	type TokenId = u64;
	type CountryCurrency = Currencies;
	type FungibleTokenTreasury = CountryFundPalletId;
	type MetaverseInfoSource = MetaverseInfoSource;
	type WeightInfo = ();
}

pub struct Members;

impl SortedMembers<AccountId> for Members {
	fn sorted_members() -> Vec<AccountId> {
		OracleMembers::get()
	}
}

impl orml_oracle::Config for Test {
	type Event = Event;
	type OnNewData = ();
	type CombineData = orml_oracle::DefaultCombineData<Self, MinimumCount, ExpiresIn>;
	type Time = Timestamp;
	type OracleKey = Key;
	type OracleValue = Price;
	type RootOperatorAccountId = RootOperatorAccountId;
	type Members = Members;
	type WeightInfo = ();
	type MaxHasDispatchedSize = ConstU32<40>;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ();
	type WeightInfo = ();
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(10) * RuntimeBlockWeights::get().max_block;
}

impl pallet_scheduler::Config for Test {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type MaxScheduledPerBlock = ConstU32<50>;
	type WeightInfo = ();
	type PreimageProvider = ();
	type NoPreimagePostponement = ();
}

impl pallet_evm::Config for Runtime {
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
	type FindAuthor = FindAuthorTruncated<Aura>;
	type PrecompilesType = MetaverseNetworkPrecompiles<Self>;
	type PrecompilesValue = PrecompilesValue;
	// type WeightInfo = pallet_evm::weights::SubstrateWeight<Self>;
}

impl evm_mapping::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type AddressMapping = EvmAddressMapping<Test>;
	type ChainId = ConstU64<2096>;
	type TransferAll = ();
	// type WeightInfo = ();
}

pub const ALICE: AccountId = AccountId::new([1u8; 32]);
pub const BOB: AccountId = AccountId::new([2u8; 32]);
pub const EVA: AccountId = AccountId::new([3u8; 32]);

pub fn alice() -> AccountId {
	<Test as evm_mapping::Config>::AddressMapping::get_account_id(&alice_evm_addr())
}

pub fn alice_evm_addr() -> EvmAddress {
	EvmAddress::from(hex_literal::hex!("1000000000000000000000000000000000000001"))
}

pub fn bob() -> AccountId {
	<Test as evm_mapping::Config>::AddressMapping::get_account_id(&bob_evm_addr())
}

pub fn bob_evm_addr() -> EvmAddress {
	EvmAddress::from(hex_literal::hex!("1000000000000000000000000000000000000002"))
}

pub fn neer_evm_address() -> EvmAddress {
	EvmAddress::try_from(NEER).unwrap()
}

pub fn nuum_evm_address() -> EvmAddress {
	EvmAddress::try_from(NUUM).unwrap()
}

pub fn erc20_address_not_exists() -> EvmAddress {
	EvmAddress::from(hex_literal::hex!("0000000000000000000000000000000200000001"))
}

pub const INITIAL_BALANCE: Balance = 1_000_000_000_000;

pub type SignedExtra = (frame_system::CheckWeight<Test>,);
pub type UncheckedExtrinsic = sp_runtime::generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
pub type Block = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		Oracle: orml_oracle,
		Timestamp: pallet_timestamp,
		Tokens: orml_tokens exclude_parts { Call },
		Balances: pallet_balances,
		Currencies: pallet_currencies,
		EvmMapping: evm_mapping,
		EvmModule: pallet_evm,
        Scheduler: pallet_scheduler,
	}
);

impl<LocalCall> SendTransactionTypes<LocalCall> for Test
where
	Call: From<LocalCall>,
{
	type OverarchingCall = Call;
	type Extrinsic = UncheckedExtrinsic;
}

#[cfg(test)]
// This function basically just builds a genesis storage key/value store
// according to our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
	use frame_support::{assert_ok, traits::GenesisBuild};
	use sp_std::collections::btree_map::BTreeMap;

	let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	let mut accounts = BTreeMap::new();

	accounts.insert(
		alice_evm_addr(),
		pallet_evm::GenesisAccount {
			nonce: 1,
			balance: INITIAL_BALANCE,
			code: vec![],
			storage: std::collections::BTreeMap::new(),
		},
	);
	
	accounts.insert(
		bob_evm_addr(),
		pallet_evm::GenesisAccount {
			nonce: 1,
			balance: INITIAL_BALANCE,
			code: Default::default(),
			storage: Default::default(),
		},
	);

	pallet_balances::GenesisConfig::<Test>::default()
		.assimilate_storage(&mut storage)
		.unwrap();
	pallet_evm::GenesisConfig::<Test> {
		chain_id: 2096,
		accounts,
	}
	.assimilate_storage(&mut storage)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(storage);
	ext.execute_with(|| {
		System::set_block_number(1);
		Timestamp::set_timestamp(1);

		assert_ok!(Currencies::update_balance(
			Origin::root(),
			ALICE,
			NEER,
			1_000_000_000_000
		));
		assert_ok!(Currencies::update_balance(Origin::root(), ALICE, NUUM, 1_000_000_000));

		assert_ok!(Currencies::update_balance(
			Origin::root(),
			EvmAddressMapping::<Test>::get_account_id(&alice_evm_addr()),
			NEER,
			1_000_000_000
		));

		assert_ok!(Currencies::update_balance(
			Origin::root(),
			EvmAddressMapping::<Test>::get_account_id(&alice_evm_addr()),
			NUUM,
			1_000_000_000
		));
	});
	ext
}

pub fn run_to_block(n: u32) {
	while System::block_number() < n {
		Scheduler::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		Scheduler::on_initialize(System::block_number());
	}
}

