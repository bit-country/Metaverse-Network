use crate::{Module, Trait};
use frame_support::{
    impl_outer_event, impl_outer_origin, impl_outer_dispatch, parameter_types, traits::EnsureOrigin, weights::Weight,
};
use frame_system as system;
use frame_system::EnsureSignedBy;
use sp_core::{sr25519, Pair, H256};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
    PalletId,
};
use primitives::{CountryId, CurrencyId, Balance};

pub type AccountId = u128;
pub type BlockNumber = u64;
pub type Amount = i128;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const COUNTRY_ID: CountryId = 0;
pub const COUNTRY_ID_NOT_EXIST: CountryId = 1;
pub const NUUM: CurrencyId = 0;
pub const COUNTRY_FUND: CurrencyId = 1;

#[derive(Clone, Eq, PartialEq)]
pub struct Runtime;

use crate as tokenization;

impl_outer_origin! {
	pub enum Origin for Runtime {}
}

impl_outer_event! {
	pub enum TestEvent for Runtime {
		frame_system<T>,
		tokenization<T>,
		orml_tokens<T>,
		pallet_balances<T>,
		orml_currencies<T>,
		country<T>,
	}
}

impl_outer_dispatch! {
	pub enum Call for Runtime where origin: Origin {
		frame_system::System,
	}
}


// Configure a mock runtime to test the pallet.

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: u32 = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}


impl frame_system::Trait for Runtime {
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Call = Call;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = TestEvent;
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type PalletInfo = ();
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
    type BlockExecutionWeight = ();
    type ExtrinsicBaseWeight = ();
    type MaximumExtrinsicWeight = ();
    type BaseCallFilter = ();
    type SystemWeightInfo = ();
}

pub type System = frame_system::Module<Runtime>;

impl orml_tokens::Trait for Runtime {
    type Event = TestEvent;
    type Balance = Balance;
    type Amount = Amount;
    type CurrencyId = CurrencyId;
    type OnReceived = ();
    type WeightInfo = ();
}

pub type Tokens = orml_tokens::Module<Runtime>;

parameter_types! {
	pub const ExistentialDeposit: Balance = 1;
}

impl pallet_balances::Trait for Runtime {
    type Balance = Balance;
    type DustRemoval = ();
    type Event = TestEvent;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = frame_system::Module<Runtime>;
    type MaxLocks = ();
    type WeightInfo = ();
}

pub type PalletBalances = pallet_balances::Module<Runtime>;
pub type AdaptedBasicCurrency = orml_currencies::BasicCurrencyAdapter<Runtime, PalletBalances, Amount, BlockNumber>;

parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = NUUM;
}

impl orml_currencies::Trait for Runtime {
    type Event = TestEvent;
    type MultiCurrency = Tokens;
    type NativeCurrency = AdaptedBasicCurrency;
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type WeightInfo = ();
}

pub type Currencies = orml_currencies::Module<Runtime>;

parameter_types! {
	pub const CountryFundPalletId: PalletId = PalletId(*b"bit/fund");
}

impl Trait for Runtime {
    type Event = TestEvent;
    type TokenId = u64;
    type CountryCurrency = Currencies;
}

pub type TokenizationModule = Module<Runtime>;

impl country::Trait for Runtime {
    type Event = TestEvent;
    type PalletId = CountryFundPalletId;
}

use frame_system::Call as SystemCall;

pub struct ExtBuilder;

impl Default for ExtBuilder {
    fn default() -> Self {
        ExtBuilder
    }
}

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let t = frame_system::GenesisConfig::default()
            .build_storage::<Runtime>()
            .unwrap();

        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}

pub fn last_event() -> TestEvent {
    frame_system::Module::<Runtime>::events()
        .pop()
        .expect("Event expected")
        .event
}