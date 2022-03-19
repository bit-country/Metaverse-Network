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

// This pallet use The Open Runtime Module Library (ORML) which is a community maintained collection
// of Substrate runtime modules. Thanks to all contributors of orml.
// Ref: https://github.com/open-web3-stack/open-runtime-module-library

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::string_lit_as_bytes)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::traits::Len;
use frame_support::{
	dispatch::{DispatchResult, DispatchResultWithPostInfo},
	ensure,
	pallet_prelude::*,
	traits::{
		schedule::{DispatchTime, Named as ScheduleNamed},
		Currency, ExistenceRequirement, Get, LockIdentifier, ReservableCurrency,
	},
	PalletId,
};
use frame_system::pallet_prelude::*;
use orml_nft::{ClassInfo, ClassInfoOf, Classes, Pallet as NftModule};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::traits::Saturating;
use sp_runtime::RuntimeDebug;
use sp_runtime::{
	traits::{AccountIdConversion, Dispatchable, One},
	DispatchError,
};
use sp_std::vec::Vec;
use sp_std::{collections::btree_map::BTreeMap, prelude::*};

use auction_manager::{Auction, CheckAuctionItemHandler};
pub use pallet::*;
use primitive_traits::NFTTrait;
use primitives::{AssetId, BlockNumber, ClassId, GroupCollectionId, Hash, ItemId, TokenId};
pub use weights::WeightInfo;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod weights;

pub type NftMetadata = Vec<u8>;
pub type Attributes = BTreeMap<Vec<u8>, Vec<u8>>;

