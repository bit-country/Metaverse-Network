use frame_support::assert_noop;
use hex_literal::hex;
use sp_core::{ByteArray, H160, U256};

use precompile_utils::data::{Address, EvmDataWriter};
use precompile_utils::testing::*;
use primitives::FungibleTokenId;

use crate::currencies::Action;
use crate::mock::*;
use orml_traits::BasicCurrency;
use orml_traits::MultiCurrency;
use pallet_evm::AddressMapping;
//use currencies_pallet::MultiSocialCurrency;

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
fn total_supply_of_foreign_currencies_works() {
	ExtBuilder::default()
		.with_balances(vec![(alice_account_id(), 100000)])
		.build()
		.execute_with(|| {
			Currencies::update_balance(
				Origin::root(),
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
			Origin::root(),
			alice_account_id(),
			FungibleTokenId::MiningResource(0),
			100000,
		);

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
				Origin::root(),
				alice_account_id(),
				FungibleTokenId::MiningResource(0),
				100000,
			);
			Currencies::update_balance(
				Origin::root(),
				bob_account_id(),
				FungibleTokenId::MiningResource(0),
				150000,
			);

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

