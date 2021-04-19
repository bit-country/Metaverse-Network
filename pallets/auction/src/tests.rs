#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok, 
	traits::{OnFinalize}};
use mock::{Event, *};

use orml_nft::Pallet as NFTPallet;
use pallet_nft as NFTModule;


fn setup() {
    // Create NFT Class Data required to create an auction needed for tests
    let class_data = NftClassData
    {
        deposit: 1,
        properties:Vec::new(),
        token_type: NFTModule::TokenType::Transferrable,
        collection_type: NFTModule::CollectionType::Collectable,
        total_supply: Default::default(),
        initial_supply: Default::default()
    };
    assert_ok!(NFTPallet::<Runtime>::create_class(&BOB, Vec::new(), class_data), CLASS_ID);
    
    // call create_auction
    assert_ok!(NftAuctionModule::create_auction(Origin::signed(BOB), (CLASS_ID,TOKEN_ID) ,100));
}

fn run_to_block(n: u64) {
    while System::block_number() < n {
        NftAuctionModule::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
    }
}

#[test]
// Private new_auction should work
fn create_new_auction_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(NftAuctionModule::new_auction(BOB, 100, 0, None), 0);    
        assert_eq!(NftAuctionModule::auctions(0), Some(AuctionInfo { bid: None, start: 0, end: None }));

        //With end block
        assert_ok!(NftAuctionModule::new_auction(BOB, 100, 0, Some(100)), 1); 
        assert_eq!(NftAuctionModule::auction_end_time(100,1), Some(()));
    });
}

#[test]
// Private remove_auction should work
fn remove_auction_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(NftAuctionModule::new_auction(BOB, 100, 0, Some(100)), 0); 
        assert_eq!(NftAuctionModule::auction_end_time(100,0), Some(()));   
        NftAuctionModule::remove_auction(0);
        assert_eq!(NftAuctionModule::auctions(0), None);
        assert_eq!(NftAuctionModule::auction_end_time(0,0), None);
    });
}


#[test]
// Private update_auction should work
fn update_auction_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(NftAuctionModule::new_auction(BOB, 100, 0, Some(100)), 0);
        assert_noop!(NftAuctionModule::update_auction(1, AuctionInfo {
            bid: Some((BOB, 100)),
            start: 10,
            end: Some(100)
        }), Error::<Runtime>::AuctionNotExist);
        assert_ok!(NftAuctionModule::update_auction(0, AuctionInfo {
            bid: Some((BOB, 200)),
            start: 20,
            end: Some(200)
        }));
        assert_eq!(NftAuctionModule::auctions(0), Some(AuctionInfo { bid: Some((BOB, 200)), start: 20, end: Some(200) }));
        //updates end time
        assert_eq!(NftAuctionModule::auction_end_time(200,0), Some(()));
    });
}

#[test]
// Private create_auction should work
fn create_auction_work() {
    ExtBuilder::default().build().execute_with(|| {
        //setup
        setup();
        
        //auction exists in auction and auction items
        assert_eq!(NftAuctionModule::auctions(0), Some(AuctionInfo { bid: None, start: 1, end: Some(101) }));
        assert_eq!(NftAuctionModule::get_auction_item(0), Some(AuctionItem { initial_amount:100, amount: 100, asset_id:TOKEN_ID, class_id:CLASS_ID, recipient: BOB, start_time: 1, end_time: 101 }));
        
        //event was triggered
        let event = mock::Event::auction(RawEvent::NewAuctionItem(0, BOB, 100,100));
        assert_eq!(last_event(), event);
    });
}

#[test]
// Private create_auction should work
fn create_auction_fail() {
    ExtBuilder::default().build().execute_with(|| {
        //account does not have permission to create auction
        assert_noop!(NftAuctionModule::create_auction(Origin::signed(BOB), (CLASS_ID,TOKEN_ID) ,100
        ), Error::<Runtime>::NoPermissionToCreateAuction);

        //token type is not transferrable
        let class_data = NftClassData
        {
            deposit: 1,
            properties:Vec::new(),
            token_type: NFTModule::TokenType::BoundToAddress,
            collection_type: NFTModule::CollectionType::Collectable,
            total_supply: Default::default(),
            initial_supply: Default::default()
        };

        assert_ok!(
            NFTPallet::<Runtime>::create_class(&BOB, Vec::new(), class_data), CLASS_ID);
        
        assert_noop!(NftAuctionModule::create_auction(Origin::signed(BOB), (CLASS_ID,TOKEN_ID) ,100), Error::<Runtime>::NoPermissionToCreateAuction);
    });
}

#[test]
// Private bid_auction should work
fn bid_auction_work() {
    ExtBuilder::default().build().execute_with(|| {
        setup();
        
        //first bid
        assert_ok!(NftAuctionModule::bid(Origin::signed(ALICE), 0, 100));

        //event was triggered
        let event = mock::Event::auction(RawEvent::Bid(0, ALICE, 100));
        assert_eq!(last_event(), event);
        
        //auction and auction items are updated
        assert_eq!(NftAuctionModule::auctions(0), Some(AuctionInfo { bid: Some((1, 100)), start: 1, end: Some(101) }));
        assert_eq!(NftAuctionModule::get_auction_item(0), Some(AuctionItem { initial_amount:100, amount: 100, asset_id:0, class_id:0, recipient: 2, start_time: 1, end_time: 101 }));

        //second bid test
        assert_ok!(NftAuctionModule::bid(Origin::signed(ALICE), 0, 200));
        assert_eq!(NftAuctionModule::auctions(0), Some(AuctionInfo { bid: Some((1, 200)), start: 1, end: Some(101) }));
        assert_eq!(NftAuctionModule::get_auction_item(0), Some(AuctionItem { initial_amount:100, amount: 200, asset_id:0, class_id:0, recipient: 2, start_time: 1, end_time: 101 }));
    });
}

