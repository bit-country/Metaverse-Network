use asset_manager::BalanceOf;
use frame_support::assert_noop;
use hex_literal::hex;
use sp_core::{H160, U256};
use sp_runtime::traits::Zero;
use sp_std::boxed::Box;

use precompile_utils::data::{Address, Bytes, EvmDataWriter};
use precompile_utils::testing::*;
use primitives::{AssetMetadata, FungibleTokenId};

use crate::currencies::Action;
use crate::mock::*;
use orml_traits::BasicCurrency;
use orml_traits::MultiCurrency;

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

// Currency Precompile Tests
#[test]
fn handles_invalid_currency_id() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				alice_evm_addr(),
				H160(hex!("0000000000000000000500000000000000000000")),
				EvmDataWriter::new_with_selector(Action::TotalSupply).build(),
			)
			.expect_cost(0)
			.expect_no_logs()
			.execute_reverts(|output| output == b"invalid currency id")
	});
}

#[test]
fn handles_non_supported_allowance() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				alice_evm_addr(),
				neer_evm_address(),
				EvmDataWriter::new_with_selector(Action::Allowance).build(),
			)
			.expect_cost(0)
			.expect_no_logs()
			.execute_error(pallet_evm::ExitError::Other("not supported".into()))
	});
}

#[test]
fn handles_non_supported_approve() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				alice_evm_addr(),
				neer_evm_address(),
				EvmDataWriter::new_with_selector(Action::Approve).build(),
			)
			.expect_cost(0)
			.expect_no_logs()
			.execute_error(pallet_evm::ExitError::Other("not supported".into()))
	});
}

#[test]
fn handles_non_supported_transfer_from() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				alice_evm_addr(),
				neer_evm_address(),
				EvmDataWriter::new_with_selector(Action::TransferFrom).build(),
			)
			.expect_cost(0)
			.expect_no_logs()
			.execute_error(pallet_evm::ExitError::Other("not supported".into()))
	});
}

#[test]
fn name_works() {
	ExtBuilder::default()
		.with_balances(vec![(alice_account_id(), 100000)])
		.build()
		.execute_with(|| {
			let asset_metadata = AssetMetadata {
				name: "NEER".as_bytes().to_vec(),
				symbol: "NEER".as_bytes().to_vec(),
				decimals: 18u8,
				minimal_balance: Zero::zero(),
			};
			AssetManager::update_native_asset_metadata(
				RuntimeOrigin::root(),
				FungibleTokenId::NativeToken(0),
				Box::new(asset_metadata),
			);
			precompiles()
				.prepare_test(
					alice_evm_addr(),
					neer_evm_address(),
					EvmDataWriter::new_with_selector(Action::Name).build(),
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(Bytes::from("NEER".as_bytes())).build());
		});
}

#[test]
fn symbol_works() {
	ExtBuilder::default()
		.with_balances(vec![(alice_account_id(), 100000)])
		.build()
		.execute_with(|| {
			let asset_metadata = AssetMetadata {
				name: "NEER".as_bytes().to_vec(),
				symbol: "NEER".as_bytes().to_vec(),
				decimals: 18u8,
				minimal_balance: Zero::zero(),
			};
			AssetManager::update_native_asset_metadata(
				RuntimeOrigin::root(),
				FungibleTokenId::NativeToken(0),
				Box::new(asset_metadata),
			);
			precompiles()
				.prepare_test(
					alice_evm_addr(),
					neer_evm_address(),
					EvmDataWriter::new_with_selector(Action::Symbol).build(),
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(Bytes::from("NEER".as_bytes())).build());
		});
}

#[test]
fn decimals_works() {
	ExtBuilder::default()
		.with_balances(vec![(alice_account_id(), 100000)])
		.build()
		.execute_with(|| {
			let asset_metadata = AssetMetadata {
				name: "NEER".as_bytes().to_vec(),
				symbol: "NEER".as_bytes().to_vec(),
				decimals: 18u8,
				minimal_balance: Zero::zero(),
			};
			AssetManager::update_native_asset_metadata(
				RuntimeOrigin::root(),
				FungibleTokenId::NativeToken(0),
				Box::new(asset_metadata),
			);
			precompiles()
				.prepare_test(
					alice_evm_addr(),
					neer_evm_address(),
					EvmDataWriter::new_with_selector(Action::Decimals).build(),
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(18)).build());
		});
}

#[test]
fn total_supply_of_foreign_currencies_works() {
	ExtBuilder::default()
		.with_balances(vec![(alice_account_id(), 100000)])
		.build()
		.execute_with(|| {
			Currencies::update_balance(
				RuntimeOrigin::root(),
				alice_account_id(),
				FungibleTokenId::MiningResource(0),
				100000,
			);
			precompiles()
				.prepare_test(
					alice_evm_addr(),
					bit_evm_address(),
					EvmDataWriter::new_with_selector(Action::TotalSupply).build(),
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(100000u64)).build());
		});
}

