use frame_support::{assert_noop, assert_ok};
use hex_literal::hex;
use sp_core::{ByteArray, H160, U256};
use sp_runtime::traits::{AccountIdConversion, Zero};
use precompile_utils::data::{Address, Bytes, EvmDataWriter};
use precompile_utils::testing::*;

use crate::mock::*;
use crate::metaverse::Action;
use evm_mapping::AddressMapping as AddressMappingEvm;
use orml_traits::BasicCurrency;
use pallet_evm::AddressMapping;

use core_primitives::MetaverseMetadata;
use primitives::{MetaverseId, FungibleTokenId};

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

fn init_test_metaverse(owner: RuntimeOrigin) {
    let metaverse_fund: AccountId =
    <Runtime as metaverse_pallet::Config>::MetaverseTreasury::get().into_account_truncating();

    Currencies::update_balance(
        RuntimeOrigin::root(),
        metaverse_fund.clone(),
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
					EvmDataWriter::new_with_selector(Action::GetMetaverse)
						.write(U256::from(METAVERSE_ID))
						.build(),
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(Bytes::from(metaverse_metadata.as_slice())).build());

        });
}