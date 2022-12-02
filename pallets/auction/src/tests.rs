#![cfg(test)]

use frame_support::{assert_noop, assert_ok};
use sp_std::collections::btree_map::BTreeMap;

use auction_manager::ListingLevel;
use core_primitives::{Attributes, CollectionType, NFTTrait, TokenType};
use mock::{Event, *};
use primitives::ItemId::NFT;
use primitives::{ClassId, FungibleTokenId};

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
		Perbill::from_percent(1u32),
		None
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
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0)
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
		assert_eq!(Balances::free_balance(ALICE), 99996);
	});
}

#[test]
// Creating auction should work
fn create_new_multicurrency_auction_work() {
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
			Perbill::from_percent(0u32),
			FungibleTokenId::MiningResource(0)
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
		assert_eq!(Balances::free_balance(ALICE), 99996);
	});
}

#[test]
// Creating auction should work
fn create_new_auction_bundle_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		init_test_nft(origin.clone());
		init_test_nft(origin.clone());
		init_test_nft(origin.clone());

		let tokens: Vec<(u32, u64, Balance)> = vec![(0, 0, 30), (0, 1, 30), (0, 2, 40)];
		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			ItemId::Bundle(tokens.clone()),
			None,
			ALICE,
			100,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0)
		));

		assert_eq!(
			AuctionModule::auctions(0),
			Some(AuctionInfo {
				bid: None,
				start: 1,
				end: Some(101),
			})
		);

		assert_eq!(
			AuctionModule::items_in_auction(ItemId::Bundle(tokens.clone())),
			Some(true)
		);
		assert_eq!(Balances::free_balance(ALICE), 99990);
	});
}

#[test]
// Creating auction should work
fn create_new_multicurrency_auction_bundle_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		init_test_nft(origin.clone());
		init_test_nft(origin.clone());
		init_test_nft(origin.clone());

		let tokens: Vec<(u32, u64, Balance)> = vec![(0, 0, 30), (0, 1, 30), (0, 2, 40)];
		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			ItemId::Bundle(tokens.clone()),
			None,
			ALICE,
			100,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32),
			FungibleTokenId::MiningResource(0)
		));

		assert_eq!(
			AuctionModule::auctions(0),
			Some(AuctionInfo {
				bid: None,
				start: 1,
				end: Some(101),
			})
		);

		assert_eq!(
			AuctionModule::items_in_auction(ItemId::Bundle(tokens.clone())),
			Some(true)
		);
		assert_eq!(Balances::free_balance(ALICE), 99990);
	});
}

#[test]
// Creating auction should work
fn create_new_auction_bundle_from_listed_nft_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		init_test_nft(origin.clone());
		init_test_nft(origin.clone());

		let tokens: Vec<(u32, u64, Balance)> = vec![(0, 0, 30), (0, 1, 30)];
		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			ItemId::NFT(0, 1),
			None,
			ALICE,
			100,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0)
		));

		assert_eq!(
			AuctionModule::auctions(0),
			Some(AuctionInfo {
				bid: None,
				start: 1,
				end: Some(101),
			})
		);

		assert_ne!(
			AuctionModule::create_auction(
				AuctionType::Auction,
				ItemId::Bundle(tokens.clone()),
				None,
				ALICE,
				100,
				0,
				ListingLevel::Global,
				Perbill::from_percent(0u32),
				FungibleTokenId::NativeToken(0)
			),
			Ok(0)
		);
	});
}

#[test]
// Creating auction should work
fn create_new_buy_now_bundle_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		init_test_nft(origin.clone());
		init_test_nft(origin.clone());
		init_test_nft(origin.clone());

		let tokens: Vec<(u32, u64, Balance)> = vec![(0, 0, 30), (0, 1, 30), (0, 2, 40)];
		assert_ok!(AuctionModule::create_auction(
			AuctionType::BuyNow,
			ItemId::Bundle(tokens.clone()),
			None,
			ALICE,
			100,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0)
		));

		assert_eq!(
			AuctionModule::auctions(0),
			Some(AuctionInfo {
				bid: None,
				start: 1,
				end: Some(101),
			})
		);

		assert_eq!(
			AuctionModule::items_in_auction(ItemId::Bundle(tokens.clone())),
			Some(true)
		);
		assert_eq!(Balances::free_balance(ALICE), 99990);
	});
}

