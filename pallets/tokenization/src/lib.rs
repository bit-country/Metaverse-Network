// This file is part of Bit.Country.
// Extension of orml vesting schedule to support multi-currencies vesting.
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

use auction_manager::SwapManager;
use bc_primitives::*;
use codec::{Decode, Encode};
use frame_support::traits::{Currency, Get, WithdrawReasons};
use frame_support::PalletId;
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::{DispatchResult, DispatchResultWithPostInfo},
	ensure,
	pallet_prelude::*,
	transactional, Parameter,
};
use frame_system::pallet_prelude::*;
use frame_system::{self as system, ensure_signed};
use orml_traits::{
	arithmetic::{Signed, SimpleArithmetic},
	BalanceStatus, BasicCurrency, BasicCurrencyExtended, BasicLockableCurrency, BasicReservableCurrency,
	LockIdentifier, MultiCurrency, MultiCurrencyExtended, MultiLockableCurrency, MultiReservableCurrency,
};
use primitives::{Balance, BitCountryId, CurrencyId, FungibleTokenId};
use sp_runtime::{
	traits::{AccountIdConversion, AtLeast32Bit, One, StaticLookup, Zero},
	DispatchError,
};
use sp_std::vec::Vec;

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

/// The maximum number of vesting schedules an account can have.
pub const MAX_VESTINGS: usize = 20;

