#[cfg(feature = "with-pioneer-runtime")]
use crate::relaychain::kusama_test_net::*;
use crate::setup::*;
use auction_manager::ListingLevel;
use core_primitives::{
	Attributes, CollectionType, MetaverseLandTrait, MetaverseTrait, NFTTrait, TokenType, UndeployedLandBlocksTrait,
};
use core_traits::{FungibleTokenId, ItemId, UndeployedLandBlockType};
use frame_system::RawOrigin;
use sp_runtime::Perbill;

#[test]
fn deploy_land_blocks_won_in_an_auction() {
	#[cfg(feature = "with-pioneer-runtime")]
	const NATIVE_TOKEN: FungibleTokenId = FungibleTokenId::NativeToken(0);

	ExtBuilder::default()
		.balances(vec![
			(AccountId::from(ALICE), NATIVE_TOKEN, 1_000 * dollar(NATIVE_TOKEN)),
			(AccountId::from(BOB), NATIVE_TOKEN, 1_000 * dollar(NATIVE_TOKEN)),
			(AccountId::from(CHARLIE), NATIVE_TOKEN, 1_000 * dollar(NATIVE_TOKEN)),
		])
		.build()
		.execute_with(|| {
			let metadata = vec![1];
			assert_eq!(
				Balances::free_balance(AccountId::from(ALICE)),
				1_000 * dollar(NATIVE_TOKEN)
			);
			assert_eq!(
				Balances::free_balance(AccountId::from(BOB)),
				1_000 * dollar(NATIVE_TOKEN)
			);
			assert_eq!(
				Balances::free_balance(AccountId::from(CHARLIE)),
				1_000 * dollar(NATIVE_TOKEN)
			);
			// Create metaverse land/estate group
			assert_ok!(Nft::create_group(RawOrigin::Root.into(), vec![1], vec![1]));
			// Create metaverses
			assert_ok!(Metaverse::create_metaverse(
				RawOrigin::Signed(AccountId::from(ALICE)).into(),
				vec![1u8]
			));
			assert_ok!(Metaverse::create_metaverse(
				RawOrigin::Signed(AccountId::from(BOB)).into(),
				vec![1u8]
			));
			// Check metaverses ownership
			assert_eq!(Metaverse::check_ownership(&AccountId::from(ALICE), &0u32.into()), true);
			assert_eq!(Metaverse::check_ownership(&AccountId::from(BOB), &1u32.into()), true);
			// Create undeployed land block
			assert_ok!(Estate::issue_undeployed_land_blocks(
				RawOrigin::Root.into(),
				AccountId::from(ALICE),
				1u32,
				1u32,
				UndeployedLandBlockType::Transferable
			));
			// Check undeployed land block ownership
			assert_eq!(
				Estate::check_undeployed_land_block(&AccountId::from(ALICE), 0u32.into()),
				Ok(true)
			);
			run_to_block(1);
			// List land block on an auction
			assert_ok!(Auction::create_new_auction(
				RawOrigin::Signed(AccountId::from(ALICE)).into(),
				ItemId::UndeployedLandBlock(0u32.into()),
				100 * dollar(NATIVE_TOKEN),
				31u32.into(),
				ListingLevel::Global
			));
			run_to_block(2);
			// Bid
			assert_ok!(Auction::bid(
				RawOrigin::Signed(AccountId::from(CHARLIE)).into(),
				0u32.into(),
				101 * dollar(NATIVE_TOKEN),
			));
			run_to_block(3);
			// Outbid
			assert_ok!(Auction::bid(
				RawOrigin::Signed(AccountId::from(BOB)).into(),
				0u32.into(),
				105 * dollar(NATIVE_TOKEN),
			));
			run_to_block(35);
			// Check land block ownership and balances
			assert_eq!(
				Estate::check_undeployed_land_block(&AccountId::from(BOB), 0u32.into()),
				Ok(true)
			);
			assert_eq!(Balances::free_balance(AccountId::from(BOB)), 894 * dollar(NATIVE_TOKEN));
			assert_eq!(
				Balances::free_balance(AccountId::from(CHARLIE)),
				1000 * dollar(NATIVE_TOKEN)
			);
			assert_eq!(Balances::free_balance(AccountId::from(ALICE)), 1102950000000000000000);
			// Deploy undeployed land block
			assert_ok!(Estate::deploy_land_block(
				RawOrigin::Signed(AccountId::from(BOB)).into(),
				0u32.into(),
				1u32.into(),
				(0i32, 0i32),
				vec![(0i32, 0i32)]
			));
			// Check if land unit was deployed and its owner is correct
			assert_eq!(Estate::check_landunit(1u32.into(), (0i32, 0i32)), Ok(true));
			assert_eq!(
				Nft::check_ownership(&AccountId::from(BOB), &(2u32.into(), 0u32.into())),
				Ok(true)
			);
		});
}

