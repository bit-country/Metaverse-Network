#![cfg(test)]

use super::*;

use crate as nft;
use frame_support::{
    construct_runtime, impl_outer_event, impl_outer_origin, impl_outer_dispatch, parameter_types, traits::EnsureOrigin, weights::Weight,
};
use orml_traits::parameter_type_with_key;
use sp_core::H256;
use sp_runtime::testing::Header;
use sp_runtime::traits::IdentityLookup;
use orml_currencies::BasicCurrencyAdapter;
use primitives::{CurrencyId, Amount, BlockNumber};

parameter_types! {
    pub const BlockHashCount: u32 = 256;
}

pub type AccountId = u128;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CLASS_ID: <Runtime as orml_nft::Config>::ClassId = 0;
pub const CLASS_ID_NOT_EXIST: <Runtime as orml_nft::Config>::ClassId = 1;
pub const TOKEN_ID: <Runtime as orml_nft::Config>::TokenId = 0;
pub const TOKEN_ID_NOT_EXIST: <Runtime as orml_nft::Config>::TokenId = 1;
pub const COLLECTION_ID: u64 = 0;

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
    type AccountStore = System;
    type MaxLocks = ();
    type WeightInfo = ();
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
		Default::default()
	};
}

parameter_types! {
    pub const GetNativeCurrencyId: CurrencyId = 0;
}

parameter_types! {
    pub CreateClassDeposit: Balance = 2;
    pub CreateAssetDeposit: Balance = 1;
    pub NftModuleId: ModuleId = ModuleId(*b"bit/bNFT");
}

impl Config for Runtime {
    type Event = Event;
    type CreateClassDeposit = CreateClassDeposit;
    type CreateAssetDeposit = CreateAssetDeposit;
    type Currency = Balances;
    type ModuleId = NftModuleId;
    type WeightInfo = ();
}

impl orml_nft::Config for Runtime {
    type ClassId = u32;
    type TokenId = u64;
    type ClassData = nft::NftClassData<Balance>;
    type TokenData = nft::NftAssetData<Balance>;
}


type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;


construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		Nft: nft::{Module, Call, Event<T>},
		OrmlNft: orml_nft::{Module, Storage, Config<T>},
		Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
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
    frame_system::Module::<Runtime>::events()
        .pop()
        .expect("Event expected")
        .event
}

