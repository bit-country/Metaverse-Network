#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::{account, whitelisted_caller};
use frame_support::traits::{Currency, Get, OnFinalize, OnInitialize};
use frame_system::RawOrigin;
use orml_benchmarking::runtime_benchmarks;
use sp_runtime::traits::{AccountIdConversion, Lookup, One, StaticLookup, UniqueSaturatedInto};

use core_primitives::RoundTrait;
use primitives::estate::{EstateInfo, OwnerId};
use primitives::staking::RoundInfo;
use primitives::{AccountId, Balance, ClassId, EstateId, FungibleTokenId, GroupCollectionId, MetaverseId, TokenId};

use crate::{
	Call, Currencies, Economy, EconomyTreasury, Estate, Event, Metaverse, MinimumStake, Mining, Runtime, System,
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

fn next_block() {
	Economy::on_finalize(System::block_number());
	System::set_block_number(System::block_number() + 1);
	Economy::on_initialize(System::block_number());
}

pub fn run_to_block(n: u32) {
	while System::block_number() < n {
		next_block();
	}
}

runtime_benchmarks! {
	{ Runtime, economy }
	// set_bit_power_exchange_rate
	set_bit_power_exchange_rate{
		let caller: AccountId = whitelisted_caller();
		let caller_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(caller.clone());
		set_balance(CURRENCY_ID, &caller, dollar(10));
	}: _(RawOrigin::Root, EXCHANGE_RATE)
	verify {
		let new_rate = Economy::get_bit_power_exchange_rate();
		assert_eq!(new_rate, EXCHANGE_RATE);
	}

	// set_power_balance
	set_power_balance{
		let caller: AccountId = whitelisted_caller();
		let caller_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(caller.clone());
		set_balance(CURRENCY_ID, &caller, dollar(10));
	}: _(RawOrigin::Root, BENEFICIARY_NFT, 123)
	verify {
		let account_id: AccountId = EconomyTreasury::get().into_sub_account_truncating(BENEFICIARY_NFT);

		let new_balance = Economy::get_power_balance(account_id);
		assert_eq!(new_balance, 123);
	}

	// stake with no estate
	stake_a{
		let caller: AccountId = whitelisted_caller();
		let caller_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(caller.clone());
		set_balance(CURRENCY_ID, &caller, dollar(1000));

		let min_stake = MinimumStake::get();
		let stake_amount = min_stake + dollar(100);

	}: stake(RawOrigin::Signed(caller.clone()), stake_amount, None)
	verify {
		let staking_balance = Economy::get_staking_info(caller.clone());
		assert_eq!(staking_balance, stake_amount);
	}

	// stake with estate
	stake_b{
		let caller: AccountId = whitelisted_caller();
		let caller_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(caller.clone());
		set_balance(CURRENCY_ID, &caller, dollar(1000));

		create_nft_group();
		set_metaverse_treasury_initial_balance();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1u8]);
		Estate::mint_estate(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1]);

		let min_stake = MinimumStake::get();
		let stake_amount = min_stake + dollar(100);

	}: stake(RawOrigin::Signed(caller.clone()), stake_amount, Some(ESTATE_ID))
	verify {
		let staking_balance = Economy::get_estate_staking_info(ESTATE_ID);
		assert_eq!(staking_balance, stake_amount);
	}


	// unstake
	unstake_a{
		let caller: AccountId = whitelisted_caller();
		let caller_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(caller.clone());
		set_balance(CURRENCY_ID, &caller, dollar(1000));

		let min_stake = MinimumStake::get();
		let stake_amount = min_stake + dollar(100);

		Economy::stake(RawOrigin::Signed(caller.clone()).into(), stake_amount, None);

		let current_round = Mining::get_current_round_info();
		let next_round = current_round.current.saturating_add(One::one());
	}: unstake(RawOrigin::Signed(caller.clone()), dollar(10), None)
	verify {
		let staking_balance = Economy::get_staking_info(caller.clone());
		assert_eq!(staking_balance, min_stake + dollar(90));

		assert_eq!(
			Economy::staking_exit_queue(caller.clone(), next_round),
			Some(dollar(10))
		);
	}

	// unstake from estate
	unstake_b{
		let caller: AccountId = whitelisted_caller();
		let caller_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(caller.clone());
		set_balance(CURRENCY_ID, &caller, dollar(1000));

		let min_stake = MinimumStake::get();
		let stake_amount = min_stake + dollar(100);

		create_nft_group();
		set_metaverse_treasury_initial_balance();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1u8]);
		Estate::mint_estate(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1]);

		Economy::stake(RawOrigin::Signed(caller.clone()).into(), stake_amount, Some(ESTATE_ID));

		let current_round = Mining::get_current_round_info();
		let next_round = current_round.current.saturating_add(One::one());
	}: unstake(RawOrigin::Signed(caller.clone()), dollar(10), Some(ESTATE_ID))
	verify {
		let staking_balance = Economy::get_estate_staking_info(ESTATE_ID);
		assert_eq!(staking_balance, min_stake + dollar(90));

		assert_eq!(
			Economy::staking_exit_queue(caller.clone(), next_round),
			Some(dollar(10))
		);
	}

	// withdraw_unreserved
	withdraw_unreserved{
		let caller: AccountId = whitelisted_caller();
		let caller_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(caller.clone());
		set_balance(CURRENCY_ID, &caller, dollar(1000));

		let min_stake = MinimumStake::get();
		let stake_amount = min_stake + dollar(1);

		let current_round = Mining::get_current_round_info();
		let next_round = current_round.current.saturating_add(One::one());

		Economy::stake(RawOrigin::Signed(caller.clone()).into(), stake_amount, None);
		Economy::unstake(RawOrigin::Signed(caller.clone()).into(), 10u32.into(), None);

		run_to_block(100);
		let next_round = current_round.current.saturating_add(One::one());
	}: _(RawOrigin::Signed(caller.clone()), next_round)
	verify {
		assert_eq!(
			Economy::staking_exit_queue(caller.clone(), next_round),
			None
		);
	}

}

#[cfg(test)]
mod tests {
	use orml_benchmarking::impl_benchmark_test_suite;

	use crate::benchmarking::utils::tests::new_test_ext;

	use super::*;

	impl_benchmark_test_suite!(new_test_ext(),);
}