#[test]
fn create_estate_from_raw_land_blocks() {
	#[cfg(feature = "with-pioneer-runtime")]
	const NATIVE_TOKEN: FungibleTokenId = FungibleTokenId::NativeToken(0);

	ExtBuilder::default()
		.balances(vec![
			(AccountId::from(ALICE), NATIVE_TOKEN, 1_000 * dollar(NATIVE_TOKEN)),
			(AccountId::from(BOB), NATIVE_TOKEN, 1_000 * dollar(NATIVE_TOKEN)),
		])
		.build()
		.execute_with(|| {
			let metadata = vec![1];
			assert_eq!(
				Balances::free_balance(AccountId::from(ALICE)),
				1_000 * dollar(NATIVE_TOKEN)
			);
			assert_eq!(
				Balances::free_balance(AccountId::from(BOB)),
				1_000 * dollar(NATIVE_TOKEN)
			);
			// Create metaverse land/estate group
			assert_ok!(Nft::create_group(RawOrigin::Root.into(), vec![1], vec![1]));
			// Create metaverses
			assert_ok!(Metaverse::create_metaverse(
				RawOrigin::Signed(AccountId::from(ALICE)).into(),
				vec![1u8]
			));
			assert_ok!(Metaverse::create_metaverse(
				RawOrigin::Signed(AccountId::from(BOB)).into(),
				vec![1u8]
			));
			// Check metaverse ownership
			assert_eq!(Metaverse::check_ownership(&AccountId::from(ALICE), &0u32.into()), true);
			// Create undeployed land block
			assert_ok!(Estate::issue_undeployed_land_blocks(
				RawOrigin::Root.into(),
				AccountId::from(ALICE),
				1u32,
				2u32,
				UndeployedLandBlockType::Transferable
			));
			// Deploy undeployed land block
			assert_ok!(Estate::deploy_land_block(
				RawOrigin::Signed(AccountId::from(ALICE)).into(),
				0u32.into(),
				0u32.into(),
				(0i32, 0i32),
				vec![(0i32, 0i32), (0i32, 1i32)]
			));
			// Check if land units were deployed
			assert_eq!(Estate::check_landunit(0u32.into(), (0i32, 0i32)), Ok(true));
			assert_eq!(Estate::check_landunit(0u32.into(), (0i32, 1i32)), Ok(true));
			// Check land units ownership
			assert_eq!(
				Nft::check_ownership(&AccountId::from(ALICE), &(0u32.into(), 0u32.into())),
				Ok(true)
			);
			assert_eq!(
				Nft::check_ownership(&AccountId::from(ALICE), &(0u32.into(), 1u32.into())),
				Ok(true)
			);
			run_to_block(1);
			// Create buy now for each land unit
			assert_ok!(Auction::create_new_buy_now(
				RawOrigin::Signed(AccountId::from(ALICE)).into(),
				ItemId::NFT(0u32.into(), 0u64.into()),
				100 * dollar(NATIVE_TOKEN),
				400u32.into(),
				ListingLevel::Local(0u32.into())
			));
			assert_ok!(Auction::create_new_buy_now(
				RawOrigin::Signed(AccountId::from(ALICE)).into(),
				ItemId::NFT(0u32.into(), 1u64.into()),
				100 * dollar(NATIVE_TOKEN),
				400u32.into(),
				ListingLevel::Local(0u32.into())
			));
			run_to_block(2);
			// Buy land units
			assert_ok!(Auction::buy_now(
				RawOrigin::Signed(AccountId::from(BOB)).into(),
				0u32.into(),
				100 * dollar(NATIVE_TOKEN),
			));
			assert_ok!(Auction::buy_now(
				RawOrigin::Signed(AccountId::from(BOB)).into(),
				1u32.into(),
				100 * dollar(NATIVE_TOKEN),
			));
			// Check updated balances and land units ownership
			assert_eq!(
				Nft::check_ownership(&AccountId::from(BOB), &(0u32.into(), 0u32.into())),
				Ok(true)
			);
			assert_eq!(
				Nft::check_ownership(&AccountId::from(BOB), &(0u32.into(), 1u32.into())),
				Ok(true)
			);
			assert_eq!(Balances::free_balance(AccountId::from(BOB)), 750 * dollar(NATIVE_TOKEN));
			assert_eq!(Balances::free_balance(AccountId::from(ALICE)), 1116000000000000000000);
			// Create estate using the purchased land blocks
			assert_ok!(Estate::create_estate(
				RawOrigin::Signed(AccountId::from(BOB)).into(),
				0u32.into(),
				vec![(0i32, 0i32), (0i32, 1i32)]
			));
			// Check estate ownership
			assert_eq!(
				Nft::check_ownership(&AccountId::from(BOB), &(1u32.into(), 0u32.into())),
				Ok(true)
			);
			assert_ne!(
				Nft::check_ownership(&AccountId::from(BOB), &(0u32.into(), 0u32.into())),
				Ok(true)
			);
			assert_ne!(
				Nft::check_ownership(&AccountId::from(BOB), &(0u32.into(), 1u32.into())),
				Ok(true)
			);
		});
}

