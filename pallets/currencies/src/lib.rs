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
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

use bc_primitives::*;
use codec::{Codec, Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResultWithPostInfo,
    ensure, pallet_prelude::*, PalletId, Parameter,
};
use frame_support::{
    pallet_prelude::*,
    traits::{
        Currency as PalletCurrency, ExistenceRequirement, Get,
        LockableCurrency as PalletLockableCurrency, ReservableCurrency as PalletReservableCurrency,
        WithdrawReasons,
    },
};
use frame_system::pallet_prelude::*;
use frame_system::{self as system, ensure_signed};
use orml_traits::{
    arithmetic::{Signed, SimpleArithmetic},
    currency::TransferAll,
    BalanceStatus, BasicCurrency, BasicCurrencyExtended, BasicLockableCurrency,
    BasicReservableCurrency, LockIdentifier, MultiCurrency, MultiCurrencyExtended,
    MultiLockableCurrency, MultiReservableCurrency,
};
use primitives::{Balance, BitCountryId, CurrencyId, FungibleTokenId};
use sp_runtime::{
    traits::{CheckedSub, MaybeSerializeDeserialize, Saturating, StaticLookup, Zero},
    DispatchError, DispatchResult,
};
use sp_std::{
    convert::{TryFrom, TryInto},
    fmt::Debug,
    marker, result,
    vec::Vec,
};

// #[cfg(test)]
// mod mock;
//
// #[cfg(test)]
// mod tests;

pub use pallet::*;

type BalanceOf<T> = <<T as Config>::MultiSocialCurrency as MultiCurrency<
    <T as frame_system::Config>::AccountId,
>>::Balance;
type CurrencyIdOf<T> = <<T as Config>::MultiSocialCurrency as MultiCurrency<
    <T as frame_system::Config>::AccountId,
>>::CurrencyId;

type AmountOf<T> = <<T as Config>::MultiSocialCurrency as MultiCurrencyExtended<
    <T as frame_system::Config>::AccountId,
