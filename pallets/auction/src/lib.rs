// This pallet use The Open Runtime Module Library (ORML) which is a community maintained collection of Substrate runtime modules.
// Thanks to all contributors of orml.
// Ref: https://github.com/open-web3-stack/open-runtime-module-library

#![cfg_attr(not(feature = "std"), no_std)]
// Disable the following two lints since they originate from an external macro (namely decl_storage)
#![allow(clippy::string_lit_as_bytes)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure,
    traits::{Currency, ExistenceRequirement, Get, ReservableCurrency, LockableCurrency},
    IterableStorageDoubleMap, Parameter,
    debug,
};

use codec::{Decode, Encode};
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, Bounded, MaybeSerializeDeserialize, Member, One, Zero},
    DispatchError, DispatchResult, RuntimeDebug,
};

use frame_system::{self as system, ensure_signed};
use pallet_nft::Module as NFTModule;
use pallet_continuum::Pallet as ContinuumModule;

use primitives::{ItemId, AuctionId, continuum::Continuum};

use auction_manager::{Auction, OnNewBidResult, AuctionHandler, Change, AuctionInfo, AuctionItem};
use frame_support::sp_runtime::traits::AtLeast32Bit;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

pub struct AuctionLogicHandler;

type ClassIdOf<T> = <T as orml_nft::Config>::ClassId;
type TokenIdOf<T> = <T as orml_nft::Config>::TokenId;
type BalanceOf<T> =
<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub trait Config:
frame_system::Config
+ pallet_nft::Config
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type AuctionTimeToClose: Get<Self::BlockNumber>;
    /// The `AuctionHandler` that allow custom bidding logic and handles auction result
    type Handler: AuctionHandler<Self::AccountId, BalanceOf<Self>, Self::BlockNumber, AuctionId>;
    type Currency: ReservableCurrency<Self::AccountId>
    + LockableCurrency<Self::AccountId, Moment=Self::BlockNumber>;
    type ContinuumHandler: Continuum<Self::AccountId>;

    // /// Weight information for extrinsics in this module.
    // type WeightInfo: WeightInfo;
}

