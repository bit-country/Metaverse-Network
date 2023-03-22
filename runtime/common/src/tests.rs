use frame_support::assert_noop;
use hex_literal::hex;
use sp_core::{H160, U256};

use precompile_utils::data::EvmDataWriter;
use precompile_utils::testing::*;

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
				H160(hex!("0000000000000000000500000000000000000000")),
				H160(hex!("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b")),
				EvmDataWriter::new_with_selector(Action::TotalSupply).build(),
			)
			.expect_cost(0)
			.expect_no_logs()
			.execute_returns(EvmDataWriter::new().write(U256::from(3500u64)).build());
	});
}
/*

   #[test]
   fn total_supply_works() {
	   new_test_ext().execute_with(|| {
		   let context = Context {
			   address: H160(hex!("0000000000000000000100000000000000000000")),
			   caller: H160(hex!("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b")),
			   apparent_value: U256::zero(),
		   };
		   let code_address: H160 =  H160(hex!("0000000000000000000100000000000000000000"));
		   let mut handle = MockHandle::new(code_address, context);
		   let action_input = Action::TotalSuply.as_bytes().to_vec();
		   handle.input = action_input;

		   let response = MultiCurrencyPrecompile::execute(&handle);
		   assert_eq!(resp.exit_status, ExitSucceed::Returned);
	   });
   }

   #[test]
   fn balance_of_works() {
	   new_test_ext().execute_with(|| {
		   let context = Context {
			   address: H160(hex!("000000000000000000100000000000000000000")),
			   caller: H160(hex!("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b")),
			   apparent_value: U256::zero(),
		   };
		   let code_address: H160 =  H160(hex!("0000000000000000000100000000000000000000"));
		   let handle = MockHandle::new(code_address, context);
		   let action_input = Action::BalanceOf.as_bytes().to_vec();
		   handle.input = action_input;

		   let response = MultiCurrencyPrecompile::execute(&handle);
		   assert_eq!(resp.exit_status, ExitSucceed::Returned);

	   });
   }

   #[test]
   fn transfer_works() {
	   new_test_ext().execute_with(|| {
		   let context = Context {
			   address: H160(hex!("0000000000000000000100000000000000000000")),
			   caller: H160(hex!("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b")),
			   apparent_value: U256::zero(),
		   };
		   let code_address: H160 =  H160(hex!("0000000000000000000100000000000000000000"));
		   let handle = MockHandle::new(code_address, context);
		   let action_input = Action::Transfer.as_bytes().to_vec();
		   handle.input = action_input;

		   let response = MultiCurrencyPrecompile::execute(&handle);
		   assert_eq!(resp.exit_status, ExitSucceed::Returned);
	   });
   }
*/
