#![cfg(feature = "runtime-benchmarks")]
use super::utils::{create_nft_group, dollar, mint_NFT, set_balance, test_attributes};
use crate::{Auction, Balances, Call, Currencies, Event, Metaverse, MinimumAuctionDuration, Nft, Runtime, System};
use auction::Config;
use auction_manager::{CheckAuctionItemHandler, ListingLevel};
use core_primitives::{Attributes, CollectionType, MetaverseInfo, MetaverseTrait, NftMetadata, TokenType};
use frame_benchmarking::{account, whitelisted_caller};
use frame_support::traits::{Currency, Get, OnFinalize, OnInitialize};
use frame_system::RawOrigin;
use orml_benchmarking::runtime_benchmarks;
use primitives::{
	AccountId, FungibleTokenId, ItemId, UndeployedLandBlock, UndeployedLandBlockId, UndeployedLandBlockType,
};
use sp_runtime::traits::{AccountIdConversion, StaticLookup, UniqueSaturatedInto};
use sp_runtime::Perbill;
use sp_std::{collections::btree_map::BTreeMap, prelude::*, vec};

//pub type AccountId = u128;
pub type LandId = u64;
pub type EstateId = u64;
pub type MetaverseId = u64;

const SEED: u32 = 0;

const METAVERSE_ID: u64 = 0;
pub const ALICE_METAVERSE_ID: MetaverseId = 1;
pub const DEMO_METAVERSE_ID: MetaverseId = 3;

const MAX_BOUND: (i32, i32) = (-100, 100);
const COORDINATE_IN_1: (i32, i32) = (-10, 10);
const COORDINATE_IN_2: (i32, i32) = (-5, 5);
const COORDINATE_OUT: (i32, i32) = (0, 101);
const COORDINATE_IN_AUCTION: (i32, i32) = (99, 99);
const ESTATE_IN_AUCTION: EstateId = 99;
const CURRENCY_ID: FungibleTokenId = FungibleTokenId::NativeToken(0);

fn next_block() {
	Auction::on_finalize(System::block_number());
	System::set_block_number(System::block_number() + 1);
	Auction::on_initialize(System::block_number());
}

pub fn run_to_block(n: u32) {
	while System::block_number() < n {
		next_block();
	}
}

runtime_benchmarks! {
	{ Runtime, auction }

	// create_new_auction at global level
	create_new_auction{
		System::set_block_number(1u32.into());
		let caller: AccountId = account("caller", 0, SEED);
		set_balance(CURRENCY_ID, &caller, dollar(10));
		create_nft_group();
		create_nft_group();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1u8]);
		mint_NFT(&caller, 2u32);
		next_block();
	}: _(RawOrigin::Signed(caller.clone()), ItemId::NFT(2,0), 100u32.into(), 100u32.into(), ListingLevel::Local(METAVERSE_ID))

	// create_new_buy_now
	create_new_buy_now{
		System::set_block_number(1u32.into());
		let caller: AccountId = account("caller", 0, SEED);
		set_balance(CURRENCY_ID, &caller, dollar(10));
		create_nft_group();
		create_nft_group();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1u8]);
		mint_NFT(&caller, 2u32);
		next_block();
	}: _(RawOrigin::Signed(caller.clone()), ItemId::NFT(2,0), 100u32.into(), 100u32.into(), ListingLevel::Local(METAVERSE_ID))

	// bid
	bid{
		System::set_block_number(1u32.into());
		let caller: AccountId = account("caller", 0, SEED);
		set_balance(CURRENCY_ID, &caller, dollar(10));
		let bidder: AccountId = account("bidder", 0, SEED);
		set_balance(CURRENCY_ID, &bidder, dollar(20));
		create_nft_group();
		create_nft_group();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1u8]);
		mint_NFT(&caller, 2u32);
		next_block();
		Auction::create_new_auction(RawOrigin::Signed(caller.clone()).into(), ItemId::NFT(2,0), 100u32.into(), 100u32.into(), ListingLevel::Local(METAVERSE_ID));
		next_block();
	}: _(RawOrigin::Signed(bidder.clone()), 0u32.into(), 100u32.into())

	// buy_now
	buy_now{
		System::set_block_number(1u32.into());
		let caller: AccountId = account("caller", 0, SEED);
		set_balance(CURRENCY_ID, &caller, dollar(10));
		let bidder: AccountId = account("bidder", 0, SEED);
		set_balance(CURRENCY_ID, &bidder, dollar(20));
		create_nft_group();
		create_nft_group();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1u8]);
		mint_NFT(&caller, 2u32);
		next_block();
		Auction::create_new_buy_now(RawOrigin::Signed(caller.clone()).into(), ItemId::NFT(2,0), 100u32.into(), 100u32.into(), ListingLevel::Local(METAVERSE_ID));
		next_block();
	}: _(RawOrigin::Signed(bidder.clone()), 0u32.into(), 100u32.into())

	authorise_metaverse_collection{
		let alice: AccountId = account("alice", 0, SEED);
		set_balance(CURRENCY_ID, &alice, dollar(10));
		create_nft_group();
		create_nft_group();
		Metaverse::create_metaverse(RawOrigin::Signed(alice.clone()).into(), vec![1u8]);
	}: _(RawOrigin::Signed(alice), 0u32.into(), METAVERSE_ID)

	remove_authorise_metaverse_collection {
		let alice: AccountId = account("alice", 0, SEED);
		set_balance(CURRENCY_ID, &alice, dollar(10));
		create_nft_group();
		create_nft_group();
		Metaverse::create_metaverse(RawOrigin::Signed(alice.clone()).into(), vec![1u8]);
		Auction::authorise_metaverse_collection(RawOrigin::Signed(alice.clone()).into(), 0u32.into(), METAVERSE_ID);
	}: _(RawOrigin::Signed(alice), 0u32.into(), METAVERSE_ID)

	on_finalize {
		System::set_block_number(1u32.into());
		let caller = account("caller", 0, SEED);
		set_balance(CURRENCY_ID, &caller, dollar(10));
		let bidder = account("bidder", 1, SEED);
		set_balance(CURRENCY_ID, &bidder, dollar(10));
		create_nft_group();
		create_nft_group();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1u8]);
		mint_NFT(&caller, 2u32);
		Auction::create_new_auction(RawOrigin::Signed(caller.clone()).into(), ItemId::NFT(2,0), 100u32.into(), MinimumAuctionDuration::get(), ListingLevel::Local(METAVERSE_ID));
		Auction::bid(RawOrigin::Signed(bidder.clone()).into(), 0u32.into(), 100u32.into());
	}: {
		Auction::on_finalize(System::block_number() + MinimumAuctionDuration::get());
	}
}

#[cfg(test)]
mod tests {
	use orml_benchmarking::impl_benchmark_test_suite;

	use crate::benchmarking::utils::tests::new_test_ext;

	use super::*;

	impl_benchmark_test_suite!(new_test_ext(),);
}
