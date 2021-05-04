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

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use country::{Countries, CountryOwner, CountryTresury};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure,
    traits::{Get, IsType, Randomness},
    Parameter, PalletId,
};
use frame_system::{self as system, ensure_signed};
use orml_traits::{
    arithmetic::{Signed, SimpleArithmetic},
    BalanceStatus, BasicCurrency, BasicCurrencyExtended, BasicLockableCurrency,
    BasicReservableCurrency, LockIdentifier, MultiCurrency, MultiCurrencyExtended,
    MultiLockableCurrency, MultiReservableCurrency,
};
use primitives::{Balance, CountryCurrencyId, CountryId, CurrencyId, TokenSymbol};
use sp_runtime::{
    traits::{
        AccountIdConversion, AtLeast32Bit, AtLeast32BitUnsigned, Hash, Member, One, StaticLookup,
        Zero,
    },
    DispatchError, DispatchResult,
};
use sp_std::vec::Vec;
use unique_asset::AssetId;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type CurrencyIdOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<
    <T as frame_system::Config>::AccountId,
>>::CurrencyId;

/// The module configuration trait.
pub trait Config: system::Config + country::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;
    /// The arithmetic type of asset identifier.
    type TokenId: Parameter + AtLeast32Bit + Default + Copy;
    type MultiCurrency: MultiCurrencyExtended<Self::AccountId, CurrencyId=CurrencyId>
    + MultiLockableCurrency<Self::AccountId, CurrencyId=CurrencyId>
    + MultiReservableCurrency<Self::AccountId, CurrencyId=CurrencyId>;
    type CountryCurrency: MultiCurrencyExtended<
        Self::AccountId,
        CurrencyId=CurrencyId,
        Balance=Balance,
    >;
}

/// A wrapper for a token name.
pub type TokenName = Vec<u8>;
/// A wrapper for a ticker name.
pub type Ticker = Vec<u8>;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Token<Balance> {
    pub ticker: Ticker,
    pub total_supply: Balance,
}

decl_storage! {
    trait Store for Module<T: Config> as CountryTokens {
        CountryTokens get(fn get_country_token): map hasher(blake2_128_concat) CountryId => Option<CountryCurrencyId>;
        /// The next asset identifier up for grabs.
        NextTokenId get(fn next_token_id): CountryCurrencyId;
        /// Details of the token corresponding to the token id.
        /// (hash) -> Token details [returns Token struct]
        Tokens get(fn token_details): map hasher(blake2_128_concat) CountryCurrencyId => Token<Balance>;
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        /// Transfer amount should be non-zero
        AmountZero,
        /// Account balance must be greater than or equal to the transfer amount
        BalanceLow,
        /// Balance should be non-zero
        BalanceZero,
        ///Insufficient balance
        InsufficientBalance,
        /// No permission to issue token
        NoPermissionTokenIssuance,
        /// Country Currency already issued for this country
        TokenAlreadyIssued,
        /// No available next token id
        NoAvailableTokenId,
        //Country Is Not Available
        CountryFundIsNotAvailable
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;
        /// Issue a new class of fungible assets for country. There are, and will only ever be, `total`
        /// such assets and they'll all belong to the `origin` initially. It will have an
        /// identifier `TokenId` instance: this will be specified in the `Issued` event.
        #[weight = 10_000]
        fn mint_token(origin, ticker: Ticker, country_id: CountryId, total_supply: Balance) -> DispatchResult{
            let who = ensure_signed(origin)?;
            //Check ownership
            ensure!(<CountryOwner<T>>::contains_key(&country_id, &who), Error::<T>::NoPermissionTokenIssuance);

            //Generate new CurrencyId
            let currency_id = NextTokenId::mutate(|id| -> Result<CountryCurrencyId, DispatchError>{

                let current_id =*id;
                *id = id.checked_add(One::one())
                .ok_or(Error::<T>::NoAvailableTokenId)?;

                Ok(current_id)
            })?;

            let token_info = Token{
                ticker,
                total_supply,
            };

            let native_currency = CurrencyId::NUUM;

            Tokens::insert(currency_id, token_info);
            CountryTokens::insert(country_id, currency_id);

            T::CountryCurrency::deposit(native_currency, &who, total_supply)?;

            Self::deposit_event(RawEvent::Issued(who, total_supply));

            Ok(())
        }

        #[weight = 10_000]
        fn transfer(
            origin,
            dest: <T::Lookup as StaticLookup>::Source,
            currency_id: CurrencyIdOf<T>,
            #[compact] amount: Balance
        ) {

            let from = ensure_signed(origin)?;
            let to = T::Lookup::lookup(dest)?;
            Self::transfer_from(currency_id, &from, &to, amount)?;
        }
    }
}

decl_event! {
    pub enum Event<T> where
        <T as system::Config>::AccountId,
        Balance = Balance,
        CurrencyId = CurrencyId
    {
        /// Some assets were issued. \[asset_id, owner, total_supply\]
        Issued(AccountId, Balance),
        /// Some assets were transferred. \[asset_id, from, to, amount\]
        Transferred(CurrencyId, AccountId, AccountId, Balance),
        /// Some assets were destroyed. \[asset_id, owner, balance\]
        Destroyed(CurrencyId, AccountId, Balance),
    }
}

impl<T: Config> Module<T> {
    fn total_issuance(currency_id: CurrencyIdOf<T>) -> Balance {
        T::CountryCurrency::total_issuance(currency_id)
    }

    fn transfer_from(
        currency_id: CurrencyIdOf<T>,
        from: &T::AccountId,
        to: &T::AccountId,
        amount: Balance,
    ) -> DispatchResult {
        if amount.is_zero() || from == to {
            return Ok(());
        }

        T::CountryCurrency::transfer(currency_id, from, to, amount)?;

        Self::deposit_event(RawEvent::Transferred(
            currency_id,
            from.clone(),
            to.clone(),
            amount,
        ));
        Ok(())
    }

    pub fn get_total_issuance(country_id: CountryId) -> Result<Balance, DispatchError> {
        let country_fund =
            <CountryTresury<T>>::get(country_id).ok_or(Error::<T>::CountryFundIsNotAvailable)?;
        let total_issuance = T::CountryCurrency::total_issuance(country_fund.currency_id);

        Ok((total_issuance))
    }

    fn get_country_fund_id(country_id: CountryId) -> T::AccountId {
        T::PalletId::get().into_sub_account(country_id)
    }
}
