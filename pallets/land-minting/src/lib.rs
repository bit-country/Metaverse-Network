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
	use primitives::{AccountId, Balance, CurrencyId, PoolId, RoundIndex, UndeployedLandBlockId};

	use crate::utils::{round_issuance_range, MintingRateInfo, PoolInfo};

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
	#[pallet::getter(fn next_class_id)]
	pub type NextPoolId<T: Config> = StorageValue<_, PoolId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn fees)]
	pub type Fees<T: Config> = StorageValue<_, (Permill, Permill), ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn token_pool)]
	pub type Pool<T: Config> = StorageMap<_, Twox64Concat, PoolId, PoolInfo<CurrencyIdOf<T>, T::AccountId>, ValueQuery>;

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
		PoolCreated(T::AccountId, u32, CurrencyIdOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// No permission
		NoPermission,
		/// Currency is not supported
		CurrencyIsNotSupported,
		/// No available next pool id
		NoAvailablePoolId,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::mint_land())]
		pub fn create_pool(
			origin: OriginFor<T>,
			currency_id: CurrencyIdOf<T>,
			max_nft_reward: u32,
			commission: Permill,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Ensure currency_id supported
			ensure!(
				currency_id == FungibleTokenId::NativeToken(0) || currency_id == FungibleTokenId::NativeToken(1),
				Error::<T>::CurrencyIsNotSupported
			);

			// TODO Check commission below threshold

			// Collect pool creation fee
			Self::collect_pool_creation_fee(&who)?;

			// Next pool id
			let next_pool_id = NextPoolId::<T>::try_mutate(|id| -> Result<PoolId, DispatchError> {
				let current_id = *id;
				*id = id.checked_add(&1u32).ok_or(Error::<T>::NoAvailablePoolId)?;
				Ok(current_id)
			})?;

			let new_pool = PoolInfo {
				creator: who.clone(),
				commission: commission,
				currency_id: currency_id,
				max: max_nft_reward,
			};

			// Add tuple class_id, currency_id
			Pool::<T>::insert(next_pool_id, new_pool);

			// Emit event for pool creation
			Self::deposit_event(Event::PoolCreated(who, max_nft_reward, currency_id));
			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::mint_land())]
		pub fn deposit(origin: OriginFor<T>, class_id: ClassId, amount: BalanceOf<T>) -> DispatchResult {
			// Ensure user is signed
			// Check if pool is full from max supply
			// Calculate exchange rate and take fee
			// Mint new token
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {}
