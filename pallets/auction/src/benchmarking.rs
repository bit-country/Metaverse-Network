// This file is part of Bit.Country

// Copyright (C) 2020-2021 Bit.Country.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Benchmarks for the estate module.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use sp_std::prelude::*;
use sp_std::vec;

#[allow(unused)]
pub use crate::Pallet as AuctionModule;
use pallet_estate::Pallet as EstateModule;
use pallet_metaverse::Pallet as MetaverseModule;
use pallet_nft::Pallet as NFTModule;
use pallet_nft::{CollectionType, TokenType};

use crate::{Call, Config};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;
use primitives::Balance;
use sp_runtime::traits::{AccountIdConversion, StaticLookup, UniqueSaturatedInto};
// use orml_traits::BasicCurrencyExtended;
use auction_manager::{CheckAuctionItemHandler, ListingLevel};
use primitives::{UndeployedLandBlock, UndeployedLandBlockId, UndeployedLandBlockType};

pub type AccountId = u128;
pub type LandId = u64;
pub type EstateId = u64;

const SEED: u32 = 0;

const METAVERSE_ID: u64 = 0;
const ALICE: AccountId = 1;
const BENEFICIARY_ID: AccountId = 99;

const MAX_BOUND: (i32, i32) = (-100, 100);
const COORDINATE_IN_1: (i32, i32) = (-10, 10);
const COORDINATE_IN_2: (i32, i32) = (-5, 5);
const COORDINATE_OUT: (i32, i32) = (0, 101);
const COORDINATE_IN_AUCTION: (i32, i32) = (99, 99);
const ESTATE_IN_AUCTION: EstateId = 99;

fn dollar(d: u32) -> Balance {
	let d: Balance = d.into();
	d.saturating_mul(1_000_000_000_000_000_000)
}

fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	let initial_balance = dollar(1000);

	<T as pallet::Config>::Currency::make_free_balance_be(&caller, initial_balance.unique_saturated_into());
	caller
}

fn mint_NFT<T: Config>(caller: T::AccountId) {
	// let caller: T::AccountId = account(name, index, SEED);
	// let initial_balance = dollar(1000);

	// <T as pallet::Config>::Currency::make_free_balance_be(&caller,
	// initial_balance.unique_saturated_into());
	NFTModule::<T>::create_group(RawOrigin::Root.into(), vec![1], vec![1]);
	NFTModule::<T>::create_class(
		RawOrigin::Signed(caller.clone()).into(),
		vec![1],
		0u32.into(),
		TokenType::Transferable,
		CollectionType::Collectable,
	);
	NFTModule::<T>::mint(
		RawOrigin::Signed(caller.clone()).into(),
		0u32.into(),
		vec![1],
		vec![2],
		vec![1, 2, 3],
		3,
	);
}

benchmarks! {
	// create_new_auction
	create_new_auction{
		frame_system::Pallet::<T>::set_block_number(1u32.into());
		// let blockNumber: BlockNumber = T::MinimumAuctionDuration::get().into().saturating_add(5u32.into());

		let caller = funded_account::<T>("caller", 0);
		// MetaverseModule::<T>::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1]);
		// EstateModule::<T>::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND)

		// T::MetaverseInfoSource::update_metaverse_token(METAVERSE_ID, 0u32.into());
		mint_NFT::<T>(caller.clone());
	}: _(RawOrigin::Signed(caller.clone()), ItemId::NFT(0), 100u32.into(), 100u32.into(), ListingLevel::Local(METAVERSE_ID))

	// create_new_buy_now
	create_new_buy_now{
		frame_system::Pallet::<T>::set_block_number(1u32.into());
		// let blockNumber: BlockNumber = T::MinimumAuctionDuration::get().into().saturating_add(5u32.into());

		let caller = funded_account::<T>("caller", 0);
		mint_NFT::<T>(caller.clone());
	}: _(RawOrigin::Signed(caller.clone()), ItemId::NFT(0), 100u32.into(), 100u32.into(), ListingLevel::Global)

	// bid
	bid{
		frame_system::Pallet::<T>::set_block_number(1u32.into());

		let caller = funded_account::<T>("caller", 0);
		let bidder = funded_account::<T>("bidder", 0);
		mint_NFT::<T>(caller.clone());

		crate::Pallet::<T>::create_new_auction(RawOrigin::Signed(caller.clone()).into(), ItemId::NFT(0), 100u32.into(), 100u32.into(), ListingLevel::Global);
	}: _(RawOrigin::Signed(bidder.clone()), 0u32.into(), 100u32.into())

	// buy_now
	buy_now{
		frame_system::Pallet::<T>::set_block_number(1u32.into());

		let caller = funded_account::<T>("caller", 0);
		let bidder = funded_account::<T>("bidder", 0);
		mint_NFT::<T>(caller.clone());

		crate::Pallet::<T>::create_new_buy_now(RawOrigin::Signed(caller.clone()).into(), ItemId::NFT(0), 100u32.into(), 100u32.into(), ListingLevel::Global);
	}: _(RawOrigin::Signed(bidder.clone()), 0u32.into(), 100u32.into())

	// bid_local
	// bid_local{
	// 	frame_system::Pallet::<T>::set_block_number(1u32.into());
	//
	// 	let caller = funded_account::<T>("caller", 0);
	// 	let bidder = funded_account::<T>("bidder", 0);
	// 	mint_NFT::<T>(caller.clone());
	//
	// 	crate::Pallet::<T>::create_new_buy_now(RawOrigin::Signed(caller.clone()).into(), ItemId::NFT(0), 100u32.into(), 100u32.into(), ListingLevel::Local(METAVERSE_ID));
	// }: _(RawOrigin::Signed(bidder.clone()), 0u32.into(), METAVERSE_ID, 100u32.into())

	// buy_now_local
	// buy_now_local{
	// 	frame_system::Pallet::<T>::set_block_number(1u32.into());
	//
	// 	let caller = funded_account::<T>("caller", 0);
	// 	let bidder = funded_account::<T>("bidder", 0);
	// 	mint_NFT::<T>(caller.clone());
	//
	// 	crate::Pallet::<T>::create_new_buy_now(RawOrigin::Signed(caller.clone()).into(), ItemId::NFT(0), 100u32.into(), 100u32.into(), ListingLevel::Local(METAVERSE_ID));
	// }: _(RawOrigin::Signed(bidder.clone()), 0u32.into(), METAVERSE_ID, 100u32.into())
}

impl_benchmark_test_suite!(Pallet, crate::benchmarking::tests::new_test_ext(), crate::mock::Test);
