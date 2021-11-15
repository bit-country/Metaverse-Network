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
pub use crate::Pallet as AuctionModule;
// use pallet_estate::Pallet as EstateModule;
use pallet_metaverse::Pallet as MetaverseModule;
use pallet_nft::Pallet as NFTModule;
use pallet_nft::{CollectionType, TokenType};

use crate::{Call, Config};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;
use primitives::Balance;
use sp_runtime::traits::{AccountIdConversion, StaticLookup, UniqueSaturatedInto};
// use orml_traits::BasicCurrencyExtended;
use auction_manager::{CheckAuctionItemHandler, ListingLevel};
use bc_primitives::{MetaverseInfo, MetaverseTrait};
use primitives::{FungibleTokenId, UndeployedLandBlock, UndeployedLandBlockId, UndeployedLandBlockType};

pub type AccountId = u128;
pub type LandId = u64;
pub type EstateId = u64;
pub type MetaverseId = u64;

const SEED: u32 = 0;

const METAVERSE_ID: u64 = 0;
const ALICE: AccountId = 1;
const BENEFICIARY_ID: AccountId = 99;
pub const ALICE_METAVERSE_ID: MetaverseId = 1;
pub const BOB_METAVERSE_ID: MetaverseId = 2;

const MAX_BOUND: (i32, i32) = (-100, 100);
const COORDINATE_IN_1: (i32, i32) = (-10, 10);
const COORDINATE_IN_2: (i32, i32) = (-5, 5);
const COORDINATE_OUT: (i32, i32) = (0, 101);
const COORDINATE_IN_AUCTION: (i32, i32) = (99, 99);
const ESTATE_IN_AUCTION: EstateId = 99;

fn dollar(d: u32) -> Balance {
	let d: Balance = d.into();
	d.saturating_mul(1_000_000_000_000_000_000)
}

fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	let initial_balance = dollar(1000);

	<T as pallet::Config>::Currency::make_free_balance_be(&caller, initial_balance.unique_saturated_into());
	caller
}

fn mint_NFT<T: Config>(caller: T::AccountId) {
	// let caller: T::AccountId = account(name, index, SEED);
	// let initial_balance = dollar(1000);

	// <T as pallet::Config>::Currency::make_free_balance_be(&caller,
	// initial_balance.unique_saturated_into());
	NFTModule::<T>::create_group(RawOrigin::Root.into(), vec![1], vec![1]);
	NFTModule::<T>::create_class(
		RawOrigin::Signed(caller.clone()).into(),
		vec![1],
		0u32.into(),
		TokenType::Transferable,
		CollectionType::Collectable,
	);
	NFTModule::<T>::mint(
		RawOrigin::Signed(caller.clone()).into(),
		0u32.into(),
		vec![1],
		vec![2],
		vec![1, 2, 3],
		3,
	);
}

pub struct MetaverseInfoSource {}

impl MetaverseTrait<AccountId> for MetaverseInfoSource {
	fn check_ownership(who: &AccountId, metaverse_id: &MetaverseId) -> bool {
		match *who {
			ALICE => *metaverse_id == ALICE_METAVERSE_ID,
			BOB => *metaverse_id == BOB_METAVERSE_ID,
			_ => false,
		}
	}

	fn get_metaverse(metaverse_id: u64) -> Option<MetaverseInfo<u128>> {
		None
	}

	fn get_metaverse_token(metaverse_id: u64) -> Option<FungibleTokenId> {
		return Some(FungibleTokenId::FungibleToken(0u32.into()));
	}

	fn update_metaverse_token(metaverse_id: u64, currency_id: FungibleTokenId) -> Result<(), DispatchError> {
		Ok(())
	}
}

