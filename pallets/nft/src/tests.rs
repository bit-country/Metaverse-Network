use frame_support::{assert_noop, assert_ok};
use orml_nft::Pallet as NftModule;
use orml_traits::MultiCurrency;
use sp_core::Pair;
use sp_runtime::traits::{BadOrigin, IdentifyAccount};
use sp_runtime::{MultiSignature, MultiSigner};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::default::Default;

use mock::*;
use primitives::{Balance, FungibleTokenId};

#[cfg(test)]
use super::*;

type AccountIdOf<Runtime> = <Runtime as frame_system::Config>::AccountId;
fn account(id: u8) -> AccountIdOf<Runtime> {
	[id; 32].into()
}

fn free_bit_balance(who: &AccountId) -> Balance {
	<Runtime as Config>::MultiCurrency::free_balance(mining_resource_id(), &who)
}

fn free_native_balance(who: AccountId) -> Balance {
	<Runtime as Config>::Currency::free_balance(who)
}

fn class_id_account() -> AccountId {
	<Runtime as Config>::Treasury::get().into_account_truncating()
}

fn test_attributes(x: u8) -> Attributes {
	let mut attr: Attributes = BTreeMap::new();
	attr.insert(vec![x, x + 5], vec![x, x + 10]);
	attr
}

fn mining_resource_id() -> FungibleTokenId {
	<Runtime as Config>::MiningResourceId::get()
}

fn init_test_nft(owner: RuntimeOrigin) {
	assert_ok!(Nft::create_group(RuntimeOrigin::root(), vec![1], vec![1],));
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

fn init_test_stackable_nft(owner: RuntimeOrigin) {
	assert_ok!(Nft::create_group(RuntimeOrigin::root(), vec![1], vec![1],));
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
	assert_ok!(Nft::mint_stackable_nft(
		owner.clone(),
		CLASS_ID,
		vec![1],
		test_attributes(1),
		100u32.into()
	));
}

fn init_bound_to_address_nft(owner: RuntimeOrigin) {
	assert_ok!(Nft::create_group(RuntimeOrigin::root(), vec![1], vec![1],));
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
		let origin = RuntimeOrigin::root();
		assert_ok!(Nft::enable_promotion(origin, true));

		assert_eq!(Nft::get_promotion_enabled(), true);

		let event = mock::RuntimeEvent::Nft(crate::Event::PromotionEnabled(true));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn disable_promotion_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::root();
		assert_ok!(Nft::enable_promotion(origin, false));

		assert_eq!(Nft::get_promotion_enabled(), false);

		let event = mock::RuntimeEvent::Nft(crate::Event::PromotionEnabled(false));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn create_group_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::root();
		assert_ok!(Nft::create_group(origin, vec![1], vec![1]));

		let collection_data = NftGroupCollectionData {
			name: vec![1],
			properties: vec![1],
		};

		assert_eq!(Nft::get_group_collection(0), Some(collection_data));
		assert_eq!(Nft::all_nft_collection_count(), 1);

		let event = mock::RuntimeEvent::Nft(crate::Event::NewNftCollectionCreated(0));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn create_group_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(account(1));

		assert_noop!(Nft::create_group(origin, vec![1], vec![1]), BadOrigin);
	});
}

#[test]
fn create_class_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(account(1));

		assert_ok!(Nft::create_group(RuntimeOrigin::root(), vec![1], vec![1],));
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

		let event = mock::RuntimeEvent::Nft(crate::Event::NewNftClassCreated(account(1), CLASS_ID));
		assert_eq!(last_event(), event);

		assert_eq!(
			free_native_balance(class_id_account()),
			class_deposit + <Runtime as Config>::StorageDepositFee::get()
		);
		assert_eq!(Balances::free_balance(account(1)), 99997);
	});
}

