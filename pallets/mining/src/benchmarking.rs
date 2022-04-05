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

//! Benchmarks for the mining module.

#![cfg(feature = "runtime-benchmarks")]
use core_primitives::MiningResourceRateInfo;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;
use orml_traits::BasicCurrencyExtended;
use primitives::{Balance, BlockNumber, staking::RoundInfo};
use sp_runtime::traits::{AccountIdConversion, StaticLookup, UniqueSaturatedInto};
use sp_std::prelude::*;
use sp_std::vec::Vec;

#[allow(unused)]
pub use crate::Pallet as MiningModule;
use crate::*;
use super::*;


pub type AccountId = u128;

const SEED: u32 = 0;
const BALANCE: u128 = 100;
const BLOCK_LENGTH: BlockNumber = 100;
const MINING_RESOURCE_RATE_INFO: MiningResourceRateInfo = MiningResourceRateInfo {
	ratio: 10,
	staking_reward: 3000,
	mining_reward: 7000
};

fn dollar(d: u32) -> Balance {
	let d: Balance = d.into();
	d.saturating_mul(1_000_000_000_000_000_000)
}
/* 
fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	let initial_balance = dollar(1000);

	T::Currency::make_free_balance_be(&caller, initial_balance.unique_saturated_into());
	caller
}
*/

benchmarks! {
    // add minting origin
    add_minting_origin {
		let who: T::AccountId = account("target", 0, SEED);
	}: _(RawOrigin::Root, who.clone()) 
	verify {
		assert_eq!(crate::Pallet::<T>::ensure_admin(RawOrigin::Root.into()), Ok(()));
		assert_eq!(crate::Pallet::<T>::is_mining_origin(&who.clone()), true);
	}

	// remove minting origin
    remove_minting_origin {
		let who: T::AccountId = account("target", 0, SEED);

		crate::Pallet::<T>::add_minting_origin(RawOrigin::Root.into(), who.clone());
	}: _(RawOrigin::Root, who.clone())
	verify {
		assert_eq!(crate::Pallet::<T>::ensure_admin(RawOrigin::Root.into()), Ok(()));
		assert_eq!(crate::Pallet::<T>::is_mining_origin(&who.clone()), false);
	}

	// update round length
    update_round_length {
	}: _(RawOrigin::Root, BLOCK_LENGTH.into())
	verify {
		let current_round = crate::Pallet::<T>::round();
		assert_eq!(current_round.length, BLOCK_LENGTH);
	}
	
	// update minting issuance config
	update_mining_issuance_config {
	}: _(RawOrigin::Root, MINING_RESOURCE_RATE_INFO)
	verify {
		assert_eq!(crate::Pallet::<T>::ensure_admin(RawOrigin::Root.into()), Ok(()));
		let mining_resource_rate_info = crate::Pallet::<T>::mining_ratio_config();
		assert_eq!(mining_resource_rate_info.ratio, MINING_RESOURCE_RATE_INFO.ratio);
		assert_eq!(mining_resource_rate_info.staking_reward, MINING_RESOURCE_RATE_INFO.staking_reward);
		assert_eq!(mining_resource_rate_info.mining_reward, MINING_RESOURCE_RATE_INFO.mining_reward);
	}
	
	// mint 
	mint {
		let origin: T::AccountId = whitelisted_caller();
		let who: T::AccountId = account("target", 0, SEED);

		crate::Pallet::<T>::add_minting_origin(RawOrigin::Root.into(), origin.clone());
	}: _(RawOrigin::Signed(origin.clone()), who.clone(), BALANCE) 
	//verify {
		// TODO: verify correct behavior
		//assert_eq!(crate::Pallet::<T>::is_mining_origin(origin.clone()), true);
	//}

	// burn
	burn {
		let origin: T::AccountId = whitelisted_caller();
		let who: T::AccountId = account("target", 0, SEED);

		crate::Pallet::<T>::add_minting_origin(RawOrigin::Root.into(), origin.clone());
		crate::Pallet::<T>::mint(RawOrigin::Signed(origin.clone()).into(), who.clone(), BALANCE); 
	}: _(RawOrigin::Signed(origin.clone()), who.clone(), BALANCE) 
	//verify {
		// TODO: verify correct behavior
	//}

	/*// deposit 
	deposit {
		let origin: T::AccountId = whitelisted_caller();

		crate::Pallet::<T>::add_minting_origin(RawOrigin::Root.into(), origin.clone());
	}: _(RawOrigin::Signed(origin.clone()), BALANCE) 
	//verify {
		// TODO: verify correct behavior
	//}

	// withdraw
	withdraw {
		let origin: T::AccountId = account("origin", 0, SEED);
		let dest: T::AccountId = account("dest", 1, SEED);

		crate::Pallet::<T>::add_minting_origin(RawOrigin::Root.into(), origin.clone());
		crate::Pallet::<T>::deposit(RawOrigin::Signed(origin.clone()).into(), BALANCE);
	}: _(RawOrigin::Signed(origin.clone()), dest.clone(), BALANCE) 
	//verify {
		// TODO: verify correct behavior
		// assert_eq!(crate::Pallet::<T>::is_mining_origin(origin.clone()), true);
	//}
		*/
}
impl_benchmark_test_suite!(Pallet, crate::benchmarking::tests::new_test_ext(), crate::mock::Test);
