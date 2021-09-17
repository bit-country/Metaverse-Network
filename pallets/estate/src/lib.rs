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
use frame_support::ensure;
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use frame_system::{ensure_root, ensure_signed};
use primitives::{Balance, BitCountryId, CurrencyId, LandId, LandUnitId, EstateId};
use sp_runtime::{
    traits::{AccountIdConversion, One},
    DispatchError, ModuleId, RuntimeDebug,
};
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use frame_support::dispatch::DispatchResult;
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::traits::{
        Currency, ExistenceRequirement, LockableCurrency, ReservableCurrency,
    };

    #[pallet::pallet]
    #[pallet::generate_store(trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        #[pallet::constant]
        type LandTreasury: Get<ModuleId>;
        /// Source of Bit Country Info
        type BitCountryInfoSource: BitCountryTrait<Self::AccountId>;
        /// Currency
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
        /// Minimum Land Price
        type MinimumLandPrice: Get<BalanceOf<Self>>;
        /// Council origin which allows to update max bound
        type CouncilOrigin: EnsureOrigin<Self::Origin>;
    }

    type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::storage]
    #[pallet::getter(fn next_land_id)]
    pub type NextLandId<T: Config> = StorageValue<_, LandId, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_land_owner)]
    pub type LandOwner<T: Config> =
    StorageDoubleMap<_, Twox64Concat, LandId, Twox64Concat, T::AccountId, (), OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_lands_by_owner)]
    pub type LandByOwner<T: Config> =
    StorageMap<_, Blake2_128Concat, T::AccountId, Vec<LandId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn all_lands_count)]
    pub(super) type AllLandsCount<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_land_blocks)]
    pub type LandBlocks<T: Config> =
    StorageDoubleMap<_, Twox64Concat, BitCountryId, Twox64Concat, (i32, i32), (), OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_land_info)]
    pub type LandInfo<T: Config> =
    StorageMap<_, Blake2_128Concat, LandId, (BitCountryId, (i32, i32)), OptionQuery>;


    /// Get max bound
    #[pallet::storage]
    #[pallet::getter(fn get_max_bounds)]
    pub type MaxBounds<T: Config> = StorageMap<_, Blake2_128Concat, BitCountryId, (i32, i32), ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn next_landunit_id)]
    pub type NextLandUnitId<T: Config> = StorageValue<_, LandUnitId, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn all_land_units_count)]
    pub(super) type AllLandUnitsCount<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_land_units)]
    pub type LandUnits<T: Config> =
    StorageDoubleMap<_, Twox64Concat, BitCountryId, Twox64Concat, (i32, i32), (), OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_land_unit)]
    pub type LandUnit<T: Config> =
    StorageMap<_, Blake2_128Concat, LandUnitId, (BitCountryId, (i32, i32)), OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_land_unit_owner)]
    pub type LandUnitOwner<T: Config> =
    StorageDoubleMap<_, Twox64Concat, LandUnitId, Twox64Concat, T::AccountId, (), OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_landunits_by_owner)]
    pub type LandUnitByOwner<T: Config> =
    StorageMap<_, Blake2_128Concat, T::AccountId, Vec<LandUnitId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn next_estate_id)]
    pub type NextEstateId<T: Config> = StorageValue<_, EstateId, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn all_estates_count)]
    pub(super) type AllEstatesCount<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_estates)]
    pub type Estates<T: Config> =
    StorageDoubleMap<_, Twox64Concat, BitCountryId, Twox64Concat, EstateId, Vec<LandUnitId>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_estate_owner)]
    pub type EstateOwner<T: Config> =
    StorageDoubleMap<_, Twox64Concat, EstateId, Twox64Concat, T::AccountId, (), OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_estate_by_owner)]
    pub type EstateByOwner<T: Config> =
    StorageMap<_, Blake2_128Concat, T::AccountId, Vec<EstateId>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    #[pallet::metadata(T::AccountId = "AccountId")]
    pub enum Event<T: Config> {
        NewLandCreated(Vec<LandId>),
        NewLandsMinted(Vec<LandUnitId>),
        TransferredLand(LandId, T::AccountId, T::AccountId),
        NewLandBlockPurchased(LandId, BitCountryId, (i32, i32)),
        TransferredLandUnit(LandUnitId, T::AccountId, T::AccountId),
        TransferredEstate(EstateId, T::AccountId, T::AccountId),
        NewLandUnitMinted(LandUnitId, BitCountryId, (i32, i32)),
        NewEstateMinted(EstateId, BitCountryId, Vec<LandUnitId>),
        MaxBoundSet(BitCountryId, (i32, i32)),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// No permission
        NoPermission,
        /// No available bitcountry id
        NoAvailableBitCountryId,
        /// No available land id
        NoAvailableLandId,
        NoAvailableEstateId,
        /// Insufficient fund
        InsufficientFund,
        /// Land id already exist
        LandIdAlreadyExist,
        EstateIdAlreadyExist,
        /// Land estate is not available
        LandBlockIsNotAvailable,
        /// Land estate is out of bound
        LandBlockIsOutOfBound,
        LandUnitIsNotAvailable,
        LandUnitIsOutOfBound,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub(super) fn set_max_bounds(
            origin: OriginFor<T>,
            bc_id: BitCountryId,
            new_bound: (i32, i32),
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            MaxBounds::<T>::insert(bc_id, new_bound);

            Self::deposit_event(Event::<T>::MaxBoundSet(bc_id, new_bound));

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub(super) fn mint_land(
            origin: OriginFor<T>,
            bc_id: BitCountryId,
            coordinate: (i32, i32),
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            /// Check whether the coordinate is exists
            ensure!(
                !LandUnits::<T>::contains_key(bc_id, coordinate),
                Error::<T>::LandUnitIsNotAvailable
            );

            /// Check whether the coordinate is within the bound
            let max_bound = MaxBounds::<T>::get(bc_id);
            ensure!(
                (coordinate.0 >= max_bound.0 && max_bound.1 >= coordinate.0)
                    && (coordinate.1 >= max_bound.0 && max_bound.1 >= coordinate.1),
                Error::<T>::LandUnitIsOutOfBound
            );

            /// Generate new land id
            let new_land_unit_id = Self::get_new_land_unit_id()?;

            /// Update total land count
            let total_land_units_count = Self::all_land_units_count();
            let new_total_land_units_count = total_land_units_count
                .checked_add(One::one())
                .ok_or("Overflow adding new count to total lands")?;
            AllLandUnitsCount::<T>::put(new_total_land_units_count);

            /// Update land blocks
            LandUnits::<T>::insert(bc_id, coordinate, ());

            Self::deposit_event(Event::<T>::NewLandUnitMinted(
                new_land_unit_id.clone(),
                bc_id,
                coordinate,
            ));

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub(super) fn mint_lands(
            origin: OriginFor<T>,
            bc_id: BitCountryId,
            coordinates: Vec<(i32, i32)>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            let max_bound = MaxBounds::<T>::get(bc_id);

            let mut new_land_unit_ids: Vec<LandUnitId> = Vec::new();
            for coordinate in &coordinates {
                /// Check whether the coordinate is within the bound
                ensure!( (coordinate.0 >= max_bound.0 && max_bound.1 >= coordinate.0)
                    && (coordinate.1 >= max_bound.0 && max_bound.1 >= coordinate.1),
                Error::<T>::LandUnitIsOutOfBound);

                /// Generate new land id
                let new_land_unit_id = Self::get_new_land_unit_id()?;
                new_land_unit_ids.push(new_land_unit_id);

                LandUnits::<T>::insert(bc_id, coordinate, ());
            };

            /// Update total land count
            let total_land_unit_count = Self::all_land_units_count();

            let new_total_land_unit_count = total_land_unit_count
                .checked_add(coordinates.len() as u64)
                .ok_or("Overflow adding new count to total lands")?;
            AllLandUnitsCount::<T>::put(new_total_land_unit_count);
            Self::deposit_event(Event::<T>::NewLandsMinted(new_land_unit_ids.clone()));

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub(super) fn transfer_land(
            origin: OriginFor<T>,
            to: T::AccountId,
            land_unit_id: LandUnitId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            if who == to {
                /// no change needed
                return Ok(().into());
            }

            LandUnitOwner::<T>::insert(land_unit_id.clone(), to.clone(), ());

            Self::add_land_unit_to_new_owner(land_unit_id, &who);
            Self::deposit_event(Event::<T>::TransferredLandUnit(
                land_unit_id.clone(),
                who.clone(),
                to,
            ));

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub(super) fn mint_estate(
            origin: OriginFor<T>,
            bc_id: BitCountryId,
            land_unit_ids: Vec<LandUnitId>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            /// TODO: Check whether any of the land unit is being used anywhere else

            /// Generate new estate id
            let new_estate_id = Self::get_new_estate_id()?;

            /// Update total estates
            let total_estates_count = Self::all_estates_count();
            let new_total_estates_count = total_estates_count
                .checked_add(One::one())
                .ok_or("Overflow adding new count to total estates")?;
            AllEstatesCount::<T>::put(new_total_estates_count);

            /// Update land blocks
            Estates::<T>::insert(bc_id, new_estate_id, &land_unit_ids);

            Self::deposit_event(Event::<T>::NewEstateMinted(
                new_estate_id.clone(),
                bc_id,
                land_unit_ids,
            ));

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub(super) fn transfer_estate(
            origin: OriginFor<T>,
            to: T::AccountId,
            estate_id: EstateId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            if who == to {
                /// no change needed
                return Ok(().into());
            }

            EstateOwner::<T>::insert(estate_id.clone(), to.clone(), ());

            Self::add_estate_to_new_owner(estate_id, &who);
            Self::deposit_event(Event::<T>::TransferredEstate(
                estate_id.clone(),
                who.clone(),
                to,
            ));

            Ok(().into())
        }


        #[pallet::weight(10_000)]
        pub(super) fn buy_land_block(
            origin: OriginFor<T>,
            bc_id: BitCountryId,
            coordinate: (i32, i32),
        ) -> DispatchResultWithPostInfo {
            /// Check ownership
            let sender = ensure_signed(origin)?;

            ensure!(
                T::BitCountryInfoSource::check_ownership(&sender, &bc_id),
                Error::<T>::NoPermission
            );

            /// Check whether the coordinate is exists
            ensure!(
                !LandBlocks::<T>::contains_key(bc_id, coordinate),
                Error::<T>::LandBlockIsNotAvailable
            );

            /// Check whether the coordinate is within the bound
            let max_bound = MaxBounds::<T>::get(bc_id);
            ensure!(
                (coordinate.0 >= max_bound.0 && max_bound.1 >= coordinate.0)
                    && (coordinate.1 >= max_bound.0 && max_bound.1 >= coordinate.1),
                Error::<T>::LandBlockIsOutOfBound
            );

            /// Check minimum balance and transfer
            let minimum_land_price = T::MinimumLandPrice::get();
            ensure!(
                T::Currency::free_balance(&sender) > minimum_land_price,
                Error::<T>::InsufficientFund
            );
            let land_treasury = Self::account_id();
            T::Currency::transfer(
                &sender,
                &land_treasury,
                minimum_land_price,
                ExistenceRequirement::KeepAlive,
            )?;

            /// Generate new land id
            let new_land_id = Self::get_new_land_id()?;

            /// Add to land owners
            LandOwner::<T>::insert(new_land_id, &sender, ());
            Self::add_land_to_new_owner(new_land_id, &sender);

            /// Update total land count
            let total_land_count = Self::all_lands_count();
            let new_total_land_count = total_land_count
                .checked_add(One::one())
                .ok_or("Overflow adding new count to total lands")?;
            AllLandsCount::<T>::put(new_total_land_count);

            /// Update land info
            LandInfo::<T>::insert(new_land_id, (bc_id, coordinate));

            /// Update land blocks
            LandBlocks::<T>::insert(bc_id, coordinate, ());

            Self::deposit_event(Event::<T>::NewLandBlockPurchased(
                new_land_id.clone(),
                bc_id,
                coordinate,
            ));

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub(super) fn buy_land(
            origin: OriginFor<T>,
            bc_id: BitCountryId,
            quantity: u8,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                T::BitCountryInfoSource::check_ownership(&sender, &bc_id),
                Error::<T>::NoPermission
            );

            let minimum_land_price = T::MinimumLandPrice::get();
            let total_cost = minimum_land_price * Into::<BalanceOf<T>>::into(quantity);
            ensure!(
                T::Currency::free_balance(&sender) > total_cost,
                Error::<T>::InsufficientFund
            );
            let land_treasury = Self::account_id();
            T::Currency::transfer(
                &sender,
                &land_treasury,
                total_cost,
                ExistenceRequirement::KeepAlive,
            )?;

            let mut new_land_ids: Vec<LandId> = Vec::new();

            for _ in 0..quantity {
                let land_id = Self::get_new_land_id()?;
                new_land_ids.push(land_id);

                LandOwner::<T>::insert(land_id, &sender, ());

                Self::add_land_to_new_owner(land_id, &sender);
            }

            let total_land_count = Self::all_lands_count();

            let new_total_land_count = total_land_count
                .checked_add(quantity.into())
                .ok_or("Overflow adding new count to total lands")?;
            AllLandsCount::<T>::put(new_total_land_count);
            Self::deposit_event(Event::<T>::NewLandCreated(new_land_ids.clone()));

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub(super) fn transfer_land_old(
            origin: OriginFor<T>,
            to: T::AccountId,
            land_id: LandId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            /// Get owner of the land
            LandOwner::<T>::try_mutate_exists(
                &land_id,
                &who,
                |land_by_owner| -> DispatchResultWithPostInfo {
                    //ensure there is record of the land owner with land id, account id and delete them
                    ensure!(land_by_owner.is_some(), Error::<T>::NoPermission);

                    if who == to {
                        /// no change needed
                        return Ok(().into());
                    }

                    *land_by_owner = None;
                    LandOwner::<T>::insert(land_id.clone(), to.clone(), ());

                    Self::add_land_to_new_owner(land_id, &who);
                    Self::deposit_event(Event::<T>::TransferredLand(
                        land_id.clone(),
                        who.clone(),
                        to,
                    ));

                    Ok(().into())
                },
            )
        }

        #[pallet::weight(10_000)]
        pub(super) fn buy_estate(
            origin: OriginFor<T>,
            bc_id: BitCountryId,
            blockId: LandId,
            coordinate: (i32, i32),
            quantity: u8,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(
                T::BitCountryInfoSource::check_ownership(&sender, &bc_id),
                Error::<T>::NoPermission
            );

            let minimum_land_price = T::MinimumLandPrice::get();
            let total_cost = minimum_land_price * Into::<BalanceOf<T>>::into(quantity);
            ensure!(
                T::Currency::free_balance(&sender) > total_cost,
                Error::<T>::InsufficientFund
            );
            let land_treasury = Self::account_id();
            T::Currency::transfer(
                &sender,
                &land_treasury,
                total_cost,
                ExistenceRequirement::KeepAlive,
            )?;

            let mut new_land_ids: Vec<LandId> = Vec::new();

            for _ in 0..quantity {
                let land_id = Self::get_new_land_id()?;
                new_land_ids.push(land_id);

                LandOwner::<T>::insert(land_id, &sender, ());

                Self::add_land_to_new_owner(land_id, &sender);
            }

            let total_land_count = Self::all_lands_count();

            let new_total_land_count = total_land_count
                .checked_add(quantity.into())
                .ok_or("Overflow adding new count to total lands")?;
            AllLandsCount::<T>::put(new_total_land_count);
            Self::deposit_event(Event::<T>::NewLandCreated(new_land_ids.clone()));

            Ok(().into())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}

impl<T: Config> Module<T> {
    /// Reads the nonce from storage, increments the stored nonce, and returns
    /// the encoded nonce to the caller.

    fn get_new_land_id() -> Result<LandId, DispatchError> {
        let land_id = NextLandId::<T>::try_mutate(|id| -> Result<LandId, DispatchError> {
            let current_id = *id;
            *id = id
                .checked_add(One::one())
                .ok_or(Error::<T>::NoAvailableLandId)?;
            Ok(current_id)
        })?;
        Ok(land_id)
    }

    fn account_id() -> T::AccountId {
        T::LandTreasury::get().into_account()
    }

    fn add_land_to_new_owner(land_id: LandId, sender: &T::AccountId) -> DispatchResult {
        if LandOwner::<T>::contains_key(land_id, &sender) {
            LandByOwner::<T>::try_mutate(&sender, |land_ids| -> DispatchResult {
                /// Check if the asset_id already in the owner
                ensure!(
                    !land_ids.iter().any(|i| land_id == *i),
                    Error::<T>::LandIdAlreadyExist
                );
                land_ids.push(land_id);
                Ok(())
            })?;
        } else {
            let mut new_land_vec = Vec::<LandId>::new();
            new_land_vec.push(land_id);
            LandByOwner::<T>::insert(&sender, new_land_vec)
        }
        Ok(())
    }

    fn add_land_unit_to_new_owner(land_unit_id: LandUnitId, sender: &T::AccountId) -> DispatchResult {
        if LandUnitOwner::<T>::contains_key(land_unit_id, &sender) {
            LandUnitByOwner::<T>::try_mutate(&sender, |land_unit_ids| -> DispatchResult {
                /// Check if the asset_id already in the owner
                ensure!(
                    !land_unit_ids.iter().any(|i| land_unit_id == *i),
                    Error::<T>::LandIdAlreadyExist
                );
                land_unit_ids.push(land_unit_id);
                Ok(())
            })?;
        } else {
            let mut new_land_unit_vec = Vec::<LandUnitId>::new();
            new_land_unit_vec.push(land_unit_id);
            LandUnitByOwner::<T>::insert(&sender, new_land_unit_vec)
        }
        Ok(())
    }

    fn add_estate_to_new_owner(estate_id: EstateId, sender: &T::AccountId) -> DispatchResult {
        if EstateOwner::<T>::contains_key(estate_id, &sender) {
            EstateByOwner::<T>::try_mutate(&sender, |estate_ids| -> DispatchResult {
                /// Check if the estate_id already in the owner
                ensure!(
                    !estate_ids.iter().any(|i| estate_id == *i),
                    Error::<T>::EstateIdAlreadyExist
                );
                estate_ids.push(estate_id);
                Ok(())
            })?;
        } else {
            let mut new_land_unit_vec = Vec::<EstateId>::new();
            new_land_unit_vec.push(estate_id);
            EstateByOwner::<T>::insert(&sender, new_land_unit_vec)
        }
        Ok(())
    }

    fn get_new_land_unit_id() -> Result<LandUnitId, DispatchError> {
        let land_unit_id = NextLandUnitId::<T>::try_mutate(|id| -> Result<LandUnitId, DispatchError> {
            let current_id = *id;
            *id = id
                .checked_add(One::one())
                .ok_or(Error::<T>::NoAvailableLandId)?;
            Ok(current_id)
        })?;
        Ok(land_unit_id)
    }

    fn get_new_estate_id() -> Result<EstateId, DispatchError> {
        let estate_id = NextEstateId::<T>::try_mutate(|id| -> Result<EstateId, DispatchError> {
            let current_id = *id;
            *id = id
                .checked_add(One::one())
                .ok_or(Error::<T>::NoAvailableEstateId)?;
            Ok(current_id)
        })?;
        Ok(estate_id)
    }
}