decl_storage! {
    trait Store for Module<T: Config> as Auction {
        /// Stores on-going and future auctions. Closed auction are removed.
        pub Auctions get(fn auctions): map hasher(twox_64_concat) AuctionId => Option<AuctionInfo<T::AccountId, BalanceOf<T>, T::BlockNumber>>;

        //Store asset with Auction
        pub AuctionItems get(fn get_auction_item): map hasher(twox_64_concat) AuctionId => Option<AuctionItem<T::AccountId, T::BlockNumber, BalanceOf<T>>>;

        /// Track the next auction ID.
        pub AuctionsIndex get(fn auctions_index): AuctionId;

        /// Index auctions by end time.
        pub AuctionEndTime get(fn auction_end_time): double_map hasher(twox_64_concat) T::BlockNumber, hasher(twox_64_concat) AuctionId => Option<()>;
    }
}
decl_event!(
    pub enum Event<T> where
        <T as frame_system::Config>::AccountId,
        Balance = BalanceOf<T>,
        // AssetId = AssetId,
    {
        /// A bid is placed. [auction_id, bidder, bidding_amount]
        Bid(AuctionId, AccountId, Balance),
        NewAuctionItem(AuctionId, AccountId ,Balance, Balance),
        AuctionFinalized(AuctionId, AccountId, Balance),
        BuyItNowFinalised(AuctionId, AccountId, Balance),
    }
);

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;

        /// The extended time for the auction to end after each successful bid
        const AuctionTimeToClose: T::BlockNumber = T::AuctionTimeToClose::get();

        #[weight = 10_000]
        fn bid(origin, id: AuctionId, value: BalanceOf<T>) {
            let from = ensure_signed(origin)?;

            let auction_item = Self::get_auction_item(id.clone()).ok_or(Error::<T>::AuctionNotExist)?;           
            ensure!(auction_item.recipient != from, Error::<T>::SelfBidNotAccepted);

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
                Self::deposit_event(RawEvent::Bid(id, from, value));

                Ok(())
            })?;

        }

        #[weight = 10_000]
        fn buy_it_now(origin, auction_id: AuctionId, value: BalanceOf<T>) {
            let from = ensure_signed(origin)?;

            let auction = Self::auctions(auction_id.clone()).ok_or(Error::<T>::AuctionNotExist)?;  
            let auction_item = Self::get_auction_item(auction_id.clone()).ok_or(Error::<T>::AuctionNotExist)?;  
            
            ensure!(auction_item.recipient != from, Error::<T>::CannotBidOnOwnAuction);

            let block_number = <frame_system::Module<T>>::block_number();
            ensure!(block_number >= auction.start, Error::<T>::AuctionNotStarted);
            if !(auction.end.is_none()) {
                let auction_end: T::BlockNumber = auction.end.unwrap();
                ensure!(block_number < auction_end, Error::<T>::AuctionIsExpired);
            }

            ensure!(auction_item.buy_it_now_amount.is_none() == false, Error::<T>::BuyItNowNotAvailable);
            let buy_it_now_amount = auction_item.buy_it_now_amount.unwrap();
            ensure!(value == buy_it_now_amount, Error::<T>::InvalidBuyItNowPrice);
            ensure!(<T as Config>::Currency::free_balance(&from) >= value, Error::<T>::InsufficientFunds);

            Self::remove_auction(auction_id.clone());
            //Unreserve balance of last bidder
             if let Some(current_bid) = auction.bid{
                let (high_bidder, high_bid_price): (T::AccountId, BalanceOf<T>) = current_bid;
                <T as Config>::Currency::unreserve(&high_bidder, high_bid_price);
            }
            //Transfer balance from buy it now user to asset owner
            let currency_transfer = <T as Config>::Currency::transfer(&from, &auction_item.recipient, value, ExistenceRequirement::KeepAlive);
            match currency_transfer {
                Err(_e) => (),
                Ok(_v) => {
                    //Transfer asset from asset owner to buy it now user
                    match auction_item.item_id {
                        ItemId::NFT(asset_id) => {                                           
                            let asset_transfer = NFTModule::<T>::do_transfer(&auction_item.recipient, &from, asset_id);
                                match asset_transfer {
                                    Err(_) => (),
                                    Ok(_) => {
                                        Self::deposit_event(RawEvent::BuyItNowFinalised(auction_id, from, value));
                                    },
                                }
                        }
                        ItemId::Spot(spot_id, country_id) => {
                            let continuum_spot = T::ContinuumHandler::transfer_spot(spot_id, &auction_item.recipient, &(from.clone(), country_id));
                            match continuum_spot{
                                    Err(_) => (),
                                    Ok(_) => {
                                        Self::deposit_event(RawEvent::BuyItNowFinalised(auction_id, from, value));
                                    },
                            }
                        }
                        _ => {} //Future implementation for Spot, Country
                    }
                },
            }
        }

        #[weight = 10_000]
        fn create_new_auction(origin, item_id: ItemId, value: BalanceOf<T>, buy_it_now_value: Option<BalanceOf<T>>) {
            let from = ensure_signed(origin)?;

            let start_time: T::BlockNumber = <system::Module<T>>::block_number();
            let end_time: T::BlockNumber = start_time + T::AuctionTimeToClose::get(); //add 7 days block for default auction

            let auction_id = Self::create_auction(item_id, Some(end_time), from.clone(), value.clone(), buy_it_now_value.clone(), start_time)?;
            Self::deposit_event(RawEvent::NewAuctionItem(auction_id, from, value ,value));
        }

        /// dummy `on_initialize` to return the weight used in `on_finalize`.
        // fn on_initialize(now: T::BlockNumber) -> Weight {
        // 	T::WeightInfo::on_finalize(<AuctionEndTime<T>>::iter_prefix(&now).count() as u32)
        // }

        fn on_finalize(now: T::BlockNumber) {
            for (auction_id, _) in <AuctionEndTime<T>>::drain_prefix(&now) {
                if let Some(auction) = <Auctions<T>>::get(&auction_id) {
                        if let Some(auction_item) = <AuctionItems<T>>::get(&auction_id){
                            Self::remove_auction(auction_id.clone());
                            //Transfer balance from high bidder to asset owner
                            if let Some(current_bid) = auction.bid{
                                let (high_bidder, high_bid_price): (T::AccountId, BalanceOf<T>) = current_bid;
                                <T as Config>::Currency::unreserve(&high_bidder, high_bid_price);
                                let currency_transfer = <T as Config>::Currency::transfer(&high_bidder, &auction_item.recipient , high_bid_price, ExistenceRequirement::KeepAlive);
                                match currency_transfer {
                                    Err(_e) => continue,
                                    Ok(_v) => {
                                        //Transfer asset from asset owner to high bidder
                                        //Check asset type and handle internal logic

                                         match auction_item.item_id {
                                            ItemId::NFT(asset_id) => {                                           
                                                let asset_transfer = NFTModule::<T>::do_transfer(&auction_item.recipient, &high_bidder, asset_id);
                                                   match asset_transfer {
                                                        Err(_) => continue,
                                                        Ok(_) => {
                                                            Self::deposit_event(RawEvent::AuctionFinalized(auction_id, high_bidder ,high_bid_price));
                                                        },
                                                    }
                                            }
                                            ItemId::Spot(spot_id, country_id) => {
                                                let continuum_spot = T::ContinuumHandler::transfer_spot(spot_id, &auction_item.recipient, &(high_bidder.clone(), country_id));
                                                match continuum_spot{
                                                     Err(_) => continue,
                                                     Ok(_) => {
                                                            Self::deposit_event(RawEvent::AuctionFinalized(auction_id, high_bidder ,high_bid_price));
                                                     },
                                                }
                                            }
                                            _ => {} //Future implementation for Spot, Country
                                        }
                                    },
                                }
                            }
                        }
                }
            }
        }
    }
}

