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

#![cfg_attr(not(feature = "std"), no_std)]

use bc_primitives::*;
use codec::{Decode, Encode};
use frame_support::{ensure, pallet_prelude::*, traits::Currency, BoundedVec, PalletId};
use frame_system::{ensure_root, ensure_signed, pallet_prelude::*};
use primitives::{Balance, CurrencyId, FungibleTokenId, MetaverseId};
use sp_runtime::{
	traits::{AccountIdConversion, One},
	DispatchError, DispatchResult, RuntimeDebug,
};
use sp_std::{convert::TryInto, vec::Vec};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use bc_primitives::{MetaverseInfo, MetaverseTrait};
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::traits::ExistenceRequirement;

	#[pallet::pallet]
	#[pallet::generate_store(trait Store)]
	pub struct Pallet<T>(PhantomData<T>);

	type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The currency type
		type Currency: Currency<Self::AccountId>;
		#[pallet::constant]
		type MetaverseTreasury: Get<PalletId>;
		#[pallet::constant]
		type MaxMetaverseMetadata: Get<u32>;
		/// Minimum contribution
		#[pallet::constant]
		type MinContribution: Get<BalanceOf<Self>>;
		/// Origin to add new metaverse
		type MetaverseCouncil: EnsureOrigin<Self::Origin>;
	}

	#[pallet::storage]
	#[pallet::getter(fn next_metaverse_id)]
	pub type NextMetaverseId<T: Config> = StorageValue<_, MetaverseId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_metaverse)]
	pub type Metaverses<T: Config> = StorageMap<_, Twox64Concat, MetaverseId, MetaverseInfo<T::AccountId>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_metaverse_owner)]
	pub type MetaverseOwner<T: Config> =
		StorageDoubleMap<_, Twox64Concat, MetaverseId, Twox64Concat, T::AccountId, (), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn all_metaverse_count)]
	pub(super) type AllMetaversesCount<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_freezing_metaverse)]
	pub(super) type FreezedMetaverses<T: Config> = StorageMap<_, Twox64Concat, MetaverseId, (), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn is_init)]
	pub(super) type Init<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn nonce)]
	pub(super) type Nonce<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewMetaverseCreated(MetaverseId, T::AccountId),
		TransferredMetaverse(MetaverseId, T::AccountId, T::AccountId),
		MetaverseFreezed(MetaverseId),
		MetaverseDestroyed(MetaverseId),
		MetaverseUnfreezed(MetaverseId),
		MetaverseMintedNewCurrency(MetaverseId, FungibleTokenId),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Metaverse info not found
		MetaverseInfoNotFound,
		/// Metaverse Id not found
		MetaverseIdNotFound,
		/// No permission
		NoPermission,
		/// No available Metaverse id
		NoAvailableMetaverseId,
		/// Fungible token already issued
		FungibleTokenAlreadyIssued,
		/// Max metadata exceed
		MaxMetadataExceeded,
		/// Contribution is insufficient
		InsufficientContribution,
		/// Only frozen metaverse can be destroy
		OnlyFrozenMetaverseCanBeDestroyed,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create_metaverse(origin: OriginFor<T>, metadata: MetaverseMetadata) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;

			ensure!(
				metadata.len() as u32 <= T::MaxMetaverseMetadata::get(),
				Error::<T>::MaxMetadataExceeded
			);

			ensure!(
				T::Currency::free_balance(&from) >= T::MinContribution::get(),
				Error::<T>::InsufficientContribution
			);

			T::Currency::transfer(
				&from,
				&Self::account_id(),
				T::MinContribution::get(),
				ExistenceRequirement::KeepAlive,
			)?;

			let metaverse_id = Self::new_metaverse(&from, metadata)?;

			MetaverseOwner::<T>::insert(metaverse_id, from.clone(), ());

			let total_metaverse_count = Self::all_metaverse_count();
			let new_total_metaverse_count = total_metaverse_count
				.checked_add(One::one())
				.ok_or("Overflow adding new count to new_total_metaverse_count")?;
			AllMetaversesCount::<T>::put(new_total_metaverse_count);
			Self::deposit_event(Event::<T>::NewMetaverseCreated(metaverse_id.clone(), from));

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn transfer_metaverse(
			origin: OriginFor<T>,
			to: T::AccountId,
			metaverse_id: MetaverseId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			/// Get owner of the metaverse
			MetaverseOwner::<T>::try_mutate_exists(
				&metaverse_id,
				&who,
				|metaverse_by_owner| -> DispatchResultWithPostInfo {
					/// Ensure there is record of the metaverse owner with metaverse id, account
					/// id and delete them
					ensure!(metaverse_by_owner.is_some(), Error::<T>::NoPermission);

					if who == to {
						/// No change needed
						return Ok(().into());
					}

					*metaverse_by_owner = None;
					MetaverseOwner::<T>::insert(metaverse_id.clone(), to.clone(), ());

					Metaverses::<T>::try_mutate_exists(&metaverse_id, |metaverse| -> DispatchResultWithPostInfo {
						let mut metaverse_record = metaverse.as_mut().ok_or(Error::<T>::NoPermission)?;
						metaverse_record.owner = to.clone();
						Self::deposit_event(Event::<T>::TransferredMetaverse(metaverse_id, who.clone(), to.clone()));

						Ok(().into())
					})
				},
			)
		}

		#[pallet::weight(10_000)]
		pub fn freeze_metaverse(origin: OriginFor<T>, metaverse_id: MetaverseId) -> DispatchResultWithPostInfo {
			/// Only Council can free a metaverse
			T::MetaverseCouncil::ensure_origin(origin)?;

			Metaverses::<T>::try_mutate(metaverse_id, |maybe_metaverse| {
				let metaverse_info = maybe_metaverse.as_mut().ok_or(Error::<T>::MetaverseInfoNotFound)?;

				metaverse_info.is_frozen = true;

				Self::deposit_event(Event::<T>::MetaverseFreezed(metaverse_id));

				Ok(().into())
			})
		}

		#[pallet::weight(10_000)]
		pub fn unfreeze_metaverse(origin: OriginFor<T>, metaverse_id: MetaverseId) -> DispatchResultWithPostInfo {
			/// Only Council can free a metaverse
			T::MetaverseCouncil::ensure_origin(origin)?;

			Metaverses::<T>::try_mutate(metaverse_id, |maybe_metaverse| {
				let metaverse_info = maybe_metaverse.as_mut().ok_or(Error::<T>::MetaverseInfoNotFound)?;

				metaverse_info.is_frozen = false;

				Self::deposit_event(Event::<T>::MetaverseUnfreezed(metaverse_id));

				Ok(().into())
			})
		}

		#[pallet::weight(10_000)]
		pub fn destroy_metaverse(origin: OriginFor<T>, metaverse_id: MetaverseId) -> DispatchResultWithPostInfo {
			/// Only Council can destroy a metaverse
			T::MetaverseCouncil::ensure_origin(origin)?;

			let metaverse_info = Metaverses::<T>::get(metaverse_id).ok_or(Error::<T>::MetaverseInfoNotFound)?;

			ensure!(metaverse_info.is_frozen, Error::<T>::OnlyFrozenMetaverseCanBeDestroyed);

			MetaverseOwner::<T>::remove(&metaverse_id, metaverse_info.owner);
			Metaverses::<T>::remove(&metaverse_id);
			Self::deposit_event(Event::<T>::MetaverseDestroyed(metaverse_id));
			Ok(().into())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}

