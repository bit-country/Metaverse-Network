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
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{AccountIdConversion, AtLeast32Bit, One, StaticLookup, Zero},
	DispatchError,
};
use sp_std::vec::Vec;

use auction_manager::SwapManager;
use core_primitives::*;
pub use pallet::*;
use primitives::staking::RoundInfo;
use primitives::{Balance, CurrencyId, FungibleTokenId, MetaverseId};

#[cfg(test)]
mod mock;

mod mining;
#[cfg(test)]
mod tests;

/// A wrapper for a token name.
pub type TokenName = Vec<u8>;

/// A wrapper for a ticker name.
pub type Ticker = Vec<u8>;

#[derive(Encode, Decode, Default, Clone, PartialEq, TypeInfo)]
pub struct Token<Balance> {
	pub ticker: Ticker,
	pub total_supply: Balance,
}

/// The maximum number of vesting schedules an account can have.
pub const MAX_VESTINGS: usize = 20;

pub const VESTING_LOCK_ID: LockIdentifier = *b"bcstvest";

#[frame_support::pallet]
pub mod pallet {
	use frame_support::sp_runtime::traits::Saturating;
	use frame_support::sp_runtime::{FixedPointNumber, SaturatedConversion};
	use frame_support::traits::OnUnbalanced;
	use pallet_balances::NegativeImbalance;
	use sp_std::convert::TryInto;

	use primitives::dex::Price;
	use primitives::estate::Estate;
	use primitives::staking::RoundInfo;
	use primitives::{FungibleTokenId, RoundIndex, TokenId, VestingSchedule};

	use crate::mining::round_issuance_range;

