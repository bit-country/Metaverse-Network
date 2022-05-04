// This file is part of Metaverse.Network & Bit.Country

// Copyright (C) 2020-2022 Metaverse.Network & Bit.Country .
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

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;
use sp_runtime::traits::{AccountIdConversion, StaticLookup, UniqueSaturatedInto};
use sp_runtime::Perbill;
use sp_std::{collections::btree_map::BTreeMap, prelude::*, vec};
// use orml_traits::BasicCurrencyExtended;
use auction_manager::{CheckAuctionItemHandler, ListingLevel};
use core_primitives::{Attributes, CollectionType, MetaverseInfo, MetaverseTrait, NftMetadata};
use pallet_nft::{NFTTrait, TokenType};
// use pallet_estate::Pallet as EstateModule;
use pallet_metaverse::Pallet as MetaverseModule;

use primitives::{
	Balance, FungibleTokenId, UndeployedLandBlock, UndeployedLandBlockId, UndeployedLandBlockType, LAND_CLASS_ID,
};

#[allow(unused)]
pub use crate::Pallet as AuctionModule;
use crate::{Call, Config};

use super::*;

pub type AccountId = u128;
pub type LandId = u64;
pub type EstateId = u64;
pub type MetaverseId = u64;

const SEED: u32 = 0;

const METAVERSE_ID: u64 = 0;
const ALICE: AccountId = 1;
const BENEFICIARY_ID: AccountId = 99;
pub const ALICE_METAVERSE_ID: MetaverseId = 1;
pub const DEMO_METAVERSE_ID: MetaverseId = 3;

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

fn test_attributes(x: u8) -> Attributes {
	let mut attr: Attributes = BTreeMap::new();
	attr.insert(vec![x, x + 5], vec![x, x + 10]);
	attr
}

fn mint_NFT<T: Config>(caller: T::AccountId) {
	//T::NFTHandler::mint_land_nft(caller.clone().into(), vec![1], test_attributes(1));
	T::NFTHandler::create_token_class(
		&caller.clone(),
		vec![1],
		test_attributes(1),
		0u32.into(),
		TokenType::Transferable,
		CollectionType::Collectable,
		Perbill::from_percent(0u32),
	);

	T::NFTHandler::mint_token(&caller.clone(), 0u32.into(), vec![1], test_attributes(1));
}

fn create_metaverse_for_account<T: Config>(caller: T::AccountId) {
	//pallet_metaverse::Pallet::<T>::create_metaverse(
	//	RawOrigin::Signed(caller.clone()).into(),
	//	vec![1u8],
	//);
}

benchmarks! {
	// create_new_auction at global level
	create_new_auction{
		frame_system::Pallet::<T>::set_block_number(1u32.into());

		let caller = funded_account::<T>("caller", 0);
		mint_NFT::<T>(caller.clone());
	}: _(RawOrigin::Signed(caller.clone()), ItemId::NFT(0,0), 100u32.into(), 100u32.into(), ListingLevel::Global)

	// create_new_buy_now
	create_new_buy_now{
		frame_system::Pallet::<T>::set_block_number(1u32.into());

		let caller = funded_account::<T>("caller", 0);
		mint_NFT::<T>(caller.clone());
	}: _(RawOrigin::Signed(caller.clone()), ItemId::NFT(0,0), 100u32.into(), 100u32.into(), ListingLevel::Global)

	// bid
	bid{
		frame_system::Pallet::<T>::set_block_number(1u32.into());

		let caller = funded_account::<T>("caller", 0);
		let bidder = funded_account::<T>("bidder", 0);
		mint_NFT::<T>(caller.clone());

		crate::Pallet::<T>::create_new_auction(RawOrigin::Signed(caller.clone()).into(), ItemId::NFT(0,0), 100u32.into(), 100u32.into(), ListingLevel::Global);
	}: _(RawOrigin::Signed(bidder.clone()), 0u32.into(), 100u32.into())

	// buy_now
	buy_now{
		frame_system::Pallet::<T>::set_block_number(1u32.into());

		let caller = funded_account::<T>("caller", 0);
		let bidder = funded_account::<T>("bidder", 0);
		mint_NFT::<T>(caller.clone());

		crate::Pallet::<T>::create_new_buy_now(RawOrigin::Signed(caller.clone()).into(), ItemId::NFT(0,0), 100u32.into(), 100u32.into(), ListingLevel::Global);
	}: _(RawOrigin::Signed(bidder.clone()), 0u32.into(), 100u32.into())

	authorise_metaverse_collection{
		let alice = funded_account::<T>("alice", 0);
		create_metaverse_for_account::<T>(alice.clone());
	}: _(RawOrigin::Signed(alice), 0u32.into(), METAVERSE_ID)

	remove_authorise_metaverse_collection {
		let alice = funded_account::<T>("alice", 0);
		create_metaverse_for_account::<T>(alice.clone());
		crate::Pallet::<T>::authorise_metaverse_collection(RawOrigin::Signed(alice.clone()).into(), 0u32.into(), METAVERSE_ID);
	}: _(RawOrigin::Signed(alice), 0u32.into(), METAVERSE_ID)
}

impl_benchmark_test_suite!(Pallet, crate::benchmarking::tests::new_test_ext(), crate::mock::Test);