pub const VESTING_LOCK_ID: LockIdentifier = *b"bcstvest";

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::sp_runtime::traits::Saturating;
	use frame_support::sp_runtime::{FixedPointNumber, SaturatedConversion};
	use primitives::dex::Price;
	use primitives::{FungibleTokenId, TokenId, VestingSchedule};
	use sp_std::convert::TryInto;

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	pub(crate) type VestingScheduleOf<T> = VestingSchedule<<T as frame_system::Config>::BlockNumber, Balance>;
	pub type ScheduledItem<T> = (
		<T as frame_system::Config>::AccountId,
		<T as frame_system::Config>::BlockNumber,
		<T as frame_system::Config>::BlockNumber,
		u32,
		Balance,
	);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The arithmetic type of asset identifier.
		type TokenId: Parameter + AtLeast32Bit + Default + Copy;
		type BCMultiCurrency: MultiCurrencyExtended<Self::AccountId, CurrencyId = FungibleTokenId, Balance = Balance>
			+ MultiLockableCurrency<Self::AccountId, CurrencyId = FungibleTokenId>;
		type FungibleTokenTreasury: Get<PalletId>;
		type BitCountryInfoSource: BitCountryTrait<Self::AccountId>;
		type LiquidityPoolManager: SwapManager<Self::AccountId, FungibleTokenId, Balance>;
		#[pallet::constant]
		/// The minimum amount transferred to call `vested_transfer`.
		type MinVestedTransfer: Get<Balance>;
		/// Required origin for vested transfer.
		type VestedTransferOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;
	}

	#[pallet::storage]
	#[pallet::getter(fn next_token_id)]
	/// The next asset identifier up for grabs.
	pub(super) type NextTokenId<T: Config> = StorageValue<_, TokenId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn token_details)]
	/// Details of the token corresponding to the token id.
	/// (hash) -> Token details [returns Token struct]
	pub(super) type FungibleTokens<T: Config> =
		StorageMap<_, Blake2_128Concat, FungibleTokenId, Token<Balance>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_country_treasury)]
	/// Details of the token corresponding to the token id.
	/// (hash) -> Token details [returns Token struct]
	pub(super) type CountryTreasury<T: Config> =
		StorageMap<_, Blake2_128Concat, BitCountryId, BitCountryFund<T::AccountId, Balance>, OptionQuery>;

	/// Vesting schedules of an account.
	#[pallet::storage]
	#[pallet::getter(fn vesting_schedules)]
	pub type VestingSchedules<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Vec<VestingScheduleOf<T>>, ValueQuery>;

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
		FungibleTokenAlreadyIssued,
		/// No available next token id
		NoAvailableTokenId,
		/// Country Fund Not Available
		BitCountryFundIsNotAvailable,
		/// Initial Social Token Supply is too low
		InitialFungibleTokenSupplyIsTooLow,
		/// Failed on updating social token for this bitcountry
		FailedOnUpdatingFungibleToken,
		/// Vesting period is zero
		ZeroVestingPeriod,
		/// Number of vests is zero
		ZeroVestingPeriodCount,
		/// Arithmetic calculation overflow
		NumOverflow,
		/// Insufficient amount of balance to lock
		InsufficientBalanceToLock,
		/// This account have too many vesting schedules
		TooManyVestingSchedules,
		/// Invalid vesting schedule
		InvalidVestingSchedule,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Issue a new class of fungible assets for bitcountry. There are, and will only ever be,
		/// `total` such assets and they'll all belong to the `origin` initially. It will have an
		/// identifier `TokenId` instance: this will be specified in the `Issued` event.
		#[pallet::weight(10_000)]
		pub fn mint_token(
			origin: OriginFor<T>,
			ticker: Ticker,
			country_id: BitCountryId,
			total_supply: Balance,
			initial_lp: (u32, u32),
			initial_backing: Balance,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(
				T::BitCountryInfoSource::check_ownership(&who, &country_id),
				Error::<T>::NoPermissionTokenIssuance
			);
			ensure!(
				!CountryTreasury::<T>::contains_key(&country_id),
				Error::<T>::FungibleTokenAlreadyIssued
			);

			let initial_pool_numerator = total_supply.saturating_mul(initial_lp.0.saturated_into());
			let initial_pool_supply = initial_pool_numerator
				.checked_div(initial_lp.1.saturated_into())
				.unwrap_or(0);
			let initial_supply_ratio =
				Price::checked_from_rational(initial_pool_supply, total_supply).unwrap_or_default();
			let supply_percent: u128 = initial_supply_ratio.saturating_mul_int(100.saturated_into());
			ensure!(
				supply_percent > 0u128 && supply_percent >= 20u128,
				Error::<T>::InitialFungibleTokenSupplyIsTooLow
			);
			// Remaining balance for bc owner
			let owner_supply = total_supply.saturating_sub(initial_pool_supply);
			// Generate new TokenId
			let currency_id = NextTokenId::<T>::mutate(|id| -> Result<FungibleTokenId, DispatchError> {
				let current_id = *id;
				if current_id == 0 {
					*id = 2;
					Ok(FungibleTokenId::FungibleToken(One::one()))
				} else {
					*id = id.checked_add(One::one()).ok_or(Error::<T>::NoAvailableTokenId)?;
					Ok(FungibleTokenId::FungibleToken(current_id))
				}
			})?;
			let fund_id: T::AccountId = T::FungibleTokenTreasury::get().into_sub_account(country_id);

			// Bit Country treasury
			let country_fund = BitCountryFund {
				vault: fund_id.clone(),
				value: total_supply,
				backing: initial_backing,
				currency_id: currency_id,
			};

			let token_info = Token { ticker, total_supply };

			// Update currency id in BC
			T::BitCountryInfoSource::update_bitcountry_token(country_id.clone(), currency_id.clone())?;

			// Store social token info
			FungibleTokens::<T>::insert(currency_id, token_info);

			CountryTreasury::<T>::insert(country_id.clone(), country_fund);
			// Deposit fund into bit country treasury
			T::BCMultiCurrency::transfer(FungibleTokenId::NativeToken(0), &who, &fund_id, initial_backing.clone())?;
			T::BCMultiCurrency::deposit(currency_id, &fund_id, total_supply.clone())?;
			// Social currency should deposit to DEX pool instead, by calling provide LP function in DEX traits.
			T::LiquidityPoolManager::add_liquidity(
				&fund_id,
				FungibleTokenId::NativeToken(0),
				currency_id,
				initial_backing,
				initial_pool_supply,
			)?;

			// The remaining token will be vested gradually 12 months.
			let now = <frame_system::Pallet<T>>::block_number();
			let vested_per_period = owner_supply.checked_div(12).ok_or("Overflow")?;
			let period_block_number: T::BlockNumber = TryInto::<T::BlockNumber>::try_into(28800u64).unwrap_or_default();

			let vesting_schedule = VestingSchedule {
				token: currency_id,
				start: now,
				period: period_block_number,
				period_count: 12,
				per_period: vested_per_period,
			};

			T::BCMultiCurrency::transfer(currency_id, &fund_id, &who, owner_supply.clone())?;
			T::BCMultiCurrency::set_lock(VESTING_LOCK_ID, currency_id, &who, owner_supply);
			<VestingSchedules<T>>::append(who.clone(), vesting_schedule.clone());
			Self::deposit_event(Event::VestingScheduleAdded(
				currency_id,
				fund_id,
				who.clone(),
				vesting_schedule,
			));

			let fund_address = Self::get_country_fund_id(country_id);
			Self::deposit_event(Event::<T>::FungibleTokenIssued(
				currency_id.clone(),
				who.clone(),
				fund_address,
				total_supply,
				country_id,
			));

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn transfer(
			origin: OriginFor<T>,
			dest: <T::Lookup as StaticLookup>::Source,
			currency_id: FungibleTokenId,
			amount: Balance,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;
			let to = T::Lookup::lookup(dest)?;
			Self::transfer_from(currency_id, &from, &to, amount)?;

			Ok(().into())
		}

		#[pallet::weight(100_000)]
		pub fn claim(origin: OriginFor<T>, currency_id: FungibleTokenId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let locked_amount = Self::do_claim(&who, currency_id);

			Self::deposit_event(Event::Claimed(currency_id.clone(), who, locked_amount));
			Ok(().into())
		}

		#[pallet::weight(100_000)]
		pub fn vested_transfer(
			origin: OriginFor<T>,
			dest: <T::Lookup as StaticLookup>::Source,
			schedule: VestingScheduleOf<T>,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;
			let to = T::Lookup::lookup(dest)?;
			let currency_id = schedule.token;
			Self::do_vested_transfer(&from, &to, currency_id.clone(), schedule.clone())?;

			Self::deposit_event(Event::VestingScheduleAdded(currency_id, from, to, schedule));
			Ok(().into())
		}

		#[pallet::weight(100_000)]
		pub fn update_vesting_schedules(
			origin: OriginFor<T>,
			who: <T::Lookup as StaticLookup>::Source,
			currency_id: FungibleTokenId,
			vesting_schedules: Vec<VestingScheduleOf<T>>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			let account = T::Lookup::lookup(who)?;
			Self::do_update_vesting_schedules(&account, currency_id.clone(), vesting_schedules)?;

			Self::deposit_event(Event::VestingSchedulesUpdated(currency_id, account));
			Ok(().into())
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	#[pallet::metadata(
    < T as frame_system::Config >::AccountId = "AccountId",
    Balance = "Balance",
    CurrencyId = "CurrencyId"
    )]
	pub enum Event<T: Config> {
		/// Some assets were issued. \[asset_id, owner, fund_id ,total_supply\]
		FungibleTokenIssued(FungibleTokenId, T::AccountId, T::AccountId, u128, u64),
		/// Some assets were transferred. \[asset_id, from, to, amount\]
		FungibleTokenTransferred(FungibleTokenId, T::AccountId, T::AccountId, Balance),
		/// Some assets were destroyed. \[asset_id, owner, balance\]
		FungibleTokenDestroyed(FungibleTokenId, T::AccountId, Balance),
		/// Added new vesting schedule. [token, from, to, vesting_schedule]
		VestingScheduleAdded(FungibleTokenId, T::AccountId, T::AccountId, VestingScheduleOf<T>),
		/// Claimed vesting. [token, who, locked_amount]
		Claimed(FungibleTokenId, T::AccountId, Balance),
		/// Updated vesting schedules. [token, who]
		VestingSchedulesUpdated(FungibleTokenId, T::AccountId),
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}

