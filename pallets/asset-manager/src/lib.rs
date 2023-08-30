// This file is part of Metaverse.Network & Bit.Country.

// Copyright (C) 2020-2022 Metaverse.Network & Bit.Country .
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

// This pallet use The Open Runtime Module Library (ORML) which is a community maintained collection
// of Substrate runtime modules. Thanks to all contributors of orml.
// Ref: https://github.com/open-web3-stack/open-runtime-module-library

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::{
	assert_ok,
	dispatch::DispatchResult,
	ensure,
	pallet_prelude::*,
	traits::{Currency, EnsureOrigin},
	transactional,
};
use frame_system::pallet_prelude::*;
use scale_info::prelude::format;
use sp_runtime::{traits::One, ArithmeticError, FixedPointNumber, FixedU128};
use sp_std::{boxed::Box, vec::Vec};
use xcm::v3::MultiLocation;
use xcm::VersionedMultiLocation;

pub use pallet::*;
use primitives::{
	AssetIds, AssetMetadata, BuyWeightRate, CurrencyId, ForeignAssetIdMapping, FungibleTokenId, Ratio, TokenId,
};

mod mock;
mod tests;

/// Type alias for currency balance.
pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use primitives::{AssetIds, AssetMetadata, EvmAddress, TokenId};

	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Currency type for withdraw and balance storage.
		type Currency: Currency<Self::AccountId>;

		/// Required origin for registering asset.
		type RegisterOrigin: EnsureOrigin<Self::RuntimeOrigin>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The given location could not be used (e.g. because it cannot be expressed in the
		/// desired version of XCM).
		BadLocation,
		/// MultiLocation existed
		MultiLocationExisted,
		/// AssetId not exists
		AssetIdNotExists,
		/// AssetId exists
		AssetIdExisted,
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// The foreign asset registered.
		ForeignAssetRegistered {
			asset_id: TokenId,
			asset_address: MultiLocation,
			metadata: AssetMetadata<BalanceOf<T>>,
		},
		/// The foreign asset updated.
		ForeignAssetUpdated {
			asset_id: TokenId,
			asset_address: MultiLocation,
			metadata: AssetMetadata<BalanceOf<T>>,
		},
		/// The asset registered.
		AssetRegistered {
			asset_id: AssetIds,
			metadata: AssetMetadata<BalanceOf<T>>,
		},
		/// The asset updated.
		AssetUpdated {
			asset_id: FungibleTokenId,
			metadata: AssetMetadata<BalanceOf<T>>,
		},
	}

	/// Next available Foreign AssetId ID.
	///
	/// NextForeignAssetId: ForeignAssetId
	#[pallet::storage]
	#[pallet::getter(fn next_foreign_asset_id)]
	pub type NextForeignAssetId<T: Config> = StorageValue<_, TokenId, ValueQuery>;

	/// Next available Stable AssetId ID.
	///
	/// NextStableAssetId: ForeignAssetId
	#[pallet::storage]
	#[pallet::getter(fn next_stable_asset_id)]
	pub type NextStableAssetId<T: Config> = StorageValue<_, TokenId, ValueQuery>;

	/// The storages for MultiLocations.
	///
	/// ForeignAssetLocations: map ForeignAssetId => Option<MultiLocation>
	#[pallet::storage]
	#[pallet::getter(fn foreign_asset_locations)]
	pub type ForeignAssetLocations<T: Config> = StorageMap<_, Twox64Concat, TokenId, MultiLocation, OptionQuery>;

	/// The storages for FungibleTokenId.
	/// Map the MultiLocation with FungibleTokenId
	/// LocationToCurrencyIds: map MultiLocation => Option<FungibleTokenId>
	#[pallet::storage]
	#[pallet::getter(fn location_to_fungible_token_ids)]
	pub type LocationToFungibleTokenIds<T: Config> =
		StorageMap<_, Twox64Concat, MultiLocation, FungibleTokenId, OptionQuery>;

	/// The storages for EvmAddress.
	///
	/// Erc20IdToAddress: map Erc20Id => Option<EvmAddress>
	#[pallet::storage]
	#[pallet::getter(fn erc20_id_to_address)]
	pub type Erc20IdToAddress<T: Config> = StorageMap<_, Twox64Concat, TokenId, EvmAddress, OptionQuery>;

	/// The storages for AssetMetadatas.
	///
	/// AssetMetadatas: map AssetIds => Option<AssetMetadata>
	#[pallet::storage]
	#[pallet::getter(fn asset_metadatas)]
	pub type AssetMetadatas<T: Config> =
		StorageMap<_, Twox64Concat, AssetIds, AssetMetadata<BalanceOf<T>>, OptionQuery>;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::DbWeight::get().read + 2 * T::DbWeight::get().write)]
		#[transactional]
		pub fn register_foreign_asset(
			origin: OriginFor<T>,
			location: Box<VersionedMultiLocation>,
			metadata: Box<AssetMetadata<BalanceOf<T>>>,
		) -> DispatchResult {
			T::RegisterOrigin::ensure_origin(origin)?;

			let location: MultiLocation = (*location).try_into().map_err(|()| Error::<T>::BadLocation)?;
			let foreign_asset_id = Self::do_register_foreign_asset(&location, &metadata)?;

			Self::deposit_event(Event::<T>::ForeignAssetRegistered {
				asset_id: foreign_asset_id,
				asset_address: location,
				metadata: *metadata,
			});

			Ok(())
		}

		#[pallet::weight(T::DbWeight::get().read + 2 * T::DbWeight::get().write)]
		#[transactional]
		pub fn update_foreign_asset(
			origin: OriginFor<T>,
			foreign_asset_id: TokenId,
			location: Box<VersionedMultiLocation>,
			metadata: Box<AssetMetadata<BalanceOf<T>>>,
		) -> DispatchResult {
			T::RegisterOrigin::ensure_origin(origin)?;

			let location: MultiLocation = (*location).try_into().map_err(|()| Error::<T>::BadLocation)?;
			Self::do_update_foreign_asset(foreign_asset_id, &location, &metadata)?;

			Self::deposit_event(Event::<T>::ForeignAssetUpdated {
				asset_id: foreign_asset_id,
				asset_address: location,
				metadata: *metadata,
			});
			Ok(())
		}

		#[pallet::weight(T::DbWeight::get().read + 2 * T::DbWeight::get().write)]
		#[transactional]
		pub fn update_native_asset_metadata(
			origin: OriginFor<T>,
			asset_id: FungibleTokenId,
			metadata: Box<AssetMetadata<BalanceOf<T>>>,
		) -> DispatchResult {
			T::RegisterOrigin::ensure_origin(origin)?;

			Self::do_update_native_asset_metadata(asset_id, &metadata)?;

			Self::deposit_event(Event::<T>::AssetUpdated {
				asset_id: asset_id,
				metadata: *metadata,
			});
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn get_next_stable_asset_id() -> Result<TokenId, DispatchError> {
		NextStableAssetId::<T>::try_mutate(|current| -> Result<TokenId, DispatchError> {
			let id = *current;
			*current = current.checked_add(One::one()).ok_or(ArithmeticError::Overflow)?;
			Ok(id)
		})
	}

	fn get_next_foreign_asset_id() -> Result<TokenId, DispatchError> {
		NextForeignAssetId::<T>::try_mutate(|current| -> Result<TokenId, DispatchError> {
			let id = *current;
			*current = current.checked_add(One::one()).ok_or(ArithmeticError::Overflow)?;
			Ok(id)
		})
	}

	fn do_register_foreign_asset(
		location: &MultiLocation,
		metadata: &AssetMetadata<BalanceOf<T>>,
	) -> Result<TokenId, DispatchError> {
		let foreign_asset_id = Self::get_next_foreign_asset_id()?;
		LocationToFungibleTokenIds::<T>::try_mutate(location, |maybe_currency_ids| -> DispatchResult {
			ensure!(maybe_currency_ids.is_none(), Error::<T>::MultiLocationExisted);
			*maybe_currency_ids = Some(FungibleTokenId::FungibleToken(foreign_asset_id));

			ForeignAssetLocations::<T>::try_mutate(foreign_asset_id, |maybe_location| -> DispatchResult {
				ensure!(maybe_location.is_none(), Error::<T>::MultiLocationExisted);
				*maybe_location = Some(location.clone());

				AssetMetadatas::<T>::try_mutate(
					AssetIds::ForeignAssetId(foreign_asset_id),
					|maybe_asset_metadatas| -> DispatchResult {
						ensure!(maybe_asset_metadatas.is_none(), Error::<T>::AssetIdExisted);

						*maybe_asset_metadatas = Some(metadata.clone());
						Ok(())
					},
				)
			})
		})?;

		Ok(foreign_asset_id)
	}

	fn do_update_foreign_asset(
		foreign_asset_id: TokenId,
		location: &MultiLocation,
		metadata: &AssetMetadata<BalanceOf<T>>,
	) -> DispatchResult {
		ForeignAssetLocations::<T>::try_mutate(foreign_asset_id, |maybe_multi_locations| -> DispatchResult {
			let old_multi_locations = maybe_multi_locations.as_mut().ok_or(Error::<T>::AssetIdNotExists)?;

			AssetMetadatas::<T>::try_mutate(
				AssetIds::ForeignAssetId(foreign_asset_id),
				|maybe_asset_metadatas| -> DispatchResult {
					ensure!(maybe_asset_metadatas.is_some(), Error::<T>::AssetIdNotExists);

					// modify location
					if location != old_multi_locations {
						LocationToFungibleTokenIds::<T>::remove(old_multi_locations.clone());
						LocationToFungibleTokenIds::<T>::try_mutate(
							location,
							|maybe_currency_ids| -> DispatchResult {
								ensure!(maybe_currency_ids.is_none(), Error::<T>::MultiLocationExisted);
								*maybe_currency_ids = Some(FungibleTokenId::FungibleToken(foreign_asset_id));
								Ok(())
							},
						)?;
					}
					*maybe_asset_metadatas = Some(metadata.clone());
					*old_multi_locations = location.clone();
					Ok(())
				},
			)
		})
	}

	fn do_update_native_asset_metadata(
		native_asset_id: FungibleTokenId,
		metadata: &AssetMetadata<BalanceOf<T>>,
	) -> DispatchResult {
		AssetMetadatas::<T>::try_mutate(
			AssetIds::NativeAssetId(native_asset_id),
			|maybe_asset_metadatas| -> DispatchResult {
				*maybe_asset_metadatas = Some(metadata.clone());
				Ok(())
			},
		)
	}
}

