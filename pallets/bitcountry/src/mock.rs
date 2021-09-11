#![cfg(test)]

use crate as bitcountry;
use super::*;
use frame_support::{
    construct_runtime, parameter_types, ord_parameter_types, weights::Weight,
    impl_outer_event, impl_outer_origin, impl_outer_dispatch, traits::EnsureOrigin,
};
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup, ModuleId, Perbill};
use primitives::{CurrencyId, Amount};
use frame_system::{EnsureSignedBy, EnsureRoot};
use frame_support::pallet_prelude::{MaybeSerializeDeserialize, Hooks, GenesisBuild};
use frame_support::sp_runtime::traits::AtLeast32Bit;

pub type AccountId = u128;
pub type AuctionId = u64;
pub type Balance = u64;
pub type BitCountryId = u64;
pub type BlockNumber = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const BITCOUNTRY_ID: BitCountryId = 0;
pub const COUNTRY_ID_NOT_EXIST: BitCountryId = 1;
pub const NUUM: CurrencyId = 0;

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

parameter_types! {
	pub const CountryFundModuleId: ModuleId = ModuleId(*b"bit/fund");
}

impl Config for Runtime {
    type Event = Event;
    type ModuleId = CountryFundModuleId;
}

pub type BitCountryModule = Module<Runtime>;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
        Country: bitcountry::{Module, Call ,Storage, Event<T>},
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