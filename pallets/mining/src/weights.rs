// This file is part of Bit.Country.

// Copyright (C) 2020-2021 Bit.Country.
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

//! Autogenerated weights for mining
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-04-15, STEPS: `20`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// ./target/release/metaverse-node
// benchmark
// --pallet=mining
// --extrinsic=*
// --steps=20
// --repeat=10
// --execution=wasm
// --wasm-execution=compiled
// --template=./template/weight-template.hbs
// --output
// ./pallets/mining/src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for mining.
pub trait WeightInfo {	fn add_minting_origin() -> Weight;	fn remove_minting_origin() -> Weight;	fn update_round_length() -> Weight;	fn update_mining_issuance_config() -> Weight;	fn mint() -> Weight;	fn burn() -> Weight;	fn deposit() -> Weight;	fn withdraw() -> Weight;}

/// Weights for mining using the for collator node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {	fn add_minting_origin() -> Weight {
		(24_400_000 as Weight)			.saturating_add(T::DbWeight::get().reads(1 as Weight))			.saturating_add(T::DbWeight::get().writes(1 as Weight))	}	fn remove_minting_origin() -> Weight {
		(57_000_000 as Weight)			.saturating_add(T::DbWeight::get().reads(1 as Weight))			.saturating_add(T::DbWeight::get().writes(1 as Weight))	}	fn update_round_length() -> Weight {
		(19_700_000 as Weight)			.saturating_add(T::DbWeight::get().reads(1 as Weight))			.saturating_add(T::DbWeight::get().writes(1 as Weight))	}	fn update_mining_issuance_config() -> Weight {
		(19_400_000 as Weight)			.saturating_add(T::DbWeight::get().reads(1 as Weight))			.saturating_add(T::DbWeight::get().writes(1 as Weight))	}	fn mint() -> Weight {
		(61_900_000 as Weight)			.saturating_add(T::DbWeight::get().reads(3 as Weight))			.saturating_add(T::DbWeight::get().writes(2 as Weight))	}	fn burn() -> Weight {
		(43_000_000 as Weight)			.saturating_add(T::DbWeight::get().reads(3 as Weight))			.saturating_add(T::DbWeight::get().writes(2 as Weight))	}	fn deposit() -> Weight {
		(79_400_000 as Weight)			.saturating_add(T::DbWeight::get().reads(4 as Weight))			.saturating_add(T::DbWeight::get().writes(3 as Weight))	}	fn withdraw() -> Weight {
		(62_400_000 as Weight)			.saturating_add(T::DbWeight::get().reads(4 as Weight))			.saturating_add(T::DbWeight::get().writes(2 as Weight))	}}

// For backwards compatibility and tests
impl WeightInfo for () {	fn add_minting_origin() -> Weight {
		(24_400_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(1 as Weight))			.saturating_add(RocksDbWeight::get().writes(1 as Weight))	}	fn remove_minting_origin() -> Weight {
		(57_000_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(1 as Weight))			.saturating_add(RocksDbWeight::get().writes(1 as Weight))	}	fn update_round_length() -> Weight {
		(19_700_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(1 as Weight))			.saturating_add(RocksDbWeight::get().writes(1 as Weight))	}	fn update_mining_issuance_config() -> Weight {
		(19_400_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(1 as Weight))			.saturating_add(RocksDbWeight::get().writes(1 as Weight))	}	fn mint() -> Weight {
		(61_900_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(3 as Weight))			.saturating_add(RocksDbWeight::get().writes(2 as Weight))	}	fn burn() -> Weight {
		(43_000_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(3 as Weight))			.saturating_add(RocksDbWeight::get().writes(2 as Weight))	}	fn deposit() -> Weight {
		(79_400_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(4 as Weight))			.saturating_add(RocksDbWeight::get().writes(3 as Weight))	}	fn withdraw() -> Weight {
		(62_400_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(4 as Weight))			.saturating_add(RocksDbWeight::get().writes(2 as Weight))	}}
