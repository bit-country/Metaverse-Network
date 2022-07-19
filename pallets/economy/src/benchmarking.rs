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

//! Benchmarks for the economy module.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use sp_std::prelude::*;
use sp_std::vec;

#[allow(unused)]
pub use crate::Pallet as EconomyModule;
use crate::{Call, Config};
// use crate::Mining as MiningModule;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::{Currency, Get};
use frame_system::{Pallet as System, RawOrigin};
use pallet_mining::Pallet as MiningModule;
use primitives::{Balance, GroupCollectionId};
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
const BENEFICIARY_NFT: (ClassId, TokenId) = (1, 0);

const ELEMENT_INDEX_ID: ElementId = 1;

const STAKING_AMOUNT: Balance = 1000;
const UNSTAKING_AMOUNT: Balance = 100;

const COLLECTION_ID: GroupCollectionId = 0;
const CLASS_ID: ClassId = 0;

fn dollar(d: u32) -> Balance {
	let d: Balance = d.into();
	d.saturating_mul(1_000_000_000_000_000_000)
}

fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	T::Currency::make_free_balance_be(&caller, dollar(10000).unique_saturated_into());
	caller
}

fn run_to_block<T: Config>(n: u64) {
	// while System::<T>::block_number() < n.into() {
	// 	System::<T>::on_finalize(System::<T>::block_number());
	System::<T>::set_block_number(25u32.into());
	// 	System::<T>::on_initialize(System::<T>::block_number());
	// 	// Mining::on_initialize(System::block_number());
	// MiningModule::<T>::on_initialize(25u32.into());
	// }
}

// fn assert_last_event(generic_event: Event) {
// 	System::assert_last_event(generic_event.into());
// }

benchmarks! {
	// set_bit_power_exchange_rate
	set_bit_power_exchange_rate{
		let caller = funded_account::<T>("caller", 0);
	}: _(RawOrigin::Root, EXCHANGE_RATE)
	verify {
		let new_rate = crate::Pallet::<T>::get_bit_power_exchange_rate();
		assert_eq!(new_rate, EXCHANGE_RATE);
	}

	// set_power_balance
	set_power_balance{
		let caller = funded_account::<T>("caller", 0);
	}: _(RawOrigin::Root, BENEFICIARY_NFT, 123)
	verify {
		// let account_id = crate::Pallet::<T>::EconomyTreasury::get().into_sub_account_truncating(BENEFICIARY_NFT);
		let account_id: T::AccountId = <<T as Config>::EconomyTreasury as Get<PalletId>>::get().into_sub_account_truncating(BENEFICIARY_NFT);

		let new_balance = crate::Pallet::<T>::get_power_balance(account_id);
		assert_eq!(new_balance, 123);
	}

	// stake
	stake{
		let caller = funded_account::<T>("caller", 0);

		let min_stake = <<T as Config>::MinimumStake as Get<BalanceOf<T>>>::get();
		let stake_amount = min_stake + 1u32.into();

	}: _(RawOrigin::Signed(caller.clone()), stake_amount)
	verify {
		let staking_balance = crate::Pallet::<T>::get_staking_info(caller.clone());
		assert_eq!(staking_balance, stake_amount);
	}

	// unstake
	unstake{
		let caller = funded_account::<T>("caller", 0);

		let min_stake = <<T as Config>::MinimumStake as Get<BalanceOf<T>>>::get();
		let stake_amount = min_stake + 100u32.into();

		crate::Pallet::<T>::stake(RawOrigin::Signed(caller.clone()).into(), stake_amount);

		let current_round = T::RoundHandler::get_current_round_info();
		let next_round = current_round.current.saturating_add(One::one());
	}: _(RawOrigin::Signed(caller.clone()), 10u32.into())
	verify {
		let staking_balance = crate::Pallet::<T>::get_staking_info(caller.clone());
		assert_eq!(staking_balance, min_stake + 90u32.into());

		assert_eq!(
			crate::Pallet::<T>::staking_exit_queue(caller.clone(), next_round),
			Some(10u32.into())
		);
	}

	// withdraw_unreserved
	withdraw_unreserved{
		let caller = funded_account::<T>("caller", 0);

		let min_stake = <<T as Config>::MinimumStake as Get<BalanceOf<T>>>::get();
		let stake_amount = min_stake + 100u32.into();

		let current_round = T::RoundHandler::get_current_round_info();
		let next_round = current_round.current.saturating_add(One::one());

		crate::Pallet::<T>::stake(RawOrigin::Signed(caller.clone()).into(), stake_amount);
		crate::Pallet::<T>::unstake(RawOrigin::Signed(caller.clone()).into(), 10u32.into());

		run_to_block::<T>(100);
		let next_round = current_round.current.saturating_add(One::one());
	}: _(RawOrigin::Signed(caller.clone()), next_round)
	verify {
		assert_eq!(
			crate::Pallet::<T>::staking_exit_queue(caller.clone(), next_round),
			None
		);
	}
}

impl_benchmark_test_suite!(Pallet, crate::benchmarking::tests::new_test_ext(), crate::mock::Test);