#[test]
fn create_class_with_royalty_fee_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(account(1));

		assert_ok!(Nft::create_group(RuntimeOrigin::root(), vec![1], vec![1],));
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

		let event = mock::RuntimeEvent::Nft(crate::Event::NewNftClassCreated(account(1), CLASS_ID));
		assert_eq!(last_event(), event);

		assert_eq!(
			free_native_balance(class_id_account()),
			class_deposit + <Runtime as Config>::StorageDepositFee::get()
		);
		assert_eq!(Balances::free_balance(account(1)), 99997);
	});
}

#[test]
fn mint_asset_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(account(1));
		assert_ok!(Nft::enable_promotion(RuntimeOrigin::root(), true));
		init_test_nft(origin.clone());

		assert_eq!(free_native_balance(class_id_account()), 4);
		assert_eq!(OrmlNft::tokens_by_owner((account(1), 0, 0)), ());

		let event = mock::RuntimeEvent::Nft(crate::Event::NewNftMinted((0, 0), (0, 0), account(1), CLASS_ID, 1, 0));
		assert_eq!(last_event(), event);

		// mint two assets
		assert_ok!(Nft::mint(origin.clone(), CLASS_ID, vec![1], test_attributes(1), 2));

		// bit balance should be 0 (minted 2 NFT)
		assert_eq!(free_bit_balance(&account(1)), 0);

		assert_eq!(OrmlNft::tokens_by_owner((account(1), 0, 0)), ());
		assert_eq!(OrmlNft::tokens_by_owner((account(1), 0, 1)), ());
		assert_eq!(OrmlNft::tokens_by_owner((account(1), 0, 2)), ());
	})
}

#[test]
fn mint_asset_with_promotion_enabled_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(account(1));
		assert_ok!(Nft::enable_promotion(RuntimeOrigin::root(), true));
		init_test_nft(origin.clone());

		// bit balance should be 0 (minted 1 NFT)
		assert_eq!(free_bit_balance(&account(1)), 0);
	})
}

#[test]
fn mint_asset_with_promotion_disabled_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(account(1));
		assert_ok!(Nft::enable_promotion(RuntimeOrigin::root(), false));
		init_test_nft(origin.clone());

		// bit balance should be 1 (minted 1 NFT)
		assert_eq!(free_bit_balance(&account(1)), 0);
	})
}

#[test]
fn mint_stackable_asset_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(account(1));
		assert_ok!(Nft::enable_promotion(RuntimeOrigin::root(), true));
		init_test_stackable_nft(origin.clone());

		assert_eq!(free_native_balance(class_id_account()), 4);
		assert_eq!(OrmlNft::tokens_by_owner((account(1), 0, 0)), ());

		assert_eq!(
			OrmlNft::get_stackable_collections_balances((0, 0, account(1))),
			100u32.into()
		);

		let event = mock::RuntimeEvent::Nft(crate::Event::NewStackableNftMinted(
			account(1),
			CLASS_ID,
			0,
			100u32.into(),
		));
		assert_eq!(last_event(), event);
	})
}

#[test]
fn mint_asset_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(account(1));
		let invalid_owner = RuntimeOrigin::signed(account(2));
		assert_ok!(Nft::create_group(RuntimeOrigin::root(), vec![1], vec![1],));
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
		let origin = RuntimeOrigin::signed(account(1));
		assert_ok!(Nft::create_group(RuntimeOrigin::root(), vec![1], vec![1],));
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
fn mint_stackable_asset_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(account(1));
		let invalid_owner = RuntimeOrigin::signed(account(2));
		assert_ok!(Nft::create_group(RuntimeOrigin::root(), vec![1], vec![1],));
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
			Nft::mint_stackable_nft(origin.clone(), CLASS_ID, vec![1], test_attributes(1), 0u32.into()),
			Error::<Runtime>::InvalidStackableNftAmount
		);
		assert_noop!(
			Nft::mint_stackable_nft(origin.clone(), 1, vec![1], test_attributes(1), 1u32.into()),
			Error::<Runtime>::ClassIdNotFound
		);
		assert_noop!(
			Nft::mint_stackable_nft(
				invalid_owner.clone(),
				CLASS_ID,
				vec![1],
				test_attributes(1),
				1u32.into()
			),
			Error::<Runtime>::NoPermission
		);
	})
}