impl<T: Config> Module<T> {
	fn transfer_from(
		currency_id: FungibleTokenId,
		from: &T::AccountId,
		to: &T::AccountId,
		amount: Balance,
	) -> DispatchResult {
		if amount.is_zero() || from == to {
			return Ok(());
		}

		T::BCMultiCurrency::transfer(currency_id, from, to, amount)?;

		Self::deposit_event(Event::<T>::FungibleTokenTransferred(
			currency_id,
			from.clone(),
			to.clone(),
			amount,
		));
		Ok(())
	}

	pub fn get_total_issuance(country_id: BitCountryId) -> Result<Balance, DispatchError> {
		let country_fund = CountryTreasury::<T>::get(country_id).ok_or(Error::<T>::BitCountryFundIsNotAvailable)?;
		let total_issuance = T::BCMultiCurrency::total_issuance(country_fund.currency_id);

		Ok(total_issuance)
	}

	pub fn get_country_fund_id(country_id: BitCountryId) -> T::AccountId {
		match CountryTreasury::<T>::get(country_id) {
			Some(fund) => fund.vault,
			_ => Default::default(),
		}
	}

	fn do_claim(who: &T::AccountId, currency_id: FungibleTokenId) -> Balance {
		let locked = Self::locked_balance(who, currency_id.clone());
		if locked.is_zero() {
			T::BCMultiCurrency::remove_lock(VESTING_LOCK_ID, currency_id, who);
		} else {
			T::BCMultiCurrency::set_lock(VESTING_LOCK_ID, currency_id, who, locked);
		}
		locked
	}

