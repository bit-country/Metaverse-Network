use frame_support::assert_noop;
use hex_literal::hex;
use sp_core::{ByteArray, H160, U256};
use sp_runtime::traits::{AccountIdConversion, Zero};
use sp_runtime::Perbill;
use sp_std::collections::btree_map::BTreeMap;

use precompile_utils::data::{Address, EvmDataWriter};
use precompile_utils::testing::*;
use primitives::FungibleTokenId;

use crate::mock::*;
use crate::nft::Action;
use orml_traits::BasicCurrency;
use pallet_evm::AddressMapping;
use orml_nft::Pallet as NftModule;

use core_primitives::{Attributes, CollectionType, NftMetadata, TokenType, NftClassData, NftAssetData};

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

fn test_attributes(x: u8) -> Attributes {
	let mut attr: Attributes = BTreeMap::new();
	attr.insert(vec![x, x + 5], vec![x, x + 10]);
	attr
}

fn init_test_nft(owner: Origin) {
	Nft::create_group(Origin::root(), vec![1], vec![1]);
	Nft::create_class(
		owner.clone(),
		vec![1],
		test_attributes(1),
		COLLECTION_ID,
		TokenType::Transferable,
		CollectionType::Collectable,
		Perbill::from_percent(0u32),
		None,
	);
	Nft::mint(owner.clone(), CLASS_ID, vec![2], test_attributes(1), 1);
}

// Nft Precompile Tests

#[test]
fn get_nft_metadata_works() {
	ExtBuilder::default()
		.with_balances(vec![(alice_account_id(), 100000)])
		.build()
		.execute_with(|| {
			init_test_nft(Origin::signed(alice_account_id()));

			//let expected_metadata = 

			precompiles()
				.prepare_test(
					alice_evm_addr(),
					nft_precompile_address(),
					EvmDataWriter::new_with_selector(Action::GetNftMetadata)
						.write(U256::from(CLASS_ID))
                        .write(U256::from(TOKEN_ID))
						.build(),
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(Vec::<u8>::from(vec![2u8])).build());
		});
}

#[test]
fn get_nft_address_works() {
	ExtBuilder::default()
		.with_balances(vec![(alice_account_id(), 100000)])
		.build()
		.execute_with(|| {
			init_test_nft(Origin::signed(alice_account_id()));

			//let nft_address = Runtime::encode_nf_evm_address()

			precompiles()
				.prepare_test(
					alice_evm_addr(),
					nft_precompile_address(),
					EvmDataWriter::new_with_selector(Action::GetNftAddress)
					    .write(U256::from(CLASS_ID))
                        .write(U256::from(TOKEN_ID))
						.build(),
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(Address::from(nft_address())).build());
		});
}

#[test]
fn get_nft_owner_works() {
	ExtBuilder::default()
		.with_balances(vec![(alice_account_id(), 100000)])
		.build()
		.execute_with(|| {
			init_test_nft(Origin::signed(alice_account_id()));

			precompiles()
				.prepare_test(
					alice_evm_addr(),
					nft_precompile_address(),
					EvmDataWriter::new_with_selector(Action::GetAssetOwner)
                        .write(U256::from(CLASS_ID))
                        .write(U256::from(TOKEN_ID))
						.build(),
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(Address::from(alice_evm_addr())).build());
		});
}

#[test]
fn get_class_fund_balance_works() {
	ExtBuilder::default()
		.with_balances(vec![(alice_account_id(), 100000)])
		.build()
		.execute_with(|| {
			init_test_nft(Origin::signed(alice_account_id()));

			let class_fund: AccountId = <Runtime as nft_pallet::Config>::PalletId::get().into_sub_account_truncating(CLASS_ID);

			Currencies::update_balance(
				Origin::root(),
				class_fund.clone(),
				FungibleTokenId::NativeToken(0),
				1000,
			);

			precompiles()
				.prepare_test(
					alice_evm_addr(),
					nft_precompile_address(),
					EvmDataWriter::new_with_selector(Action::GetClassFundBalance)
						.write(U256::from(CLASS_ID))
						.build(),
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(1000u64)).build());
		});
}