#[test]
fn mint_exceed_max_batch_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(account(1));
		assert_ok!(Nft::create_group(RuntimeOrigin::root(), vec![1], vec![1]));
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
		let origin = RuntimeOrigin::signed(account(1));
		init_test_nft(origin.clone());
		assert_ok!(Nft::transfer(origin, account(2), (0, 0)));
		let event = mock::RuntimeEvent::Nft(crate::Event::TransferedNft(account(1), account(2), 0, (0, 0)));
		assert_eq!(last_event(), event);
	})
}

#[test]
fn transfer_stackable_nft_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(account(1));
		init_test_stackable_nft(origin.clone());
		assert_eq!(
			OrmlNft::get_stackable_collections_balances((0, 0, account(1))),
			100u32.into()
		);

		assert_ok!(Nft::transfer_stackable_nft(origin, account(2), (0, 0), 50u32.into()));
		assert_eq!(
			OrmlNft::get_stackable_collections_balances((0, 0, account(2))),
			50u32.into()
		);
		assert_eq!(
			OrmlNft::get_stackable_collections_balances((0, 0, account(1))),
			50u32.into()
		);

		let event = mock::RuntimeEvent::Nft(crate::Event::TransferedStackableNft(
			account(1),
			account(2),
			(0, 0),
			50u32.into(),
		));
		assert_eq!(last_event(), event);
	})
}

#[test]
fn transfer_stackable_nft_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(account(1));
		let failing_origin = RuntimeOrigin::signed(account(2));
		init_test_stackable_nft(origin.clone());
		init_test_nft(origin.clone());

		assert_noop!(
			Nft::transfer_stackable_nft(origin.clone(), account(2), (0, 1), 0u32.into()),
			Error::<Runtime>::InvalidStackableNftTransfer
		);

		assert_noop!(
			Nft::transfer_stackable_nft(origin.clone(), account(2), (0, 1), 10u32.into()),
			Error::<Runtime>::InvalidStackableNftTransfer
		);

		assert_noop!(
			Nft::transfer_stackable_nft(origin.clone(), account(2), (0, 0), 101u32.into()),
			Error::<Runtime>::InvalidStackableNftTransfer
		);

		ReservedStackableNftBalance::<Runtime>::insert(account(1), (0, 0), 70);

		assert_noop!(
			Nft::transfer_stackable_nft(origin.clone(), account(2), (0, 0), 71u128),
			Error::<Runtime>::InvalidStackableNftTransfer
		);

		assert_noop!(
			Nft::transfer_stackable_nft(failing_origin, account(1), (0, 0), 10u32.into()),
			Error::<Runtime>::InvalidStackableNftTransfer
		);

		ReservedStackableNftBalance::<Runtime>::insert(account(1), (0, 0), 0);

		assert_ok!(Nft::transfer_stackable_nft(origin, account(2), (0, 0), 71u32.into()));
	})
}

#[test]
fn burn_nft_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(account(1));
		init_test_nft(origin.clone());
		assert_ok!(Nft::mint(origin.clone(), CLASS_ID, vec![1], test_attributes(1), 1));
		assert_ok!(Nft::burn(origin, (0, 1)));
		let event = mock::RuntimeEvent::Nft(crate::Event::BurnedNft((0, 1)));
		assert_eq!(last_event(), event);
	})
}

