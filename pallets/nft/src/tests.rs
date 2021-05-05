#[cfg(test)]
use super::*;
use mock::{Event, *};

use primitives::{Balance};

use orml_nft::Pallet as NftModule;
use sp_std::vec::Vec;

use frame_support::{assert_noop, assert_ok};
use sp_runtime::AccountId32;

fn free_balance(who: &AccountId) -> Balance {
    <Runtime as Config>::Currency::free_balance(who)
}

fn reserved_balance(who: &AccountId) -> Balance {
    <Runtime as Config>::Currency::reserved_balance(who)
}

fn class_id_account() -> AccountId {
    <Runtime as Config>::ModuleId::get().into_sub_account(CLASS_ID)
}

fn init_test_nft(owner: Origin) {
    assert_ok!(Nft::create_group(
        owner.clone(),
        vec![1],
        vec![1],
    ));
    assert_ok!(Nft::create_class(
        owner.clone(),
        vec![1],
        vec![1],
        COLLECTION_ID,
        TokenType::Transferrable,
        CollectionType::Collectable,
    ));
    assert_ok!(Nft::mint(
        owner.clone(),
        CLASS_ID,
        vec![1],
        vec![1],
        vec![1],
        1
    ));
}        


#[test]
fn create_group_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);
        assert_ok!(Nft::create_group(
            origin.clone(),
            vec![1],
            vec![1],
        ));

        let collection_data = NftGroupCollectionData
        {
            name: vec![1],
            properties:vec![1],
            owner: ALICE,
        };

        assert_eq!(Nft::get_group_collection(0), Some(collection_data));
        assert_eq!(Nft::all_nft_collection_count(), 1);

        let event = mock::Event::nft(RawEvent::NewNftCollectionCreated(ALICE, 0));
        assert_eq!(last_event(), event);
    });
}

#[test]
fn create_group_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);
        //find way to set next collection id
    });
}


#[test]
fn create_class_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);
        assert_ok!(Nft::create_group(
            origin.clone(),
            vec![1],
            vec![1],
        ));
        assert_ok!(Nft::create_class(
            origin.clone(),
            vec![1],
            vec![1],
            COLLECTION_ID,
            TokenType::Transferrable,
            CollectionType::Collectable,
        ));

        let class_data = NftClassData
        {
            deposit: 2,
            properties: vec![1],
            token_type: TokenType::Transferrable,
            collection_type: CollectionType::Collectable,
            total_supply: Default::default(),
            initial_supply: Default::default()
        };

        let class_info = orml_nft::ClassInfo::<u64, AccountId, NftClassData<u128>> {
            metadata: vec![1],
			total_issuance: Default::default(),
			owner: ALICE,
			data: class_data,
        };

        assert_eq!(Nft::get_class_collection(0), 0);
        assert_eq!(Nft::all_nft_collection_count(), 1);
        assert_eq!(NftModule::<Runtime>::classes(CLASS_ID), Some(class_info));

        let event = mock::Event::nft(RawEvent::NewNftClassCreated(ALICE, CLASS_ID));
        assert_eq!(last_event(), event);

        assert_eq!(
            reserved_balance(&class_id_account()),
            <Runtime as Config>::CreateClassDeposit::get()
        );
    });
}


#[test]
fn create_class_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);
        let invalid_owner = Origin::signed(BOB);
        assert_ok!(Nft::create_group(
            origin.clone(),
            vec![1],
            vec![1],
        ));

        //collection does not exist
        assert_noop!(Nft::create_class(
            origin.clone(),
            vec![1],
            vec![1],
            1,
            TokenType::Transferrable,
            CollectionType::Collectable,
        ), Error::<Runtime>::CollectionIsNotExist);

        //no permission
        assert_noop!(Nft::create_class(
            invalid_owner.clone(),
            vec![1],
            vec![1],
            0,
            TokenType::Transferrable,
            CollectionType::Collectable,
        ), Error::<Runtime>::NoPermission);
    });
}


#[test]
fn mint_asset_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);

        init_test_nft(origin.clone());
        
        assert_eq!(
            reserved_balance(&class_id_account()),
            <Runtime as Config>::CreateClassDeposit::get() +  <Runtime as Config>::CreateAssetDeposit::get()
        );
        assert_eq!(Nft::next_asset_id(), 1);
        assert_eq!(Nft::get_assets_by_owner(ALICE), vec![0]);
        assert_eq!(Nft::get_asset(0),Some((CLASS_ID, TOKEN_ID)));

        let event = mock::Event::nft(RawEvent::NewNftMinted(0, 0, ALICE, CLASS_ID, 1));
        assert_eq!(last_event(), event);

        //mint two assets
        assert_ok!(Nft::mint(
            origin.clone(),
            CLASS_ID,
            vec![1],
            vec![1],
            vec![1],
            2
        ));

        assert_eq!(Nft::next_asset_id(), 3);
        assert_eq!(Nft::get_assets_by_owner(ALICE), vec![0,1,2]);
        assert_eq!(Nft::get_asset(1),Some((CLASS_ID, 1)));
        assert_eq!(Nft::get_asset(2),Some((CLASS_ID, 2)));
    })
}