#[test]
// Creating auction should work
fn create_new_multicurrency_buy_now_bundle_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		init_test_nft(origin.clone());
		init_test_nft(origin.clone());
		init_test_nft(origin.clone());

		let tokens: Vec<(u32, u64, Balance)> = vec![(0, 0, 30), (0, 1, 30), (0, 2, 40)];
		assert_ok!(AuctionModule::create_auction(
			AuctionType::BuyNow,
			ItemId::Bundle(tokens.clone()),
			None,
			ALICE,
			100,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32),
			FungibleTokenId::MiningResource(0)
		));

		assert_eq!(
			AuctionModule::auctions(0),
			Some(AuctionInfo {
				bid: None,
				start: 1,
				end: Some(101),
			})
		);

		assert_eq!(
			AuctionModule::items_in_auction(ItemId::Bundle(tokens.clone())),
			Some(true)
		);
		assert_eq!(Balances::free_balance(ALICE), 99990);
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
			Perbill::from_percent(0u32),
			None
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
				Perbill::from_percent(0u32),
				FungibleTokenId::NativeToken(0)
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
			Perbill::from_percent(0u32),
			None
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
				Perbill::from_percent(0u32),
				FungibleTokenId::NativeToken(0)
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
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0)
		));

		// ALICE balance is 100000 - 1 (network fee) - 6 (minting fee) = 99993
		assert_eq!(Balances::free_balance(ALICE), 99993);

		assert_noop!(
			AuctionModule::create_auction(
				AuctionType::Auction,
				ItemId::NFT(0, 0),
				None,
				ALICE,
				100,
				0,
				ListingLevel::Global,
				Perbill::from_percent(0u32),
				FungibleTokenId::NativeToken(0)
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
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0)
		));

		// ALICE balance is 100 000 - 1 (network fee) - 12 (minting fees) = 99987
		assert_eq!(Balances::free_balance(ALICE), 99987);

		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			ItemId::NFT(0, 1),
			None,
			ALICE,
			100,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0)
		));

		// ALICE balance is 99987 - 1 (network fee) = 99986
		assert_eq!(Balances::free_balance(ALICE), 99986);

		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			ItemId::NFT(0, 2),
			None,
			ALICE,
			100,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0)
		));

		// ALICE balance is 99986 - 1 (network fee) = 99985
		assert_eq!(Balances::free_balance(ALICE), 99985);

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
				Perbill::from_percent(0u32),
				FungibleTokenId::NativeToken(0)
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
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0)
		));

		// ALICE balance is 99985 - 1 (network fee) = 99984
		assert_eq!(Balances::free_balance(ALICE), 99984);
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
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0)
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
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0),
		));
		run_to_block(95);
		assert_ok!(AuctionModule::bid(bidder, 0, 200));
		assert_eq!(
			AuctionModule::auctions(0),
			Some(AuctionInfo {
				bid: Some((1, 200)),
				start: 1,
				end: Some(101),
			})
		);
		assert_eq!(last_event(), Event::AuctionModule(crate::Event::Bid(0, ALICE, 200)));
		assert_eq!(Balances::reserved_balance(ALICE), 200);
	});
}

#[test]
// Walk the happy path
fn bid_anti_snipe_duration_works() {
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
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0),
		));

		run_to_block(96);

		assert_ok!(AuctionModule::bid(bidder.clone(), 0, 200));

		assert_eq!(
			AuctionModule::auctions(0),
			Some(AuctionInfo {
				bid: Some((1, 200)),
				start: 1,
				end: Some(106),
			})
		);
		assert_eq!(AuctionModule::auction_end_time(106, 0), Some(()));
		assert_eq!(AuctionModule::auction_end_time(101, 0), None);
		assert_eq!(last_event(), Event::AuctionModule(crate::Event::Bid(0, ALICE, 200)));

		// Move to the next block, test if auction keeps extending
		run_to_block(97);
		// Ensure another bid doesn't increase the end time
		assert_ok!(AuctionModule::bid(bidder.clone(), 0, 201));
		assert_eq!(AuctionModule::auction_end_time(106, 0), Some(()));
		assert_eq!(Balances::reserved_balance(ALICE), 201);

		run_to_block(107);
		// Verify if auction finalized with new end time.
		assert_eq!(
			last_event(),
			Event::AuctionModule(crate::Event::AuctionFinalized(0, 1, 201))
		);
	});
}

#[test]
// Walk the happy path
fn bid_anti_snipe_duration_works_with_local_auction() {
	ExtBuilder::default().build().execute_with(|| {
		let owner = Origin::signed(ALICE);
		let bidder = Origin::signed(BOB);

		init_test_nft(owner.clone());

		assert_ok!(AuctionModule::create_new_auction(
			owner,
			ItemId::NFT(0, 0),
			100,
			101,
			ListingLevel::Local(ALICE_METAVERSE_ID),
			FungibleTokenId::NativeToken(0),
		));
		assert_eq!(AuctionModule::items_in_auction(ItemId::NFT(0, 0)), Some(true));

		run_to_block(96);

		assert_ok!(AuctionModule::bid(bidder.clone(), 0, 200));

		assert_eq!(
			AuctionModule::auctions(0),
			Some(AuctionInfo {
				bid: Some((BOB, 200)),
				start: 1,
				end: Some(106),
			})
		);
		assert_eq!(AuctionModule::auction_end_time(106, 0), Some(()));
		assert_eq!(AuctionModule::auction_end_time(101, 0), None);
		assert_eq!(last_event(), Event::AuctionModule(crate::Event::Bid(0, BOB, 200)));

		// Move to the next block, test if auction keeps extending
		run_to_block(97);
		// Ensure another bid doesn't increase the end time
		assert_ok!(AuctionModule::bid(bidder.clone(), 0, 201));
		assert_eq!(AuctionModule::auction_end_time(106, 0), Some(()));
		assert_eq!(Balances::reserved_balance(BOB), 201);

		let auction_item = AuctionModule::get_auction_item(0).unwrap();
		assert_eq!(auction_item.amount, 201);
		run_to_block(107);
		// Verify if auction finalized with new end time.
		assert_eq!(
			last_event(),
			Event::AuctionModule(crate::Event::AuctionFinalized(0, BOB, 201))
		);
	});
}

