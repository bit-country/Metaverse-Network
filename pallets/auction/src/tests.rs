#![cfg(test)]

use frame_support::{assert_noop, assert_ok};
use sp_std::collections::btree_map::BTreeMap;

use auction_manager::ListingLevel;
use core_primitives::{Attributes, CollectionType, NFTTrait, TokenType};
use mock::{Event, *};
use primitives::ItemId::NFT;

use super::*;

fn init_test_nft(owner: Origin) {
	//Create group collection before class
	assert_ok!(NFTModule::<Runtime>::create_group(Origin::root(), vec![1], vec![1]));

	assert_ok!(NFTModule::<Runtime>::create_class(
		owner.clone(),
		vec![1],
		test_attributes(1),
		COLLECTION_ID,
		TokenType::Transferable,
		CollectionType::Collectable,
		Perbill::from_percent(1u32)
	));

	assert_ok!(NFTModule::<Runtime>::mint(
		owner.clone(),
		CLASS_ID,
		vec![1],
		test_attributes(1),
		1
	));
}

fn test_attributes(x: u8) -> Attributes {
	let mut attr: Attributes = BTreeMap::new();
	attr.insert(vec![x, x + 5], vec![x, x + 10]);
	attr
}

#[test]
// Creating auction should work
fn create_new_auction_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		init_test_nft(origin.clone());

		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			ItemId::NFT(0, 0),
			None,
			ALICE,
			100,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32)
		));

		assert_eq!(
			AuctionModule::auctions(0),
			Some(AuctionInfo {
				bid: None,
				start: 1,
				end: Some(101),
			})
		);
		assert_eq!(AuctionModule::items_in_auction(ItemId::NFT(0, 0)), Some(true));
	});
}

#[test]
// Creating auction should work
fn create_new_auction_should_work_for_valid_estate() {
	ExtBuilder::default().build().execute_with(|| {
		let item_id: ItemId = ItemId::Estate(ESTATE_ID_EXIST);
		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			item_id,
			None,
			ALICE,
			100,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32)
		));
		assert_eq!(
			AuctionModule::auctions(0),
			Some(AuctionInfo {
				bid: None,
				start: 1,
				end: Some(101),
			})
		);
		assert_eq!(AuctionModule::items_in_auction(item_id), Some(true));
	});
}

#[test]
// Creating auction should work
fn create_new_auction_should_fail_for_non_exist_estate() {
	ExtBuilder::default().build().execute_with(|| {
		let item_id: ItemId = ItemId::Estate(ESTATE_ID_NOT_EXIST);
		assert_noop!(
			AuctionModule::create_auction(
				AuctionType::Auction,
				item_id,
				None,
				ALICE,
				100,
				0,
				ListingLevel::Global,
				Perbill::from_percent(0u32),
			),
			Error::<Runtime>::EstateDoesNotExist
		);
	});
}

#[test]
// Creating auction should work
fn create_new_auction_should_work_for_valid_landunit() {
	ExtBuilder::default().build().execute_with(|| {
		let item_id: ItemId = ItemId::LandUnit(LAND_UNIT_EXIST, ALICE_METAVERSE_ID);
		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			item_id,
			None,
			ALICE,
			100,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32),
		));
		assert_eq!(
			AuctionModule::auctions(0),
			Some(AuctionInfo {
				bid: None,
				start: 1,
				end: Some(101),
			})
		);
		assert_eq!(AuctionModule::items_in_auction(item_id), Some(true));
	});
}

#[test]
// Creating auction should work
fn create_new_auction_should_work_for_non_exist_landunit() {
	ExtBuilder::default().build().execute_with(|| {
		let item_id: ItemId = ItemId::LandUnit(LAND_UNIT_NOT_EXIST, ALICE_METAVERSE_ID);
		assert_noop!(
			AuctionModule::create_auction(
				AuctionType::Auction,
				item_id,
				None,
				ALICE,
				100,
				0,
				ListingLevel::Global,
				Perbill::from_percent(0u32),
			),
			Error::<Runtime>::LandUnitDoesNotExist
		);
	});
}

