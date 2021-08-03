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

use frame_benchmarking::{account, whitelisted_caller, benchmarks, impl_benchmark_test_suite};
use frame_support::traits::Get;
use frame_system::RawOrigin;
use sp_runtime::traits::{AccountIdConversion, StaticLookup, UniqueSaturatedInto};

pub use crate::Pallet as NFTModule;
pub use crate::*;
use orml_traits::BasicCurrencyExtended;
use primitives::Balance;

pub struct Pallet<T: Config>(crate::Pallet<T>);

pub trait Config: crate::Config + orml_nft::Config + social_currencies::Config {}

const SEED: u32 = 0;

fn dollar(d: u32) -> Balance {
    let d: Balance = d.into();
    d.saturating_mul(1_000_000_000_000_000_000)
}

benchmarks! {
	// create NFT group
    create_group{
    }: _(RawOrigin::Root ,vec![1], vec![1] )

    create_class{
        let caller = whitelisted_caller();
        let initial_balance = dollar(1000);
        <T as social_currencies::Config>::NativeCurrency::update_balance(&caller, initial_balance.unique_saturated_into())?;
        crate::Pallet::<T>::create_group(RawOrigin::Root.into(), vec![1],vec![1]);
    }: _(RawOrigin::Signed(caller),vec![1], 0u32.into(), TokenType::Transferable, CollectionType::Collectable)
}

