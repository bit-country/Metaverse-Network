#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::{account, whitelisted_caller};
use frame_support::traits::{Currency, Get, OnFinalize, OnInitialize};
use frame_system::RawOrigin;
use orml_benchmarking::runtime_benchmarks;
use sp_runtime::traits::{AccountIdConversion, Lookup, One, StaticLookup, UniqueSaturatedInto};

use core_primitives::RoundTrait;
use governance::{ReferendumParameters, VoteThreshold};
use primitives::estate::{EstateInfo, OwnerId};
use primitives::staking::RoundInfo;
use primitives::{
	AccountId, Balance, BlockNumber, ClassId, EstateId, FungibleTokenId, GroupCollectionId, MetaverseId, TokenId,
};

use crate::{
	Call, Currencies, Economy, EconomyTreasury, Estate, Event, Governance, Metaverse, MinimumStake, Mining, Runtime,
	System,
};

use super::utils::{create_nft_group, dollar, mint_NFT, set_balance, set_metaverse_treasury_initial_balance};

const MAX_BOUND: (i32, i32) = (-100, 100);
const COORDINATE_IN_1: (i32, i32) = (-10, 10);

const EXCHANGE_RATE: Balance = 10000;
const BENEFICIARY_NFT: (ClassId, TokenId) = (1, 0);
const ESTATE_NFT: (ClassId, TokenId) = (1, 0);

const METAVERSE_ID: MetaverseId = 0;
const ESTATE_ID: EstateId = 0;

const STAKING_AMOUNT: Balance = 1000;
const UNSTAKING_AMOUNT: Balance = 100;

const COLLECTION_ID: GroupCollectionId = 0;
const CLASS_ID: ClassId = 0;

const CURRENCY_ID: FungibleTokenId = FungibleTokenId::NativeToken(0);

const REFERENDUM_PARAMETERS: ReferendumParameters<BlockNumber> = ReferendumParameters {
	voting_threshold: Some(VoteThreshold::RelativeMajority),
	min_proposal_launch_period: 12,
	voting_period: 5,
	enactment_period: 10,
	local_vote_locking_period: 30,
	max_proposals_per_metaverse: 10,
};

fn next_block() {
	Economy::on_finalize(System::block_number());
	System::set_block_number(System::block_number() + 1);
	Economy::on_initialize(System::block_number());
	Governance::on_initialize(System::block_number());
}

pub fn run_to_block(n: u32) {
	while System::block_number() < n {
		next_block();
	}
}

runtime_benchmarks! {
	{ Runtime, governance }
	// update_referendum_parameters
	update_referendum_parameters{
		let caller: AccountId = whitelisted_caller();
		let caller_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(caller.clone());
		set_balance(CURRENCY_ID, &caller, dollar(100));

		create_nft_group();
		set_metaverse_treasury_initial_balance();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1u8]);

	}: _(RawOrigin::Root, METAVERSE_ID, REFERENDUM_PARAMETERS)
}

#[cfg(test)]
mod tests {
	use orml_benchmarking::impl_benchmark_test_suite;

	use crate::benchmarking::utils::tests::new_test_ext;

	use super::*;

	impl_benchmark_test_suite!(new_test_ext(),);
}