#[test]
// Private create_auction should work
fn create_auction_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let owner = Origin::signed(ALICE);

		assert_ok!(NFTModule::<Runtime>::create_group(Origin::root(), vec![1], vec![1]));

		assert_ok!(NFTModule::<Runtime>::create_class(
			owner.clone(),
			vec![1],
			Default::default(),
			COLLECTION_ID,
			TokenType::Transferable,
			CollectionType::Collectable,
			Perbill::from_percent(0u32)
		));

		assert_ok!(NFTModule::<Runtime>::mint(
			owner.clone(),
			CLASS_ID,
			vec![1],
			Default::default(),
			1
		));
		//account does not have permission to create auction
		assert_noop!(
			AuctionModule::create_auction(
				AuctionType::Auction,
				ItemId::NFT(0, 0),
				None,
				BOB,
				100,
				0,
				ListingLevel::Global,
				Perbill::from_percent(0u32)
			),
			Error::<Runtime>::NoPermissionToCreateAuction
		);

		assert_ok!(NFTModule::<Runtime>::create_class(
			owner.clone(),
			vec![1],
			Default::default(),
			COLLECTION_ID,
			TokenType::BoundToAddress,
			CollectionType::Collectable,
			Perbill::from_percent(0u32)
		));

		assert_ok!(NFTModule::<Runtime>::mint(
			owner.clone(),
			1,
			vec![1],
			Default::default(),
			1
		));

		//Class is BoundToAddress
		assert_noop!(
			AuctionModule::create_auction(
				AuctionType::Auction,
				ItemId::NFT(1, 0),
				None,
				ALICE,
				100,
				0,
				ListingLevel::Global,
				Perbill::from_percent(0u32)
			),
			Error::<Runtime>::NoPermissionToCreateAuction
		);

		//Asset is already in an auction
		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			ItemId::NFT(0, 0),
			None,
			ALICE,
			100,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32)
		));
		assert_noop!(
			AuctionModule::create_auction(
				AuctionType::Auction,
				ItemId::NFT(0, 0),
				None,
				ALICE,
				100,
				0,
				ListingLevel::Global,
				Perbill::from_percent(0u32)
			),
			Error::<Runtime>::ItemAlreadyInAuction
		);
	});
}

#[test]
// Creating auction should work
fn create_new_auction_should_fail_when_exceed_finality_limit() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);

		// Create 4 nfts

		init_test_nft(origin.clone());
		init_test_nft(origin.clone());
		init_test_nft(origin.clone());
		init_test_nft(origin.clone());

		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			ItemId::NFT(0, 0),
			None,
			ALICE,
			100,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32)
		));

		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			ItemId::NFT(0, 1),
			None,
			ALICE,
			100,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32)
		));

		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			ItemId::NFT(0, 2),
			None,
			ALICE,
			100,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32)
		));

		// Mocking max finality is 3
		// 4th auction with new block should fail
		assert_noop!(
			AuctionModule::create_auction(
				AuctionType::Auction,
				ItemId::NFT(0, 3),
				None,
				ALICE,
				100,
				0,
				ListingLevel::Global,
				Perbill::from_percent(0u32)
			),
			Error::<Runtime>::ExceedFinalityLimit
		);

		run_to_block(2);

		// Should able to create auction for next block
		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			ItemId::NFT(0, 3),
			None,
			ALICE,
			100,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32)
		));
	});
}

#[test]
// Private remove_auction should work
fn remove_auction_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		init_test_nft(origin.clone());
		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			ItemId::NFT(0, 0),
			None,
			ALICE,
			100,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32)
		));
		AuctionModule::remove_auction(0, ItemId::NFT(0, 0));
		assert_eq!(AuctionModule::auctions(0), None);
		assert_eq!(AuctionModule::items_in_auction(ItemId::NFT(0, 0)), None);
	});
}

#[test]
// Walk the happy path
fn bid_works() {
	ExtBuilder::default().build().execute_with(|| {
		let owner = Origin::signed(BOB);
		let bidder = Origin::signed(ALICE);

		init_test_nft(owner.clone());
		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			ItemId::NFT(0, 0),
			None,
			BOB,
			100,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32)
		));

		assert_ok!(AuctionModule::bid(bidder, 0, 200));
		assert_eq!(last_event(), Event::AuctionModule(crate::Event::Bid(0, ALICE, 200)));
		assert_eq!(Balances::reserved_balance(ALICE), 200);
	});
}

