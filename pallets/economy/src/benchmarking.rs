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
// use primitives::Balance;
use crate::{Call, Config};
use pallet_mining::Token;
use pallet_nft::Pallet as NFTModule;
use pallet_nft::{Attributes, CollectionType, TokenType};
use sp_runtime::traits::{AccountIdConversion, StaticLookup, UniqueSaturatedInto};

pub type AccountId = u128;
pub type Balance = u128;

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

fn dollar(d: u32) -> Balance {
	let d: Balance = d.into();
	d.saturating_mul(1_000_000_000_000_000_000)
}

fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	T::Currency::make_free_balance_be(&caller, dollar(10000).unique_saturated_into());
	caller
}

fn mint_NFT<T: Config>(caller: T::AccountId) {
	// NFTModule::<T>::create_group(RawOrigin::Root.into(), vec![1], vec![1]);
	// NFTModule::<T>::create_class(
	// 	RawOrigin::Signed(caller.clone()).into(),
	// 	vec![1],
	// 	test_attributes(1),
	// 	0u32.into(),
	// 	TokenType::Transferable,
	// 	CollectionType::Collectable,
	// );
	// NFTModule::<T>::mint(
	// 	RawOrigin::Signed(caller.clone()).into(),
	// 	0u32.into(),
	// 	vec![1],
	// 	test_attributes(1),
	// 	3,
	// );
}

