use frame_support::assert_ok;
use orml_traits::MultiCurrency;
use xcm::latest::{Junction, Junction::*, Junctions::*, MultiLocation, NetworkId};
use xcm_emulator::TestExt;

use core_traits::{Balance, FungibleTokenId};
use pioneer_runtime::{Balances, KUsdPerSecond, KsmPerSecond, NeerPerSecond, Origin, Tokens, XTokens};

use crate::kusama_test::{Development, Karura, KusamaNet, Sibling, TestNet};
use crate::setup::{
	development_account, karura_account, ksm_amount, kusd_amount, native_amount, sibling_account, ALICE, BOB,
	PARA_ID_DEVELOPMENT, PARA_ID_SIBLING,
};

#[test]
fn transfer_native_to_sibling() {
	TestNet::reset();

	let alice_initial_balance = native_amount(10000);
	let bob_initial_balance = native_amount(10000);
	let transfer_amount = native_amount(1);

	Development::execute_with(|| {
		assert_eq!(Balances::free_balance(&ALICE.into()), alice_initial_balance);
		assert_eq!(Balances::free_balance(&sibling_account()), 0);
	});

	Sibling::execute_with(|| {
		assert_eq!(Balances::free_balance(&BOB.into()), bob_initial_balance);
	});

	Development::execute_with(|| {
		assert_ok!(XTokens::transfer(
			Origin::signed(ALICE.into()),
			FungibleTokenId::NativeToken(0),
			transfer_amount,
			Box::new(
				MultiLocation::new(
					1,
					X2(
						Parachain(PARA_ID_SIBLING),
						Junction::AccountId32 {
							network: NetworkId::Any,
							id: BOB.into(),
						}
					)
				)
				.into()
			),
			8_000_000_000_000,
		));

		// Confirm that Alice's balance is initial balance - amount transferred
		assert_eq!(
			Balances::free_balance(&ALICE.into()),
			alice_initial_balance - transfer_amount
		);

		// Verify that the amount transferred is now part of the sibling account here
		assert_eq!(Balances::free_balance(&sibling_account()), transfer_amount);
	});

	Sibling::execute_with(|| {
		// Verify that BOB now has initial balance + amount transferred - fee
		assert_eq!(
			Balances::free_balance(&BOB.into()),
			bob_initial_balance + transfer_amount - native_fee(),
		);
	});
}

// The fee associated with transferring Native tokens
fn native_fee() -> Balance {
	let (_asset, fee) = NeerPerSecond::get();
	// NOTE: it is possible that in different machines this value may differ. We shall see.
	fee.div_euclid(10_000) * 8
}
