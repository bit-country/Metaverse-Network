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

use sp_std::prelude::*;
use sp_std::vec;

#[allow(unused)]
pub use crate::Pallet as EstateModule;
use crate::{Call, Config};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::Get;
use frame_system::RawOrigin;
use primitives::Balance;
use sp_runtime::traits::{AccountIdConversion, StaticLookup, UniqueSaturatedInto};

pub struct Pallet<T: Config>(crate::Pallet<T>);

const SEED: u32 = 0;

const METAVERSE_ID: u64 = 1;

const MAX_BOUND: (i32, i32) = (-100, 100);

fn dollar(d: u32) -> Balance {
	let d: Balance = d.into();
	d.saturating_mul(1_000_000_000_000_000_000)
}

benchmarks! {
	// set_max_bounds
	set_max_bounds{
	}: _(RawOrigin::Root ,METAVERSE_ID, MAX_BOUND)
	verify {
		assert_eq!(crate::Pallet::<T>::get_max_bounds(METAVERSE_ID), MAX_BOUND)
	}
}
//
//	create_class{
//		let caller = whitelisted_caller();
//		let initial_balance = dollar(1000);
//		<T as social_currencies::Config>::NativeCurrency::update_balance(&caller,
// initial_balance.unique_saturated_into())?; 		crate::Pallet::<T>::create_group(RawOrigin::Root.
// into(), vec![1],vec![1]); 	}: _(RawOrigin::Signed(caller),vec![1], 0u32.into(),
// TokenType::Transferable, CollectionType::Collectable)

impl_benchmark_test_suite!(Pallet, crate::benchmarking::tests::new_test_ext(), crate::mock::Test);