#[test]
fn purchase_estate_and_modify_its_structure() {
	#[cfg(feature = "with-pioneer-runtime")]
	const NATIVE_TOKEN: FungibleTokenId = FungibleTokenId::NativeToken(0);

	ExtBuilder::default()
		.balances(vec![
			(AccountId::from(ALICE), NATIVE_TOKEN, 1_000 * dollar(NATIVE_TOKEN)),
			(AccountId::from(BOB), NATIVE_TOKEN, 1_000 * dollar(NATIVE_TOKEN)),
		])
		.build()
		.execute_with(|| {
			let metadata = vec![1];
			assert_eq!(
				Balances::free_balance(AccountId::from(ALICE)),
				1_000 * dollar(NATIVE_TOKEN)
			);
			assert_eq!(
				Balances::free_balance(AccountId::from(BOB)),
				1_000 * dollar(NATIVE_TOKEN)
			);
			// Create metaverse land/estate group
			assert_ok!(Nft::create_group(RawOrigin::Root.into(), vec![1], vec![1]));
			// Create metaverses
			assert_ok!(Metaverse::create_metaverse(
				RawOrigin::Signed(AccountId::from(ALICE)).into(),
				vec![1u8]
			));
			assert_ok!(Metaverse::create_metaverse(
				RawOrigin::Signed(AccountId::from(BOB)).into(),
				vec![1u8]
			));
			// Check metaverse ownership
			assert_eq!(Metaverse::check_ownership(&AccountId::from(ALICE), &0u32.into()), true);
			// Create undeployed land block
			assert_ok!(Estate::issue_undeployed_land_blocks(
				RawOrigin::Root.into(),
				AccountId::from(ALICE),
				1u32,
				3u32,
				UndeployedLandBlockType::Transferable
			));
			// Deploy undeployed land block
			assert_ok!(Estate::deploy_land_block(
				RawOrigin::Signed(AccountId::from(ALICE)).into(),
				0u32.into(),
				0u32.into(),
				(0i32, 0i32),
				vec![(0i32, 0i32), (0i32, 1i32), (1i32, 0i32)]
			));
			// Check if land units were deployed
			assert_eq!(Estate::check_landunit(0u32.into(), (0i32, 0i32)), Ok(true));
			assert_eq!(Estate::check_landunit(0u32.into(), (0i32, 1i32)), Ok(true));
			assert_eq!(Estate::check_landunit(0u32.into(), (1i32, 0i32)), Ok(true));
			// Check land units ownership
			assert_eq!(
				Nft::check_ownership(&AccountId::from(ALICE), &(0u32.into(), 0u32.into())),
				Ok(true)
			);
			assert_eq!(
				Nft::check_ownership(&AccountId::from(ALICE), &(0u32.into(), 1u32.into())),
				Ok(true)
			);
			assert_eq!(
				Nft::check_ownership(&AccountId::from(ALICE), &(0u32.into(), 2u32.into())),
				Ok(true)
			);
			// Create estate using 2 land blocks
			assert_ok!(Estate::create_estate(
				RawOrigin::Signed(AccountId::from(ALICE)).into(),
				0u32.into(),
				vec![(0i32, 0i32), (0i32, 1i32)]
			));
			// Check estate and left land unit ownership
			assert_eq!(
				Nft::check_ownership(&AccountId::from(ALICE), &(1u32.into(), 0u32.into())),
				Ok(true)
			);
			assert_eq!(
				Nft::check_ownership(&AccountId::from(ALICE), &(0u32.into(), 2u32.into())),
				Ok(true)
			);
			assert_ne!(
				Nft::check_ownership(&AccountId::from(ALICE), &(0u32.into(), 0u32.into())),
				Ok(true)
			);
			assert_ne!(
				Nft::check_ownership(&AccountId::from(ALICE), &(0u32.into(), 1u32.into())),
				Ok(true)
			);
			// Create buy now for a land unit and an estate
			run_to_block(1);
			assert_ok!(Auction::create_new_buy_now(
				RawOrigin::Signed(AccountId::from(ALICE)).into(),
				ItemId::NFT(1u32.into(), 0u64.into()),
				100 * dollar(NATIVE_TOKEN),
				400u32.into(),
				ListingLevel::Local(0u32.into())
			));
			assert_ok!(Auction::create_new_buy_now(
				RawOrigin::Signed(AccountId::from(ALICE)).into(),
				ItemId::NFT(0u32.into(), 2u64.into()),
				100 * dollar(NATIVE_TOKEN),
				400u32.into(),
				ListingLevel::Local(0u32.into())
			));
			run_to_block(2);
			// Buy land units
			assert_ok!(Auction::buy_now(
				RawOrigin::Signed(AccountId::from(BOB)).into(),
				0u32.into(),
				100 * dollar(NATIVE_TOKEN),
			));
			assert_ok!(Auction::buy_now(
				RawOrigin::Signed(AccountId::from(BOB)).into(),
				1u32.into(),
				100 * dollar(NATIVE_TOKEN),
			));
			// Check estate, left land unit ownership, and updated balances
			assert_eq!(
				Nft::check_ownership(&AccountId::from(BOB), &(1u32.into(), 0u32.into())),
				Ok(true)
			);
			assert_eq!(
				Nft::check_ownership(&AccountId::from(BOB), &(0u32.into(), 2u32.into())),
				Ok(true)
			);
			assert_eq!(Balances::free_balance(AccountId::from(BOB)), 750 * dollar(NATIVE_TOKEN));
			assert_eq!(Balances::free_balance(AccountId::from(ALICE)), 1109000000000000000000);
			// Add land unit to estate
			assert_ok!(Estate::add_land_unit_to_estate(
				RawOrigin::Signed(AccountId::from(BOB)).into(),
				0u32.into(),
				vec![(1i32, 0i32)]
			));
			// Check estate and land units ownership
			assert_eq!(
				Nft::check_ownership(&AccountId::from(BOB), &(1u32.into(), 0u32.into())),
				Ok(true)
			);
			assert_ne!(
				Nft::check_ownership(&AccountId::from(BOB), &(0u32.into(), 2u32.into())),
				Ok(true)
			);
			// Remove land unit from estate
			assert_ok!(Estate::remove_land_unit_from_estate(
				RawOrigin::Signed(AccountId::from(BOB)).into(),
				0u32.into(),
				vec![(0i32, 1i32)]
			));
			// Check estate and land units ownership
			assert_eq!(
				Nft::check_ownership(&AccountId::from(BOB), &(1u32.into(), 0u32.into())),
				Ok(true)
			);
			assert_eq!(
				Nft::check_ownership(&AccountId::from(BOB), &(0u32.into(), 3u32.into())),
				Ok(true)
			);
		});
}
