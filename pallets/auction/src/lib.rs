// This pallet use The Open Runtime Module Library (ORML) which is a community maintained collection of Substrate runtime modules.
// Thanks to all contributors of orml.
// https://github.com/open-web3-stack/open-runtime-module-library

#![cfg_attr(not(feature = "std"), no_std)]
// Disable the following two lints since they originate from an external macro (namely decl_storage)
#![allow(clippy::string_lit_as_bytes)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure,
    traits::{ExistenceRequirement, Get, Currency, ReservableCurrency},
    weights::Weight,
    IterableStorageDoubleMap, Parameter,
};

use codec::{Decode, Encode};
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, Bounded, MaybeSerializeDeserialize, Member, One, Zero},
    DispatchError, DispatchResult, RuntimeDebug,
};

use frame_system::{self as system, ensure_signed};
use orml_nft::Module as NftModule;
use primitives::{AccountId, BlockNumber, CollectionId};
use sp_std::result;

mod auction;

pub use crate::auction::{Auction, AuctionHandler, Change, OnNewBidResult};

#[cfg(test)]
mod tests;

pub struct AuctionLogicHandler;

type ClassIdOf<T> = <T as orml_nft::Config>::ClassId;
type AssetIdOf<T> = <T as orml_nft::Config>::TokenId;

#[cfg_attr(feature = "std", derive(PartialEq, Eq))]
#[derive(Encode, Decode, Clone, RuntimeDebug)]
pub struct AuctionItem<AccountId, BlockNumber, Balance, AssetId, ClassId> {
    asset_id: AssetId,
    class_id: ClassId,
    recipient: AccountId,
    initial_amount: Balance,
    /// Current amount for sale
    amount: Balance,
    /// Auction start time
    start_time: BlockNumber,
    end_time: BlockNumber,
}

/// Auction info.
#[cfg_attr(feature = "std", derive(PartialEq, Eq))]
#[derive(Encode, Decode, Clone, RuntimeDebug)]
pub struct AuctionInfo<AccountId, Balance, BlockNumber> {
    /// Current bidder and bid price.
    pub bid: Option<(AccountId, Balance)>,
    /// Define which block this auction will be started.
    pub start: BlockNumber,
    /// Define which block this auction will be ended.
    pub end: Option<BlockNumber>,
}

pub trait Config: frame_system::Config + orml_nft::Config + pallet_balances::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;

    type AuctionTimeToClose: Get<Self::BlockNumber>;

    /// The auction ID type
    type AuctionId: Parameter
    + Member
    + AtLeast32BitUnsigned
    + Default
    + Copy
    + MaybeSerializeDeserialize
    + Bounded;

    /// The `AuctionHandler` that allow custom bidding logic and handles auction
    /// result
    type Handler: AuctionHandler<Self::AccountId, Self::Balance, Self::BlockNumber, Self::AuctionId>;
    type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

    // /// Weight information for extrinsics in this module.
    // type WeightInfo: WeightInfo;
}

