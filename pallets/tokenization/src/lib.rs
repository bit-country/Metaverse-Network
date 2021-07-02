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
use frame_support::{
    dispatch::{DispatchResultWithPostInfo, DispatchResult},
    decl_error, decl_event, decl_module, decl_storage, ensure, Parameter,
    pallet_prelude::*
};
use frame_system::{self as system, ensure_signed};
use orml_traits::{MultiCurrency, MultiCurrencyExtended};
use primitives::{Balance, CountryId, CurrencyId};
use sp_runtime::{
    traits::{AtLeast32Bit, One, StaticLookup, Zero, AccountIdConversion},
    DispatchError
};
use sp_std::vec::Vec;
use frame_support::sp_runtime::ModuleId;
use bc_country::*;
use frame_support::traits::{Get, Currency};
use frame_system::pallet_prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// A wrapper for a token name.
pub type TokenName = Vec<u8>;
    
/// A wrapper for a ticker name.
pub type Ticker = Vec<u8>;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Token<Balance> {
    pub ticker: Ticker,
    pub total_supply: Balance,
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::config]
    pub trait Config: frame_system::Config {

        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        
        /// The arithmetic type of asset identifier.
        type TokenId: Parameter + AtLeast32Bit + Default + Copy;
        
        type CountryCurrency: MultiCurrencyExtended<
            Self::AccountId,
            CurrencyId=CurrencyId,
            Balance=Balance,
        >;
        
        type SocialTokenTreasury: Get<ModuleId>;
        
        type CountryInfoSource: BCCountry<Self::AccountId>;
    }

    #[pallet::storage]
    #[pallet::getter(fn next_token_id)]
    /// The next asset identifier up for grabs.
    pub(super) type NextTokenId<T: Config> = StorageValue<_, CurrencyId, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn token_details)]
    /// Details of the token corresponding to the token id.
    /// (hash) -> Token details [returns Token struct]
    pub(super) type SocialTokens<T: Config> = 
        StorageMap<_, Blake2_128Concat, CurrencyId, Token<Balance>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_country_treasury)]
    /// Details of the token corresponding to the token id.
    /// (hash) -> Token details [returns Token struct]
    pub(super) type CountryTreasury<T: Config> = 
        StorageMap<_, Blake2_128Concat, CountryId, CountryFund<T::AccountId,Balance>, OptionQuery>;

    #[pallet::error]
    pub enum Error<T> {
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
         /// Country Currency already issued for this bitcountry
         SocialTokenAlreadyIssued,
         /// No available next token id
         NoAvailableTokenId,
         //Country Is Not Available
         CountryFundIsNotAvailable
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Issue a new class of fungible assets for bitcountry. There are, and will only ever be, `total`
        /// such assets and they'll all belong to the `origin` initially. It will have an
        /// identifier `TokenId` instance: this will be specified in the `Issued` event.
        #[pallet::weight(10_000)]
        pub fn mint_token(
            origin: OriginFor<T>, 
            ticker: Ticker, 
            country_id: CountryId, 
            total_supply: Balance
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                T::CountryInfoSource::check_ownership(&who, &country_id), 
                Error::<T>::NoPermissionTokenIssuance
            );
            ensure!(
                !CountryTreasury::<T>::contains_key(&country_id), 
                Error::<T>::SocialTokenAlreadyIssued
            );

            //Generate new CurrencyId
            let currency_id = NextTokenId::<T>::mutate(|id| -> Result<CurrencyId, DispatchError>{
                let current_id =*id;
                if current_id == 0 {
                   *id = 2;
                    Ok(One::one())
                }
                else{
                    *id = id.checked_add(One::one())
                    .ok_or(Error::<T>::NoAvailableTokenId)?;
                    Ok(current_id)
                }
            })?;
            let fund_id: T::AccountId = T::SocialTokenTreasury::get().into_sub_account(country_id);

            //Country treasury
            let country_fund = CountryFund {
                vault: fund_id,
                value: total_supply,
                backing: 0, //0 NUUM backing for now,
                currency_id: currency_id,
            };

            let token_info = Token{
                ticker,
                total_supply,
            };
            //Store social token info
            SocialTokens::<T>::insert(currency_id, token_info);

            CountryTreasury::<T>::insert(country_id, country_fund);
            //TODO Add initial LP
            T::CountryCurrency::deposit(currency_id, &who, total_supply)?;
            let fund_address = Self::get_country_fund_id(country_id);
            Self::deposit_event(Event::<T>::SocialTokenIssued(currency_id.clone(), who, fund_address ,total_supply));
            
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn transfer(
            origin: OriginFor<T>, 
            dest: <T::Lookup as StaticLookup>::Source,
            currency_id: CurrencyId,
            // #[compact] amount: Balance
            amount: Balance
        ) -> DispatchResultWithPostInfo {

            let from = ensure_signed(origin)?;
            let to = T::Lookup::lookup(dest)?;
            Self::transfer_from(currency_id, &from, &to, amount)?;

            Ok(().into())
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    #[pallet::metadata(
        <T as frame_system::Config >::AccountId = "AccountId",
        Balance = "Balance",
        CurrencyId = "CurrencyId"
    )]
    pub enum Event<T: Config> {
        /// Some assets were issued. \[asset_id, owner, total_supply\]
        SocialTokenIssued(CurrencyId, T::AccountId, T::AccountId , u128),
        /// Some assets were transferred. \[asset_id, from, to, amount\]
        SocialTokenTransferred(CurrencyId, T::AccountId, T::AccountId, Balance),
        /// Some assets were destroyed. \[asset_id, owner, balance\]
        SocialTokenDestroyed(CurrencyId, T::AccountId, Balance),
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}

impl<T: Config> Module<T> {
    fn transfer_from(
        currency_id: CurrencyId,
        from: &T::AccountId,
        to: &T::AccountId,
        amount: Balance,
    ) -> DispatchResult {
        if amount.is_zero() || from == to {
            return Ok(());
        }

        T::CountryCurrency::transfer(currency_id, from, to, amount)?;

        Self::deposit_event(Event::<T>::SocialTokenTransferred(
            currency_id,
            from.clone(),
            to.clone(),
            amount,
        ));
        Ok(())
    }

    pub fn get_total_issuance(country_id: CountryId) -> Result<Balance, DispatchError> {
        let country_fund =
            CountryTreasury::<T>::get(country_id).ok_or(Error::<T>::CountryFundIsNotAvailable)?;
        let total_issuance = T::CountryCurrency::total_issuance(country_fund.currency_id);

        Ok(total_issuance)
    }

    pub fn get_country_fund_id(country_id: CountryId) -> T::AccountId {
        match CountryTreasury::<T>::get(country_id) {
            Some(fund) => fund.vault,
            _ => Default::default()
        }
    }
}


