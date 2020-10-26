#![cfg_attr(not(feature = "std"), no_std)]
// Disable the following two lints since they originate from an external macro (namely decl_storage)
#![allow(clippy::string_lit_as_bytes)]

use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, ensure, weights::Weight, IterableStorageDoubleMap, Parameter,
	traits::Get
};
use codec::{Decode, Encode};
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, Bounded, MaybeSerializeDeserialize, Member, One, Zero},
	DispatchError, DispatchResult, RuntimeDebug
};

use frame_system::{self as system, ensure_signed};
use sp_std::result;
use unique_asset::{AssetByOwner, AssetId};

mod auction;

pub use crate::auction::{Auction, AuctionHandler, AuctionInfo, Change, OnNewBidResult};

#[cfg(test)]
mod tests;

pub struct AuctionLogicHandler;

// pub type Balance = u128;
pub type AccountId = u128;
pub type BlockNumber = u32;

#[cfg_attr(feature = "std", derive(PartialEq, Eq))]
#[derive(Encode, Decode, Clone, RuntimeDebug)]
pub struct AuctionItem<AccountId, BlockNumber, Balance> {
	asset_id: AssetId,
	recipient: AccountId,
	initial_amount: Balance,
	/// Current amount for sale
	amount: Balance,
	/// Auction start time
	start_time: BlockNumber,
}

pub trait Trait: frame_system::Trait + unique_asset::Trait{
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	/// The extended time for the auction to end after each successful bid
	type AuctionTimeToClose: Get<Self::BlockNumber>;

	/// The balance type for bidding
	type Balance: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize;

	/// The auction ID type
	type AuctionId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + Bounded;

	/// The `AuctionHandler` that allow custom bidding logic and handles auction
	/// result
	type Handler: AuctionHandler<Self::AccountId, Self::Balance, Self::BlockNumber, Self::AuctionId>;

	// /// Weight information for extrinsics in this module.
	// type WeightInfo: WeightInfo;
}

decl_storage! {
	trait Store for Module<T: Trait> as Auction {
		/// Stores on-going and future auctions. Closed auction are removed.
		pub Auctions get(fn auctions): map hasher(twox_64_concat) T::AuctionId => Option<AuctionInfo<T::AccountId, T::Balance, T::BlockNumber>>;

		//Store asset with Auction
		pub AuctionItems get(fn get_auction_item): map hasher(twox_64_concat) T::AuctionId => Option<AuctionItem<T::AccountId, T::BlockNumber, T::Balance>>;

		/// Track the next auction ID.
		pub AuctionsIndex get(fn auctions_index): T::AuctionId;

		/// Index auctions by end time.
		pub AuctionEndTime get(fn auction_end_time): double_map hasher(twox_64_concat) T::BlockNumber, hasher(twox_64_concat) T::AuctionId => Option<()>;
	}
}

decl_event!(
	pub enum Event<T> where
		<T as frame_system::Trait>::AccountId,
		<T as Trait>::Balance,
		// AssetId = AssetId,
		<T as Trait>::AuctionId,
	{
		/// A bid is placed. [auction_id, bidder, bidding_amount]
		Bid(AuctionId, AccountId, Balance),
		NewAuctionItem(AuctionId, AccountId ,Balance, Balance),
	}
);

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
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
				match bid_result.auction_end_change {
					Change::NewValue(new_end) => {
						if let Some(old_end_block) = auction.end {
							<AuctionEndTime<T>>::remove(&old_end_block, id);
						}
						if let Some(new_end_block) = new_end {
							<AuctionEndTime<T>>::insert(&new_end_block, id, ());
						}
						auction.end = new_end;
					},
					Change::NoChange => {},
				}
				auction.bid = Some((from.clone(), value));

				Self::deposit_event(RawEvent::Bid(id, from, value));

				Ok(())
			})?;

		}


		#[weight = 10_000]
		fn create_auction(origin, id: T::AuctionId ,asset_id: AssetId, value: T::Balance) {
			let from = ensure_signed(origin)?;

			//Check ownership
			ensure!(<AssetByOwner<T>>::contains_key(&from, &asset_id), Error::<T>::NoPermissionToCreateAuction);

			let start_time = <system::Module<T>>::block_number();

			let end_time: T::BlockNumber = start_time + T::AuctionTimeToClose::get(); //add 7 days block for default auction



			let auction_id = Self::new_auction(from.clone(), value, start_time, None)?;

			let new_auction_item = AuctionItem {
				asset_id,
				recipient : from.clone(),
				initial_amount : value,
				amount : value,
				start_time : start_time
			};

			<AuctionItems<T>>::insert(
				auction_id,
				new_auction_item
			);

			Self::deposit_event(RawEvent::NewAuctionItem(id, from, value ,value));
		}

		/// dummy `on_initialize` to return the weight used in `on_finalize`.
		// fn on_initialize(now: T::BlockNumber) -> Weight {
		// 	T::WeightInfo::on_finalize(<AuctionEndTime<T>>::iter_prefix(&now).count() as u32)
		// }

		fn on_finalize(now: T::BlockNumber) {
			Self::_on_finalize(now);
		}
	}
}