#[test]
fn burn_nft_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(account(1));

		init_test_stackable_nft(origin.clone());

		assert_noop!(Nft::burn(origin.clone(), (0, 0)), Error::<Runtime>::InvalidAssetType);

		assert_ok!(Nft::transfer_stackable_nft(
			origin.clone(),
			account(2),
			(0, 0),
			100u32.into()
		));

		assert_noop!(Nft::burn(origin.clone(), (0, 0)), Error::<Runtime>::InvalidAssetType);
	})
}

#[test]
fn transfer_batch_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(account(1));
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
		assert_ok!(Nft::transfer_batch(
			origin,
			vec![(account(2), (1, 0)), (account(2), (1, 1))]
		));
		let event = mock::RuntimeEvent::Nft(crate::Event::TransferedNft(account(1), account(2), 1, (1, 1)));
		assert_eq!(last_event(), event);
	})
}

#[test]
fn transfer_batch_exceed_length_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(account(1));
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
			Nft::transfer_batch(
				origin,
				vec![
					(account(2), (0, 0)),
					(account(2), (0, 1)),
					(account(2), (0, 2)),
					(account(2), (0, 3))
				]
			),
			Error::<Runtime>::ExceedMaximumBatchTransfer
		);
	})
}

#[test]
fn transfer_batch_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(account(1));
		init_test_stackable_nft(origin.clone());
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
			Nft::transfer_batch(origin.clone(), vec![(account(2), (0, 0)), (account(2), (0, 1))]),
			Error::<Runtime>::InvalidAssetType
		);
		assert_noop!(
			Nft::transfer_batch(origin.clone(), vec![(account(2), (0, 3)), (account(2), (0, 6))]),
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
	let origin = RuntimeOrigin::signed(account(1));
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Nft::do_transfer(account(1), account(2), (0, 0)),
			Error::<Runtime>::ClassIdNotFound
		);

		init_test_nft(origin.clone());

		assert_noop!(
			Nft::do_transfer(account(2), account(1), (0, 0)),
			Error::<Runtime>::NoPermission
		);

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
			Nft::do_transfer(account(1), account(2), (0, 1)),
			Error::<Runtime>::AssetInfoNotFound
		);

		init_test_stackable_nft(origin.clone());

		assert_noop!(
			Nft::do_transfer(account(1), account(2), (0, 1)),
			Error::<Runtime>::InvalidAssetType
		);

		assert_ok!(Nft::transfer_stackable_nft(
			origin.clone(),
			account(2),
			(0, 1),
			100u32.into()
		));

		assert_noop!(
			Nft::do_transfer(account(1), account(2), (0, 1)),
			Error::<Runtime>::InvalidAssetType
		);
	})
}

#[test]
fn do_transfer_should_fail_if_bound_to_address() {
	let origin = RuntimeOrigin::signed(account(1));
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Nft::do_transfer(account(1), account(2), (0, 0)),
			Error::<Runtime>::ClassIdNotFound
		);

		init_bound_to_address_nft(origin.clone());

		// Owner allowed to transfer
		assert_ok!(Nft::transfer(origin.clone(), account(2), (0, 0)));

		let event = mock::RuntimeEvent::Nft(crate::Event::TransferedNft(account(1), account(2), 0, (0, 0)));
		assert_eq!(last_event(), event);

		// Reject ownership if account(2) tries to transfer
		assert_noop!(
			Nft::do_transfer(account(2), account(1), (0, 0)),
			Error::<Runtime>::NonTransferable
		);
	})
}

#[test]
fn do_check_nft_ownership_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(account(1));
		init_test_nft(origin.clone());
		assert_ok!(Nft::check_nft_ownership(&account(1), &(CLASS_ID, TOKEN_ID)), true);
		assert_ok!(Nft::check_nft_ownership(&account(2), &(CLASS_ID, TOKEN_ID)), false);
	})
}

#[test]
fn do_check_nft_ownership_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Nft::check_nft_ownership(&account(1), &(CLASS_ID, TOKEN_ID)),
			Error::<Runtime>::AssetInfoNotFound
		);
	})
}