	use super::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
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
		type MiningCurrency: MultiCurrencyExtended<Self::AccountId, CurrencyId = FungibleTokenId, Balance = Balance>;
		#[pallet::constant]
		type BitMiningTreasury: Get<PalletId>;
		type BitMiningResourceId: Get<FungibleTokenId>;
		/// Origin used to administer the pallet
		type EstateHandler: Estate<Self::AccountId>;
		type AdminOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;
	}

	/// Minting origins
	#[pallet::storage]
	#[pallet::getter(fn minting_origin)]
	pub type MintingOrigins<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, (), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn round)]
	/// Current round index and next round scheduled transition
	pub type Round<T: Config> = StorageValue<_, RoundInfo<T::BlockNumber>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn mining_ratio_config)]
	/// Mining resource issuance ratio config
	pub type MiningConfig<T: Config> = StorageValue<_, MiningResourceRateInfo, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn current_mining_resource_allocation)]
	/// Mining resource issuance ratio config
	pub type CurrentMiningResourceAllocation<T: Config> = StorageValue<_, MiningRange<Balance>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Mining resource minted [amount]
		MiningResourceMinted(Balance),
		/// Mining resource burned [amount]
		MiningResourceBurned(Balance),
		/// Deposit mining resource [who, amount]
		DepositMiningResource(T::AccountId, Balance),
		/// Withdraw mining resource [who, amount]
		WithdrawMiningResource(T::AccountId, Balance),
		/// Add new mining origins [who]
		AddNewMiningOrigin(T::AccountId),
		/// Remove mining origin [who]
		/// Add new mining origins [who]
		RemoveMiningOrigin(T::AccountId),
		/// New round
		NewMiningRound(RoundIndex),
		/// Round length update
		RoundLengthUpdated(T::BlockNumber),
		/// New mining config update
		MiningConfigUpdated(T::BlockNumber, MiningResourceRateInfo),
	}

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
		/// No permission to interact with mining resource
		NoPermission,
		/// Origins already exist
		OriginsAlreadyExist,
		/// Origin is not exist
		OriginsIsNotExist,
		/// Round update is on progress
		RoundUpdateIsOnProgress,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Issue mining resource on metaverse. There are, and will only ever be, `total`
		/// such assets and they'll all belong to the `origin` initially. It will have an
		/// identifier `TokenId` instance: this will be specified in the `Issued` event.
		#[pallet::weight(10_000)]
		pub fn mint(origin: OriginFor<T>, amount: Balance) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Self::do_mint(who, amount)?;

			Ok(().into())
		}

		/// Burn mining resource on metaverse. There are, and will only ever be, `total`
		/// such assets and they'll all belong to the `origin` initially. It will have an
		/// identifier `TokenId` instance: this will be specified in the `Issued` event.
		#[pallet::weight(10_000)]
		pub fn burn(origin: OriginFor<T>, amount: Balance) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Self::do_burn(who, amount)?;

			Ok(().into())
		}

		/// Deposit Mining Resource from address to mining treasury
		#[pallet::weight(100_000)]
		pub fn deposit(origin: OriginFor<T>, amount: Balance) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			Self::do_deposit(who, amount)?;
			Ok(().into())
		}

		/// Withdraw Mining Resource from mining engine to destination wallet
		#[pallet::weight(100_000)]
		pub fn withdraw(origin: OriginFor<T>, dest: T::AccountId, amount: Balance) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			Self::do_withdraw(who, dest, amount)?;
			Ok(().into())
		}

		/// Add new Minting Origin to Mining Resource
		#[pallet::weight(100_000)]
		pub fn add_minting_origin(origin: OriginFor<T>, who: T::AccountId) -> DispatchResultWithPostInfo {
			T::AdminOrigin::ensure_origin(origin)?;
			Self::do_add_minting_origin(who)?;
			Ok(().into())
		}

		/// Remove Minting Origin to Mining Resource
		#[pallet::weight(100_000)]
		pub fn remove_minting_origin(origin: OriginFor<T>, who: T::AccountId) -> DispatchResultWithPostInfo {
			T::AdminOrigin::ensure_origin(origin)?;
			Self::do_remove_minting_origin(who)?;
			Ok(().into())
		}

		#[pallet::weight(100_000)]
		pub fn update_round_length(origin: OriginFor<T>, length: T::BlockNumber) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			let mut current_round = Round::<T>::get();
			ensure!(length >= Zero::zero(), Error::<T>::AmountZero);

			current_round.length = length.saturated_into::<u32>();

			Round::<T>::put(current_round);

			Self::deposit_event(Event::<T>::RoundLengthUpdated(length));

			Ok(().into())
		}

		#[pallet::weight(100_000)]
		pub fn update_mining_issuance_config(
			origin: OriginFor<T>,
			config: MiningResourceRateInfo,
		) -> DispatchResultWithPostInfo {
			T::AdminOrigin::ensure_origin(origin)?;
			let round = <Round<T>>::get();
			let current_block = <system::Pallet<T>>::block_number();
			ensure!(!round.should_update(current_block), Error::<T>::RoundUpdateIsOnProgress);

			MiningConfig::<T>::put(config.clone());

			Self::deposit_event(Event::<T>::MiningConfigUpdated(current_block, config));

			Ok(().into())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_initialize(n: T::BlockNumber) -> Weight {
			let mut round = <Round<T>>::get();
			if round.should_update(n) {
				// mutate round
				round.update(n);

				let allocation_range = round_issuance_range::<T>(<MiningConfig<T>>::get());
				Round::<T>::put(round);
				CurrentMiningResourceAllocation::<T>::put(allocation_range);
				Self::deposit_event(Event::NewMiningRound(round.current));
				0
			} else {
				0
			}
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn bit_mining_resource_account_id() -> T::AccountId {
		T::BitMiningTreasury::get().into_account()
	}

	fn bit_mining_resource_currency_id() -> FungibleTokenId {
		T::BitMiningResourceId::get()
	}

	pub fn is_mining_origin(who: &T::AccountId) -> bool {
		let minting_origin = Self::minting_origin(who);
		minting_origin == Some(())
	}

	pub fn ensure_admin(o: T::Origin) -> DispatchResult {
		T::AdminOrigin::try_origin(o).map(|_| ()).or_else(ensure_root)?;
		Ok(())
	}

	fn do_mint(who: T::AccountId, amount: Balance) -> DispatchResult {
		if amount.is_zero() {
			return Ok(());
		}

		ensure!(Self::is_mining_origin(&who), Error::<T>::NoPermission);

		let mining_treasury = Self::bit_mining_resource_account_id();
		//Deposit Bit mining to mining treasury
		T::MiningCurrency::deposit(Self::bit_mining_resource_currency_id(), &mining_treasury, amount)?;

		Self::deposit_event(Event::<T>::MiningResourceMinted(amount));

		Ok(())
	}

	fn do_burn(who: T::AccountId, amount: Balance) -> DispatchResult {
		if amount.is_zero() {
			return Ok(());
		}
		ensure!(Self::is_mining_origin(&who), Error::<T>::NoPermission);

		let mining_treasury = Self::bit_mining_resource_account_id();
		ensure!(
			T::MiningCurrency::can_slash(Self::bit_mining_resource_currency_id(), &mining_treasury, amount),
			Error::<T>::BalanceZero
		);
		//Deposit Bit mining to mining treasury
		T::MiningCurrency::slash(Self::bit_mining_resource_currency_id(), &mining_treasury, amount);

		Self::deposit_event(Event::<T>::MiningResourceBurned(amount));

		Ok(())
	}

	fn do_deposit(who: T::AccountId, amount: Balance) -> DispatchResult {
		if amount.is_zero() {
			return Ok(());
		}

		let mining_treasury = Self::bit_mining_resource_account_id();
		ensure!(
			T::MiningCurrency::free_balance(Self::bit_mining_resource_currency_id(), &who) >= amount,
			Error::<T>::BalanceLow
		);

		T::MiningCurrency::transfer(Self::bit_mining_resource_currency_id(), &who, &mining_treasury, amount)?;

		Self::deposit_event(Event::DepositMiningResource(who, amount.clone()));

		Ok(())
	}

	fn do_withdraw(who: T::AccountId, dest: T::AccountId, amount: Balance) -> DispatchResult {
		if amount.is_zero() || who == dest {
			return Ok(());
		}

		ensure!(Self::is_mining_origin(&who), Error::<T>::NoPermission);

		let mining_treasury = Self::bit_mining_resource_account_id();
		ensure!(
			T::MiningCurrency::free_balance(Self::bit_mining_resource_currency_id(), &mining_treasury) >= amount,
			Error::<T>::BalanceLow
		);

		T::MiningCurrency::transfer(Self::bit_mining_resource_currency_id(), &mining_treasury, &dest, amount)?;

		Self::deposit_event(Event::WithdrawMiningResource(dest, amount.clone()));

		Ok(())
	}

	fn do_add_minting_origin(who: T::AccountId) -> DispatchResult {
		ensure!(!Self::is_mining_origin(&who), Error::<T>::OriginsAlreadyExist);

		MintingOrigins::<T>::insert(who.clone(), ());
		Self::deposit_event(Event::AddNewMiningOrigin(who));
		Ok(())
	}

	fn do_remove_minting_origin(who: T::AccountId) -> DispatchResult {
		ensure!(Self::is_mining_origin(&who), Error::<T>::OriginsIsNotExist);

		MintingOrigins::<T>::remove(who.clone());
		Self::deposit_event(Event::RemoveMiningOrigin(who));
		Ok(())
	}
}

impl<T: Config> RoundTrait<T::BlockNumber> for Pallet<T> {
	fn get_current_round_info() -> RoundInfo<T::BlockNumber> {
		Round::<T>::get()
	}
}
