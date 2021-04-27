#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};
use pallet_nft::{TokenType, CollectionType};


fn init_test_nft(owner: Origin) {

    //Create group collection before class
    assert_ok!(NFTModule::<Runtime>::create_group(
        owner.clone(),
        vec![1],
        vec![1]
    ));

    assert_ok!(NFTModule::<Runtime>::create_class(
        owner.clone(),
        vec![1],
        vec![1],
        COLLECTION_ID,
        TokenType::Transferrable,
        CollectionType::Collectable,
    ));

    assert_ok!(NFTModule::<Runtime>::mint(
        owner.clone(),
        CLASS_ID,
        vec![1],
        vec![1],
        vec![1],
        1
    ));    
}

#[test]
// Private new_auction should work
fn create_new_auction_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);
        init_test_nft(origin.clone());

        assert_eq!(NFTModule::<Runtime>::get_asset(0), Some((CLASS_ID, 0)));
        assert_ok!(NftAuctionModule::create_auction(origin.clone(), ItemId::NFT(0), 100)); 
        assert_eq!(last_event(), Event::auction(RawEvent::NewAuctionItem(0, ALICE, 100 ,100)));
    });
}

#[test]
// Walk the happy path
fn bid_works() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = Origin::signed(BOB);
        let bidder = Origin::signed(ALICE);
        
        init_test_nft(owner.clone());        
        assert_ok!(NftAuctionModule::create_auction(owner, ItemId::NFT(0), 100));
        
        assert_ok!(NftAuctionModule::bid(bidder, 0, 200));
        assert_eq!(last_event(), Event::auction(RawEvent::Bid(0, ALICE, 200)));

        assert_eq!(Balances::reserved_balance(ALICE), 200);
    });
}

#[test]
fn cannot_bid_on_non_existent_auction() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            NftAuctionModule::bid(Origin::signed(ALICE), 0, 10), 
            Error::<Runtime>::AuctionNotExist
        ); 

        assert_eq!(Balances::free_balance(ALICE), 100000);
    });
}

#[test]
fn cannot_bid_with_insufficient_funds() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = Origin::signed(BOB);
        let bidder = Origin::signed(ALICE);
        
        init_test_nft(owner.clone());        
        assert_ok!(NftAuctionModule::create_auction(owner, ItemId::NFT(0), 600));
        
        assert_noop!(
            NftAuctionModule::bid(bidder, 0, 100001), 
            Error::<Runtime>::InsufficientFreeBalance
        );

        assert_eq!(Balances::free_balance(ALICE), 100000);

    });
}

#[test]
fn cannot_bid_on_own_auction() {
    ExtBuilder::default().build().execute_with(|| {        
        let owner = Origin::signed(ALICE);
        
        init_test_nft(owner.clone());        
        assert_ok!(NftAuctionModule::create_auction(owner.clone(), ItemId::NFT(0), 100));
        
        assert_noop!(
            NftAuctionModule::bid(owner, 0, 50), 
            Error::<Runtime>::SelfBidNotAccepted
        );
    });
}

#[test]
fn asset_transfers_after_auction() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = Origin::signed(BOB);
        let bidder = Origin::signed(ALICE);
        
        // Make sure balances start off as we expect
        assert_eq!(Balances::free_balance(BOB), 500);
        assert_eq!(Balances::free_balance(ALICE), 100000);

        // Setup NFT and verify that BOB has ownership
        init_test_nft(owner.clone());            
        assert_eq!(NFTModule::<Runtime>::get_assets_by_owner(BOB), [0]);

        assert_ok!(NftAuctionModule::create_auction(owner, ItemId::NFT(0), 100));
    
        assert_ok!(NftAuctionModule::bid(bidder, 0, 200));
        assert_eq!(last_event(), Event::auction(RawEvent::Bid(0, ALICE, 200)));

        run_to_block(102);
        
        // Verify asset transfers to alice after end of auction
        assert_eq!(
            last_event(), 
            Event::auction(RawEvent::AuctionFinalized(0, 1 ,200))
        );          

        // Verify transfer of funs (minus gas)
        assert_eq!(Balances::free_balance(BOB), 697);
        assert_eq!(Balances::free_balance(ALICE), 99800);

        // Verify Alice has the NFT and Bob doesn't
        assert_eq!(NFTModule::<Runtime>::get_assets_by_owner(ALICE), [0]);                
        assert_eq!(NFTModule::<Runtime>::get_assets_by_owner(BOB), Vec::<u64>::new());
    });
}

#[test]
fn cannot_bid_on_ended_auction() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = Origin::signed(BOB);
        let bidder = Origin::signed(ALICE);
        
        init_test_nft(owner.clone());        
        assert_ok!(NftAuctionModule::create_auction(owner, ItemId::NFT(0), 100));

        System::set_block_number(101);
                
        assert_noop!(
            NftAuctionModule::bid(bidder, 0, 200), 
            Error::<Runtime>::AuctionIsExpired
        );

        assert_eq!(Balances::free_balance(ALICE), 100000);
    });
}