#[test]
// Walk the happy path
fn bid_multicurrency_works() {
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
			Perbill::from_percent(0u32),
			FungibleTokenId::MiningResource(0),
		));

		assert_ok!(AuctionModule::bid(bidder, 0, 200));
		assert_eq!(last_event(), Event::AuctionModule(crate::Event::Bid(0, ALICE, 200)));

		assert_eq!(
			Tokens::accounts(ALICE, FungibleTokenId::MiningResource(0)).reserved,
			200
		);
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
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0)
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
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0)
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
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0)
		));

		assert_ok!(AuctionModule::bid(bidder, 0, 200));
		assert_eq!(last_event(), Event::AuctionModule(crate::Event::Bid(0, ALICE, 200)));

		// BOB should have 500 - 1 (network reserve fee) - 3 minting fee = 496
		assert_eq!(Balances::free_balance(BOB), 496);

		run_to_block(102);
		// Verify asset transfers to alice after end of auction
		assert_eq!(
			last_event(),
			Event::AuctionModule(crate::Event::AuctionFinalized(0, 1, 200))
		);

		// Verify transfer of fund (minus gas)
		// BOB only receive 200 - 2 (1% royalty fee) - 2 (1% network fee) - 4 (minting fee) = 193
		// 500 + 193 = 693
		assert_eq!(Balances::free_balance(BOB), 693);
		assert_eq!(Balances::free_balance(ALICE), 99800);

		// Verify Alice has the NFT and Bob doesn't
		assert_eq!(NFTModule::<Runtime>::check_ownership(&ALICE, &(0, 0)), Ok(true));
		assert_eq!(NFTModule::<Runtime>::check_ownership(&BOB, &(0, 0)), Ok(false));
	});
}

#[test]
fn asset_transfers_after_multicurrency_auction() {
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
			Perbill::from_percent(0u32),
			FungibleTokenId::MiningResource(0)
		));

		assert_ok!(AuctionModule::bid(bidder, 0, 200));
		assert_eq!(last_event(), Event::AuctionModule(crate::Event::Bid(0, ALICE, 200)));

		// BOB should have 500 - 1 (network reserve fee) - 3 minting fee = 496
		assert_eq!(Balances::free_balance(BOB), 496);

		run_to_block(102);
		// Verify asset transfers to alice after end of auction
		assert_eq!(
			last_event(),
			Event::AuctionModule(crate::Event::AuctionFinalized(0, 1, 200))
		);

		// Verify transfer of fund
		assert_eq!(Tokens::accounts(BOB, FungibleTokenId::MiningResource(0)).free, 5196);
		assert_eq!(Tokens::accounts(ALICE, FungibleTokenId::MiningResource(0)).free, 9800);

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
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0)
		));

		System::set_block_number(101);

		assert_noop!(AuctionModule::bid(bidder, 0, 200), Error::<Runtime>::AuctionIsExpired);

		assert_eq!(Balances::free_balance(ALICE), 100000);
	});
}

#[test]
// Buy now should work
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
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0)
		));

		//assert_eq!(Balances::free_balance(BOB), 499);

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
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0)
		));

		assert_ok!(AuctionModule::buy_now(buyer.clone(), 1, 200));

		assert_eq!(AuctionModule::auctions(0), None);
		// check account received asset
		assert_eq!(NFTModule::<Runtime>::check_ownership(&ALICE, &(0, 0)), Ok(true));
		assert_eq!(NFTModule::<Runtime>::check_ownership(&ALICE, &(0, 1)), Ok(true));

		// check balances were transferred
		assert_eq!(Balances::free_balance(ALICE), 99600);
		// initial balance is 500 - sold 2 x 200 = 900
		// royalty fee 1% for both sales is 8
		// network fee 1% for both sales is 8
		// 900 - 16 + 7 for deposit minting = 885
		assert_eq!(Balances::free_balance(BOB), 888);

		// event was triggered
		let event = mock::Event::AuctionModule(crate::Event::BuyNowFinalised(1, ALICE, 200));
		assert_eq!(last_event(), event);

		// check of auction item is still valid
		assert_eq!(AuctionItems::<Runtime>::get(1), None);
		// Check that auction is over
		assert_noop!(
			AuctionModule::buy_now(buyer.clone(), 1, 150),
			Error::<Runtime>::AuctionDoesNotExist
		);
	});
}

