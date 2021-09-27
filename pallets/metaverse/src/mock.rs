#![cfg(test)]

use super::*;
use crate as metaverse;
use frame_support::pallet_prelude::{GenesisBuild, Hooks, MaybeSerializeDeserialize};
use frame_support::sp_runtime::traits::AtLeast32Bit;
use frame_support::{
	construct_runtime, ord_parameter_types, parameter_types, traits::EnsureOrigin, weights::Weight, PalletId,
};
use frame_system::{EnsureRoot, EnsureSignedBy};
use primitives::{Amount, CurrencyId};
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup, Perbill};

pub type AccountId = u128;
pub type AuctionId = u64;
pub type Balance = u64;
pub type MetaverseId = u64;
pub type BlockNumber = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const METAVERSE_ID: MetaverseId = 0;
pub const COUNTRY_ID_NOT_EXIST: MetaverseId = 1;
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
	type OnSetCode = ();
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
	type MaxReserves = ();
	type ReserveIdentifier = ();
}

parameter_types! {
	pub const MetaverseFundPalletId: PalletId = PalletId(*b"bit/fund");
	pub const MaxTokenMetadata: u32 = 1024;
	pub const MinContribution: Balance = 1;
}

ord_parameter_types! {
	pub const One: AccountId = 1;
	pub const Two: AccountId = 2;
}

impl Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type MetaverseTreasury = MetaverseFundPalletId;
	type MaxMetaverseMetadata = MaxTokenMetadata;
	type MinContribution = MinContribution;
	type MetaverseCouncil = EnsureSignedBy<One, AccountId>;
}

pub type MetaverseModule = Module<Runtime>;

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
		Metaverse: metaverse::{Pallet, Call ,Storage, Event<T>},
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
