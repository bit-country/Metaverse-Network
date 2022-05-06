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
use orml_nft::{ClassInfo, ClassInfoOf, Classes, Pallet as NftModule, TokenInfo, TokenInfoOf, TokenMetadataOf, Tokens};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::traits::Saturating;
use sp_runtime::{
	traits::{AccountIdConversion, Dispatchable, One},
	DispatchError,
};
use sp_runtime::{Perbill, RuntimeDebug};
use sp_std::vec::Vec;
use sp_std::{collections::btree_map::BTreeMap, prelude::*};

use auction_manager::{Auction, CheckAuctionItemHandler};
pub use pallet::*;
pub use primitive_traits::{Attributes, NFTTrait, NftClassData, NftGroupCollectionData, NftMetadata, TokenType};
use primitive_traits::{CollectionType, NftAssetData, NftAssetDataV1, NftClassDataV1};
use primitives::{
	AssetId, BlockNumber, ClassId, GroupCollectionId, Hash, ItemId, TokenId, ESTATE_CLASS_ID, LAND_CLASS_ID,
};
pub use weights::WeightInfo;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod weights;

const TIMECAPSULE_ID: LockIdentifier = *b"bctimeca";

#[derive(codec::Encode, codec::Decode, Clone, frame_support::RuntimeDebug, PartialEq)]
pub enum StorageVersion {
	V0,
	V1,
}

#[frame_support::pallet]
pub mod pallet {
	use orml_traits::{MultiCurrency, MultiCurrencyExtended};
	use sp_runtime::ArithmeticError;

