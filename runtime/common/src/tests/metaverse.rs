use frame_support::{assert_noop, assert_ok};
use hex_literal::hex;
use precompile_utils::data::{Address, Bytes, EvmDataWriter};
use precompile_utils::testing::*;
use sp_core::{ByteArray, H160, U256};
use sp_runtime::traits::{AccountIdConversion, Zero};

use crate::metaverse::Action;
use crate::mock::*;
use evm_mapping::AddressMapping as AddressMappingEvm;
use evm_mapping::AddressMapping;
use orml_traits::BasicCurrency;

use core_primitives::MetaverseMetadata;
use primitives::{FungibleTokenId, MetaverseId};

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

fn init_test_metaverse(owner: RuntimeOrigin) {
	let metaverse_treasury: AccountId =
		<Runtime as metaverse_pallet::Config>::MetaverseTreasury::get().into_account_truncating();

	Currencies::update_balance(
		RuntimeOrigin::root(),
		metaverse_treasury.clone(),
		FungibleTokenId::NativeToken(0),
		1000,
	);

	Nft::create_group(RuntimeOrigin::root(), vec![1], vec![1]);
	assert_ok!(Metaverse::create_metaverse(owner, vec![2]));
}

// Metaverse Precompile Tests
#[test]
fn get_metaverse_metadata_works() {
	ExtBuilder::default()
		.with_balances(vec![(alice_account_id(), 100000)])
		.build()
		.execute_with(|| {
			init_test_metaverse(RuntimeOrigin::signed(alice_account_id()));

			let metaverse_metadata: MetaverseMetadata = vec![2];

			precompiles()
				.prepare_test(
					alice_evm_addr(),
					metaverse_precompile_address(),
					EvmDataWriter::new_with_selector(Action::GetMetaverseMetadata)
						.write(U256::from(METAVERSE_ID))
						.build(),
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write(Bytes::from(metaverse_metadata.as_slice()))
						.build(),
				);
		});
}

#[test]
fn get_metaverse_owner_works() {
	ExtBuilder::default()
		.with_balances(vec![(alice_account_id(), 100000)])
		.build()
		.execute_with(|| {
			init_test_metaverse(RuntimeOrigin::signed(alice_account_id()));

			precompiles()
				.prepare_test(
					alice_evm_addr(),
					metaverse_precompile_address(),
					EvmDataWriter::new_with_selector(Action::GetMetaverseOwner)
						.write(U256::from(METAVERSE_ID))
						.build(),
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(Address::from(alice_evm_addr())).build());
		});
}

#[test]
fn get_metaverse_fund_balance_works() {
	ExtBuilder::default()
		.with_balances(vec![(alice_account_id(), 100000)])
		.build()
		.execute_with(|| {
			init_test_metaverse(RuntimeOrigin::signed(alice_account_id()));

			precompiles()
				.prepare_test(
					alice_evm_addr(),
					metaverse_precompile_address(),
					EvmDataWriter::new_with_selector(Action::GetMetaverseFundBalance)
						.write(U256::from(METAVERSE_ID))
						.build(),
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(999u64)).build());
		});
}

#[test]
fn create_metaverse_works() {
	ExtBuilder::default()
		.with_balances(vec![(alice_account_id(), 100000)])
		.build()
		.execute_with(|| {
			let metaverse_treasury: AccountId =
				<Runtime as metaverse_pallet::Config>::MetaverseTreasury::get().into_account_truncating();

			Currencies::update_balance(
				RuntimeOrigin::root(),
				metaverse_treasury.clone(),
				FungibleTokenId::NativeToken(0),
				1000,
			);

			Nft::create_group(RuntimeOrigin::root(), vec![1], vec![1]);

			assert_eq!(Metaverse::get_metaverse(METAVERSE_ID).is_some(), false);

			EvmMapping::claim_default_account(RuntimeOrigin::signed(alice_account_id()));

			precompiles()
				.prepare_test(
					alice_evm_addr(),
					metaverse_precompile_address(),
					EvmDataWriter::new_with_selector(Action::CreateMetaverse)
						.write(Vec::<u8>::from(vec![2u8])) // metaverse metadata
						.build(),
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(1u64)).build());

			assert_eq!(Metaverse::get_metaverse(METAVERSE_ID).is_some(), true);
		});
}

#[test]
fn withdraw_metaverse_fund_balance_works() {
	ExtBuilder::default()
		.with_balances(vec![(alice_account_id(), 100000)])
		.build()
		.execute_with(|| {
			init_test_metaverse(RuntimeOrigin::signed(alice_account_id()));

			EvmMapping::claim_default_account(RuntimeOrigin::signed(alice_account_id()));

			precompiles()
				.prepare_test(
					alice_evm_addr(),
					metaverse_precompile_address(),
					EvmDataWriter::new_with_selector(Action::WithdrawFromMetaverseFund)
						.write(U256::from(METAVERSE_ID))
						.build(),
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(1u64)).build());

			assert_eq!(
				<Runtime as metaverse_pallet::Config>::Currency::free_balance(&alice_account_id()),
				100998
			);
		});
}

#[test]
fn transfer_metaverse_works() {
	ExtBuilder::default()
		.with_balances(vec![(alice_account_id(), 100000), (bob_account_id(), 50000)])
		.build()
		.execute_with(|| {
			init_test_metaverse(RuntimeOrigin::signed(alice_account_id()));

			assert_eq!(
				Metaverse::get_metaverse_owner(alice_account_id(), METAVERSE_ID),
				Some(())
			);
			EvmMapping::claim_default_account(RuntimeOrigin::signed(alice_account_id()));
			EvmMapping::claim_default_account(RuntimeOrigin::signed(bob_account_id()));

			precompiles()
				.prepare_test(
					alice_evm_addr(),
					metaverse_precompile_address(),
					EvmDataWriter::new_with_selector(Action::TransferMetaverse)
						.write(Address::from(bob_evm_addr()))
						.write(U256::from(METAVERSE_ID))
						.build(),
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(1u64)).build());

			assert_eq!(Metaverse::get_metaverse_owner(bob_account_id(), METAVERSE_ID), Some(()));
		});
}