#[test]
// Buy now should work
fn multicurrency_buy_now_work() {
	ExtBuilder::default().build().execute_with(|| {
		let owner = Origin::signed(BOB);
		let buyer = Origin::signed(ALICE);

		init_test_nft(owner.clone());

		assert_ok!(AuctionModule::create_auction(
			AuctionType::BuyNow,
			ItemId::NFT(0, 0),
			None,
			BOB,
			200,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32),
			FungibleTokenId::MiningResource(0)
		));

		assert_ok!(AuctionModule::buy_now(buyer.clone(), 0, 200));

		assert_eq!(AuctionModule::auctions(0), None);
		// check account received asset
		assert_eq!(NFTModule::<Runtime>::check_ownership(&ALICE, &(0, 0)), Ok(true));

		// check balances were transferred
		assert_eq!(Tokens::accounts(BOB, FungibleTokenId::MiningResource(0)).free, 5196);
		assert_eq!(Tokens::accounts(ALICE, FungibleTokenId::MiningResource(0)).free, 9800);

		// event was triggered
		let event = mock::Event::AuctionModule(crate::Event::BuyNowFinalised(0, ALICE, 200));
		assert_eq!(last_event(), event);

		// check of auction item is still valid
		assert_eq!(AuctionItems::<Runtime>::get(0), None);
		// Check that auction is over
		assert_noop!(
			AuctionModule::buy_now(buyer.clone(), 0, 150),
			Error::<Runtime>::AuctionDoesNotExist
		);
	});
}

#[test]
// Test if buying now bundle should work
fn buy_now_with_bundle_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let owner = Origin::signed(BOB);
		let buyer = Origin::signed(ALICE);

		// create 3 nfts
		init_test_nft(owner.clone());
		init_test_nft(owner.clone());
		init_test_nft(owner.clone());

		// call create_auction
		assert_ok!(AuctionModule::create_auction(
			AuctionType::BuyNow,
			ItemId::Bundle(vec![(0, 0, 60), (0, 1, 70), (0, 2, 70)]),
			None,
			BOB,
			200,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0)
		));

		// BOB Balance is 500 - 1 (network reserve fee) - 9 (minting fee) = 490
		assert_eq!(Balances::free_balance(BOB), 490);

		// buy now successful
		assert_ok!(AuctionModule::buy_now(buyer.clone(), 0, 200));

		assert_eq!(AuctionModule::auctions(0), None);
		// check account received asset
		assert_eq!(NFTModule::<Runtime>::check_ownership(&ALICE, &(0, 0)), Ok(true));
		assert_eq!(NFTModule::<Runtime>::check_ownership(&ALICE, &(0, 1)), Ok(true));
		assert_eq!(NFTModule::<Runtime>::check_ownership(&ALICE, &(0, 2)), Ok(true));

		// check balances were transferred
		assert_eq!(Balances::free_balance(ALICE), 99800);

		// initial BOB balance is 500
		// bundle buy now price is 200
		// 200 - 2 (1% royalty_fee) - 2 (1% network fee) - 10 (minting fee) = 186
		// 500 + 186 = 686
		assert_eq!(Balances::free_balance(BOB), 686);

		// event was triggered
		let event = mock::Event::AuctionModule(crate::Event::BuyNowFinalised(0, ALICE, 200));
		assert_eq!(last_event(), event);

		// Check that auction is over
		assert_noop!(
			AuctionModule::buy_now(buyer.clone(), 1, 150),
			Error::<Runtime>::AuctionDoesNotExist
		);
	});
}

