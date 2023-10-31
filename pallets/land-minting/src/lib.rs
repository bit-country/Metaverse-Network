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
	use sp_core::U256;
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

		#[pallet::constant]
		type PoolAccount: Get<PalletId>;
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

	/// Keep track of Pool detail
	#[pallet::storage]
	#[pallet::getter(fn pool)]
	pub type Pool<T: Config> = StorageMap<_, Twox64Concat, PoolId, PoolInfo<CurrencyIdOf<T>, T::AccountId>, ValueQuery>;

	/// Pool ledger that keeps track of Pool id and balance
	#[pallet::storage]
	#[pallet::getter(fn pool_ledger)]
	pub type PoolLedger<T: Config> = StorageMap<_, Twox64Concat, PoolId, BalanceOf<T>, ValueQuery>;

	/// Network ledger
	#[pallet::storage]
	#[pallet::getter(fn network_ledger)]
	pub type NetworkLedger<T: Config> = StorageMap<_, Twox64Concat, CurrencyIdOf<T>, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn minimum_redeem)]
	pub type MinimumRedeem<T: Config> = StorageMap<_, Twox64Concat, CurrencyIdOf<T>, BalanceOf<T>, ValueQuery>;

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
		/// Pool doesn't exists
		PoolDoesNotExist,
		/// Overflow
		Overflow,
		/// Below minimum redemption
		BelowMinimumRedeem,
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
				*id = id.checked_add(1u32).ok_or(Error::<T>::NoAvailablePoolId)?;
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
		pub fn deposit(origin: OriginFor<T>, pool_id: PoolId, amount: BalanceOf<T>) -> DispatchResult {
			// Ensure user is signed
			let who = ensure_signed(origin)?;
			// Check if pool exists
			let pool_instance = Pool::<T>::get(pool_id).ok_or(Error::<T>::PoolDoesNotExist)?;

			// Get currencyId from pool detail
			let currency_id = pool_instance.currency_id;

			// Get network ledger balance from currency id
			let network_ledger_balance = Self::network_ledger(currency_id);

			// Collect deposit fee for protocol
			// Assuming there's a function `collect_deposit_fee` that deducts a fee for deposits.
			let amount_after_fee = Self::collect_deposit_fee(&who, amount)?;

			let v_currency_id = T::CurrencyIdManagement::convert_to_vcurrency(currency_id)
				.map_err(|_| Error::<T>::CurrencyIsNotSupported)?;
			// Calculate vAmount as receipt of amount locked. The formula based on vAmount = (amount * vAmount
			// total issuance)/network ledger balance
			let v_amount_total_issuance = T::MultiCurrency::total_issuance(v_currency_id);
			let v_amount = U256::from(amount_after_fee.saturated_into::<u128>())
				.saturating_mul(v_amount_total_issuance.saturated_into::<u128>().into())
				.checked_div(network_ledger_balance.saturated_into::<u128>().into())
				.ok_or(ArithmeticError::Overflow)?
				.as_u128()
				.saturated_into();

			// Deposit vAmount to user using T::MultiCurrency::deposit
			T::MultiCurrency::deposit(currency_id, &who, v_amount)?;

			// Transfer amount to PoolAccount using T::MultiCurrency::transfer
			// Assuming `PoolAccount` is an associated type that represents the pool's account ID or a method to
			// get it.
			T::MultiCurrency::transfer(
				currency_id,
				&who,
				&T::PoolAccount::get().into_account_truncating(),
				amount,
			)?;

			// Emit deposit event
			Self::deposit_event(Event::Deposited(who, pool_id, amount));
			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::mint_land())]
		pub fn redeem(
			origin: OriginFor<T>,
			pool_id: PoolId,
			vcurrency_id: CurrencyIdOf<T>,
			vamount: BalanceOf<T>,
		) -> DispatchResult {
			// Ensure user is signed
			let who = ensure_signed(origin)?;
			ensure!(
				vamount >= MinimumRedeem::<T>::get(vcurrency_id),
				Error::<T>::BelowMinimumRedeem
			);

			let currency_id = T::CurrencyIdManagement::convert_to_currency(vcurrency_id)
				.map_err(|_| Error::<T>::NotSupportTokenType)?;

			// Check if pool exists
			let pool_instance = Pool::<T>::get(pool_id).ok_or(Error::<T>::PoolDoesNotExist)?;

			ensure!(
				currency_id == pool_instance.currency_id,
				Error::<T>::CurrencyIsNotSupported
			);

			// Get network ledger balance from currency id
			let network_ledger_balance = Self::network_ledger(currency_id);

			// Collect deposit fee for protocol
			// Assuming there's a function `collect_deposit_fee` that deducts a fee for deposits.
			let amount_after_fee = Self::collect_deposit_fee(&who, vamount)?;
			let vamount = vamount
				.checked_sub(&amount_after_fee)
				.ok_or(ArithmeticError::Overflow)?;
			// Calculate vAmount as receipt of amount locked. The formula based on vAmount = (amount * vAmount
			// total issuance)/network ledger balance
			let v_amount_total_issuance = T::MultiCurrency::total_issuance(vcurrency_id);
			let currency_amount = U256::from(vamount.saturated_into::<u128>())
				.saturating_mul(network_ledger_balance.saturated_into::<u128>().into())
				.checked_div(v_amount_total_issuance.saturated_into::<u128>().into())
				.ok_or(Error::<T>::CalculationOverflow)?
				.as_u128()
				.saturated_into();

			// Emit deposit event
			Self::deposit_event(Event::Deposited(who, pool_id, vamount));
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {}