fn test_attributes(x: u8) -> Attributes {
	let mut attr: Attributes = BTreeMap::new();
	attr.insert(vec![x, x + 5], vec![x, x + 10]);
	attr
}

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
		// let account_id = crate::Pallet::<T>::EconomyTreasury::get().into_sub_account(BENEFICIARY_NFT);
		let account_id: T::AccountId = <<T as Config>::EconomyTreasury as Get<PalletId>>::get().into_sub_account(BENEFICIARY_NFT);

		let new_balance = crate::Pallet::<T>::get_power_balance(account_id);
		assert_eq!(new_balance, 123);
	}

	// mint_element
	mint_element{
		let caller = funded_account::<T>("caller", 0);

		// ElementIndex::<Runtime>::insert(
		// 	ELEMENT_INDEX_ID,
		// 	ElementInfo {
		// 		power_price: 10,
		// 		compositions: vec![],
		// 	},
		// );

		// 	crate::Pallet::<T>::create_metaverse(RawOrigin::Root.into(), caller.clone(), vec![1]);
	}: _(RawOrigin::Signed(caller.clone()), 1, 2)
	verify {
		// let account_id = crate::Pallet::<T>::EconomyTreasury::get().into_sub_account(BENEFICIARY_NFT);
		// let account_id: T::AccountId = <<T as Config>::EconomyTreasury as Get<PalletId>>::get().into_sub_account(BENEFICIARY_NFT);
		//
		// let new_balance = crate::Pallet::<T>::get_power_balance(account_id);
		// assert_eq!(new_balance, 123);
	}

	// stake
	stake{
		let caller = funded_account::<T>("caller", 0);
	}: _(RawOrigin::Signed(caller.clone()), 1000u32.into())
	verify {
		let staking_balance = crate::Pallet::<T>::get_staking_info(caller.clone());
		assert_eq!(staking_balance, 1000u32.into());
	}

	// unstake
	unstake{
		let caller = funded_account::<T>("caller", 0);

		crate::Pallet::<T>::stake(RawOrigin::Signed(caller.clone()).into(), 1000u32.into());

	}: _(RawOrigin::Signed(caller.clone()), 100u32.into())
	verify {
		let staking_balance = crate::Pallet::<T>::get_staking_info(caller.clone());
		assert_eq!(staking_balance, 900u32.into());
	}



	// // authorize_power_generator_collection
	// authorize_power_generator_collection{
	// 	let caller = funded_account::<T>("caller", 0);
	// }: _(RawOrigin::Root, BENEFICIARY_NFT, 123)
	// verify {
	// 	// let account_id = crate::Pallet::<T>::EconomyTreasury::get().into_sub_account(BENEFICIARY_NFT);
	// 	let account_id: T::AccountId = <<T as Config>::EconomyTreasury as Get<PalletId>>::get().into_sub_account(BENEFICIARY_NFT);
	//
	// 	let new_balance = crate::Pallet::<T>::get_power_balance(account_id);
	// 	assert_eq!(new_balance, 123);
	// }

	// // transfer_metaverse
	// transfer_metaverse {
	// 	let caller = funded_account::<T>("caller", 0);
	// 	let target = funded_account::<T>("target", 0);
	//
	// 	crate::Pallet::<T>::create_metaverse(RawOrigin::Root.into(), caller.clone(), vec![1]);
	// }: _(RawOrigin::Signed(caller.clone()), target.clone(), 0)
	// verify {
	// 	let metaverse = crate::Pallet::<T>::get_metaverse(0);
	// 	match metaverse {
	// 		Some(a) => {
	// 			assert_eq!(a.owner, target.clone());
	// 			assert_eq!(a.is_frozen, false);
	// 			assert_eq!(a.metadata, vec![1]);
	// 		}
	// 		_ => {
	// 			// Should fail test
	// 			assert_eq!(0, 1);
	// 		}
	// 	}
	// }
	//
	// // freeze_metaverse
	// freeze_metaverse{
	// 	let caller = funded_account::<T>("caller", 0);
	// 	let target = funded_account::<T>("target", 0);
	//
	// 	crate::Pallet::<T>::create_metaverse(RawOrigin::Root.into(), caller.clone(), vec![1]);
	// }: _(RawOrigin::Root, 0)
	// verify {
	// 	let metaverse = crate::Pallet::<T>::get_metaverse(0);
	// 	match metaverse {
	// 		Some(a) => {
	// 			assert_eq!(a.is_frozen, true);
	// 		}
	// 		_ => {
	// 			// Should fail test
	// 			assert_eq!(0, 1);
	// 		}
	// 	}
	// }
	//
	// // unfreeze_metaverse
	// unfreeze_metaverse{
	// 	let caller = funded_account::<T>("caller", 0);
	// 	let target = funded_account::<T>("target", 0);
	//
	// 	crate::Pallet::<T>::create_metaverse(RawOrigin::Root.into(), caller.clone(), vec![1]);
	// 	crate::Pallet::<T>::freeze_metaverse(RawOrigin::Root.into(), 0);
	// }: _(RawOrigin::Root, 0)
	// verify {
	// 	let metaverse = crate::Pallet::<T>::get_metaverse(0);
	// 	match metaverse {
	// 		Some(a) => {
	// 			// Verify details of Metaverse
	// 			assert_eq!(a.is_frozen, false);
	// 		}
	// 		_ => {
	// 			// Should fail test
	// 			assert_eq!(0, 1);
	// 		}
	// 	}
	// }
	//
	// // destroy_metaverse
	// destroy_metaverse{
	// 	let caller = funded_account::<T>("caller", 0);
	// 	let target = funded_account::<T>("target", 0);
	//
	// 	crate::Pallet::<T>::create_metaverse(RawOrigin::Root.into(), caller.clone(), vec![1]);
	// 	crate::Pallet::<T>::freeze_metaverse(RawOrigin::Root.into(), 0);
	// }: _(RawOrigin::Root, 0)
	// verify {
	// 	assert_eq!(crate::Pallet::<T>::get_metaverse(0), None);
	// }
	//
	// // register_metaverse
	// register_metaverse{
	// 	let caller = funded_account::<T>("caller", 0);
	// 	let target = funded_account::<T>("target", 0);
	//
	// 	crate::Pallet::<T>::create_metaverse(RawOrigin::Root.into(), caller.clone(), vec![1]);
	// }: _(RawOrigin::Signed(caller.clone()), 0)
	// verify {
	// 	let metaverse = crate::Pallet::<T>::get_registered_metaverse(0);
	// 	match metaverse {
	// 		Some(a) => {
	// 			assert_eq!(1, 1);
	// 		}
	// 		_ => {
	// 			// Should fail test
	// 			assert_eq!(0, 1);
	// 		}
	// 	}
	// }
	//
	// // stake
	// stake{
	// 	let caller = funded_account::<T>("caller", 0);
	// 	let target = funded_account::<T>("target", 0);
	// 	let amount = <<T as Config>::MinStakingAmount as Get<BalanceOf<T>>>::get();
	//
	// 	crate::Pallet::<T>::create_metaverse(RawOrigin::Root.into(), caller.clone(), vec![1]);
	// 	crate::Pallet::<T>::register_metaverse(RawOrigin::Signed(caller.clone()).into(), 0);
	// }: _(RawOrigin::Signed(caller.clone()), 0, (amount+1u32.into()).into())
	// verify {
	// 	let staking_info = crate::Pallet::<T>::staking_info(caller);
	// 	assert_eq!(staking_info, (amount+1u32.into()).into());
	// }
	//
	// // unstake_and_withdraw
	// unstake_and_withdraw{
	// 	let caller = funded_account::<T>("caller", 0);
	// 	let target = funded_account::<T>("target", 0);
	//
	// 	let amount = <<T as Config>::MinStakingAmount as Get<BalanceOf<T>>>::get();
	//
	//
	// 	crate::Pallet::<T>::create_metaverse(RawOrigin::Root.into(), caller.clone(), vec![1]);
	// 	crate::Pallet::<T>::register_metaverse(RawOrigin::Signed(caller.clone()).into(), 0);
	// 	crate::Pallet::<T>::stake(RawOrigin::Signed(caller.clone()).into(), 0, (amount+1u32.into()).into());
	// }: _(RawOrigin::Signed(caller.clone()), 0, 1u32.into())
	// verify {
	// 	let staking_info = crate::Pallet::<T>::staking_info(caller);
	// 	assert_eq!(staking_info, amount.into());
	// }
}

impl_benchmark_test_suite!(Pallet, crate::benchmarking::tests::new_test_ext(), crate::mock::Test);
