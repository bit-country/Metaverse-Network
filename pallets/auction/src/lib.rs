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

use frame_support::traits::{Currency, ExistenceRequirement, LockableCurrency, ReservableCurrency};
use frame_support::{ensure, pallet_prelude::*, transactional};
use frame_system::{self as system, ensure_signed};
use sp_core::sp_std::convert::TryInto;
use sp_runtime::SaturatedConversion;
use sp_runtime::{
	traits::{CheckedDiv, One, Saturating, Zero},
	DispatchError, DispatchResult, Perbill,
};
use sp_std::vec::Vec;

use auction_manager::{Auction, AuctionHandler, AuctionInfo, AuctionItem, AuctionType, Change, OnNewBidResult};
use core_primitives::UndeployedLandBlocksTrait;
pub use pallet::*;
use pallet_nft::Pallet as NFTModule;
use primitives::{continuum::MapTrait, estate::Estate, AuctionId, ItemId, NftOffer};
pub use weights::WeightInfo;

//#[cfg(feature = "runtime-benchmarks")]
//pub mod benchmarking;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod weights;

pub struct AuctionLogicHandler;

pub mod migration_v2 {
	use codec::FullCodec;
	use codec::{Decode, Encode};
	use scale_info::TypeInfo;
	#[cfg(feature = "std")]
	use serde::{Deserialize, Serialize};
	use sp_runtime::{traits::AtLeast32BitUnsigned, DispatchError, RuntimeDebug};
	use sp_std::{
		cmp::{Eq, PartialEq},
		fmt::Debug,
		vec::Vec,
	};

	use auction_manager::{AuctionType, ListingLevel};
	use primitives::{AssetId, EstateId, FungibleTokenId, MetaverseId};

