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
//! DATE: 2023-07-18, STEPS: `20`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// ./target/release/metaverse-node
// benchmark
// pallet
// --execution=wasm
// --wasm-execution=compiled
// --pallet
// estate
// --extrinsic
// *
// --steps
// 20
// --repeat
// 10
// --template=./template/weight-template.hbs
// --output
// ./pallets/estate/src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for estate.
pub trait WeightInfo {	fn mint_land() -> Weight;	fn mint_lands() -> Weight;	fn transfer_land() -> Weight;	fn mint_estate() -> Weight;	fn dissolve_estate() -> Weight;	fn add_land_unit_to_estate() -> Weight;	fn remove_land_unit_from_estate() -> Weight;	fn create_estate() -> Weight;	fn transfer_estate() -> Weight;	fn issue_undeployed_land_blocks() -> Weight;	fn freeze_undeployed_land_blocks() -> Weight;	fn unfreeze_undeployed_land_blocks() -> Weight;	fn approve_undeployed_land_blocks() -> Weight;	fn unapprove_undeployed_land_blocks() -> Weight;	fn transfer_undeployed_land_blocks() -> Weight;	fn deploy_land_block() -> Weight;	fn burn_undeployed_land_blocks() -> Weight;	fn create_lease_offer() -> Weight;	fn accept_lease_offer() -> Weight;	fn cancel_lease() -> Weight;	fn remove_expired_lease() -> Weight;	fn remove_lease_offer() -> Weight;	fn collect_rent() -> Weight;	fn on_initialize() -> Weight;}