#[test]
// Test if buying now bundle should work
fn multicurrency_buy_now_with_bundle_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let owner = Origin::signed(BOB);
		let buyer = Origin::signed(ALICE);

		// create 3 nfts
		init_test_nft(owner.clone());
		init_test_nft(owner.clone());
		init_test_nft(owner.clone());

		// call create_auction
		assert_ok!(AuctionModule::create_auction(
			AuctionType::BuyNow,
			ItemId::Bundle(vec![(0, 0, 60), (0, 1, 70), (0, 2, 70)]),
			None,
			BOB,
			200,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32),
			FungibleTokenId::MiningResource(0)
		));

		// BOB Balance is 500 - 1 (network reserve fee) - 9 (minting fee) = 490
		assert_eq!(Balances::free_balance(BOB), 490);

		// buy now successful
		assert_ok!(AuctionModule::buy_now(buyer.clone(), 0, 200));

		assert_eq!(AuctionModule::auctions(0), None);
		// check account received asset
		assert_eq!(NFTModule::<Runtime>::check_ownership(&ALICE, &(0, 0)), Ok(true));
		assert_eq!(NFTModule::<Runtime>::check_ownership(&ALICE, &(0, 1)), Ok(true));
		assert_eq!(NFTModule::<Runtime>::check_ownership(&ALICE, &(0, 2)), Ok(true));

		// check balances were transferred
		assert_eq!(Tokens::accounts(BOB, FungibleTokenId::MiningResource(0)).free, 5195);
		assert_eq!(Tokens::accounts(ALICE, FungibleTokenId::MiningResource(0)).free, 9800);

		// event was triggered
		let event = mock::Event::AuctionModule(crate::Event::BuyNowFinalised(0, ALICE, 200));
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
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0)
		));

		// BOB balance is 500 - 1 (network fee) - 3 (minting fee) = 496
		assert_eq!(Balances::free_balance(BOB), 496);

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
		assert_ok!(Balances::reserve(&ALICE, 99996));
		assert_noop!(
			AuctionModule::buy_now(buyer.clone(), 0, 150),
			Error::<Runtime>::InsufficientFreeBalance
		);
		assert_eq!(Balances::unreserve(&ALICE, 99996), 0);
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

		// BOB balance is 500 + 150 - 1 (royalty fee) - 1 (network fee) - 3 (minting fee) = 645
		assert_eq!(Balances::free_balance(BOB), 645);

		assert_noop!(
			AuctionModule::create_auction(
				AuctionType::BuyNow,
				ItemId::NFT(0, 0),
				None,
				BOB,
				150,
				0,
				ListingLevel::Global,
				Perbill::from_percent(0u32),
				FungibleTokenId::NativeToken(0)
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
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0)
		));
		assert_noop!(
			AuctionModule::bid(participant.clone(), 0, 200),
			Error::<Runtime>::InvalidAuctionType
		);

		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			ItemId::NFT(0, 1),
			None,
			BOB,
			150,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0)
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
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0)
		));
		assert_eq!(AuctionModule::items_in_auction(ItemId::NFT(0, 0)), Some(true));
		assert_ok!(AuctionModule::bid(bidder, 0, 100));
		// BOB's should have 500 - 1 (network reserve fee) - 3 (minting fee) = 496
		assert_eq!(Balances::free_balance(BOB), 496);
		run_to_block(102);
		assert_eq!(AuctionModule::auctions(0), None);
		// check account received asset
		assert_eq!(NFTModule::<Runtime>::check_ownership(&ALICE, &(0, 0)), Ok(true));
		// check balances were transferred
		assert_eq!(Balances::free_balance(ALICE), 99900);
		// BOB's initial balance is 500
		// 100 - 1 (1% of 100 as royalty fee) - 1 (1% of 100 as network fee) - 3 minting fee = 96
		// 500 + 95 = 595
		assert_eq!(Balances::free_balance(BOB), 595);
		// asset is not longer in auction
		assert_eq!(AuctionModule::items_in_auction(ItemId::NFT(0, 0)), None);
		// auction item should not in storage
		assert_eq!(AuctionItems::<Runtime>::get(0), None);
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
			Perbill::from_percent(10u32),
			FungibleTokenId::NativeToken(0)
		));
		assert_eq!(AuctionModule::items_in_auction(ItemId::NFT(0, 0)), Some(true));
		assert_ok!(AuctionModule::bid(bidder, 0, 100));
		// Free balance of Alice is 99997 - 1 = 99996
		assert_eq!(Balances::free_balance(ALICE), 99996);
		run_to_block(102);
		assert_eq!(AuctionModule::auctions(0), None);
		// check account received asset
		assert_eq!(NFTModule::<Runtime>::check_ownership(&BOB, &(0, 0)), Ok(true));
		// check balances were transferred
		// Bob bid 100 for item, his new balance will be 500 - 100
		assert_eq!(Balances::free_balance(BOB), 400);
		// Alice only receive 88 for item sold:
		// cost breakdown 100 - 10 (listing fee) - 1 (1% network fee) - 1 (1% royalty fee)
		// Free balance of Alice is 99997 + 88 = 100085
		assert_eq!(Balances::free_balance(ALICE), 100085);
		// asset is not longer in auction
		assert_eq!(AuctionModule::items_in_auction(ItemId::NFT(0, 0)), None);
		// event was triggered
		let event = mock::Event::AuctionModule(crate::Event::AuctionFinalized(0, BOB, 100));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn auction_bundle_should_update_new_price_according_new_bid() {
	ExtBuilder::default().build().execute_with(|| {
		let owner = Origin::signed(ALICE);
		let bidder = Origin::signed(BOB);
		init_test_nft(owner.clone());
		init_test_nft(owner.clone());

		// After minting new NFTs, it costs 6 unit
		assert_eq!(Balances::free_balance(ALICE), 99994);

		let tokens = vec![(0, 0, 100), (0, 1, 100)];
		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			ItemId::Bundle(tokens.clone()),
			None,
			ALICE,
			200,
			0,
			ListingLevel::Local(ALICE_METAVERSE_ID),
			Perbill::from_percent(10u32),
			FungibleTokenId::NativeToken(0)
		));
		assert_eq!(
			AuctionModule::items_in_auction(ItemId::Bundle(tokens.clone())),
			Some(true)
		);
		assert_ok!(AuctionModule::bid(bidder, 0, 300));
		// Free balance of Alice is 99994 - 1 (network reserve fee)
		assert_eq!(Balances::free_balance(ALICE), 99993);

		let tokens_after_bid = vec![(0, 0, 150), (0, 1, 150)];
		let item_updated_after_bid = AuctionModule::items_in_auction(ItemId::Bundle(tokens.clone()));
		let auction_item = AuctionModule::get_auction_item(0).unwrap();

		assert_eq!(auction_item.item_id, ItemId::Bundle(tokens_after_bid));
	})
}

