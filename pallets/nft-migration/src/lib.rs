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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
use frame_support::{
	pallet_prelude::*,
	traits::{Contains, GetCallMetadata, PalletInfoAccess},
};
use frame_system::offchain::{CreateSignedTransaction, SendUnsignedTransaction, SubmitTransaction};
use frame_system::pallet_prelude::*;
use sp_runtime::{offchain::http::Request, DispatchResult};
use sp_std::prelude::*;

use core_primitives::{
	CollectionType, NFTMigrationTrait, NftAssetData, NftClassData, NftGroupCollectionData, NftMetadata, TokenType,
};
use primitives::{Attributes, ClassId, GroupCollectionId, TokenId};

pub use pallet::*;
pub use weights::WeightInfo;

mod mock;
mod tests;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
pub mod weights;

pub const PIONEER_COLLECTIONS_HTTP_ENDPOINT: &str = "";
pub const PIONEER_CLASSES_HTTP_ENDPOINT: &str = "";
pub const PIONEER_TOKENS_HTTP_ENDPOINT: &str = "";

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::traits::{Currency, ReservableCurrency};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: CreateSignedTransaction<Call<Self>> + frame_system::Config {
		/// Runtime event type
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Currency type
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		/// NFT trait required for minting NFTs
		type NFTSource: NFTMigrationTrait<Self::AccountId, BalanceOf<Self>, ClassId = ClassId, TokenId = TokenId>;
		/// Accounts that can set start migration
		type MigrationOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;
		/// Extrinsics' weights
		type WeightInfo: WeightInfo;
	}
	pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::error]
	pub enum Error<T> {
		/// Migration is already active
		MigrationInProgress,
		/// No Pioneer data is found at given endpoint
		PioneerDataNotFound,
		/// Provided migration data is inconsistent with the current state of the chain
		InconsistentMigrationData,
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// Started the nft data migration
		MigrationStarted,
		/// Fetched the nft collections data
		FetchedCollectionData,
		/// Fetched the nft classes data
		FetchedClassData,
		/// Fetched the nft token data
		FetchedTokenData,
		/// Completed the nft collections data migration
		CollectionDataMigrationCompleted,
		/// Completed the nft classes data migration
		ClassDataMigrationCompleted,
		/// Completed the nft tokens data migration
		TokenDataMigrationCompleted,
		/// Completed the nft data migration
		MigrationCompleted,
	}

	#[pallet::storage]
	#[pallet::getter(fn is_migration_active)]
	pub type ActiveMigrationStatus<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// Offchain worker hook
		fn offchain_worker(block_number: BlockNumberFor<T>) {
			if Self::is_migration_active() {
				Self::migrate_pioneer_nft_data();
				Self::deposit_event(Event::<T>::MigrationCompleted);
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(<T as Config>::WeightInfo::start_migration())]
		pub fn start_migration(origin: OriginFor<T>) -> DispatchResult {
			T::MigrationOrigin::ensure_origin(origin)?;
			ensure!(!Self::is_migration_active(), Error::<T>::MigrationInProgress);
			ActiveMigrationStatus::<T>::put(true);
			Self::deposit_event(Event::<T>::MigrationStarted);
			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::start_migration())]
		pub fn migrate_collection_unsigned(
			origin: OriginFor<T>,
			collection_id: GroupCollectionId,
			collection_data: NftGroupCollectionData,
		) -> DispatchResult {
			ensure_none(origin)?;
			ensure!(
				T::NFTSource::get_next_collection_id() == collection_id,
				Error::<T>::InconsistentMigrationData
			);
			T::NFTSource::migrate_collection(collection_id, collection_data);
			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::start_migration())]
		pub fn migrate_class_unsigned(
			origin: OriginFor<T>,
			owner: T::AccountId,
			collection_id: GroupCollectionId,
			class_id: ClassId,
			metadata: NftMetadata,
			class_data: NftClassData<BalanceOf<T>>,
		) -> DispatchResult {
			ensure_none(origin)?;
			ensure!(
				T::NFTSource::get_next_class_id() == class_id,
				Error::<T>::InconsistentMigrationData
			);
			T::NFTSource::migrate_class(&owner, class_id, collection_id, metadata, class_data)?;
			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::start_migration())]
		pub fn migrate_token_unsigned(
			origin: OriginFor<T>,
			owner: T::AccountId,
			class_id: ClassId,
			token_id: TokenId,
			metadata: NftMetadata,
			token_data: NftAssetData<BalanceOf<T>>,
		) -> DispatchResult {
			ensure_none(origin)?;
			ensure!(
				T::NFTSource::get_next_token_id(class_id) == token_id,
				Error::<T>::InconsistentMigrationData
			);
			T::NFTSource::migrate_token(&owner, token_id, class_id, metadata, token_data)?;
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Internal Pioneer data migration flow
	fn migrate_pioneer_nft_data() -> DispatchResult {
		let pioneer_collections_data: Vec<(GroupCollectionId, NftGroupCollectionData)> =
			Self::fetch_pioneer_nft_collections_data(PIONEER_COLLECTIONS_HTTP_ENDPOINT)?;
		Self::create_nft_collections_from_pioneer_data(&pioneer_collections_data)?;

		let pioneer_class_data: Vec<(
			T::AccountId,
			GroupCollectionId,
			ClassId,
			NftMetadata,
			NftClassData<BalanceOf<T>>,
		)> = Self::fetch_pioneer_nft_class_data(PIONEER_CLASSES_HTTP_ENDPOINT)?;
		Self::create_nft_classes_from_pioneer_data(&pioneer_class_data)?;

		let pioneer_token_data: Vec<(T::AccountId, ClassId, TokenId, NftMetadata, NftAssetData<BalanceOf<T>>)> =
			Self::fetch_pioneer_nft_token_data(PIONEER_TOKENS_HTTP_ENDPOINT)?;
		Self::mint_nft_tokens_from_pioneer_data(&pioneer_token_data)?;

		ActiveMigrationStatus::<T>::put(false);
		Ok(())
	}

	/// Fetches Pioneer collections data from database via HTTP
	fn fetch_pioneer_nft_collections_data(
		endpoint_address: &str,
	) -> Result<Vec<(GroupCollectionId, NftGroupCollectionData)>, DispatchError> {
		let pioneer_collections_request = Request::get(endpoint_address);
		// TODO: Add correct request header
		let pending = pioneer_collections_request
			.add_header("X-Auth", "hunter2")
			.send()
			.unwrap();
		let mut response = pending.wait().unwrap();
		let body = response.body();
		ensure!(!body.error().is_none(), Error::<T>::PioneerDataNotFound);
		// TODO: Process data into Vec<(GroupCollectionId, NftGroupCollectionData>
		//let collection_data = body.collect::<Vec<(GroupCollectionId, NftGroupCollectionData)>>();
		//Self::deposit_event(Event::<T>::FetchedCollectionData);
		return Ok(vec![]);
	}

	/// Fetches Pioneer classes data from database via HTTP
	fn fetch_pioneer_nft_class_data(
		endpoint_address: &str,
	) -> Result<
		Vec<(
			T::AccountId,
			GroupCollectionId,
			ClassId,
			NftMetadata,
			NftClassData<BalanceOf<T>>,
		)>,
		DispatchError,
	> {
		let pioneer_classes_request = Request::get(endpoint_address);
		// TODO: Add correct request header
		let pending = pioneer_classes_request.add_header("X-Auth", "hunter2").send().unwrap();
		let mut response = pending.wait().unwrap();
		let body = response.body();
		ensure!(!body.error().is_none(), Error::<T>::PioneerDataNotFound);
		// TODO: Process data into Vec<(T::AccountId, GroupCollectionId, ClassId, NftMetadata, NftClassData<BalanceOf<T>>)>
		// NftClassData<BalanceOf<T>>)> Self::deposit_event(Event::<T>::FetchedClassData);
		return Ok(vec![]);
	}
	/// Fetches Pioneer tokens data from database via HTTP
	fn fetch_pioneer_nft_token_data(
		endpoint_address: &str,
	) -> Result<Vec<(T::AccountId, ClassId, TokenId, NftMetadata, NftAssetData<BalanceOf<T>>)>, DispatchError> {
		let pioneer_tokens_request = Request::get(endpoint_address);
		// TODO: Add correct request header
		let pending = pioneer_tokens_request.add_header("X-Auth", "hunter2").send().unwrap();
		let mut response = pending.wait().unwrap();
		let body = response.body();
		ensure!(!body.error().is_none(), Error::<T>::PioneerDataNotFound);
		// TODO: Process data into Vec<(T::AccountId, ClassId, TokenId, NftMetadata, NftAssetData<BalanceOf<T>>)>
		// NftAssetData<BalanceOf<T>>)> Self::deposit_event(Event::<T>::FetchedTokenData);
		return Ok(vec![]);
	}

	fn create_nft_collections_from_pioneer_data(
		pioneer_collections_data: &Vec<(GroupCollectionId, NftGroupCollectionData)>,
	) -> DispatchResult {
		for (collection_id, collection_data) in pioneer_collections_data.iter() {
			let call = Call::migrate_collection_unsigned {
				collection_id: *collection_id,
				collection_data: (*collection_data).clone(),
			};
			SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
				.map_err(|()| "Unable to submit unsigned collection migration transaction.")?;
		}
		//Self::deposit_event(Event::<T>::CollectionDataMigrationCompleted);
		Ok(())
	}

	fn create_nft_classes_from_pioneer_data(
		pioneer_class_data: &Vec<(
			T::AccountId,
			GroupCollectionId,
			ClassId,
			NftMetadata,
			NftClassData<BalanceOf<T>>,
		)>,
	) -> DispatchResult {
		for (owner, collection_id, class_id, metadata, class_data) in pioneer_class_data.iter() {
			let call = Call::migrate_class_unsigned {
				owner: (*owner).clone(),
				collection_id: *collection_id,
				class_id: *class_id,
				metadata: (*metadata).clone(),
				class_data: (*class_data).clone(),
			};
			SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
				.map_err(|()| "Unable to submit unsigned class migration transaction.")?;
		}
		//Self::deposit_event(Event::<T>::ClassDataMigrationCompleted);
		Ok(())
	}

	/// Internally migrate Pioneer tokens using the  fetched data
	fn mint_nft_tokens_from_pioneer_data(
		pioneer_token_data: &Vec<(T::AccountId, ClassId, TokenId, NftMetadata, NftAssetData<BalanceOf<T>>)>,
	) -> DispatchResult {
		for (owner, class_id, token_id, metadata, token_data) in pioneer_token_data.iter() {
			let call = Call::migrate_token_unsigned {
				owner: (*owner).clone(),
				class_id: *class_id,
				token_id: *token_id,
				metadata: (*metadata).clone(),
				token_data: (*token_data).clone(),
			};
			SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
				.map_err(|()| "Unable to submit unsigned token migration transaction.")?;
		}
		//Self::deposit_event(Event::<T>::TokenDataMigrationCompleted);
		Ok(())
	}
}
