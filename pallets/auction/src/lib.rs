// This pallet use The Open Runtime Module Library (ORML) which is a community maintained collection of Substrate runtime modules.
// Thanks to all contributors of orml.
// Ref: https://github.com/open-web3-stack/open-runtime-module-library

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::string_lit_as_bytes)]

use auction_manager::{Auction, OnNewBidResult, AuctionHandler, Change, AuctionInfo, AuctionItem, AuctionType};
use frame_support::{traits::{Currency, ExistenceRequirement, ReservableCurrency, LockableCurrency}};
use frame_system::{self as system, ensure_signed};
use pallet_continuum::Pallet as ContinuumModule;
use pallet_nft::Module as NFTModule;
use primitives::{ItemId, AuctionId, AssetId, continuum::Continuum};
use sp_runtime::{traits::{One, Zero}, DispatchError, DispatchResult};
pub use pallet::*;
use frame_support::{ensure, pallet_prelude::*, transactional};
use sp_runtime::SaturatedConversion;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod mock;

pub struct AuctionLogicHandler;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::OriginFor;
    use super::*;
    use auction_manager::ListingLevel;
    use primitives::{SocialTokenCurrencyId, Balance, CountryId};
    use orml_traits::{MultiCurrencyExtended, MultiReservableCurrency, MultiCurrency};
    use bc_country::BCCountry;
    use frame_support::sp_runtime::traits::{CheckedSub, CheckedAdd};

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    pub(super) type ClassIdOf<T> = <T as orml_nft::Config>::ClassId;
    pub(super) type TokenIdOf<T> = <T as orml_nft::Config>::TokenId;
    pub(super) type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_nft::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        #[pallet::constant]
        type AuctionTimeToClose: Get<Self::BlockNumber>;
        /// The `AuctionHandler` that allow custom bidding logic and handles auction result
        type Handler: AuctionHandler<Self::AccountId, BalanceOf<Self>, Self::BlockNumber, AuctionId>;
        type Currency: ReservableCurrency<Self::AccountId>
        + LockableCurrency<Self::AccountId, Moment=Self::BlockNumber>;
        type ContinuumHandler: Continuum<Self::AccountId>;
        type SocialTokenCurrency: MultiReservableCurrency<Self::AccountId, CurrencyId=SocialTokenCurrencyId, Balance=Balance>;
        type CountryInfoSource: BCCountry<Self::AccountId>;
        type MinimumAuctionDuration: Get<Self::BlockNumber>;
    }

    #[pallet::storage]
    #[pallet::getter(fn auctions)]
    /// Stores on-going and future auctions. Closed auction are removed.
    pub(super) type Auctions<T: Config> = StorageMap<_, Twox64Concat, AuctionId, AuctionInfo<T::AccountId, BalanceOf<T>, T::BlockNumber>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_auction_item)]
    //Store asset with Auction
    pub(super) type AuctionItems<T: Config> = StorageMap<_, Twox64Concat, AuctionId, AuctionItem<T::AccountId, T::BlockNumber, BalanceOf<T>>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn assets_in_auction)]
    /// Track which Assets are in auction
    pub(super) type AssetsInAuction<T: Config> = StorageMap<_, Twox64Concat, AssetId, bool, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn auctions_index)]
    /// Track the next auction ID.
    pub(super) type AuctionsIndex<T: Config> = StorageValue<_, AuctionId, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn auction_end_time)]
    /// Index auctions by end time.
    pub(super) type AuctionEndTime<T: Config> = StorageDoubleMap<_, Twox64Concat, T::BlockNumber, Twox64Concat, AuctionId, (), OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    #[pallet::metadata()]
    pub enum Event<T: Config> {
        /// A bid is placed. [auction_id, bidder, bidding_amount]
        Bid(AuctionId, T::AccountId, BalanceOf<T>),
        NewAuctionItem(AuctionId, T::AccountId, BalanceOf<T>, BalanceOf<T>),
        AuctionFinalized(AuctionId, T::AccountId, BalanceOf<T>),
        BuyNowFinalised(AuctionId, T::AccountId, BalanceOf<T>),
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Bidding on global listing
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        #[transactional]
        pub(super) fn bid(origin: OriginFor<T>, id: AuctionId, value: BalanceOf<T>) -> DispatchResultWithPostInfo {
            let from = ensure_signed(origin)?;

            let auction_item: AuctionItem<T::AccountId, T::BlockNumber, BalanceOf<T>> = Self::get_auction_item(id.clone()).ok_or(Error::<T>::AuctionNotExist)?;
            ensure!(auction_item.auction_type == AuctionType::Auction, Error::<T>::InvalidAuctionType);
            ensure!(auction_item.recipient != from, Error::<T>::SelfBidNotAccepted);
            ensure!(auction_item.listing_level == ListingLevel::Global, Error::<T>::WrongListingLevel);

            <Auctions<T>>::try_mutate_exists(id, |auction| -> DispatchResult {
                let mut auction = auction.as_mut().ok_or(Error::<T>::AuctionNotExist)?;

                let block_number = <frame_system::Module<T>>::block_number();

                // make sure auction is started
                ensure!(block_number >= auction.start, Error::<T>::AuctionNotStarted);

                let auction_end: Option<T::BlockNumber> = auction.end;

                ensure!(block_number < auction_end.unwrap(), Error::<T>::AuctionIsExpired);

                if let Some(ref current_bid) = auction.bid {
                    ensure!(value > current_bid.1, Error::<T>::InvalidBidPrice);
                } else {
                    ensure!(!value.is_zero(), Error::<T>::InvalidBidPrice);
                }
                let bid_result = T::Handler::on_new_bid(
                    block_number,
                    id,
                    (from.clone(), value),
                    auction.bid.clone(),
                );

                ensure!(bid_result.accept_bid, Error::<T>::BidNotAccepted);

                ensure!(<T as Config>::Currency::free_balance(&from) >= value, "You don't have enough free balance for this bid");

                Self::auction_bid_handler(block_number, id, (from.clone(), value), auction.bid.clone())?;

                auction.bid = Some((from.clone(), value));
                Self::deposit_event(Event::Bid(id, from, value));

                Ok(())
            })?;

            Ok(().into())
        }

        /// Bidding on local marketplace listing
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        #[transactional]
        pub(super) fn bid_local(origin: OriginFor<T>, id: AuctionId, bc_id: CountryId, value: BalanceOf<T>) -> DispatchResultWithPostInfo {
            let from = ensure_signed(origin)?;

            let auction_item: AuctionItem<T::AccountId, T::BlockNumber, BalanceOf<T>> = Self::get_auction_item(id.clone()).ok_or(Error::<T>::AuctionNotExist)?;
            ensure!(auction_item.auction_type == AuctionType::Auction, Error::<T>::InvalidAuctionType);
            ensure!(auction_item.recipient != from, Error::<T>::SelfBidNotAccepted);
            ensure!(auction_item.listing_level == ListingLevel::Local(bc_id), Error::<T>::WrongListingLevel);

            let social_currency_id = auction_item.currency_id;

            <Auctions<T>>::try_mutate_exists(id, |auction| -> DispatchResult {
                let mut auction = auction.as_mut().ok_or(Error::<T>::AuctionNotExist)?;

                let block_number = <frame_system::Module<T>>::block_number();

                // make sure auction is started
                ensure!(block_number >= auction.start, Error::<T>::AuctionNotStarted);

                let auction_end: Option<T::BlockNumber> = auction.end;

                ensure!(block_number < auction_end.unwrap(), Error::<T>::AuctionIsExpired);

                if let Some(ref current_bid) = auction.bid {
                    ensure!(value > current_bid.1, Error::<T>::InvalidBidPrice);
                } else {
                    ensure!(!value.is_zero(), Error::<T>::InvalidBidPrice);
                }
                let bid_result = T::Handler::on_new_bid(
                    block_number,
                    id,
                    (from.clone(), value),
                    auction.bid.clone(),
                );

                ensure!(bid_result.accept_bid, Error::<T>::BidNotAccepted);

                ensure!(T::SocialTokenCurrency::free_balance(social_currency_id, &from) >= value.saturated_into(), "You don't have enough free balance for this bid");

                Self::auction_bid_handler(block_number, id, (from.clone(), value), auction.bid.clone())?;

                auction.bid = Some((from.clone(), value));
                Self::deposit_event(Event::Bid(id, from, value));

                Ok(())
            })?;

            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub(super) fn buy_now(origin: OriginFor<T>, auction_id: AuctionId, value: BalanceOf<T>) -> DispatchResultWithPostInfo {
            let from = ensure_signed(origin)?;

            let auction = Self::auctions(auction_id.clone()).ok_or(Error::<T>::AuctionNotExist)?;
            let auction_item = Self::get_auction_item(auction_id.clone()).ok_or(Error::<T>::AuctionNotExist)?;
            ensure!(auction_item.auction_type == AuctionType::BuyNow, Error::<T>::InvalidAuctionType);

            ensure!(auction_item.recipient != from, Error::<T>::CannotBidOnOwnAuction);

            let block_number = <frame_system::Module<T>>::block_number();
            ensure!(block_number >= auction.start, Error::<T>::AuctionNotStarted);
            if !(auction.end.is_none()) {
                let auction_end: T::BlockNumber = auction.end.unwrap();
                ensure!(block_number < auction_end, Error::<T>::AuctionIsExpired);
            }

            ensure!(value == auction_item.amount, Error::<T>::InvalidBuyItNowPrice);
            ensure!(<T as Config>::Currency::free_balance(&from) >= value, Error::<T>::InsufficientFunds);

            Self::remove_auction(auction_id.clone(), auction_item.item_id);
            //Transfer balance from buy it now user to asset owner
            let currency_transfer = <T as Config>::Currency::transfer(&from, &auction_item.recipient, value, ExistenceRequirement::KeepAlive);
            match currency_transfer {
                Err(_e) => {}
                Ok(_v) => {
                    //Transfer asset from asset owner to buy it now user
                    match auction_item.item_id {
                        ItemId::NFT(asset_id) => {
                            let asset_transfer = NFTModule::<T>::do_transfer(&auction_item.recipient, &from, asset_id);
                            <AssetsInAuction<T>>::remove(asset_id);
                            match asset_transfer {
                                Err(_) => (),
                                Ok(_) => {
                                    Self::deposit_event(Event::BuyNowFinalised(auction_id, from, value));
                                }
                            }
                        }
                        ItemId::Spot(spot_id, country_id) => {
                            let continuum_spot = T::ContinuumHandler::transfer_spot(spot_id, &auction_item.recipient, &(from.clone(), country_id));
                            match continuum_spot {
                                Err(_) => (),
                                Ok(_) => {
                                    Self::deposit_event(Event::BuyNowFinalised(auction_id, from, value));
                                }
                            }
                        }
                        _ => {} //Future implementation for Spot, Country
                    }
                }
            }
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub(super) fn buy_now_local(origin: OriginFor<T>, auction_id: AuctionId, bc_id: CountryId, value: BalanceOf<T>) -> DispatchResultWithPostInfo {
            let from = ensure_signed(origin)?;

            let auction = Self::auctions(auction_id.clone()).ok_or(Error::<T>::AuctionNotExist)?;
            let auction_item = Self::get_auction_item(auction_id.clone()).ok_or(Error::<T>::AuctionNotExist)?;
            ensure!(auction_item.auction_type == AuctionType::BuyNow, Error::<T>::InvalidAuctionType);
            ensure!(auction_item.recipient != from, Error::<T>::CannotBidOnOwnAuction);
            ensure!(auction_item.listing_level == ListingLevel::Local(bc_id), Error::<T>::WrongListingLevel);

            let block_number = <frame_system::Module<T>>::block_number();
            ensure!(block_number >= auction.start, Error::<T>::AuctionNotStarted);
            if !(auction.end.is_none()) {
                let auction_end: T::BlockNumber = auction.end.unwrap();
                ensure!(block_number < auction_end, Error::<T>::AuctionIsExpired);
            }

            let social_currency_id = auction_item.currency_id;

            ensure!(value == auction_item.amount, Error::<T>::InvalidBuyItNowPrice);
            ensure!(T::SocialTokenCurrency::free_balance(social_currency_id, &from) >= value.saturated_into(), Error::<T>::InsufficientFunds);

            Self::remove_auction(auction_id.clone(), auction_item.item_id);
            //Transfer balance from buy it now user to asset owner

            let social_currency_transfer = T::SocialTokenCurrency::transfer(social_currency_id, &from, &auction_item.recipient, value.saturated_into());
            match social_currency_transfer {
                Err(_e) => {}
                Ok(_v) => {
                    //Transfer asset from asset owner to buy it now user
                    match auction_item.item_id {
                        ItemId::NFT(asset_id) => {
                            let asset_transfer = NFTModule::<T>::do_transfer(&auction_item.recipient, &from, asset_id);
                            <AssetsInAuction<T>>::remove(asset_id);
                            match asset_transfer {
                                Err(_) => (),
                                Ok(_) => {
                                    Self::deposit_event(Event::BuyNowFinalised(auction_id, from, value));
                                }
                            }
                        }
                        ItemId::Spot(spot_id, country_id) => {
                            let continuum_spot = T::ContinuumHandler::transfer_spot(spot_id, &auction_item.recipient, &(from.clone(), country_id));
                            match continuum_spot {
                                Err(_) => (),
                                Ok(_) => {
                                    Self::deposit_event(Event::BuyNowFinalised(auction_id, from, value));
                                }
                            }
                        }
                        _ => {} //Future implementation for Spot, Country
                    }
                }
            }
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub(super) fn create_new_auction(origin: OriginFor<T>, item_id: ItemId, value: BalanceOf<T>, end_time: T::BlockNumber, listing_level: ListingLevel) -> DispatchResultWithPostInfo {
            let from = ensure_signed(origin)?;

            let start_time: T::BlockNumber = <system::Module<T>>::block_number();

            let remaining_time: T::BlockNumber = end_time.checked_sub(&start_time).ok_or("Overflow")?;

            ensure!(remaining_time >= T::MinimumAuctionDuration::get(),
            Error::<T>::AuctionEndIsLessThanMinimumDuration);

            let auction_id = Self::create_auction(AuctionType::Auction, item_id, Some(end_time), from.clone(), value.clone(), start_time, listing_level)?;
            Self::deposit_event(Event::NewAuctionItem(auction_id, from, value, value));

            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub(super) fn create_new_buy_now(origin: OriginFor<T>, item_id: ItemId, value: BalanceOf<T>, end_time: T::BlockNumber, listing_level: ListingLevel) -> DispatchResultWithPostInfo {
            let from = ensure_signed(origin)?;

            let start_time: T::BlockNumber = <system::Module<T>>::block_number();
            let remaining_time: T::BlockNumber = end_time.checked_sub(&start_time).ok_or("Overflow")?;

            ensure!(remaining_time >= T::MinimumAuctionDuration::get(),
            Error::<T>::AuctionEndIsLessThanMinimumDuration);

            let auction_id = Self::create_auction(AuctionType::BuyNow, item_id, Some(end_time), from.clone(), value.clone(), start_time, listing_level)?;
            Self::deposit_event(Event::NewAuctionItem(auction_id, from, value, value));

            Ok(().into())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        /// dummy `on_initialize` to return the weight used in `on_finalize`.
        // fn on_initialize(now: T::BlockNumber) -> Weight {
        // 	T::WeightInfo::on_finalize(<AuctionEndTime<T>>::iter_prefix(&now).count() as u32)
        // }

        fn on_finalize(now: T::BlockNumber) {
            for (auction_id, _) in <AuctionEndTime<T>>::drain_prefix(&now) {
                if let Some(auction) = <Auctions<T>>::get(&auction_id) {
                    if let Some(auction_item) = <AuctionItems<T>>::get(&auction_id) {
                        Self::remove_auction(auction_id.clone(), auction_item.item_id);
                        //Transfer balance from high bidder to asset owner
                        if let Some(current_bid) = auction.bid {
                            let (high_bidder, high_bid_price): (T::AccountId, BalanceOf<T>) = current_bid;
                            //Handle global listing
                            if auction_item.listing_level == ListingLevel::Global {
                                <T as Config>::Currency::unreserve(&high_bidder, high_bid_price);
                                let currency_transfer = <T as Config>::Currency::transfer(&high_bidder, &auction_item.recipient, high_bid_price, ExistenceRequirement::KeepAlive);
                                match currency_transfer {
                                    Err(_e) => continue,
                                    Ok(_v) => {
                                        //Transfer asset from asset owner to high bidder
                                        //Check asset type and handle internal logic

                                        match auction_item.item_id {
                                            ItemId::NFT(asset_id) => {
                                                let asset_transfer = NFTModule::<T>::do_transfer(&auction_item.recipient, &high_bidder, asset_id);
                                                <AssetsInAuction<T>>::remove(asset_id);
                                                match asset_transfer {
                                                    Err(_) => continue,
                                                    Ok(_) => {
                                                        Self::deposit_event(Event::AuctionFinalized(auction_id, high_bidder, high_bid_price));
                                                    }
                                                }
                                            }
                                            ItemId::Spot(spot_id, country_id) => {
                                                let continuum_spot = T::ContinuumHandler::transfer_spot(spot_id, &auction_item.recipient, &(high_bidder.clone(), country_id));
                                                match continuum_spot {
                                                    Err(_) => continue,
                                                    Ok(_) => {
                                                        Self::deposit_event(Event::AuctionFinalized(auction_id, high_bidder, high_bid_price));
                                                    }
                                                }
                                            }
                                            _ => {} //Future implementation for Spot, Country
                                        }
                                    }
                                }
                            } else { // Handle local bit country social token transfer
                                let social_currency_id = auction_item.currency_id.clone();
                                T::SocialTokenCurrency::unreserve(social_currency_id, &high_bidder, high_bid_price.saturated_into());
                                let social_currency_transfer = T::SocialTokenCurrency::transfer(social_currency_id, &high_bidder, &auction_item.recipient, high_bid_price.saturated_into());
                                match social_currency_transfer {
                                    Err(_e) => continue,
                                    Ok(_v) => {
                                        //Transfer asset from asset owner to high bidder
                                        //Check asset type and handle internal logic

                                        match auction_item.item_id {
                                            ItemId::NFT(asset_id) => {
                                                let asset_transfer = NFTModule::<T>::do_transfer(&auction_item.recipient, &high_bidder, asset_id);
                                                <AssetsInAuction<T>>::remove(asset_id);
                                                match asset_transfer {
                                                    Err(_) => continue,
                                                    Ok(_) => {
                                                        Self::deposit_event(Event::AuctionFinalized(auction_id, high_bidder, high_bid_price));
                                                    }
                                                }
                                            }
                                            ItemId::Spot(spot_id, country_id) => {
                                                let continuum_spot = T::ContinuumHandler::transfer_spot(spot_id, &auction_item.recipient, &(high_bidder.clone(), country_id));
                                                match continuum_spot {
                                                    Err(_) => continue,
                                                    Ok(_) => {
                                                        Self::deposit_event(Event::AuctionFinalized(auction_id, high_bidder, high_bid_price));
                                                    }
                                                }
                                            }
                                            _ => {} //Future implementation for Spot, Country
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        AuctionNotExist,
        AssetIsNotExist,
        AuctionNotStarted,
        AuctionIsExpired,
        AuctionTypeIsNotSupported,
        BidNotAccepted,
        InsufficientFreeBalance,
        InvalidBidPrice,
        NoAvailableAuctionId,
        NoPermissionToCreateAuction,
        SelfBidNotAccepted,
        CannotBidOnOwnAuction,
        InvalidBuyItNowPrice,
        InsufficientFunds,
        // Invalid auction type
        InvalidAuctionType,
        // Asset already in Auction
        AssetAlreadyInAuction,
        // Wrong Listing Level
        WrongListingLevel,
        // Social Token Currency is not exist
        SocialTokenCurrencyNotFound,
        // Minimum Duration Is Too Low
        AuctionEndIsLessThanMinimumDuration,
    }

    impl<T: Config> Auction<T::AccountId, T::BlockNumber> for Pallet<T> {
        type Balance = BalanceOf<T>;

        fn update_auction(
            id: AuctionId,
            info: AuctionInfo<T::AccountId, Self::Balance, T::BlockNumber>,
        ) -> DispatchResult {
            let auction = <Auctions<T>>::get(id).ok_or(Error::<T>::AuctionNotExist)?;
            if let Some(old_end) = auction.end {
                <AuctionEndTime<T>>::remove(&old_end, id);
            }
            if let Some(new_end) = info.end {
                <AuctionEndTime<T>>::insert(&new_end, id, ());
            }
            <Auctions<T>>::insert(id, info);
            Ok(())
        }

        fn new_auction(
            _recipient: T::AccountId,
            _initial_amount: Self::Balance,
            start: T::BlockNumber,
            end: Option<T::BlockNumber>,
        ) -> Result<AuctionId, DispatchError> {
            let auction: AuctionInfo<T::AccountId, Self::Balance, T::BlockNumber> = AuctionInfo {
                bid: None,
                start,
                end,
            };

            let auction_id: AuctionId =
                AuctionsIndex::<T>::try_mutate(|n| -> Result<AuctionId, DispatchError> {
                    let id = *n;
                    ensure!(
                        id != AuctionId::max_value(),
                        Error::<T>::NoAvailableAuctionId
                    );
                    *n = n
                        .checked_add(One::one())
                        .ok_or(Error::<T>::NoAvailableAuctionId)?;
                    Ok(id)
                })?;

            <Auctions<T>>::insert(auction_id, auction);

            if let Some(end_block) = end {
                <AuctionEndTime<T>>::insert(&end_block, auction_id, ());
            }

            Ok(auction_id)
        }

        fn create_auction(
            auction_type: AuctionType,
            item_id: ItemId,
            _end: Option<T::BlockNumber>,
            recipient: T::AccountId,
            initial_amount: Self::Balance,
            _start: T::BlockNumber,
            listing_level: ListingLevel,
        ) -> Result<AuctionId, DispatchError> {
            match item_id {
                ItemId::NFT(asset_id) => {
                    //FIXME - Remove in prod - For debugging purpose
                    debug::info!("Asset id {}", asset_id);
                    //Get asset detail
                    let asset = NFTModule::<T>::get_asset(asset_id).ok_or(Error::<T>::AssetIsNotExist)?;
                    //Check ownership
                    let class_info = orml_nft::Pallet::<T>::classes(asset.0).ok_or(Error::<T>::NoPermissionToCreateAuction)?;
                    let class_info_data = class_info.data;
                    let token_info = orml_nft::Pallet::<T>::tokens(asset.0, asset.1).ok_or(Error::<T>::NoPermissionToCreateAuction)?;
                    ensure!(recipient == token_info.owner, Error::<T>::NoPermissionToCreateAuction);
                    ensure!(class_info_data.token_type.is_transferable(), Error::<T>::NoPermissionToCreateAuction);
                    ensure!(Self::assets_in_auction(asset_id) == None, Error::<T>::AssetAlreadyInAuction);

                    let start_time = <system::Module<T>>::block_number();
                    let end_time: T::BlockNumber = start_time + T::AuctionTimeToClose::get(); //add 7 days block for default auction
                    let auction_id = Self::new_auction(recipient.clone(), initial_amount, start_time, Some(end_time))?;
                    let mut currency_id: SocialTokenCurrencyId = SocialTokenCurrencyId::NativeToken(0);
                    if let ListingLevel::Local(bc_id) = listing_level {
                        currency_id = T::CountryInfoSource::get_country_token(bc_id).ok_or(Error::<T>::SocialTokenCurrencyNotFound)?;
                    }

                    let new_auction_item = AuctionItem {
                        item_id,
                        recipient: recipient.clone(),
                        initial_amount: initial_amount,
                        amount: initial_amount,
                        start_time,
                        end_time,
                        auction_type,
                        listing_level,
                        currency_id,
                    };

                    <AuctionItems<T>>::insert(
                        auction_id,
                        new_auction_item,
                    );

                    <AssetsInAuction<T>>::insert(
                        asset_id,
                        true,
                    );

                    Self::deposit_event(Event::NewAuctionItem(auction_id, recipient, initial_amount, initial_amount));

                    Ok(auction_id)
                }
                ItemId::Spot(_spot_id, _country_id) => {
                    //TODO Check if spot_id is not owned by any
                    let start_time = <system::Module<T>>::block_number();
                    let end_time: T::BlockNumber = start_time + T::AuctionTimeToClose::get(); //add 7 days block for default auction
                    let auction_id = Self::new_auction(recipient.clone(), initial_amount, start_time, Some(end_time))?;

                    let new_auction_item = AuctionItem {
                        item_id,
                        recipient: recipient.clone(),
                        initial_amount: initial_amount,
                        amount: initial_amount,
                        start_time,
                        end_time,
                        auction_type,
                        listing_level: ListingLevel::Global,
                        currency_id: SocialTokenCurrencyId::NativeToken(0),
                    };

                    <AuctionItems<T>>::insert(
                        auction_id,
                        new_auction_item,
                    );

                    Self::deposit_event(Event::NewAuctionItem(auction_id, recipient, initial_amount, initial_amount));

                    Ok(auction_id)
                }
                _ => Err(Error::<T>::AuctionTypeIsNotSupported.into())
            }
        }

        fn remove_auction(id: AuctionId, item_id: ItemId) {
            if let Some(auction) = <Auctions<T>>::get(&id) {
                if let Some(end_block) = auction.end {
                    <AuctionEndTime<T>>::remove(end_block, id);
                    <Auctions<T>>::remove(&id);
                    match item_id {
                        ItemId::NFT(asset_id) => {
                            <AssetsInAuction<T>>::remove(asset_id);
                        }
                        _ => {}
                    }
                }
            }
        }

        fn auction_bid_handler(
            _now: T::BlockNumber,
            id: AuctionId,
            new_bid: (T::AccountId, Self::Balance),
            last_bid: Option<(T::AccountId, Self::Balance)>,
        ) -> DispatchResult {
            let (new_bidder, new_bid_price) = new_bid;
            ensure!(!new_bid_price.is_zero(), Error::<T>::InvalidBidPrice);

            <AuctionItems<T>>::try_mutate_exists(id, |auction_item| -> DispatchResult {
                let mut auction_item = auction_item.as_mut().ok_or("Auction is not exists")?;

                let last_bid_price = last_bid.clone().map_or(Zero::zero(), |(_, price)| price); //get last bid price
                let last_bidder = last_bid.as_ref().map(|(who, _)| who);

                if let Some(last_bidder) = last_bidder {
                    //unlock reserve amount
                    if !last_bid_price.is_zero() {
                        //Unreserve balance of last bidder
                        <T as Config>::Currency::unreserve(&last_bidder, last_bid_price);
                    }
                }

                //Lock fund of new bidder
                //Reserve balance
                <T as Config>::Currency::reserve(&new_bidder, new_bid_price)?;
                auction_item.amount = new_bid_price.clone();

                Ok(())
            })
        }

        fn local_auction_bid_handler(
            _now: T::BlockNumber,
            id: AuctionId,
            new_bid: (T::AccountId, Self::Balance),
            last_bid: Option<(T::AccountId, Self::Balance)>,
            social_currency_id: SocialTokenCurrencyId,
        ) -> DispatchResult {
            let (new_bidder, new_bid_price) = new_bid;
            ensure!(!new_bid_price.is_zero(), Error::<T>::InvalidBidPrice);

            <AuctionItems<T>>::try_mutate_exists(id, |auction_item| -> DispatchResult {
                let mut auction_item = auction_item.as_mut().ok_or("Auction is not exists")?;

                let last_bid_price = last_bid.clone().map_or(Zero::zero(), |(_, price)| price); //get last bid price
                let last_bidder = last_bid.as_ref().map(|(who, _)| who);

                if let Some(last_bidder) = last_bidder {
                    //unlock reserve amount
                    if !last_bid_price.is_zero() {
                        //Un-reserve balance of last bidder
                        T::SocialTokenCurrency::unreserve(social_currency_id, &last_bidder, last_bid_price.saturated_into());
                    }
                }

                //Lock fund of new bidder
                //Reserve balance
                T::SocialTokenCurrency::reserve(social_currency_id, &new_bidder, new_bid_price.saturated_into())?;
                auction_item.amount = new_bid_price.clone();

                Ok(())
            })
        }

        fn auction_info(id: AuctionId) -> Option<AuctionInfo<T::AccountId, Self::Balance, T::BlockNumber>> {
            Self::auctions(id)
        }

        fn check_item_in_auction(asset_id: AssetId) -> bool {
            if Self::assets_in_auction(asset_id) == Some(true) {
                return true;
            }
            return false;
        }
    }

    impl<T: Config> AuctionHandler<T::AccountId, BalanceOf<T>, T::BlockNumber, AuctionId>
    for Module<T>
    {
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

        fn on_auction_ended(_id: AuctionId, _winner: Option<(T::AccountId, BalanceOf<T>)>) {}
    }
}