decl_storage! {
    trait Store for Module<T: Config> as Auction {
        /// Stores on-going and future auctions. Closed auction are removed.
        pub Auctions get(fn auctions): map hasher(twox_64_concat) T::AuctionId => Option<AuctionInfo<T::AccountId, T::Balance, T::BlockNumber>>;

        //Store asset with Auction
        pub AuctionItems get(fn get_auction_item): map hasher(twox_64_concat) T::AuctionId => Option<AuctionItem<T::AccountId, T::BlockNumber, T::Balance, AssetIdOf<T>, ClassIdOf<T>>>;

        /// Track the next auction ID.
        pub AuctionsIndex get(fn auctions_index): T::AuctionId;

        /// Index auctions by end time.
        pub AuctionEndTime get(fn auction_end_time): double_map hasher(twox_64_concat) T::BlockNumber, hasher(twox_64_concat) T::AuctionId => Option<()>;
    }
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Config>::AccountId,
        <T as pallet_balances::Config>::Balance,
        // AssetId = AssetId,
        <T as Config>::AuctionId,
    {
        /// A bid is placed. [auction_id, bidder, bidding_amount]
        Bid(AuctionId, AccountId, Balance),
        NewAuctionItem(AuctionId, AccountId ,Balance, Balance),
        AuctionFinalized(AuctionId, AccountId, Balance),
    }
);

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;

        /// The extended time for the auction to end after each successful bid
        const AuctionTimeToClose: T::BlockNumber = T::AuctionTimeToClose::get();

        #[weight = 10_000]
        fn bid(origin, id: T::AuctionId, value: T::Balance) {
            let from = ensure_signed(origin)?;

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

                ensure!(<pallet_balances::Module<T>>::free_balance(&from) >= value, "You don't have enough free balance for this bid");

                Self::auction_bid_handler(block_number, id, (from.clone(), value), auction.bid.clone())?;

                auction.bid = Some((from.clone(), value));
                Self::deposit_event(RawEvent::Bid(id, from, value));

                Ok(())
            })?;

        }


        #[weight = 10_000]
        fn create_auction(origin, asset: (ClassIdOf<T>, AssetIdOf<T>), initial_price: T::Balance) {
            let from = ensure_signed(origin)?;

            //Check ownership

            let asset_info = NftModule::<T>::tokens(asset.0, asset.1).ok_or(Error::<T>::AssetInfoNotFound)?;
            ensure!(from == asset_info.owner, Error::<T>::NoPermissionToCreateAuction);

            let start_time = <system::Module<T>>::block_number();

            let end_time: T::BlockNumber = start_time + T::AuctionTimeToClose::get(); // get auction time to close from runtime. Can be dynamic in the new update

            let auction_id = Self::new_auction(from.clone(), initial_price, start_time, Some(end_time))?;

            let new_auction_item = AuctionItem {
                asset_id: asset.1,
                class_id: asset.0 ,
                recipient : from.clone(),
                initial_amount : initial_price,
                amount : initial_price,
                start_time : start_time,
                end_time: end_time
            };

            <AuctionItems<T>>::insert(
                auction_id,
                new_auction_item
            );

            Self::deposit_event(RawEvent::NewAuctionItem(auction_id, from, initial_price ,initial_price));
        }

        /// dummy `on_initialize` to return the weight used in `on_finalize`.
        // fn on_initialize(now: T::BlockNumber) -> Weight {
        // 	T::WeightInfo::on_finalize(<AuctionEndTime<T>>::iter_prefix(&now).count() as u32)
        // }

        fn on_finalize(now: T::BlockNumber) {
            for (auction_id, _) in <AuctionEndTime<T>>::drain_prefix(&now) {
                if let Some(auction) = <Auctions<T>>::take(&auction_id) {
                        if let Some(auction_item) = <AuctionItems<T>>::take(&auction_id){
                            Self::remove_auction(auction_id.clone());
                            //Transfer balance from high bidder to asset owner
                            if let Some(current_bid) = auction.bid{
                                let (high_bidder, high_bid_price): (T::AccountId, T::Balance) = current_bid;
                                <pallet_balances::Module<T>>::unreserve(&high_bidder, high_bid_price);
                                let currency_transfer = <pallet_balances::Module<T> as Currency<_>>::transfer(&high_bidder, &auction_item.recipient , high_bid_price, ExistenceRequirement::KeepAlive);
                                match currency_transfer {
                                    Err(_e) => continue,
                                    Ok(_v) => {
                                        //Transfer asset from asset owner to high bidder
                                        let asset_transfer = NftModule::<T>::transfer(&auction_item.recipient, &high_bidder, (auction_item.class_id, auction_item.asset_id));
                                        match asset_transfer {
                                            Err(_e) => continue,
                                            Ok(_v) => {
                                                Self::deposit_event(RawEvent::AuctionFinalized(auction_id, high_bidder ,high_bid_price));
                                            },
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
        AuctionNotStarted,
        AuctionIsExpired,
        BidNotAccepted,
        InvalidBidPrice,
        NoAvailableAuctionId,
        NoPermissionToCreateAuction,
        AssetInfoNotFound
    }
}

impl<T: Config> Module<T> {
    fn auction_info(
        id: T::AuctionId,
    ) -> Option<AuctionInfo<T::AccountId, T::Balance, T::BlockNumber>> {
        Self::auctions(id)
    }

    fn update_auction(
        id: T::AuctionId,
        info: AuctionInfo<T::AccountId, T::Balance, T::BlockNumber>,
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
        recipient: T::AccountId,
        initial_amount: T::Balance,
        start: T::BlockNumber,
        end: Option<T::BlockNumber>,
    ) -> Result<T::AuctionId, DispatchError> {
        let auction: AuctionInfo<T::AccountId, T::Balance, T::BlockNumber> = AuctionInfo {
            bid: None,
            start,
            end,
        };

        let auction_id: T::AuctionId =
            <AuctionsIndex<T>>::try_mutate(|n| -> Result<T::AuctionId, DispatchError> {
                let id = *n;
                ensure!(
                    id != T::AuctionId::max_value(),
                    Error::<T>::NoAvailableAuctionId
                );
                *n += One::one();
                Ok(id)
            })?;

        <Auctions<T>>::insert(auction_id, auction);

        if let Some(end_block) = end {
            <AuctionEndTime<T>>::insert(&end_block, auction_id, ());
        }

        Ok(auction_id)
    }

    fn remove_auction(id: T::AuctionId) {
        if let Some(auction) = <Auctions<T>>::take(&id) {
            if let Some(end_block) = auction.end {
                <AuctionEndTime<T>>::remove(end_block, id);
                <Auctions<T>>::remove(&id)
            }
        }
    }

    /// increment `new_bidder` reference and decrement `last_bidder` reference
    /// if any
    fn swap_bidders(new_bidder: &T::AccountId, last_bidder: Option<&T::AccountId>) {
        system::Module::<T>::inc_ref(new_bidder);

        if let Some(who) = last_bidder {
            system::Module::<T>::dec_ref(who);
        }
    }

    fn auction_bid_handler(
        now: T::BlockNumber,
        id: T::AuctionId,
        new_bid: (T::AccountId, T::Balance),
        last_bid: Option<(T::AccountId, T::Balance)>,
    ) -> DispatchResult {
        let (new_bidder, new_bid_price) = new_bid;
        ensure!(!new_bid_price.is_zero(), Error::<T>::InvalidBidPrice);

        <AuctionItems<T>>::try_mutate_exists(id, |auction_item| -> DispatchResult {
            let mut auction_item = auction_item.as_mut().ok_or("Auction is not exists")?;

            let last_bid_price = last_bid.clone().map_or(Zero::zero(), |(_, price)| price); //get last bid price
            let last_bidder = last_bid.as_ref().map(|(who, _)| who);

            if let Some(last_bidder) = last_bidder {
                //unlock reserve amount
                if (!last_bid_price.is_zero()) {
                    //Unreserve balance of last bidder
                    <pallet_balances::Module<T>>::unreserve(&last_bidder, last_bid_price);
                }
            }

            //Lock fund of new bidder
            //Reserve balance
            <pallet_balances::Module<T>>::reserve(&new_bidder, new_bid_price)?;
            auction_item.recipient = new_bidder.clone();
            auction_item.amount = new_bid_price.clone();

            Ok(())
        })
    }
}


impl<T: Config> AuctionHandler<T::AccountId, T::Balance, T::BlockNumber, T::AuctionId>
for Module<T>
{
    fn on_new_bid(
        now: T::BlockNumber,
        _id: T::AuctionId,
        new_bid: (T::AccountId, T::Balance),
        _last_bid: Option<(T::AccountId, T::Balance)>,
    ) -> OnNewBidResult<T::BlockNumber> {
        OnNewBidResult {
            accept_bid: true,
            auction_end_change: Change::NoChange,
        }
    }

    fn on_auction_ended(_id: T::AuctionId, _winner: Option<(T::AccountId, T::Balance)>) {}
}