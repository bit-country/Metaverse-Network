#[cfg(feature = "with-pioneer-runtime")]
use crate::setup::pioneer_imports::NATIVE_TOKEN;
#[cfg(feature = "with-pioneer-runtime")]
use crate::relaychain::kusama_test_net::*;
use crate::setup::*;
use core_primitives::NFTTrait;
use core_primitives::{Attributes, CollectionType, TokenType};
use core_traits::ItemId;
use frame_system::RawOrigin;
use setup::{run_to_block, test_attributes};
use sp_runtime::Perbill;

#[test]
fn test_list_nft() {
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
			// Create metaverse
			assert_ok!(Metaverse::create_metaverse(
				RawOrigin::Signed(AccountId::from(ALICE)).into(),
				vec![1u8]
			));
			// Check metaverse ownership
			// Create NFT group
			assert_ok!(Nft::create_group(RawOrigin::Root.into(), vec![2], vec![2]));
			// Create NFT class
			assert_ok!(Nft::create_class(
				RawOrigin::Signed(AccountId::from(ALICE)).into(),
				vec![1],
				test_attributes(1),
				1u32.into(),
				TokenType::Transferable,
				CollectionType::Collectable,
				Perbill::from_percent(0u32),
				None,
			));
			// Mint NFT
			assert_ok!(Nft::mint(
				RawOrigin::Signed(AccountId::from(ALICE)).into(),
				2u32.into(),
				vec![3],
				test_attributes(3),
				1,
			));
			run_to_block(1);
			// Check NFT ownership
			assert_ok!(NFTTrait::check_ownership(
				&AccountId::from(ALICE),
				&(2u32.into(), 0u32.into())
			));
			// List NFT as a buy now on a metaverse
			assert_ok!(Auction::create_new_buy_now(
				RawOrigin::Signed(AccountId::from(ALICE)).into(),
				ItemId::NFT(2, 0),
				100u32.into(),
				100u32.into(),
				ListingLevel::Local(0u32.into())
			));
			run_to_block(2);
			// Buy the NFT
			assert_ok!(Auction::buy_now(
				RawOrigin::Signed(AccountId::from(BOB)).into(),
				0u32.into(),
				100u32.into(),
			));
			// Check NFT ownership
			assert_ok!(NFTTrait::check_ownership(
				&AccountId::from(BOB),
				&(2u32.into(), 0u32.into())
			));
			assert_eq!(
				Balances::free_balance(AccountId::from(ALICE)),
				1_000 * dollar(NATIVE_TOKEN) + 100u32.into()
			);
			assert_eq!(
				Balances::free_balance(AccountId::from(BOB)),
				1_000 * dollar(NATIVE_TOKEN) - 100u32.into()
			);
		});
}
