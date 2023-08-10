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

//! Autogenerated weights for auction
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-07-19, STEPS: `20`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// ./target/release/metaverse-node
// benchmark
// pallet
// --execution=wasm
// --wasm-execution=compiled
// --pallet
// auction
// --extrinsic
// *
// --steps
// 20
// --repeat
// 10
// --template=./template/weight-template.hbs
// --output
// ./pallets/auction/src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for auction.
pub trait WeightInfo {	fn create_new_auction() -> Weight;	fn create_new_buy_now() -> Weight;	fn bid() -> Weight;	fn buy_now() -> Weight;	fn cancel_listing() -> Weight;	fn authorise_metaverse_collection() -> Weight;	fn remove_authorise_metaverse_collection() -> Weight;	fn make_offer() -> Weight;	fn withdraw_offer() -> Weight;	fn accept_offer() -> Weight;	fn on_finalize() -> Weight;}

/// Weights for auction using the for collator node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {	// Storage: Metaverse Metaverses (r:1 w:0)
	// Proof Skipped: Metaverse Metaverses (max_values: None, max_size: None, mode: Measured)
	// Storage: Auction ItemsInAuction (r:1 w:1)
	// Proof Skipped: Auction ItemsInAuction (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Tokens (r:1 w:1)
	// Proof Skipped: OrmlNFT Tokens (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Classes (r:1 w:0)
	// Proof Skipped: OrmlNFT Classes (max_values: None, max_size: None, mode: Measured)
	// Storage: Auction MetaverseCollection (r:1 w:0)
	// Proof Skipped: Auction MetaverseCollection (max_values: None, max_size: None, mode: Measured)
	// Storage: Metaverse MetaverseOwner (r:1 w:0)
	// Proof Skipped: Metaverse MetaverseOwner (max_values: None, max_size: None, mode: Measured)
	// Storage: Auction AuctionEndTime (r:1 w:1)
	// Proof Skipped: Auction AuctionEndTime (max_values: None, max_size: None, mode: Measured)
	// Storage: System Account (r:1 w:1)
	// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	// Storage: Auction AuctionsIndex (r:1 w:1)
	// Proof Skipped: Auction AuctionsIndex (max_values: Some(1), max_size: None, mode: Measured)
	// Storage: Auction AuctionItems (r:0 w:1)
	// Proof Skipped: Auction AuctionItems (max_values: None, max_size: None, mode: Measured)
	// Storage: Auction Auctions (r:0 w:1)
	// Proof Skipped: Auction Auctions (max_values: None, max_size: None, mode: Measured)
	fn create_new_auction() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `3695`
		//  Estimated: `57373`
		// Minimum execution time: 59_917 nanoseconds.
		Weight::from_parts(61_885_000, 57373)
			.saturating_add(T::DbWeight::get().reads(9))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	// Storage: Metaverse Metaverses (r:1 w:0)
	// Proof Skipped: Metaverse Metaverses (max_values: None, max_size: None, mode: Measured)
	// Storage: System Account (r:2 w:2)
	// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	// Storage: Auction ItemsInAuction (r:1 w:1)
	// Proof Skipped: Auction ItemsInAuction (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Tokens (r:1 w:1)
	// Proof Skipped: OrmlNFT Tokens (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Classes (r:1 w:0)
	// Proof Skipped: OrmlNFT Classes (max_values: None, max_size: None, mode: Measured)
	// Storage: Auction MetaverseCollection (r:1 w:0)
	// Proof Skipped: Auction MetaverseCollection (max_values: None, max_size: None, mode: Measured)
	// Storage: Metaverse MetaverseOwner (r:1 w:0)
	// Proof Skipped: Metaverse MetaverseOwner (max_values: None, max_size: None, mode: Measured)
	// Storage: Auction AuctionEndTime (r:1 w:1)
	// Proof Skipped: Auction AuctionEndTime (max_values: None, max_size: None, mode: Measured)
	// Storage: Auction AuctionsIndex (r:1 w:1)
	// Proof Skipped: Auction AuctionsIndex (max_values: Some(1), max_size: None, mode: Measured)
	// Storage: Auction AuctionItems (r:0 w:1)
	// Proof Skipped: Auction AuctionItems (max_values: None, max_size: None, mode: Measured)
	// Storage: Auction Auctions (r:0 w:1)
	// Proof Skipped: Auction Auctions (max_values: None, max_size: None, mode: Measured)
	fn create_new_buy_now() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `3868`
		//  Estimated: `61706`
		// Minimum execution time: 72_460 nanoseconds.
		Weight::from_parts(74_722_000, 61706)
			.saturating_add(T::DbWeight::get().reads(10))
			.saturating_add(T::DbWeight::get().writes(8))
	}
	// Storage: Auction AuctionItems (r:1 w:1)
	// Proof Skipped: Auction AuctionItems (max_values: None, max_size: None, mode: Measured)
	// Storage: System Account (r:2 w:2)
	// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	// Storage: Auction Auctions (r:1 w:1)
	// Proof Skipped: Auction Auctions (max_values: None, max_size: None, mode: Measured)
	// Storage: Tokens Accounts (r:1 w:1)
	// Proof: Tokens Accounts (max_values: None, max_size: Some(121), added: 2596, mode: MaxEncodedLen)
	// Storage: Auction AuctionEndTime (r:0 w:2)
	// Proof Skipped: Auction AuctionEndTime (max_values: None, max_size: None, mode: Measured)
	fn bid() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `4133`
		//  Estimated: `25151`
		// Minimum execution time: 64_755 nanoseconds.
		Weight::from_parts(75_918_000, 25151)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	// Storage: Auction AuctionItems (r:1 w:1)
	// Proof Skipped: Auction AuctionItems (max_values: None, max_size: None, mode: Measured)
	// Storage: Auction Auctions (r:1 w:1)
	// Proof Skipped: Auction Auctions (max_values: None, max_size: None, mode: Measured)
	// Storage: Tokens Accounts (r:4 w:3)
	// Proof: Tokens Accounts (max_values: None, max_size: Some(121), added: 2596, mode: MaxEncodedLen)
	// Storage: System Account (r:3 w:2)
	// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	// Storage: OrmlNFT Classes (r:1 w:0)
	// Proof Skipped: OrmlNFT Classes (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Tokens (r:1 w:1)
	// Proof Skipped: OrmlNFT Tokens (max_values: None, max_size: None, mode: Measured)
	// Storage: Nft LockedCollection (r:1 w:0)
	// Proof Skipped: Nft LockedCollection (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT StackableCollection (r:1 w:0)
	// Proof Skipped: OrmlNFT StackableCollection (max_values: None, max_size: None, mode: Measured)
	// Storage: Auction ItemsInAuction (r:0 w:1)
	// Proof Skipped: Auction ItemsInAuction (max_values: None, max_size: None, mode: Measured)
	// Storage: Auction AuctionEndTime (r:0 w:1)
	// Proof Skipped: Auction AuctionEndTime (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT TokensByOwner (r:0 w:2)
	// Proof Skipped: OrmlNFT TokensByOwner (max_values: None, max_size: None, mode: Measured)
	fn buy_now() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `5079`
		//  Estimated: `78754`
		// Minimum execution time: 117_209 nanoseconds.
		Weight::from_parts(121_030_000, 78754)
			.saturating_add(T::DbWeight::get().reads(13))
			.saturating_add(T::DbWeight::get().writes(12))
	}
	// Storage: Auction Auctions (r:1 w:1)
	// Proof Skipped: Auction Auctions (max_values: None, max_size: None, mode: Measured)
	// Storage: Auction AuctionItems (r:1 w:1)
	// Proof Skipped: Auction AuctionItems (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Tokens (r:1 w:1)
	// Proof Skipped: OrmlNFT Tokens (max_values: None, max_size: None, mode: Measured)
	// Storage: System Account (r:1 w:1)
	// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	// Storage: Auction ItemsInAuction (r:0 w:1)
	// Proof Skipped: Auction ItemsInAuction (max_values: None, max_size: None, mode: Measured)
	// Storage: Auction AuctionEndTime (r:0 w:1)
	// Proof Skipped: Auction AuctionEndTime (max_values: None, max_size: None, mode: Measured)
	fn cancel_listing() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `4000`
		//  Estimated: `30028`
		// Minimum execution time: 49_038 nanoseconds.
		Weight::from_parts(50_823_000, 30028)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	// Storage: Metaverse MetaverseOwner (r:1 w:0)
	// Proof Skipped: Metaverse MetaverseOwner (max_values: None, max_size: None, mode: Measured)
	// Storage: Auction MetaverseCollection (r:1 w:1)
	// Proof Skipped: Auction MetaverseCollection (max_values: None, max_size: None, mode: Measured)
	fn authorise_metaverse_collection() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1307`
		//  Estimated: `7564`
		// Minimum execution time: 21_943 nanoseconds.
		Weight::from_parts(23_046_000, 7564)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Metaverse MetaverseOwner (r:1 w:0)
	// Proof Skipped: Metaverse MetaverseOwner (max_values: None, max_size: None, mode: Measured)
	// Storage: Auction MetaverseCollection (r:1 w:1)
	// Proof Skipped: Auction MetaverseCollection (max_values: None, max_size: None, mode: Measured)
	fn remove_authorise_metaverse_collection() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1360`
		//  Estimated: `7670`
		// Minimum execution time: 22_905 nanoseconds.
		Weight::from_parts(23_607_000, 7670)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: OrmlNFT Tokens (r:1 w:0)
	// Proof Skipped: OrmlNFT Tokens (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Classes (r:1 w:0)
	// Proof Skipped: OrmlNFT Classes (max_values: None, max_size: None, mode: Measured)
	// Storage: Auction Offers (r:1 w:1)
	// Proof Skipped: Auction Offers (max_values: None, max_size: None, mode: Measured)
	// Storage: System Account (r:1 w:1)
	// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn make_offer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1805`
		//  Estimated: `15443`
		// Minimum execution time: 36_625 nanoseconds.
		Weight::from_parts(38_079_000, 15443)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Auction Offers (r:1 w:1)
	// Proof Skipped: Auction Offers (max_values: None, max_size: None, mode: Measured)
	// Storage: System Account (r:1 w:1)
	// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn withdraw_offer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1534`
		//  Estimated: `6612`
		// Minimum execution time: 27_880 nanoseconds.
		Weight::from_parts(28_621_000, 6612)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: OrmlNFT Tokens (r:1 w:1)
	// Proof Skipped: OrmlNFT Tokens (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Classes (r:1 w:0)
	// Proof Skipped: OrmlNFT Classes (max_values: None, max_size: None, mode: Measured)
	// Storage: Auction Offers (r:1 w:1)
	// Proof Skipped: Auction Offers (max_values: None, max_size: None, mode: Measured)
	// Storage: System Account (r:2 w:2)
	// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	// Storage: Nft LockedCollection (r:1 w:0)
	// Proof Skipped: Nft LockedCollection (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT StackableCollection (r:1 w:0)
	// Proof Skipped: OrmlNFT StackableCollection (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT TokensByOwner (r:0 w:2)
	// Proof Skipped: OrmlNFT TokensByOwner (max_values: None, max_size: None, mode: Measured)
	fn accept_offer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2326`
		//  Estimated: `31537`
		// Minimum execution time: 68_875 nanoseconds.
		Weight::from_parts(72_454_000, 31537)
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	fn on_finalize() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `774`
		//  Estimated: `0`
		// Minimum execution time: 5_673 nanoseconds.
		Weight::from_parts(7_467_000, 0)
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {	fn create_new_auction() -> Weight {
		Weight::from_parts(61_885_000, 57373)
			.saturating_add(RocksDbWeight::get().reads(9))
			.saturating_add(RocksDbWeight::get().writes(7))
	}
	fn create_new_buy_now() -> Weight {
		Weight::from_parts(74_722_000, 61706)
			.saturating_add(RocksDbWeight::get().reads(10))
			.saturating_add(RocksDbWeight::get().writes(8))
	}
	fn bid() -> Weight {
		Weight::from_parts(75_918_000, 25151)
			.saturating_add(RocksDbWeight::get().reads(5))
			.saturating_add(RocksDbWeight::get().writes(7))
	}
	fn buy_now() -> Weight {
		Weight::from_parts(121_030_000, 78754)
			.saturating_add(RocksDbWeight::get().reads(13))
			.saturating_add(RocksDbWeight::get().writes(12))
	}
	fn cancel_listing() -> Weight {
		Weight::from_parts(50_823_000, 30028)
			.saturating_add(RocksDbWeight::get().reads(4))
			.saturating_add(RocksDbWeight::get().writes(6))
	}
	fn authorise_metaverse_collection() -> Weight {
		Weight::from_parts(23_046_000, 7564)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	fn remove_authorise_metaverse_collection() -> Weight {
		Weight::from_parts(23_607_000, 7670)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	fn make_offer() -> Weight {
		Weight::from_parts(38_079_000, 15443)
			.saturating_add(RocksDbWeight::get().reads(4))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
	fn withdraw_offer() -> Weight {
		Weight::from_parts(28_621_000, 6612)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
	fn accept_offer() -> Weight {
		Weight::from_parts(72_454_000, 31537)
			.saturating_add(RocksDbWeight::get().reads(7))
			.saturating_add(RocksDbWeight::get().writes(6))
	}
	fn on_finalize() -> Weight {
		Weight::from_parts(7_467_000, 0)
	}
}
