#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::{account, whitelisted_caller};
use frame_support::traits::{Currency, Get, OnFinalize, OnInitialize};
use frame_system::RawOrigin;
use orml_benchmarking::runtime_benchmarks;
use primitives::{AccountId, Balance, BlockNumber, ClassId, FungibleTokenId, Hash, TokenId};
use sp_core::Encode;
use sp_io::hashing::keccak_256;
use sp_std::vec::Vec;

use super::utils::{create_nft_group, dollar, mint_NFT, set_balance, set_metaverse_treasury_initial_balance};

use crate::{
	Call, Event, MinimumCampaignCoolingOffPeriod, MinimumCampaignDuration, MinimumRewardPool, Reward, Runtime, System,
};

const CURRENCY_ID: FungibleTokenId = FungibleTokenId::NativeToken(0);

fn next_block() {
	Reward::on_finalize(System::block_number());
	System::set_block_number(System::block_number() + 1);
	Reward::on_initialize(System::block_number());
}

pub fn run_to_block(n: u32) {
	while System::block_number() < n {
		next_block();
	}
}

pub fn get_hash(value: u64) -> Hash {
	Hash::from_low_u64_be(value)
}

pub fn get_claim_hash(who: AccountId, balance: Balance) -> Hash {
	let mut leaf: Vec<u8> = who.encode();
	leaf.extend(balance.encode());
	keccak_256(&leaf).into()
}

pub fn get_claim_nft_hash(who: AccountId, token: (ClassId, TokenId)) -> Hash {
	let mut leaf: Vec<u8> = who.encode();
	leaf.extend(token.encode());
	keccak_256(&leaf).into()
}