#[test]
// Auction finalize with bundle and listing fee works
fn on_finalize_with_bundle_with_listing_fee_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let owner = Origin::signed(ALICE);
		let bidder = Origin::signed(BOB);
		init_test_nft(owner.clone());
		init_test_nft(owner.clone());

		// After minting new NFTs, it costs 6 unit
		assert_eq!(Balances::free_balance(ALICE), 99994);

		let tokens = vec![(0, 0, 80), (0, 1, 120)];
		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			ItemId::Bundle(tokens.clone()),
			None,
			ALICE,
			200,
			0,
			ListingLevel::Local(ALICE_METAVERSE_ID),
			Perbill::from_percent(10u32),
			FungibleTokenId::NativeToken(0)
		));
		assert_eq!(
			AuctionModule::items_in_auction(ItemId::Bundle(tokens.clone())),
			Some(true)
		);
		assert_ok!(AuctionModule::bid(bidder, 0, 400));
		// asset is not longer in auction
		assert_eq!(AuctionModule::items_in_auction(ItemId::Bundle(tokens.clone())), None);
		// 400 will split to 2 nfts according to percentage
		let updated_token_with_price = vec![(0, 0, 160), (0, 1, 240)];
		// check latest updated item_id
		assert_eq!(
			AuctionModule::items_in_auction(ItemId::Bundle(updated_token_with_price)),
			Some(true)
		);

		// Free balance of Alice is 99994 - 1 (network reserve fee)
		assert_eq!(Balances::free_balance(ALICE), 99993);
		run_to_block(102);
		assert_eq!(AuctionModule::auctions(0), None);
		// check account received asset
		assert_eq!(NFTModule::<Runtime>::check_ownership(&BOB, &(0, 0)), Ok(true));
		assert_eq!(NFTModule::<Runtime>::check_ownership(&BOB, &(0, 1)), Ok(true));
		// check balances were transferred
		// Bob bid 400 for item, his new balance will be 500 - 400
		assert_eq!(Balances::free_balance(BOB), 100);
		// Alice only receive 176 for item solds
		// Cost breakdown 400 - 4 (royalty) - 4 (1% network fee) - 40 (listing fee) = 352
		// Free balance of Alice is 99994 + 352 = 100346
		assert_eq!(Balances::free_balance(ALICE), 100346);
		// event was triggered
		let event = mock::Event::AuctionModule(crate::Event::AuctionFinalized(0, BOB, 400));
		assert_eq!(last_event(), event);
	});
}

#[test]
// Auction finalize with bundle and listing fee works
fn on_finalize_with_undeployed_land_block_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let bidder = Origin::signed(BOB);
		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			ItemId::UndeployedLandBlock(UNDEPLOYED_LAND_BLOCK_ID_EXIST),
			None,
			ALICE,
			200,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0)
		));
		assert_eq!(
			AuctionModule::items_in_auction(ItemId::UndeployedLandBlock(UNDEPLOYED_LAND_BLOCK_ID_EXIST)),
			Some(true)
		);
		assert_ok!(AuctionModule::bid(bidder, 0, 200));
		run_to_block(102);
		assert_eq!(AuctionModule::auctions(0), None);
		// event was triggered
		let event = mock::Event::AuctionModule(crate::Event::AuctionFinalized(0, BOB, 200));
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
			FungibleTokenId::NativeToken(0),
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
			FungibleTokenId::NativeToken(0),
		));
		assert_eq!(AuctionModule::items_in_auction(ItemId::NFT(0, 0)), Some(true))
	});
}

#[test]
// Creating auction for undeployed land block should work
fn create_new_auction_for_undeployed_land_block_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			ItemId::UndeployedLandBlock(UNDEPLOYED_LAND_BLOCK_ID_EXIST),
			None,
			ALICE,
			100,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0),
		));

		assert_eq!(
			AuctionModule::auctions(0),
			Some(AuctionInfo {
				bid: None,
				start: 1,
				end: Some(101),
			})
		);

		assert_eq!(
			AuctionModule::items_in_auction(ItemId::UndeployedLandBlock(UNDEPLOYED_LAND_BLOCK_ID_EXIST)),
			Some(true)
		);
	});
}