#[test]
fn do_withdraw_funds_from_class_fund_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(account(1));
		assert_noop!(
			Nft::withdraw_funds_from_class_fund(origin.clone(), NON_EXISTING_CLASS_ID),
			Error::<Runtime>::ClassIdNotFound
		);
		assert_ok!(Nft::create_group(RuntimeOrigin::root(), vec![1], vec![1]));
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
		let non_class_owner_origin = RuntimeOrigin::signed(account(2));
		assert_noop!(
			Nft::withdraw_funds_from_class_fund(non_class_owner_origin, CLASS_ID),
			Error::<Runtime>::NoPermission
		);
	})
}

#[test]
fn do_withdraw_funds_from_class_fund_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(account(1));
		init_test_nft(origin.clone());
		let class_fund: AccountId = <Runtime as Config>::PalletId::get().into_sub_account_truncating(CLASS_ID);
		assert_ok!(<Runtime as Config>::Currency::transfer(
			origin.clone(),
			class_fund.clone(),
			100
		));
		assert_eq!(free_native_balance(account(1)), 99896);
		assert_eq!(free_native_balance(class_fund.clone()), 100);
		assert_ok!(Nft::withdraw_funds_from_class_fund(origin.clone(), CLASS_ID));
		assert_eq!(free_native_balance(account(1)), 99995);
		assert_eq!(free_native_balance(class_fund), 1);
	})
}

#[test]
fn setting_hard_limit_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(account(1));
		let failing_origin = RuntimeOrigin::signed(account(2));
		assert_ok!(Nft::create_group(RuntimeOrigin::root(), vec![1], vec![1],));
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
		let origin = RuntimeOrigin::signed(account(1));
		let class_deposit = <Runtime as Config>::ClassMintingFee::get();
		assert_ok!(Nft::create_group(RuntimeOrigin::root(), vec![1], vec![1],));
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
		let event = mock::RuntimeEvent::Nft(crate::Event::HardLimitSet(CLASS_ID));
		assert_eq!(last_event(), event);
	})
}

#[test]
fn force_updating_total_issuance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(account(1));
		let _class_deposit = <Runtime as Config>::ClassMintingFee::get();
		assert_ok!(Nft::create_group(RuntimeOrigin::root(), vec![1], vec![1],));
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
		assert_ok!(Nft::force_update_total_issuance(
			RuntimeOrigin::root(),
			CLASS_ID,
			TOKEN_ID,
			TOKEN_ID_2
		));
		assert_eq!(
			NftModule::<Runtime>::classes(CLASS_ID).unwrap().total_issuance,
			TOKEN_ID_2
		);
		let event = mock::RuntimeEvent::Nft(crate::Event::ClassTotalIssuanceUpdated(CLASS_ID, TOKEN_ID_2));
		assert_eq!(last_event(), event);
	})
}

#[test]
fn force_updating_total_issuance_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(account(1));
		let _class_deposit = <Runtime as Config>::ClassMintingFee::get();
		assert_ok!(Nft::create_group(RuntimeOrigin::root(), vec![1], vec![1],));
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
			Nft::force_update_total_issuance(RuntimeOrigin::root(), NON_EXISTING_CLASS_ID, TOKEN_ID, TOKEN_ID_2),
			Error::<Runtime>::ClassIdNotFound
		);

		assert_noop!(
			Nft::force_update_total_issuance(RuntimeOrigin::root(), CLASS_ID, TOKEN_ID_1, TOKEN_ID_2),
			Error::<Runtime>::InvalidCurrentTotalIssuance
		);
	})
}