	/// Returns locked balance of any social token based on current block number.
	fn locked_balance(who: &T::AccountId, currency_id: FungibleTokenId) -> Balance {
		let now = <frame_system::Pallet<T>>::block_number();
		<VestingSchedules<T>>::mutate_exists(who, |maybe_schedules| {
			let total = if let Some(schedules) = maybe_schedules.as_mut() {
				let mut total: Balance = Zero::zero();
				schedules.retain(|s| {
					if s.token == currency_id {
						let amount = s.locked_amount(now);
						total = total.saturating_add(amount);
						!amount.is_zero()
					} else {
						false
					}
				});
				total
			} else {
				Zero::zero()
			};
			if total.is_zero() {
				*maybe_schedules = None;
			}
			total
		})
	}

	#[transactional]
	fn do_vested_transfer(
		from: &T::AccountId,
		to: &T::AccountId,
		currency_id: FungibleTokenId,
		schedule: VestingScheduleOf<T>,
	) -> DispatchResult {
		let schedule_amount = Self::ensure_valid_vesting_schedule(&currency_id, &schedule)?;

		ensure!(
			<VestingSchedules<T>>::decode_len(to).unwrap_or(0) < MAX_VESTINGS,
			Error::<T>::TooManyVestingSchedules
		);

		let total_amount = Self::locked_balance(to, schedule.token)
			.checked_add(schedule_amount)
			.ok_or(Error::<T>::NumOverflow)?;

		T::BCMultiCurrency::transfer(schedule.token, from, to, schedule_amount)?;
		T::BCMultiCurrency::set_lock(VESTING_LOCK_ID, schedule.token, to, total_amount);
		<VestingSchedules<T>>::append(to, schedule);
		Ok(())
	}

	fn do_update_vesting_schedules(
		who: &T::AccountId,
		currency_id: FungibleTokenId,
		schedules: Vec<VestingScheduleOf<T>>,
	) -> DispatchResult {
		let total_amount =
			schedules
				.iter()
				.try_fold::<_, _, Result<Balance, Error<T>>>(Zero::zero(), |acc_amount, schedule| {
					let amount = Self::ensure_valid_vesting_schedule(&currency_id, schedule)?;
					Ok(acc_amount + amount)
				})?;
		ensure!(
			T::BCMultiCurrency::free_balance(currency_id.clone(), who) >= total_amount,
			Error::<T>::InsufficientBalanceToLock,
		);

		T::BCMultiCurrency::set_lock(VESTING_LOCK_ID, currency_id, who, total_amount);
		<VestingSchedules<T>>::insert(who, schedules);

		Ok(())
	}

	/// Returns `Ok(amount)` if valid schedule, or error.
	fn ensure_valid_vesting_schedule(
		currency_id: &FungibleTokenId,
		schedule: &VestingScheduleOf<T>,
	) -> Result<Balance, Error<T>> {
		ensure!(schedule.token == *currency_id, Error::<T>::InvalidVestingSchedule);
		ensure!(!schedule.period.is_zero(), Error::<T>::ZeroVestingPeriod);
		ensure!(!schedule.period_count.is_zero(), Error::<T>::ZeroVestingPeriodCount);
		ensure!(schedule.end().is_some(), Error::<T>::NumOverflow);

		let total = schedule.total_amount().ok_or(Error::<T>::NumOverflow)?;

		ensure!(total >= T::MinVestedTransfer::get(), Error::<T>::BalanceLow);

		Ok(total)
	}
}