#[test]
fn total_supply_of_native_currencies_works() {
	ExtBuilder::default()
		.with_balances(vec![(alice_account_id(), 100000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					alice_evm_addr(),
					neer_evm_address(),
					EvmDataWriter::new_with_selector(Action::TotalSupply).build(),
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(100000u64)).build());
		});
}

#[test]
fn balance_of_foreign_currencies_works() {
	ExtBuilder::default().build().execute_with(|| {
		Currencies::update_balance(
			RuntimeOrigin::root(),
			alice_account_id(),
			FungibleTokenId::MiningResource(0),
			100000,
		);
		EvmMapping::claim_default_account(RuntimeOrigin::signed(alice_account_id()));

		precompiles()
			.prepare_test(
				alice_evm_addr(),
				bit_evm_address(),
				EvmDataWriter::new_with_selector(Action::BalanceOf)
					.write(Address::from(alice_evm_addr()))
					.build(),
			)
			.expect_cost(0)
			.expect_no_logs()
			.execute_returns(EvmDataWriter::new().write(U256::from(100000u64)).build());
	});
}

#[test]
fn balance_of_native_currencies_works() {
	ExtBuilder::default()
		.with_balances(vec![(alice_account_id(), 100000)])
		.build()
		.execute_with(|| {
			let mut evm_writer = EvmDataWriter::new_with_selector(Action::BalanceOf);
			EvmMapping::claim_default_account(RuntimeOrigin::signed(alice_account_id()));
			precompiles()
				.prepare_test(
					alice_evm_addr(),
					neer_evm_address(),
					EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address::from(alice_evm_addr()))
						.build(),
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(100000u64)).build());
		});
}

#[test]
fn transfer_foreign_currencies_works() {
	ExtBuilder::default()
		.with_balances(vec![(alice_account_id(), 10000), (bob_account_id(), 5000)])
		.build()
		.execute_with(|| {
			Currencies::update_balance(
				RuntimeOrigin::root(),
				alice_account_id(),
				FungibleTokenId::MiningResource(0),
				100000,
			);
			Currencies::update_balance(
				RuntimeOrigin::root(),
				bob_account_id(),
				FungibleTokenId::MiningResource(0),
				150000,
			);
			EvmMapping::claim_default_account(RuntimeOrigin::signed(alice_account_id()));
			EvmMapping::claim_default_account(RuntimeOrigin::signed(bob_account_id()));

			let mut evm_writer = EvmDataWriter::new_with_selector(Action::Transfer);
			evm_writer.write_pointer(1000u64.to_be_bytes().to_vec());
			evm_writer.write_pointer(bob_evm_addr().to_fixed_bytes().to_vec());

			precompiles()
				.prepare_test(
					alice_evm_addr(),
					bit_evm_address(),
					EvmDataWriter::new_with_selector(Action::Transfer)
						.write(Address::from(bob_evm_addr()))
						.write(U256::from(1000u64))
						.build(),
				)
				.expect_cost(1756)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(1u64).build());

			assert_eq!(
				<Runtime as currencies_pallet::Config>::MultiSocialCurrency::free_balance(
					FungibleTokenId::MiningResource(0),
					&bob_account_id()
				),
				151000
			);
			assert_eq!(
				<Runtime as currencies_pallet::Config>::MultiSocialCurrency::free_balance(
					FungibleTokenId::MiningResource(0),
					&alice_account_id()
				),
				99000
			);
		});
}

#[test]
fn transfer_native_currencies_works() {
	ExtBuilder::default()
		.with_balances(vec![(alice_account_id(), 100000), (bob_account_id(), 150000)])
		.build()
		.execute_with(|| {
			EvmMapping::claim_default_account(RuntimeOrigin::signed(alice_account_id()));
			EvmMapping::claim_default_account(RuntimeOrigin::signed(bob_account_id()));

			precompiles()
				.prepare_test(
					alice_evm_addr(),
					neer_evm_address(),
					EvmDataWriter::new_with_selector(Action::Transfer)
						.write(Address::from(bob_evm_addr()))
						.write(U256::from(1000u64))
						.build(),
				)
				.expect_cost(1756)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(1u64).build());

			assert_eq!(
				<Runtime as currencies_pallet::Config>::NativeCurrency::free_balance(&bob_account_id()),
				151000
			);
			assert_eq!(
				<Runtime as currencies_pallet::Config>::NativeCurrency::free_balance(&alice_account_id()),
				99000
			);
		});
}