#[test]
fn force_updating_new_royal_fee_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(account(1));
		let _class_deposit = <Runtime as Config>::ClassMintingFee::get();
		assert_ok!(Nft::create_group(RuntimeOrigin::root(), vec![1], vec![1],));
		assert_ok!(Nft::create_class(
			origin.clone(),
			vec![1],
			test_attributes(1),
			COLLECTION_ID,
			TokenType::Transferable,
			CollectionType::Collectable,
			Perbill::from_percent(20u32),
			None
		));
		assert_ok!(Nft::force_update_royalty_fee(
			RuntimeOrigin::root(),
			CLASS_ID,
			Perbill::from_percent(0u32)
		));

		assert_eq!(
			NftModule::<Runtime>::classes(CLASS_ID).unwrap().data.royalty_fee,
			Perbill::from_percent(0u32)
		);
		let event = mock::RuntimeEvent::Nft(crate::Event::ClassRoyaltyFeeUpdated(
			CLASS_ID,
			Perbill::from_percent(0u32),
		));
		assert_eq!(last_event(), event);
	})
}

#[test]
fn force_updating_new_royal_fee_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(account(2));
		let _class_deposit = <Runtime as Config>::ClassMintingFee::get();
		assert_ok!(Nft::create_group(RuntimeOrigin::root(), vec![1], vec![1],));
		assert_ok!(Nft::create_class(
			origin.clone(),
			vec![1],
			test_attributes(1),
			COLLECTION_ID,
			TokenType::Transferable,
			CollectionType::Collectable,
			Perbill::from_percent(20u32),
			None
		));
		// Non-root signer is not allowed
		assert_noop!(
			Nft::force_update_royalty_fee(RuntimeOrigin::signed(account(1)), CLASS_ID, Perbill::from_percent(0u32)),
			BadOrigin
		);

		// New royalty fee should not exceed the limit
		assert_noop!(
			Nft::force_update_royalty_fee(RuntimeOrigin::root(), CLASS_ID, Perbill::from_percent(u32::MAX)),
			Error::<Runtime>::RoyaltyFeeExceedLimit
		);
	})
}

#[test]
fn validate_signature() {
	ExtBuilder::default().build().execute_with(|| {
		let alice_pair = sp_core::sr25519::Pair::from_string("//Alice", None).unwrap();
		let alice_signer = MultiSigner::Sr25519(alice_pair.public());
		let alice = alice_signer.clone().into_account();
		let mint_data: PreSignedMint<ClassId, TokenId, AccountId, BlockNumber, Balance> = PreSignedMint {
			class_id: 0,
			attributes: test_attributes(1),
			metadata: vec![],
			only_account: None,
			mint_price: None,
			token_id: None,
			expired: 1000u64,
		};
		let encoded_data = Encode::encode(&mint_data);
		let signature = MultiSignature::Sr25519(alice_pair.sign(&encoded_data));
		assert_ok!(Nft::validate_signature(&encoded_data, &signature, &alice));

		let mut wrapped_data: Vec<u8> = Vec::new();
		wrapped_data.extend(b"<Bytes>");
		wrapped_data.extend(&encoded_data);
		wrapped_data.extend(b"</Bytes>");

		let signature = MultiSignature::Sr25519(alice_pair.sign(&wrapped_data));
		assert_ok!(Nft::validate_signature(&encoded_data, &signature, &alice));
	})
}

#[test]
fn pre_signed_mints_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = account(1);
		let user_1_pair = sp_core::sr25519::Pair::from_string("//Alice", None).unwrap();
		let user_1_signer = MultiSigner::Sr25519(user_1_pair.public());
		let user_1 = user_1_signer.clone().into_account();
		let mint_data: PreSignedMint<ClassId, TokenId, AccountId, BlockNumber, Balance> = PreSignedMint {
			class_id: 0,
			attributes: test_attributes(1),
			metadata: vec![],
			only_account: None,
			mint_price: None,
			token_id: None,
			expired: 1000u64,
		};
		let message = Encode::encode(&mint_data);
		let signature = MultiSignature::Sr25519(user_1_pair.sign(&message));
		assert_ok!(Balances::transfer(
			RuntimeOrigin::signed(alice.clone()),
			user_1.clone(),
			100
		));

		init_test_nft(RuntimeOrigin::signed(user_1.clone()));

		// User id already signed message so Alice should able to mint nft from pre-signed message
		assert_ok!(Nft::mint_pre_signed(
			RuntimeOrigin::signed(alice.clone()),
			Box::new(mint_data.clone()),
			signature.clone(),
			user_1.clone(),
		));
		assert_eq!(OrmlNft::tokens_by_owner((alice, 0, 0)), ());
	})
}

