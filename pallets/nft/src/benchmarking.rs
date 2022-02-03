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

//! Benchmarks for the nft module.

#![cfg(feature = "runtime-benchmarks")]

use sp_std::prelude::*;
use sp_std::vec;

use crate::Call;
#[allow(unused)]
use crate::Pallet as NftModule;
pub use crate::*;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::Get;
use frame_system::RawOrigin;
use orml_traits::BasicCurrencyExtended;
use primitives::Balance;
use sp_runtime::traits::{AccountIdConversion, StaticLookup, UniqueSaturatedInto};

pub struct Pallet<T: Config>(crate::Pallet<T>);

const SEED: u32 = 0;

fn dollar(d: u32) -> Balance {
	let d: Balance = d.into();
	d.saturating_mul(1_000_000_000_000_000_000)
}

fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	T::Currency::make_free_balance_be(&caller, dollar(100).unique_saturated_into());
	caller
}

fn test_attributes(x: u8) -> Attributes {
	let mut attr: Attributes = BTreeMap::new();
	attr.insert(vec![x, x + 5], vec![x, x + 10]);
	attr
}

benchmarks! {

	// create NFT group
	create_group{
	}: _(RawOrigin::Root ,vec![1], vec![1] )

	create_class{
		let caller = whitelisted_caller();
		let initial_balance = dollar(1000);

		<T as pallet::Config>::Currency::make_free_balance_be(&caller, initial_balance.unique_saturated_into());

		crate::Pallet::<T>::create_group(RawOrigin::Root.into(), vec![1], vec![1]);
	}: _(RawOrigin::Signed(caller), vec![1], test_attributes(1), 0u32.into(), TokenType::Transferable, CollectionType::Collectable)

	mint{
		let caller = funded_account::<T>("caller", 0);
		let initial_balance = dollar(1000);

		<T as pallet::Config>::Currency::make_free_balance_be(&caller, initial_balance.unique_saturated_into());
		crate::Pallet::<T>::create_group(RawOrigin::Root.into(), vec![1], vec![1]);
		crate::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), vec![1], test_attributes(1), 0u32.into(), TokenType::Transferable, CollectionType::Collectable);
	}: _(RawOrigin::Signed(caller), 0u32.into(), vec![1], test_attributes(1), 3 )

	transfer{
		let caller = funded_account::<T>("caller", 0);
		let target = funded_account::<T>("target", 0);
		let initial_balance = dollar(1000);

		<T as pallet::Config>::Currency::make_free_balance_be(&caller, initial_balance.unique_saturated_into());
		crate::Pallet::<T>::create_group(RawOrigin::Root.into(), vec![1], vec![1]);
		crate::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), vec![1], test_attributes(1), 0u32.into(), TokenType::Transferable, CollectionType::Collectable);
		crate::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), 0u32.into(), vec![1], test_attributes(1), 3);
	}: _(RawOrigin::Signed(caller), target.clone(),  0u32.into() )

	transfer_batch{
		let caller = funded_account::<T>("caller", 0);
		let target1 = funded_account::<T>("target1", 0);
		let target2 = funded_account::<T>("target2", 0);
		let initial_balance = dollar(1000);

		<T as pallet::Config>::Currency::make_free_balance_be(&caller, initial_balance.unique_saturated_into());
		crate::Pallet::<T>::create_group(RawOrigin::Root.into(), vec![1], vec![1]);
		crate::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), vec![1], test_attributes(1), 0u32.into(), TokenType::Transferable, CollectionType::Collectable);
		crate::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), 0u32.into(), vec![1], test_attributes(1), 3);
		crate::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), 0u32.into(), vec![1], test_attributes(1), 3);
	}: _(RawOrigin::Signed(caller), vec![(target1.clone(), 0u32.into()), (target2.clone(), 1u32.into())] )

	sign_asset{
		let caller = funded_account::<T>("caller", 0);
		let signer = funded_account::<T>("target", 0);
		let initial_balance = dollar(1000);

		<T as pallet::Config>::Currency::make_free_balance_be(&caller, initial_balance.unique_saturated_into());
		crate::Pallet::<T>::create_group(RawOrigin::Root.into(), vec![1], vec![1]);
		crate::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), vec![1], test_attributes(1), 0u32.into(), TokenType::Transferable, CollectionType::Collectable);
		crate::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), 0u32.into(), vec![1], test_attributes(1), 3);
	}: _(RawOrigin::Signed(signer), 0u32.into(), 100u32.into() )

}

impl_benchmark_test_suite!(Pallet, crate::benchmarking::tests::new_test_ext(), crate::mock::Test);