decl_error! {
    /// Error for auction module.
    pub enum Error for Module<T: Config> {
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
        BuyItNowNotAvailable,
    }
}

impl<T: Config> Auction<T::AccountId, T::BlockNumber> for Module<T> {
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
            AuctionsIndex::try_mutate(|n| -> Result<AuctionId, DispatchError> {
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
        item_id: ItemId,
        end: Option<T::BlockNumber>,
        recipient: T::AccountId,
        initial_amount: Self::Balance,
        buy_it_now_amount: Option<Self::Balance>,
        start: T::BlockNumber,
    ) -> Result<AuctionId, DispatchError> {
        match item_id {
            ItemId::NFT(asset_id) => {
                //FIXME - Remove in prod - For debugging purpose
                debug::info!("Asset id {}", asset_id);
                //Get asset detail
                let asset = NFTModule::<T>::get_asset(asset_id).ok_or(Error::<T>::AssetIsNotExist)?;
                //Check ownership
                let class_info = orml_nft::Pallet::<T>::classes(asset.0).ok_or(Error::<T>::NoPermissionToCreateAuction)?;
                ensure!(recipient == class_info.owner, Error::<T>::NoPermissionToCreateAuction);
                let class_info_data = class_info.data;
                ensure!(class_info_data.token_type.is_transferrable(), Error::<T>::NoPermissionToCreateAuction);

                let start_time = <system::Module<T>>::block_number();
                let end_time: T::BlockNumber = start_time + T::AuctionTimeToClose::get(); //add 7 days block for default auction
                let auction_id = Self::new_auction(recipient.clone(), initial_amount, start_time, Some(end_time))?;

                let new_auction_item = AuctionItem {
                    item_id,
                    recipient: recipient.clone(),
                    initial_amount: initial_amount,
                    amount: initial_amount,
                    buy_it_now_amount: buy_it_now_amount,
                    start_time,
                    end_time,
                };

                <AuctionItems<T>>::insert(
                    auction_id,
                    new_auction_item,
                );

                Self::deposit_event(RawEvent::NewAuctionItem(auction_id, recipient, initial_amount, initial_amount));

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
                    buy_it_now_amount: buy_it_now_amount,
                    start_time,
                    end_time,
                };

                <AuctionItems<T>>::insert(
                    auction_id,
                    new_auction_item,
                );

                Self::deposit_event(RawEvent::NewAuctionItem(auction_id, recipient, initial_amount, initial_amount));

                Ok(auction_id)
            }
            _ => Err(Error::<T>::AuctionTypeIsNotSupported.into())
        }
    }

    fn remove_auction(id: AuctionId) {
        if let Some(auction) = <Auctions<T>>::get(&id) {
            if let Some(end_block) = auction.end {
                <AuctionEndTime<T>>::remove(end_block, id);
                <Auctions<T>>::remove(&id)
            }
        }
    }

    /// increment `new_bidder` reference and decrement `last_bidder` reference
    /// if any
    fn swap_bidders(new_bidder: &T::AccountId, last_bidder: Option<&T::AccountId>) {
        system::Module::<T>::inc_consumers(new_bidder);

        if let Some(who) = last_bidder {
            system::Module::<T>::dec_consumers(who);
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

    fn auction_info(id: AuctionId) -> Option<AuctionInfo<T::AccountId, Self::Balance, T::BlockNumber>> {
        Self::auctions(id)
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
