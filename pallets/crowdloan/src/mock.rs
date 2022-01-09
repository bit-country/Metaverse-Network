#![cfg(test)]

use frame_support::{construct_runtime, ord_parameter_types, parameter_types, PalletId};
use frame_system::EnsureSignedBy;
use sp_core::H256;
use sp_runtime::traits::{ConvertInto, Identity};
use sp_runtime::{testing::Header, traits::IdentityLookup, DispatchError, Perbill};

use auction_manager::{Auction, AuctionInfo, AuctionType, CheckAuctionItemHandler, ListingLevel};
use primitives::FungibleTokenId;

use crate as crowdloan;

use super::*;

pub type AccountId = u128;
pub type Balance = u128;
pub type MetaverseId = u64;
pub type BlockNumber = u64;
pub type EstateId = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 5;
pub const BENEFICIARY_ID: AccountId = 99;
pub const METAVERSE_ID: MetaverseId = 0;
pub const DOLLARS: Balance = 1_000_000_000_000_000_000;
pub const ALICE_METAVERSE_ID: MetaverseId = 1;
pub const BOB_METAVERSE_ID: MetaverseId = 2;
pub const MAX_BOUND: (i32, i32) = (-100, 100);
pub const COORDINATE_IN_1: (i32, i32) = (-10, 10);
pub const COORDINATE_IN_2: (i32, i32) = (-5, 5);
pub const COORDINATE_OUT: (i32, i32) = (0, 101);
pub const COORDINATE_IN_AUCTION: (i32, i32) = (99, 99);
pub const ESTATE_IN_AUCTION: EstateId = 99;

pub const BOND_AMOUNT_1: Balance = 1000;
pub const BOND_AMOUNT_2: Balance = 2000;
pub const BOND_AMOUNT_BELOW_MINIMUM: Balance = 100;
pub const BOND_LESS_AMOUNT_1: Balance = 100;

ord_parameter_types! {
	pub const One: AccountId = ALICE;
}

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
	type BaseCallFilter = frame_support::traits::Everything;
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
	pub const MinVestedTransfer: Balance = 10;
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

impl pallet_vesting::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type BlockNumberToBalance = ConvertInto;
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = ();
	const MAX_VESTING_SCHEDULES: u32 = 20;
}

parameter_types! {
	pub const MinBlocksPerRound: u32 = 10;
	pub const MinimumStake: Balance = 200;
	/// Reward payments are delayed by 2 hours (2 * 300 * block_time)
	pub const RewardPaymentDelay: u32 = 2;
}

pub struct VestingScheduleTrait;

impl VestingSchedule<AccountId> for VestingScheduleTrait {
	type Moment = ();
	type Currency = Balances;

	fn vesting_balance(who: &AccountId) -> Option<Balance> {
		None
	}

	fn add_vesting_schedule(
		who: &AccountId,
		locked: Balance,
		per_block: Balance,
		starting_block: Self::Moment,
	) -> DispatchResult {
		Ok(())
	}

	fn can_add_vesting_schedule(
		who: &AccountId,
		locked: Balance,
		per_block: Balance,
		starting_block: Self::Moment,
	) -> DispatchResult {
		Ok(())
	}

	fn remove_vesting_schedule(who: &AccountId, schedule_index: u32) -> DispatchResult {
		Ok(())
	}
}

impl Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type VestingSchedule = Vesting;
	type BlockNumberToBalance = ConvertInto;
	type WeightInfo = ();
}

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Vesting: pallet_vesting::{Pallet, Call, Storage, Config<T> ,Event<T>},
		Crowdloan: crowdloan:: {Pallet, Call, Storage, Event<T>},
	}
);

pub type CrowdloanModule = Pallet<Runtime>;

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
			balances: vec![(ALICE, 100000), (BOB, 100000), (BENEFICIARY_ID, 1000000)],
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
