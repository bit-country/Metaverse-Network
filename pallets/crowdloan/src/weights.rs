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

//! Autogenerated weights for crowdloan
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-04-05, STEPS: `20`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/metaverse-node
// benchmark
// --chain=dev
// --pallet=crowdloan
// --extrinsic=*
// --steps=20
// --repeat=10
// --execution=wasm
// --wasm-execution=compiled
// --template=./template/weight-template.hbs
// --output
// ./pallets/crowdloan/src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for crowdloan.
pub trait WeightInfo {	fn set_distributor_origin() -> Weight;	fn remove_distributor_origin() -> Weight;	fn transfer_unlocked_reward() -> Weight;	fn transfer_vested_reward() -> Weight;	fn remove_vested_reward() -> Weight;}

/// Weights for crowdloan using the for collator node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {	fn set_distributor_origin() -> Weight {
		(11_000_000 as Weight)			.saturating_add(T::DbWeight::get().reads(1 as Weight))			.saturating_add(T::DbWeight::get().writes(1 as Weight))	}	fn remove_distributor_origin() -> Weight {
		(12_000_000 as Weight)			.saturating_add(T::DbWeight::get().reads(1 as Weight))			.saturating_add(T::DbWeight::get().writes(1 as Weight))	}	fn transfer_unlocked_reward() -> Weight {
		(30_000_000 as Weight)			.saturating_add(T::DbWeight::get().reads(3 as Weight))			.saturating_add(T::DbWeight::get().writes(2 as Weight))	}	fn transfer_vested_reward() -> Weight {
		(47_000_000 as Weight)			.saturating_add(T::DbWeight::get().reads(5 as Weight))			.saturating_add(T::DbWeight::get().writes(4 as Weight))	}	fn remove_vested_reward() -> Weight {
		(28_000_000 as Weight)			.saturating_add(T::DbWeight::get().reads(3 as Weight))			.saturating_add(T::DbWeight::get().writes(3 as Weight))	}}

// For backwards compatibility and tests
impl WeightInfo for () {	fn set_distributor_origin() -> Weight {
		(11_000_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(1 as Weight))			.saturating_add(RocksDbWeight::get().writes(1 as Weight))	}	fn remove_distributor_origin() -> Weight {
		(12_000_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(1 as Weight))			.saturating_add(RocksDbWeight::get().writes(1 as Weight))	}	fn transfer_unlocked_reward() -> Weight {
		(30_000_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(3 as Weight))			.saturating_add(RocksDbWeight::get().writes(2 as Weight))	}	fn transfer_vested_reward() -> Weight {
		(47_000_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(5 as Weight))			.saturating_add(RocksDbWeight::get().writes(4 as Weight))	}	fn remove_vested_reward() -> Weight {
		(28_000_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(3 as Weight))			.saturating_add(RocksDbWeight::get().writes(3 as Weight))	}}