runtime_benchmarks! {
	{ Runtime, reward }

	// create campaign
	create_campaign {
		System::set_block_number(1u32.into());
		let origin: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &origin, dollar(1000));
		let campaign_end  = System::block_number() + MinimumCampaignDuration::get();
	}: _(RawOrigin::Signed(origin.clone()), origin.clone(), MinimumRewardPool::get(), campaign_end.clone(), MinimumCampaignCoolingOffPeriod::get(), vec![1], CURRENCY_ID)

	// create nft campaign
	create_nft_campaign {
		System::set_block_number(1u32.into());
		let origin: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &origin, dollar(1000));
		let campaign_end  = System::block_number() + MinimumCampaignDuration::get();
		create_nft_group();
		mint_NFT(&origin, 0u32.into());
		mint_NFT(&origin, 0u32.into());
	}: _(RawOrigin::Signed(origin.clone()), origin.clone(), vec![(0u32.into(),1u64.into())], campaign_end.clone(), MinimumCampaignCoolingOffPeriod::get(), vec![1])

	// claim reward
	claim_reward{
		System::set_block_number(1u32.into());
		let origin: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &origin, dollar(1000));
		let claiming_account: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &claiming_account, dollar(10));

		let who: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &who, dollar(1000));
		Reward::add_set_reward_origin(RawOrigin::Root.into(), who.clone());

		let campaign_end  = System::block_number() + MinimumCampaignDuration::get();
		Reward::create_campaign(RawOrigin::Signed(origin.clone()).into(), origin.clone(), MinimumRewardPool::get(), campaign_end.clone(), MinimumCampaignCoolingOffPeriod::get(), vec![1], CURRENCY_ID);
		Reward::set_reward(RawOrigin::Signed(who.clone()).into(), 0u32.into(), claiming_account.clone(), 5u32.into());
		let claiming_block = MinimumCampaignDuration::get() + MinimumCampaignCoolingOffPeriod::get();
		run_to_block(claiming_block);
	}: _(RawOrigin::Signed(claiming_account.clone()), 0u32.into())

	// claim reward root
	claim_reward_root{
		System::set_block_number(1u32.into());
		let origin: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &origin, dollar(1000));
		let claiming_account: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &claiming_account, dollar(10));

		let who: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &who, dollar(1000));
		Reward::add_set_reward_origin(RawOrigin::Root.into(), who.clone());

		let campaign_end  = System::block_number() + MinimumCampaignDuration::get();
		Reward::create_campaign(RawOrigin::Signed(origin.clone()).into(), origin.clone(), MinimumRewardPool::get(), campaign_end.clone(), MinimumCampaignCoolingOffPeriod::get(), vec![1], CURRENCY_ID);
		Reward::set_reward_root(RawOrigin::Signed(who.clone()).into(), 0u32.into(), 5u32.into(), get_claim_hash(claiming_account.clone(), 5u32.into()));
		let claiming_block = MinimumCampaignDuration::get() + MinimumCampaignCoolingOffPeriod::get();
		run_to_block(claiming_block);
	}: _(RawOrigin::Signed(claiming_account.clone()), 0u32.into(), 5u32.into(), vec![])

	// claim  NFT reward
	claim_nft_reward{
		System::set_block_number(1u32.into());
		let origin: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &origin, dollar(1000));
		let claiming_account: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &claiming_account, dollar(10));

		let who: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &who, dollar(1000));
		Reward::add_set_reward_origin(RawOrigin::Root.into(), who.clone());

		let campaign_end  = System::block_number() + MinimumCampaignDuration::get();
		create_nft_group();
		mint_NFT(&origin, 0u32.into());
		mint_NFT(&origin, 0u32.into());
		Reward::create_nft_campaign(RawOrigin::Signed(origin.clone()).into(), origin.clone(), vec![(0u32.into(),1u64.into())], campaign_end.clone(), MinimumCampaignCoolingOffPeriod::get(), vec![1]);
		Reward::set_nft_reward(RawOrigin::Signed(who.clone()).into(), 0u32.into(), claiming_account.clone());
		let claiming_block = MinimumCampaignDuration::get() + MinimumCampaignCoolingOffPeriod::get();
		run_to_block(claiming_block);
	}: _(RawOrigin::Signed(claiming_account.clone()), 0u32.into())

	// claim  NFT reward using merkle root
	claim_nft_reward_root{
		System::set_block_number(1u32.into());
		let origin: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &origin, dollar(1000));
		let claiming_account: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &claiming_account, dollar(10));

		let who: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &who, dollar(1000));
		Reward::add_set_reward_origin(RawOrigin::Root.into(), who.clone());

		let campaign_end  = System::block_number() + MinimumCampaignDuration::get();
		create_nft_group();
		mint_NFT(&origin, 0u32.into());
		mint_NFT(&origin, 0u32.into());
		Reward::create_nft_campaign(RawOrigin::Signed(origin.clone()).into(), origin.clone(), vec![(0u32.into(),1u64.into())], campaign_end.clone(), MinimumCampaignCoolingOffPeriod::get(), vec![1]);
		Reward::set_nft_reward_root(RawOrigin::Signed(who.clone()).into(), 0u32.into(), get_claim_nft_hash(claiming_account.clone(), (0u32, 1u64)));
		let claiming_block = MinimumCampaignDuration::get() + MinimumCampaignCoolingOffPeriod::get();
		run_to_block(claiming_block);
	}: _(RawOrigin::Signed(claiming_account.clone()), 0u32.into(), vec![(0u32, 1u64)], vec![])

	// set reward
	set_reward{
		System::set_block_number(1u32.into());
		let origin: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &origin, dollar(1000));
		let claiming_account: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &claiming_account, dollar(10));

		let who: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &who, dollar(1000));
		Reward::add_set_reward_origin(RawOrigin::Root.into(), who.clone());

		let campaign_end  = System::block_number() + MinimumCampaignDuration::get();
		Reward::create_campaign(RawOrigin::Signed(origin.clone()).into(), origin.clone(), MinimumRewardPool::get(), campaign_end.clone(), MinimumCampaignCoolingOffPeriod::get(), vec![1], CURRENCY_ID);
	}: _(RawOrigin::Signed(who.clone()), 0u32.into(), claiming_account.clone(), 5u32.into())

	// set reward root
	set_reward_root{
		System::set_block_number(1u32.into());
		let origin: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &origin, dollar(1000));
		let claiming_account: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &claiming_account, dollar(10));

		let who: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &who, dollar(1000));
		Reward::add_set_reward_origin(RawOrigin::Root.into(), who.clone());

		let campaign_end  = System::block_number() + MinimumCampaignDuration::get();
		Reward::create_campaign(RawOrigin::Signed(origin.clone()).into(), origin.clone(), MinimumRewardPool::get(), campaign_end.clone(), MinimumCampaignCoolingOffPeriod::get(), vec![1], CURRENCY_ID);
	}: _(RawOrigin::Signed(who.clone()), 0u32.into(), 5u32.into(), get_hash(1u64))


	// set nft reward
	set_nft_reward {
		System::set_block_number(1u32.into());
		let origin: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &origin, dollar(1000));
		let claiming_account: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &claiming_account, dollar(10));

		let who: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &who, dollar(1000));
		Reward::add_set_reward_origin(RawOrigin::Root.into(), who.clone());

		let campaign_end  = System::block_number() + MinimumCampaignDuration::get();
		create_nft_group();
		mint_NFT(&origin, 0u32.into());
		mint_NFT(&origin, 0u32.into());

		Reward::create_nft_campaign(RawOrigin::Signed(origin.clone()).into(), origin.clone(), vec![(0u32.into(),1u64.into())], campaign_end.clone(), MinimumCampaignCoolingOffPeriod::get(), vec![1]);
	}: _(RawOrigin::Signed(who.clone()), 0u32.into(), claiming_account.clone())

	// set nft reward using merkle root
	set_nft_reward_root {
		System::set_block_number(1u32.into());
		let origin: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &origin, dollar(1000));
		let claiming_account: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &claiming_account, dollar(10));

		let who: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &who, dollar(1000));
		Reward::add_set_reward_origin(RawOrigin::Root.into(), who.clone());

		let campaign_end  = System::block_number() + MinimumCampaignDuration::get();
		create_nft_group();
		mint_NFT(&origin, 0u32.into());
		mint_NFT(&origin, 0u32.into());

		Reward::create_nft_campaign(RawOrigin::Signed(origin.clone()).into(), origin.clone(), vec![(0u32.into(),1u64.into())], campaign_end.clone(), MinimumCampaignCoolingOffPeriod::get(), vec![1]);
	}: _(RawOrigin::Signed(who.clone()), 0u32.into(), get_hash(1u64))

	// close_campaign
	close_campaign{
		System::set_block_number(1u32.into());
		let origin: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &origin, dollar(1000));

		let campaign_end  = System::block_number() + MinimumCampaignDuration::get();
		Reward::create_campaign(RawOrigin::Signed(origin.clone()).into(), origin.clone(), MinimumRewardPool::get(), campaign_end.clone(), MinimumCampaignCoolingOffPeriod::get(), vec![1], CURRENCY_ID);
		run_to_block(2 * (campaign_end + MinimumCampaignCoolingOffPeriod::get()));
	}: _(RawOrigin::Signed(origin.clone()), 0u32.into())

	// close nft campaign
	close_nft_campaign {
		System::set_block_number(1u32.into());
		let origin: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &origin, dollar(1000));

		let campaign_end  = System::block_number() + MinimumCampaignDuration::get();
		create_nft_group();
		mint_NFT(&origin, 0u32.into());
		mint_NFT(&origin, 0u32.into());
		Reward::create_nft_campaign(RawOrigin::Signed(origin.clone()).into(), origin.clone(), vec![(0u32.into(),1u64.into())], campaign_end.clone(), MinimumCampaignCoolingOffPeriod::get(), vec![1]);
		run_to_block(2 * (campaign_end + MinimumCampaignCoolingOffPeriod::get()));
	}: _(RawOrigin::Signed(origin.clone()), 0u32.into(), 1u64)

	// cancel_campaign
	cancel_campaign{
		System::set_block_number(1u32.into());
		let origin: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &origin, dollar(1000));

		let campaign_end = System::block_number() + MinimumCampaignDuration::get();
		Reward::create_campaign(RawOrigin::Signed(origin.clone()).into(), origin.clone(), MinimumRewardPool::get(), campaign_end.clone(), MinimumCampaignCoolingOffPeriod::get(), vec![1], CURRENCY_ID);
		run_to_block(MinimumCampaignDuration::get());
	}: _(RawOrigin::Root, 0u32.into())

	// cancel nft campaign
	cancel_nft_campaign {
		System::set_block_number(1u32.into());
		let origin: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &origin, dollar(1000));

		let campaign_end  = System::block_number() + MinimumCampaignDuration::get();
		create_nft_group();
		mint_NFT(&origin, 0u32.into());
		mint_NFT(&origin, 0u32.into());
		Reward::create_nft_campaign(RawOrigin::Signed(origin.clone()).into(), origin.clone(), vec![(0u32.into(),1u64.into())], campaign_end.clone(), MinimumCampaignCoolingOffPeriod::get(), vec![1]);
		run_to_block(MinimumCampaignDuration::get());
	}: _(RawOrigin::Root, 0u32.into(), 1u64)

	// add set reward origin
	add_set_reward_origin {
		let who: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &who, dollar(1000));

	}: _(RawOrigin::Root, who.clone())
	verify {
		assert_eq!(Reward::is_set_reward_origin(&who.clone()), true);
	}

	// remove set reward origin
	remove_set_reward_origin {
		let who: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &who, dollar(1000));

		Reward::add_set_reward_origin(RawOrigin::Root.into(), who.clone());
	}: _(RawOrigin::Root, who.clone())
	verify {
		assert_eq!(Reward::is_set_reward_origin(&who.clone()), false);
	}

	// on finalize
	on_finalize {
		System::set_block_number(1u32.into());
		let origin: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &origin, dollar(1000));
		let claiming_account: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &claiming_account, dollar(10));

		let campaign_end  = System::block_number() + MinimumCampaignDuration::get();
		Reward::create_campaign(RawOrigin::Signed(origin.clone()).into(), origin.clone(), MinimumRewardPool::get(), campaign_end.clone(), MinimumCampaignCoolingOffPeriod::get(), vec![1], CURRENCY_ID);
		Reward::set_reward(RawOrigin::Root.into(), 0u32.into(), claiming_account.clone(), 5u32.into());
	}: {
		Reward::on_finalize(campaign_end);
	}
}

#[cfg(test)]
mod tests {
	use orml_benchmarking::impl_benchmark_test_suite;

	use crate::benchmarking::utils::tests::new_test_ext;

	use super::*;

	impl_benchmark_test_suite!(new_test_ext(),);
}