const TIMECAPSULE_ID: LockIdentifier = *b"bctimeca";

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub struct NftGroupCollectionData {
	pub name: NftMetadata,
	// Metadata from ipfs
	pub properties: NftMetadata,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct NftClassData<Balance> {
	// Minimum balance to create a collection of Asset
	pub deposit: Balance,
	pub attributes: Attributes,
	pub token_type: TokenType,
	pub collection_type: CollectionType,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct NftAssetData<Balance> {
	// Deposit balance to create each token
	pub deposit: Balance,
	pub attributes: Attributes,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum TokenType {
	Transferable,
	BoundToAddress,
}

impl TokenType {
	pub fn is_transferable(&self) -> bool {
		match *self {
			TokenType::Transferable => true,
			_ => false,
		}
	}
}

impl Default for TokenType {
	fn default() -> Self {
		TokenType::Transferable
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CollectionType {
	Collectable,
	Wearable,
	Executable(Vec<u8>),
}

// Collection extension for fast retrieval
impl CollectionType {
	pub fn is_collectable(&self) -> bool {
		match *self {
			CollectionType::Collectable => true,
			_ => false,
		}
	}

	pub fn is_executable(&self) -> bool {
		match *self {
			CollectionType::Executable(_) => true,
			_ => false,
		}
	}

	pub fn is_wearable(&self) -> bool {
		match *self {
			CollectionType::Wearable => true,
			_ => false,
		}
	}
}

impl Default for CollectionType {
	fn default() -> Self {
		CollectionType::Collectable
	}
}

#[derive(codec::Encode, codec::Decode, Clone, frame_support::RuntimeDebug, PartialEq)]
pub enum StorageVersion {
	V0,
	V1,
}

pub mod migrations {
	use frame_support::pallet_prelude::*;
	use orml_nft::{ClassInfo, ClassInfoOf, Classes};

	use super::{BalanceOf, Config, NftClassData, NftMetadata, NftModule, TokenIdOf};

	//	pub fn upgrade_class_data_v2<T: Config>() -> Weight {
	//		log::info!("Start upgrading nft class data v2");
	//		let mut num_nft_classes = 0;
	//
	//		Classes::<T>::translate(|_k, class_info: Option<ClassInfoOf<T>>| {
	//			num_nft_classes += 1;
	//
	//			match class_info {
	//				Some(info) => {
	//					let class_data = NftClassData {
	//						deposit: info.data.deposit,
	//						token_type: info.data.token_type,
	//						collection_type: info.data.collection_type,
	//						attributes: info.data.attributes,
	//					};
	//
	//					class_data.try_into().ok()
	//				}
	//				_ => None,
	//			}
	//		});
	//
	//		//		Classes::<T>::translate::<ClassInfoOf<>>(
	//		//			|_k, class_info: ClassInfo<TokenIdOf<T>, T::AccountId, NftClassData<BalanceOf<T>>,
	// NftMetadata>| 		// { 				num_nft_classes += 1;
	//		//
	//		//				let v: Option<ClassInfo<TokenIdOf<T>, T::AccountId, NftClassData<BalanceOf<T>>,
	// NftMetadata>> = 		//					class_info.try_into().ok();
	//		//
	//		//				if v.is_none() {
	//		//					log::warn!(
	//		//						target: "runtime::nft",
	//		//						"migration: Failed to upgrade nft class data v1 to v2"
	//		//					);
	//		//				}
	//		//
	//		//				v
	//		//			},
	//		//		);
	//
	//		0
	//	}
}

#[frame_support::pallet]
pub mod pallet {
	use orml_traits::{MultiCurrency, MultiCurrencyExtended};

	use primitives::{ClassId, FungibleTokenId, ItemId};

	use super::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ orml_nft::Config<TokenData = NftAssetData<BalanceOf<Self>>, ClassData = NftClassData<BalanceOf<Self>>>
	{
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The data deposit per byte to calculate fee
		#[pallet::constant]
		type DataDepositPerByte: Get<BalanceOf<Self>>;
		/// Currency type for reserve/unreserve balance
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		//NFT Module Id
		#[pallet::constant]
		type PalletId: Get<PalletId>;
		/// Weight info
		type WeightInfo: WeightInfo;
		/// Auction Handler
		type AuctionHandler: Auction<Self::AccountId, Self::BlockNumber> + CheckAuctionItemHandler;
		/// Max transfer batch
		#[pallet::constant]
		type MaxBatchTransfer: Get<u32>;
		/// Max batch minting
		#[pallet::constant]
		type MaxBatchMinting: Get<u32>;
		/// Max metadata length
		#[pallet::constant]
		type MaxMetadata: Get<u32>;
		/// Multi currency type for promotion incentivization
		type MultiCurrency: MultiCurrencyExtended<
			Self::AccountId,
			CurrencyId = FungibleTokenId,
			Balance = BalanceOf<Self>,
		>;
		/// Fungible token id for promotion incentive
		#[pallet::constant]
		type MiningResourceId: Get<FungibleTokenId>;
		/// Incentive for promotion
		type PromotionIncentive: Get<BalanceOf<Self>>;
	}

	pub type ClassIdOf<T> = <T as orml_nft::Config>::ClassId;
	pub type TokenIdOf<T> = <T as orml_nft::Config>::TokenId;
	pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::storage]
	#[pallet::getter(fn get_asset)]
	pub(super) type Assets<T: Config> =
		StorageMap<_, Blake2_128Concat, AssetId, (ClassIdOf<T>, TokenIdOf<T>), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_assets_by_owner)]
	pub(super) type AssetsByOwner<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Vec<(ClassIdOf<T>, TokenIdOf<T>)>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_group_collection)]
	pub(super) type GroupCollections<T: Config> =
		StorageMap<_, Blake2_128Concat, GroupCollectionId, NftGroupCollectionData, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_class_collection)]
	pub(super) type ClassDataCollection<T: Config> =
		StorageMap<_, Blake2_128Concat, ClassIdOf<T>, GroupCollectionId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_group_collection_id)]
	pub(super) type NextGroupCollectionId<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn all_nft_collection_count)]
	pub(super) type AllNftGroupCollection<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_asset_id)]
	pub(super) type NextAssetId<T: Config> = StorageValue<_, AssetId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_asset_supporters)]
	pub(super) type AssetSupporters<T: Config> =
		StorageMap<_, Blake2_128Concat, (ClassIdOf<T>, TokenIdOf<T>), Vec<T::AccountId>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_promotion_enabled)]
	pub(super) type PromotionEnabled<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_locked_collection)]
	pub(super) type LockedCollection<T: Config> = StorageMap<_, Blake2_128Concat, ClassIdOf<T>, (), OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New NFT Group Collection created
		NewNftCollectionCreated(GroupCollectionId),
		/// New NFT Collection/Class created
		NewNftClassCreated(<T as frame_system::Config>::AccountId, ClassIdOf<T>),
		/// Emit event when new nft minted - show the first and last asset mint
		NewNftMinted(
			(ClassIdOf<T>, TokenIdOf<T>),
			(ClassIdOf<T>, TokenIdOf<T>),
			<T as frame_system::Config>::AccountId,
			ClassIdOf<T>,
			u32,
			TokenIdOf<T>,
		),
		/// Emit event when new time capsule minted
		NewTimeCapsuleMinted(
			(ClassIdOf<T>, TokenIdOf<T>),
			<T as frame_system::Config>::AccountId,
			ClassIdOf<T>,
			TokenIdOf<T>,
			T::BlockNumber,
			Vec<u8>,
		),
		/// Successfully transfer NFT
		TransferedNft(
			<T as frame_system::Config>::AccountId,
			<T as frame_system::Config>::AccountId,
			TokenIdOf<T>,
			(ClassIdOf<T>, TokenIdOf<T>),
		),
		/// Successfully force transfer NFT
		ForceTransferredNft(
			<T as frame_system::Config>::AccountId,
			<T as frame_system::Config>::AccountId,
			TokenIdOf<T>,
			(ClassIdOf<T>, TokenIdOf<T>),
		),
		/// Signed on NFT
		SignedNft(TokenIdOf<T>, <T as frame_system::Config>::AccountId),
		/// Promotion enabled
		PromotionEnabled(bool),
		/// Burn NFT
		BurnedNft((ClassIdOf<T>, TokenIdOf<T>)),
		/// Executed NFT
		ExecutedNft(AssetId),
		/// Scheduled time capsule
		ScheduledTimeCapsule(AssetId, Vec<u8>, T::BlockNumber),
		/// Collection is locked
		CollectionLocked(ClassIdOf<T>),
		/// Collection is unlocked
		CollectionUnlocked(ClassIdOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Attempted to initialize the metaverse after it had already been initialized.
		AlreadyInitialized,
		/// Asset Info not found
		AssetInfoNotFound,
		/// Asset Id not found
		AssetIdNotFound,
		/// No permission
		NoPermission,
		/// No available collection id
		NoAvailableCollectionId,
		/// Collection id is not exist
		CollectionIsNotExist,
		/// Class Id not found
		ClassIdNotFound,
		/// Non Transferable
		NonTransferable,
		/// Invalid quantity
		InvalidQuantity,
		/// No available asset id
		NoAvailableAssetId,
		/// Asset Id is already exist
		AssetIdAlreadyExist,
		/// Asset Id is currently in an auction
		AssetAlreadyInAuction,
		/// Sign your own Asset
		SignOwnAsset,
		/// Exceed maximum batch transfer
		ExceedMaximumBatchTransfer,
		/// Exceed maximum batch minting
		ExceedMaximumBatchMinting,
		/// Exceed maximum length metadata
		ExceedMaximumMetadataLength,
		/// Error when signing support
		EmptySupporters,
		/// Insufficient Balance
		InsufficientBalance,
		/// Time-capsule executed too early
		TimecapsuleExecutedTooEarly,
		/// Only Time capsule collection
		OnlyForTimeCapsuleCollectionType,
		/// Timecapsule execution logic is invalid
		TimeCapsuleExecutionLogicIsInvalid,
		/// Timecapsule scheduled error
		ErrorWhenScheduledTimeCapsule,
		/// Collection already locked
		CollectionAlreadyLocked,
		/// Collection is locked
		CollectionIsLocked,
		/// Collection is not locked
		CollectionIsNotLocked,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::create_group())]
		pub fn create_group(
			origin: OriginFor<T>,
			name: NftMetadata,
			properties: NftMetadata,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			ensure!(
				name.len() as u32 <= T::MaxMetadata::get() && properties.len() as u32 <= T::MaxMetadata::get(),
				Error::<T>::ExceedMaximumMetadataLength
			);

			let next_group_collection_id = Self::do_create_group_collection(name.clone(), properties.clone())?;

			let collection_data = NftGroupCollectionData { name, properties };

			GroupCollections::<T>::insert(next_group_collection_id, collection_data);

			let all_collection_count = Self::all_nft_collection_count();
			let new_all_nft_collection_count = all_collection_count
				.checked_add(One::one())
				.ok_or("Overflow adding a new collection to total collection")?;

			AllNftGroupCollection::<T>::set(new_all_nft_collection_count);

			Self::deposit_event(Event::<T>::NewNftCollectionCreated(next_group_collection_id));
			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::create_class())]
		pub fn create_class(
			origin: OriginFor<T>,
			metadata: NftMetadata,
			attributes: Attributes,
			collection_id: GroupCollectionId,
			token_type: TokenType,
			collection_type: CollectionType,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			ensure!(
				metadata.len() as u32 <= T::MaxMetadata::get(),
				Error::<T>::ExceedMaximumMetadataLength
			);
			let next_class_id = NftModule::<T>::next_class_id();
			ensure!(
				GroupCollections::<T>::contains_key(collection_id),
				Error::<T>::CollectionIsNotExist
			);

			// Class fund
			let class_fund: T::AccountId = T::PalletId::get().into_sub_account(next_class_id);

			// Secure deposit of token class owner
			let class_deposit = Self::calculate_fee_deposit(&attributes)?;
			// Transfer fund to pot
			<T as Config>::Currency::transfer(&sender, &class_fund, class_deposit, ExistenceRequirement::KeepAlive)?;
			// Reserve pot fund
			<T as Config>::Currency::reserve(&class_fund, <T as Config>::Currency::free_balance(&class_fund))?;

			let class_data = NftClassData {
				deposit: class_deposit,
				token_type,
				collection_type,
				attributes: attributes,
			};

			NftModule::<T>::create_class(&sender, metadata, class_data)?;
			ClassDataCollection::<T>::insert(next_class_id, collection_id);

			Self::deposit_event(Event::<T>::NewNftClassCreated(sender, next_class_id));

			Ok(().into())
		}

		#[pallet::weight(< T as Config >::WeightInfo::mint(* quantity))]
		pub fn mint(
			origin: OriginFor<T>,
			class_id: ClassIdOf<T>,
			metadata: NftMetadata,
			attributes: Attributes,
			quantity: u32,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			ensure!(!Self::is_collection_locked(&class_id), Error::<T>::CollectionIsLocked);
			ensure!(quantity >= 1, Error::<T>::InvalidQuantity);
			ensure!(
				quantity <= T::MaxBatchMinting::get(),
				Error::<T>::ExceedMaximumBatchMinting
			);
			ensure!(
				metadata.len() as u32 <= T::MaxMetadata::get(),
				Error::<T>::ExceedMaximumMetadataLength
			);

			let class_info = NftModule::<T>::classes(class_id).ok_or(Error::<T>::ClassIdNotFound)?;
			ensure!(sender == class_info.owner, Error::<T>::NoPermission);
			let token_deposit = Self::calculate_fee_deposit(&attributes)?;
			let class_fund: T::AccountId = T::PalletId::get().into_sub_account(class_id);
			let deposit = token_deposit.saturating_mul(Into::<BalanceOf<T>>::into(quantity));

			<T as Config>::Currency::transfer(&sender, &class_fund, deposit, ExistenceRequirement::KeepAlive)?;
			<T as Config>::Currency::reserve(&class_fund, deposit)?;

			let new_nft_data = NftAssetData {
				deposit,
				attributes: attributes,
			};

			let mut new_asset_ids: Vec<(ClassIdOf<T>, TokenIdOf<T>)> = Vec::new();
			let mut last_token_id: TokenIdOf<T> = Default::default();

			for _ in 0..quantity {
				let token_id = NftModule::<T>::mint(&sender, class_id, metadata.clone(), new_nft_data.clone())?;
				new_asset_ids.push((class_id, token_id));

				if AssetsByOwner::<T>::contains_key(&sender) {
					AssetsByOwner::<T>::try_mutate(&sender, |asset_ids| -> DispatchResult {
						asset_ids.push((class_id, token_id));
						Ok(())
					})?;
				} else {
					let mut assets = Vec::<(ClassIdOf<T>, TokenIdOf<T>)>::new();
					assets.push((class_id, token_id));
					AssetsByOwner::<T>::insert(&sender, assets)
				}
				last_token_id = token_id;
			}

			// If promotion enabled
			if Self::is_promotion_enabled() {
				T::MultiCurrency::deposit(T::MiningResourceId::get(), &sender, T::PromotionIncentive::get())?;
			};

			Self::deposit_event(Event::<T>::NewNftMinted(
				*new_asset_ids.first().unwrap(),
				*new_asset_ids.last().unwrap(),
				sender,
				class_id,
				quantity,
				last_token_id,
			));

			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::transfer())]
		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			asset_id: (ClassIdOf<T>, TokenIdOf<T>),
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			ensure!(
				Self::check_item_on_listing(asset_id.0, asset_id.1)? == false,
				Error::<T>::AssetAlreadyInAuction
			);

			let token_id = Self::do_transfer(&sender, &to, asset_id)?;

			Self::deposit_event(Event::<T>::TransferedNft(sender, to, token_id, asset_id.clone()));

			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::transfer_batch(tos.len() as u32))]
		pub fn transfer_batch(
			origin: OriginFor<T>,
			tos: Vec<(T::AccountId, (ClassIdOf<T>, TokenIdOf<T>))>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			ensure!(
				tos.len() as u32 <= T::MaxBatchTransfer::get(),
				Error::<T>::ExceedMaximumBatchTransfer
			);

			for (_i, x) in tos.iter().enumerate() {
				let item = &x;
				let owner = &sender.clone();

				let class_info = NftModule::<T>::classes((item.1).0).ok_or(Error::<T>::ClassIdNotFound)?;
				let data = class_info.data;

				match data.token_type {
					TokenType::Transferable => {
						let asset_info =
							NftModule::<T>::tokens((item.1).0, (item.1).1).ok_or(Error::<T>::AssetInfoNotFound)?;
						ensure!(owner.clone() == asset_info.owner, Error::<T>::NoPermission);
						Self::handle_asset_ownership_transfer(&owner, &item.0, item.1)?;
						NftModule::<T>::transfer(&owner, &item.0, item.1)?;
						Self::deposit_event(Event::<T>::TransferedNft(
							owner.clone(),
							item.0.clone(),
							(item.1).1.clone(),
							item.1.clone(),
						));
					}
					_ => (),
				};
			}

			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::sign_asset())]
		pub fn sign_asset(
			origin: OriginFor<T>,
			asset_id: (ClassIdOf<T>, TokenIdOf<T>),
			contribution: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let asset_by_owner: Vec<(ClassIdOf<T>, TokenIdOf<T>)> = Self::get_assets_by_owner(&sender);
			ensure!(!asset_by_owner.contains(&asset_id), Error::<T>::SignOwnAsset);

			// Add contribution into class fund
			let class_fund = Self::get_class_fund(&asset_id.0);

			ensure!(
				<T as Config>::Currency::free_balance(&sender) > contribution,
				Error::<T>::InsufficientBalance
			);
			// Transfer contribution to class fund pot
			<T as Config>::Currency::transfer(&sender, &class_fund, contribution, ExistenceRequirement::KeepAlive)?;
			// Reserve pot fund
			<T as Config>::Currency::reserve(&class_fund, contribution)?;

			if AssetSupporters::<T>::contains_key(&asset_id) {
				AssetSupporters::<T>::try_mutate(asset_id, |supporters| -> DispatchResult {
					let supporters = supporters.as_mut().ok_or(Error::<T>::EmptySupporters)?;
					supporters.push(sender);
					Ok(())
				})?;
			} else {
				let mut new_supporters = Vec::new();
				new_supporters.push(sender);
				AssetSupporters::<T>::insert(asset_id, new_supporters);
			}
			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::sign_asset())]
		pub fn enable_promotion(origin: OriginFor<T>, enable: bool) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			PromotionEnabled::<T>::put(enable);

			Self::deposit_event(Event::<T>::PromotionEnabled(enable));
			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::sign_asset())]
		pub fn burn(origin: OriginFor<T>, asset_id: (ClassIdOf<T>, TokenIdOf<T>)) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let asset_by_owner: Vec<(ClassIdOf<T>, TokenIdOf<T>)> = Self::get_assets_by_owner(&sender);

			ensure!(asset_by_owner.contains(&asset_id), Error::<T>::NoPermission);

			NftModule::<T>::burn(&sender, asset_id)?;
			Self::deposit_event(Event::<T>::BurnedNft(asset_id));
			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::sign_asset())]
		pub fn force_lock_collection(origin: OriginFor<T>, class_id: ClassIdOf<T>) -> DispatchResult {
			ensure_root(origin)?;

			ensure!(
				!LockedCollection::<T>::contains_key(class_id),
				Error::<T>::CollectionAlreadyLocked
			);

			LockedCollection::<T>::insert(class_id.clone(), ());
			Self::deposit_event(Event::<T>::CollectionLocked(class_id));

			Ok(())
		}

		#[pallet::weight(T::WeightInfo::sign_asset())]
		pub fn force_unlock_collection(origin: OriginFor<T>, class_id: ClassIdOf<T>) -> DispatchResult {
			ensure_root(origin)?;

			ensure!(
				LockedCollection::<T>::contains_key(class_id),
				Error::<T>::CollectionIsNotLocked
			);

			LockedCollection::<T>::remove(class_id.clone());
			Self::deposit_event(Event::<T>::CollectionUnlocked(class_id));

			Ok(())
		}

		/// Force NFT transfer which only triggered by governance
		#[pallet::weight(T::WeightInfo::transfer())]
		pub fn force_transfer(
			origin: OriginFor<T>,
			from: T::AccountId,
			to: T::AccountId,
			asset_id: (ClassIdOf<T>, TokenIdOf<T>),
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			ensure!(
				Self::check_item_on_listing(asset_id.0, asset_id.1)? == false,
				Error::<T>::AssetAlreadyInAuction
			);

			let token_id = Self::do_force_transfer(&from, &to, asset_id)?;

			Self::deposit_event(Event::<T>::ForceTransferredNft(from, to, token_id, asset_id.clone()));

			Ok(().into())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_runtime_upgrade() -> Weight {
			Self::upgrade_class_data_v2();

			0
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn is_promotion_enabled() -> bool {
		Self::get_promotion_enabled()
	}

	pub fn get_class_fund(class_id: &ClassIdOf<T>) -> T::AccountId {
		T::PalletId::get().into_sub_account(class_id)
	}

	fn do_create_group_collection(name: Vec<u8>, properties: Vec<u8>) -> Result<GroupCollectionId, DispatchError> {
		let next_group_collection_id =
			NextGroupCollectionId::<T>::try_mutate(|collection_id| -> Result<GroupCollectionId, DispatchError> {
				let current_id = *collection_id;

				*collection_id = collection_id
					.checked_add(One::one())
					.ok_or(Error::<T>::NoAvailableCollectionId)?;

				Ok(current_id)
			})?;

		let collection_data = NftGroupCollectionData { name, properties };

		<GroupCollections<T>>::insert(next_group_collection_id, collection_data);

		Ok(next_group_collection_id)
	}

	fn handle_asset_ownership_transfer(
		sender: &T::AccountId,
		to: &T::AccountId,
		asset_id: (ClassIdOf<T>, TokenIdOf<T>),
	) -> DispatchResult {
		// Remove asset from sender
		AssetsByOwner::<T>::try_mutate(&sender, |asset_ids| -> DispatchResult {
			// Check if the asset_id already in the owner
			let asset_index = asset_ids.iter().position(|x| *x == asset_id).unwrap();
			asset_ids.remove(asset_index);

			Ok(())
		})?;

		// Insert asset to recipient

		if AssetsByOwner::<T>::contains_key(to) {
			AssetsByOwner::<T>::try_mutate(&to, |asset_ids| -> DispatchResult {
				// Check if the asset_id already in the owner
				asset_ids.push(asset_id);
				Ok(())
			})?;
		} else {
			let mut asset_ids = Vec::<(ClassIdOf<T>, TokenIdOf<T>)>::new();
			asset_ids.push(asset_id);
			AssetsByOwner::<T>::insert(&to, asset_ids);
		}

		Ok(())
	}

	pub fn do_transfer(
		sender: &T::AccountId,
		to: &T::AccountId,
		asset_id: (ClassIdOf<T>, TokenIdOf<T>),
	) -> Result<<T as orml_nft::Config>::TokenId, DispatchError> {
		ensure!(!Self::is_collection_locked(&asset_id.0), Error::<T>::CollectionIsLocked);

		let class_info = NftModule::<T>::classes(asset_id.0).ok_or(Error::<T>::ClassIdNotFound)?;
		let data = class_info.data;

		match data.token_type {
			TokenType::Transferable => {
				let check_ownership = Self::check_nft_ownership(&sender, &asset_id)?;
				ensure!(check_ownership, Error::<T>::NoPermission);

				Self::handle_asset_ownership_transfer(&sender, &to, asset_id)?;

				NftModule::<T>::transfer(&sender, &to, asset_id.clone())?;
				Ok(asset_id.1)
			}
			TokenType::BoundToAddress => Err(Error::<T>::NonTransferable.into()),
		}
	}

	pub fn check_nft_ownership(
		sender: &T::AccountId,
		asset_id: &(ClassIdOf<T>, TokenIdOf<T>),
	) -> Result<bool, DispatchError> {
		let asset_info = NftModule::<T>::tokens(asset_id.0, asset_id.1).ok_or(Error::<T>::AssetInfoNotFound)?;
		if sender == &asset_info.owner {
			return Ok(true);
		}

		return Ok(false);
	}

	/// Check if the NFT collection is locked
	pub fn is_collection_locked(class_id: &ClassIdOf<T>) -> bool {
		let is_locked = LockedCollection::<T>::get(class_id).is_some();
		return is_locked;
	}

	/// Force transfer NFT only for governance override action
	fn do_force_transfer(
		sender: &T::AccountId,
		to: &T::AccountId,
		asset_id: (ClassIdOf<T>, TokenIdOf<T>),
	) -> Result<<T as orml_nft::Config>::TokenId, DispatchError> {
		ensure!(!Self::is_collection_locked(&asset_id.0), Error::<T>::CollectionIsLocked);

		Self::handle_asset_ownership_transfer(&sender, &to, asset_id)?;

		NftModule::<T>::transfer(&sender, &to, asset_id.clone())?;
		Ok(asset_id.1)
	}

	/// Calculate deposit fee
	fn calculate_fee_deposit(attributes: &Attributes) -> Result<BalanceOf<T>, DispatchError> {
		// Accumulate lens of attributes length
		let attributes_len = attributes.iter().fold(0, |accumulate, (k, v)| {
			accumulate.saturating_add(v.len().saturating_add(k.len()) as u32)
		});

		ensure!(
			attributes_len <= T::MaxMetadata::get(),
			Error::<T>::ExceedMaximumMetadataLength
		);

		let deposit_required = T::DataDepositPerByte::get().saturating_mul(attributes_len.into());

		Ok(deposit_required)
	}

	pub fn upgrade_class_data_v2() -> Weight {
		log::info!("Start upgrading nft class data v2");
		let mut num_nft_classes = 0;

		orml_nft::Classes::<T>::translate(|_k, class_info: ClassInfoOf<T>| {
			num_nft_classes += 1;
			log::info!("Upgrading class data");
			let new_data = NftClassData {
				deposit: class_info.data.deposit,
				attributes: class_info.data.attributes,
				token_type: class_info.data.token_type,
				collection_type: class_info.data.collection_type,
			};

			let v: ClassInfoOf<T> = ClassInfo {
				metadata: class_info.metadata,
				total_issuance: class_info.total_issuance,
				owner: class_info.owner,
				data: new_data,
			};
			Some(v)
		});
		log::info!("Classes upgraded: {}", num_nft_classes);
		//		Classes::<T>::translate::<ClassInfoOf<>>(
		//			|_k, class_info: ClassInfo<TokenIdOf<T>, T::AccountId, NftClassData<BalanceOf<T>>, NftMetadata>|
		// { 				num_nft_classes += 1;
		//
		//				let v: Option<ClassInfo<TokenIdOf<T>, T::AccountId, NftClassData<BalanceOf<T>>, NftMetadata>> =
		//					class_info.try_into().ok();
		//
		//				if v.is_none() {
		//					log::warn!(
		//						target: "runtime::nft",
		//						"migration: Failed to upgrade nft class data v1 to v2"
		//					);
		//				}
		//
		//				v
		//			},
		//		);

		0
	}
}

impl<T: Config> NFTTrait<T::AccountId> for Pallet<T> {
	type TokenId = TokenIdOf<T>;
	type ClassId = ClassIdOf<T>;

	fn check_ownership(who: &T::AccountId, asset_id: &(Self::ClassId, Self::TokenId)) -> Result<bool, DispatchError> {
		let asset_info = NftModule::<T>::tokens(asset_id.0, asset_id.1).ok_or(Error::<T>::AssetInfoNotFound)?;

		Ok(who == &asset_info.owner)
	}

	fn check_nft_ownership(who: &T::AccountId, nft: &(Self::ClassId, Self::TokenId)) -> Result<bool, DispatchError> {
		let asset_info = NftModule::<T>::tokens(nft.0, nft.1).ok_or(Error::<T>::AssetInfoNotFound)?;

		Ok(who == &asset_info.owner)
	}

	fn get_nft_detail(
		asset_id: (Self::ClassId, Self::TokenId),
	) -> Result<(GroupCollectionId, Self::ClassId, Self::TokenId), DispatchError> {
		let group_collection_id = ClassDataCollection::<T>::get(asset_id.0);

		Ok((group_collection_id, asset_id.0, asset_id.1))
	}

	fn get_nft_group_collection(nft_collection: &Self::ClassId) -> Result<GroupCollectionId, DispatchError> {
		let group_collection_id = ClassDataCollection::<T>::get(nft_collection);
		Ok(group_collection_id)
	}

	fn check_collection_and_class(
		collection_id: GroupCollectionId,
		class_id: Self::ClassId,
	) -> Result<bool, DispatchError> {
		ensure!(
			ClassDataCollection::<T>::contains_key(class_id),
			Error::<T>::ClassIdNotFound
		);

		let class_collection_id = ClassDataCollection::<T>::get(class_id);

		Ok(class_collection_id == collection_id)
	}

	fn check_item_on_listing(class_id: Self::ClassId, token_id: Self::TokenId) -> Result<bool, DispatchError> {
		let fixed_class_id = TryInto::<ClassId>::try_into(class_id).unwrap_or_default();
		let fixed_nft_id = TryInto::<TokenId>::try_into(token_id).unwrap_or_default();

		Ok(T::AuctionHandler::check_item_in_auction(ItemId::NFT(
			fixed_class_id,
			fixed_nft_id,
		)))
	}

	fn transfer_nft(sender: &T::AccountId, to: &T::AccountId, nft: &(Self::ClassId, Self::TokenId)) -> DispatchResult {
		Self::do_transfer(sender, to, nft.clone())?;

		Ok(())
	}

	fn is_transferable(nft: &(Self::ClassId, Self::TokenId)) -> Result<bool, DispatchError> {
		let class_info = NftModule::<T>::classes(nft.0).ok_or(Error::<T>::ClassIdNotFound)?;
		let data = class_info.data;
		Ok(data.token_type.is_transferable())
	}

	fn get_class_fund(class_id: &Self::ClassId) -> T::AccountId {
		T::PalletId::get().into_sub_account(class_id)
	}
}