#[test]
// Bidding for undeployed land block should work
fn bidding_for_undeployed_land_block_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			ItemId::UndeployedLandBlock(UNDEPLOYED_LAND_BLOCK_ID_EXIST),
			None,
			ALICE,
			100,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0),
		));

		assert_eq!(
			AuctionModule::auctions(0),
			Some(AuctionInfo {
				bid: None,
				start: 1,
				end: Some(101),
			})
		);

		assert_eq!(
			AuctionModule::items_in_auction(ItemId::UndeployedLandBlock(UNDEPLOYED_LAND_BLOCK_ID_EXIST)),
			Some(true)
		);

		// bidding should work
		assert_ok!(AuctionModule::bid(Origin::signed(BOB.clone()), 0, 200));

		// bidding should fail if not metaverse owner
		assert_noop!(
			AuctionModule::bid(Origin::signed(NO_METAVERSE_OWNER.clone()), 0, 300),
			Error::<Runtime>::MetaverseOwnerOnly
		);
	});
}

#[test]
// Creating buy now for undeployed land block should work
fn create_buy_now_for_undeployed_land_block_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(AuctionModule::create_auction(
			AuctionType::BuyNow,
			ItemId::UndeployedLandBlock(UNDEPLOYED_LAND_BLOCK_ID_EXIST),
			None,
			ALICE,
			100,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0),
		));

		assert_eq!(
			AuctionModule::auctions(0),
			Some(AuctionInfo {
				bid: None,
				start: 1,
				end: Some(101),
			})
		);

		assert_eq!(
			AuctionModule::items_in_auction(ItemId::UndeployedLandBlock(UNDEPLOYED_LAND_BLOCK_ID_EXIST)),
			Some(true)
		);
	});
}

#[test]
// Buy now for undeployed land block should fail if not metaverse owner
fn buy_now_for_undeployed_land_block_should_fail_if_not_metaverse_owner() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(AuctionModule::create_auction(
			AuctionType::BuyNow,
			ItemId::UndeployedLandBlock(UNDEPLOYED_LAND_BLOCK_ID_EXIST),
			None,
			ALICE,
			100,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0),
		));

		assert_eq!(
			AuctionModule::auctions(0),
			Some(AuctionInfo {
				bid: None,
				start: 1,
				end: Some(101),
			})
		);

		assert_eq!(
			AuctionModule::items_in_auction(ItemId::UndeployedLandBlock(UNDEPLOYED_LAND_BLOCK_ID_EXIST)),
			Some(true)
		);

		// bidding should fail if not metaverse owner
		assert_noop!(
			AuctionModule::buy_now(Origin::signed(NO_METAVERSE_OWNER.clone()), 0, 100),
			Error::<Runtime>::MetaverseOwnerOnly
		);
	});
}

#[test]
// Making offer for a NFT should fail
fn make_offer_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		init_test_nft(Origin::signed(ALICE));
		assert_noop!(
			AuctionModule::make_offer(Origin::signed(ALICE), (0, 0), 150),
			Error::<Runtime>::NoPermissionToMakeOffer
		);

		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			ItemId::NFT(0, 0),
			None,
			ALICE,
			100,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0),
		));
		assert_noop!(
			AuctionModule::make_offer(Origin::signed(BOB), (0, 0), 150),
			Error::<Runtime>::NoPermissionToMakeOffer
		);

		init_test_nft(Origin::signed(ALICE));
		assert_ok!(AuctionModule::make_offer(Origin::signed(BOB), (0, 1), 150));
		assert_noop!(
			AuctionModule::make_offer(Origin::signed(BOB), (0, 1), 150),
			Error::<Runtime>::OfferAlreadyExists
		);
		assert_eq!(Balances::free_balance(BOB), 350);
	});
}

#[test]
// Making offer for a NFT should work
fn make_offer_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		init_test_nft(Origin::signed(ALICE));
		assert_ok!(AuctionModule::make_offer(Origin::signed(BOB), (0, 0), 150));

		let event = mock::Event::AuctionModule(crate::Event::NftOfferMade(0, 0, BOB, 150));
		assert_eq!(last_event(), event);

		let offer = NftOffer {
			amount: 150,
			end_block: 11,
		};
		assert_eq!(Offers::<Runtime>::get((0, 0), BOB), Some(offer));

		assert_eq!(Balances::free_balance(BOB), 350);
	});
}

#[test]
// Accepting offer for a NFT should fail
fn accept_offer_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		init_test_nft(Origin::signed(ALICE));
		assert_ok!(AuctionModule::make_offer(Origin::signed(BOB), (0, 0), 150));
		assert_eq!(Balances::free_balance(BOB), 350);
		assert_noop!(
			AuctionModule::accept_offer(Origin::signed(BOB), (0, 0), BOB),
			Error::<Runtime>::NoPermissionToAcceptOffer
		);

		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			ItemId::NFT(0, 0),
			None,
			ALICE,
			100,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0),
		));
		assert_noop!(
			AuctionModule::accept_offer(Origin::signed(ALICE), (0, 0), BOB),
			Error::<Runtime>::NoPermissionToAcceptOffer
		);

		init_test_nft(Origin::signed(ALICE));
		assert_ok!(AuctionModule::make_offer(Origin::signed(BOB), (0, 1), 150));
		run_to_block(100);
		assert_noop!(
			AuctionModule::accept_offer(Origin::signed(ALICE), (0, 1), BOB),
			Error::<Runtime>::OfferIsExpired
		);
	});
}

