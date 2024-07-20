#![cfg(test)]

use frame_support::{assert_err, assert_noop, assert_ok};
use sp_runtime::{traits::BadOrigin, Perbill};
use std::collections::BTreeMap;

use super::*;
use mock::{RuntimeCall, RuntimeEvent, RuntimeOrigin, *};

fn get_test_collection_data() -> NftGroupCollectionData {
	NftGroupCollectionData {
		name: "Pioneer NFT Collections".as_bytes().to_vec(),
		properties: "Pioneer NFT Collections Properties".as_bytes().to_vec(),
	}
}

fn init_test_collection() {
	assert_ok!(Nft::create_group(RuntimeOrigin::root(), vec![1], vec![1],));
}

fn init_test_class(owner: RuntimeOrigin) {
	init_test_collection();
	assert_ok!(Nft::create_class(
		owner.clone(),
		vec![1],
		test_attributes(1),
		0,
		TokenType::Transferable,
		CollectionType::Collectable,
		Perbill::from_percent(0u32),
		None
	));
}

fn get_test_metadata() -> NftMetadata {
	let metadata: NftMetadata = "Migration".as_bytes().to_vec();
	metadata
}

fn test_attributes(x: u8) -> core_primitives::Attributes {
	let mut attr: core_primitives::Attributes = BTreeMap::new();
	attr.insert(vec![x, x + 5], vec![x, x + 10]);
	attr
}

fn get_test_class_data() -> NftClassData<Balance> {
	NftClassData {
		deposit: 1,
		attributes: test_attributes(10),
		token_type: TokenType::Transferable,
		collection_type: CollectionType::Collectable,
		is_locked: false,
		royalty_fee: Perbill::from_percent(4),
		mint_limit: Some(100),
		total_minted_tokens: 0,
	}
}

fn get_test_token_data() -> NftAssetData<Balance> {
	NftAssetData {
		deposit: 1,
		attributes: test_attributes(20),
		is_locked: false,
	}
}

#[test]
fn start_migration_should_not_work_from_non_admin_account() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(NftMigration::start_migration(RuntimeOrigin::signed(BOB)), BadOrigin);
	});
}

#[test]
fn start_migration_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(NftMigration::is_migration_active(), false);
		assert_ok!(NftMigration::start_migration(RuntimeOrigin::signed(ALICE)));
		assert_eq!(last_event(), RuntimeEvent::NftMigration(crate::Event::MigrationStarted));
		assert_eq!(NftMigration::is_migration_active(), true);
	});
}

#[test]
fn migrating_single_collection_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let collection_id = Nft::get_next_collection_id();
		assert_ok!(NftMigration::migrate_collection_unsigned(
			RuntimeOrigin::none(),
			collection_id,
			get_test_collection_data()
		));
		//assert_eq!(Nft::get_group_collection(collection_id), Some(get_test_collection_data()));
		//assert_eq!(Nft::get_next_collection_id(), collection_id + 1);
	});
}

#[test]
fn migrating_single_collection_with_invalid_collection_id_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let collection_id = Nft::get_next_collection_id() + 1;
		assert_noop!(
			NftMigration::migrate_collection_unsigned(RuntimeOrigin::none(), collection_id, get_test_collection_data()),
			Error::<Runtime>::InconsistentMigrationData
		);
	});
}

#[test]
fn migrating_single_class_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		init_test_collection();
		let class_id = Nft::get_next_class_id();
		assert_ok!(NftMigration::migrate_class_unsigned(
			RuntimeOrigin::none(),
			ALICE,
			COLLECTION_ID,
			class_id,
			get_test_metadata(),
			get_test_class_data(),
		));
	});
}

#[test]
fn migrating_single_class_with_invalid_class_id_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		init_test_collection();
		let class_id = Nft::get_next_class_id() + 1;
		assert_noop!(
			NftMigration::migrate_class_unsigned(
				RuntimeOrigin::none(),
				ALICE,
				COLLECTION_ID,
				class_id,
				get_test_metadata(),
				get_test_class_data(),
			),
			Error::<Runtime>::InconsistentMigrationData
		);
	});
}

#[test]
fn migrating_single_token_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		init_test_class(RuntimeOrigin::signed(ALICE));
		let token_id = Nft::get_next_token_id(CLASS_ID);
		assert_ok!(NftMigration::migrate_token_unsigned(
			RuntimeOrigin::none(),
			ALICE,
			CLASS_ID,
			token_id,
			get_test_metadata(),
			get_test_token_data(),
		));
	});
}

#[test]
fn migrating_single_token_with_invalid_class_id_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		init_test_class(RuntimeOrigin::signed(ALICE));
		let token_id = Nft::get_next_token_id(CLASS_ID) + 1;
		assert_noop!(
			NftMigration::migrate_token_unsigned(
				RuntimeOrigin::none(),
				ALICE,
				CLASS_ID,
				token_id,
				get_test_metadata(),
				get_test_token_data(),
			),
			Error::<Runtime>::InconsistentMigrationData
		);
	});
}
