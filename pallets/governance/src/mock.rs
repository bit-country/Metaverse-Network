#![cfg(test)]

use super::*;
use crate as governance;
use codec::Encode;
use frame_support::{construct_runtime, ord_parameter_types, parameter_types, Parameter};

use frame_support::dispatch::DispatchError;
use frame_support::pallet_prelude::{EnsureOrigin, Member};
use frame_support::{pallet_prelude::Hooks, weights::Weight};
use frame_system::{EnsureRoot, EnsureSignedBy};
use metaverse_primitive::{MetaverseInfo as MetaversePrimitiveInfo, MetaverseLandTrait, MetaverseTrait};
use primitives::FungibleTokenId;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, Hash, IdentityLookup},
	Perbill,
};

parameter_types! {
	pub const BlockHashCount: u32 = 256;
	pub const MaximumBlockWeight: u32 = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
}

pub type AccountId = u64;
pub type Balance = u64;
pub type CountryId = u64;
pub type BlockNumber = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const ALICE_COUNTRY_ID: CountryId = 1;
pub const BOB_COUNTRY_ID: CountryId = 2;
pub const PROPOSAL_DESCRIPTION: [u8; 2] = [1, 2];
//pub const PROPOSAL_PARAMETER: CountryParameter = CountryParameter::MaxParametersPerProposal(2);
pub const REFERENDUM_PARAMETERS: ReferendumParameters<BlockNumber> = ReferendumParameters {
	voting_threshold: Some(VoteThreshold::RelativeMajority),
	min_proposal_launch_period: 12,
	voting_period: 5,
	enactment_period: 10,
	max_proposals_per_country: 1,
};

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
	pub MaximumSchedulerWeight: Weight = 128;
}
impl pallet_scheduler::Config for Runtime {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type MaxScheduledPerBlock = ();
	type WeightInfo = ();
}

pub struct MetaverseInfo {}

impl MetaverseTrait<AccountId> for MetaverseInfo {
	fn check_ownership(who: &AccountId, country_id: &CountryId) -> bool {
		match *who {
			ALICE => *country_id == ALICE_COUNTRY_ID,
			BOB => *country_id == BOB_COUNTRY_ID,
			_ => false,
		}
	}

	fn get_metaverse(metaverse_id: u64) -> Option<MetaversePrimitiveInfo<AccountId>> {
		None
	}

	fn get_metaverse_token(metaverse_id: u64) -> Option<FungibleTokenId> {
		None
	}

	fn update_metaverse_token(metaverse_id: u64, currency_id: FungibleTokenId) -> Result<(), DispatchError> {
		Ok(())
	}
}

pub struct MetaverseLandInfo {}

impl MetaverseLandTrait<AccountId> for MetaverseLandInfo {
	fn get_user_land_units(who: &u64, metaverse_id: &u64) -> Vec<(i32, i32)> {
		Vec::default()
	}

	fn is_user_own_metaverse_land(who: &u64, metaverse_id: &u64) -> bool {
		match *metaverse_id {
			ALICE_COUNTRY_ID => *who == ALICE,
			BOB_COUNTRY_ID => *who == ALICE || *who == BOB,
			_ => false,
		}
	}
}

parameter_types! {
	pub const DefaultVotingPeriod: BlockNumber = 10;
	pub const DefaultEnactmentPeriod: BlockNumber = 2;
	pub const DefaultProposalLaunchPeriod: BlockNumber = 15;
	pub const DefaultMaxParametersPerProposal: u8 = 3;
	pub const DefaultMaxProposalsPerCountry: u8 = 2;
	pub const OneBlock: BlockNumber = 1;
	pub const MinimumProposalDeposit: Balance = 50;
	pub const DefaultPreimageByteDeposit: Balance = 1;
}

ord_parameter_types! {
	pub const One: AccountId = 1;
	pub const Two: AccountId = 2;
}
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, RuntimeDebug, MaxEncodedLen)]
pub enum ProposalType {
	Any,
	JustTransfer,
}

impl Default for ProposalType {
	fn default() -> Self {
		Self::JustTransfer
	}
}

impl InstanceFilter<Call> for ProposalType {
	fn filter(&self, c: &Call) -> bool {
		match self {
			ProposalType::Any => true,
			ProposalType::JustTransfer => matches!(c, Call::Balances(pallet_balances::Call::transfer(..))),
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		self == &ProposalType::Any || self == o
	}
}

impl Config for Runtime {
	type Event = Event;
	type DefaultVotingPeriod = DefaultVotingPeriod;
	type DefaultEnactmentPeriod = DefaultEnactmentPeriod;
	type DefaultProposalLaunchPeriod = DefaultProposalLaunchPeriod;
	type DefaultMaxParametersPerProposal = DefaultMaxParametersPerProposal;
	type DefaultMaxProposalsPerCountry = DefaultMaxProposalsPerCountry;
	type DefaultPreimageByteDeposit = DefaultPreimageByteDeposit;
	type MinimumProposalDeposit = MinimumProposalDeposit;
	type OneBlock = OneBlock;
	type Currency = Balances;
	type MetaverseInfo = MetaverseInfo;
	type PalletsOrigin = OriginCaller;
	type Proposal = Call;
	type Scheduler = Scheduler;
	type MetaverseLandInfo = MetaverseLandInfo;
	type MetaverseCouncil = EnsureSignedBy<One, AccountId>;
	type ProposalType = ProposalType;
}

pub type GovernanceModule = Pallet<Runtime>;

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
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>},
		Governance: governance::{Pallet, Call ,Storage, Event<T>},
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
			balances: vec![(ALICE, 100000), (BOB, 500)],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(block_number));
		ext
	}
}

pub fn last_event() -> Event {
	frame_system::Pallet::<Runtime>::events()
		.pop()
		.expect("Event expected")
		.event
}

fn next_block() {
	GovernanceModule::on_finalize(System::block_number());
	System::set_block_number(System::block_number() + 1);
	GovernanceModule::on_initialize(System::block_number());
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		next_block();
	}
}

pub fn set_balance_proposal(value: u64) -> Vec<u8> {
	Call::Balances(pallet_balances::Call::set_balance(BOB, value, 100)).encode()
}

pub fn set_balance_proposal_hash(value: u64) -> H256 {
	BlakeTwo256::hash(&set_balance_proposal(value)[..])
}

pub fn add_preimage(hash: H256, does_preimage_updates_jury: bool) {
	let preimage_status = PreimageStatus::Available {
		data: set_balance_proposal(4),
		provider: ALICE,
		deposit: 200,
		since: 1,
		/// None if it's not imminent.
		expiry: Some(150),
	};
	Preimages::<Runtime>::insert(hash, preimage_status);
}