/// Weights for estate using the for collator node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {	// Storage: Estate LandUnits (r:1 w:1)
	// Proof Skipped: Estate LandUnits (max_values: None, max_size: None, mode: Measured)
	// Storage: Metaverse Metaverses (r:1 w:0)
	// Proof Skipped: Metaverse Metaverses (max_values: None, max_size: None, mode: Measured)
	// Storage: Nft LockedCollection (r:1 w:0)
	// Proof Skipped: Nft LockedCollection (max_values: None, max_size: None, mode: Measured)
	// Storage: System Account (r:1 w:1)
	// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	// Storage: OrmlNFT NextTokenId (r:1 w:1)
	// Proof Skipped: OrmlNFT NextTokenId (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Tokens (r:1 w:1)
	// Proof Skipped: OrmlNFT Tokens (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Classes (r:1 w:1)
	// Proof Skipped: OrmlNFT Classes (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate AllLandUnitsCount (r:1 w:1)
	// Proof Skipped: Estate AllLandUnitsCount (max_values: Some(1), max_size: None, mode: Measured)
	// Storage: OrmlNFT TokensByOwner (r:0 w:1)
	// Proof Skipped: OrmlNFT TokensByOwner (max_values: None, max_size: None, mode: Measured)
	fn mint_land() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2339`
		//  Estimated: `36660`
		// Minimum execution time: 56_882 nanoseconds.
		Weight::from_parts(59_273_000, 36660)
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	// Storage: Estate LandUnits (r:2 w:2)
	// Proof Skipped: Estate LandUnits (max_values: None, max_size: None, mode: Measured)
	// Storage: Metaverse Metaverses (r:1 w:0)
	// Proof Skipped: Metaverse Metaverses (max_values: None, max_size: None, mode: Measured)
	// Storage: Nft LockedCollection (r:1 w:0)
	// Proof Skipped: Nft LockedCollection (max_values: None, max_size: None, mode: Measured)
	// Storage: System Account (r:1 w:1)
	// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	// Storage: OrmlNFT NextTokenId (r:1 w:1)
	// Proof Skipped: OrmlNFT NextTokenId (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Tokens (r:2 w:2)
	// Proof Skipped: OrmlNFT Tokens (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Classes (r:1 w:1)
	// Proof Skipped: OrmlNFT Classes (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate AllLandUnitsCount (r:1 w:1)
	// Proof Skipped: Estate AllLandUnitsCount (max_values: Some(1), max_size: None, mode: Measured)
	// Storage: OrmlNFT TokensByOwner (r:0 w:2)
	// Proof Skipped: OrmlNFT TokensByOwner (max_values: None, max_size: None, mode: Measured)
	fn mint_lands() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2339`
		//  Estimated: `41610`
		// Minimum execution time: 82_026 nanoseconds.
		Weight::from_parts(83_541_000, 41610)
			.saturating_add(T::DbWeight::get().reads(10))
			.saturating_add(T::DbWeight::get().writes(10))
	}
	// Storage: Estate LandUnits (r:1 w:1)
	// Proof Skipped: Estate LandUnits (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Tokens (r:1 w:1)
	// Proof Skipped: OrmlNFT Tokens (max_values: None, max_size: None, mode: Measured)
	// Storage: Auction ItemsInAuction (r:1 w:0)
	// Proof Skipped: Auction ItemsInAuction (max_values: None, max_size: None, mode: Measured)
	// Storage: Nft LockedCollection (r:1 w:0)
	// Proof Skipped: Nft LockedCollection (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT StackableCollection (r:1 w:0)
	// Proof Skipped: OrmlNFT StackableCollection (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Classes (r:1 w:0)
	// Proof Skipped: OrmlNFT Classes (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT TokensByOwner (r:0 w:2)
	// Proof Skipped: OrmlNFT TokensByOwner (max_values: None, max_size: None, mode: Measured)
	fn transfer_land() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1915`
		//  Estimated: `28255`
		// Minimum execution time: 46_193 nanoseconds.
		Weight::from_parts(47_423_000, 28255)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: Estate NextEstateId (r:1 w:1)
	// Proof Skipped: Estate NextEstateId (max_values: Some(1), max_size: None, mode: Measured)
	// Storage: Metaverse Metaverses (r:1 w:0)
	// Proof Skipped: Metaverse Metaverses (max_values: None, max_size: None, mode: Measured)
	// Storage: Nft LockedCollection (r:1 w:0)
	// Proof Skipped: Nft LockedCollection (max_values: None, max_size: None, mode: Measured)
	// Storage: System Account (r:1 w:1)
	// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	// Storage: OrmlNFT NextTokenId (r:1 w:1)
	// Proof Skipped: OrmlNFT NextTokenId (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Tokens (r:1 w:1)
	// Proof Skipped: OrmlNFT Tokens (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Classes (r:1 w:1)
	// Proof Skipped: OrmlNFT Classes (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate LandUnits (r:1 w:1)
	// Proof Skipped: Estate LandUnits (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate AllLandUnitsCount (r:1 w:1)
	// Proof Skipped: Estate AllLandUnitsCount (max_values: Some(1), max_size: None, mode: Measured)
	// Storage: Estate AllEstatesCount (r:1 w:1)
	// Proof Skipped: Estate AllEstatesCount (max_values: Some(1), max_size: None, mode: Measured)
	// Storage: Estate EstateOwner (r:0 w:1)
	// Proof Skipped: Estate EstateOwner (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate Estates (r:0 w:1)
	// Proof Skipped: Estate Estates (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT TokensByOwner (r:0 w:1)
	// Proof Skipped: OrmlNFT TokensByOwner (max_values: None, max_size: None, mode: Measured)
	fn mint_estate() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2356`
		//  Estimated: `47210`
		// Minimum execution time: 62_931 nanoseconds.
		Weight::from_parts(64_479_000, 47210)
			.saturating_add(T::DbWeight::get().reads(10))
			.saturating_add(T::DbWeight::get().writes(11))
	}
	// Storage: Estate EstateOwner (r:1 w:1)
	// Proof Skipped: Estate EstateOwner (max_values: None, max_size: None, mode: Measured)
	// Storage: Auction ItemsInAuction (r:1 w:0)
	// Proof Skipped: Auction ItemsInAuction (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Tokens (r:2 w:2)
	// Proof Skipped: OrmlNFT Tokens (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate Estates (r:1 w:1)
	// Proof Skipped: Estate Estates (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Classes (r:2 w:2)
	// Proof Skipped: OrmlNFT Classes (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate AllEstatesCount (r:1 w:1)
	// Proof Skipped: Estate AllEstatesCount (max_values: Some(1), max_size: None, mode: Measured)
	// Storage: System Account (r:2 w:2)
	// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	// Storage: Estate LandUnits (r:1 w:1)
	// Proof Skipped: Estate LandUnits (max_values: None, max_size: None, mode: Measured)
	// Storage: Metaverse Metaverses (r:1 w:0)
	// Proof Skipped: Metaverse Metaverses (max_values: None, max_size: None, mode: Measured)
	// Storage: Nft LockedCollection (r:1 w:0)
	// Proof Skipped: Nft LockedCollection (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT NextTokenId (r:1 w:1)
	// Proof Skipped: OrmlNFT NextTokenId (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT TokensByOwner (r:0 w:2)
	// Proof Skipped: OrmlNFT TokensByOwner (max_values: None, max_size: None, mode: Measured)
	fn dissolve_estate() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `3118`
		//  Estimated: `67224`
		// Minimum execution time: 99_063 nanoseconds.
		Weight::from_parts(110_008_000, 67224)
			.saturating_add(T::DbWeight::get().reads(14))
			.saturating_add(T::DbWeight::get().writes(13))
	}
	// Storage: Estate EstateOwner (r:1 w:0)
	// Proof Skipped: Estate EstateOwner (max_values: None, max_size: None, mode: Measured)
	// Storage: Auction ItemsInAuction (r:1 w:0)
	// Proof Skipped: Auction ItemsInAuction (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Tokens (r:2 w:1)
	// Proof Skipped: OrmlNFT Tokens (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate Estates (r:1 w:1)
	// Proof Skipped: Estate Estates (max_values: None, max_size: None, mode: Measured)
	// Storage: System Account (r:1 w:1)
	// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	// Storage: Estate LandUnits (r:1 w:1)
	// Proof Skipped: Estate LandUnits (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Classes (r:1 w:1)
	// Proof Skipped: OrmlNFT Classes (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT TokensByOwner (r:0 w:1)
	// Proof Skipped: OrmlNFT TokensByOwner (max_values: None, max_size: None, mode: Measured)
	fn add_land_unit_to_estate() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2593`
		//  Estimated: `38079`
		// Minimum execution time: 69_608 nanoseconds.
		Weight::from_parts(71_706_000, 38079)
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	// Storage: Estate EstateOwner (r:1 w:0)
	// Proof Skipped: Estate EstateOwner (max_values: None, max_size: None, mode: Measured)
	// Storage: Auction ItemsInAuction (r:1 w:0)
	// Proof Skipped: Auction ItemsInAuction (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Tokens (r:2 w:1)
	// Proof Skipped: OrmlNFT Tokens (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate Estates (r:1 w:1)
	// Proof Skipped: Estate Estates (max_values: None, max_size: None, mode: Measured)
	// Storage: System Account (r:2 w:2)
	// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	// Storage: Estate LandUnits (r:1 w:1)
	// Proof Skipped: Estate LandUnits (max_values: None, max_size: None, mode: Measured)
	// Storage: Metaverse Metaverses (r:1 w:0)
	// Proof Skipped: Metaverse Metaverses (max_values: None, max_size: None, mode: Measured)
	// Storage: Nft LockedCollection (r:1 w:0)
	// Proof Skipped: Nft LockedCollection (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT NextTokenId (r:1 w:1)
	// Proof Skipped: OrmlNFT NextTokenId (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Classes (r:1 w:1)
	// Proof Skipped: OrmlNFT Classes (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT TokensByOwner (r:0 w:1)
	// Proof Skipped: OrmlNFT TokensByOwner (max_values: None, max_size: None, mode: Measured)
	fn remove_land_unit_from_estate() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `3027`
		//  Estimated: `60226`
		// Minimum execution time: 86_578 nanoseconds.
		Weight::from_parts(90_257_000, 60226)
			.saturating_add(T::DbWeight::get().reads(12))
			.saturating_add(T::DbWeight::get().writes(8))
	}
	// Storage: System Account (r:2 w:2)
	// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	// Storage: Estate NextEstateId (r:1 w:1)
	// Proof Skipped: Estate NextEstateId (max_values: Some(1), max_size: None, mode: Measured)
	// Storage: Metaverse Metaverses (r:1 w:0)
	// Proof Skipped: Metaverse Metaverses (max_values: None, max_size: None, mode: Measured)
	// Storage: Nft LockedCollection (r:1 w:0)
	// Proof Skipped: Nft LockedCollection (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT NextTokenId (r:1 w:1)
	// Proof Skipped: OrmlNFT NextTokenId (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Tokens (r:3 w:3)
	// Proof Skipped: OrmlNFT Tokens (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Classes (r:2 w:2)
	// Proof Skipped: OrmlNFT Classes (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate LandUnits (r:2 w:2)
	// Proof Skipped: Estate LandUnits (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate AllEstatesCount (r:1 w:1)
	// Proof Skipped: Estate AllEstatesCount (max_values: Some(1), max_size: None, mode: Measured)
	// Storage: Estate EstateOwner (r:0 w:1)
	// Proof Skipped: Estate EstateOwner (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate Estates (r:0 w:1)
	// Proof Skipped: Estate Estates (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT TokensByOwner (r:0 w:3)
	// Proof Skipped: OrmlNFT TokensByOwner (max_values: None, max_size: None, mode: Measured)
	fn create_estate() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `3192`
		//  Estimated: `66058`
		// Minimum execution time: 121_302 nanoseconds.
		Weight::from_parts(131_741_000, 66058)
			.saturating_add(T::DbWeight::get().reads(14))
			.saturating_add(T::DbWeight::get().writes(17))
	}
	// Storage: Estate EstateOwner (r:1 w:1)
	// Proof Skipped: Estate EstateOwner (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate Estates (r:1 w:0)
	// Proof Skipped: Estate Estates (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate EstateLeases (r:1 w:0)
	// Proof Skipped: Estate EstateLeases (max_values: None, max_size: None, mode: Measured)
	// Storage: Auction ItemsInAuction (r:1 w:0)
	// Proof Skipped: Auction ItemsInAuction (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Tokens (r:1 w:1)
	// Proof Skipped: OrmlNFT Tokens (max_values: None, max_size: None, mode: Measured)
	// Storage: Nft LockedCollection (r:1 w:0)
	// Proof Skipped: Nft LockedCollection (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT StackableCollection (r:1 w:0)
	// Proof Skipped: OrmlNFT StackableCollection (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Classes (r:1 w:0)
	// Proof Skipped: OrmlNFT Classes (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT TokensByOwner (r:0 w:2)
	// Proof Skipped: OrmlNFT TokensByOwner (max_values: None, max_size: None, mode: Measured)
	fn transfer_estate() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2025`
		//  Estimated: `38025`
		// Minimum execution time: 52_643 nanoseconds.
		Weight::from_parts(54_808_000, 38025)
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: System Account (r:1 w:1)
	// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	// Storage: Estate NextUndeployedLandBlockId (r:1 w:1)
	// Proof Skipped: Estate NextUndeployedLandBlockId (max_values: Some(1), max_size: None, mode: Measured)
	// Storage: Estate TotalUndeployedLandUnit (r:1 w:1)
	// Proof Skipped: Estate TotalUndeployedLandUnit (max_values: Some(1), max_size: None, mode: Measured)
	// Storage: Estate UndeployedLandBlocks (r:0 w:20)
	// Proof Skipped: Estate UndeployedLandBlocks (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate UndeployedLandBlocksOwner (r:0 w:20)
	// Proof Skipped: Estate UndeployedLandBlocksOwner (max_values: None, max_size: None, mode: Measured)
	fn issue_undeployed_land_blocks() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1558`
		//  Estimated: `9825`
		// Minimum execution time: 159_581 nanoseconds.
		Weight::from_parts(162_875_000, 9825)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(43))
	}
	// Storage: Estate UndeployedLandBlocks (r:1 w:1)
	// Proof Skipped: Estate UndeployedLandBlocks (max_values: None, max_size: None, mode: Measured)
	// Storage: Auction ItemsInAuction (r:1 w:0)
	// Proof Skipped: Auction ItemsInAuction (max_values: None, max_size: None, mode: Measured)
	fn freeze_undeployed_land_blocks() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1442`
		//  Estimated: `7834`
		// Minimum execution time: 21_762 nanoseconds.
		Weight::from_parts(22_833_000, 7834)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Estate UndeployedLandBlocks (r:1 w:1)
	// Proof Skipped: Estate UndeployedLandBlocks (max_values: None, max_size: None, mode: Measured)
	// Storage: Auction ItemsInAuction (r:1 w:0)
	// Proof Skipped: Auction ItemsInAuction (max_values: None, max_size: None, mode: Measured)
	fn unfreeze_undeployed_land_blocks() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1442`
		//  Estimated: `7834`
		// Minimum execution time: 20_225 nanoseconds.
		Weight::from_parts(21_423_000, 7834)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Estate UndeployedLandBlocks (r:1 w:1)
	// Proof Skipped: Estate UndeployedLandBlocks (max_values: None, max_size: None, mode: Measured)
	// Storage: Auction ItemsInAuction (r:1 w:0)
	// Proof Skipped: Auction ItemsInAuction (max_values: None, max_size: None, mode: Measured)
	fn approve_undeployed_land_blocks() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1442`
		//  Estimated: `7834`
		// Minimum execution time: 20_246 nanoseconds.
		Weight::from_parts(21_819_000, 7834)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Estate UndeployedLandBlocks (r:1 w:1)
	// Proof Skipped: Estate UndeployedLandBlocks (max_values: None, max_size: None, mode: Measured)
	// Storage: Auction ItemsInAuction (r:1 w:0)
	// Proof Skipped: Auction ItemsInAuction (max_values: None, max_size: None, mode: Measured)
	fn unapprove_undeployed_land_blocks() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1475`
		//  Estimated: `7900`
		// Minimum execution time: 20_346 nanoseconds.
		Weight::from_parts(21_237_000, 7900)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Auction ItemsInAuction (r:1 w:0)
	// Proof Skipped: Auction ItemsInAuction (max_values: None, max_size: None, mode: Measured)
	// Storage: System Account (r:1 w:1)
	// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	// Storage: Estate UndeployedLandBlocks (r:1 w:1)
	// Proof Skipped: Estate UndeployedLandBlocks (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate UndeployedLandBlocksOwner (r:0 w:2)
	// Proof Skipped: Estate UndeployedLandBlocksOwner (max_values: None, max_size: None, mode: Measured)
	fn transfer_undeployed_land_blocks() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2040`
		//  Estimated: `13673`
		// Minimum execution time: 38_679 nanoseconds.
		Weight::from_parts(41_173_000, 13673)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: Auction ItemsInAuction (r:1 w:0)
	// Proof Skipped: Auction ItemsInAuction (max_values: None, max_size: None, mode: Measured)
	// Storage: Metaverse MetaverseOwner (r:1 w:0)
	// Proof Skipped: Metaverse MetaverseOwner (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate UndeployedLandBlocks (r:1 w:1)
	// Proof Skipped: Estate UndeployedLandBlocks (max_values: None, max_size: None, mode: Measured)
	// Storage: System Account (r:2 w:2)
	// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	// Storage: Estate LandUnits (r:1 w:1)
	// Proof Skipped: Estate LandUnits (max_values: None, max_size: None, mode: Measured)
	// Storage: Metaverse Metaverses (r:1 w:0)
	// Proof Skipped: Metaverse Metaverses (max_values: None, max_size: None, mode: Measured)
	// Storage: Nft LockedCollection (r:1 w:0)
	// Proof Skipped: Nft LockedCollection (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT NextTokenId (r:1 w:1)
	// Proof Skipped: OrmlNFT NextTokenId (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Tokens (r:1 w:1)
	// Proof Skipped: OrmlNFT Tokens (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Classes (r:1 w:1)
	// Proof Skipped: OrmlNFT Classes (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate AllLandUnitsCount (r:1 w:1)
	// Proof Skipped: Estate AllLandUnitsCount (max_values: Some(1), max_size: None, mode: Measured)
	// Storage: Estate TotalUndeployedLandUnit (r:1 w:1)
	// Proof Skipped: Estate TotalUndeployedLandUnit (max_values: Some(1), max_size: None, mode: Measured)
	// Storage: Estate UndeployedLandBlocksOwner (r:0 w:1)
	// Proof Skipped: Estate UndeployedLandBlocksOwner (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT TokensByOwner (r:0 w:1)
	// Proof Skipped: OrmlNFT TokensByOwner (max_values: None, max_size: None, mode: Measured)
	fn deploy_land_block() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2802`
		//  Estimated: `64897`
		// Minimum execution time: 91_163 nanoseconds.
		Weight::from_parts(100_659_000, 64897)
			.saturating_add(T::DbWeight::get().reads(13))
			.saturating_add(T::DbWeight::get().writes(11))
	}
	// Storage: Estate UndeployedLandBlocks (r:1 w:1)
	// Proof Skipped: Estate UndeployedLandBlocks (max_values: None, max_size: None, mode: Measured)
	// Storage: Auction ItemsInAuction (r:1 w:0)
	// Proof Skipped: Auction ItemsInAuction (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate TotalUndeployedLandUnit (r:1 w:1)
	// Proof Skipped: Estate TotalUndeployedLandUnit (max_values: Some(1), max_size: None, mode: Measured)
	// Storage: Estate UndeployedLandBlocksOwner (r:0 w:1)
	// Proof Skipped: Estate UndeployedLandBlocksOwner (max_values: None, max_size: None, mode: Measured)
	fn burn_undeployed_land_blocks() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1304`
		//  Estimated: `10661`
		// Minimum execution time: 23_162 nanoseconds.
		Weight::from_parts(32_196_000, 10661)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: Estate EstateOwner (r:1 w:0)
	// Proof Skipped: Estate EstateOwner (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Tokens (r:1 w:0)
	// Proof Skipped: OrmlNFT Tokens (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate EstateLeaseOffers (r:2 w:1)
	// Proof Skipped: Estate EstateLeaseOffers (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate EstateLeases (r:1 w:0)
	// Proof Skipped: Estate EstateLeases (max_values: None, max_size: None, mode: Measured)
	// Storage: Auction ItemsInAuction (r:1 w:0)
	// Proof Skipped: Auction ItemsInAuction (max_values: None, max_size: None, mode: Measured)
	// Storage: System Account (r:1 w:1)
	// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn create_lease_offer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1986`
		//  Estimated: `27383`
		// Minimum execution time: 94_497 nanoseconds.
		Weight::from_parts(98_902_000, 27383)
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Estate EstateLeases (r:1 w:1)
	// Proof Skipped: Estate EstateLeases (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate EstateOwner (r:1 w:0)
	// Proof Skipped: Estate EstateOwner (max_values: None, max_size: None, mode: Measured)
	// Storage: Auction ItemsInAuction (r:1 w:0)
	// Proof Skipped: Auction ItemsInAuction (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Tokens (r:1 w:1)
	// Proof Skipped: OrmlNFT Tokens (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate EstateLeaseOffers (r:1 w:1)
	// Proof Skipped: Estate EstateLeaseOffers (max_values: None, max_size: None, mode: Measured)
	// Storage: System Account (r:1 w:1)
	// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	// Storage: Estate EstateLeasors (r:0 w:1)
	// Proof Skipped: Estate EstateLeasors (max_values: None, max_size: None, mode: Measured)
	fn accept_lease_offer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2311`
		//  Estimated: `28844`
		// Minimum execution time: 53_956 nanoseconds.
		Weight::from_parts(57_019_000, 28844)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	// Storage: Estate EstateLeases (r:1 w:1)
	// Proof Skipped: Estate EstateLeases (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate EstateLeasors (r:1 w:1)
	// Proof Skipped: Estate EstateLeasors (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate EstateOwner (r:1 w:0)
	// Proof Skipped: Estate EstateOwner (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Tokens (r:1 w:1)
	// Proof Skipped: OrmlNFT Tokens (max_values: None, max_size: None, mode: Measured)
	// Storage: System Account (r:1 w:1)
	// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn cancel_lease() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `4017`
		//  Estimated: `28571`
		// Minimum execution time: 55_907 nanoseconds.
		Weight::from_parts(57_720_000, 28571)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: Estate EstateLeases (r:1 w:1)
	// Proof Skipped: Estate EstateLeases (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate EstateLeasors (r:1 w:1)
	// Proof Skipped: Estate EstateLeasors (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate EstateOwner (r:1 w:0)
	// Proof Skipped: Estate EstateOwner (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Tokens (r:1 w:1)
	// Proof Skipped: OrmlNFT Tokens (max_values: None, max_size: None, mode: Measured)
	// Storage: System Account (r:1 w:1)
	// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn remove_expired_lease() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `4017`
		//  Estimated: `28571`
		// Minimum execution time: 56_789 nanoseconds.
		Weight::from_parts(57_681_000, 28571)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: Estate EstateLeaseOffers (r:1 w:1)
	// Proof Skipped: Estate EstateLeaseOffers (max_values: None, max_size: None, mode: Measured)
	// Storage: System Account (r:1 w:1)
	// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn remove_lease_offer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `3250`
		//  Estimated: `8328`
		// Minimum execution time: 33_999 nanoseconds.
		Weight::from_parts(41_923_000, 8328)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Estate EstateOwner (r:1 w:0)
	// Proof Skipped: Estate EstateOwner (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Tokens (r:1 w:0)
	// Proof Skipped: OrmlNFT Tokens (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate EstateLeasors (r:1 w:0)
	// Proof Skipped: Estate EstateLeasors (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate EstateLeases (r:1 w:1)
	// Proof Skipped: Estate EstateLeases (max_values: None, max_size: None, mode: Measured)
	// Storage: System Account (r:1 w:1)
	// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn collect_rent() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `4017`
		//  Estimated: `28571`
		// Minimum execution time: 51_821 nanoseconds.
		Weight::from_parts(53_171_000, 28571)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn on_initialize() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 176 nanoseconds.
		Weight::from_parts(191_000, 0)
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {	fn mint_land() -> Weight {
		Weight::from_parts(59_273_000, 36660)
			.saturating_add(RocksDbWeight::get().reads(8))
			.saturating_add(RocksDbWeight::get().writes(7))
	}
	fn mint_lands() -> Weight {
		Weight::from_parts(83_541_000, 41610)
			.saturating_add(RocksDbWeight::get().reads(10))
			.saturating_add(RocksDbWeight::get().writes(10))
	}
	fn transfer_land() -> Weight {
		Weight::from_parts(47_423_000, 28255)
			.saturating_add(RocksDbWeight::get().reads(6))
			.saturating_add(RocksDbWeight::get().writes(4))
	}
	fn mint_estate() -> Weight {
		Weight::from_parts(64_479_000, 47210)
			.saturating_add(RocksDbWeight::get().reads(10))
			.saturating_add(RocksDbWeight::get().writes(11))
	}
	fn dissolve_estate() -> Weight {
		Weight::from_parts(110_008_000, 67224)
			.saturating_add(RocksDbWeight::get().reads(14))
			.saturating_add(RocksDbWeight::get().writes(13))
	}
	fn add_land_unit_to_estate() -> Weight {
		Weight::from_parts(71_706_000, 38079)
			.saturating_add(RocksDbWeight::get().reads(8))
			.saturating_add(RocksDbWeight::get().writes(6))
	}
	fn remove_land_unit_from_estate() -> Weight {
		Weight::from_parts(90_257_000, 60226)
			.saturating_add(RocksDbWeight::get().reads(12))
			.saturating_add(RocksDbWeight::get().writes(8))
	}
	fn create_estate() -> Weight {
		Weight::from_parts(131_741_000, 66058)
			.saturating_add(RocksDbWeight::get().reads(14))
			.saturating_add(RocksDbWeight::get().writes(17))
	}
	fn transfer_estate() -> Weight {
		Weight::from_parts(54_808_000, 38025)
			.saturating_add(RocksDbWeight::get().reads(8))
			.saturating_add(RocksDbWeight::get().writes(4))
	}
	fn issue_undeployed_land_blocks() -> Weight {
		Weight::from_parts(162_875_000, 9825)
			.saturating_add(RocksDbWeight::get().reads(3))
			.saturating_add(RocksDbWeight::get().writes(43))
	}
	fn freeze_undeployed_land_blocks() -> Weight {
		Weight::from_parts(22_833_000, 7834)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	fn unfreeze_undeployed_land_blocks() -> Weight {
		Weight::from_parts(21_423_000, 7834)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	fn approve_undeployed_land_blocks() -> Weight {
		Weight::from_parts(21_819_000, 7834)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	fn unapprove_undeployed_land_blocks() -> Weight {
		Weight::from_parts(21_237_000, 7900)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	fn transfer_undeployed_land_blocks() -> Weight {
		Weight::from_parts(41_173_000, 13673)
			.saturating_add(RocksDbWeight::get().reads(3))
			.saturating_add(RocksDbWeight::get().writes(4))
	}
	fn deploy_land_block() -> Weight {
		Weight::from_parts(100_659_000, 64897)
			.saturating_add(RocksDbWeight::get().reads(13))
			.saturating_add(RocksDbWeight::get().writes(11))
	}
	fn burn_undeployed_land_blocks() -> Weight {
		Weight::from_parts(32_196_000, 10661)
			.saturating_add(RocksDbWeight::get().reads(3))
			.saturating_add(RocksDbWeight::get().writes(3))
	}
	fn create_lease_offer() -> Weight {
		Weight::from_parts(98_902_000, 27383)
			.saturating_add(RocksDbWeight::get().reads(7))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
	fn accept_lease_offer() -> Weight {
		Weight::from_parts(57_019_000, 28844)
			.saturating_add(RocksDbWeight::get().reads(6))
			.saturating_add(RocksDbWeight::get().writes(5))
	}
	fn cancel_lease() -> Weight {
		Weight::from_parts(57_720_000, 28571)
			.saturating_add(RocksDbWeight::get().reads(5))
			.saturating_add(RocksDbWeight::get().writes(4))
	}
	fn remove_expired_lease() -> Weight {
		Weight::from_parts(57_681_000, 28571)
			.saturating_add(RocksDbWeight::get().reads(5))
			.saturating_add(RocksDbWeight::get().writes(4))
	}
	fn remove_lease_offer() -> Weight {
		Weight::from_parts(41_923_000, 8328)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
	fn collect_rent() -> Weight {
		Weight::from_parts(53_171_000, 28571)
			.saturating_add(RocksDbWeight::get().reads(5))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
	fn on_initialize() -> Weight {
		Weight::from_parts(191_000, 0)
	}
}