// This file is part of Metaverse.Network & Bit.Country.

// Copyright (C) 2020-2022 Metaverse.Network & Bit.Country .
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::weights::Weight;
use scale_info::TypeInfo;
use sp_runtime::traits::Convert;
use sp_std::{collections::btree_map::BTreeMap, marker::PhantomData, prelude::*};

pub const RATIO: u64 = 9000;

pub mod precompile;
//mod tests;
//mod weights;

/// Convert gas to weight
pub struct GasToWeight;
impl Convert<u64, Weight> for GasToWeight {
	fn convert(gas: u64) -> Weight {
		gas.saturating_mul(RATIO)
	}
}

/// Convert weight to gas
pub struct WeightToGas;
impl Convert<Weight, u64> for WeightToGas {
	fn convert(weight: Weight) -> u64 {
		weight
			.checked_div(RATIO)
			.expect("Compile-time constant is not zero; qed;")
	}
}