#[test]
// Walk the happy path
fn bid_works_for_valid_estate() {
	ExtBuilder::default().build().execute_with(|| {
		let bidder = Origin::signed(ALICE);
		let item_id: ItemId = ItemId::Estate(ESTATE_ID_EXIST);

		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			item_id,
			None,
			BOB,
			100,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32)
		));

		assert_ok!(AuctionModule::bid(bidder, 0, 200));
		assert_eq!(last_event(), Event::AuctionModule(crate::Event::Bid(0, ALICE, 200)));
		assert_eq!(Balances::reserved_balance(ALICE), 200);
	});
}

#[test]
// Walk the happy path
fn bid_works_for_valid_land_unit() {
	ExtBuilder::default().build().execute_with(|| {
		let bidder = Origin::signed(ALICE);
		let item_id: ItemId = ItemId::LandUnit(LAND_UNIT_EXIST, ALICE_METAVERSE_ID);

		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			item_id,
			None,
			BOB,
			100,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32)
		));

		assert_ok!(AuctionModule::bid(bidder, 0, 200));
		assert_eq!(last_event(), Event::AuctionModule(crate::Event::Bid(0, ALICE, 200)));
		assert_eq!(Balances::reserved_balance(ALICE), 200);
	});
}

#[test]
fn cannot_bid_on_non_existent_auction() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			AuctionModule::bid(Origin::signed(ALICE), 0, 10),
			Error::<Runtime>::AuctionDoesNotExist
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
		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			ItemId::NFT(0, 0),
			None,
			BOB,
			600,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32)
		));

		assert_noop!(
			AuctionModule::bid(bidder, 0, 100001),
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
		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			ItemId::NFT(0, 0),
			None,
			ALICE,
			100,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32)
		));

		assert_noop!(
			AuctionModule::bid(owner, 0, 50),
			Error::<Runtime>::CannotBidOnOwnAuction
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
		assert_eq!(NFTModule::<Runtime>::check_ownership(&BOB, &(0, 0)), Ok(true));

		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			ItemId::NFT(0, 0),
			None,
			BOB,
			100,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32)
		));

		assert_ok!(AuctionModule::bid(bidder, 0, 200));
		assert_eq!(last_event(), Event::AuctionModule(crate::Event::Bid(0, ALICE, 200)));

		run_to_block(102);
		// Verify asset transfers to alice after end of auction
		assert_eq!(
			last_event(),
			Event::AuctionModule(crate::Event::AuctionFinalized(0, 1, 200))
		);

		// Verify transfer of fund (minus gas)
		// BOB only receive 200 - 2 (1% of 200 as royalty fee) - 4 minting fee =
		assert_eq!(Balances::free_balance(BOB), 695);
		assert_eq!(Balances::free_balance(ALICE), 99800);

		// Verify Alice has the NFT and Bob doesn't
		assert_eq!(NFTModule::<Runtime>::check_ownership(&ALICE, &(0, 0)), Ok(true));
		assert_eq!(NFTModule::<Runtime>::check_ownership(&BOB, &(0, 0)), Ok(false));
	});
}

#[test]
fn cannot_bid_on_ended_auction() {
	ExtBuilder::default().build().execute_with(|| {
		let owner = Origin::signed(BOB);
		let bidder = Origin::signed(ALICE);

		init_test_nft(owner.clone());
		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			ItemId::NFT(0, 0),
			None,
			BOB,
			150,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32)
		));

		System::set_block_number(101);

		assert_noop!(AuctionModule::bid(bidder, 0, 200), Error::<Runtime>::AuctionIsExpired);

		assert_eq!(Balances::free_balance(ALICE), 100000);
	});
}

