#![cfg(test)]

use crate as governance;
use super::*;
use frame_support::{
    construct_runtime, parameter_types, ord_parameter_types, weights::Weight,
    impl_outer_event, impl_outer_origin, impl_outer_dispatch, traits::EnsureOrigin,
};

use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup};
use primitives::{CurrencyId, Amount};
use frame_system::EnsureSignedBy;
use frame_support::pallet_prelude::{MaybeSerializeDeserialize, Hooks};
use frame_support::sp_runtime::traits::AtLeast32Bit;

parameter_types! {
    pub const BlockHashCount: u32 = 256;
    pub const MaximumBlockWeight: u32 = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
}

pub type AccountId = u128;
pub type Balance = u64;
pub type CountryId = u64;
pub type BlockNumber = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CLASS_ID: u32 = 0;
pub const COLLECTION_ID: u64 = 0;
pub const ALICE_COUNTRY_ID: CountryId = 1;
pub const BOB_COUNTRY_ID: CountryId = 2;
pub const PROPOSAL_DESCRIPTION: [u8;2] = [1,2];
pub const PROPOSAL_PARAMETERS: [CountryParameter;2] = [CountryParameter::MaxProposals(2), CountryParameter::MaxParametersPerProposal(2)];
pub const REFERENDUM_PARAMETERS: ReferendumParameters<BlockNumber> = ReferendumParameters {
    voting_threshold: Some(VoteThreshold::RelativeMajority),
    min_proposal_launch_period: 12,
    voting_period:5, 
    enactment_period: 10, 
    max_params_per_proposal: 2,
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

pub struct CountryInfo {}

impl BCCountry<AccountId> for CountryInfo {
    fn check_ownership(who: &AccountId, country_id: &CountryId) -> bool {
        match *who {
            ALICE => *country_id == ALICE_COUNTRY_ID,
            BOB => *country_id == BOB_COUNTRY_ID,
            _ => false,
        }
    }

    fn get_country(country_id: CountryId) -> Option<Country<AccountId>> {
        None
    }

    fn get_country_token(country_id: CountryId) -> Option<CurrencyId> {
        None
    }

    fn is_member(who: &AccountId, country_id: &CountryId) -> bool {
        match *country_id {
            ALICE_COUNTRY_ID => *who == ALICE,
            BOB_COUNTRY_ID =>   *who == ALICE || *who == BOB,
            _ => false,
        }
    }
}

parameter_types! {
    pub const DefaultVotingPeriod: BlockNumber = 10; 
    pub const DefaultEnactmentPeriod: BlockNumber = 5; 
    pub const DefaultProposalLaunchPeriod: BlockNumber = 15; 
    pub const DefaultMaxParametersPerProposal: u8 = 3;
    pub const DefaultMaxProposalsPerCountry: u8 = 2;
    pub const OneBlock: BlockNumber = 1;
}

impl Config for Runtime {
    type Event = Event;
    type DefaultVotingPeriod = DefaultVotingPeriod;
    type DefaultEnactmentPeriod = DefaultEnactmentPeriod;
    type DefaultProposalLaunchPeriod = DefaultProposalLaunchPeriod;
    type DefaultMaxParametersPerProposal =  DefaultMaxParametersPerProposal;
    type DefaultMaxProposalsPerCountry = DefaultMaxProposalsPerCountry;
    type OneBlock = OneBlock;
    type Currency = Balances;
    type CountryInfo = CountryInfo;
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
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
        Governance: governance::{Module, Call ,Storage, Event<T>},
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