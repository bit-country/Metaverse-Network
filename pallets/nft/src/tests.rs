use frame_support::{assert_noop, assert_ok};
use orml_nft::Pallet as NftModule;
use orml_traits::MultiCurrency;
use sp_runtime::traits::BadOrigin;
use sp_std::default::Default;

use mock::*;
use primitives::{Balance, FungibleTokenId};

#[cfg(test)]
use super::*;

fn free_bit_balance(who: &AccountId) -> Balance {
	<Runtime as Config>::MultiCurrency::free_balance(mining_resource_id(), &who)
}

fn free_native_balance(who: AccountId) -> Balance {
	<Runtime as Config>::Currency::free_balance(who)
}

fn class_id_account() -> AccountId {
	<Runtime as Config>::Treasury::get().into_account()
}

fn test_attributes(x: u8) -> Attributes {
	let mut attr: Attributes = BTreeMap::new();
	attr.insert(vec![x, x + 5], vec![x, x + 10]);
	attr
}

fn mining_resource_id() -> FungibleTokenId {
	<Runtime as Config>::MiningResourceId::get()
}

fn init_test_nft(owner: Origin) {
	assert_ok!(Nft::create_group(Origin::root(), vec![1], vec![1],));
	assert_ok!(Nft::create_class(
		owner.clone(),
		vec![1],
		test_attributes(1),
		COLLECTION_ID,
		TokenType::Transferable,
		CollectionType::Collectable,
		Perbill::from_percent(0u32),
		None
	));
	assert_ok!(Nft::mint(owner.clone(), CLASS_ID, vec![1], test_attributes(1), 1));
}

fn init_bound_to_address_nft(owner: Origin) {
	assert_ok!(Nft::create_group(Origin::root(), vec![1], vec![1],));
	assert_ok!(Nft::create_class(
		owner.clone(),
		vec![1],
		test_attributes(1),
		COLLECTION_ID,
		TokenType::BoundToAddress,
		CollectionType::Collectable,
		Perbill::from_percent(0u32),
		None
	));
	assert_ok!(Nft::mint(owner.clone(), CLASS_ID, vec![1], test_attributes(1), 1));
}

#[test]
fn enable_promotion_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::root();
		assert_ok!(Nft::enable_promotion(origin, true));

		assert_eq!(Nft::get_promotion_enabled(), true);

		let event = mock::Event::Nft(crate::Event::PromotionEnabled(true));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn disable_promotion_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::root();
		assert_ok!(Nft::enable_promotion(origin, false));

		assert_eq!(Nft::get_promotion_enabled(), false);

		let event = mock::Event::Nft(crate::Event::PromotionEnabled(false));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn create_group_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::root();
		assert_ok!(Nft::create_group(origin, vec![1], vec![1]));

		let collection_data = NftGroupCollectionData {
			name: vec![1],
			properties: vec![1],
		};

		assert_eq!(Nft::get_group_collection(0), Some(collection_data));
		assert_eq!(Nft::all_nft_collection_count(), 1);

		let event = mock::Event::Nft(crate::Event::NewNftCollectionCreated(0));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn create_group_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);

		assert_noop!(Nft::create_group(origin, vec![1], vec![1]), BadOrigin);
	});
}

#[test]
fn create_class_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);

		assert_ok!(Nft::create_group(Origin::root(), vec![1], vec![1],));
		assert_ok!(Nft::create_class(
			origin.clone(),
			vec![1],
			test_attributes(1),
			COLLECTION_ID,
			TokenType::Transferable,
			CollectionType::Collectable,
			Perbill::from_percent(0u32),
			None
		));
		let class_deposit = <Runtime as Config>::ClassMintingFee::get();
		assert_eq!(Nft::get_class_collection(0), 0);
		assert_eq!(Nft::all_nft_collection_count(), 1);
		assert_eq!(
			NftModule::<Runtime>::classes(CLASS_ID).unwrap().data,
			NftClassData {
				deposit: class_deposit,
				token_type: TokenType::Transferable,
				collection_type: CollectionType::Collectable,
				is_locked: false,
				attributes: test_attributes(1),
				royalty_fee: Perbill::from_percent(0u32),
				mint_limit: None,
				total_minted_tokens: 0u32,
			}
		);

		let event = mock::Event::Nft(crate::Event::NewNftClassCreated(ALICE, CLASS_ID));
		assert_eq!(last_event(), event);

		assert_eq!(free_native_balance(class_id_account()), class_deposit);
	});
}