#[test]
// Private bid_auction should work
fn buy_now_work() {
	ExtBuilder::default().build().execute_with(|| {
		let owner = Origin::signed(BOB);
		let buyer = Origin::signed(ALICE);

		init_test_nft(owner.clone());

		// call create_auction
		assert_ok!(AuctionModule::create_auction(
			AuctionType::BuyNow,
			ItemId::NFT(0, 0),
			None,
			BOB,
			200,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32)
		));

		// buy now successful
		assert_ok!(AuctionModule::buy_now(buyer.clone(), 0, 200));

		assert_ok!(NFTModule::<Runtime>::mint(
			owner.clone(),
			CLASS_ID,
			vec![1],
			Default::default(),
			1
		));

		assert_ok!(AuctionModule::create_auction(
			AuctionType::BuyNow,
			ItemId::NFT(0, 1),
			None,
			BOB,
			200,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32)
		));

		assert_ok!(AuctionModule::buy_now(buyer.clone(), 1, 200));

		assert_eq!(AuctionModule::auctions(0), None);
		// check account received asset
		assert_eq!(NFTModule::<Runtime>::check_ownership(&ALICE, &(0, 0)), Ok(true));
		assert_eq!(NFTModule::<Runtime>::check_ownership(&ALICE, &(0, 1)), Ok(true));

		// check balances were transferred
		assert_eq!(Balances::free_balance(ALICE), 99600);
		// initial balance is 500 - sold 2 x 200 = 900
		// royalty fee is 1% for both sales is 8
		// 900 - 8 + 7 for deposit minting = 885
		assert_eq!(Balances::free_balance(BOB), 892);

		// event was triggered
		let event = mock::Event::AuctionModule(crate::Event::BuyNowFinalised(1, ALICE, 200));
		assert_eq!(last_event(), event);

		// Check that auction is over
		assert_noop!(
			AuctionModule::buy_now(buyer.clone(), 1, 150),
			Error::<Runtime>::AuctionDoesNotExist
		);
	});
}

#[test]
// Private bid_auction should work
fn buy_now_works_for_valid_estate() {
	ExtBuilder::default().build().execute_with(|| {
		// let owner = Origin::signed(BOB);
		let buyer = Origin::signed(ALICE);

		let item_id: ItemId = ItemId::Estate(ESTATE_ID_EXIST);
		// call create_auction
		assert_ok!(AuctionModule::create_auction(
			AuctionType::BuyNow,
			item_id,
			None,
			BOB,
			150,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32)
		));

		// buy now successful
		assert_ok!(AuctionModule::buy_now(buyer.clone(), 0, 150));

		assert_eq!(Balances::free_balance(BOB), 650);

		let item_id_1: ItemId = ItemId::Estate(ESTATE_ID_EXIST_1);
		assert_ok!(AuctionModule::create_auction(
			AuctionType::BuyNow,
			item_id_1,
			None,
			BOB,
			150,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32)
		));

		assert_ok!(AuctionModule::buy_now(buyer.clone(), 1, 150));

		assert_eq!(AuctionModule::auctions(0), None);

		// check balances were transferred
		assert_eq!(Balances::free_balance(ALICE), 99700);
		assert_eq!(Balances::free_balance(BOB), 800);

		// event was triggered
		let event = mock::Event::AuctionModule(crate::Event::BuyNowFinalised(1, ALICE, 150));
		assert_eq!(last_event(), event);

		// Check that auction is over
		assert_noop!(
			AuctionModule::buy_now(buyer.clone(), 1, 150),
			Error::<Runtime>::AuctionDoesNotExist
		);
	});
}

