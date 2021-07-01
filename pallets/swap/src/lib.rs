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
use frame_support::{ensure, pallet_prelude::*, transactional};
use frame_system::{ensure_root, ensure_signed};
use primitives::{Balance, CountryId, CurrencyId, SocialTokenCurrencyId};
use sp_runtime::{traits::{AccountIdConversion, One}, DispatchError, ModuleId, RuntimeDebug, DispatchResult};
use bc_country::*;
use sp_std::vec::Vec;
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;


#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Status for TradingPair
#[derive(Clone, Copy, Encode, Decode, RuntimeDebug, PartialEq, Eq)]
pub enum TradingPairStatus {
    /// Default status,
    /// can withdraw liquidity, re-enable and list this trading pair.
    NotEnabled,
    /// TradingPair is Enabled,
    /// can add/remove liquidity, trading and disable this trading pair.
    Enabled,
}

impl Default for TradingPairStatus {
    fn default() -> Self {
        Self::NotEnabled
    }
}

pub use pallet::*;
use primitives::dex::{TradingPair, Price, Ratio};
use orml_traits::MultiCurrency;
use frame_support::sp_runtime::FixedPointNumber;
use frame_support::traits::{Currency, ExistenceRequirement};
use pallet_balances::Error::ExistentialDeposit;
use frame_support::sp_runtime::traits::{Zero, UniqueSaturatedInto};
use sp_core::U256;
use sp_core::sp_std::convert::TryInto;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use orml_traits::MultiCurrencyExtended;
    use frame_support::traits::Currency;
    use primitives::dex::TradingPair;
    use primitives::SocialTokenCurrencyId;

    #[pallet::pallet]
    #[pallet::generate_store(trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The DEX's module id, keep all assets in DEX.
        #[pallet::constant]
        type ModuleId: Get<ModuleId>;
        /// Social token currency system
        type SocialTokenCurrency: MultiCurrencyExtended<Self::AccountId, CurrencyId=CurrencyId, Balance=Balance>;
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
    pub type TradingPairStatuses<T: Config> =
    StorageMap<_, Twox64Concat, TradingPair, TradingPairStatus, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    #[pallet::metadata(T::AccountId = "AccountId")]
    pub enum Event<T: Config> {
        NewCountryCreated(CountryId),
        TransferredCountry(CountryId, T::AccountId, T::AccountId),
        /// Add liquidity success. \[who, currency_id_0, pool_0_increment,
        /// currency_id_1, pool_1_increment, share_increment\]
        AddLiquidity(T::AccountId, SocialTokenCurrencyId, Balance, SocialTokenCurrencyId, Balance, Balance),
        /// Remove liquidity from the trading pool success. \[who,
        /// currency_id_0, pool_0_decrement, currency_id_1, pool_1_decrement,
        /// share_decrement\]
        RemoveLiquidity(T::AccountId, SocialTokenCurrencyId, Balance, SocialTokenCurrencyId, Balance, Balance),
        /// Use supply currency to swap target currency. \[trader, trading_path,
        /// supply_currency_amount, target_currency_amount\]
        Swap(T::AccountId, Vec<SocialTokenCurrencyId>, Balance, Balance),
    }

    #[pallet::error]
    pub enum Error<T> {
        //Country Info not found
        CountryInfoNotFound,
        //Country Id not found
        CountryIdNotFound,
        //No permission
        NoPermission,
        //No available bitcountry id
        NoAvailableCountryId,
        //Social Token is not valid
        InvalidSocialTokenIds,
        //Trading pair is in NotEnabled status
        NotEnabledTradingPair,
        //Trading pair must be Enabled
        TradingPairMustBeEnabled,
        //Invalid Liquidity Input
        InvalidLiquidityIncrement,
        //Invalid trading currency
        InvalidTradingCurrency,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        #[transactional]
        pub fn add_liquidity(
            origin: OriginFor<T>,
            token_id_a: SocialTokenCurrencyId,
            token_id_b: SocialTokenCurrencyId,
            #[pallet::compact] max_amount_a: Balance,
            #[pallet::compact] max_amount_b: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let trading_pair = TradingPair::from_token_currency_ids(token_id_a, token_id_b)
                .ok_or(Error::<T>::InvalidSocialTokenIds)?;

            match Self::trading_pair_statuses(trading_pair) {
                TradingPairStatus::Enabled => Self::do_add_liquidity(
                    &who,
                    token_id_a,
                    token_id_b,
                    max_amount_a,
                    max_amount_b,
                ),
                TradingPairStatus::NotEnabled => Err(Error::<T>::NotEnabledTradingPair.into())
            }?;

            Ok(().into())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}

impl<T: Config> Pallet<T> {
    fn account_id() -> T::AccountId {
        T::ModuleId::get().into_account()
    }

    fn do_add_liquidity(
        who: &T::AccountId,
        currency_id_a: SocialTokenCurrencyId,
        currency_id_b: SocialTokenCurrencyId,
        max_amount_a: Balance,
        max_amount_b: Balance,
    ) -> DispatchResult {
        let trading_pair = TradingPair::new(currency_id_a, currency_id_b);
        let lp_share_social_token_id = trading_pair.get_dex_share_social_currency_id().ok_or(Error::<T>::InvalidSocialTokenIds)?;

        ensure!(
			matches!(
				Self::trading_pair_statuses(trading_pair),
				TradingPairStatus::<_, _>::Enabled
			),
			Error::<T>::TradingPairMustBeEnabled,
		);

        LiquidityPool::<T>::try_mutate(trading_pair, |(pool_0, pool_1)| -> DispatchResult{
            let total_shares = T::SocialTokenCurrency::total_issuance(lp_share_social_token_id);
            let (max_amount_0, max_amount_1) = if currency_id_a == trading_pair.0 {
                (max_amount_a, max_amount_b)
            } else {
                (max_amount_b, max_amount_a)
            };
            let (pool_0_increment, pool_1_increment, share_increment): (Balance, Balance, Balance) = (0, 0, 0);
            //First LP - Initial pool without any share token
            if total_shares.is_zero() {
                //Calculate share amount
                let share_amount = if max_amount_0 > max_amount_1 {
                    //Token 0 > Token 1
                    //Find the price token 1 of token 0
                    let initial_price_1_in_0 =
                        Price::checked_from_rational(max_amount_0, max_amount_1).unwrap_or_default();
                    initial_price_1_in_0
                        .saturating_mul_int(max_amount_1)
                        .saturating_add(max_amount_0)
                } else {
                    //Token 0 < Token 1
                    //Find the price token 0 of token 1
                    let initial_price_0_in_1: Price =
                        Price::checked_from_rational(max_amount_1, max_amount_0).unwrap_or_default();
                    initial_price_0_in_1
                        .saturating_mul_int(max_amount_0)
                        .saturating_add(max_amount_1)
                };

                (max_amount_0, max_amount_1, share_amount)
            } else { // pools already exists then adding to existing share pool
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
                T::NativeCurrency::transfer(who, &dex_module_account_id, pool_0_increment, ExistenceRequirement::KeepAlive)?;
            } else {
                T::SocialTokenCurrency::transfer(trading_pair.0, who, &dex_module_account_id, pool_0_increment)?;
            }

            if trading_pair.1.is_native_token_currency_id() {
                T::NativeCurrency::transfer(who, &dex_module_account_id, pool_1_increment, ExistentialDeposit::KeepAlive)?;
            } else {
                T::SocialTokenCurrency::transfer(trading_pair.1, who, &dex_module_account_id, pool_1_increment)?;
            }

            T::SocialTokenCurrency::deposit(lp_share_currency_id, who, share_increment)?;

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
        currency_id_a: SocialTokenCurrencyId,
        currency_id_b: SocialTokenCurrencyId,
        remove_share: Balance,
        by_withdraw: bool,
    ) -> DispatchResult {
        if remove_share.is_zero() {
            return Ok(());
        }
        let trading_pair = TradingPair::from_token_currency_ids(currency_id_a, currency_id_b).ok_or(Error::<T>::InvalidSocialTokenIds)?;
        let lp_share_currency_id = trading_pair
            .get_dex_share_social_currency_id()
            .ok_or(Error::<T>::InvalidSocialTokenIds);

        LiquidityPool::<T>::try_mutate(trading_pair, |(pool_0, pool_1)| -> DispatchResult{
            let total_shares = T::SocialTokenCurrency::total_issuance(lp_share_currency_id);
            let proportion = Ratio::checked_from_rational(remove_share, total_shares).unwrap_or_default();

            let pool_0_decrement = proportion.saturating_mul_int(*pool_0);
            let pool_1_decrement = proportion.saturating_mul_int(*pool_1);
            let dex_module_account_id = Self::account_id();

            T::SocialTokenCurrency::withdraw(&lp_share_currency_id, &who, remove_share)?;

            if trading_pair.0.is_native_token_currency_id() {
                T::NativeCurrency::transfer(&dex_module_account_id, &who, pool_0_increment, ExistenceRequirement::KeepAlive)?;
            } else {
                T::SocialTokenCurrency::transfer(trading_pair.0, &dex_module_account_id, who, pool_0_increment)?;
            }

            if trading_pair.1.is_native_token_currency_id() {
                T::NativeCurrency::transfer(&dex_module_account_id, &who, pool_1_increment, ExistentialDeposit::KeepAlive)?;
            } else {
                T::SocialTokenCurrency::transfer(trading_pair.1, &dex_module_account_id, who, pool_1_increment)?;
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

    /// Swap with exact supply
    fn swap_social_token_for_exact_NUUM(
        who: &T::AccountId,
        supply_currency: SocialTokenCurrencyId,
        supply_amount: Balance,
        target_currency: SocialTokenCurrencyId,
        min_target_amount: Balance,
    ) -> Result<Balance, DispatchError> {
        ensure!(target_currency.is_native_token_currency_id(), Error::<T>::InvalidTradingCurrency);

        ensure!(
            matches!(
                Self::trading_pair_statuses(TradingPair::new(supply_currency, target_currency)),
                TradingPairStatus::<_,_>::Enabled
            ),
            Error::<T>::TradingPairMustBeEnabled
        );
    }
}
