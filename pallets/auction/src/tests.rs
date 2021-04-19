#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};
use pallet_nft::{TokenType, CollectionType};

#[test]
// Private new_auction should work
fn create_new_auction_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);

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
        assert_ok!(NftAuctionModule::create_new_auction(origin, ItemId::NFT(0), 100));
    });
}