benchmarks! {
	// create_new_auction at global level
	// create_new_auction{
	// 	frame_system::Pallet::<T>::set_block_number(1u32.into());
	//
	// 	let caller = funded_account::<T>("caller", 0);
	//
	// 	mint_NFT::<T>(caller.clone());
	// }: _(RawOrigin::Signed(caller.clone()), ItemId::NFT(0), 100u32.into(), 100u32.into(), ListingLevel::Global)

	// create_new_auction at local metaverse level
	create_new_auction{
		frame_system::Pallet::<T>::set_block_number(1u32.into());
		// <T as pallet::Config>::MetaverseInfoSource = MetaverseInfoSource;
		// <T as pallet::Config>::MetaverseInfoSource::update_metaverse_token(METAVERSE_ID, FungibleTokenId::NativeToken(0u32.into()));

		let caller = funded_account::<T>("caller", 0);
		// MetaverseModule::<T>::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1]);

		// T::MetaverseInfoSource::update_metaverse_token(METAVERSE_ID, 0u32.into());
		mint_NFT::<T>(caller.clone());
	}: _(RawOrigin::Signed(caller.clone()), ItemId::NFT(0), 100u32.into(), 100u32.into(), ListingLevel::Local(METAVERSE_ID))

	// create_new_buy_now
	create_new_buy_now{
		frame_system::Pallet::<T>::set_block_number(1u32.into());
		// let blockNumber: BlockNumber = T::MinimumAuctionDuration::get().into().saturating_add(5u32.into());

		let caller = funded_account::<T>("caller", 0);
		mint_NFT::<T>(caller.clone());
	}: _(RawOrigin::Signed(caller.clone()), ItemId::NFT(0), 100u32.into(), 100u32.into(), ListingLevel::Global)

	// bid
	bid{
		frame_system::Pallet::<T>::set_block_number(1u32.into());

		let caller = funded_account::<T>("caller", 0);
		let bidder = funded_account::<T>("bidder", 0);
		mint_NFT::<T>(caller.clone());

		crate::Pallet::<T>::create_new_auction(RawOrigin::Signed(caller.clone()).into(), ItemId::NFT(0), 100u32.into(), 100u32.into(), ListingLevel::Global);
	}: _(RawOrigin::Signed(bidder.clone()), 0u32.into(), 100u32.into())

	// buy_now
	buy_now{
		frame_system::Pallet::<T>::set_block_number(1u32.into());

		let caller = funded_account::<T>("caller", 0);
		let bidder = funded_account::<T>("bidder", 0);
		mint_NFT::<T>(caller.clone());

		crate::Pallet::<T>::create_new_buy_now(RawOrigin::Signed(caller.clone()).into(), ItemId::NFT(0), 100u32.into(), 100u32.into(), ListingLevel::Global);
	}: _(RawOrigin::Signed(bidder.clone()), 0u32.into(), 100u32.into())

	// bid_local
	bid_local{
		frame_system::Pallet::<T>::set_block_number(1u32.into());

		let caller = funded_account::<T>("caller", 0);
		let bidder = funded_account::<T>("bidder", 0);
		mint_NFT::<T>(caller.clone());

		// TODO: need to set FungibleTokenCurrency balance
		// 	<T as pallet::Config>::FungibleTokenCurrency::reserve( FungibleTokenId::FungibleToken(0u32.into()), &caller, initial_balance.unique_saturated_into());

		crate::Pallet::<T>::create_new_auction(RawOrigin::Signed(caller.clone()).into(), ItemId::NFT(0), 100u32.into(), 100u32.into(), ListingLevel::Local(METAVERSE_ID));
	}: _(RawOrigin::Signed(bidder.clone()), 0u32.into(), METAVERSE_ID, 10u32.into())

	// buy_now_local
	// buy_now_local{
	// 	frame_system::Pallet::<T>::set_block_number(1u32.into());
	//
	// 	let caller = funded_account::<T>("caller", 0);
	// 	let bidder = funded_account::<T>("bidder", 0);
	// 	mint_NFT::<T>(caller.clone());

	// TODO: need to set FungibleTokenCurrency balance
	// 	<T as pallet::Config>::FungibleTokenCurrency::reserve( FungibleTokenId::FungibleToken(0u32.into()), &caller, initial_balance.unique_saturated_into());

	// 	crate::Pallet::<T>::create_new_buy_now(RawOrigin::Signed(caller.clone()).into(), ItemId::NFT(0), 1u32.into(), 100u32.into(), ListingLevel::Local(METAVERSE_ID));
	// }: _(RawOrigin::Signed(bidder.clone()), 0u32.into(), METAVERSE_ID, 100u32.into())
}

