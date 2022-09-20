#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::{account, whitelisted_caller};
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;
use orml_benchmarking::runtime_benchmarks;
use sp_runtime::traits::{AccountIdConversion, Lookup, StaticLookup, UniqueSaturatedInto};

use auction_manager::AuctionType;
use core_primitives::TokenType;
use primitives::{AccountId, Balance, FungibleTokenId};

use crate::{Call, Continuum, Currencies, Event, Metaverse, Runtime, System};

use super::utils::{create_nft_group, dollar, set_balance, set_metaverse_treasury_initial_balance};

const CURRENCY_ID: FungibleTokenId = FungibleTokenId::NativeToken(0);

runtime_benchmarks! {
	{ Runtime, continuum }

	set_allow_buy_now{
	}: _(RawOrigin::Root, true)

	set_max_bounds{
	}: _(RawOrigin::Root, (10i32, 10i32))

	issue_map_slot{
		Continuum::set_max_bounds(RawOrigin::Root.into(), (0i32, 0i32));
	}: _(RawOrigin::Root, (0i32, 0i32), TokenType::Transferable)

	create_new_auction{
		Continuum::set_max_bounds(RawOrigin::Root.into(), (0i32, 0i32));
		Continuum::issue_map_slot(RawOrigin::Root.into(), (0i32, 0i32), TokenType::Transferable);
		set_balance(CURRENCY_ID, &Continuum::account_id(), dollar(10));
	}: _(RawOrigin::Root, (0i32, 0i32), AuctionType::Auction,  1u32.into(), 200u32.into())

	buy_map_spot{
		let buyer: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &buyer, dollar(10));
		create_nft_group();
		set_metaverse_treasury_initial_balance();
		Metaverse::create_metaverse(RawOrigin::Signed(buyer.clone()).into(), vec![1u8]);
		Continuum::set_max_bounds(RawOrigin::Root.into(), (0i32, 0i32));
		Continuum::set_allow_buy_now(RawOrigin::Root.into(), true);
		Continuum::issue_map_slot(RawOrigin::Root.into(), (0i32, 0i32), TokenType::Transferable);
		set_balance(CURRENCY_ID, &Continuum::account_id(), dollar(10));
		Continuum::create_new_auction(RawOrigin::Root.into(), (0i32, 0i32), AuctionType::BuyNow,  1u32.into(), 200u32.into());
	}: _(RawOrigin::Signed(buyer.clone()), 0u32.into(), 1u32.into(), 0u32.into())

	bid_map_spot{
		let bidder: AccountId = whitelisted_caller();
		set_balance(CURRENCY_ID, &bidder, dollar(10));
		create_nft_group();
		set_metaverse_treasury_initial_balance();
		Metaverse::create_metaverse(RawOrigin::Signed(bidder.clone()).into(), vec![1u8]);
		Continuum::set_max_bounds(RawOrigin::Root.into(), (0i32, 0i32));
		Continuum::issue_map_slot(RawOrigin::Root.into(), (0i32, 0i32), TokenType::Transferable);
		set_balance(CURRENCY_ID, &Continuum::account_id(), dollar(10));
		Continuum::create_new_auction(RawOrigin::Root.into(), (0i32, 0i32), AuctionType::Auction,  1u32.into(), 200u32.into());
	}: _(RawOrigin::Signed(bidder.clone()), 0u32.into(), 2u32.into(), 0u32.into())
}

#[cfg(test)]
mod tests {
	use orml_benchmarking::impl_benchmark_test_suite;

	use crate::benchmarking::utils::tests::new_test_ext;

	use super::*;

	impl_benchmark_test_suite!(new_test_ext(),);
}