#[test]
fn create_class_works() {
	ExtBuilder::default()
		.with_balances(vec![(alice_account_id(), 100000)])
		.build()
		.execute_with(|| {
			init_test_nft(Origin::signed(alice_account_id()));

			assert_eq!(NftModule::<Runtime>::classes(CLASS_ID_2), None);

			precompiles()
				.prepare_test(
					alice_evm_addr(),
					nft_precompile_address(),
					EvmDataWriter::new_with_selector(Action::CreateClass)
						.write(U256::from(COLLECTION_ID)) // collection id
						.write(Vec::<u8>::from(vec![2u8])) // metadata
						.write(U256::from(1u32)) // royalty fee
						.write(U256::from(10u32))  // class mint limit
						.build(),
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(1u64)).build());
			
			assert_eq!(NftModule::<Runtime>::classes(CLASS_ID_2).is_some(), true);
		});
}

#[test]
fn mint_nft_works() {
	ExtBuilder::default()
		.with_balances(vec![(alice_account_id(), 100000)])
		.build()
		.execute_with(|| {
			init_test_nft(Origin::signed(alice_account_id()));

			assert_eq!(NftModule::<Runtime>::tokens(CLASS_ID, TOKEN_ID_2), None);

			let nft_metadata: NftMetadata = vec![3u8];
			precompiles()
				.prepare_test(
					alice_evm_addr(),
					nft_precompile_address(),
					EvmDataWriter::new_with_selector(Action::MintNfts)
						.write(U256::from(CLASS_ID)) // class id
						.write(Vec::<u8>::from(nft_metadata)) // metadata
						.write(U256::from(1u32))  // quantity
						.build(),
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(1u64)).build());
			
			assert_eq!(NftModule::<Runtime>::tokens(CLASS_ID, TOKEN_ID_2).is_some(), true);
		});
}

#[test]
fn transfer_nft_works() {
	ExtBuilder::default()
		.with_balances(vec![(alice_account_id(), 100000)])
		.build()
		.execute_with(|| {
			init_test_nft(Origin::signed(alice_account_id()));

			assert_eq!(NftModule::<Runtime>::tokens(CLASS_ID, TOKEN_ID).unwrap().owner, alice_account_id());
			precompiles()
				.prepare_test(
					alice_evm_addr(),
					nft_precompile_address(),
					EvmDataWriter::new_with_selector(Action::TransferNft)
						.write(Address::from(bob_evm_addr()))
						.write(U256::from(CLASS_ID))
						.write(U256::from(TOKEN_ID))
						.build(),
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(1u64)).build());

			assert_eq!(NftModule::<Runtime>::tokens(CLASS_ID, TOKEN_ID).unwrap().owner, bob_account_id());
		});
}

#[test]
fn burn_nft_works() {
	ExtBuilder::default()
		.with_balances(vec![(alice_account_id(), 100000)])
		.build()
		.execute_with(|| {
			init_test_nft(Origin::signed(alice_account_id()));

			precompiles()
				.prepare_test(
					alice_evm_addr(),
					nft_precompile_address(),
					EvmDataWriter::new_with_selector(Action::BurnNft)
						.write(U256::from(CLASS_ID))
                        .write(U256::from(TOKEN_ID))
						.build(),
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(1u64)).build());
			assert_eq!(NftModule::<Runtime>::tokens(CLASS_ID, TOKEN_ID), None);
		});
}

#[test]
fn withdraw_from_class_fund_works() {
	ExtBuilder::default()
		.with_balances(vec![(alice_account_id(), 100000)])
		.build()
		.execute_with(|| {
			init_test_nft(Origin::signed(alice_account_id()));

			let class_fund: AccountId = <Runtime as nft_pallet::Config>::PalletId::get().into_sub_account_truncating(CLASS_ID);

			Currencies::update_balance(
				Origin::root(),
				class_fund.clone(),
				FungibleTokenId::NativeToken(0),
				1000,
			);
			assert_eq!(
				<Runtime as currencies_pallet::Config>::NativeCurrency::free_balance(&class_fund),
				1000
			);

			precompiles()
				.prepare_test(
					alice_evm_addr(),
					nft_precompile_address(),
					EvmDataWriter::new_with_selector(Action::WithdrawFromClassFund)
						.write(U256::from(CLASS_ID))
						.write(Address::from(alice_evm_addr()))
						.build(),
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(1u64)).build());
		
			assert_eq!(
				<Runtime as currencies_pallet::Config>::NativeCurrency::free_balance(&class_fund),
				0
			);
		});
}