impl_benchmark_test_suite!(Pallet, crate::benchmarking::tests::new_test_ext(), crate::mock::Test);

// below are code samples
// #[cfg(test)]
// mod mock {
// 	use super::*;
// 	use crate as auction;
// 	use frame_support::{construct_runtime, pallet_prelude::Hooks, parameter_types, PalletId};
// 	use orml_traits::parameter_type_with_key;
// 	use primitives::{continuum::Continuum, estate::Estate, Amount, AuctionId, CurrencyId, EstateId,
// FungibleTokenId}; 	use sp_core::H256;
// 	use sp_runtime::traits::AccountIdConversion;
// 	use sp_runtime::{testing::Header, traits::IdentityLookup};
//
// 	use auction_manager::{CheckAuctionItemHandler, ListingLevel};
// 	use bc_primitives::{MetaverseInfo, MetaverseTrait};
// 	use frame_support::traits::{Contains, Nothing};
// 	use log::info;
//
// 	parameter_types! {
// 		pub const BlockHashCount: u32 = 256;
// 	}
//
// 	pub type AccountId = u128;
// 	pub type Balance = u128;
// 	pub type BlockNumber = u64;
// 	pub type MetaverseId = u64;
//
// 	pub const ALICE: AccountId = 1;
// 	pub const BOB: AccountId = 2;
// 	pub const CLASS_ID: u32 = 0;
// 	pub const COLLECTION_ID: u64 = 0;
// 	pub const ALICE_METAVERSE_ID: MetaverseId = 1;
// 	pub const BOB_METAVERSE_ID: MetaverseId = 2;
//
// 	pub const ESTATE_ID_EXIST: EstateId = 0;
// 	pub const ESTATE_ID_EXIST_1: EstateId = 1;
// 	pub const ESTATE_ID_NOT_EXIST: EstateId = 99;
// 	pub const LAND_UNIT_EXIST: (i32, i32) = (0, 0);
// 	pub const LAND_UNIT_EXIST_1: (i32, i32) = (1, 1);
// 	pub const LAND_UNIT_NOT_EXIST: (i32, i32) = (99, 99);
//
// 	pub struct MetaverseInfoSource {}
//
// 	impl MetaverseTrait<AccountId> for MetaverseInfoSource {
// 		fn check_ownership(who: &AccountId, metaverse_id: &MetaverseId) -> bool {
// 			match *who {
// 				ALICE => *metaverse_id == ALICE_METAVERSE_ID,
// 				BOB => *metaverse_id == BOB_METAVERSE_ID,
// 				_ => false,
// 			}
// 		}
//
// 		fn get_metaverse(metaverse_id: u64) -> Option<MetaverseInfo<u128>> {
// 			None
// 		}
//
// 		fn get_metaverse_token(metaverse_id: u64) -> Option<FungibleTokenId> {
// 			info!("get_metaverse_token metaverse_id: {:?}", metaverse_id);
//
// 			return Some(FungibleTokenId::FungibleToken(0u32.into()));
// 		}
//
// 		fn update_metaverse_token(metaverse_id: u64, currency_id: FungibleTokenId) -> Result<(),
// DispatchError> { 			Ok(())
// 		}
// 	}
//
//
// 	impl frame_system::Config for Runtime {
// 		type Origin = Origin;
// 		type Index = u64;
// 		type BlockNumber = BlockNumber;
// 		type Call = Call;
// 		type Hash = H256;
// 		type Hashing = ::sp_runtime::traits::BlakeTwo256;
// 		type AccountId = AccountId;
// 		type Lookup = IdentityLookup<Self::AccountId>;
// 		type Header = Header;
// 		type Event = Event;
// 		type BlockHashCount = BlockHashCount;
// 		type BlockWeights = ();
// 		type BlockLength = ();
// 		type Version = ();
// 		type PalletInfo = PalletInfo;
// 		type AccountData = pallet_balances::AccountData<Balance>;
// 		type OnNewAccount = ();
// 		type OnKilledAccount = ();
// 		type DbWeight = ();
// 		type BaseCallFilter = frame_support::traits::Everything;
// 		type SystemWeightInfo = ();
// 		type SS58Prefix = ();
// 		type OnSetCode = ();
// 	}
//
// 	parameter_types! {
// 	pub const ExistentialDeposit: u64 = 1;
// }
//
// 	impl pallet_balances::Config for Runtime {
// 		type Balance = Balance;
// 		type Event = Event;
// 		type DustRemoval = ();
// 		type ExistentialDeposit = ExistentialDeposit;
// 		type AccountStore = System;
// 		type MaxLocks = ();
// 		type WeightInfo = ();
// 		type MaxReserves = ();
// 		type ReserveIdentifier = ();
// 	}
//
// 	pub struct Continuumm;
//
// 	impl Continuum<u128> for Continuumm {
// 		fn transfer_spot(spot_id: u64, from: &AccountId, to: &(AccountId, u64)) -> Result<u64,
// DispatchError> { 			Ok(1)
// 		}
// 	}
//
// 	pub struct EstateHandler;
//
// 	impl Estate<u128> for EstateHandler {
// 		fn transfer_estate(estate_id: EstateId, from: &AccountId, to: &AccountId) -> Result<EstateId,
// DispatchError> { 			Ok(1)
// 		}
//
// 		fn transfer_landunit(
// 			coordinate: (i32, i32),
// 			from: &AccountId,
// 			to: &(AccountId, MetaverseId),
// 		) -> Result<(i32, i32), DispatchError> {
// 			Ok((0, 0))
// 		}
//
// 		fn check_estate(estate_id: EstateId) -> Result<bool, DispatchError> {
// 			match estate_id {
// 				ESTATE_ID_EXIST | ESTATE_ID_EXIST_1 => Ok(true),
// 				ESTATE_ID_NOT_EXIST => Ok(false),
// 				_ => Ok(false),
// 			}
// 		}
//
// 		fn check_landunit(metaverse_id: MetaverseId, coordinate: (i32, i32)) -> Result<bool,
// DispatchError> { 			match coordinate {
// 				LAND_UNIT_EXIST | LAND_UNIT_EXIST_1 => Ok(true),
// 				LAND_UNIT_NOT_EXIST => Ok(false),
// 				_ => Ok(false),
// 			}
// 		}
//
// 		fn get_total_land_units() -> u64 {
// 			100
// 		}
//
// 		fn get_total_undeploy_land_units() -> u64 {
// 			100
// 		}
// 	}
//
// 	pub struct Handler;
//
// 	impl AuctionHandler<AccountId, Balance, BlockNumber, AuctionId> for Handler {
// 		fn on_new_bid(
// 			now: BlockNumber,
// 			id: AuctionId,
// 			new_bid: (AccountId, Balance),
// 			last_bid: Option<(AccountId, Balance)>,
// 		) -> OnNewBidResult<BlockNumber> {
// 			//Test with Alice bid
// 			if new_bid.0 == ALICE {
// 				OnNewBidResult {
// 					accept_bid: true,
// 					auction_end_change: Change::NoChange,
// 				}
// 			} else {
// 				OnNewBidResult {
// 					accept_bid: false,
// 					auction_end_change: Change::NoChange,
// 				}
// 			}
// 		}
//
// 		fn on_auction_ended(_id: AuctionId, _winner: Option<(AccountId, Balance)>) {}
// 	}
//
// 	parameter_type_with_key! {
// 		pub ExistentialDeposits: |_currency_id: FungibleTokenId| -> Balance {
// 			Default::default()
// 		};
// 	}
//
// 	parameter_types! {
// 	pub const MetaverseTreasuryPalletId: PalletId = PalletId(*b"bit/trsy");
// 	pub TreasuryModuleAccount: AccountId = MetaverseTreasuryPalletId::get().into_account();
// 	pub const MetaverseFundPalletId: PalletId = PalletId(*b"bit/fund");
// }
//
// 	impl orml_tokens::Config for Runtime {
// 		type Event = Event;
// 		type Balance = Balance;
// 		type Amount = Amount;
// 		type CurrencyId = FungibleTokenId;
// 		type WeightInfo = ();
// 		type ExistentialDeposits = ExistentialDeposits;
// 		type OnDust = orml_tokens::TransferDust<Runtime, TreasuryModuleAccount>;
// 		type MaxLocks = ();
// 		type DustRemovalWhitelist = Nothing;
// 	}
//
// 	parameter_types! {
// 		pub const AuctionTimeToClose: u64 = 100; //Test auction end within 100 blocks
// 		pub const MinimumAuctionDuration: u64 = 10; //Test auction end within 100 blocks
// 	}
//
// 	impl Config for Runtime {
// 		type Event = Event;
// 		type AuctionTimeToClose = AuctionTimeToClose;
// 		type Handler = Handler;
// 		type Currency = Balances;
// 		type ContinuumHandler = Continuumm;
// 		type FungibleTokenCurrency = Tokens;
// 		type MetaverseInfoSource = MetaverseInfoSource;
// 		type MinimumAuctionDuration = MinimumAuctionDuration;
// 		type EstateHandler = EstateHandler;
// 	}
//
// 	parameter_types! {
// 		pub CreateClassDeposit: Balance = 2;
// 		pub CreateAssetDeposit: Balance = 1;
// 		pub NftPalletId: PalletId = PalletId(*b"bit/bNFT");
// 	}
//
// 	impl pallet_nft::Config for Runtime {
// 		type Event = Event;
// 		type CreateClassDeposit = CreateClassDeposit;
// 		type CreateAssetDeposit = CreateAssetDeposit;
// 		type Currency = Balances;
// 		type PalletId = NftPalletId;
// 		type WeightInfo = ();
// 		type AuctionHandler = MockAuctionManager;
// 	}
//
// 	parameter_types! {
// 	pub MaxClassMetadata: u32 = 1024;
// 	pub MaxTokenMetadata: u32 = 1024;
// }
//
// 	impl orml_nft::Config for Runtime {
// 		type ClassId = u32;
// 		type TokenId = u64;
// 		type ClassData = pallet_nft::NftClassData<Balance>;
// 		type TokenData = pallet_nft::NftAssetData<Balance>;
// 		type MaxClassMetadata = MaxClassMetadata;
// 		type MaxTokenMetadata = MaxTokenMetadata;
// 	}
//
// 	type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
// 	type Block = frame_system::mocking::MockBlock<Runtime>;
//
// 	construct_runtime!(
// 		pub enum Runtime where
// 			Block = Block,
// 			NodeBlock = Block,
// 			UncheckedExtrinsic = UncheckedExtrinsic
// 			{
// 				System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
// 				Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
// 				Tokens: orml_tokens::{Pallet, Call, Storage, Config<T>, Event<T>},
// 				NFTModule: pallet_nft::{Pallet, Storage ,Call, Event<T>},
// 				OrmlNft: orml_nft::{Pallet, Storage, Config<T>},
// 				AuctionModule: auction::{Pallet, Call, Storage, Event<T>},
// 			}
// 	);
//
// 	use frame_system::Call as SystemCall;
//
// 	pub fn new_test_ext() -> sp_io::TestExternalities {
// 		let t = frame_system::GenesisConfig::default()
// 			.build_storage::<Runtime>()
// 			.unwrap();
//
// 		let mut ext = sp_io::TestExternalities::new(t);
// 		ext.execute_with(|| System::set_block_number(1));
// 		ext
// 	}
// }
//
// #[cfg(test)]
// mod tests {
// 	use super::mock::*;
// 	use super::*;
// 	use frame_benchmarking::impl_benchmark_test_suite;
//
// 	impl_benchmark_test_suite!(Pallet, super::new_test_ext(), super::Runtime,);
// }
