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

//! Benchmarks for the economy module.

#![cfg(feature = "runtime-benchmarks")]

// use super::*;
use sp_std::{collections::btree_map::BTreeMap, prelude::*, vec};

#[allow(unused)]
use crate::{Balances, Call, Currencies, Economy, Event, Nft, Runtime, System};
use economy::Config;
use frame_benchmarking::{account, whitelisted_caller};
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;
use orml_benchmarking::runtime_benchmarks;
use primitives::{Attributes, Balance, ClassId, CollectionType, FungibleTokenId, GroupCollectionId, TokenType};
use sp_runtime::traits::{AccountIdConversion, StaticLookup, UniqueSaturatedInto};

pub type AccountId = u128;

const SEED: u32 = 0;

const ALICE: AccountId = 1;
const BENEFICIARY_ID: AccountId = 99;

const MAX_BOUND: (i32, i32) = (-100, 100);
const COORDINATE_IN_1: (i32, i32) = (-10, 10);
const COORDINATE_IN_2: (i32, i32) = (-5, 5);
const COORDINATE_OUT: (i32, i32) = (0, 101);
const COORDINATE_IN_AUCTION: (i32, i32) = (99, 99);

const EXCHANGE_RATE: Balance = 10000;
// const BENEFICIARY_NFT: (ClassId, TokenId) = (1, 0);
//
// const ELEMENT_INDEX_ID: ElementId = 1;

const STAKING_AMOUNT: Balance = 1000;
const UNSTAKING_AMOUNT: Balance = 100;

const COLLECTION_ID: GroupCollectionId = 0;
const CLASS_ID: ClassId = 0;

const NATIVE: FungibleTokenId = FungibleTokenId::NativeToken(0);

fn dollar(d: u32) -> Balance {
	let d: Balance = d.into();
	d.saturating_mul(1_000_000_000_000_000_000)
}

// fn funded_account(name: &'static str, index: u32) -> AccountId {
//     let caller: AccountId = account(name, index, SEED);
// 	Currencies::make_free_balance_be(&caller, dollar(10000).unique_saturated_into());
//     caller
// }

fn assert_last_event(generic_event: Event) {
	System::assert_last_event(generic_event.into());
}

fn mint_NFT() {
	//caller: T::AccountId) {
	Nft::create_group(RawOrigin::Root.into(), vec![1], vec![1]);
	Nft::create_class(
		// RawOrigin::Signed(caller.clone()).into(),
		RawOrigin::Root.into(),
		vec![1],
		test_attributes(1),
		0u32.into(),
		TokenType::Transferable,
		CollectionType::Collectable,
	);
	Nft::mint(
		// RawOrigin::Signed(caller.clone()).into(),
		RawOrigin::Root.into(),
		0u32.into(),
		vec![1],
		test_attributes(1),
		3,
	);
}

fn test_attributes(x: u8) -> Attributes {
	let mut attr: Attributes = BTreeMap::new();
	attr.insert(vec![x, x + 5], vec![x, x + 10]);
	attr
}

runtime_benchmarks! {
	{ Runtime, economy }

	// // set_bit_power_exchange_rate
	// set_bit_power_exchange_rate{
	// 	let from: AccountId = whitelisted_caller();
	//
	// }: set_bit_power_exchange_rate(RawOrigin::Root, EXCHANGE_RATE)
	// verify {
	// 	assert_eq!(1, 1);
	// }

	// // set_bit_power_exchange_rate
	authorize_power_generator_collection{
		let funder: AccountId = account("funder", 0, SEED);

		mint_NFT();

	}: _(RawOrigin::Root, COLLECTION_ID, CLASS_ID)
	verify {
		// let new_rate = crate::Pallet::<T>::get_bit_power_exchange_rate();
		// assert_eq!(crate::Pallet::<T>::get_authorized_generator_collection((COLLECTION_ID, CLASS_ID)), Some(()))

		// assert_last_event(module_dex::Event::EnableTradingPair{trading_pair: trading_pair}.into());

		assert_eq!(1, 1);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::benchmarking::utils::tests::new_test_ext;
	use orml_benchmarking::impl_benchmark_test_suite;

	impl_benchmark_test_suite!(new_test_ext(),);
}