#[test]
// Private bid_auction should work
fn buy_now_works_for_valid_landunit() {
	ExtBuilder::default().build().execute_with(|| {
		// let owner = Origin::signed(BOB);
		let buyer = Origin::signed(ALICE);

		let item_id: ItemId = ItemId::LandUnit(LAND_UNIT_EXIST, ALICE_METAVERSE_ID);
		// call create_auction
		assert_ok!(AuctionModule::create_auction(
			AuctionType::BuyNow,
			item_id,
			None,
			BOB,
			150,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32)
		));

		// buy now successful
		assert_ok!(AuctionModule::buy_now(buyer.clone(), 0, 150));

		assert_eq!(Balances::free_balance(BOB), 650);

		let item_id_1: ItemId = ItemId::LandUnit(LAND_UNIT_EXIST_1, ALICE_METAVERSE_ID);
		assert_ok!(AuctionModule::create_auction(
			AuctionType::BuyNow,
			item_id_1,
			None,
			BOB,
			150,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32)
		));

		assert_ok!(AuctionModule::buy_now(buyer.clone(), 1, 150));

		assert_eq!(AuctionModule::auctions(0), None);

		// check balances were transferred
		assert_eq!(Balances::free_balance(ALICE), 99700);
		assert_eq!(Balances::free_balance(BOB), 800);

		// event was triggered
		let event = mock::Event::AuctionModule(crate::Event::BuyNowFinalised(1, ALICE, 150));
		assert_eq!(last_event(), event);

		// Check that auction is over
		assert_noop!(
			AuctionModule::buy_now(buyer.clone(), 1, 150),
			Error::<Runtime>::AuctionDoesNotExist
		);
	});
}

#[test]
// Private bid_auction should work
fn buy_now_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let owner = Origin::signed(BOB);
		let buyer = Origin::signed(ALICE);
		// we need this to test auction not started scenario
		System::set_block_number(1);
		init_test_nft(owner.clone());

		// call create_auction
		assert_ok!(AuctionModule::create_auction(
			AuctionType::BuyNow,
			ItemId::NFT(0, 0),
			None,
			BOB,
			150,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32)
		));

		// no auction id
		assert_noop!(
			AuctionModule::buy_now(buyer.clone(), 1, 150),
			Error::<Runtime>::AuctionDoesNotExist
		);
		// user is seller
		assert_noop!(
			AuctionModule::buy_now(owner.clone(), 0, 150),
			Error::<Runtime>::CannotBidOnOwnAuction
		);
		// buy it now value is less than buy_now_amount
		assert_noop!(
			AuctionModule::buy_now(buyer.clone(), 0, 100),
			Error::<Runtime>::InvalidBuyNowPrice
		);
		// buy it now value is more than buy_now_amount
		assert_noop!(
			AuctionModule::buy_now(buyer.clone(), 0, 200),
			Error::<Runtime>::InvalidBuyNowPrice
		);
		// user does not have enough balance in wallet
		assert_ok!(Balances::reserve(&ALICE, 100000));
		assert_noop!(
			AuctionModule::buy_now(buyer.clone(), 0, 150),
			Error::<Runtime>::InsufficientFreeBalance
		);
		assert_eq!(Balances::unreserve(&ALICE, 100000), 0);
		// auction has not started or is over
		System::set_block_number(0);
		assert_noop!(
			AuctionModule::buy_now(buyer.clone(), 0, 150),
			Error::<Runtime>::AuctionHasNotStarted
		);
		System::set_block_number(101);
		assert_noop!(
			AuctionModule::buy_now(buyer.clone(), 0, 150),
			Error::<Runtime>::AuctionIsExpired
		);
		System::set_block_number(1);
		assert_ok!(AuctionModule::buy_now(buyer.clone(), 0, 150));
		assert_noop!(
			AuctionModule::create_auction(
				AuctionType::BuyNow,
				ItemId::NFT(0, 0),
				None,
				BOB,
				150,
				0,
				ListingLevel::Global,
				Perbill::from_percent(0u32)
			),
			Error::<Runtime>::NoPermissionToCreateAuction
		);
	});
}

#[test]
// Private bid_auction should work
fn invalid_auction_type() {
	ExtBuilder::default().build().execute_with(|| {
		let owner = Origin::signed(BOB);
		init_test_nft(owner.clone());
		let participant = Origin::signed(ALICE);
		assert_ok!(AuctionModule::create_auction(
			AuctionType::BuyNow,
			ItemId::NFT(0, 0),
			None,
			BOB,
			150,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32)
		));
		assert_noop!(
			AuctionModule::bid(participant.clone(), 0, 200),
			Error::<Runtime>::InvalidAuctionType
		);
		AuctionModule::remove_auction(0, ItemId::NFT(0, 0));
		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			ItemId::NFT(0, 0),
			None,
			BOB,
			150,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32)
		));
		assert_noop!(
			AuctionModule::buy_now(participant.clone(), 1, 150),
			Error::<Runtime>::InvalidAuctionType
		);
	});
}

