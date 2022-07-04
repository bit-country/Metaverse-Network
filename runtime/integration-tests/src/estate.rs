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
				Estate::check_undeployed_land_block(
					&AccountId::from(ALICE),
					0u32.into()
				),
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
				Estate::check_undeployed_land_block(
					&AccountId::from(BOB),
					0u32.into()
				),
				Ok(true)
			);
			assert_eq!(Balances::free_balance(AccountId::from(BOB)), 894 * dollar(NATIVE_TOKEN));
			assert_eq!(
				Balances::free_balance(AccountId::from(CHARLIE)),
				1000 * dollar(NATIVE_TOKEN)
			);
			assert_eq!(
				Balances::free_balance(AccountId::from(ALICE)),
				1102950000000000000000
			);
			// Deploy undeployed land block
			assert_ok!(Estate::deploy_land_block(
				RawOrigin::Signed(AccountId::from(BOB)).into(),
				0u32.into(),
				1u32.into(),
				(0i32, 0i32),
				vec![(0i32, 0i32)]
			));
			// Check if land unit was deployed
			assert_eq!(Estate::check_landunit(1u32.into(), (0i32, 0i32)), Ok(true));
		});
}

/* 
#[test]
fn create_estate_from_purchased_land_blocks() {
}

#[test]
fn buy_estate_from_marketplace_and_modify_its_structure() {
}
*/

