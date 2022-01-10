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

//! Benchmarks for the crowdloan module.

#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::assert_ok;
use frame_support::traits::{Currency, Get};
use frame_system::{Origin, RawOrigin};
use sp_runtime::traits::{AccountIdConversion, StaticLookup, UniqueSaturatedInto};
use sp_std::prelude::*;
use sp_std::vec;

use primitives::{UndeployedLandBlock, UndeployedLandBlockId, UndeployedLandBlockType};
// use orml_traits::BasicCurrencyExtended;
use primitives::Balance;

#[allow(unused)]
pub use crate::Pallet as CrowdloanModule;
pub use crate::*;

pub type AccountId = u128;
pub type LandId = u64;
pub type EstateId = u64;

const SEED: u32 = 0;

const METAVERSE_ID: u64 = 1;
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

benchmarks! {

	// set distributor
	set_distributor_origin{
		let distributor: T::AccountId = whitelisted_caller();

	}: _(RawOrigin::Root, distributor.clone())
	verify {
		assert_eq!(crate::Pallet::<T>::is_accepted_origin(&distributor), true)
	}

	// remove distributor
	remove_distributor_origin{
		let distributor: T::AccountId = whitelisted_caller();

		crate::Pallet::<T>::set_distributor_origin(RawOrigin::Root.into(), distributor.clone());

	}: _(RawOrigin::Root, distributor.clone())
	verify {
		assert_eq!(crate::Pallet::<T>::is_accepted_origin(&distributor), false)
	}

	// transfer_unlocked_reward
	transfer_unlocked_reward{
		let caller = funded_account::<T>("caller", 0);
		crate::Pallet::<T>::set_distributor_origin(RawOrigin::Root.into(), caller.clone());

		let target: T::AccountId = account("target", 0, SEED);

	}: _(RawOrigin::Signed(caller.clone()), target, 100u32.into())

	// transfer_vested_reward
	transfer_vested_reward{
		let caller = funded_account::<T>("caller", 0);
		crate::Pallet::<T>::set_distributor_origin(RawOrigin::Root.into(), caller.clone());

		let target: T::AccountId = account("target", 0, SEED);
		let target_lookup = T::Lookup::unlookup(target.clone());

		let vested_schedule = VestingInfo::new(100u32.into(), 10u32.into(), 1u32.into());

	}: _(RawOrigin::Signed(caller.clone()), target_lookup, vested_schedule)

	// remove_vested_reward
	remove_vested_reward{
		let caller = funded_account::<T>("caller", 0);
		crate::Pallet::<T>::set_distributor_origin(RawOrigin::Root.into(), caller.clone());

		let target: T::AccountId = account("target", 0, SEED);
		let target_lookup = T::Lookup::unlookup(target.clone());

		let vested_schedule = VestingInfo::new(100u32.into(), 10u32.into(), 1u32.into());

		crate::Pallet::<T>::transfer_vested_reward(RawOrigin::Signed(caller.clone()).into(), target_lookup.clone(), vested_schedule);
	}: _(RawOrigin::Root, target, 0)
}

impl_benchmark_test_suite!(Pallet, crate::benchmarking::tests::new_test_ext(), crate::mock::Test);
