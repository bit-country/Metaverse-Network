#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};
use pallet_nft::{TokenType, CollectionType};
use primitives::BlockNumber;

#[test]
// Creating auction should work
fn create_new_auction_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);

        //Create NFT before creating an auction
        assert_ok!(NFTModule::<Runtime>::create_group(
            origin.clone(),
            vec![1],
            vec![1]
        ));

        assert_ok!(NFTModule::<Runtime>::create_class(
			origin.clone(),
			vec![1],
            vec![1],
            COLLECTION_ID,
            TokenType::Transferrable,
            CollectionType::Collectable,
		));

        assert_ok!(NFTModule::<Runtime>::mint(
			origin.clone(),
			CLASS_ID,
            vec![1],
            vec![1],
            vec![1],
            1
		));
        assert_eq!(NFTModule::<Runtime>::get_asset(0), Some((CLASS_ID, 0)));

        //Create auction
        assert_ok!(NftAuctionModule::create_auction(origin.clone(), ItemId::NFT(0), 100));
        assert_eq!(NftAuctionModule::auctions_index(),1);
        assert_eq!(last_event(), Event::auction(RawEvent::NewAuctionItem(0, ALICE, 100, 100)));
    });
}

#[test]
// Creating with non-existing asset should fail
fn create_auction_with_non_existing_asset_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);

        // Cannot create auction with non-existing asset
        assert_noop!(NftAuctionModule::create_auction(origin.clone(), ItemId::NFT(0), 100),Error::<Runtime>::AssetIsNotExist);
    });
}

#[test]
// Bidding should work
fn bidding_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(BOB);
        let bidder_origin = Origin::signed(ALICE);
       
        //Create group collection before class
        assert_ok!(NFTModule::<Runtime>::create_group(
            origin.clone(),
            vec![1],
            vec![1]
        ));

        assert_ok!(NFTModule::<Runtime>::create_class(
			origin.clone(),
			vec![1],
            vec![1],
            COLLECTION_ID,
            TokenType::Transferrable,
            CollectionType::Collectable,
		));

        assert_ok!(NFTModule::<Runtime>::mint(
			origin.clone(),
			CLASS_ID,
            vec![1],
            vec![1],
            vec![1],
            1
		));

        assert_eq!(NFTModule::<Runtime>::get_asset(0), Some((CLASS_ID, 0)));

        // Create auction and bid on it
        assert_ok!(NftAuctionModule::create_auction(origin.clone(), ItemId::NFT(0), 100));
        assert_ok!(NftAuctionModule::update_auction(0, AuctionInfo {
            bid: None,
            start: 1,
            end: Some(101),
        })); 
        assert_eq!(NftAuctionModule::get_auction_item(0),Some(AuctionItem {
            item_id: ItemId::NFT(0),
            recipient: BOB,
            initial_amount: 100,
            amount:100,
            start_time: 1,
            end_time: 101
        }));
        assert_ok!(NftAuctionModule::bid(bidder_origin.clone(), 0, 101));
        assert_eq!(last_event(), Event::auction(RawEvent::Bid(0, ALICE, 101)));
        assert_eq!(NftAuctionModule::get_auction_item(0),Some(AuctionItem {
            item_id: ItemId::NFT(0),
            recipient: ALICE,
            initial_amount: 100,
            amount:101,
            start_time: 1,
            end_time: 101
        }));

    });
}

#[test]
// Bidding with insufficient funds shoud fail
fn bidding_with_isufficient_funds_fails() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(BOB);
        let bidder_origin = Origin::signed(ALICE);
       
        //Create group collection before class
        assert_ok!(NFTModule::<Runtime>::create_group(
            origin.clone(),
            vec![1],
            vec![1]
        ));

        assert_ok!(NFTModule::<Runtime>::create_class(
			origin.clone(),
			vec![1],
            vec![1],
            COLLECTION_ID,
            TokenType::Transferrable,
            CollectionType::Collectable,
		));

        assert_ok!(NFTModule::<Runtime>::mint(
			origin.clone(),
			CLASS_ID,
            vec![1],
            vec![1],
            vec![1],
            1
		));

        assert_eq!(NFTModule::<Runtime>::get_asset(0), Some((CLASS_ID, 0)));

        // Create auction and bid on it
        assert_ok!(NftAuctionModule::create_auction(origin.clone(), ItemId::NFT(0), 100)); 
        assert_noop!(NftAuctionModule::bid(bidder_origin.clone(), 0, 100001),DispatchError::Other("You don\'t have enough free balance for this bid"));
    });
}

