// This file is part of Bit.Country
// The DEX logic influences by Acala DEX and Uniswap V2 Smart contract

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

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::dispatch::DispatchResult;
use frame_support::pallet_prelude::*;
use frame_support::sp_runtime::traits::{UniqueSaturatedInto, Zero};
use frame_support::sp_runtime::FixedPointNumber;
use frame_support::traits::{Currency, ExistenceRequirement};
use frame_support::{ensure, transactional, PalletId};
use frame_system::ensure_signed;
use frame_system::pallet_prelude::*;
use orml_traits::MultiCurrency;
use scale_info::TypeInfo;
use sp_core::sp_std::convert::TryInto;
use sp_core::U256;
use sp_runtime::{traits::AccountIdConversion, DispatchError, RuntimeDebug};
use sp_std::vec;

use auction_manager::SwapManager;
pub use pallet::*;
use primitives::dex::{Price, Ratio, TradingPair};
use primitives::{Balance, FungibleTokenId, MetaverseId};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Status for TradingPair
#[derive(Clone, Copy, Encode, Decode, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub enum TradingPairStatus {
	/// can withdraw liquidity, re-enable and list this trading pair.
	NotEnabled,
	/// Default status,
	/// TradingPair is Enabled,
	/// can add/remove liquidity, trading and disable this trading pair.
	Enabled,
}

impl Default for TradingPairStatus {
	fn default() -> Self {
		Self::Enabled
	}
}

#[frame_support::pallet]
pub mod pallet {
	use frame_support::traits::Currency;
	use orml_traits::MultiCurrencyExtended;
	use sp_std::vec::Vec;

	use primitives::dex::TradingPair;
	use primitives::FungibleTokenId;

	use super::*;