#[test]
fn create_class_with_royalty_fee_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);

		assert_ok!(Nft::create_group(Origin::root(), vec![1], vec![1],));
		assert_ok!(Nft::create_class(
			origin.clone(),
			vec![1],
			test_attributes(1),
			COLLECTION_ID,
			TokenType::Transferable,
			CollectionType::Collectable,
			Perbill::from_percent(10u32),
			None
		));
		let class_deposit = <Runtime as Config>::ClassMintingFee::get();
		assert_eq!(Nft::get_class_collection(0), 0);
		assert_eq!(Nft::all_nft_collection_count(), 1);
		assert_eq!(
			NftModule::<Runtime>::classes(CLASS_ID).unwrap().data,
			NftClassData {
				deposit: class_deposit,
				token_type: TokenType::Transferable,
				collection_type: CollectionType::Collectable,
				is_locked: false,
				attributes: test_attributes(1),
				royalty_fee: Perbill::from_percent(10u32),
				mint_limit: None,
				total_minted_tokens: 0u32,
			}
		);

		let event = mock::Event::Nft(crate::Event::NewNftClassCreated(ALICE, CLASS_ID));
		assert_eq!(last_event(), event);

		assert_eq!(free_native_balance(class_id_account()), class_deposit);
	});
}

#[test]
fn mint_asset_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		assert_ok!(Nft::enable_promotion(Origin::root(), true));
		init_test_nft(origin.clone());

		assert_eq!(free_native_balance(class_id_account()), 3);
		assert_eq!(OrmlNft::tokens_by_owner((ALICE, 0, 0)), ());

		let event = mock::Event::Nft(crate::Event::NewNftMinted((0, 0), (0, 0), ALICE, CLASS_ID, 1, 0));
		assert_eq!(last_event(), event);

		// mint two assets
		assert_ok!(Nft::mint(origin.clone(), CLASS_ID, vec![1], test_attributes(1), 2));

		// bit balance should be 0 (minted 2 NFT)
		assert_eq!(free_bit_balance(&ALICE), 0);

		assert_eq!(OrmlNft::tokens_by_owner((ALICE, 0, 0)), ());
		assert_eq!(OrmlNft::tokens_by_owner((ALICE, 0, 1)), ());
		assert_eq!(OrmlNft::tokens_by_owner((ALICE, 0, 2)), ());
	})
}

#[test]
fn mint_asset_with_promotion_enabled_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		assert_ok!(Nft::enable_promotion(Origin::root(), true));
		init_test_nft(origin.clone());

		// bit balance should be 0 (minted 1 NFT)
		assert_eq!(free_bit_balance(&ALICE), 0);
	})
}

#[test]
fn mint_asset_with_promotion_disabled_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		assert_ok!(Nft::enable_promotion(Origin::root(), false));
		init_test_nft(origin.clone());

		// bit balance should be 1 (minted 1 NFT)
		assert_eq!(free_bit_balance(&ALICE), 0);
	})
}

#[test]
fn mint_asset_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		let invalid_owner = Origin::signed(BOB);
		assert_ok!(Nft::create_group(Origin::root(), vec![1], vec![1],));
		assert_ok!(Nft::create_class(
			origin.clone(),
			vec![1],
			test_attributes(1),
			COLLECTION_ID,
			TokenType::Transferable,
			CollectionType::Collectable,
			Perbill::from_percent(0u32),
			Some(10)
		));
		assert_noop!(
			Nft::mint(origin.clone(), CLASS_ID, vec![1], test_attributes(1), 0),
			Error::<Runtime>::InvalidQuantity
		);
		assert_noop!(
			Nft::mint(origin.clone(), 1, vec![1], test_attributes(1), 1),
			Error::<Runtime>::ClassIdNotFound
		);
		assert_noop!(
			Nft::mint(invalid_owner.clone(), CLASS_ID, vec![1], test_attributes(1), 1),
			Error::<Runtime>::NoPermission
		);
	})
}

#[test]
fn mint_exceed_max_minting_limit_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		assert_ok!(Nft::create_group(Origin::root(), vec![1], vec![1],));
		assert_ok!(Nft::create_class(
			origin.clone(),
			vec![1],
			test_attributes(1),
			COLLECTION_ID,
			TokenType::Transferable,
			CollectionType::Collectable,
			Perbill::from_percent(0u32),
			Some(10)
		));
		assert_noop!(
			Nft::mint(origin.clone(), CLASS_ID, vec![1], test_attributes(1), 11),
			Error::<Runtime>::ExceededMintingLimit
		);
		assert_ok!(Nft::mint(origin.clone(), CLASS_ID, vec![1], test_attributes(1), 10));
		assert_noop!(
			Nft::mint(origin.clone(), CLASS_ID, vec![1], test_attributes(1), 1),
			Error::<Runtime>::ExceededMintingLimit
		);
	})
}

