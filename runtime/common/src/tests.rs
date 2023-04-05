use frame_support::assert_noop;
use hex_literal::hex;
use sp_core::{H160, U256, ByteArray};

use precompile_utils::data::EvmDataWriter;
use precompile_utils::testing::*;
use primitives::FungibleTokenId;

use crate::currencies::Action;
use crate::mock::*;

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

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
		.with_balances(vec![(ALICE, 100000)])
		.build()
		.execute_with(|| {
			Currencies::update_balance(Origin::root(), ALICE, FungibleTokenId::MiningResource(0), 100000);
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
		.with_balances(vec![(ALICE, 100000)])
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
fn balance_of_works() {
	ExtBuilder::default()
		.with_balances(vec![(ALICE, 100000)])
		.build()
		.execute_with(|| {
			let mut evm_writer = EvmDataWriter::new_with_selector(Action::BalanceOf);
			evm_writer.write_pointer(charlie_evm_addr().to_fixed_bytes().to_vec());
			precompiles()
				.prepare_test(alice_evm_addr(), neer_evm_address(), evm_writer.build())
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(0u64)).build());

			let alice: AccountId = into_account_id(alice_evm_addr());
			//assert_eq!(alice.as_slice(), ALICE.as_slice());
			Currencies::update_balance(Origin::root(), alice, FungibleTokenId::MiningResource(0), 100000);

			evm_writer = EvmDataWriter::new_with_selector(Action::BalanceOf);
			evm_writer.write_pointer(alice_evm_addr().to_fixed_bytes().to_vec());
			precompiles()
				.prepare_test(alice_evm_addr(), bit_evm_address(), evm_writer.build())
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(100000u64)).build());
		});
}

#[test]
fn transfer_works() {
	ExtBuilder::default()
		.with_balances(vec![(ALICE, 100000), (BOB, 200000)])
		.build()
		.execute_with(|| {
			let mut evm_writer = EvmDataWriter::new_with_selector(Action::Transfer);
			evm_writer.write_pointer(1000u64.to_be_bytes().to_vec());
			evm_writer.write_pointer(bob_evm_addr().to_fixed_bytes().to_vec());

			let alice: AccountId = into_account_id(alice_evm_addr());
			//assert_eq!(alice.as_slice(), ALICE.as_slice());
			Currencies::update_balance(Origin::root(), alice, FungibleTokenId::MiningResource(0), 100000);

			let bob: AccountId = into_account_id(bob_evm_addr());
			//assert_eq!(alice.as_slice(), ALICE.as_slice());
			Currencies::update_balance(Origin::root(), bob, FungibleTokenId::MiningResource(0), 200000);

			precompiles()
				.prepare_test(alice_evm_addr(), bit_evm_address(), evm_writer.build())
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(1u64).build());
		});
}

/*
	#[test]
	fn handles_invalid_currency_precompile_selector() {
		ExtBuilder::default().build().execute_with(|| {
			precompiles()
				.prepare_test(
					currency_precompile_evm_addr(),
					neer_evm_address(),
					EvmDataWriter::new_with_selector(9876u32).build(),
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_reverts(|output| output == b"invalid currency precompile selector")
		});
	}



*/
