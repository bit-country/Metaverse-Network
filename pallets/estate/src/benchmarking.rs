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
use primitives::staking::RoundInfo;
use sp_std::prelude::*;
use sp_std::vec;

#[allow(unused)]
pub use crate::Pallet as EstateModule;
// use crate::{
// 	pallet::{MintingRateConfig, Round},
// 	Call, Config, MintingRateInfo, Range,
// };

use crate::*;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;
use primitives::estate::{EstateInfo, OwnerId};
use primitives::Balance;
use sp_runtime::traits::{AccountIdConversion, StaticLookup, UniqueSaturatedInto};
// use orml_traits::BasicCurrencyExtended;
use primitives::UndeployedLandBlockType;

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
const ESTATE_ID: EstateId = 0;

fn dollar(d: u32) -> Balance {
	let d: Balance = d.into();
	d.saturating_mul(1_000_000_000_000_000_000)
}

fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	T::Currency::make_free_balance_be(&caller, dollar(1000000).unique_saturated_into());
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

fn get_estate_info(lands: Vec<(i32, i32)>) -> EstateInfo {
	return EstateInfo {
		metaverse_id: METAVERSE_ID,
		land_units: lands,
	};
}

benchmarks! {
	// set_max_bounds
	set_max_bounds{
	}: _(RawOrigin::Root, METAVERSE_ID, MAX_BOUND)
	verify {
		assert_eq!(crate::Pallet::<T>::get_max_bounds(METAVERSE_ID), MAX_BOUND)
	}

	// mint_land as non-token
	mint_land_n {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		crate::Pallet::<T>::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
	}: mint_land(RawOrigin::Root, caller.clone(), METAVERSE_ID, COORDINATE_IN_1, false)
	verify {
		assert_eq!(crate::Pallet::<T>::get_land_units(METAVERSE_ID, COORDINATE_IN_1), Some(OwnerId::Account(caller.clone())));
	}

	// mint_land as tokens
	mint_land_t {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		crate::Pallet::<T>::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
	}: mint_land(RawOrigin::Root, caller.clone(), METAVERSE_ID, COORDINATE_IN_1, true)
	verify {
		assert_eq!(crate::Pallet::<T>::get_land_units(METAVERSE_ID, COORDINATE_IN_1), Some(OwnerId::Token(0)));
	}

	// mint_lands as non-tokens
	mint_lands_n {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		crate::Pallet::<T>::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
	}: mint_lands(RawOrigin::Root, caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1, COORDINATE_IN_2], false)
	verify {
		assert_eq!(crate::Pallet::<T>::get_land_units(METAVERSE_ID, COORDINATE_IN_1), Some(OwnerId::Account(caller.clone())));
		assert_eq!(crate::Pallet::<T>::get_land_units(METAVERSE_ID, COORDINATE_IN_2), Some(OwnerId::Account(caller.clone())))
	}

	// mint_lands as tokens
	mint_lands_t {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		crate::Pallet::<T>::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
	}: mint_lands(RawOrigin::Root, caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1, COORDINATE_IN_2], true)
	verify {
		assert_eq!(crate::Pallet::<T>::get_land_units(METAVERSE_ID, COORDINATE_IN_1), Some(OwnerId::Token(0)));
		assert_eq!(crate::Pallet::<T>::get_land_units(METAVERSE_ID, COORDINATE_IN_2), Some(OwnerId::Token(1)))
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
		crate::Pallet::<T>::mint_land(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, COORDINATE_IN_1, false);

	}: _(RawOrigin::Signed(caller.clone()), target.clone(), METAVERSE_ID, COORDINATE_IN_1)
	verify {
		// TODO: issue with blow line
		// assert_eq!(crate::Pallet::<T>::get_land_units(METAVERSE_ID, COORDINATE_IN_1), target.clone())
	}

	// mint_estate as non-tokens
	mint_estate_n {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		crate::Pallet::<T>::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
	}: mint_estate(RawOrigin::Root, caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1], false)
	verify {
		assert_eq!(crate::Pallet::<T>::get_estates(0), Some(get_estate_info(vec![COORDINATE_IN_1])));
		assert_eq!(crate::Pallet::<T>::get_estate_owner(0), Some(OwnerId::Account(caller.clone())));
	}

	// mint_estate as tokens
	mint_estate_t {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		crate::Pallet::<T>::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
	}: mint_estate(RawOrigin::Root, caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1], true)
	verify {
		assert_eq!(crate::Pallet::<T>::get_estates(0), Some(get_estate_info(vec![COORDINATE_IN_1])));
		assert_eq!(crate::Pallet::<T>::get_estate_owner(0), Some(OwnerId::Token(0)));
		assert_eq!(crate::Pallet::<T>::get_land_units(METAVERSE_ID, COORDINATE_IN_1), Some(OwnerId::Token(1)));
	}

	// dissolve_estate
	dissolve_estate {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		crate::Pallet::<T>::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
		crate::Pallet::<T>::mint_estate(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1], false);
	}: _(RawOrigin::Signed(caller.clone()), 0)
	verify {
		assert_eq!(crate::Pallet::<T>::get_estates(0), None)
	}

	// add_land_unit_to_estate
	add_land_unit_to_estate {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		crate::Pallet::<T>::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
		crate::Pallet::<T>::mint_estate(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1], false);
		crate::Pallet::<T>::mint_land(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, COORDINATE_IN_2, false);
	}: _(RawOrigin::Signed(caller.clone()), 0, vec![COORDINATE_IN_2])
	verify {
		assert_eq!(crate::Pallet::<T>::get_estates(0), Some(get_estate_info(vec![COORDINATE_IN_1, COORDINATE_IN_2])))
	}

	// remove_land_unit_from_estate
	remove_land_unit_from_estate {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		crate::Pallet::<T>::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
		crate::Pallet::<T>::mint_estate(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1, COORDINATE_IN_2], false);
	}: _(RawOrigin::Signed(caller.clone()), 0, vec![COORDINATE_IN_2])
	verify {
		assert_eq!(crate::Pallet::<T>::get_estates(0), Some(get_estate_info(vec![COORDINATE_IN_1])))
	}

	// create_estate as non-token
	create_estate_n {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		crate::Pallet::<T>::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
		crate::Pallet::<T>::mint_lands(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1, COORDINATE_IN_2], false);

	}: create_estate(RawOrigin::Root, caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1, COORDINATE_IN_2], false)
	verify {
		assert_eq!(crate::Pallet::<T>::get_estates(0), Some(get_estate_info(vec![COORDINATE_IN_1, COORDINATE_IN_2])));
		assert_eq!(crate::Pallet::<T>::get_estate_owner(0), Some(OwnerId::Account(caller.clone())));
		//assert_eq!(crate::Pallet::<T>::get_land_units(METAVERSE_ID, COORDINATE_IN_1), Some(OwnerId::Account(caller.clone())));
		//assert_eq!(crate::Pallet::<T>::get_land_units(METAVERSE_ID, COORDINATE_IN_2), Some(OwnerId::Account(caller)))
	}

	// create_estate as token
	create_estate_t {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		crate::Pallet::<T>::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
		crate::Pallet::<T>::mint_lands(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1, COORDINATE_IN_2], true);

	}: create_estate(RawOrigin::Root, caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1, COORDINATE_IN_2], false)
	verify {
		assert_eq!(crate::Pallet::<T>::get_estates(0), Some(get_estate_info(vec![COORDINATE_IN_1, COORDINATE_IN_2])));
		assert_eq!(crate::Pallet::<T>::get_estate_owner(0), Some(OwnerId::Token(0)));
		//assert_eq!(crate::Pallet::<T>::get_land_units(METAVERSE_ID, COORDINATE_IN_1), Some(OwnerId::Account(caller.clone())));
		//assert_eq!(crate::Pallet::<T>::get_land_units(METAVERSE_ID, COORDINATE_IN_2), Some(OwnerId::Account(caller)))
	}

	// transfer_estate
	transfer_estate {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		let target: T::AccountId = account("target", 0, SEED);
		let target_lookup = T::Lookup::unlookup(target.clone());

		crate::Pallet::<T>::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
		crate::Pallet::<T>::mint_estate(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1, COORDINATE_IN_2],  false);

	}: _(RawOrigin::Signed(caller.clone()), target.clone(), 0)
	verify {
		assert_eq!(crate::Pallet::<T>::get_estate_owner(0), Some(OwnerId::Account(target.clone())))
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
	}: _(RawOrigin::Signed(caller.clone()), Default::default(), METAVERSE_ID, vec![COORDINATE_IN_1, COORDINATE_IN_2], false)
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

	// bond_more
	bond_more {
		let min_stake = T::MinimumStake::get();

		let caller = funded_account::<T>("caller", 10000);

		crate::Pallet::<T>::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
		crate::Pallet::<T>::mint_estate(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1], false);
	}: _(RawOrigin::Signed(caller.clone()), 0, T::MinimumStake::get())
	verify {
		assert_eq!(crate::Pallet::<T>::estate_stake(0, caller.clone()), T::MinimumStake::get())
	}

	// bond_less
	bond_less {
		let caller = funded_account::<T>("caller", 10000);

		let min_stake = T::MinimumStake::get();
		let bond_amount = min_stake + 1u32.into();

		crate::Pallet::<T>::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
		crate::Pallet::<T>::mint_estate(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1], false);
		crate::Pallet::<T>::bond_more(RawOrigin::Signed(caller.clone()).into(), 0, bond_amount);
	}: _(RawOrigin::Signed(caller.clone()), 0, 1u32.into())
	verify {
		assert_eq!(crate::Pallet::<T>::estate_stake(0, caller.clone()),  T::MinimumStake::get())
	}

	// leave_staking
	leave_staking {
		let caller = funded_account::<T>("caller", 10000);

		let min_stake = T::MinimumStake::get();
		let bond_amount = min_stake + 1u32.into();

		crate::Pallet::<T>::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
		crate::Pallet::<T>::mint_estate(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1], false);
		crate::Pallet::<T>::bond_more(RawOrigin::Signed(caller.clone()).into(), 0, bond_amount);
	}: _(RawOrigin::Signed(caller.clone()), 0)
	verify {
		assert_eq!(crate::Pallet::<T>::exit_queue(caller.clone(), 0), Some(()))
	}
}

impl_benchmark_test_suite!(Pallet, crate::benchmarking::tests::new_test_ext(), crate::mock::Test);
