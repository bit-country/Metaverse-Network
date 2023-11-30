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

//! Autogenerated weights for estate
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2021-11-03, STEPS: `20`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 128

// Executed Command:
// ./target/release/metaverse-node
// benchmark
// --chain=dev
// --pallet=estate
// --extrinsic=*
// --steps=20
// --repeat=10
// --execution=wasm
// --wasm-execution=compiled
// --template=./template/runtime-weight-template.hbs
// --output
// ./pallets/estate/src/weights.rs


#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for estate.
pub struct WeightInfo<T>(PhantomData<T>);

impl<T: frame_system::Config> spp::WeightInfo for WeightInfo<T>  {
	fn mint_land() -> Weight {
		Weight::from_parts(59_273_000, 36660)
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	fn mint_lands() -> Weight {
		Weight::from_parts(83_541_000, 41610)
			.saturating_add(T::DbWeight::get().reads(10))
			.saturating_add(T::DbWeight::get().writes(10))
	}
	fn transfer_land() -> Weight {
		Weight::from_parts(47_423_000, 28255)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	fn mint_estate() -> Weight {
		Weight::from_parts(64_479_000, 47210)
			.saturating_add(T::DbWeight::get().reads(10))
			.saturating_add(T::DbWeight::get().writes(11))
	}
	fn dissolve_estate() -> Weight {
		Weight::from_parts(110_008_000, 67224)
			.saturating_add(T::DbWeight::get().reads(14))
			.saturating_add(T::DbWeight::get().writes(13))
	}
	fn add_land_unit_to_estate() -> Weight {
		Weight::from_parts(71_706_000, 38079)
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	fn remove_land_unit_from_estate() -> Weight {
		Weight::from_parts(90_257_000, 60226)
			.saturating_add(T::DbWeight::get().reads(12))
			.saturating_add(T::DbWeight::get().writes(8))
	}
	fn create_estate() -> Weight {
		Weight::from_parts(131_741_000, 66058)
			.saturating_add(T::DbWeight::get().reads(14))
			.saturating_add(T::DbWeight::get().writes(17))
	}
	fn transfer_estate() -> Weight {
		Weight::from_parts(54_808_000, 38025)
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	fn issue_undeployed_land_blocks() -> Weight {
		Weight::from_parts(162_875_000, 9825)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(43))
	}
	fn freeze_undeployed_land_blocks() -> Weight {
		Weight::from_parts(22_833_000, 7834)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn unfreeze_undeployed_land_blocks() -> Weight {
		Weight::from_parts(21_423_000, 7834)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn approve_undeployed_land_blocks() -> Weight {
		Weight::from_parts(21_819_000, 7834)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn unapprove_undeployed_land_blocks() -> Weight {
		Weight::from_parts(21_237_000, 7900)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn transfer_undeployed_land_blocks() -> Weight {
		Weight::from_parts(41_173_000, 13673)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	fn deploy_land_block() -> Weight {
		Weight::from_parts(100_659_000, 64897)
			.saturating_add(T::DbWeight::get().reads(13))
			.saturating_add(T::DbWeight::get().writes(11))
	}
	fn burn_undeployed_land_blocks() -> Weight {
		Weight::from_parts(32_196_000, 10661)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn create_lease_offer() -> Weight {
		Weight::from_parts(98_902_000, 27383)
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn accept_lease_offer() -> Weight {
		Weight::from_parts(57_019_000, 28844)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	fn cancel_lease() -> Weight {
		Weight::from_parts(57_720_000, 28571)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	fn remove_expired_lease() -> Weight {
		Weight::from_parts(57_681_000, 28571)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	fn remove_lease_offer() -> Weight {
		Weight::from_parts(41_923_000, 8328)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn collect_rent() -> Weight {
		Weight::from_parts(53_171_000, 28571)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn on_initialize() -> Weight {
		Weight::from_parts(191_000, 0)
	}
}