	pub(super) type BalanceOf<T> =
		<<T as Config>::NativeCurrency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::pallet]
	#[pallet::generate_store(trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The DEX's module id, keep all assets in DEX.
		#[pallet::constant]
		type PalletId: Get<PalletId>;
		/// Social token currency system
		type FungibleTokenCurrency: MultiCurrencyExtended<
			Self::AccountId,
			CurrencyId = FungibleTokenId,
			Balance = Balance,
		>;
		/// Native currency system
		type NativeCurrency: Currency<Self::AccountId>;
		/// Exchange fee
		type GetSwapFee: Get<(u32, u32)>;
	}

	#[pallet::storage]
	#[pallet::getter(fn liquidity_pool)]
	pub type LiquidityPool<T: Config> = StorageMap<_, Twox64Concat, TradingPair, (Balance, Balance), ValueQuery>;

	/// Status for TradingPair.
	#[pallet::storage]
	#[pallet::getter(fn trading_pair_statuses)]
	pub type TradingPairStatuses<T: Config> = StorageMap<_, Twox64Concat, TradingPair, TradingPairStatus, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (crate) fn deposit_event)]
	pub enum Event<T: Config> {
		NewCountryCreated(MetaverseId),
		TransferredCountry(MetaverseId, T::AccountId, T::AccountId),
		/// Add liquidity success. \[who, currency_id_0, pool_0_increment,
		/// currency_id_1, pool_1_increment, share_increment\]
		AddLiquidity(
			T::AccountId,
			FungibleTokenId,
			Balance,
			FungibleTokenId,
			Balance,
			Balance,
		),
		/// Remove liquidity from the trading pool success. \[who,
		/// currency_id_0, pool_0_decrement, currency_id_1, pool_1_decrement,
		/// share_decrement\]
		RemoveLiquidity(
			T::AccountId,
			FungibleTokenId,
			Balance,
			FungibleTokenId,
			Balance,
			Balance,
		),
		/// Use supply currency to swap target currency. \[trader, trading_path,
		/// supply_currency_amount, target_currency_amount\]
		Swap(T::AccountId, Vec<FungibleTokenId>, Balance, Balance),
	}

	#[pallet::error]
	pub enum Error<T> {
		//Country Info not found
		CountryInfoNotFound,
		//Country Id not found
		CountryIdNotFound,
		//No permission
		NoPermission,
		//No available metaverse id
		NoAvailableCountryId,
		//Social Token is not valid
		InvalidFungibleTokenIds,
		//Trading pair is in NotEnabled status
		NotEnabledTradingPair,
		//Trading pair must be Enabled
		TradingPairMustBeEnabled,
		//Invalid Liquidity Input
		InvalidLiquidityIncrement,
		//Invalid trading currency
		InvalidTradingCurrency,
		//Insufficient Liquidity in the pool
		InsufficientLiquidity,
		//Insufficient Target Amount
		InsufficientTargetAmount,
		//Too much Supply Amount
		TooMuchSupplyAmount,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		#[transactional]
		pub fn add_liquidity(
			origin: OriginFor<T>,
			token_id_a: FungibleTokenId,
			token_id_b: FungibleTokenId,
			#[pallet::compact] max_amount_a: Balance,
			#[pallet::compact] max_amount_b: Balance,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let trading_pair = TradingPair::from_token_currency_ids(token_id_a, token_id_b)
				.ok_or(Error::<T>::InvalidFungibleTokenIds)?;

			match Self::trading_pair_statuses(trading_pair) {
				TradingPairStatus::Enabled => {
					Self::do_add_liquidity(&who, token_id_a, token_id_b, max_amount_a, max_amount_b)
				}
				TradingPairStatus::NotEnabled => Err(Error::<T>::NotEnabledTradingPair.into()),
			}?;

			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		#[transactional]
		pub fn remove_liquidity(
			origin: OriginFor<T>,
			currency_id_1: FungibleTokenId,
			currency_id_2: FungibleTokenId,
			remove_amount: Balance,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			Self::do_remove_liquidity(&who, currency_id_1, currency_id_2, remove_amount)?;
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		#[transactional]
		pub fn swap_native_token_with_exact_supply(
			origin: OriginFor<T>,
			supply_currency: FungibleTokenId,
			target_currency: FungibleTokenId,
			#[pallet::compact] supply_amount: Balance,
			#[pallet::compact] min_target_amount: Balance,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let _ = Self::do_swap_native_token_for_social_token(
				&who,
				supply_currency,
				supply_amount,
				target_currency,
				min_target_amount,
			)?;
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		#[transactional]
		pub fn swap_social_token_with_exact_native_token(
			origin: OriginFor<T>,
			supply_currency: FungibleTokenId,
			target_currency: FungibleTokenId,
			#[pallet::compact] target_amount: Balance,
			#[pallet::compact] max_supply_amount: Balance,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let _ = Self::do_swap_token_for_exact_native_token(
				&who,
				supply_currency,
				target_currency,
				target_amount,
				max_supply_amount,
			)?;
			Ok(().into())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}

impl<T: Config> Pallet<T> {
	fn account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}

	fn do_add_liquidity(
		who: &T::AccountId,
		currency_id_a: FungibleTokenId,
		currency_id_b: FungibleTokenId,
		max_amount_a: Balance,
		max_amount_b: Balance,
	) -> DispatchResult {
		let trading_pair = TradingPair::new(currency_id_a, currency_id_b);
		let lp_share_social_token_id = trading_pair
			.get_dex_share_social_currency_id()
			.ok_or(Error::<T>::InvalidFungibleTokenIds)?;

		ensure!(
			matches!(Self::trading_pair_statuses(trading_pair), TradingPairStatus::Enabled),
			Error::<T>::TradingPairMustBeEnabled,
		);

		LiquidityPool::<T>::try_mutate(trading_pair, |(pool_0, pool_1)| -> DispatchResult {
			let total_shares = T::FungibleTokenCurrency::total_issuance(lp_share_social_token_id);
			let (max_amount_0, max_amount_1) = if currency_id_a == trading_pair.0 {
				(max_amount_a, max_amount_b)
			} else {
				(max_amount_b, max_amount_a)
			};
			let (pool_0_increment, pool_1_increment, share_increment): (Balance, Balance, Balance) =
                // First LP - Initial pool without any share token
                if total_shares.is_zero() {
                    // Calculate share amount
                    let share_amount = if max_amount_0 > max_amount_1 {
                        // Token 0 > Token 1
                        // Find the price token 1 of token 0
                        let initial_price_1_in_0 =
                            Price::checked_from_rational(max_amount_0, max_amount_1).unwrap_or_default();
                        initial_price_1_in_0
                            .saturating_mul_int(max_amount_1)
                            .saturating_add(max_amount_0)
                    } else {
                        // Token 0 < Token 1
                        // Find the price token 0 of token 1
                        let initial_price_0_in_1: Price =
                            Price::checked_from_rational(max_amount_1, max_amount_0).unwrap_or_default();
                        initial_price_0_in_1
                            .saturating_mul_int(max_amount_0)
                            .saturating_add(max_amount_1)
                    };

                    (max_amount_0, max_amount_1, share_amount)
                } else {// pools already exists then adding to existing share pool
                    let price_0_1 = Price::checked_from_rational(*pool_1, *pool_0).unwrap_or_default();
                    let input_price_0_1 = Price::checked_from_rational(max_amount_1, max_amount_0).unwrap_or_default();
                    // input_price_0_1 is more than actual price 0 1 in the pool, calculate the actual amount 0
                    if input_price_0_1 <= price_0_1 {
                        // existing price 1 / 0 of the pool
                        let price_1_0 = Price::checked_from_rational(*pool_0, *pool_1).unwrap_or_default();
                        let amount_0 = price_1_0.saturating_mul_int(max_amount_1);
                        let share_increment = Ratio::checked_from_rational(amount_0, *pool_0)
                            .and_then(|n| n.checked_mul_int(total_shares))
                            .unwrap_or_default();
                        (amount_0, max_amount_1, share_increment)
                    } else {
                        // existing price 0 / 1 of the pool
                        // input_price_1_0 is more than actual price 1 0 in the pool, calculate the actual amount 1
                        let amount_1 = price_0_1.saturating_mul_int(max_amount_0);
                        let share_increment = Ratio::checked_from_rational(amount_1, *pool_1)
                            .and_then(|n| n.checked_mul_int(total_shares))
                            .unwrap_or_default();
                        (max_amount_0, amount_1, share_increment)
                    }
                };

			ensure!(
				!share_increment.is_zero() && !pool_0_increment.is_zero() && !pool_1_increment.is_zero(),
				Error::<T>::InvalidLiquidityIncrement,
			);

			let dex_module_account_id = Self::account_id();
			if trading_pair.0.is_native_token_currency_id() {
				let pool_0_incr_balance: BalanceOf<T> =
					TryInto::<BalanceOf<T>>::try_into(pool_0_increment).unwrap_or_default();
				T::NativeCurrency::transfer(
					who,
					&dex_module_account_id,
					pool_0_incr_balance,
					ExistenceRequirement::AllowDeath,
				)?;
			} else {
				T::FungibleTokenCurrency::transfer(trading_pair.0, who, &dex_module_account_id, pool_0_increment)?;
			}

			if trading_pair.1.is_native_token_currency_id() {
				let pool_1_incr_balance: BalanceOf<T> =
					TryInto::<BalanceOf<T>>::try_into(pool_1_increment).unwrap_or_default();
				T::NativeCurrency::transfer(
					who,
					&dex_module_account_id,
					pool_1_incr_balance,
					ExistenceRequirement::AllowDeath,
				)?;
			} else {
				T::FungibleTokenCurrency::transfer(trading_pair.1, who, &dex_module_account_id, pool_1_increment)?;
			}

			T::FungibleTokenCurrency::deposit(lp_share_social_token_id, who, share_increment)?;

			*pool_0 = pool_0.saturating_add(pool_0_increment);
			*pool_1 = pool_1.saturating_add(pool_1_increment);

			Self::deposit_event(Event::AddLiquidity(
				who.clone(),
				trading_pair.0,
				pool_0_increment,
				trading_pair.1,
				pool_1_increment,
				share_increment,
			));

			Ok(())
		})
	}

	#[transactional]
	fn do_remove_liquidity(
		who: &T::AccountId,
		currency_id_a: FungibleTokenId,
		currency_id_b: FungibleTokenId,
		remove_share: Balance,
	) -> DispatchResult {
		if remove_share.is_zero() {
			return Ok(());
		}
		let trading_pair = TradingPair::from_token_currency_ids(currency_id_a, currency_id_b)
			.ok_or(Error::<T>::InvalidFungibleTokenIds)?;
		let lp_share_currency_id = trading_pair
			.get_dex_share_social_currency_id()
			.ok_or(Error::<T>::InvalidFungibleTokenIds)?;

		LiquidityPool::<T>::try_mutate(trading_pair, |(pool_0, pool_1)| -> DispatchResult {
			let total_shares = T::FungibleTokenCurrency::total_issuance(lp_share_currency_id);
			let proportion = Ratio::checked_from_rational(remove_share, total_shares).unwrap_or_default();

			let pool_0_decrement = proportion.saturating_mul_int(*pool_0);
			let pool_1_decrement = proportion.saturating_mul_int(*pool_1);
			let dex_module_account_id = Self::account_id();

			T::FungibleTokenCurrency::withdraw(lp_share_currency_id, &who, remove_share)?;

			if trading_pair.0.is_native_token_currency_id() {
				let pool_0_decr_balance: BalanceOf<T> =
					TryInto::<BalanceOf<T>>::try_into(pool_0_decrement).unwrap_or_default();
				T::NativeCurrency::transfer(
					&dex_module_account_id,
					&who,
					pool_0_decr_balance,
					ExistenceRequirement::KeepAlive,
				)?;
			} else {
				T::FungibleTokenCurrency::transfer(trading_pair.0, &dex_module_account_id, who, pool_0_decrement)?;
			}

			if trading_pair.1.is_native_token_currency_id() {
				let pool_1_decr_balance: BalanceOf<T> =
					TryInto::<BalanceOf<T>>::try_into(pool_1_decrement).unwrap_or_default();
				T::NativeCurrency::transfer(
					&dex_module_account_id,
					&who,
					pool_1_decr_balance,
					ExistenceRequirement::KeepAlive,
				)?;
			} else {
				T::FungibleTokenCurrency::transfer(trading_pair.1, &dex_module_account_id, who, pool_1_decrement)?;
			}

			*pool_0 = pool_0.saturating_sub(pool_0_decrement);
			*pool_1 = pool_1.saturating_sub(pool_1_decrement);

			Self::deposit_event(Event::RemoveLiquidity(
				who.clone(),
				trading_pair.0,
				pool_0_decrement,
				trading_pair.1,
				pool_1_decrement,
				remove_share,
			));

			Ok(())
		})
	}

	/// Get how much target amount will be got for specific supply amount
	/// and price impact
	fn get_amount_out(supply_pool: Balance, target_pool: Balance, supply_amount: Balance) -> Balance {
		if supply_amount.is_zero() || supply_pool.is_zero() || target_pool.is_zero() {
			Zero::zero()
		} else {
			let (fee_numerator, fee_denominator) = T::GetSwapFee::get();
			let supply_amount_with_fee =
				supply_amount.saturating_mul(fee_denominator.saturating_sub(fee_numerator).unique_saturated_into());
			let numerator: U256 = U256::from(supply_amount_with_fee).saturating_mul(U256::from(target_pool));
			let denominator: U256 = U256::from(supply_pool)
				.saturating_mul(U256::from(fee_denominator))
				.saturating_add(U256::from(supply_amount_with_fee));

			numerator
				.checked_div(denominator)
				.and_then(|n| TryInto::<Balance>::try_into(n).ok())
				.unwrap_or_else(Zero::zero)
		}
	}

	/// Get how much supply amount will be paid for specific target amount.
	fn get_amount_in(supply_pool: Balance, target_pool: Balance, target_amount: Balance) -> Balance {
		if target_amount.is_zero() || supply_pool.is_zero() || target_pool.is_zero() {
			Zero::zero()
		} else {
			let (fee_numerator, fee_denominator) = T::GetSwapFee::get();
			let numerator: U256 = U256::from(supply_pool)
				.saturating_mul(U256::from(target_amount))
				.saturating_mul(U256::from(fee_denominator));
			let denominator: U256 = U256::from(target_pool)
				.saturating_sub(U256::from(target_amount))
				.saturating_mul(U256::from(fee_denominator.saturating_sub(fee_numerator)));

			numerator
				.checked_div(denominator)
				.and_then(|r| r.checked_add(U256::one())) // add 1 to result so that correct the possible losses caused by remainder discarding in
				.and_then(|n| TryInto::<Balance>::try_into(n).ok())
				.unwrap_or_else(Zero::zero)
		}
	}

	fn get_liquidity(currency_id_a: FungibleTokenId, currency_id_b: FungibleTokenId) -> (Balance, Balance) {
		let trading_pair = TradingPair::new(currency_id_a, currency_id_b);
		let (pool_0, pool_1) = Self::liquidity_pool(trading_pair);
		if currency_id_a == trading_pair.0 {
			(pool_0, pool_1)
		} else {
			(pool_1, pool_0)
		}
	}

	/// Swap native token for social token
	/// Exact native token in, social token out
	#[transactional]
	fn do_swap_native_token_for_social_token(
		who: &T::AccountId,
		supply_currency: FungibleTokenId,
		amount_in: Balance,
		target_currency: FungibleTokenId,
		amount_out_min: Balance,
	) -> Result<Balance, DispatchError> {
		ensure!(
			supply_currency.is_native_token_currency_id(),
			Error::<T>::InvalidTradingCurrency
		);
		ensure!(
			target_currency.is_social_token_currency_id(),
			Error::<T>::InvalidTradingCurrency
		);

		ensure!(
			matches!(
				Self::trading_pair_statuses(TradingPair::new(supply_currency, target_currency)),
				TradingPairStatus::Enabled
			),
			Error::<T>::TradingPairMustBeEnabled
		);

		let (supply_pool, target_pool) = Self::get_liquidity(supply_currency, target_currency);
		ensure!(
			!supply_pool.is_zero() && !target_pool.is_zero(),
			Error::<T>::InsufficientLiquidity
		);

		let social_token_out = Self::get_amount_out(supply_pool, target_pool, amount_in);
		ensure!(!social_token_out.is_zero(), Error::<T>::InsufficientLiquidity);

		ensure!(social_token_out >= amount_out_min, Error::<T>::InsufficientTargetAmount);

		let dex_module_account_id = Self::account_id();

		// Transfer native token in
		let native_token_amount_in_balance: BalanceOf<T> =
			TryInto::<BalanceOf<T>>::try_into(amount_in).unwrap_or_default();
		T::NativeCurrency::transfer(
			who,
			&dex_module_account_id,
			native_token_amount_in_balance,
			ExistenceRequirement::KeepAlive,
		)?;

		Self::_swap(supply_currency, target_currency, amount_in, social_token_out);

		// Transfer out the social token
		T::FungibleTokenCurrency::transfer(target_currency, &dex_module_account_id, who, social_token_out)?;

		Self::deposit_event(Event::Swap(
			who.clone(),
			vec![supply_currency, target_currency],
			amount_in,
			social_token_out,
		));

		Ok(social_token_out)
	}

	/// Swap social token with exact target native token
	/// Social token in, Exact native token out
	#[transactional]
	fn do_swap_token_for_exact_native_token(
		who: &T::AccountId,
		supply_currency: FungibleTokenId,
		target_currency: FungibleTokenId,
		amount_out: Balance,
		amount_in_max: Balance,
	) -> Result<Balance, DispatchError> {
		ensure!(
			supply_currency.is_social_token_currency_id(),
			Error::<T>::InvalidTradingCurrency
		);
		ensure!(
			target_currency.is_native_token_currency_id(),
			Error::<T>::InvalidTradingCurrency
		);

		ensure!(
			matches!(
				Self::trading_pair_statuses(TradingPair::new(supply_currency, target_currency)),
				TradingPairStatus::Enabled
			),
			Error::<T>::TradingPairMustBeEnabled
		);

		let (supply_pool, target_pool) = Self::get_liquidity(supply_currency, target_currency);
		ensure!(
			!supply_pool.is_zero() && !target_pool.is_zero(),
			Error::<T>::InsufficientLiquidity
		);
		let supply_amount_in = Self::get_amount_in(supply_pool, target_pool, amount_out);
		ensure!(!supply_amount_in.is_zero(), Error::<T>::InsufficientLiquidity);

		ensure!(supply_amount_in <= amount_in_max, Error::<T>::TooMuchSupplyAmount);
		let dex_module_account_id = Self::account_id();

		T::FungibleTokenCurrency::transfer(supply_currency, &who, &dex_module_account_id, supply_amount_in)?;
		Self::_swap(supply_currency, target_currency, supply_amount_in, amount_out);

		let amount_out_balance: BalanceOf<T> = TryInto::<BalanceOf<T>>::try_into(amount_out).unwrap_or_default();
		T::NativeCurrency::transfer(
			&dex_module_account_id,
			&who,
			amount_out_balance,
			ExistenceRequirement::KeepAlive,
		)?;

		Self::deposit_event(Event::Swap(
			who.clone(),
			vec![supply_currency, target_currency],
			supply_amount_in,
			amount_out,
		));

		Ok(supply_amount_in)
	}

	fn _swap(
		supply_currency_id: FungibleTokenId,
		target_currency_id: FungibleTokenId,
		supply_incr: Balance,
		target_decr: Balance,
	) {
		if let Some(trading_pair) = TradingPair::from_token_currency_ids(supply_currency_id, target_currency_id) {
			LiquidityPool::<T>::mutate(trading_pair, |(pool_0, pool_1)| {
				if supply_currency_id == trading_pair.0 {
					*pool_0 = pool_0.saturating_add(supply_incr);
					*pool_1 = pool_1.saturating_sub(target_decr);
				} else {
					*pool_0 = pool_0.saturating_sub(target_decr);
					*pool_1 = pool_1.saturating_add(supply_incr);
				}
			});
		}
	}
}

impl<T: Config> SwapManager<T::AccountId, FungibleTokenId, Balance> for Pallet<T> {
	fn add_liquidity(
		who: &T::AccountId,
		token_id_a: FungibleTokenId,
		token_id_b: FungibleTokenId,
		max_amount_a: Balance,
		max_amount_b: Balance,
	) -> DispatchResult {
		Self::do_add_liquidity(who, token_id_a, token_id_b, max_amount_a, max_amount_b)?;
		Ok(())
	}
}