#[test]
// Asset transferred correctly after bidding
fn asset_transfer_after_auction_end_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(BOB);
        //let bidder_origin = Origin::signed(ALICE);
       
        //Create group collection before class
        assert_ok!(NFTModule::<Runtime>::create_group(
            origin.clone(),
            vec![1],
            vec![1]
        ));

        assert_ok!(NFTModule::<Runtime>::create_class(
			origin.clone(),
			vec![1],
            vec![1],
            COLLECTION_ID,
            TokenType::Transferrable,
            CollectionType::Collectable,
		));

        assert_ok!(NFTModule::<Runtime>::mint(
			origin.clone(),
			CLASS_ID,
            vec![1],
            vec![1],
            vec![1],
            1
		));

        assert_eq!(NFTModule::<Runtime>::get_asset(0), Some((CLASS_ID, 0)));
        assert_eq!(NFTModule::<Runtime>::check_nft_ownership(&BOB,&0),Ok(true));
        assert_eq!(NFTModule::<Runtime>::check_nft_ownership(&ALICE,&0),Ok(false));
        // Create auction and bid on it
        assert_ok!(NftAuctionModule::create_auction(origin.clone(), ItemId::NFT(0), 100));
        assert_ok!(NftAuctionModule::bid(Origin::signed(ALICE), 0, 20000));
        assert_eq!(last_event(), Event::auction(RawEvent::Bid(0, ALICE, 20000)));
        // Simulate the auction's end
        run_to_block(102);
        assert_eq!(Balances::free_balance(&ALICE), 80000);
        assert_eq!(Balances::free_balance(&BOB), 21997);
        assert_eq!(NFTModule::<Runtime>::check_nft_ownership(&BOB,&0),Ok(false));
        assert_eq!(NFTModule::<Runtime>::check_nft_ownership(&ALICE,&0),Ok(true));
        assert_eq!(last_event(), Event::auction(RawEvent::AuctionFinalized(0, ALICE, 20000))); 
    });
}

#[test]
// Self bidding fails
fn self_bidding_fails() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(BOB);
        let bidder_origin = Origin::signed(ALICE);
       
        //Create group collection before class
        assert_ok!(NFTModule::<Runtime>::create_group(
            origin.clone(),
            vec![1],
            vec![1]
        ));

        assert_ok!(NFTModule::<Runtime>::create_class(
			origin.clone(),
			vec![1],
            vec![1],
            COLLECTION_ID,
            TokenType::Transferrable,
            CollectionType::Collectable,
		));

        assert_ok!(NFTModule::<Runtime>::mint(
			origin.clone(),
			CLASS_ID,
            vec![1],
            vec![1],
            vec![1],
            1
		));

        assert_eq!(NFTModule::<Runtime>::get_asset(0), Some((CLASS_ID, 0)));

        // Create auction and bid on it
        assert_ok!(NftAuctionModule::create_auction(origin.clone(), ItemId::NFT(0), 100)); 
        assert_noop!(NftAuctionModule::bid(origin.clone(), 0, 201),Error::<Runtime>::BidNotAccepted);
    });
}

#[test]
// Bidding on expired auction fails
fn bidding_on_expired_auction_fails() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(BOB);
        let bidder_origin = Origin::signed(ALICE);
       
        //Create group collection before class
        assert_ok!(NFTModule::<Runtime>::create_group(
            origin.clone(),
            vec![1],
            vec![1]
        ));

        assert_ok!(NFTModule::<Runtime>::create_class(
			origin.clone(),
			vec![1],
            vec![1],
            COLLECTION_ID,
            TokenType::Transferrable,
            CollectionType::Collectable,
		));

        assert_ok!(NFTModule::<Runtime>::mint(
			origin.clone(),
			CLASS_ID,
            vec![1],
            vec![1],
            vec![1],
            1
		));

        assert_eq!(NFTModule::<Runtime>::get_asset(0), Some((CLASS_ID, 0)));

        // Create auction and bid on it
        assert_ok!(NftAuctionModule::create_auction(origin.clone(), ItemId::NFT(0), 100));
        assert_ok!(NftAuctionModule::update_auction(0, AuctionInfo {
            bid: None,
            start: 0,
            end: Some(1),
        }));
        assert_noop!(NftAuctionModule::bid(bidder_origin.clone(), 0, 1000), Error::<Runtime>::AuctionIsExpired);
    });
}

#[test]
// Bidding on auction that have not started fails
fn bidding_on_auction_that_have_not_started_fails() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(BOB);
        let bidder_origin = Origin::signed(ALICE);
       
        //Create group collection before class
        assert_ok!(NFTModule::<Runtime>::create_group(
            origin.clone(),
            vec![1],
            vec![1]
        ));

        assert_ok!(NFTModule::<Runtime>::create_class(
			origin.clone(),
			vec![1],
            vec![1],
            COLLECTION_ID,
            TokenType::Transferrable,
            CollectionType::Collectable,
		));

        assert_ok!(NFTModule::<Runtime>::mint(
			origin.clone(),
			CLASS_ID,
            vec![1],
            vec![1],
            vec![1],
            1
		));

        assert_eq!(NFTModule::<Runtime>::get_asset(0), Some((CLASS_ID, 0)));

        // Create auction and bid on it
        assert_ok!(NftAuctionModule::create_auction(origin.clone(), ItemId::NFT(0), 100)); 
        assert_ok!(NftAuctionModule::update_auction(0, AuctionInfo {
            bid: None,
            start: 200,
            end: Some(300),
        }));
        assert_noop!(NftAuctionModule::bid(bidder_origin.clone(), 0, 1000),Error::<Runtime>::AuctionNotStarted);
    });
}