#[test]
fn pre_signed_mint_should_work_with_only_account() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = account(1);
		let user_1_pair = sp_core::sr25519::Pair::from_string("//Alice", None).unwrap();
		let user_1_signer = MultiSigner::Sr25519(user_1_pair.public());
		let user_1 = user_1_signer.clone().into_account();
		let mint_data: PreSignedMint<ClassId, TokenId, AccountId, BlockNumber, Balance> = PreSignedMint {
			class_id: 0,
			attributes: test_attributes(1),
			metadata: vec![],
			only_account: Some(account(2)),
			mint_price: None,
			token_id: None,
			expired: 1000u64,
		};
		let message = Encode::encode(&mint_data);
		let signature = MultiSignature::Sr25519(user_1_pair.sign(&message));
		assert_ok!(Balances::transfer(
			RuntimeOrigin::signed(alice.clone()),
			user_1.clone(),
			100
		));

		init_test_nft(RuntimeOrigin::signed(user_1.clone()));

		assert_noop!(
			Nft::mint_pre_signed(
				RuntimeOrigin::signed(alice.clone()),
				Box::new(mint_data.clone()),
				signature.clone(),
				user_1.clone(),
			),
			Error::<Runtime>::NoPermission
		);

		assert_ok!(Nft::mint_pre_signed(
			RuntimeOrigin::signed(account(2)),
			Box::new(mint_data.clone()),
			signature.clone(),
			user_1.clone(),
		));
		assert_eq!(OrmlNft::tokens_by_owner((account(2), 0, 0)), ());
	})
}

#[test]
fn pre_signed_mint_should_collect_fee_with_mint_price() {
	ExtBuilder::default().build().execute_with(|| {
		let alice = account(1);
		let user_1_pair = sp_core::sr25519::Pair::from_string("//Alice", None).unwrap();
		let user_1_signer = MultiSigner::Sr25519(user_1_pair.public());
		let user_1 = user_1_signer.clone().into_account();
		let mint_data: PreSignedMint<ClassId, TokenId, AccountId, BlockNumber, Balance> = PreSignedMint {
			class_id: 0,
			attributes: test_attributes(1),
			metadata: vec![],
			only_account: None,
			mint_price: Some(50),
			token_id: None,
			expired: 1000u64,
		};
		let message = Encode::encode(&mint_data);
		let signature = MultiSignature::Sr25519(user_1_pair.sign(&message));
		assert_ok!(Balances::transfer(
			RuntimeOrigin::signed(alice.clone()),
			user_1.clone(),
			100
		));
		assert_eq!(Balances::free_balance(user_1.clone()), 100);
		assert_eq!(Balances::free_balance(alice.clone()), 99900);

		init_test_nft(RuntimeOrigin::signed(user_1.clone()));
		assert_eq!(Balances::free_balance(user_1.clone()), 96); // Deduct fee

		assert_ok!(Nft::mint_pre_signed(
			RuntimeOrigin::signed(alice.clone()),
			Box::new(mint_data.clone()),
			signature.clone(),
			user_1.clone(),
		));
		assert_eq!(Balances::free_balance(user_1.clone()), 146); // Get 50 mint fees from NFT
		assert_eq!(Balances::free_balance(alice.clone()), 99849); // Pay 1 mint fee for protocol and 50 mint price = 99900 - 51
		assert_eq!(OrmlNft::tokens_by_owner((account(2), 0, 0)), ());
	})
}