	#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum V1ItemId {
		NFT(AssetId),
		Spot(u64, MetaverseId),
		Country(MetaverseId),
		Block(u64),
		Estate(EstateId),
		LandUnit((i32, i32), MetaverseId),
	}

	#[cfg_attr(feature = "std", derive(PartialEq, Eq))]
	#[derive(Encode, Decode, Clone, RuntimeDebug, TypeInfo)]
	pub struct AuctionItem<AccountId, BlockNumber, Balance> {
		pub item_id: V1ItemId,
		pub recipient: AccountId,
		pub initial_amount: Balance,
		/// Current amount for sale
		pub amount: Balance,
		/// Auction start time
		pub start_time: BlockNumber,
		pub end_time: BlockNumber,
		pub auction_type: AuctionType,
		pub listing_level: ListingLevel<AccountId>,
		pub currency_id: FungibleTokenId,
		pub listing_fee: Balance,
	}
}

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResultWithPostInfo, traits::tokens::currency};
	use frame_support::log;
	use frame_support::sp_runtime::traits::CheckedSub;
	use frame_system::ensure_root;
	use frame_system::pallet_prelude::OriginFor;
	use orml_traits::{MultiCurrency, MultiReservableCurrency};
	use sp_runtime::traits::CheckedAdd;
	use sp_runtime::ArithmeticError;

	use auction_manager::{AuctionItemV1, CheckAuctionItemHandler, ListingLevel};
	use core_primitives::{MetaverseTrait, NFTTrait};
	use primitives::{AssetId, Balance, ClassId, FungibleTokenId, MetaverseId, TokenId};

	use crate::migration_v2::V1ItemId;

	use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	pub(super) type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Default auction close time if there is no end time specified
		#[pallet::constant]
		type AuctionTimeToClose: Get<Self::BlockNumber>;

		/// The `AuctionHandler` trait that allow custom bidding logic and handles auction result
		type Handler: AuctionHandler<Self::AccountId, BalanceOf<Self>, Self::BlockNumber, AuctionId>;

		/// Native currency type that handles currency in auction
		type Currency: ReservableCurrency<Self::AccountId>
			+ LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;

		/// Multi currencies type that handles different currency type in auction
		type FungibleTokenCurrency: MultiReservableCurrency<
			Self::AccountId,
			CurrencyId = FungibleTokenId,
			Balance = Balance,
		>;
		/// Continuum protocol handler for Continuum Spot Auction
		type ContinuumHandler: MapTrait<Self::AccountId>;

		/// Metaverse info trait for getting information from metaverse
		type MetaverseInfoSource: MetaverseTrait<Self::AccountId>;

		/// Minimum auction duration when new listing created.
		#[pallet::constant]
		type MinimumAuctionDuration: Get<Self::BlockNumber>;

		/// Estate handler that support land and estate listing
		type EstateHandler: Estate<Self::AccountId>;

		/// Max number of listing can be finalised in a single block
		#[pallet::constant]
		type MaxFinality: Get<u32>;

		/// Max number of items in bundle can be finalised in an auction
		#[pallet::constant]
		type MaxBundleItem: Get<u32>;

		/// NFT trait type that handler NFT implementation
		type NFTHandler: NFTTrait<Self::AccountId, BalanceOf<Self>, ClassId = ClassId, TokenId = TokenId>;

		/// Network fee that will be reserved when an item is listed for auction or buy now.
		/// The fee will be unreserved after the auction or buy now is completed.
		#[pallet::constant]
		type NetworkFeeReserve: Get<BalanceOf<Self>>;

		/// Network fee that will be collected when auction or buy now is completed.
		#[pallet::constant]
		type NetworkFeeCommission: Get<Perbill>;

		/// Weight info
		type WeightInfo: WeightInfo;

		/// Offer duration
		#[pallet::constant]
		type OfferDuration: Get<Self::BlockNumber>;

		/// Minimum listing price
		#[pallet::constant]
		type MinimumListingPrice: Get<BalanceOf<Self>>;
	}

	#[pallet::storage]
	#[pallet::getter(fn auctions)]
	/// Stores on-going and future auctions. Closed auction are removed.
	pub(super) type Auctions<T: Config> =
		StorageMap<_, Twox64Concat, AuctionId, AuctionInfo<T::AccountId, BalanceOf<T>, T::BlockNumber>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_auction_item)]
	/// Store asset with Auction
	pub(super) type AuctionItems<T: Config> =
		StorageMap<_, Twox64Concat, AuctionId, AuctionItem<T::AccountId, T::BlockNumber, BalanceOf<T>>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn items_in_auction)]
	/// Track which Assets are in auction
	pub(super) type ItemsInAuction<T: Config> = StorageMap<_, Twox64Concat, ItemId<BalanceOf<T>>, bool, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn auctions_index)]
	/// Track the next auction ID.
	pub(super) type AuctionsIndex<T: Config> = StorageValue<_, AuctionId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn auction_end_time)]
	/// Index auctions by end time.
	pub(super) type AuctionEndTime<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::BlockNumber, Twox64Concat, AuctionId, (), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn authorised_metaverse_collection)]
	/// Local marketplace collection authorisation
	pub(super) type MetaverseCollection<T: Config> =
		StorageDoubleMap<_, Twox64Concat, MetaverseId, Twox64Concat, ClassId, (), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn nft_offers)]
	/// Index NFT offers by token and oferror
	pub(super) type Offers<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		(ClassId, TokenId),
		Blake2_128Concat,
		T::AccountId,
		NftOffer<BalanceOf<T>, T::BlockNumber>,
		OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A bid is placed. [auction_id, bidder, bidding_amount]
		Bid(AuctionId, T::AccountId, BalanceOf<T>),
		/// New auction item created. [auction_id, bidder, listing_level, initial_amount,
		/// initial_amount, end_block]
		NewAuctionItem(
			AuctionId,
			T::AccountId,
			ListingLevel<T::AccountId>,
			BalanceOf<T>,
			BalanceOf<T>,
			T::BlockNumber,
		),
		/// Auction finalized. [auction_id, bidder, amount]
		AuctionFinalized(AuctionId, T::AccountId, BalanceOf<T>),
		/// Buy finalized. [auction_id, bidder, amount]
		BuyNowFinalised(AuctionId, T::AccountId, BalanceOf<T>),
		/// Listing finalized with no bid. [auction_id]
		AuctionFinalizedNoBid(AuctionId),
		/// NFT Collection authorized for listing in marketplace. [class_id, metaverse_id]
		CollectionAuthorizedInMetaverse(ClassId, MetaverseId),
		/// NFT Collection authorization removed for listing in marketplace. [class_id,
		/// metaverse_id]
		CollectionAuthorizationRemoveInMetaverse(ClassId, MetaverseId),
		/// Cancel listing with auction id. [class_id,
		/// metaverse_id]
		AuctionCancelled(AuctionId),
		/// Nft offer is made [class_id, token_id, account_id, offer amount]
		NftOfferMade(ClassId, TokenId, T::AccountId, BalanceOf<T>),
		/// Nft offer is accepted [class_id, token_id, account_id]
		NftOfferAccepted(ClassId, TokenId, T::AccountId),
		/// Nft offer is withdrawn [class_id, token_id, account_id]
		NftOfferWithdrawn(ClassId, TokenId, T::AccountId),
	}

	/// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Auction does not exist
		AuctionDoesNotExist,
		/// Asset for listing does not exist
		AssetDoesNotExist,
		/// Auction has not started
		AuctionHasNotStarted,
		/// Auction is expired
		AuctionIsExpired,
		/// Auction type is supported for listing
		AuctionTypeIsNotSupported,
		/// Bid is not accepted e.g owner == bidder, listing stop accepting bid
		BidIsNotAccepted,
		/// Insufficient free balance for bidding
		InsufficientFreeBalance,
		/// Bid price is invalid
		InvalidBidPrice,
		/// Auction is not found, either expired and not valid
		NoAvailableAuctionId,
		/// Has no permission to create auction. Check listing authorization
		NoPermissionToCreateAuction,
		/// Has no permission to cancel auction.
		NoPermissionToCancelAuction,
		/// Self bidding is not accepted
		CannotBidOnOwnAuction,
		/// Buy now input price is not valid
		InvalidBuyNowPrice,
		/// Invalid auction type
		InvalidAuctionType,
		/// Asset already in Auction
		ItemAlreadyInAuction,
		/// Wrong Listing Level
		WrongListingLevel,
		/// Social Token Currency is not exist
		FungibleTokenCurrencyNotFound,
		/// Minimum Duration Is Too Low
		AuctionEndIsLessThanMinimumDuration,
		/// There is too many auction ends at the same time.
		ExceedFinalityLimit,
		/// There is too many item inside the bundle.
		ExceedBundleLimit,
		/// Estate does not exist, check if estate id is correct
		EstateDoesNotExist,
		/// Land unit does not exist, check if estate id is correct
		LandUnitDoesNotExist,
		/// Undeployed land block does not exist or is not available for auction
		UndeployedLandBlockDoesNotExistOrNotAvailable,
		/// User has no permission to authorise collection
		NoPermissionToAuthoriseCollection,
		/// Collection has already authorised
		CollectionAlreadyAuthorised,
		/// Collection is not authorised
		CollectionIsNotAuthorised,
		/// Auction already started or got bid
		AuctionAlreadyStartedOrBid,
		/// The account has already made offer for a given NFT
		OfferAlreadyExists,
		/// The NFT offer does not exist
		OfferDoesNotExist,
		/// The NFT offer is expired
		OfferIsExpired,
		/// No permission to make offer for a NFT.
		NoPermissionToMakeOffer,
		/// No permission to accept offer for a NFT.
		NoPermissionToAcceptOffer,
		/// Listing price is below the minimum.
		ListingPriceIsBelowMinimum,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// User bid for any available auction.
		///
		/// The dispatch origin for this call must be _Signed_.
		/// `id`: auction id that user wants to bid
		/// `value`: the value of the bid
		/// Fund will be reserved if bid accepted and release the fund of previous bidder at the
		/// same time
		///
		///
		/// Emits `Bid` if successful.
		#[pallet::weight(T::WeightInfo::bid())]
		#[transactional]
		pub fn bid(origin: OriginFor<T>, id: AuctionId, value: BalanceOf<T>) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;

			let auction_item: AuctionItem<T::AccountId, T::BlockNumber, BalanceOf<T>> =
				Self::get_auction_item(id.clone()).ok_or(Error::<T>::AuctionDoesNotExist)?;

			ensure!(
				!auction_item.item_id.is_map_spot(),
				Error::<T>::AuctionTypeIsNotSupported
			);

			Self::auction_bid_handler(from, id, value)?;

			Ok(().into())
		}

		/// User buy for any available buy now listing.
		///
		/// The dispatch origin for this call must be _Signed_.
		/// `auction_id`: the id of auction that user want to bid
		/// `value`: the bid value
		/// Fund will be transfer immediately if buy now price is accepted and asset will be
		/// transferred to sender
		///
		///
		/// Emits `BuyNowFinalised` if successful.
		#[pallet::weight(T::WeightInfo::buy_now())]
		#[transactional]
		pub fn buy_now(origin: OriginFor<T>, auction_id: AuctionId, value: BalanceOf<T>) -> DispatchResult {
			let from = ensure_signed(origin)?;

			let auction_item: AuctionItem<T::AccountId, T::BlockNumber, BalanceOf<T>> =
				Self::get_auction_item(auction_id.clone()).ok_or(Error::<T>::AuctionDoesNotExist)?;
			ensure!(
				!auction_item.item_id.is_map_spot(),
				Error::<T>::AuctionTypeIsNotSupported
			);

			Self::buy_now_handler(from, auction_id, value)?;

			Ok(())
		}

		/// User create new auction listing if they are metaverse owner of their local marketplace
		/// or NFT collection has authorized to list
		///
		/// The dispatch origin for this call must be _Signed_.
		/// - `item_id`: he enum of what item type want to list
		/// - `value`: value of the listing
		/// - `listing_level`: if listing is on local or global marketplace
		/// - `end_time`: the listing end time.
		///
		/// Emits `NewAuctionItem` if successful.
		#[pallet::weight(T::WeightInfo::create_new_auction())]
		#[transactional]
		pub fn create_new_auction(
			origin: OriginFor<T>,
			item_id: ItemId<BalanceOf<T>>,
			value: BalanceOf<T>,
			end_time: T::BlockNumber,
			listing_level: ListingLevel<T::AccountId>,
			currency_id: FungibleTokenId,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;
			// Only support NFT on marketplace
			ensure!(
				(matches!(item_id, ItemId::NFT(_, _)) && matches!(listing_level, ListingLevel::Local(_)))
					|| (matches!(item_id, ItemId::Bundle(_)) && matches!(listing_level, ListingLevel::Local(_)))
					|| (matches!(item_id, ItemId::UndeployedLandBlock(_))
						&& matches!(listing_level, ListingLevel::Global)),
				Error::<T>::NoPermissionToCreateAuction
			);

			let start_time: T::BlockNumber = <system::Pallet<T>>::block_number();

			let remaining_time: T::BlockNumber = end_time.checked_sub(&start_time).ok_or(ArithmeticError::Overflow)?;

			// Ensure auction duration is valid
			ensure!(
				remaining_time >= T::MinimumAuctionDuration::get(),
				Error::<T>::AuctionEndIsLessThanMinimumDuration
			);

			let mut listing_fee: Perbill = Perbill::from_percent(0u32);
			if let ListingLevel::Local(metaverse_id) = listing_level {
				listing_fee = T::MetaverseInfoSource::get_metaverse_marketplace_listing_fee(metaverse_id)?;
			}

			Self::create_auction(
				AuctionType::Auction,
				item_id,
				Some(end_time),
				from.clone(),
				value.clone(),
				start_time,
				listing_level.clone(),
				listing_fee,
				currency_id,
			)?;
			Ok(().into())
		}

		/// User create new buy-now's listing if they are metaverse owner of their local marketplace
		/// or NFT collection has authorized to list
		///
		/// The dispatch origin for this call must be _Signed_.
		/// - `item_id`: the enum of what item type want to list
		/// - `value`: value of the listing
		/// - `listing_level`: if listing is on local or global marketplace
		/// - `end_time`: the listing end time.
		///
		/// Emits `NewAuctionItem` if successful.
		#[pallet::weight(T::WeightInfo::create_new_buy_now())]
		#[transactional]
		pub fn create_new_buy_now(
			origin: OriginFor<T>,
			item_id: ItemId<BalanceOf<T>>,
			value: BalanceOf<T>,
			end_time: T::BlockNumber,
			listing_level: ListingLevel<T::AccountId>,
			currency_id: FungibleTokenId,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;
			ensure!(
				(matches!(item_id, ItemId::NFT(_, _)) && matches!(listing_level, ListingLevel::Local(_)))
					|| (matches!(item_id, ItemId::Bundle(_)) && matches!(listing_level, ListingLevel::Local(_)))
					|| (matches!(item_id, ItemId::UndeployedLandBlock(_))
						&& matches!(listing_level, ListingLevel::Global)),
				Error::<T>::NoPermissionToCreateAuction
			);

			let start_time: T::BlockNumber = <system::Pallet<T>>::block_number();
			let remaining_time: T::BlockNumber = end_time.checked_sub(&start_time).ok_or(ArithmeticError::Overflow)?;

			// Ensure auction duration is valid
			ensure!(
				remaining_time >= T::MinimumAuctionDuration::get(),
				Error::<T>::AuctionEndIsLessThanMinimumDuration
			);

			let mut listing_fee: Perbill = Perbill::from_percent(0u32);
			if let ListingLevel::Local(metaverse_id) = listing_level {
				listing_fee = T::MetaverseInfoSource::get_metaverse_marketplace_listing_fee(metaverse_id)?;
			}

			Self::create_auction(
				AuctionType::BuyNow,
				item_id,
				Some(end_time),
				from.clone(),
				value.clone(),
				start_time,
				listing_level.clone(),
				listing_fee,
				currency_id,
			)?;

			Ok(().into())
		}

		/// Metaverse owner can authorize collection that sell in their local marketplace
		///
		/// The dispatch origin for this call must be _Signed_. Only owner of metaverse can make
		/// this call
		/// - `class_id`: the nft collection that want to authorize
		/// - `metaverse_id`: the metaverse id that user want to authorize
		///
		/// Emits `CollectionAuthorizedInMetaverse` if successful.
		#[pallet::weight(T::WeightInfo::authorise_metaverse_collection())]
		#[transactional]
		pub fn authorise_metaverse_collection(
			origin: OriginFor<T>,
			class_id: ClassId,
			metaverse_id: MetaverseId,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;
			ensure!(
				T::MetaverseInfoSource::check_ownership(&from, &metaverse_id),
				Error::<T>::NoPermissionToAuthoriseCollection
			);

			ensure!(
				!MetaverseCollection::<T>::contains_key(metaverse_id, class_id),
				Error::<T>::CollectionAlreadyAuthorised
			);

			MetaverseCollection::<T>::insert(metaverse_id.clone(), class_id.clone(), ());

			Self::deposit_event(Event::<T>::CollectionAuthorizedInMetaverse(class_id, metaverse_id));

			Ok(().into())
		}

		/// Metaverse owner can remove authorized collection that sell in their local marketplace
		///
		/// The dispatch origin for this call must be _Signed_. Only owner of metaverse can make
		/// this call
		/// - `class_id`: the nft collection that want to authorize
		/// - `metaverse_id`: the metaverse id that user want to authorize
		///
		/// Emits `CollectionAuthorizationRemoveInMetaverse` if successful.
		#[pallet::weight(T::WeightInfo::remove_authorise_metaverse_collection())]
		#[transactional]
		pub fn remove_authorise_metaverse_collection(
			origin: OriginFor<T>,
			class_id: ClassId,
			metaverse_id: MetaverseId,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;
			ensure!(
				T::MetaverseInfoSource::check_ownership(&from, &metaverse_id),
				Error::<T>::NoPermissionToAuthoriseCollection
			);

			ensure!(
				MetaverseCollection::<T>::contains_key(metaverse_id, class_id),
				Error::<T>::CollectionIsNotAuthorised
			);

			MetaverseCollection::<T>::remove(metaverse_id.clone(), class_id.clone());
			Self::deposit_event(Event::<T>::CollectionAuthorizationRemoveInMetaverse(
				class_id,
				metaverse_id,
			));
			Ok(().into())
		}

		/// Cancel listing that has no bid or buy now.
		///
		/// The dispatch origin for this call must be _Root_.
		/// this call
		/// - `from`: the listing owner who created this listing
		/// - `auction_id`: the auction id that wish to cancel
		///
		/// Emits `CollectionAuthorizationRemoveInMetaverse` if successful.
		#[pallet::weight(T::WeightInfo::remove_authorise_metaverse_collection())]
		#[transactional]
		pub fn cancel_listing(
			origin: OriginFor<T>,
			from: T::AccountId,
			auction_id: AuctionId,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			ensure!(Auctions::<T>::contains_key(auction_id), Error::<T>::AuctionDoesNotExist);
			let auction_item = AuctionItems::<T>::get(auction_id).ok_or(Error::<T>::AuctionDoesNotExist)?;

			match auction_item.clone().item_id {
				ItemId::NFT(class_id, token_id) => {
					ensure!(
						T::NFTHandler::check_ownership(&from, &(class_id, token_id))?,
						Error::<T>::NoPermissionToCancelAuction
					);

					ensure!(auction_item.recipient == from, Error::<T>::AuctionAlreadyStartedOrBid);

					Self::remove_auction(auction_id, auction_item.item_id);
					T::NFTHandler::set_lock_nft((class_id, token_id), false)?;
					T::Currency::unreserve(&auction_item.recipient, T::NetworkFeeReserve::get());

					Self::deposit_event(Event::<T>::AuctionCancelled(auction_id));
					Self::deposit_event(Event::<T>::AuctionFinalizedNoBid(auction_id));

					Ok(().into())
				}
				ItemId::Bundle(tokens) => {
					ensure!(auction_item.recipient == from, Error::<T>::AuctionAlreadyStartedOrBid);
					for item in tokens {
						// Check ownership
						ensure!(
							T::NFTHandler::check_ownership(&from, &(item.0, item.1))?,
							Error::<T>::NoPermissionToCancelAuction
						);

						// Lock NFT
						T::NFTHandler::set_lock_nft((item.0, item.1), false)?
					}

					Self::remove_auction(auction_id, auction_item.item_id);
					T::Currency::unreserve(&auction_item.recipient, T::NetworkFeeReserve::get());

					Self::deposit_event(Event::<T>::AuctionCancelled(auction_id));
					Self::deposit_event(Event::<T>::AuctionFinalizedNoBid(auction_id));

					Ok(().into())
				}
				_ => Err(Error::<T>::NoPermissionToCancelAuction.into()),
			}
		}

		/// Make offer for an NFT asset
		///
		/// The dispatch origin for this call must be _Signed_.
		/// Only accounts that does not own the NFT asset can make this call
		/// - `asset`: the NFT for which an offer will be made.
		/// - `offer_amount`: the  amount of native tokens offered in exchange of the nft.
		///
		/// Emits `NftOfferMade` if successful.
		#[pallet::weight(T::WeightInfo::make_offer())]
		#[transactional]
		pub fn make_offer(
			origin: OriginFor<T>,
			asset: (ClassId, TokenId),
			offer_amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let offeror = ensure_signed(origin)?;
			ensure!(
				!T::NFTHandler::check_ownership(&offeror, &asset)?,
				Error::<T>::NoPermissionToMakeOffer
			);
			ensure!(
				T::NFTHandler::is_transferable(&asset)?,
				Error::<T>::NoPermissionToMakeOffer
			);
			ensure!(
				!Offers::<T>::contains_key(asset.clone(), offeror.clone()),
				Error::<T>::OfferAlreadyExists
			);

			T::Currency::reserve(&offeror, offer_amount);
			let offer_end_block = <frame_system::Pallet<T>>::block_number() + T::OfferDuration::get();
			let offer = NftOffer {
				amount: offer_amount,
				end_block: offer_end_block,
			};
			Offers::<T>::insert(asset, offeror.clone(), offer);

			Self::deposit_event(Event::<T>::NftOfferMade(asset.0, asset.1, offeror, offer_amount));

			Ok(().into())
		}

		/// Accept offer for an NFT asset
		///
		/// The dispatch origin for this call must be _Signed_.
		/// Only NFT owner can make this call.
		/// - `asset`: the NFT for which te offer will be accepted.
		/// - `offeror`: the account whose offer will be accepted.
		///
		/// Emits `NftOfferAccepted` if successful.
		#[pallet::weight(T::WeightInfo::accept_offer())]
		#[transactional]
		pub fn accept_offer(
			origin: OriginFor<T>,
			asset: (ClassId, TokenId),
			offeror: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let owner = ensure_signed(origin)?;
			// Check ownership
			ensure!(
				T::NFTHandler::check_ownership(&owner, &asset)?,
				Error::<T>::NoPermissionToAcceptOffer
			);
			ensure!(
				T::NFTHandler::is_transferable(&asset)?,
				Error::<T>::NoPermissionToAcceptOffer
			);
			let offer = Self::nft_offers(asset.clone(), offeror.clone()).ok_or(Error::<T>::OfferDoesNotExist)?;
			ensure!(
				offer.end_block >= <frame_system::Pallet<T>>::block_number(),
				Error::<T>::OfferIsExpired
			);

			T::Currency::unreserve(&offeror, offer.amount);
			<T as Config>::Currency::transfer(&offeror, &owner, offer.amount, ExistenceRequirement::KeepAlive)?;
			T::NFTHandler::transfer_nft(&owner, &offeror, &asset)?;
			Offers::<T>::remove(asset, offeror.clone());

			Self::deposit_event(Event::<T>::NftOfferAccepted(asset.0, asset.1, offeror));
			Ok(().into())
		}

		/// Withdraw offer for an NFT asset
		///
		/// The dispatch origin for this call must be _Signed_.
		/// Only account which have already made an offer for the given NFT can make this call.
		/// - `asset`: the NFT for which te offer will be withdrawn
		///
		/// Emits `NftOfferWithdrawn` if successful.
		#[pallet::weight(T::WeightInfo::withdraw_offer())]
		#[transactional]
		pub fn withdraw_offer(origin: OriginFor<T>, asset: (ClassId, TokenId)) -> DispatchResultWithPostInfo {
			let offeror = ensure_signed(origin)?;
			let offer = Self::nft_offers(asset.clone(), offeror.clone()).ok_or(Error::<T>::OfferDoesNotExist)?;

			T::Currency::unreserve(&offeror, offer.amount);
			Offers::<T>::remove(asset, offeror.clone());

			Self::deposit_event(Event::<T>::NftOfferWithdrawn(asset.0, asset.1, offeror));
			Ok(().into())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		/// Hooks that call every new block finalized.
		fn on_initialize(now: T::BlockNumber) -> Weight {
			let mut total_item = 0;
			for (auction_id, _) in <AuctionEndTime<T>>::drain_prefix(&now) {
				total_item += 1;
				if let Some(auction) = <Auctions<T>>::get(&auction_id) {
					T::Handler::on_auction_ended(auction_id, auction.bid);
				};
			}

			T::WeightInfo::on_finalize().saturating_mul(total_item)
		}

		//		fn on_runtime_upgrade() -> Weight {
		//			Self::upgrade_auction_item_data_v2();
		//			0
		//		}
	}

	impl<T: Config> Auction<T::AccountId, T::BlockNumber> for Pallet<T> {
		type Balance = BalanceOf<T>;

		/// Internal update auction extension
		fn update_auction(
			id: AuctionId,
			info: AuctionInfo<T::AccountId, Self::Balance, T::BlockNumber>,
		) -> DispatchResult {
			let auction = <Auctions<T>>::get(id).ok_or(Error::<T>::AuctionDoesNotExist)?;
			if let Some(old_end) = auction.end {
				<AuctionEndTime<T>>::remove(&old_end, id);
			}
			if let Some(new_end) = info.end {
				<AuctionEndTime<T>>::insert(&new_end, id, ());
			}
			<Auctions<T>>::insert(id, info);
			Ok(())
		}

		/// Internal create new auction item struct extension. This function will be executed inside
		/// create_auction
		fn new_auction(
			_recipient: T::AccountId,
			_initial_amount: Self::Balance,
			start: T::BlockNumber,
			end: Option<T::BlockNumber>,
		) -> Result<AuctionId, DispatchError> {
			let auction: AuctionInfo<T::AccountId, Self::Balance, T::BlockNumber> =
				AuctionInfo { bid: None, start, end };

			let auction_id: AuctionId = AuctionsIndex::<T>::try_mutate(|n| -> Result<AuctionId, DispatchError> {
				let id = *n;
				ensure!(id != AuctionId::max_value(), Error::<T>::NoAvailableAuctionId);
				*n = n.checked_add(One::one()).ok_or(Error::<T>::NoAvailableAuctionId)?;
				Ok(id)
			})?;

			<Auctions<T>>::insert(auction_id, auction);

			if let Some(end_block) = end {
				<AuctionEndTime<T>>::insert(&end_block, auction_id, ());
			}

			Ok(auction_id)
		}

		/// Internal create auction extension
		fn create_auction(
			auction_type: AuctionType,
			item_id: ItemId<Self::Balance>,
			_end: Option<T::BlockNumber>,
			recipient: T::AccountId,
			initial_amount: Self::Balance,
			_start: T::BlockNumber,
			listing_level: ListingLevel<T::AccountId>,
			listing_fee: Perbill,
			currency_id: FungibleTokenId,
		) -> Result<AuctionId, DispatchError> {
			ensure!(
				initial_amount.clone() >= T::MinimumListingPrice::get(),
				Error::<T>::ListingPriceIsBelowMinimum
			);

			ensure!(
				Self::items_in_auction(item_id.clone()) == None,
				Error::<T>::ItemAlreadyInAuction
			);

			let start_time = <system::Pallet<T>>::block_number();

			let mut end_time = start_time + T::AuctionTimeToClose::get();
			if let Some(_end_block) = _end {
				end_time = _end_block
			}

			match item_id.clone() {
				ItemId::NFT(class_id, token_id) => {
					// Check ownership
					let is_owner = T::NFTHandler::check_ownership(&recipient, &(class_id, token_id))?;
					ensure!(is_owner, Error::<T>::NoPermissionToCreateAuction);

					let is_transferable = T::NFTHandler::is_transferable(&(class_id, token_id))?;
					ensure!(is_transferable, Error::<T>::NoPermissionToCreateAuction);

					// Ensure NFT authorised to sell
					if let ListingLevel::Local(metaverse_id) = listing_level {
						ensure!(
							MetaverseCollection::<T>::contains_key(metaverse_id, class_id)
								|| T::MetaverseInfoSource::check_ownership(&recipient, &metaverse_id)
								|| T::MetaverseInfoSource::check_if_metaverse_estate(metaverse_id, &class_id)?,
							Error::<T>::NoPermissionToCreateAuction
						);
					}

					// Ensure auction end time below limit
					ensure!(
						Self::check_valid_finality(&end_time, One::one()),
						Error::<T>::ExceedFinalityLimit
					);

					
					// Reserve network deposit fee
					<T as Config>::Currency::reserve(&recipient, T::NetworkFeeReserve::get())?;

					T::NFTHandler::set_lock_nft((class_id, token_id), true)?;
					let auction_id = Self::new_auction(recipient.clone(), initial_amount, start_time, Some(end_time))?;
					//let mut currency_id: FungibleTokenId = FungibleTokenId::NativeToken(0);

					let new_auction_item = AuctionItem {
						item_id: item_id.clone(),
						recipient: recipient.clone(),
						initial_amount: initial_amount,
						amount: initial_amount,
						start_time,
						end_time,
						auction_type,
						listing_level: listing_level.clone(),
						currency_id,
						listing_fee,
					};

					<AuctionItems<T>>::insert(auction_id, new_auction_item);

					Self::deposit_event(Event::NewAuctionItem(
						auction_id,
						recipient,
						listing_level,
						initial_amount,
						initial_amount,
						end_time,
					));
					<ItemsInAuction<T>>::insert(item_id, true);
					Ok(auction_id)
				}
				ItemId::Spot(_spot_id, _metaverse_id) => {
					// Ensure auction end time below limit
					ensure!(
						Self::check_valid_finality(&end_time, One::one()),
						Error::<T>::ExceedFinalityLimit
					);

					let auction_id = Self::new_auction(recipient.clone(), initial_amount, start_time, Some(end_time))?;

					// Reserve network deposit fee
					<T as Config>::Currency::reserve(&recipient, T::NetworkFeeReserve::get())?;

					let new_auction_item = AuctionItem {
						item_id: item_id.clone(),
						recipient: recipient.clone(),
						initial_amount,
						amount: initial_amount,
						start_time,
						end_time,
						auction_type,
						listing_level: listing_level.clone(),
						currency_id: FungibleTokenId::NativeToken(0),
						listing_fee,
					};

					<AuctionItems<T>>::insert(auction_id, new_auction_item);

					Self::deposit_event(Event::NewAuctionItem(
						auction_id,
						recipient,
						listing_level,
						initial_amount,
						initial_amount,
						end_time,
					));
					<ItemsInAuction<T>>::insert(item_id, true);
					Ok(auction_id)
				}
				ItemId::Bundle(tokens) => {
					ensure!(
						(tokens.len() as u32) < T::MaxBundleItem::get(),
						Error::<T>::ExceedBundleLimit
					);

					// Make sure total item bundle is not exceed max finality
					ensure!(
						Self::check_valid_finality(&end_time, tokens.len() as u32),
						Error::<T>::ExceedFinalityLimit
					);

					for item in tokens {
						// Check ownership
						let is_owner = T::NFTHandler::check_ownership(&recipient, &(item.0, item.1))?;
						ensure!(is_owner, Error::<T>::NoPermissionToCreateAuction);

						let is_transferable = T::NFTHandler::is_transferable(&(item.0, item.1))?;
						ensure!(is_transferable, Error::<T>::NoPermissionToCreateAuction);

						ensure!(
							Self::items_in_auction(ItemId::NFT(item.0, item.1)) == None,
							Error::<T>::ItemAlreadyInAuction
						);
						// Lock NFT
						T::NFTHandler::set_lock_nft((item.0, item.1), true)?
					}

					let auction_id = Self::new_auction(recipient.clone(), initial_amount, start_time, Some(end_time))?;
					//let mut currency_id: FungibleTokenId = FungibleTokenId::NativeToken(0);

					// Reserve network deposit fee
					<T as Config>::Currency::reserve(&recipient, T::NetworkFeeReserve::get())?;

					let new_auction_item = AuctionItem {
						item_id: item_id.clone(),
						recipient: recipient.clone(),
						initial_amount,
						amount: initial_amount,
						start_time,
						end_time,
						auction_type,
						listing_level: listing_level.clone(),
						currency_id,
						listing_fee,
					};

					<AuctionItems<T>>::insert(auction_id, new_auction_item);

					Self::deposit_event(Event::NewAuctionItem(
						auction_id,
						recipient,
						listing_level,
						initial_amount,
						initial_amount,
						end_time,
					));
					<ItemsInAuction<T>>::insert(item_id, true);
					Ok(auction_id)
				}
				ItemId::UndeployedLandBlock(undeployed_land_block_id) => {
					// Ensure the undeployed land block exist and can be used in auction
					ensure!(
						T::EstateHandler::check_undeployed_land_block(&recipient, undeployed_land_block_id)?,
						Error::<T>::UndeployedLandBlockDoesNotExistOrNotAvailable
					);

					// Ensure auction end time below limit
					ensure!(
						Self::check_valid_finality(&end_time, One::one()),
						Error::<T>::ExceedFinalityLimit
					);

					let auction_id = Self::new_auction(recipient.clone(), initial_amount, start_time, Some(end_time))?;

					// Reserve network deposit fee
					<T as Config>::Currency::reserve(&recipient, T::NetworkFeeReserve::get())?;

					let new_auction_item = AuctionItem {
						item_id: item_id.clone(),
						recipient: recipient.clone(),
						initial_amount,
						amount: initial_amount,
						start_time,
						end_time,
						auction_type,
						listing_level: ListingLevel::Global,
						currency_id: FungibleTokenId::NativeToken(0),
						listing_fee,
					};

					<AuctionItems<T>>::insert(auction_id, new_auction_item);

					Self::deposit_event(Event::NewAuctionItem(
						auction_id,
						recipient,
						listing_level,
						initial_amount,
						initial_amount,
						end_time,
					));
					<ItemsInAuction<T>>::insert(item_id, true);
					Ok(auction_id)
				}
				_ => Err(Error::<T>::AuctionTypeIsNotSupported.into()),
			}
		}

		/// Internal remove auction extension
		fn remove_auction(id: AuctionId, item_id: ItemId<Self::Balance>) {
			if let Some(auction) = <Auctions<T>>::get(&id) {
				if let Some(end_block) = auction.end {
					<AuctionEndTime<T>>::remove(end_block, id);
					<Auctions<T>>::remove(&id);
					<ItemsInAuction<T>>::remove(item_id);
					<AuctionItems<T>>::remove(&id);
				}
			}
		}

		/// Internal auction bid handler
		fn auction_bid_handler(from: T::AccountId, id: AuctionId, value: Self::Balance) -> DispatchResult {
			let auction_item: AuctionItem<T::AccountId, T::BlockNumber, BalanceOf<T>> =
				Self::get_auction_item(id.clone()).ok_or(Error::<T>::AuctionDoesNotExist)?;
			ensure!(
				auction_item.auction_type == AuctionType::Auction,
				Error::<T>::InvalidAuctionType
			);
			ensure!(auction_item.recipient != from, Error::<T>::CannotBidOnOwnAuction);

			<Auctions<T>>::try_mutate_exists(id, |auction| -> DispatchResult {
				let mut auction = auction.as_mut().ok_or(Error::<T>::AuctionDoesNotExist)?;

				let block_number = <system::Pallet<T>>::block_number();

				// make sure auction is started
				ensure!(block_number >= auction.start, Error::<T>::AuctionHasNotStarted);

				let auction_end: Option<T::BlockNumber> = auction.end;

				ensure!(block_number < auction_end.unwrap(), Error::<T>::AuctionIsExpired);

				if let Some(ref current_bid) = auction.bid {
					ensure!(value > current_bid.1, Error::<T>::InvalidBidPrice);
				} else {
					ensure!(!value.is_zero(), Error::<T>::InvalidBidPrice);
				}
				// implement hooks for future event
				let bid_result = T::Handler::on_new_bid(block_number, id, (from.clone(), value), auction.bid.clone());

				ensure!(bid_result.accept_bid, Error::<T>::BidIsNotAccepted);

				if auction_item.currency_id == FungibleTokenId::NativeToken(0) {
					ensure!(
						<T as Config>::Currency::free_balance(&from) >= value,
						Error::<T>::InsufficientFreeBalance
					);
				}
				else {
					ensure!(
						T::FungibleTokenCurrency::free_balance(auction_item.currency_id.clone(), &from) >= value.saturated_into(),
						Error::<T>::InsufficientFreeBalance
					);
				}
					

				Self::swap_new_bid(id, (from.clone(), value), auction.bid.clone())?;

				auction.bid = Some((from.clone(), value));
				Self::deposit_event(Event::Bid(id, from, value));

				Ok(())
			})?;

			Ok(())
		}

		/// Internal auction bid handler for local marketplace
		fn local_auction_bid_handler(
			_now: T::BlockNumber,
			id: AuctionId,
			new_bid: (T::AccountId, Self::Balance),
			last_bid: Option<(T::AccountId, Self::Balance)>,
			social_currency_id: FungibleTokenId,
		) -> DispatchResult {
			let (new_bidder, new_bid_price) = new_bid;
			ensure!(!new_bid_price.is_zero(), Error::<T>::InvalidBidPrice);

			<AuctionItems<T>>::try_mutate_exists(id, |auction_item| -> DispatchResult {
				let mut auction_item = auction_item.as_mut().ok_or(Error::<T>::AuctionDoesNotExist)?;

				let last_bid_price = last_bid.clone().map_or(Zero::zero(), |(_, price)| price); // get last bid price
				let last_bidder = last_bid.as_ref().map(|(who, _)| who);

				if let Some(last_bidder) = last_bidder {
					// unlock reserve amount
					if !last_bid_price.is_zero() {
						// Un-reserve balance of last bidder
						T::FungibleTokenCurrency::unreserve(
							social_currency_id,
							&last_bidder,
							last_bid_price.saturated_into(),
						);
					}
				}

				// Lock fund of new bidder
				// Reserve balance
				T::FungibleTokenCurrency::reserve(social_currency_id, &new_bidder, new_bid_price.saturated_into())?;
				auction_item.amount = new_bid_price.clone();

				Ok(())
			})
		}

		/// Internal get auction info
		fn auction_info(id: AuctionId) -> Option<AuctionInfo<T::AccountId, Self::Balance, T::BlockNumber>> {
			Self::auctions(id)
		}

		/// Internal get auction info
		fn auction_item(id: AuctionId) -> Option<AuctionItem<T::AccountId, T::BlockNumber, Self::Balance>> {
			Self::get_auction_item(id)
		}

		/// Internal update auction item id
		fn update_auction_item(id: AuctionId, item_id: ItemId<Self::Balance>) -> DispatchResult {
			let auction_item = AuctionItems::<T>::get(id).ok_or(Error::<T>::AuctionDoesNotExist)?;
			ensure!(
				item_id.is_map_spot() && auction_item.item_id.is_map_spot(),
				Error::<T>::AuctionTypeIsNotSupported
			);

			let spot_detail = item_id
				.get_map_spot_detail()
				.ok_or(Error::<T>::AuctionTypeIsNotSupported)?;
			let old_spot_detail = auction_item
				.item_id
				.get_map_spot_detail()
				.ok_or(Error::<T>::AuctionTypeIsNotSupported)?;

			AuctionItems::<T>::try_mutate_exists(&id, |maybe_auction_item| -> DispatchResult {
				let auction_item_record = maybe_auction_item.as_mut().ok_or(Error::<T>::AuctionDoesNotExist)?;

				auction_item_record.item_id = ItemId::Spot(*old_spot_detail.0, *spot_detail.1);

				Ok(())
			});

			Ok(())
		}

		/// Collect royalty fee for auction
		fn collect_royalty_fee(
			high_bid_price: &Self::Balance,
			high_bidder: &T::AccountId,
			asset_id: &(ClassId, TokenId),
			social_currency_id: FungibleTokenId,
		) -> DispatchResult {
			// Get royalty fee
			let nft_details = T::NFTHandler::get_nft_detail((asset_id.0, asset_id.1))?;
			let royalty_fee: Self::Balance = nft_details.royalty_fee * *high_bid_price;
			let class_fund = T::NFTHandler::get_class_fund(&asset_id.0);

			// Transfer loyalty fee from winner to class fund pot
			Self::fee_transfer_handler(&high_bidder, &class_fund, social_currency_id, royalty_fee)?;

			Ok(())
		}

		/// Internal buy now handler
		fn buy_now_handler(from: T::AccountId, auction_id: AuctionId, value: Self::Balance) -> DispatchResult {
			let auction = Self::auctions(auction_id.clone()).ok_or(Error::<T>::AuctionDoesNotExist)?;
			let auction_item = Self::get_auction_item(auction_id.clone()).ok_or(Error::<T>::AuctionDoesNotExist)?;

			ensure!(
				auction_item.auction_type == AuctionType::BuyNow,
				Error::<T>::InvalidAuctionType
			);

			ensure!(auction_item.recipient != from, Error::<T>::CannotBidOnOwnAuction);

			let block_number = <system::Pallet<T>>::block_number();
			ensure!(block_number >= auction.start, Error::<T>::AuctionHasNotStarted);
			if !(auction.end.is_none()) {
				let auction_end: T::BlockNumber = auction.end.unwrap();
				ensure!(block_number < auction_end, Error::<T>::AuctionIsExpired);
			}

			ensure!(value == auction_item.amount, Error::<T>::InvalidBuyNowPrice);
			if auction_item.currency_id == FungibleTokenId::NativeToken(0) {
				ensure!(
					<T as Config>::Currency::free_balance(&from) >= value,
					Error::<T>::InsufficientFreeBalance
				);
			}
			else  {
				ensure!(
					T::FungibleTokenCurrency::free_balance(auction_item.currency_id.clone(), &from) >= value.saturated_into(),
					Error::<T>::InsufficientFreeBalance
				);
			}

			Self::remove_auction(auction_id.clone(), auction_item.item_id.clone());

			// Unreserve network deposit fee
			<T as Config>::Currency::unreserve(&auction_item.recipient, T::NetworkFeeReserve::get());

			// Transfer balance from buy it now user to asset owner
			let mut currency_transfer;
			if auction_item.currency_id == FungibleTokenId::NativeToken(0) {
				currency_transfer = <T as Config>::Currency::transfer(
					&from,
					&auction_item.recipient,
					value,
					ExistenceRequirement::KeepAlive,
				);
			}
			else {
				currency_transfer = T::FungibleTokenCurrency::transfer(
					auction_item.currency_id,
					&from,
					&auction_item.recipient,
					value.saturated_into()
				);
			}

			match currency_transfer {
				Err(_e) => {}
				Ok(_v) => {
					// Transfer asset from asset owner to buy it now user
					<ItemsInAuction<T>>::remove(auction_item.item_id.clone());

					// Collect network commission fee
					Self::collect_network_fee(&value, &auction_item.recipient, auction_item.currency_id);

					match auction_item.item_id {
						ItemId::NFT(class_id, token_id) => {
							Self::collect_listing_fee(
								&value,
								&auction_item.recipient,
								auction_item.currency_id,
								auction_item.listing_level.clone(),
								auction_item.listing_fee.clone(),
							)?;

							Self::collect_royalty_fee(
								&value,
								&auction_item.recipient,
								&(class_id, token_id),
								auction_item.currency_id,
							)?;

							T::NFTHandler::set_lock_nft((class_id, token_id), false);

							let asset_transfer =
								T::NFTHandler::transfer_nft(&auction_item.recipient, &from, &(class_id, token_id));

							match asset_transfer {
								Err(_) => (),
								Ok(_) => {
									Self::deposit_event(Event::BuyNowFinalised(auction_id, from, value));
								}
							}
						}
						ItemId::Spot(spot_id, metaverse_id) => {
							let continuum_spot = T::ContinuumHandler::transfer_spot(
								spot_id,
								auction_item.recipient.clone(),
								(from.clone(), metaverse_id),
							);
							match continuum_spot {
								Err(_) => (),
								Ok(_) => {
									Self::deposit_event(Event::BuyNowFinalised(auction_id, from, value));
								}
							}
						}
						ItemId::Bundle(tokens) => {
							// Collect listing fee once
							Self::collect_listing_fee(
								&value,
								&auction_item.recipient,
								auction_item.currency_id,
								auction_item.listing_level.clone(),
								auction_item.listing_fee,
							)?;

							for token in tokens {
								// Collect royalty fee of each nft sold in the bundle
								Self::collect_royalty_fee(
									&token.2,
									&auction_item.recipient,
									&(token.0, token.1),
									auction_item.currency_id,
								)?;
								T::NFTHandler::set_lock_nft((token.0, token.1), false)?;
								T::NFTHandler::transfer_nft(&auction_item.recipient, &from, &(token.0, token.1))?;
							}

							Self::deposit_event(Event::BuyNowFinalised(auction_id, from, value));
						}
						ItemId::UndeployedLandBlock(undeployed_land_block_id) => {
							let undeployed_land_block = T::EstateHandler::transfer_undeployed_land_block(
								&auction_item.recipient,
								&from.clone(),
								undeployed_land_block_id,
							);

							match undeployed_land_block {
								Err(_) => (),
								Ok(_) => {
									Self::deposit_event(Event::BuyNowFinalised(auction_id, from, value));
								}
							}
						}
						_ => {} // Future implementation for other items
					}
				}
			}
			Ok(())
		}
	}

	impl<T: Config> CheckAuctionItemHandler<BalanceOf<T>> for Pallet<T> {
		fn check_item_in_auction(item_id: ItemId<BalanceOf<T>>) -> bool {
			Self::items_in_auction(item_id) == Some(true)
		}
	}

	impl<T: Config> AuctionHandler<T::AccountId, BalanceOf<T>, T::BlockNumber, AuctionId> for Pallet<T> {
		fn on_new_bid(
			_now: T::BlockNumber,
			_id: AuctionId,
			_new_bid: (T::AccountId, BalanceOf<T>),
			_last_bid: Option<(T::AccountId, BalanceOf<T>)>,
		) -> OnNewBidResult<T::BlockNumber> {
			OnNewBidResult {
				accept_bid: true,
				auction_end_change: Change::NoChange,
			}
		}

		fn on_auction_ended(auction_id: AuctionId, winner: Option<(T::AccountId, BalanceOf<T>)>) {
			if let Some(auction_item) = <AuctionItems<T>>::get(&auction_id) {
				Self::remove_auction(auction_id.clone(), auction_item.item_id.clone());

				// Unreserve network deposit fee
				<T as Config>::Currency::unreserve(&auction_item.recipient, T::NetworkFeeReserve::get());
				// Transfer balance from high bidder to asset owner
				if let Some(current_bid) = winner {
					let (high_bidder, high_bid_price): (T::AccountId, BalanceOf<T>) = current_bid;

					// Handle listing
					if auction_item.currency_id == FungibleTokenId::NativeToken(0) {
						<T as Config>::Currency::unreserve(&high_bidder, high_bid_price);
					}
					else {
						T::FungibleTokenCurrency::unreserve(auction_item.currency_id, &high_bidder, high_bid_price.saturated_into());
					}

					// Handle balance transfer
					let mut currency_transfer;
					if auction_item.currency_id == FungibleTokenId::NativeToken(0) {
						currency_transfer = <T as Config>::Currency::transfer(
							&high_bidder,
							&auction_item.recipient,
							high_bid_price,
							ExistenceRequirement::KeepAlive,
						);
					}
					else {
						currency_transfer = T::FungibleTokenCurrency::transfer(
							auction_item.currency_id,
							&high_bidder,
							&auction_item.recipient,
							high_bid_price.saturated_into()
						);
					}

					

					if let Ok(_transfer_succeeded) = currency_transfer {
						// Collect network commission fee
						Self::collect_network_fee(
							&high_bid_price,
							&auction_item.recipient,
							auction_item.currency_id,
						);

						// Transfer asset from asset owner to high bidder
						// Check asset type and handle internal logic
						match auction_item.item_id.clone() {
							ItemId::NFT(class_id, token_id) => {
								Self::collect_listing_fee(
									&high_bid_price,
									&auction_item.recipient,
									auction_item.currency_id,
									auction_item.listing_level.clone(),
									auction_item.listing_fee,
								);

								Self::collect_royalty_fee(
									&high_bid_price,
									&auction_item.recipient,
									&(class_id, token_id),
									auction_item.currency_id,
								);

								T::NFTHandler::set_lock_nft((class_id, token_id), false);
								let asset_transfer = T::NFTHandler::transfer_nft(
									&auction_item.recipient,
									&high_bidder,
									&(class_id, token_id),
								);
								if let Ok(_transferred) = asset_transfer {
									Self::deposit_event(Event::AuctionFinalized(
										auction_id,
										high_bidder,
										high_bid_price,
									));
								}
							}
							ItemId::Spot(spot_id, metaverse_id) => {
								let continuum_spot = T::ContinuumHandler::transfer_spot(
									spot_id,
									auction_item.recipient.clone(),
									(high_bidder.clone(), metaverse_id),
								);

								if let Ok(_continuum_spot) = continuum_spot {
									Self::deposit_event(Event::AuctionFinalized(
										auction_id,
										high_bidder,
										high_bid_price,
									));
								}
							}
							ItemId::Bundle(tokens) => {
								// Collect listing fee once
								Self::collect_listing_fee(
									&high_bid_price,
									&auction_item.recipient,
									auction_item.currency_id,
									auction_item.listing_level.clone(),
									auction_item.listing_fee,
								);

								for token in tokens {
									// Collect royalty fee of each nft sold in the bundle
									Self::collect_royalty_fee(
										&token.2,
										&auction_item.recipient,
										&(token.0, token.1),
										auction_item.currency_id,
									);
									T::NFTHandler::set_lock_nft((token.0, token.1), false);
									T::NFTHandler::transfer_nft(
										&auction_item.recipient,
										&high_bidder,
										&(token.0, token.1),
									);
								}

								Self::deposit_event(Event::AuctionFinalized(
									auction_id,
									high_bidder.clone(),
									high_bid_price,
								));
							}
							ItemId::UndeployedLandBlock(undeployed_land_block_id) => {
								let undeployed_land_block = T::EstateHandler::transfer_undeployed_land_block(
									&auction_item.recipient,
									&high_bidder.clone(),
									undeployed_land_block_id,
								);

								if let Ok(_) = undeployed_land_block {
									Self::deposit_event(Event::AuctionFinalized(
										auction_id,
										high_bidder,
										high_bid_price,
									));
								}
							}
							_ => {} // Future implementation for Metaverse
						}
						<ItemsInAuction<T>>::remove(auction_item.item_id.clone());
						<AuctionItems<T>>::remove(auction_id.clone());
					}
				} else {
					if let ItemId::NFT(class_id, token_id) = auction_item.item_id.clone() {
						T::NFTHandler::set_lock_nft((class_id, token_id), false);
					}

					if let ItemId::Bundle(tokens) = auction_item.item_id.clone() {
						for token in tokens {
							T::NFTHandler::set_lock_nft((token.0, token.1), false);
						}
					}

					Self::deposit_event(Event::AuctionFinalizedNoBid(auction_id));
				}
			}
		}
	}

	impl<T: Config> Pallet<T> {
		fn check_valid_finality(end: &T::BlockNumber, quantity: u32) -> bool {
			let existing_auctions_same_block: u32 = <AuctionEndTime<T>>::iter_prefix_values(end).count() as u32;
			let total_auction_in_same_block = existing_auctions_same_block.saturating_add(quantity);

			T::MaxFinality::get() >= total_auction_in_same_block
		}

		/// Collect listing fee for auction
		fn collect_listing_fee(
			high_bid_price: &BalanceOf<T>,
			high_bidder: &T::AccountId,
			social_currency_id: FungibleTokenId,
			listing_level: ListingLevel<T::AccountId>,
			listing_fee: Perbill,
		) -> DispatchResult {
			if let ListingLevel::Local(metaverse_id) = listing_level {
				let metaverse_fund = T::MetaverseInfoSource::get_metaverse_treasury(metaverse_id);
				let listing_fee_amount = listing_fee * *high_bid_price;

				Self::fee_transfer_handler(&high_bidder, &metaverse_fund, social_currency_id, listing_fee_amount)?;
			}
			Ok(())
		}

		/// Collect network fee for auction
		fn collect_network_fee(
			high_bid_price: &BalanceOf<T>,
			recipient: &T::AccountId,
			social_currency_id: FungibleTokenId,
		) -> DispatchResult {
			let network_fund = T::MetaverseInfoSource::get_network_treasury();
			let network_fee: BalanceOf<T> = T::NetworkFeeCommission::get() * *high_bid_price;

			Self::fee_transfer_handler(&recipient, &network_fund, social_currency_id, network_fee)?;

			Ok(())
		}

		/// Handle fee transfer from one account to another
		fn fee_transfer_handler(
			from: &T::AccountId,
			to: &T::AccountId,
			social_currency_id: FungibleTokenId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			if social_currency_id == FungibleTokenId::NativeToken(0) {
				// Check if account free_balance + network fee less than ED
				let amount_plus_free_balance = T::Currency::free_balance(to).saturating_add(amount);
				// Only transfer fee if amount plus balance greater than ED, never fail
				if amount_plus_free_balance >= T::Currency::minimum_balance() {
					<T as Config>::Currency::transfer(from, to, amount, ExistenceRequirement::KeepAlive)?;
				}
			} else {
				// Check if account free_balance + network fee less than ED
				let amount_plus_free_balance = T::FungibleTokenCurrency::free_balance(social_currency_id.clone(), to)
					.saturating_add(amount.saturated_into());
				// Only transfer fee if amount plus balance greater than ED, never fail
				if amount_plus_free_balance >= T::FungibleTokenCurrency::minimum_balance(social_currency_id.clone()) {
					T::FungibleTokenCurrency::transfer(social_currency_id.clone(), from, to, amount.saturated_into())?;
				}
			}
			Ok(())
		}

		pub fn upgrade_auction_item_data_v2() -> Weight {
			log::info!("Start upgrading auction item data v2");
			let mut num_auction_items = 0;

			AuctionItems::<T>::translate(
				|_k, auction_v1: AuctionItemV1<T::AccountId, T::BlockNumber, BalanceOf<T>>| {
					num_auction_items += 1;
					let v2: AuctionItem<T::AccountId, T::BlockNumber, BalanceOf<T>> = AuctionItem {
						item_id: auction_v1.item_id,
						recipient: auction_v1.recipient,
						initial_amount: auction_v1.initial_amount,
						amount: auction_v1.amount,
						start_time: auction_v1.start_time,
						end_time: auction_v1.end_time,
						auction_type: auction_v1.auction_type,
						listing_level: auction_v1.listing_level,
						currency_id: auction_v1.currency_id,
						listing_fee: Perbill::from_percent(0u32),
					};
					Some(v2)
				},
			);

			log::info!("{} auction items upgraded:", num_auction_items);
			0
		}

		// Runtime upgrade V1 - may required for production release
		//		pub fn upgrade_asset_auction_data_v2() -> Weight {
		//			log::info!("Start upgrading nft class data v2");
		//			let mut num_auction_item = 0;
		//
		//			AuctionItems::<T>::translate(
		//				|_k, auction_v1: migration_v2::AuctionItem<T::AccountId, T::BlockNumber, BalanceOf<T>>| {
		//					num_auction_item += 1;
		//
		//					log::info!("Upgrading auction items data");
		//
		//					let asset_id = auction_v1.item_id;
		//
		//					match asset_id {
		//						V1ItemId::NFT(asset_id) => {
		//							num_auction_item += 1;
		//							let token = T::NFTHandler::get_asset_id(asset_id).unwrap();
		//							let v2_item_id = ItemId::NFT(token.0, token.1);
		//
		//							let v: AuctionItem<T::AccountId, T::BlockNumber, BalanceOf<T>> = AuctionItem {
		//								item_id: v2_item_id,
		//								recipient: auction_v1.recipient,
		//								initial_amount: auction_v1.initial_amount,
		//								amount: auction_v1.amount,
		//								start_time: auction_v1.start_time,
		//								end_time: auction_v1.end_time,
		//								auction_type: auction_v1.auction_type,
		//								listing_level: auction_v1.listing_level,
		//								currency_id: auction_v1.currency_id,
		//							};
		//							Some(v)
		//						}
		//						_ => None,
		//					}
		//				},
		//			);
		//
		//			log::info!("Asset Item in Auction upgraded: {}", num_auction_item);
		//			0
		//		}

		pub fn swap_new_bid(
			id: AuctionId,
			new_bid: (T::AccountId, BalanceOf<T>),
			last_bid: Option<(T::AccountId, BalanceOf<T>)>,
		) -> DispatchResult {
			let (new_bidder, new_bid_price) = new_bid;
			ensure!(!new_bid_price.is_zero(), Error::<T>::InvalidBidPrice);

			<AuctionItems<T>>::try_mutate_exists(id, |auction_item| -> DispatchResult {
				let mut auction_item = auction_item.as_mut().ok_or(Error::<T>::AuctionDoesNotExist)?;

				let last_bid_price = last_bid.clone().map_or(Zero::zero(), |(_, price)| price); // get last bid price
				let last_bidder = last_bid.as_ref().map(|(who, _)| who);

				if let Some(last_bidder) = last_bidder {
					//unlock reserve amount
					if !last_bid_price.is_zero() {
						// Unreserve balance of last bidder
						if auction_item.currency_id == FungibleTokenId::NativeToken(0) {
							<T as Config>::Currency::unreserve(&last_bidder, last_bid_price);
						}
						else {
							T::FungibleTokenCurrency::unreserve(auction_item.currency_id, &last_bidder, last_bid_price.saturated_into());
						}
					}
				}

				// Lock fund of new bidder
				// Reserve balance
				if auction_item.currency_id == FungibleTokenId::NativeToken(0) {
					<T as Config>::Currency::reserve(&new_bidder, new_bid_price)?;
				}
				else {
					T::FungibleTokenCurrency::reserve(auction_item.currency_id, &new_bidder, new_bid_price.saturated_into())?;
				}
				// Update new bid price for individual item on bundle
				if let ItemId::Bundle(tokens) = &auction_item.item_id {
					let mut new_bundle: Vec<(ClassId, TokenId, BalanceOf<T>)> = Vec::new();
					let total_amount = auction_item.amount.clone();

					for token in tokens {
						let new_price: BalanceOf<T> = Perbill::from_rational(token.2, total_amount) * new_bid_price;
						new_bundle.push((token.0, token.1, new_price))
					}
					ItemsInAuction::<T>::remove(ItemId::Bundle(tokens.clone()));
					ItemsInAuction::<T>::insert(ItemId::Bundle(new_bundle.clone()), true);
					auction_item.item_id = ItemId::Bundle(new_bundle);
				}

				auction_item.amount = new_bid_price.clone();

				Ok(())
			})
		}
	}
}