#[test]
// Private auction_bid_handler should work
fn on_finalize_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let owner = Origin::signed(BOB);
		let bidder = Origin::signed(ALICE);
		init_test_nft(owner.clone());
		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			ItemId::NFT(0, 0),
			None,
			BOB,
			100,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32)
		));
		assert_eq!(AuctionModule::items_in_auction(ItemId::NFT(0, 0)), Some(true));
		assert_ok!(AuctionModule::bid(bidder, 0, 100));
		run_to_block(102);
		assert_eq!(AuctionModule::auctions(0), None);
		// check account received asset
		assert_eq!(NFTModule::<Runtime>::check_ownership(&ALICE, &(0, 0)), Ok(true));
		// check balances were transferred
		assert_eq!(Balances::free_balance(ALICE), 99900);
		// BOB only receive 596 - 1 (1% of 100 as royalty fee) + 1 minting fee = 589
		assert_eq!(Balances::free_balance(BOB), 596);
		// asset is not longer in auction
		assert_eq!(AuctionModule::items_in_auction(ItemId::NFT(0, 0)), None);
		// event was triggered
		let event = mock::Event::AuctionModule(crate::Event::AuctionFinalized(0, ALICE, 100));
		assert_eq!(last_event(), event);
	});
}

#[test]
// Auction finalize with listing fee works
fn on_finalize_with_listing_fee_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let owner = Origin::signed(ALICE);
		let bidder = Origin::signed(BOB);
		init_test_nft(owner.clone());
		// After minting new NFT, it costs 3 unit
		assert_eq!(Balances::free_balance(ALICE), 99997);
		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			ItemId::NFT(0, 0),
			None,
			ALICE,
			100,
			0,
			ListingLevel::Local(ALICE_METAVERSE_ID),
			Perbill::from_percent(10u32)
		));
		assert_eq!(AuctionModule::items_in_auction(ItemId::NFT(0, 0)), Some(true));
		assert_ok!(AuctionModule::bid(bidder, 0, 100));
		run_to_block(102);
		assert_eq!(AuctionModule::auctions(0), None);
		// check account received asset
		assert_eq!(NFTModule::<Runtime>::check_ownership(&BOB, &(0, 0)), Ok(true));
		// check balances were transferred
		// Bob bid 100 for item, his new balance will be 500 - 100
		assert_eq!(Balances::free_balance(BOB), 400);
		// Alice only receive 89 for item sold - cost breakdown 100 - 1 (royalty) - 10 (listing fee)
		// Free balance of Alice is 99997 + 86 = 100086
		assert_eq!(Balances::free_balance(ALICE), 100086);
		// asset is not longer in auction
		assert_eq!(AuctionModule::items_in_auction(ItemId::NFT(0, 0)), None);
		// event was triggered
		let event = mock::Event::AuctionModule(crate::Event::AuctionFinalized(0, BOB, 100));
		assert_eq!(last_event(), event);
	});
}

#[test]
// List item on local marketplace should work if metaverse owner
fn list_item_on_auction_local_marketplace_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		init_test_nft(origin.clone());
		assert_ok!(AuctionModule::create_new_auction(
			origin,
			ItemId::NFT(0, 0),
			100,
			102,
			ListingLevel::Local(ALICE_METAVERSE_ID),
		));
		assert_eq!(AuctionModule::items_in_auction(ItemId::NFT(0, 0)), Some(true))
	});
}

#[test]
// List item on local marketplace should work if metaverse owner
fn list_item_on_buy_now_local_marketplace_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		init_test_nft(origin.clone());
		assert_ok!(AuctionModule::create_new_auction(
			origin,
			ItemId::NFT(0, 0),
			100,
			102,
			ListingLevel::Local(ALICE_METAVERSE_ID),
		));
		assert_eq!(AuctionModule::items_in_auction(ItemId::NFT(0, 0)), Some(true))
	});
}