decl_error! {
	/// Error for auction module.
	pub enum Error for Module<T: Trait> {
		AuctionNotExist,
		AuctionNotStarted,
		BidNotAccepted,
		InvalidBidPrice,
		NoAvailableAuctionId,
		NoPermissionToCreateAuction
	}
}

impl<T: Trait> Module<T> {
	fn _on_finalize(now: T::BlockNumber) {
		for (auction_id, _) in <AuctionEndTime<T>>::drain_prefix(&now) {
			if let Some(auction) = <Auctions<T>>::take(&auction_id) {
				T::Handler::on_auction_ended(auction_id, auction.bid);
			}
		}
	}

	fn auction_info(id: T::AuctionId) -> Option<AuctionInfo<T::AccountId, T::Balance, T::BlockNumber>> {
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
		
		let auction: AuctionInfo<T::AccountId, T::Balance, T::BlockNumber> = AuctionInfo { bid: None, start, end };

		let auction_id: T::AuctionId = <AuctionsIndex<T>>::try_mutate(|n| -> Result<T::AuctionId, DispatchError> {
			let id = *n;
			ensure!(id != T::AuctionId::max_value(), Error::<T>::NoAvailableAuctionId);
			*n += One::one();
			Ok(id)
		})?;

		// <Auctions<T>>::insert(auction_id, auction);

		// if let Some(end_block) = end {
		// 	<AuctionEndTime<T>>::insert(&end_block, auction_id, ());
		// }

		Ok(auction_id)
	}

	fn remove_auction(id: T::AuctionId) {
		if let Some(auction) = <Auctions<T>>::take(&id) {
			if let Some(end_block) = auction.end {
				<AuctionEndTime<T>>::remove(end_block, id);
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
}

// impl<T: Trait> Auction<T::AccountId, T::BlockNumber> for Module<T> {
// 	type AuctionId = T::AuctionId;
// 	type Balance = T::Balance;

	

// }

impl<T: Trait> AuctionHandler<T::AccountId, T::Balance, T::BlockNumber, T::AuctionId> for Module<T> {
	fn on_new_bid(
		now: T::BlockNumber,
		_id: T::AuctionId,
		new_bid: (T::AccountId, T::Balance),
		_last_bid: Option<(T::AccountId, T::Balance)>,
	) -> OnNewBidResult<T::BlockNumber> {
		
		
		// if new_bid.0 == ALICE {
		// 	OnNewBidResult {
		// 		accept_bid: true,
		// 		auction_end_change: Change::NewValue(Some(now + BID_EXTEND_BLOCK)),
		// 	}
		// } else {
		// 	OnNewBidResult {
		// 		accept_bid: false,
		// 		auction_end_change: Change::NoChange,
		// 	}			
		// }
		OnNewBidResult {
			accept_bid: false,
			auction_end_change: Change::NoChange,
		}
	}

	fn on_auction_ended(_id: T::AuctionId, _winner: Option<(T::AccountId, T::Balance)>) {}
}
