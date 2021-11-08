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
pub use crate::Pallet as MetaverseModule;
use crate::{Call, Config};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;
use primitives::Balance;
use sp_runtime::traits::{AccountIdConversion, StaticLookup, UniqueSaturatedInto};
// use orml_traits::BasicCurrencyExtended;
// use primitives::{UndeployedLandBlock, UndeployedLandBlockId, UndeployedLandBlockType};

pub type AccountId = u128;
// pub type LandId = u64;
// pub type EstateId = u64;

const SEED: u32 = 0;

const METAVERSE_ID: u64 = 1;
const ALICE: AccountId = 1;
const BENEFICIARY_ID: AccountId = 99;

const MAX_BOUND: (i32, i32) = (-100, 100);
const COORDINATE_IN_1: (i32, i32) = (-10, 10);
const COORDINATE_IN_2: (i32, i32) = (-5, 5);
const COORDINATE_OUT: (i32, i32) = (0, 101);
const COORDINATE_IN_AUCTION: (i32, i32) = (99, 99);
// const ESTATE_IN_AUCTION: EstateId = 99;

fn dollar(d: u32) -> Balance {
	let d: Balance = d.into();
	d.saturating_mul(1_000_000_000_000_000_000)
}

// fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
// 	let caller: T::AccountId = account(name, index, SEED);
// 	T::Currency::make_free_balance_be(&caller, dollar(100).unique_saturated_into());
// 	caller
// }

// fn issue_new_undeployed_land_block<T: Config>(n: u32) -> Result<bool, &'static str> {
// 	let caller = funded_account::<T>("caller", 0);
// 	EstateModule::<T>::issue_undeployed_land_blocks(
// 		RawOrigin::Root.into(),
// 		caller,
// 		n,
// 		100,
// 		UndeployedLandBlockType::Transferable,
// 	);
//
// 	Ok(true)
// }

benchmarks! {
	// // set_max_bounds
	// set_max_bounds{
	// }: _(RawOrigin::Root, METAVERSE_ID, MAX_BOUND)
	// verify {
	// 	assert_eq!(crate::Pallet::<T>::get_max_bounds(METAVERSE_ID), MAX_BOUND)
	// }

	// create_metaverse
	create_metaverse{
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());
	}: _(RawOrigin::Signed(caller.clone()), vec![1])
	verify {
		// assert_eq!(crate::Pallet::<T>::get_max_bounds(METAVERSE_ID), MAX_BOUND)
	}
}

impl_benchmark_test_suite!(Pallet, crate::benchmarking::tests::new_test_ext(), crate::mock::Test);
