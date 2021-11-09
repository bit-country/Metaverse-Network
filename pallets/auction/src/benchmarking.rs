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
use crate::{Call, Config};
use auction_manager::ListingLevel;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;
use primitives::Balance;
use sp_runtime::traits::{AccountIdConversion, StaticLookup, UniqueSaturatedInto};

const SEED: u32 = 0;

const METAVERSE_ID: u64 = 1;

const MAX_BOUND: (i32, i32) = (-100, 100);

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
	// set_max_bounds
// 	set_max_bounds{
// 	}: _(RawOrigin::Root,METAVERSE_ID, MAX_BOUND)
// 	verify {
// 		assert_eq!(crate::Pallet::<T>::get_max_bounds(METAVERSE_ID), MAX_BOUND)
// 	}
//
// 	active_issue_undeploy_land_block{
// 		// INITIALIZE RUNTIME STATE
// 		let minting_info = 	MintingRateInfo {
// 			expect: Default::default(),
// 			// 10% minting rate per annual
// 			annual: 10,
// 			// Max 100 millions land unit
// 			max: 100_000_000,
// 		};
// 		// Pre issue 5 land blocks x 100 land units
// 		issue_new_undeployed_land_block::<T>(5)?;
// 		let min_block_per_round = 5u32;
//
// 		let new_round = RoundInfo::new(1u32, 0u32.into(), min_block_per_round.into());
//
// 		Round::<T>::put(new_round);
// 		let high_inflation_rate = MintingRateInfo {
// 			expect: Default::default(),
// 			annual: 20,
// 			// Max 100 millions land unit
// 			max: 100_000_000,
// 		};
// 		MintingRateConfig::<T>::put(high_inflation_rate);
//
// //
// //		// PREPARE RUN_TO_BLOCK LOOP
// //		let before_running_round_index = EstateModule::<T>::round().current;
// //		let round_length: T::BlockNumber = EstateModule::<T>::round().length.into();
// //
// //
// //		let mut now = <frame_system::Pallet<T>>::block_number() + 1u32.into();
// //		let mut counter = 0usize;
// //		let end = EstateModule::<T>::round().first + (round_length * min_block_per_round.into());
//
// 	}: {
// 		EstateModule::<T>::on_initialize(6u32.into());
// 	}

	//create_new_auction
	create_new_auction{
		let caller = funded_account::<T>("caller", 0);

		// TODO: need to mint a new NFT first
	}: _(RawOrigin::Signed(caller.clone()), ItemId::NFT(0), 100.into(), 0, ListingLevel::Global)

}

impl_benchmark_test_suite!(Pallet, crate::benchmarking::tests::new_test_ext(), crate::mock::Test);
