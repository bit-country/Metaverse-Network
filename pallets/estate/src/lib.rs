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
use frame_support::pallet_prelude::*;
use frame_support::{dispatch::DispatchResult, ensure, traits::Get, PalletId};
use frame_system::pallet_prelude::*;
use frame_system::{ensure_root, ensure_signed};
use primitives::{Balance, CurrencyId, EstateId, LandId, MetaverseId};
use sp_runtime::{
	print,
	traits::{AccountIdConversion, One},
	DispatchError, RuntimeDebug,
};
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::traits::{Currency, ExistenceRequirement, LockableCurrency, ReservableCurrency};
	use primitives::AccountId;

	#[pallet::pallet]
	#[pallet::generate_store(trait Store)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		#[pallet::constant]
		type LandTreasury: Get<PalletId>;
		/// Source of Bit Country Info
		type MetaverseInfoSource: MetaverseTrait<Self::AccountId>;
		/// Currency
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		/// Minimum Land Price
		type MinimumLandPrice: Get<BalanceOf<Self>>;
		/// Council origin which allows to update max bound
		type CouncilOrigin: EnsureOrigin<Self::Origin>;
	}

	type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Get max bound
	#[pallet::storage]
	#[pallet::getter(fn get_max_bounds)]
	pub type MaxBounds<T: Config> = StorageMap<_, Blake2_128Concat, MetaverseId, (i32, i32), ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn all_land_units_count)]
	pub(super) type AllLandUnitsCount<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_land_units)]
	pub type LandUnits<T: Config> =
		StorageDoubleMap<_, Twox64Concat, MetaverseId, Twox64Concat, (i32, i32), T::AccountId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_estate_id)]
	pub type NextEstateId<T: Config> = StorageValue<_, EstateId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn all_estates_count)]
	pub(super) type AllEstatesCount<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_estates)]
	pub type Estates<T: Config> =
		StorageDoubleMap<_, Twox64Concat, MetaverseId, Twox64Concat, EstateId, Vec<(i32, i32)>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_estate_owner)]
	pub type EstateOwner<T: Config> =
		StorageDoubleMap<_, Twox64Concat, EstateId, Twox64Concat, T::AccountId, (), OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	#[pallet::metadata(T::AccountId = "AccountId")]
	pub enum Event<T: Config> {
		NewLandsMinted(MetaverseId, Vec<(i32, i32)>),
		TransferredLandUnit(MetaverseId, (i32, i32), T::AccountId, T::AccountId),
		TransferredEstate(EstateId, T::AccountId, T::AccountId),
		NewLandUnitMinted(MetaverseId, (i32, i32)),
		NewEstateMinted(EstateId, MetaverseId, Vec<(i32, i32)>),
		MaxBoundSet(MetaverseId, (i32, i32)),
	}

	#[pallet::error]
	pub enum Error<T> {
		// No permission
		NoPermission,
		// NoAvailableLandId,
		NoAvailableEstateId,
		// Insufficient fund
		InsufficientFund,
		// Estate id already exist
		EstateIdAlreadyExist,
		// Land unit is not available
		LandUnitIsNotAvailable,
		// Land unit is out of bound
		LandUnitIsOutOfBound,
		// No max bound set
		NoMaxBoundSet,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn set_max_bounds(
			origin: OriginFor<T>,
			metaverse_id: MetaverseId,
			new_bound: (i32, i32),
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			MaxBounds::<T>::insert(metaverse_id, new_bound);

			Self::deposit_event(Event::<T>::MaxBoundSet(metaverse_id, new_bound));

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn mint_land(
			origin: OriginFor<T>,
			beneficiary: T::AccountId,
			metaverse_id: MetaverseId,
			coordinate: (i32, i32),
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			// Mint land unit
			Self::mint_land_unit(metaverse_id, &beneficiary, coordinate, false)?;

			// Update total land count
			let total_land_units_count = Self::all_land_units_count();
			let new_total_land_units_count = total_land_units_count
				.checked_add(One::one())
				.ok_or("Overflow adding new count to total lands")?;
			AllLandUnitsCount::<T>::put(new_total_land_units_count);

			// Update land units
			LandUnits::<T>::insert(metaverse_id, coordinate, beneficiary.clone());

			Self::deposit_event(Event::<T>::NewLandUnitMinted(metaverse_id, coordinate));

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn mint_lands(
			origin: OriginFor<T>,
			beneficiary: T::AccountId,
			metaverse_id: MetaverseId,
			coordinates: Vec<(i32, i32)>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			// Mint land units
			for coordinate in coordinates.clone() {
				Self::mint_land_unit(metaverse_id, &beneficiary, coordinate, false)?;
			}

			// Update total land count
			let total_land_unit_count = Self::all_land_units_count();

			let new_total_land_unit_count = total_land_unit_count
				.checked_add(coordinates.len() as u64)
				.ok_or("Overflow adding new count to total lands")?;
			AllLandUnitsCount::<T>::put(new_total_land_unit_count);
			Self::deposit_event(Event::<T>::NewLandsMinted(metaverse_id.clone(), coordinates.clone()));

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn transfer_land(
			origin: OriginFor<T>,
			to: T::AccountId,
			metaverse_id: MetaverseId,
			coordinate: (i32, i32),
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			LandUnits::<T>::try_mutate_exists(
				&metaverse_id,
				&coordinate,
				|land_unit_owner| -> DispatchResultWithPostInfo {
					// ensure there is record of the land unit with bit country id and coordinate
					ensure!(land_unit_owner.is_some(), Error::<T>::NoPermission);

					// ensure the land unit is belong to the correct owner
					// let mut land_unit_owner_record = land_unit_owner.as_mut().ok_or(Error::<T>::NoPermission)?;

					let owner = land_unit_owner.as_ref().map(|(t)| t);
					ensure!(owner == Some(&who), Error::<T>::NoPermission);

					if who == to {
						// no change needed
						return Ok(().into());
					}

					*land_unit_owner = None;
					LandUnits::<T>::insert(metaverse_id.clone(), coordinate.clone(), to.clone());

					// Update
					Self::deposit_event(Event::<T>::TransferredLandUnit(
						metaverse_id.clone(),
						coordinate.clone(),
						who.clone(),
						to.clone(),
					));

					Ok(().into())
				},
			)
		}

		/// Mint new estate with no existing land unit
		#[pallet::weight(10_000)]
		pub fn mint_estate(
			origin: OriginFor<T>,
			beneficiary: T::AccountId,
			metaverse_id: MetaverseId,
			coordinates: Vec<(i32, i32)>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			// Generate new estate id
			let new_estate_id = Self::get_new_estate_id()?;

			// Generate sub account from estate
			let estate_account_id: T::AccountId = T::LandTreasury::get().into_sub_account(new_estate_id);

			// Mint land units
			for coordinate in coordinates.clone() {
				Self::mint_land_unit(metaverse_id, &estate_account_id, coordinate, false)?;
			}

			// Update estate information
			Self::update_estate_information(new_estate_id, metaverse_id, &beneficiary, coordinates);

			Ok(().into())
		}

		/// Create new estate from existing land units
		#[pallet::weight(10_000)]
		pub fn create_estate(
			origin: OriginFor<T>,
			beneficiary: T::AccountId,
			metaverse_id: MetaverseId,
			coordinates: Vec<(i32, i32)>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			// Generate new estate id
			let new_estate_id = Self::get_new_estate_id()?;

			// Generate sub account from estate
			let estate_account_id: T::AccountId = T::LandTreasury::get().into_sub_account(new_estate_id);

			// Mint land units
			for coordinate in coordinates.clone() {
				Self::mint_land_unit(metaverse_id, &estate_account_id, coordinate, true)?;
			}

			// Update estate information
			Self::update_estate_information(new_estate_id, metaverse_id, &beneficiary, coordinates.clone());

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn transfer_estate(
			origin: OriginFor<T>,
			to: T::AccountId,
			estate_id: EstateId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			EstateOwner::<T>::try_mutate_exists(&estate_id, &who, |estate_by_owner| -> DispatchResultWithPostInfo {
				//ensure there is record of the estate owner with estate id and account id
				ensure!(estate_by_owner.is_some(), Error::<T>::NoPermission);

				if who == to {
					// no change needed
					return Ok(().into());
				}

				*estate_by_owner = None;
				EstateOwner::<T>::insert(estate_id.clone(), to.clone(), ());

				Self::deposit_event(Event::<T>::TransferredEstate(estate_id.clone(), who.clone(), to));

				Ok(().into())
			})
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}

impl<T: Config> Pallet<T> {
	// Reads the nonce from storage, increments the stored nonce, and returns
	// the encoded nonce to the caller.
	fn account_id() -> T::AccountId {
		T::LandTreasury::get().into_account()
	}

	fn get_new_estate_id() -> Result<EstateId, DispatchError> {
		let estate_id = NextEstateId::<T>::try_mutate(|id| -> Result<EstateId, DispatchError> {
			let current_id = *id;
			*id = id.checked_add(One::one()).ok_or(Error::<T>::NoAvailableEstateId)?;
			Ok(current_id)
		})?;
		Ok(estate_id)
	}

	fn mint_land_unit(
		metaverse_id: MetaverseId,
		beneficiary: &T::AccountId,
		coordinate: (i32, i32),
		existing_land_units: bool,
	) -> DispatchResult {
		// Ensure the max bound is set for the bit country
		ensure!(MaxBounds::<T>::contains_key(metaverse_id), Error::<T>::NoMaxBoundSet);

		let max_bound = MaxBounds::<T>::get(metaverse_id);

		if existing_land_units {
			let key = LandUnits::<T>::contains_key(metaverse_id, coordinate);
			// Check whether the coordinate exists
			ensure!(
				LandUnits::<T>::contains_key(metaverse_id, coordinate),
				Error::<T>::LandUnitIsNotAvailable
			);
		} else {
			ensure!(
				!LandUnits::<T>::contains_key(metaverse_id, coordinate),
				Error::<T>::LandUnitIsNotAvailable
			);
		}

		// Check whether the coordinate is within the bound
		ensure!(
			(coordinate.0 >= max_bound.0 && max_bound.1 >= coordinate.0)
				&& (coordinate.1 >= max_bound.0 && max_bound.1 >= coordinate.1),
			Error::<T>::LandUnitIsOutOfBound
		);

		LandUnits::<T>::insert(metaverse_id, coordinate, beneficiary.clone());
		Ok(())
	}

	fn update_estate_information(
		new_estate_id: EstateId,
		metaverse_id: MetaverseId,
		beneficiary: &T::AccountId,
		coordinates: Vec<(i32, i32)>,
	) -> DispatchResult {
		// Update total estates
		let total_estates_count = Self::all_estates_count();
		let new_total_estates_count = total_estates_count
			.checked_add(One::one())
			.ok_or("Overflow adding new count to total estates")?;
		AllEstatesCount::<T>::put(new_total_estates_count);

		// Update estates
		Estates::<T>::insert(metaverse_id, new_estate_id, coordinates.clone());

		EstateOwner::<T>::insert(new_estate_id, beneficiary.clone(), {});

		Self::deposit_event(Event::<T>::NewEstateMinted(
			new_estate_id.clone(),
			metaverse_id,
			coordinates.clone(),
		));

		Ok(())
	}
}