#[test]
fn mint_exceed_max_batch_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		assert_ok!(Nft::create_group(Origin::root(), vec![1], vec![1]));
		assert_ok!(Nft::create_class(
			origin.clone(),
			vec![1],
			test_attributes(1),
			COLLECTION_ID,
			TokenType::Transferable,
			CollectionType::Collectable,
			Perbill::from_percent(0u32),
			None
		));
		assert_noop!(
			Nft::mint(origin.clone(), CLASS_ID, vec![1], test_attributes(1), 20),
			Error::<Runtime>::ExceedMaximumBatchMinting
		);
	})
}

#[test]
fn transfer_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		init_test_nft(origin.clone());
		assert_ok!(Nft::transfer(origin, BOB, (0, 0)));
		let event = mock::Event::Nft(crate::Event::TransferedNft(ALICE, BOB, 0, (0, 0)));
		assert_eq!(last_event(), event);
	})
}

#[test]
fn burn_nft_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		init_test_nft(origin.clone());
		assert_ok!(Nft::mint(origin.clone(), CLASS_ID, vec![1], test_attributes(1), 1));
		assert_ok!(Nft::burn(origin, (0, 1)));
		let event = mock::Event::Nft(crate::Event::BurnedNft((0, 1)));
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
			test_attributes(1),
			COLLECTION_ID,
			TokenType::Transferable,
			CollectionType::Collectable,
			Perbill::from_percent(0u32),
			None
		));
		assert_ok!(Nft::mint(origin.clone(), 1, vec![1], test_attributes(1), 4));
		assert_ok!(Nft::transfer_batch(origin, vec![(BOB, (1, 0)), (BOB, (1, 1))]));
		let event = mock::Event::Nft(crate::Event::TransferedNft(ALICE, BOB, 1, (1, 1)));
		assert_eq!(last_event(), event);
	})
}

#[test]
fn transfer_batch_exceed_length_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		init_test_nft(origin.clone());
		assert_ok!(Nft::create_class(
			origin.clone(),
			vec![1],
			test_attributes(1),
			COLLECTION_ID,
			TokenType::Transferable,
			CollectionType::Collectable,
			Perbill::from_percent(0u32),
			None
		));
		assert_ok!(Nft::mint(origin.clone(), 1, vec![1], test_attributes(1), 4));
		assert_noop!(
			Nft::transfer_batch(origin, vec![(BOB, (0, 0)), (BOB, (0, 1)), (BOB, (0, 2)), (BOB, (0, 3))]),
			Error::<Runtime>::ExceedMaximumBatchTransfer
		);
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
			test_attributes(1),
			COLLECTION_ID,
			TokenType::Transferable,
			CollectionType::Collectable,
			Perbill::from_percent(0u32),
			None
		));
		assert_ok!(Nft::mint(origin.clone(), 1, vec![1], test_attributes(1), 1));
		assert_noop!(
			Nft::transfer_batch(origin.clone(), vec![(BOB, (0, 3)), (BOB, (0, 4))]),
			Error::<Runtime>::AssetInfoNotFound
		);
	})
}

#[test]
fn do_create_group_collection_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Nft::do_create_group_collection(vec![1], vec![1]));
		let collection_data = NftGroupCollectionData {
			name: vec![1],
			properties: vec![1],
		};
		assert_eq!(Nft::get_group_collection(0), Some(collection_data));
	})
}

#[test]
fn do_transfer_should_fail() {
	let origin = Origin::signed(ALICE);
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(Nft::do_transfer(ALICE, BOB, (0, 0)), Error::<Runtime>::ClassIdNotFound);

		init_test_nft(origin.clone());

		assert_noop!(Nft::do_transfer(BOB, ALICE, (0, 0)), Error::<Runtime>::NoPermission);

		assert_ok!(Nft::create_class(
			origin.clone(),
			vec![1],
			test_attributes(1),
			COLLECTION_ID,
			TokenType::BoundToAddress,
			CollectionType::Collectable,
			Perbill::from_percent(0u32),
			None
		));
		assert_ok!(Nft::mint(origin.clone(), 1, vec![1], test_attributes(1), 1));

		assert_noop!(
			Nft::do_transfer(ALICE, BOB, (0, 1)),
			Error::<Runtime>::AssetInfoNotFound
		);
	})
}

#[test]
fn do_transfer_should_fail_if_bound_to_address() {
	let origin = Origin::signed(ALICE);
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(Nft::do_transfer(ALICE, BOB, (0, 0)), Error::<Runtime>::ClassIdNotFound);

		init_bound_to_address_nft(origin.clone());

		// Owner allowed to transfer
		assert_ok!(Nft::transfer(origin.clone(), BOB, (0, 0)));

		let event = mock::Event::Nft(crate::Event::TransferedNft(ALICE, BOB, 0, (0, 0)));
		assert_eq!(last_event(), event);

		// Reject ownership if BOB tries to transfer
		assert_noop!(Nft::do_transfer(BOB, ALICE, (0, 0)), Error::<Runtime>::NonTransferable);
	})
}