#[test]
// Private bid_auction should fail
fn bid_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
    
    // placing a bid when auction does not exist
    assert_noop!(NftAuctionModule::bid(Origin::signed(BOB), 0, 100
    ), Error::<Runtime>::AuctionNotExist);

    // placing a bid when auction has not started
    assert_ok!(NftAuctionModule::new_auction(BOB, 100, 2, Some(100)), 0);
    assert_noop!(NftAuctionModule::bid(Origin::signed(BOB), 0, 100
    ), Error::<Runtime>::AuctionNotStarted);

    // placing a bid when auction has passed
    assert_ok!(NftAuctionModule::new_auction(BOB, 0, 0, Some(0)), 1);
    assert_noop!(NftAuctionModule::bid(Origin::signed(BOB), 1, 100
    ), Error::<Runtime>::AuctionIsExpired);

    //placing a bid with an amount of zero
    assert_ok!(NftAuctionModule::new_auction(BOB, 200, 0, Some(100)), 2);
    assert_noop!(NftAuctionModule::bid(Origin::signed(BOB), 2, 0
    ), Error::<Runtime>::InvalidBidPrice);

    //placing a bid when user has not accepted bid
    assert_noop!(NftAuctionModule::bid(Origin::signed(BOB), 2, 1
    ), Error::<Runtime>::BidNotAccepted);

     //placing a bid when user has underbid
     assert_noop!(NftAuctionModule::bid(Origin::signed(BOB), 2, 1
    ), Error::<Runtime>::BidNotAccepted);

    //placing bid lower than the previous bid
    setup();
    assert_ok!(NftAuctionModule::bid(Origin::signed(ALICE), 3, 100
    ));
    assert_noop!(NftAuctionModule::bid(Origin::signed(BOB), 3, 10), Error::<Runtime>::InvalidBidPrice);

    //placing bid higher than available balance
    assert_noop!(NftAuctionModule::bid(Origin::signed(ALICE), 3, 1000000000), "You don\'t have enough free balance for this bid");

   // check if a bid was created in any of the previous test cases 
   assert_eq!(NftAuctionModule::auctions(0), Some(AuctionInfo { bid: None, start: 2, end: Some(100) }));
   assert_eq!(NftAuctionModule::get_auction_item(0), None);
});

}

#[test]
// Private new_auction should work
fn on_auction_bid_handler() {
    ExtBuilder::default().build().execute_with(|| {
        setup();
        assert_eq!(NftAuctionModule::get_auction_item(0), Some(AuctionItem { initial_amount:100, amount: 100, asset_id:0, class_id:0, recipient: 2, start_time: 1, end_time: 101 }));

        assert_ok!(NftAuctionModule::auction_bid_handler(1,0,(ALICE,1000),Some((ALICE,1))));

        assert_eq!(NftAuctionModule::get_auction_item(0), Some(AuctionItem { initial_amount:100, amount: 1000, asset_id:0, class_id:0, recipient: 2, start_time: 1, end_time: 101 }));
    });
}

#[test]
// Private auction_bid_handler should not work
fn on_auction_bid_handler_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
         setup();
         //fail when bid is zero
         assert_noop!(NftAuctionModule::auction_bid_handler(1,0,(ALICE,0),Some((ALICE,1))), Error::<Runtime>::InvalidBidPrice);
         //fail when auction does not exist
         assert_noop!(NftAuctionModule::auction_bid_handler(1,2,(ALICE,1),Some((ALICE,1))), "Auction is not exists");
         //check failed auction bid handler calls does not mutate state
         assert_eq!(NftAuctionModule::get_auction_item(0), Some(AuctionItem { initial_amount:100, amount: 100, asset_id:0, class_id:0, recipient: 2, start_time: 1, end_time: 101 }));
    });
}


#[test]
// Private auction_bid_handler should not work
fn on_finalize_should_work() {
    ExtBuilder::default().build().execute_with(|| {
         setup();
         let nft_data = NftAssetData
         {
             deposit: 1,
             properties:Vec::new(),
             name: Vec::new(),
             description: Vec::new(),
         };
         assert_ok!(NFTPallet::<Runtime>::mint(&BOB, 0,Vec::new(), nft_data), 0);
         assert_ok!(NftAuctionModule::bid(Origin::signed(ALICE), 0, 100));
         run_to_block(102);
         assert_eq!(NftAuctionModule::auctions(0), None);
        // check account received asset
        assert_eq!(NFTPallet::<Runtime>::tokens(CLASS_ID, TOKEN_ID).unwrap().owner, ALICE);
        // check balances were transferred
        assert_eq!(Balances::free_balance(ALICE), 99900);
        assert_eq!(Balances::free_balance(BOB), 100);
        //event was triggered
        let event = mock::Event::auction(RawEvent::AuctionFinalized(0, ALICE, 100));
        assert_eq!(last_event(), event);
    });
}
