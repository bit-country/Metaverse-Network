// This file is part of Bit.Country
// The DEX logic influences by Acala DEX and Uniswap V2

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
use frame_support::ensure;
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
        /// Trading pair is in NotEnabled status
        NotEnabledTradingPair,
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
    fn do_add_liquidity(
        who: &T::AccountId,
        currency_id_a: SocialTokenCurrencyId,
        currency_id_b: SocialTokenCurrencyId,
        max_amount_a: Balance,
        max_amount_b: Balance,
    ) -> DispatchResult {
        Ok(())
    }
}
}