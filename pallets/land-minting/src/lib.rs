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

use frame_support::pallet_prelude::*;
use frame_support::{
	dispatch::DispatchResult,
	ensure, log,
	traits::{Currency, ExistenceRequirement, Get},
	transactional, PalletId,
};
use frame_system::pallet_prelude::*;
use frame_system::{ensure_root, ensure_signed};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{AccountIdConversion, Convert, One, Saturating, Zero},
	ArithmeticError, DispatchError, Perbill, SaturatedConversion,
};
use sp_std::vec::Vec;

use auction_manager::{Auction, CheckAuctionItemHandler};
use core_primitives::*;
pub use pallet::*;
use primitives::estate::EstateInfo;
use primitives::{
	estate::{Estate, LandUnitStatus, LeaseContract, OwnerId},
	Attributes, ClassId, EstateId, FungibleTokenId, ItemId, MetaverseId, NftMetadata, TokenId, UndeployedLandBlock,
	UndeployedLandBlockId, UndeployedLandBlockType,
};
pub use utils::{MintingRateInfo, Range};
pub use weights::WeightInfo;

//#[cfg(feature = "runtime-benchmarks")]
//pub mod benchmarking;

#[cfg(test)]
mod mock;
mod utils;

#[cfg(test)]
mod tests;

pub mod weights;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::traits::{Currency, Imbalance, ReservableCurrency};
	use orml_traits::{MultiCurrency, MultiReservableCurrency};
	use sp_runtime::traits::{CheckedAdd, CheckedSub, Zero};
	use sp_runtime::Permill;

	use primitives::estate::EstateInfo;
	use primitives::staking::{Bond, RoundInfo, StakeSnapshot};
	use primitives::{Balance, CurrencyId, RoundIndex, UndeployedLandBlockId};

	use crate::utils::{round_issuance_range, MintingRateInfo};

	use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Land treasury source
		#[pallet::constant]
		type LandTreasury: Get<PalletId>;

		/// Source of metaverse info
		type MetaverseInfoSource: MetaverseTrait<Self::AccountId>;

		/// Currency type
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		/// Multi currencies type that handles different currency type in auction
		type MultiCurrency: MultiReservableCurrency<Self::AccountId, CurrencyId = FungibleTokenId, Balance = Balance>;

		/// Weight implementation for estate extrinsics
		type WeightInfo: WeightInfo;

		/// Minimum staking balance
		#[pallet::constant]
		type MinimumStake: Get<BalanceOf<Self>>;

		/// Delay of staking reward payment (in number of rounds)
		#[pallet::constant]
		type RewardPaymentDelay: Get<u32>;

		/// NFT trait required for land and estate tokenization
		type NFTTokenizationSource: NFTTrait<Self::AccountId, BalanceOf<Self>, ClassId = ClassId, TokenId = TokenId>;

		/// Default max bound for each metaverse mapping system, this could change through proposal
		type DefaultMaxBound: Get<(i32, i32)>;

		/// Network fee charged when deploying a land block or creating an estate
		#[pallet::constant]
		type NetworkFee: Get<BalanceOf<Self>>;

		/// Storage deposit free charged when saving data into the blockchain.
		/// The fee will be unreserved after the storage is freed.
		#[pallet::constant]
		type StorageDepositFee: Get<BalanceOf<Self>>;

		/// Allows converting block numbers into balance
		type BlockNumberToBalance: Convert<Self::BlockNumber, BalanceOf<Self>>;
	}

	pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	pub type CurrencyIdOf<T> =
		<<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;

	#[pallet::storage]
	#[pallet::getter(fn fees)]
	pub type Fees<T: Config> = StorageValue<_, (Permill, Permill), ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn token_pool)]
	pub type TokenPool<T: Config> = StorageMap<_, Twox64Concat, (ClassId, CurrencyIdOf<T>), BalanceOf<T>, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub minting_rate_config: MintingRateInfo,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			GenesisConfig {
				minting_rate_config: Default::default(),
			}
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New staking round started [Starting Block, Round, Total Land Unit]
		NewRound(T::BlockNumber, RoundIndex, u64),
		/// New pool created
		PoolCreated(T::AccountId, ClassId, u32, CurrencyIdOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// No permission
		NoPermission,
		/// Currency is not supported
		CurrencyIsNotSupported,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::mint_land())]
		pub fn create_pool(origin: OriginFor<T>, max_supply: u32, currency_id: CurrencyIdOf<T>) -> DispatchResult {
			// Check if user signed
			// Emit events
			// Check if user signed
			let who = ensure_signed(origin)?;

			// Ensure currency_id supported
			ensure!(
				currency_id == FungibleTokenId::NativeToken(0) || currency_id == FungibleTokenId::NativeToken(1),
				Error::<T>::CurrencyIsNotSupported
			);

			// Collect pool creation fee
			Self::collect_pool_creation_fee(&who)?;

			// Create new NFT collection
			// This will return a unique collection ID for the new pool
			let class_id = T::NFTTokenizationSource.create_collection(who.clone(), max_supply, currency_id)?;

			// Add tuple class_id, currency_id
			TokenPool::<T>::insert((class_id, currency_id), Zero::zero);

			// Emit event for pool creation
			Self::deposit_event(Event::PoolCreated(who, class_id, max_supply, currency_id));
			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::mint_land())]
		pub fn deposit(origin: OriginFor<T>, class_id: ClassId, amount: BalanceOf<T>) -> DispatchResult {
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {}