#[test]
// Accepting offer for a NFT should work
fn accept_offer_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		init_test_nft(Origin::signed(ALICE));
		assert_ok!(AuctionModule::make_offer(Origin::signed(BOB), (0, 0), 150));
		assert_eq!(Balances::free_balance(BOB), 350);
		assert_ok!(AuctionModule::accept_offer(Origin::signed(ALICE), (0, 0), BOB));

		let event = mock::Event::AuctionModule(crate::Event::NftOfferAccepted(0, 0, BOB));
		assert_eq!(last_event(), event);

		assert_eq!(Offers::<Runtime>::get((0, 0), BOB), None);
		assert_eq!(Balances::free_balance(BOB), 350);
		assert_eq!(Balances::free_balance(ALICE), 100147);
	});
}

#[test]
// Withdrawing offer for a NFT should work
fn withdraw_offer_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		init_test_nft(Origin::signed(ALICE));
		assert_noop!(
			AuctionModule::withdraw_offer(Origin::signed(BOB), (0, 0)),
			Error::<Runtime>::OfferDoesNotExist
		);
		assert_eq!(Balances::free_balance(BOB), 500);
	});
}

#[test]
// Withdrawing offer for a NFT should work
fn withdraw_offer_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		init_test_nft(Origin::signed(ALICE));
		assert_ok!(AuctionModule::make_offer(Origin::signed(BOB), (0, 0), 150));

		assert_eq!(Balances::free_balance(BOB), 350);
		assert_ok!(AuctionModule::withdraw_offer(Origin::signed(BOB), (0, 0)));

		let event = mock::Event::AuctionModule(crate::Event::NftOfferWithdrawn(0, 0, BOB));
		assert_eq!(last_event(), event);

		assert_eq!(Offers::<Runtime>::get((0, 0), BOB), None);
		assert_eq!(Balances::free_balance(BOB), 500);
	});
}

#[test]
fn finalize_auction_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let owner = Origin::signed(BOB);
		let bidder = Origin::signed(ALICE);
		init_test_nft(owner.clone());
		init_test_nft(owner.clone());
		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			ItemId::NFT(0, 0),
			None,
			BOB,
			100,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0)
		));

		run_to_block(10);

		assert_noop!(
			AuctionModule::finalize_auction(Origin::signed(BOB), 100),
			Error::<Runtime>::AuctionDoesNotExist
		);

		assert_noop!(
			AuctionModule::finalize_auction(Origin::signed(BOB), 0),
			Error::<Runtime>::AuctionIsNotExpired
		);

		run_to_block(102);

		assert_noop!(
			AuctionModule::finalize_auction(Origin::signed(BOB), 0),
			Error::<Runtime>::AuctionDoesNotExist
		);
	});
}

#[test]
fn cancel_listing_should_work() {
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
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0)
		));
		assert_eq!(Balances::free_balance(BOB), 496);
		assert_eq!(AuctionModule::items_in_auction(ItemId::NFT(0, 0)), Some(true));
		assert_eq!(
			AuctionModule::auctions(0),
			Some(AuctionInfo {
				bid: None,
				start: 1,
				end: Some(101),
			})
		);

		run_to_block(2);
		assert_ok!(AuctionModule::cancel_listing(Origin::signed(BOB), 0));
		assert_eq!(Balances::free_balance(BOB), 497);

		assert_eq!(AuctionModule::items_in_auction(ItemId::NFT(0, 0)), None);
		assert_eq!(AuctionModule::auctions(0), None);

		let event = mock::Event::AuctionModule(crate::Event::AuctionFinalizedNoBid(0));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn cancel_listing_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let owner = Origin::signed(BOB);
		let bidder = Origin::signed(ALICE);
		init_test_nft(owner.clone());
		init_test_nft(owner.clone());
		assert_ok!(AuctionModule::create_auction(
			AuctionType::Auction,
			ItemId::NFT(0, 0),
			None,
			BOB,
			100,
			0,
			ListingLevel::Global,
			Perbill::from_percent(0u32),
			FungibleTokenId::NativeToken(0)
		));

		run_to_block(10);

		assert_noop!(
			AuctionModule::cancel_listing(Origin::signed(ALICE), 0),
			Error::<Runtime>::NoPermissionToCancelAuction
		);

		assert_noop!(
			AuctionModule::cancel_listing(Origin::signed(BOB), 1),
			Error::<Runtime>::AuctionDoesNotExist
		);
	});
}