impl<T: Config> Pallet<T> {
	/// Reads the nonce from storage, increments the stored nonce, and returns
	/// the encoded nonce to the caller.

	fn new_metaverse(owner: &T::AccountId, metadata: MetaverseMetadata) -> Result<MetaverseId, DispatchError> {
		let metaverse_id = NextMetaverseId::<T>::try_mutate(|id| -> Result<MetaverseId, DispatchError> {
			let current_id = *id;
			*id = id.checked_add(One::one()).ok_or(Error::<T>::NoAvailableMetaverseId)?;
			Ok(current_id)
		})?;

		let metaverse_info = MetaverseInfo {
			owner: owner.clone(),
			currency_id: FungibleTokenId::NativeToken(0),
			metadata,
			is_frozen: false,
		};

		Metaverses::<T>::insert(metaverse_id, metaverse_info);

		Ok(metaverse_id)
	}

	/// The account ID of the treasury pot.
	///
	/// This actually does computation. If you need to keep using it, then make sure you cache the
	/// value and only call this once.
	pub fn account_id() -> T::AccountId {
		T::MetaverseTreasury::get().into_account()
	}
}

impl<T: Config> MetaverseTrait<T::AccountId> for Pallet<T> {
	fn check_ownership(who: &T::AccountId, metaverse_id: &MetaverseId) -> bool {
		Self::get_metaverse_owner(metaverse_id, who) == Some(())
	}

	fn get_metaverse(metaverse_id: MetaverseId) -> Option<MetaverseInfo<T::AccountId>> {
		Self::get_metaverse(metaverse_id)
	}

	fn get_metaverse_token(metaverse_id: MetaverseId) -> Option<FungibleTokenId> {
		if let Some(country) = Self::get_metaverse(metaverse_id) {
			return Some(country.currency_id);
		}
		None
	}

	fn update_metaverse_token(metaverse_id: MetaverseId, currency_id: FungibleTokenId) -> Result<(), DispatchError> {
		Metaverses::<T>::try_mutate_exists(&metaverse_id, |metaverse| {
			let mut metaverse_record = metaverse.as_mut().ok_or(Error::<T>::NoPermission)?;

			ensure!(
				metaverse_record.currency_id == FungibleTokenId::NativeToken(0),
				Error::<T>::FungibleTokenAlreadyIssued
			);

			metaverse_record.currency_id = currency_id.clone();
			Self::deposit_event(Event::<T>::MetaverseMintedNewCurrency(metaverse_id, currency_id));
			Ok(())
		})
	}
}