>>::Amount;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type MultiSocialCurrency: TransferAll<Self::AccountId>
            + MultiCurrencyExtended<Self::AccountId, CurrencyId = FungibleTokenId>
            + MultiLockableCurrency<Self::AccountId, CurrencyId = FungibleTokenId>
            + MultiReservableCurrency<Self::AccountId, CurrencyId = FungibleTokenId>;
        type NativeCurrency: BasicCurrencyExtended<
                Self::AccountId,
                Balance = BalanceOf<Self>,
                Amount = AmountOf<Self>,
            > + BasicLockableCurrency<Self::AccountId, Balance = BalanceOf<Self>>;
        #[pallet::constant]
        /// The native currency id
        type GetNativeCurrencyId: Get<FungibleTokenId>;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Transfer amount should be non-zero
        AmountZero,
        /// Account balance must be greater than or equal to the transfer amount
        BalanceLow,
        /// Account balance must be greater than or equal to the transfer amount
        BalanceTooLow,
        /// Balance should be non-zero
        BalanceZero,
        ///Insufficient balance
        InsufficientBalance,
        /// No permission to issue token
        NoPermissionTokenIssuance,
        /// Country Currency already issued for this bitcountry
        FungibleTokenAlreadyIssued,
        /// No available next token id
        NoAvailableTokenId,
        //Country Is Not Available
        BitCountryFundIsNotAvailable,
        AmountIntoBalanceFailed,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn transfer(
            origin: OriginFor<T>,
            dest: <T::Lookup as StaticLookup>::Source,
            currency_id: CurrencyIdOf<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let from = ensure_signed(origin)?;
            let to = T::Lookup::lookup(dest)?;
            <Self as MultiCurrency<T::AccountId>>::transfer(currency_id, &from, &to, amount)?;
            Ok(().into())
        }

        /// Transfer some native currency to another account.
        ///
        /// The dispatch origin for this call must be `Signed` by the
        /// transactor.
        #[pallet::weight(10_000)]
        pub fn transfer_native_currency(
            origin: OriginFor<T>,
            dest: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let from = ensure_signed(origin)?;
            let to = T::Lookup::lookup(dest)?;
            T::NativeCurrency::transfer(&from, &to, amount)?;

            Self::deposit_event(Event::Transferred(
                T::GetNativeCurrencyId::get(),
                from,
                to,
                amount,
            ));
            Ok(().into())
        }

        /// update amount of account `who` under `currency_id`.
        ///
        /// The dispatch origin of this call must be _Root_.
        #[pallet::weight(10_000)]
        pub fn update_balance(
            origin: OriginFor<T>,
            who: <T::Lookup as StaticLookup>::Source,
            currency_id: CurrencyIdOf<T>,
            amount: AmountOf<T>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            let dest = T::Lookup::lookup(who)?;
            <Self as MultiCurrencyExtended<T::AccountId>>::update_balance(
                currency_id,
                &dest,
                amount,
            )?;
            Ok(().into())
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub (crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Currency transfer success. [currency_id, from, to, amount]
        Transferred(CurrencyIdOf<T>, T::AccountId, T::AccountId, BalanceOf<T>),
        /// Update balance success. [currency_id, who, amount]
        BalanceUpdated(CurrencyIdOf<T>, T::AccountId, AmountOf<T>),
        /// Deposit success. [currency_id, who, amount]
        Deposited(CurrencyIdOf<T>, T::AccountId, BalanceOf<T>),
        /// Withdraw success. [currency_id, who, amount]
        Withdrawn(CurrencyIdOf<T>, T::AccountId, BalanceOf<T>),
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}

impl<T: Config> MultiCurrency<T::AccountId> for Pallet<T> {
    type CurrencyId = CurrencyIdOf<T>;
    type Balance = BalanceOf<T>;

    fn minimum_balance(currency_id: Self::CurrencyId) -> Self::Balance {
        match currency_id {
            id if id == T::GetNativeCurrencyId::get() => T::NativeCurrency::minimum_balance(),
            _ => T::MultiSocialCurrency::minimum_balance(currency_id),
        }
    }

    fn total_issuance(currency_id: Self::CurrencyId) -> Self::Balance {
        match currency_id {
            id if id == T::GetNativeCurrencyId::get() => T::NativeCurrency::total_issuance(),
            _ => T::MultiSocialCurrency::total_issuance(currency_id),
        }
    }

    fn total_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
        match currency_id {
            id if id == T::GetNativeCurrencyId::get() => T::NativeCurrency::total_balance(who),
            _ => T::MultiSocialCurrency::total_balance(currency_id, who),
        }
    }

    fn free_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
        match currency_id {
            id if id == T::GetNativeCurrencyId::get() => T::NativeCurrency::free_balance(who),
            _ => T::MultiSocialCurrency::free_balance(currency_id, who),
        }
    }

    fn ensure_can_withdraw(
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> DispatchResult {
        match currency_id {
            id if id == T::GetNativeCurrencyId::get() => {
                T::NativeCurrency::ensure_can_withdraw(who, amount)
            }
            _ => T::MultiSocialCurrency::ensure_can_withdraw(currency_id, who, amount),
        }
    }

    fn transfer(
        currency_id: Self::CurrencyId,
        from: &T::AccountId,
        to: &T::AccountId,
        amount: Self::Balance,
    ) -> DispatchResult {
        if amount.is_zero() || from == to {
            return Ok(());
        }

        match currency_id {
            id if id == T::GetNativeCurrencyId::get() => {
                T::NativeCurrency::transfer(from, to, amount)?
            }
            _ => T::MultiSocialCurrency::transfer(currency_id, from, to, amount)?,
        }

        Self::deposit_event(Event::Transferred(
            currency_id,
            from.clone(),
            to.clone(),
            amount,
        ));
        Ok(())
    }

    fn deposit(
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> DispatchResult {
        if amount.is_zero() {
            return Ok(());
        }
        match currency_id {
            id if id == T::GetNativeCurrencyId::get() => T::NativeCurrency::deposit(who, amount)?,
            _ => T::MultiSocialCurrency::deposit(currency_id, who, amount)?,
        }
        Self::deposit_event(Event::Deposited(currency_id, who.clone(), amount));
        Ok(())
    }

    fn withdraw(
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> DispatchResult {
        if amount.is_zero() {
            return Ok(());
        }
        match currency_id {
            id if id == T::GetNativeCurrencyId::get() => T::NativeCurrency::withdraw(who, amount)?,
            _ => T::MultiSocialCurrency::withdraw(currency_id, who, amount)?,
        }
        Self::deposit_event(Event::Withdrawn(currency_id, who.clone(), amount));
        Ok(())
    }

    fn can_slash(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> bool {
        match currency_id {
            id if id == T::GetNativeCurrencyId::get() => T::NativeCurrency::can_slash(who, amount),
            _ => T::MultiSocialCurrency::can_slash(currency_id, who, amount),
        }
    }

    fn slash(
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> Self::Balance {
        match currency_id {
            id if id == T::GetNativeCurrencyId::get() => T::NativeCurrency::slash(who, amount),
            _ => T::MultiSocialCurrency::slash(currency_id, who, amount),
        }
    }
}

impl<T: Config> MultiCurrencyExtended<T::AccountId> for Pallet<T> {
    type Amount = AmountOf<T>;

    fn update_balance(
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        by_amount: Self::Amount,
    ) -> DispatchResult {
        match currency_id {
            id if id == T::GetNativeCurrencyId::get() => {
                T::NativeCurrency::update_balance(who, by_amount)?
            }
            _ => T::MultiSocialCurrency::update_balance(currency_id, who, by_amount)?,
        }
        Self::deposit_event(Event::BalanceUpdated(currency_id, who.clone(), by_amount));
        Ok(())
    }
}

impl<T: Config> MultiLockableCurrency<T::AccountId> for Pallet<T> {
    type Moment = T::BlockNumber;

    fn set_lock(
        lock_id: LockIdentifier,
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> DispatchResult {
        if currency_id == T::GetNativeCurrencyId::get() {
            T::NativeCurrency::set_lock(lock_id, who, amount)
        } else {
            T::MultiSocialCurrency::set_lock(lock_id, currency_id, who, amount)
        }
    }

    fn extend_lock(
        lock_id: LockIdentifier,
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> DispatchResult {
        if currency_id == T::GetNativeCurrencyId::get() {
            T::NativeCurrency::extend_lock(lock_id, who, amount)
        } else {
            T::MultiSocialCurrency::extend_lock(lock_id, currency_id, who, amount)
        }
    }

    fn remove_lock(
        lock_id: LockIdentifier,
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
    ) -> DispatchResult {
        if currency_id == T::GetNativeCurrencyId::get() {
            T::NativeCurrency::remove_lock(lock_id, who)
        } else {
            T::MultiSocialCurrency::remove_lock(lock_id, currency_id, who)
        }
    }
}

pub struct Currency<T, GetCurrencyId>(marker::PhantomData<T>, marker::PhantomData<GetCurrencyId>);

impl<T, GetCurrencyId> BasicCurrency<T::AccountId> for Currency<T, GetCurrencyId>
where
    T: Config,
    GetCurrencyId: Get<CurrencyIdOf<T>>,
{
    type Balance = BalanceOf<T>;

    fn minimum_balance() -> Self::Balance {
        <Pallet<T>>::minimum_balance(GetCurrencyId::get())
    }

    fn total_issuance() -> Self::Balance {
        <Pallet<T>>::total_issuance(GetCurrencyId::get())
    }

    fn total_balance(who: &T::AccountId) -> Self::Balance {
        <Pallet<T>>::total_balance(GetCurrencyId::get(), who)
    }

    fn free_balance(who: &T::AccountId) -> Self::Balance {
        <Pallet<T>>::free_balance(GetCurrencyId::get(), who)
    }

    fn ensure_can_withdraw(who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
        <Pallet<T>>::ensure_can_withdraw(GetCurrencyId::get(), who, amount)
    }

    fn transfer(from: &T::AccountId, to: &T::AccountId, amount: Self::Balance) -> DispatchResult {
        <Pallet<T> as MultiCurrency<T::AccountId>>::transfer(GetCurrencyId::get(), from, to, amount)
    }

    fn deposit(who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
        <Pallet<T>>::deposit(GetCurrencyId::get(), who, amount)
    }

    fn withdraw(who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
        <Pallet<T>>::withdraw(GetCurrencyId::get(), who, amount)
    }

    fn can_slash(who: &T::AccountId, amount: Self::Balance) -> bool {
        <Pallet<T>>::can_slash(GetCurrencyId::get(), who, amount)
    }

    fn slash(who: &T::AccountId, amount: Self::Balance) -> Self::Balance {
        <Pallet<T>>::slash(GetCurrencyId::get(), who, amount)
    }
}

impl<T, GetCurrencyId> BasicCurrencyExtended<T::AccountId> for Currency<T, GetCurrencyId>
where
    T: Config,
    GetCurrencyId: Get<CurrencyIdOf<T>>,
{
    type Amount = AmountOf<T>;

    fn update_balance(who: &T::AccountId, by_amount: Self::Amount) -> DispatchResult {
        <Pallet<T> as MultiCurrencyExtended<T::AccountId>>::update_balance(
            GetCurrencyId::get(),
            who,
            by_amount,
        )
    }
}

/// Adapt other currency traits implementation to `BasicCurrency`.
pub struct BasicCurrencyAdapter<T, Currency, Amount, Moment>(
    marker::PhantomData<(T, Currency, Amount, Moment)>,
);

type PalletBalanceOf<A, Currency> = <Currency as PalletCurrency<A>>::Balance;

// Adapt `frame_support::traits::Currency`
impl<T, AccountId, Currency, Amount, Moment> BasicCurrency<AccountId>
    for BasicCurrencyAdapter<T, Currency, Amount, Moment>
where
    Currency: PalletCurrency<AccountId>,
    T: Config,
{
    type Balance = PalletBalanceOf<AccountId, Currency>;

    fn minimum_balance() -> Self::Balance {
        Currency::minimum_balance()
    }

    fn total_issuance() -> Self::Balance {
        Currency::total_issuance()
    }

    fn total_balance(who: &AccountId) -> Self::Balance {
        Currency::total_balance(who)
    }

    fn free_balance(who: &AccountId) -> Self::Balance {
        Currency::free_balance(who)
    }

    fn ensure_can_withdraw(who: &AccountId, amount: Self::Balance) -> DispatchResult {
        let new_balance = Self::free_balance(who)
            .checked_sub(&amount)
            .ok_or(Error::<T>::BalanceTooLow)?;

        Currency::ensure_can_withdraw(who, amount, WithdrawReasons::all(), new_balance)
    }

    fn transfer(from: &AccountId, to: &AccountId, amount: Self::Balance) -> DispatchResult {
        Currency::transfer(from, to, amount, ExistenceRequirement::AllowDeath)
    }

    fn deposit(who: &AccountId, amount: Self::Balance) -> DispatchResult {
        let _ = Currency::deposit_creating(who, amount);
        Ok(())
    }

    fn withdraw(who: &AccountId, amount: Self::Balance) -> DispatchResult {
        Currency::withdraw(
            who,
            amount,
            WithdrawReasons::all(),
            ExistenceRequirement::AllowDeath,
        )
        .map(|_| ())
    }

    fn can_slash(who: &AccountId, amount: Self::Balance) -> bool {
        Currency::can_slash(who, amount)
    }

    fn slash(who: &AccountId, amount: Self::Balance) -> Self::Balance {
        let (_, gap) = Currency::slash(who, amount);
        gap
    }
}

// Adapt `frame_support::traits::Currency`
impl<T, AccountId, Currency, Amount, Moment> BasicCurrencyExtended<AccountId>
    for BasicCurrencyAdapter<T, Currency, Amount, Moment>
where
    Amount: Signed
        + TryInto<PalletBalanceOf<AccountId, Currency>>
        + TryFrom<PalletBalanceOf<AccountId, Currency>>
        + SimpleArithmetic
        + Codec
        + Copy
        + MaybeSerializeDeserialize
        + Debug
        + Default,
    Currency: PalletCurrency<AccountId>,
    T: Config,
{
    type Amount = Amount;

    fn update_balance(who: &AccountId, by_amount: Self::Amount) -> DispatchResult {
        let by_balance = by_amount
            .abs()
            .try_into()
            .map_err(|_| Error::<T>::AmountIntoBalanceFailed)?;
        if by_amount.is_positive() {
            Self::deposit(who, by_balance)
        } else {
            Self::withdraw(who, by_balance)
        }
    }
}

// Adapt `frame_support::traits::LockableCurrency`
impl<T, AccountId, Currency, Amount, Moment> BasicLockableCurrency<AccountId>
    for BasicCurrencyAdapter<T, Currency, Amount, Moment>
where
    Currency: PalletLockableCurrency<AccountId>,
    T: Config,
{
    type Moment = Moment;

    fn set_lock(lock_id: LockIdentifier, who: &AccountId, amount: Self::Balance) -> DispatchResult {
        Currency::set_lock(lock_id, who, amount, WithdrawReasons::all());
        Ok(())
    }

    fn extend_lock(
        lock_id: LockIdentifier,
        who: &AccountId,
        amount: Self::Balance,
    ) -> DispatchResult {
        Currency::extend_lock(lock_id, who, amount, WithdrawReasons::all());
        Ok(())
    }

    fn remove_lock(lock_id: LockIdentifier, who: &AccountId) -> DispatchResult {
        Currency::remove_lock(lock_id, who);
        Ok(())
    }
}

// Adapt `frame_support::traits::ReservableCurrency`
impl<T, AccountId, Currency, Amount, Moment> BasicReservableCurrency<AccountId>
    for BasicCurrencyAdapter<T, Currency, Amount, Moment>
where
    Currency: PalletReservableCurrency<AccountId>,
    T: Config,
{
    fn can_reserve(who: &AccountId, value: Self::Balance) -> bool {
        Currency::can_reserve(who, value)
    }

    fn slash_reserved(who: &AccountId, value: Self::Balance) -> Self::Balance {
        let (_, gap) = Currency::slash_reserved(who, value);
        gap
    }

    fn reserved_balance(who: &AccountId) -> Self::Balance {
        Currency::reserved_balance(who)
    }

    fn reserve(who: &AccountId, value: Self::Balance) -> DispatchResult {
        Currency::reserve(who, value)
    }

    fn unreserve(who: &AccountId, value: Self::Balance) -> Self::Balance {
        Currency::unreserve(who, value)
    }

    fn repatriate_reserved(
        slashed: &AccountId,
        beneficiary: &AccountId,
        value: Self::Balance,
        status: BalanceStatus,
    ) -> result::Result<Self::Balance, DispatchError> {
        Currency::repatriate_reserved(slashed, beneficiary, value, status)
    }
}