pub struct ForeignAssetMapping<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> ForeignAssetIdMapping<TokenId, MultiLocation, AssetMetadata<BalanceOf<T>>> for ForeignAssetMapping<T> {
	fn get_asset_metadata(asset_ids: AssetIds) -> Option<AssetMetadata<BalanceOf<T>>> {
		Pallet::<T>::asset_metadatas(asset_ids)
	}

	fn get_multi_location(foreign_asset_id: TokenId) -> Option<MultiLocation> {
		Pallet::<T>::foreign_asset_locations(foreign_asset_id)
	}

	fn get_currency_id(multi_location: MultiLocation) -> Option<FungibleTokenId> {
		Pallet::<T>::location_to_fungible_token_ids(multi_location)
	}
}

pub struct BuyWeightRateOfForeignAsset<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> BuyWeightRate for BuyWeightRateOfForeignAsset<T>
where
	BalanceOf<T>: Into<u128>,
{
	fn calculate_rate(location: MultiLocation) -> Option<Ratio> {
		if let Some(FungibleTokenId::FungibleToken(foreign_asset_id)) =
			Pallet::<T>::location_to_fungible_token_ids(location)
		{
			if let Some(asset_metadata) = Pallet::<T>::asset_metadatas(AssetIds::ForeignAssetId(foreign_asset_id)) {
				let minimum_balance = asset_metadata.minimal_balance.into();
				let rate = FixedU128::saturating_from_rational(minimum_balance, T::Currency::minimum_balance().into());
				log::debug!(target: "asset-manager::weight", "ForeignAsset: {}, MinimumBalance: {}, rate:{:?}", foreign_asset_id, minimum_balance, rate);
				return Some(rate);
			}
		}
		None
	}
}
