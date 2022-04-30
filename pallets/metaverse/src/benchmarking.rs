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
use crate::*;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;
use primitives::Balance;
use sp_runtime::traits::{AccountIdConversion, StaticLookup, UniqueSaturatedInto};
use sp_runtime::Perbill;

pub type AccountId = u128;

const SEED: u32 = 0;

const ALICE: AccountId = 1;
const BENEFICIARY_ID: AccountId = 99;

const MAX_BOUND: (i32, i32) = (-100, 100);
const COORDINATE_IN_1: (i32, i32) = (-10, 10);
const COORDINATE_IN_2: (i32, i32) = (-5, 5);
const COORDINATE_OUT: (i32, i32) = (0, 101);
const COORDINATE_IN_AUCTION: (i32, i32) = (99, 99);

fn dollar(d: u32) -> Balance {
	let d: Balance = d.into();
	d.saturating_mul(1_000_000_000_000_000_000)
}

fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	T::Currency::make_free_balance_be(&caller, dollar(100).unique_saturated_into());
	caller
}

benchmarks! {
	// create_metaverse
	create_metaverse{
		let caller = funded_account::<T>("caller", 0);
	}: _(RawOrigin::Signed(caller.clone()), vec![1])
	verify {
		let metaverse = crate::Pallet::<T>::get_metaverse(0);
		match metaverse {
			Some(a) => {
				assert_eq!(a.owner, caller.clone());
				assert_eq!(a.is_frozen, false);
				assert_eq!(a.metadata, vec![1]);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	}

	// transfer_metaverse
	transfer_metaverse {
		let caller = funded_account::<T>("caller", 0);
		let target = funded_account::<T>("target", 0);

		crate::Pallet::<T>::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1]);
	}: _(RawOrigin::Signed(caller.clone()), target.clone(), 0)
	verify {
		let metaverse = crate::Pallet::<T>::get_metaverse(0);
		match metaverse {
			Some(a) => {
				assert_eq!(a.owner, target.clone());
				assert_eq!(a.is_frozen, false);
				assert_eq!(a.metadata, vec![1]);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	}

	// freeze_metaverse
	freeze_metaverse{
		let caller = funded_account::<T>("caller", 0);
		let target = funded_account::<T>("target", 0);

		crate::Pallet::<T>::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1]);
	}: _(RawOrigin::Root, 0)
	verify {
		let metaverse = crate::Pallet::<T>::get_metaverse(0);
		match metaverse {
			Some(a) => {
				assert_eq!(a.is_frozen, true);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	}

	// unfreeze_metaverse
	unfreeze_metaverse{
		let caller = funded_account::<T>("caller", 0);
		let target = funded_account::<T>("target", 0);

		crate::Pallet::<T>::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1]);
		crate::Pallet::<T>::freeze_metaverse(RawOrigin::Root.into(), 0);
	}: _(RawOrigin::Root, 0)
	verify {
		let metaverse = crate::Pallet::<T>::get_metaverse(0);
		match metaverse {
			Some(a) => {
				// Verify details of Metaverse
				assert_eq!(a.is_frozen, false);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	}

	// destroy_metaverse
	destroy_metaverse{
		let caller = funded_account::<T>("caller", 0);
		let target = funded_account::<T>("target", 0);

		crate::Pallet::<T>::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1]);
		crate::Pallet::<T>::freeze_metaverse(RawOrigin::Root.into(), 0);
	}: _(RawOrigin::Root, 0)
	verify {
		assert_eq!(crate::Pallet::<T>::get_metaverse(0), None);
	}

	// register_metaverse
	register_metaverse{
		let caller = funded_account::<T>("caller", 0);
		let target = funded_account::<T>("target", 0);

		crate::Pallet::<T>::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1]);
	}: _(RawOrigin::Signed(caller.clone()), 0)
	verify {
		let metaverse = crate::Pallet::<T>::get_registered_metaverse(0);
		match metaverse {
			Some(a) => {
				assert_eq!(1, 1);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	}

	// stake
	stake{
		let caller = funded_account::<T>("caller", 0);
		let target = funded_account::<T>("target", 0);
		let amount = <<T as Config>::MinStakingAmount as Get<BalanceOf<T>>>::get();

		crate::Pallet::<T>::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1]);
		crate::Pallet::<T>::register_metaverse(RawOrigin::Signed(caller.clone()).into(), 0);
	}: _(RawOrigin::Signed(caller.clone()), 0, (amount+1u32.into()).into())
	verify {
		let staking_info = crate::Pallet::<T>::staking_info(caller);
		assert_eq!(staking_info, (amount+1u32.into()).into());
	}

	// unstake_and_withdraw
	unstake_and_withdraw{
		let caller = funded_account::<T>("caller", 0);
		let target = funded_account::<T>("target", 0);

		let amount = <<T as Config>::MinStakingAmount as Get<BalanceOf<T>>>::get();


		crate::Pallet::<T>::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1]);
		crate::Pallet::<T>::register_metaverse(RawOrigin::Signed(caller.clone()).into(), 0);
		crate::Pallet::<T>::stake(RawOrigin::Signed(caller.clone()).into(), 0, (amount+1u32.into()).into());
	}: _(RawOrigin::Signed(caller.clone()), 0, 1u32.into())
	verify {
		let staking_info = crate::Pallet::<T>::staking_info(caller);
		assert_eq!(staking_info, amount.into());
	}

	// update metaverse marketplace listing fee
	update_metaverse_listing_fee {
		let caller = funded_account::<T>("caller", 0);
		crate::Pallet::<T>::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1]);
		crate::Pallet::<T>::register_metaverse(RawOrigin::Signed(caller.clone()).into(), 0);
	}: _(RawOrigin::Signed(caller.clone()), 0, Perbill::from_percent(1u32))
	verify {
		assert_eq!(crate::Pallet::<T>::get_metaverse_marketplace_listing_fee(0), Perbill::from_percent(1u32))
	}
}

impl_benchmark_test_suite!(Pallet, crate::benchmarking::tests::new_test_ext(), crate::mock::Test);