	use primitive_traits::{CollectionType, NftAssetData, NftGroupCollectionData, NftMetadata, TokenType};
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
		/// Default minting price per NFT token
		#[pallet::constant]
		type AssetMintingFee: Get<BalanceOf<Self>>;
		/// Default minting price per NFT token class
		#[pallet::constant]
		type ClassMintingFee: Get<BalanceOf<Self>>;
		/// Treasury
		#[pallet::constant]
		type Treasury: Get<PalletId>;
		/// Currency type for reserve/unreserve balance
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		/// NFT Module Id
		#[pallet::constant]
		type PalletId: Get<PalletId>;
		/// Weight info
		type WeightInfo: WeightInfo;
		/// Auction Handler
		type AuctionHandler: Auction<Self::AccountId, Self::BlockNumber> + CheckAuctionItemHandler<BalanceOf<Self>>;
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
	}

	pub type ClassIdOf<T> = <T as orml_nft::Config>::ClassId;
	pub type TokenIdOf<T> = <T as orml_nft::Config>::TokenId;
	pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

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

	#[pallet::genesis_config]
	pub struct GenesisConfig {}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			GenesisConfig {}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			// Pre-mint group collection for lands
			let land_collection_data = NftGroupCollectionData {
				name: "MetaverseLands".as_bytes().to_vec(),
				properties: "MetaverseId;Coordinates".as_bytes().to_vec(),
			};
			let land_collection_id = <Pallet<T>>::next_group_collection_id();
			<GroupCollections<T>>::insert(land_collection_id, land_collection_data);
			<NextGroupCollectionId<T>>::set(land_collection_id + 1);
			<AllNftGroupCollection<T>>::set(land_collection_id + 1);
			<Pallet<T>>::deposit_event(Event::NewNftCollectionCreated(land_collection_id));

			// Pre-mint group collection for estates
			let estate_collection_data = NftGroupCollectionData {
				name: "MetaverseEstate".as_bytes().to_vec(),
				properties: "MetaverseId;EstateId".as_bytes().to_vec(),
			};
			let estate_collection_id = <Pallet<T>>::next_group_collection_id();
			<GroupCollections<T>>::insert(estate_collection_id, estate_collection_data);
			<NextGroupCollectionId<T>>::set(estate_collection_id + 1);
			<AllNftGroupCollection<T>>::set(estate_collection_id + 1);
			<Pallet<T>>::deposit_event(Event::NewNftCollectionCreated(estate_collection_id));
		}
	}

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
		/// Collection id does not exist
		CollectionDoesNotExist,
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
		/// NFT Royalty fee exceed 50%
		RoyaltyFeeExceedLimit,
		/// NFT Asset is locked e.g on marketplace, or other locks
		AssetIsLocked,
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
			let new_all_nft_collection_count = all_collection_count.checked_add(One::one()).ok_or("Overflow")?;

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
			royalty_fee: Perbill,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let class_id = Self::do_create_class(
				&sender,
				metadata,
				attributes,
				collection_id,
				token_type,
				collection_type,
				royalty_fee,
			)?;
			Self::deposit_event(Event::<T>::NewNftClassCreated(sender, class_id));

			Ok(().into())
		}

		#[pallet::weight(< T as Config >::WeightInfo::mint() * * quantity as u64)]
		pub fn mint(
			origin: OriginFor<T>,
			class_id: ClassIdOf<T>,
			metadata: NftMetadata,
			attributes: Attributes,
			quantity: u32,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let minting_outcome = Self::do_mint_nfts(&sender, class_id, metadata, attributes, quantity)?;

			Self::deposit_event(Event::<T>::NewNftMinted(
				*minting_outcome.0.first().unwrap(),
				*minting_outcome.0.last().unwrap(),
				sender,
				class_id,
				quantity,
				minting_outcome.1,
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

		#[pallet::weight(T::WeightInfo::transfer_batch() * tos.len() as u64)]
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

			let token_info = NftModule::<T>::tokens(asset_id.0, asset_id.1).ok_or(Error::<T>::AssetInfoNotFound)?;

			ensure!(token_info.owner != sender, Error::<T>::SignOwnAsset);

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
			Self::do_burn(&sender, &asset_id)?;
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
		//		fn on_runtime_upgrade() -> Weight {
		//			Self::upgrade_class_data_v2();
		//			0
		//		}
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

	pub fn do_transfer(
		sender: &T::AccountId,
		to: &T::AccountId,
		asset_id: (ClassIdOf<T>, TokenIdOf<T>),
	) -> Result<<T as orml_nft::Config>::TokenId, DispatchError> {
		ensure!(!Self::is_collection_locked(&asset_id.0), Error::<T>::CollectionIsLocked);

		let class_info = NftModule::<T>::classes(asset_id.0).ok_or(Error::<T>::ClassIdNotFound)?;
		let data = class_info.data;
		let token_info = NftModule::<T>::tokens(asset_id.0, asset_id.1).ok_or(Error::<T>::AssetInfoNotFound)?;

		ensure!(!token_info.data.is_locked, Error::<T>::AssetIsLocked);

		match data.token_type {
			TokenType::Transferable => {
				let check_ownership = Self::check_nft_ownership(&sender, &asset_id)?;
				ensure!(check_ownership, Error::<T>::NoPermission);

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

		NftModule::<T>::transfer(&sender, &to, asset_id.clone())?;
		Ok(asset_id.1)
	}

	fn do_mint_nfts(
		sender: &T::AccountId,
		class_id: ClassIdOf<T>,
		metadata: NftMetadata,
		attributes: Attributes,
		quantity: u32,
	) -> Result<(Vec<(ClassIdOf<T>, TokenIdOf<T>)>, TokenIdOf<T>), DispatchError> {
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
		ensure!(sender.clone() == class_info.owner, Error::<T>::NoPermission);
		let class_fund: T::AccountId = T::Treasury::get().into_account();
		let deposit = T::AssetMintingFee::get().saturating_mul(Into::<BalanceOf<T>>::into(quantity));
		<T as Config>::Currency::transfer(&sender, &class_fund, deposit, ExistenceRequirement::KeepAlive)?;

		let new_nft_data = NftAssetData {
			deposit,
			attributes: attributes,
			is_locked: false,
		};

		let mut new_asset_ids: Vec<(ClassIdOf<T>, TokenIdOf<T>)> = Vec::new();
		let mut last_token_id: TokenIdOf<T> = Default::default();

		for _ in 0..quantity {
			let token_id = NftModule::<T>::mint(&sender, class_id, metadata.clone(), new_nft_data.clone())?;
			new_asset_ids.push((class_id, token_id));

			last_token_id = token_id;
		}
		Ok((new_asset_ids, last_token_id))
	}

	fn do_create_class(
		sender: &T::AccountId,
		metadata: NftMetadata,
		attributes: Attributes,
		collection_id: GroupCollectionId,
		token_type: TokenType,
		collection_type: CollectionType,
		royalty_fee: Perbill,
	) -> Result<<T as orml_nft::Config>::ClassId, DispatchError> {
		ensure!(
			metadata.len() as u32 <= T::MaxMetadata::get(),
			Error::<T>::ExceedMaximumMetadataLength
		);
		let next_class_id = NftModule::<T>::next_class_id();
		ensure!(
			GroupCollections::<T>::contains_key(collection_id),
			Error::<T>::CollectionDoesNotExist
		);

		ensure!(
			royalty_fee <= Perbill::from_percent(25u32),
			Error::<T>::RoyaltyFeeExceedLimit
		);

		// Class fund
		let class_fund: T::AccountId = T::Treasury::get().into_account();

		// Secure deposit of token class owner
		let class_deposit = T::ClassMintingFee::get();
		// Transfer fund to pot
		<T as Config>::Currency::transfer(&sender, &class_fund, class_deposit, ExistenceRequirement::KeepAlive)?;

		let class_data = NftClassData {
			deposit: class_deposit,
			token_type,
			collection_type,
			attributes,
			is_locked: false,
			royalty_fee,
		};

		NftModule::<T>::create_class(&sender, metadata, class_data)?;
		ClassDataCollection::<T>::insert(next_class_id, collection_id);
		Ok(next_class_id)
	}

	fn do_burn(sender: &T::AccountId, asset_id: &(ClassIdOf<T>, TokenIdOf<T>)) -> DispatchResult {
		NftModule::<T>::burn(&sender, *asset_id)?;
		Ok(())
	}

	pub fn upgrade_class_data_v2() -> Weight {
		log::info!("Start upgrading nft class data v2");
		log::info!("Start upgrading nft token data v2");
		let mut num_nft_classes = 0;
		let mut num_nft_tokens = 0;
		let mut asset_by_owner_updates = 0;

		Classes::<T>::translate(
			|_k,
			 class_info: ClassInfo<
				T::TokenId,
				T::AccountId,
				NftClassDataV1<BalanceOf<T>>,
				BoundedVec<u8, T::MaxClassMetadata>,
			>| {
				num_nft_classes += 1;
				log::info!("Upgrading class data");
				log::info!("Class id {:?}", _k);

				let new_data = NftClassData {
					deposit: class_info.data.deposit,
					attributes: class_info.data.attributes,
					token_type: class_info.data.token_type,
					collection_type: class_info.data.collection_type,
					is_locked: false,
					royalty_fee: Perbill::from_percent(0u32),
				};

				let v: ClassInfoOf<T> = ClassInfo {
					metadata: class_info.metadata,
					total_issuance: class_info.total_issuance,
					owner: class_info.owner,
					data: new_data,
				};
				Some(v)
			},
		);
		Tokens::<T>::translate(
			|_k, _k2, token_info: TokenInfo<T::AccountId, NftAssetDataV1<BalanceOf<T>>, TokenMetadataOf<T>>| {
				num_nft_tokens += 1;
				log::info!("Upgrading existing token data to set is_locked");
				log::info!("Token id {:?}", _k);

				let new_data = NftAssetData {
					deposit: token_info.data.deposit,
					attributes: token_info.data.attributes,
					is_locked: false,
				};

				let v: TokenInfoOf<T> = TokenInfo {
					metadata: token_info.metadata,
					owner: token_info.owner,
					data: new_data,
				};
				Some(v)
			},
		);

		log::info!("Classes upgraded: {}", num_nft_classes);
		log::info!("Tokens upgraded: {}", num_nft_tokens);
		0
	}
}

impl<T: Config> NFTTrait<T::AccountId, BalanceOf<T>> for Pallet<T> {
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

	fn get_nft_detail(asset_id: (Self::ClassId, Self::TokenId)) -> Result<NftClassData<BalanceOf<T>>, DispatchError> {
		let asset_info = NftModule::<T>::classes(asset_id.0).ok_or(Error::<T>::AssetInfoNotFound)?;

		Ok(asset_info.data)
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

	fn create_token_class(
		sender: &T::AccountId,
		metadata: NftMetadata,
		attributes: Attributes,
		collection_id: GroupCollectionId,
		token_type: TokenType,
		collection_type: CollectionType,
		royalty_fee: Perbill,
	) -> Result<ClassId, DispatchError> {
		let class_id = Self::do_create_class(
			sender,
			metadata,
			attributes,
			collection_id,
			token_type,
			collection_type,
			royalty_fee,
		)?;
		Ok(TryInto::<ClassId>::try_into(class_id).unwrap_or_default())
	}

	fn mint_token(
		sender: &T::AccountId,
		class_id: ClassId,
		metadata: NftMetadata,
		attributes: Attributes,
	) -> Result<TokenId, DispatchError> {
		let class: Self::ClassId = TryInto::<Self::ClassId>::try_into(class_id).unwrap_or_default();
		let outcome = Self::do_mint_nfts(sender, class, metadata, attributes, 1)?;
		let nft_token = *outcome.0.first().unwrap();
		Ok(TryInto::<TokenId>::try_into(nft_token.1).unwrap_or_default())
	}

	fn burn_nft(account: &T::AccountId, nft: &(Self::ClassId, Self::TokenId)) -> DispatchResult {
		Self::do_burn(account, nft)?;

		Ok(())
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

	fn set_lock_collection(class_id: Self::ClassId, is_locked: bool) -> sp_runtime::DispatchResult {
		Classes::<T>::try_mutate(class_id, |class_info| -> DispatchResult {
			let info = class_info.as_mut().ok_or(Error::<T>::ClassIdNotFound)?;
			info.data.is_locked = is_locked;
			Ok(())
		})
	}

	fn set_lock_nft(token_id: (Self::ClassId, Self::TokenId), is_locked: bool) -> sp_runtime::DispatchResult {
		Tokens::<T>::try_mutate(token_id.0, token_id.1, |token_info| -> DispatchResult {
			let t = token_info.as_mut().ok_or(Error::<T>::AssetInfoNotFound)?;
			t.data.is_locked = is_locked;
			Ok(())
		})
	}
}