#[test]
fn mint_asset_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);
        let invalid_owner = Origin::signed(BOB);
        assert_ok!(Nft::create_group(
            origin.clone(),
            vec![1],
            vec![1],
        ));
        assert_ok!(Nft::create_class(
            origin.clone(),
            vec![1],
            vec![1],
            COLLECTION_ID,
            TokenType::Transferrable,
            CollectionType::Collectable,
        ));
        assert_noop!(Nft::mint(
            origin.clone(),
            CLASS_ID,
            vec![1],
            vec![1],
            vec![1],
            0
        ), Error::<Runtime>::InvalidQuantity);
        assert_noop!(Nft::mint(
            origin.clone(),
            1,
            vec![1],
            vec![1],
            vec![1],
            1
        ), Error::<Runtime>::ClassIdNotFound);
        assert_noop!(Nft::mint(
            invalid_owner.clone(),
            CLASS_ID,
            vec![1],
            vec![1],
            vec![1],
            1
        ), Error::<Runtime>::NoPermission);

        //TODO No asset id
    })
}


#[test]
fn transfer_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);
        init_test_nft(origin.clone());
        assert_ok!(Nft::transfer(origin, BOB,0));
        let event = mock::Event::nft(RawEvent::TransferedNft(1, 2, 0));
        assert_eq!(last_event(), event);
    })
}

#[test]
fn transfer_batch_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);
        init_test_nft(origin.clone());
        assert_ok!(Nft::create_class(
            origin.clone(),
            vec![1],
            vec![1],
            COLLECTION_ID,
            TokenType::Transferrable,
            CollectionType::Collectable,
        ));
        assert_ok!(Nft::mint(
            origin.clone(),
            1,
            vec![1],
            vec![1],
            vec![1],
            1
        ));
        assert_ok!(Nft::transfer_batch(origin, vec![(BOB,0),(BOB,1)]));
        let event = mock::Event::nft(RawEvent::TransferedNft(1, 2, 0));
        assert_eq!(last_event(), event);
    })
}

#[test]
fn transfer_batch_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
    let origin = Origin::signed(ALICE);
        init_test_nft(origin.clone());
        assert_ok!(Nft::create_class(
            origin.clone(),
            vec![1],
            vec![1],
            COLLECTION_ID,
            TokenType::Transferrable,
            CollectionType::Collectable,
        ));
        assert_ok!(Nft::mint(
            origin.clone(),
            1,
            vec![1],
            vec![1],
            vec![1],
            1
        ));
        assert_noop!(Nft::transfer_batch(origin.clone(), vec![(BOB,3),(BOB,4)]), Error::<Runtime>::AssetIdNotFound);
        //TO DO add test case for ClassIdNotFound
         //TO DO add test case for AssetInfoNotFound
    })
}

#[test]
fn do_create_group_collection_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Nft::do_create_group_collection(&ALICE, vec![1], vec![1]));
        let collection_data = NftGroupCollectionData
        {
            name: vec![1],
            properties:vec![1],
            owner: ALICE,
        };
        assert_eq!(Nft::get_group_collection(0), Some(collection_data));
    })
}

#[test]
fn do_handle_asset_ownership_transfer_should_work() {
    let origin = Origin::signed(ALICE);
    ExtBuilder::default().build().execute_with(|| {
        init_test_nft(origin.clone());
        assert_ok!(Nft::handle_asset_ownership_transfer(&ALICE, &BOB, 0));
        assert_eq!(Nft::get_assets_by_owner(ALICE), Vec::<u64>::new());
        assert_eq!(Nft::get_assets_by_owner(BOB), vec![0]);
    })
}

#[test]
fn do_transfer_should_work() {
    let origin = Origin::signed(ALICE);
    ExtBuilder::default().build().execute_with(|| {
    init_test_nft(origin.clone());
    assert_ok!(Nft::do_transfer(&ALICE, &BOB, 0));
    assert_eq!(Nft::get_assets_by_owner(ALICE), Vec::<u64>::new());
    assert_eq!(Nft::get_assets_by_owner(BOB), vec![0]);
})
}


#[test]
fn do_transfer_should_fail() {
    let origin = Origin::signed(ALICE);
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(Nft::do_transfer(&ALICE, &BOB, 0), Error::<Runtime>::AssetIdNotFound);
        //TODO add test for ClassIdNotFound

        init_test_nft(origin.clone());

        assert_noop!(Nft::do_transfer(&BOB, &ALICE, 0), Error::<Runtime>::NoPermission);

        assert_ok!(Nft::create_class(
            origin.clone(),
            vec![1],
            vec![1],
            COLLECTION_ID,
            TokenType::BoundToAddress,
            CollectionType::Collectable,
        ));
        assert_ok!(Nft::mint(
            origin.clone(),
            1,
            vec![1],
            vec![1],
            vec![1],
            1
        ));

        assert_noop!(Nft::do_transfer(&ALICE, &BOB, 1), Error::<Runtime>::NonTransferrable);
    })
}


#[test]
fn do_check_nft_ownership_should_work() {
    let origin = Origin::signed(ALICE);
    ExtBuilder::default().build().execute_with(|| {
        init_test_nft(origin.clone());
        assert_ok!(Nft::check_nft_ownership(&ALICE, &TOKEN_ID), true);
        assert_ok!(Nft::check_nft_ownership(&BOB, &TOKEN_ID), false);
    })
}


#[test]
fn do_check_nft_ownership_should_fail() {
    let origin = Origin::signed(ALICE);
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(Nft::check_nft_ownership(&ALICE, &TOKEN_ID), Error::<Runtime>::AssetIdNotFound);
        //TODO: ClassIdNotFound
        //TODO: add test case for AssetInfoNotFound
    })
}