#[test]
fn do_check_nft_ownership_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		init_test_nft(origin.clone());
		assert_ok!(Nft::check_nft_ownership(&ALICE, &(CLASS_ID, TOKEN_ID)), true);
		assert_ok!(Nft::check_nft_ownership(&BOB, &(CLASS_ID, TOKEN_ID)), false);
	})
}

#[test]
fn do_check_nft_ownership_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Nft::check_nft_ownership(&ALICE, &(CLASS_ID, TOKEN_ID)),
			Error::<Runtime>::AssetInfoNotFound
		);
	})
}

#[test]
fn do_withdraw_funds_from_class_fund_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		assert_noop!(
			Nft::withdraw_funds_from_class_fund(origin.clone(), NON_EXISTING_CLASS_ID),
			Error::<Runtime>::ClassIdNotFound
		);
		assert_ok!(Nft::create_group(Origin::root(), vec![1], vec![1]));
		assert_ok!(Nft::create_class(
			origin.clone(),
			vec![1],
			test_attributes(1),
			COLLECTION_ID,
			TokenType::Transferable,
			CollectionType::Collectable,
			Perbill::from_percent(0u32),
			None
		));
		let non_class_owner_origin = Origin::signed(BOB);
		assert_noop!(
			Nft::withdraw_funds_from_class_fund(non_class_owner_origin, CLASS_ID),
			Error::<Runtime>::NoPermission
		);
	})
}

#[test]
fn do_withdraw_funds_from_class_fund_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		init_test_nft(origin.clone());
		let class_fund: AccountId = <Runtime as Config>::PalletId::get().into_sub_account(CLASS_ID);
		assert_ok!(<Runtime as Config>::Currency::transfer(origin.clone(), class_fund, 100));
		assert_eq!(free_native_balance(ALICE), 99897);
		assert_eq!(free_native_balance(class_fund), 100);
		assert_ok!(Nft::withdraw_funds_from_class_fund(origin.clone(), CLASS_ID));
		assert_eq!(free_native_balance(ALICE), 99996);
		assert_eq!(free_native_balance(class_fund), 1);
	})
}

#[test]
fn setting_hard_limit_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		let failing_origin = Origin::signed(BOB);
		assert_ok!(Nft::create_group(Origin::root(), vec![1], vec![1],));
		assert_noop!(
			Nft::set_hard_limit(failing_origin.clone(), CLASS_ID, 10u32),
			Error::<Runtime>::NoPermission
		);

		assert_ok!(Nft::create_class(
			origin.clone(),
			vec![1],
			test_attributes(1),
			COLLECTION_ID,
			TokenType::Transferable,
			CollectionType::Collectable,
			Perbill::from_percent(0u32),
			Some(1u32)
		));
		assert_noop!(
			Nft::set_hard_limit(origin.clone(), CLASS_ID_1, 10u32),
			Error::<Runtime>::HardLimitIsAlreadySet
		);

		assert_ok!(Nft::mint(origin.clone(), CLASS_ID, vec![1], test_attributes(1), 1));
		assert_ok!(Nft::mint(origin.clone(), CLASS_ID, vec![1], test_attributes(1), 1));
		assert_noop!(
			Nft::set_hard_limit(origin.clone(), CLASS_ID, 1u32),
			Error::<Runtime>::TotalMintedAssetsForClassExceededProposedLimit
		);
	})
}

#[test]
fn setting_hard_limit_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		let class_deposit = <Runtime as Config>::ClassMintingFee::get();
		assert_ok!(Nft::create_group(Origin::root(), vec![1], vec![1],));
		assert_ok!(Nft::create_class(
			origin.clone(),
			vec![1],
			test_attributes(1),
			COLLECTION_ID,
			TokenType::Transferable,
			CollectionType::Collectable,
			Perbill::from_percent(0u32),
			None
		));
		assert_ok!(Nft::set_hard_limit(origin.clone(), CLASS_ID, 10u32));
		assert_eq!(
			NftModule::<Runtime>::classes(CLASS_ID).unwrap().data,
			NftClassData {
				deposit: class_deposit,
				token_type: TokenType::Transferable,
				collection_type: CollectionType::Collectable,
				is_locked: false,
				attributes: test_attributes(1),
				royalty_fee: Perbill::from_percent(0u32),
				mint_limit: Some(10u32),
				total_minted_tokens: 0u32,
			}
		);
		let event = mock::Event::Nft(crate::Event::HardLimitSet(CLASS_ID));
		assert_eq!(last_event(), event);
	})
}
