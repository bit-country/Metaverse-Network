#[cfg(test)]
use frame_support::{
    impl_outer_event, impl_outer_origin, impl_outer_dispatch, parameter_types, traits::EnsureOrigin, weights::Weight,
};
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup};
use orml_currencies::BasicCurrencyAdapter;

use super::*;

use crate as nft;

parameter_types! {
    pub const BlockHashCount: u64 = 256;
}

pub type AccountId = u128;
pub type BlockNumber = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CLASS_ID: <Runtime as orml_nft::Config>::ClassId = 0;
pub const CLASS_ID_NOT_EXIST: <Runtime as orml_nft::Config>::ClassId = 1;
pub const TOKEN_ID: <Runtime as orml_nft::Config>::TokenId = 0;
pub const TOKEN_ID_NOT_EXIST: <Runtime as orml_nft::Config>::TokenId = 1;
pub const COLLECTION_ID: u64 = 0;

#[derive(Clone, Eq, PartialEq)]
pub struct Runtime;

impl_outer_origin! {
	pub enum Origin for Runtime {}
}

impl_outer_event! {
	pub enum TestEvent for Runtime {
		frame_system<T>,
		nft<T>,
	}
}

impl_outer_dispatch! {
	pub enum Call for Runtime where origin: Origin {
		frame_system::System,
	}
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
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
    type BaseCallFilter = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Runtime {
    type Balance = Balance;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = frame_system::Pallet<Runtime>;
    type MaxLocks = ();
    type WeightInfo = ();
}

parameter_types! {
    pub const GetNativeCurrencyId: CurrencyId = 0;
}

impl orml_currencies::Config for Runtime {
    type Event = Event;
    type MultiCurrency = Tokens;
    type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type WeightInfo = ();
}

parameter_types! {
    pub CreateClassDeposit: Balance = 2;
    pub CreateAssetDeposit: Balance = 1;
    pub NftModuleId: ModuleId = ModuleId(*b"bit/bNFT");
}

impl nft::Config for Runtime {
    type Event = Event;
    type CreateClassDeposit = CreateClassDeposit;
    type CreateAssetDeposit = CreateAssetDeposit;
    type Randomness = RandomnessCollectiveFlip;
    type Currency = Balances;
    type WeightInfo = weights::module_nft::WeightInfo<Runtime>;
    type ModuleId = NftModuleId;
}

impl orml_nft::Config for Runtime {
    type ClassId = u32;
    type TokenId = u64;
    type ClassData = nft::NftClassData<Balance>;
    type TokenData = nft::NftAssetData<Balance>;
}

pub type NftModule = Module<Runtime>;

use frame_system::Call as SystemCall;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;


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
            balances: vec![(ALICE, 100000)],
        }
            .assimilate_storage(&mut t)
            .unwrap();

        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}

pub fn last_event() -> Event {
    frame_system::Pallet::<Runtime>::events()
        .pop()
        .expect("Event expected")
        .event
}

