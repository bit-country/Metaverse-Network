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
pub use crate::Pallet as EstateModule;
use crate::{
	pallet::{MintingRateConfig, Round},
	Call, Config, MintingRateInfo, Range,
};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;
use primitives::Balance;
use sp_runtime::traits::{AccountIdConversion, StaticLookup, UniqueSaturatedInto};
// use orml_traits::BasicCurrencyExtended;
use primitives::{UndeployedLandBlock, UndeployedLandBlockId, UndeployedLandBlockType};

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
	T::Currency::make_free_balance_be(&caller, dollar(100).unique_saturated_into());
	caller
}

fn issue_new_undeployed_land_block<T: Config>(n: u32) -> Result<bool, &'static str> {
	let caller = funded_account::<T>("caller", 0);
	EstateModule::<T>::issue_undeployed_land_blocks(
		RawOrigin::Root.into(),
		caller,
		n,
		100,
		UndeployedLandBlockType::Transferable,
	);

	Ok(true)
}

benchmarks! {
	// set_max_bounds
	set_max_bounds{
	}: _(RawOrigin::Root, METAVERSE_ID, MAX_BOUND)
	verify {
		assert_eq!(crate::Pallet::<T>::get_max_bounds(METAVERSE_ID), MAX_BOUND)
	}

	// mint_land
	mint_land {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		crate::Pallet::<T>::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
	}: _(RawOrigin::Root, caller.clone(), METAVERSE_ID, COORDINATE_IN_1)
	verify {
		assert_eq!(crate::Pallet::<T>::get_land_units(METAVERSE_ID, COORDINATE_IN_1), caller.clone())
	}

	// mint_lands
	mint_lands {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		crate::Pallet::<T>::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
	}: _(RawOrigin::Root, caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1, COORDINATE_IN_2])
	verify {
		assert_eq!(crate::Pallet::<T>::get_land_units(METAVERSE_ID, COORDINATE_IN_1), caller.clone());
		assert_eq!(crate::Pallet::<T>::get_land_units(METAVERSE_ID, COORDINATE_IN_2), caller.clone())
	}

	// transfer_land
	transfer_land {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		let target: T::AccountId = account("target", 0, SEED);
		let target_lookup = T::Lookup::unlookup(target.clone());

		let initial_balance = dollar(1000);

		// <T as pallet::Config>::Currency::make_free_balance_be(&caller, initial_balance.unique_saturated_into());
		crate::Pallet::<T>::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
		crate::Pallet::<T>::mint_land(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, COORDINATE_IN_1);

	}: _(RawOrigin::Signed(caller.clone()), target.clone(), METAVERSE_ID, COORDINATE_IN_1)
	verify {
		// TODO: issue with blow line
		// assert_eq!(crate::Pallet::<T>::get_land_units(METAVERSE_ID, COORDINATE_IN_1), target.clone())
	}

	// mint_estate
	mint_estate {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		crate::Pallet::<T>::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
	}: _(RawOrigin::Root, caller, METAVERSE_ID, vec![COORDINATE_IN_1])
	verify {
		assert_eq!(crate::Pallet::<T>::get_estates(0), Some(vec![COORDINATE_IN_1]))
	}

	// create_estate
	create_estate {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		crate::Pallet::<T>::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
		crate::Pallet::<T>::mint_lands(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1, COORDINATE_IN_2]);

	}: _(RawOrigin::Root, caller, METAVERSE_ID, vec![COORDINATE_IN_1, COORDINATE_IN_2])
	verify {
		assert_eq!(crate::Pallet::<T>::get_estates(0), Some(vec![COORDINATE_IN_1, COORDINATE_IN_2]))
	}

	// transfer_estate
	transfer_estate {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		let target: T::AccountId = account("target", 0, SEED);
		let target_lookup = T::Lookup::unlookup(target.clone());

		crate::Pallet::<T>::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
		crate::Pallet::<T>::mint_estate(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1, COORDINATE_IN_2]);

	}: _(RawOrigin::Signed(caller.clone()), target.clone(), 0)
	verify {
		assert_eq!(crate::Pallet::<T>::get_estate_owner(target.clone(), 0), Some(()))
	}

	// issue_undeployed_land_blocks
	issue_undeployed_land_blocks {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

	}: _(RawOrigin::Root, caller.clone(), 20, 100, UndeployedLandBlockType::BoundToAddress)
	verify {
		let issued_undeployed_land_block = crate::Pallet::<T>::get_undeployed_land_block(0);
		match issued_undeployed_land_block {
			Some(a) => {
				// Verify details of UndeployedLandBlock
				assert_eq!(a.owner, caller.clone());
				assert_eq!(a.number_land_units, 100);
				assert_eq!(a.undeployed_land_block_type, UndeployedLandBlockType::BoundToAddress);
				assert_eq!(a.is_frozen, false);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	}

	// freeze_undeployed_land_blocks
	freeze_undeployed_land_blocks {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		issue_new_undeployed_land_block::<T>(5)?;
	}: _(RawOrigin::Root, 0)
	verify {
				let issued_undeployed_land_block = crate::Pallet::<T>::get_undeployed_land_block(0);
		match issued_undeployed_land_block {
			Some(a) => {
				// Verify details of UndeployedLandBlock
				assert_eq!(a.is_frozen, true);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	}

	// unfreeze_undeployed_land_blocks
	unfreeze_undeployed_land_blocks {
	let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		issue_new_undeployed_land_block::<T>(5)?;
		crate::Pallet::<T>::freeze_undeployed_land_blocks(RawOrigin::Root.into(), Default::default());
	}: _(RawOrigin::Root, 0)
	verify {
		let issued_undeployed_land_block = crate::Pallet::<T>::get_undeployed_land_block(0);
		match issued_undeployed_land_block {
			Some(a) => {
				// Verify details of UndeployedLandBlock
				assert_eq!(a.is_frozen, false);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	}

	// burn_undeployed_land_blocks
	burn_undeployed_land_blocks {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		issue_new_undeployed_land_block::<T>(5)?;
		crate::Pallet::<T>::freeze_undeployed_land_blocks(RawOrigin::Root.into(), Default::default());
	}: _(RawOrigin::Root, 0)
	verify {
		assert_eq!(crate::Pallet::<T>::get_undeployed_land_block(0), None)
	}

	// approve_undeployed_land_blocks
	approve_undeployed_land_blocks {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		let target: T::AccountId = account("target", 0, SEED);
		let target_lookup = T::Lookup::unlookup(target.clone());

		crate::Pallet::<T>::issue_undeployed_land_blocks(RawOrigin::Root.into(), caller.clone(), 5, 100, UndeployedLandBlockType::BoundToAddress);
	}: _(RawOrigin::Signed(caller.clone()), target.clone(), Default::default())
	verify {
		let issued_undeployed_land_block = crate::Pallet::<T>::get_undeployed_land_block(0);
		match issued_undeployed_land_block {
			Some(a) => {
				// Verify details of UndeployedLandBlock
				assert_eq!(a.approved, Some(target.clone()));
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	}

	// unapprove_undeployed_land_blocks
	unapprove_undeployed_land_blocks {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		crate::Pallet::<T>::issue_undeployed_land_blocks(RawOrigin::Root.into(), caller.clone(), 5, 100, UndeployedLandBlockType::BoundToAddress);
	}: _(RawOrigin::Signed(caller.clone()), Default::default())
	verify {
		let issued_undeployed_land_block = crate::Pallet::<T>::get_undeployed_land_block(0);
		match issued_undeployed_land_block {
			Some(a) => {
				// Verify details of UndeployedLandBlock
				assert_eq!(a.approved, None);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	}

	// transfer_undeployed_land_blocks
	transfer_undeployed_land_blocks {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		let target: T::AccountId = account("target", 0, SEED);
		let target_lookup = T::Lookup::unlookup(target.clone());

		crate::Pallet::<T>::issue_undeployed_land_blocks(RawOrigin::Root.into(), caller.clone(), 5, 100, UndeployedLandBlockType::Transferable);
	}: _(RawOrigin::Signed(caller.clone()), target.clone(), Default::default())
	verify {
		let issued_undeployed_land_block = crate::Pallet::<T>::get_undeployed_land_block(0);
		match issued_undeployed_land_block {
			Some(a) => {
				// Verify details of UndeployedLandBlock
				assert_eq!(a.owner, target.clone());
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	}

	// deploy_land_block
	deploy_land_block {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		crate::Pallet::<T>::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
		crate::Pallet::<T>::issue_undeployed_land_blocks(RawOrigin::Root.into(), caller.clone(), 5, 100, UndeployedLandBlockType::Transferable);
	}: _(RawOrigin::Signed(caller.clone()), Default::default(), METAVERSE_ID, vec![COORDINATE_IN_1, COORDINATE_IN_2])
	verify {
		let issued_undeployed_land_block = crate::Pallet::<T>::get_undeployed_land_block(0);
		match issued_undeployed_land_block {
			Some(a) => {
				// Verify details of UndeployedLandBlock
				assert_eq!(a.number_land_units, 98);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	}

	active_issue_undeploy_land_block{
		// INITIALIZE RUNTIME STATE
		let minting_info = 	MintingRateInfo {
			expect: Default::default(),
			// 10% minting rate per annual
			annual: 10,
			// Max 100 millions land unit
			max: 100_000_000,
		};
		// Pre issue 5 land blocks x 100 land units
		issue_new_undeployed_land_block::<T>(5)?;
		let min_block_per_round = 5u32;

		let new_round = RoundInfo::new(1u32, 0u32.into(), min_block_per_round.into());

		Round::<T>::put(new_round);
		let high_inflation_rate = MintingRateInfo {
			expect: Default::default(),
			annual: 20,
			// Max 100 millions land unit
			max: 100_000_000,
		};
		MintingRateConfig::<T>::put(high_inflation_rate);

//
//		// PREPARE RUN_TO_BLOCK LOOP
//		let before_running_round_index = EstateModule::<T>::round().current;
//		let round_length: T::BlockNumber = EstateModule::<T>::round().length.into();
//
//
//		let mut now = <frame_system::Pallet<T>>::block_number() + 1u32.into();
//		let mut counter = 0usize;
//		let end = EstateModule::<T>::round().first + (round_length * min_block_per_round.into());

	}: {
		EstateModule::<T>::on_initialize(6u32.into());
	}
}

impl_benchmark_test_suite!(Pallet, crate::benchmarking::tests::new_test_ext(), crate::mock::Test);
