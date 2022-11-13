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

//! Benchmarks for the nft module.

#![cfg(feature = "runtime-benchmarks")]
use crate::Call;
#[allow(unused)]
use crate::Pallet as NftModule;
pub use crate::*;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::Get;
use frame_system::RawOrigin;
use orml_traits::BasicCurrencyExtended;
use primitive_traits::CollectionType;
use primitives::{AssetId, Balance, ClassId};
//use core_primitives::NFTTrait;
use scale_info::Type;
use sp_runtime::traits::{AccountIdConversion, StaticLookup, UniqueSaturatedInto};
use sp_runtime::Perbill;
use sp_std::prelude::*;
use sp_std::vec;

pub struct Pallet<T: Config>(crate::Pallet<T>);

const SEED: u32 = 0;
const ASSET_0: AssetId = 0;
const ASSET_1: AssetId = 1;

fn dollar(d: u32) -> Balance {
	let d: Balance = d.into();
	d.saturating_mul(1_000_000_000_000_000_000)
}

fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	T::Currency::make_free_balance_be(&caller, dollar(100).unique_saturated_into());
	caller
}

fn get_class_fund<T: Config>(class_id: ClassId) -> T::AccountId {
	T::PalletId::get().into_sub_account_truncating(class_id)
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
	}: _(RawOrigin::Signed(caller), vec![1], test_attributes(1), 0u32.into(), TokenType::Transferable, CollectionType::Collectable, Perbill::from_percent(22u32), None)

	mint{
		let caller = funded_account::<T>("caller", 0);
		let initial_balance = dollar(1000);

		<T as pallet::Config>::Currency::make_free_balance_be(&caller, initial_balance.unique_saturated_into());
		crate::Pallet::<T>::create_group(RawOrigin::Root.into(), vec![1], vec![1]);
		crate::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), vec![1], test_attributes(1), 0u32.into(), TokenType::Transferable, CollectionType::Collectable, Perbill::from_percent(0u32),None);
	}: _(RawOrigin::Signed(caller), 0u32.into(), vec![1], test_attributes(1), 3 )

	transfer{
		let caller = funded_account::<T>("caller", 0);
		let target = funded_account::<T>("target", 0);
		let initial_balance = dollar(1000);

		<T as pallet::Config>::Currency::make_free_balance_be(&caller, initial_balance.unique_saturated_into());
		crate::Pallet::<T>::create_group(RawOrigin::Root.into(), vec![1], vec![1]);
		crate::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), vec![1], test_attributes(1), 0u32.into(), TokenType::Transferable, CollectionType::Collectable, Perbill::from_percent(0u32),None);
		crate::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), 0u32.into(), vec![1], test_attributes(1), 3);
	}: _(RawOrigin::Signed(caller), target.clone(), (0u32.into(), 0u32.into()))

	transfer_batch{
		let caller = funded_account::<T>("caller", 0);
		let target1 = funded_account::<T>("target1", 0);
		let target2 = funded_account::<T>("target2", 0);
		let initial_balance = dollar(1000);

		<T as pallet::Config>::Currency::make_free_balance_be(&caller, initial_balance.unique_saturated_into());
		crate::Pallet::<T>::create_group(RawOrigin::Root.into(), vec![1], vec![1]);
		crate::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), vec![1], test_attributes(1), 0u32.into(), TokenType::Transferable, CollectionType::Collectable, Perbill::from_percent(0u32),None);
		crate::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), 0u32.into(), vec![1], test_attributes(1), 3);
		crate::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), 0u32.into(), vec![1], test_attributes(1), 3);
	}: _(RawOrigin::Signed(caller), vec![(target1.clone(), (0u32.into(), 0u32.into())), (target2.clone(), (0u32.into(), 1u32.into()))] )

	sign_asset{
		let caller = funded_account::<T>("caller", 0);
		let signer = funded_account::<T>("target", 0);
		let initial_balance = dollar(1000);

		<T as pallet::Config>::Currency::make_free_balance_be(&caller, initial_balance.unique_saturated_into());
		crate::Pallet::<T>::create_group(RawOrigin::Root.into(), vec![1], vec![1]);
		crate::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), vec![1], test_attributes(1), 0u32.into(), TokenType::Transferable, CollectionType::Collectable, Perbill::from_percent(0u32), None);
		crate::Pallet::<T>::mint(RawOrigin::Signed(caller.clone()).into(), 0u32.into(), vec![1], test_attributes(1), 3);
	}: _(RawOrigin::Signed(signer), (0u32.into(), 0u32.into()), 100u32.into() )
	set_hard_limit{
		let signer = funded_account::<T>("target", 0);
		crate::Pallet::<T>::create_group(RawOrigin::Root.into(), vec![1], vec![1]);
		crate::Pallet::<T>::create_class(RawOrigin::Signed(signer.clone()).into(), vec![1], test_attributes(1), 0u32.into(), TokenType::Transferable, CollectionType::Collectable, Perbill::from_percent(0u32), None);
	}:	_(RawOrigin::Signed(signer), 0u32.into(), 10u32)
	withdraw_funds_from_class_fund{
		let caller = funded_account::<T>("caller", 0);
		crate::Pallet::<T>::create_group(RawOrigin::Root.into(), vec![1], vec![1]);
		crate::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), vec![1], test_attributes(1), 0u32.into(), TokenType::Transferable, CollectionType::Collectable, Perbill::from_percent(0u32), None);
		let class_fund = get_class_fund::<T>(0u32.into());
		T::Currency::make_free_balance_be(&class_fund, dollar(100).unique_saturated_into());
	}: _(RawOrigin::Signed(caller), 0u32.into())
	force_update_total_issuance{
		let caller = funded_account::<T>("caller", 0);
		crate::Pallet::<T>::create_group(RawOrigin::Root.into(), vec![1], vec![1]);
		crate::Pallet::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), vec![1], test_attributes(1), 0u32.into(), TokenType::Transferable, CollectionType::Collectable, Perbill::from_percent(0u32), None);
	}: _(RawOrigin::Root, 0u32.into(), 0u32.into(), 1u32.into())
}

impl_benchmark_test_suite!(Pallet, crate::benchmarking::tests::new_test_ext(), crate::mock::Test);
