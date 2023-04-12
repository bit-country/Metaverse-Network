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

use core_primitives::{Attributes, CollectionType, NftMetadata, TokenType};

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
	Nft::mint(owner.clone(), CLASS_ID, vec![1], test_attributes(1), 1);
}

// Nft Precompile Tests
#[test]
fn get_class_fund_balance_works() {
	ExtBuilder::default()
		.with_balances(vec![(alice_account_id(), 100000)])
		.build()
		.execute_with(|| {
			init_test_nft(Origin::signed(alice_account_id()));

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
				.execute_returns(EvmDataWriter::new().write(U256::from(2u64)).build());
		});
}
