use frame_support::assert_ok;
use orml_traits::MultiCurrency;
use xcm::latest::{Junction, Junction::*, Junctions::*, MultiLocation, NetworkId};
use xcm_emulator::TestExt;

use core_traits::{Balance, FungibleTokenId};
use pioneer_runtime::{Balances, KUsdPerSecond, KarPerSecond, KsmPerSecond, NeerPerSecond, Origin, Tokens, XTokens};

use crate::kusama_test::{Development, Karura, KusamaNet, Sibling, TestNet};
use crate::setup::{
	bit_amount, development_account, kar_amount, karura_account, ksm_amount, kusd_amount, native_amount,
	sibling_account, ALICE, BOB, PARA_ID_DEVELOPMENT, PARA_ID_SIBLING,
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

#[test]
fn transfer_bit_to_sibling() {
	TestNet::reset();

	let alice_initial_balance = bit_amount(10000);
	let bob_initial_balance = bit_amount(10000);
	let transfer_amount = bit_amount(100);

	Development::execute_with(|| {
		assert_ok!(Tokens::deposit(
			FungibleTokenId::MiningResource(0),
			&ALICE.into(),
			alice_initial_balance
		));
		assert_eq!(
			Tokens::free_balance(FungibleTokenId::MiningResource(0), &ALICE.into()),
			alice_initial_balance,
		);
		assert_eq!(
			Tokens::free_balance(FungibleTokenId::MiningResource(0), &sibling_account()),
			0,
		);
	});

	Sibling::execute_with(|| {
		assert_ok!(Tokens::deposit(
			FungibleTokenId::MiningResource(0),
			&BOB.into(),
			bob_initial_balance
		));
		assert_eq!(
			Tokens::free_balance(FungibleTokenId::MiningResource(0), &BOB.into()),
			bob_initial_balance,
		);
	});

	Development::execute_with(|| {
		assert_ok!(XTokens::transfer(
			Origin::signed(ALICE.into()),
			FungibleTokenId::MiningResource(0),
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
			Tokens::free_balance(FungibleTokenId::MiningResource(0), &ALICE.into()),
			alice_initial_balance - transfer_amount
		);

		// Verify that the amount transferred is now part of the sibling account here
		assert_eq!(
			Tokens::free_balance(FungibleTokenId::MiningResource(0), &sibling_account()),
			transfer_amount
		);
	});

	Sibling::execute_with(|| {
		// Verify that BOB now has initial balance + amount transferred - fee
		assert_eq!(
			Tokens::free_balance(FungibleTokenId::MiningResource(0), &BOB.into()),
			bob_initial_balance + transfer_amount - native_fee(),
		);
	});
}

#[test]
fn transfer_kusd_to_development() {
	TestNet::reset();

	let alice_initial_balance = kusd_amount(1000);
	let bob_initial_balance = kusd_amount(1000);
	let transfer_amount = kusd_amount(200);

	Karura::execute_with(|| {
		assert_ok!(Tokens::deposit(
			FungibleTokenId::Stable(0),
			&ALICE.into(),
			alice_initial_balance
		));

		assert_eq!(
			Tokens::free_balance(FungibleTokenId::Stable(0), &development_account()),
			0
		);
	});

	Development::execute_with(|| {
		assert_ok!(Tokens::deposit(
			FungibleTokenId::Stable(0),
			&BOB.into(),
			bob_initial_balance
		));
		assert_eq!(
			Tokens::free_balance(FungibleTokenId::Stable(0), &BOB.into()),
			bob_initial_balance,
		);

		assert_ok!(Tokens::deposit(
			FungibleTokenId::Stable(0),
			&karura_account().into(),
			bob_initial_balance
		));
	});

	Karura::execute_with(|| {
		assert_ok!(XTokens::transfer(
			Origin::signed(ALICE.into()),
			FungibleTokenId::Stable(0),
			transfer_amount,
			Box::new(
				MultiLocation::new(
					1,
					X2(
						Parachain(PARA_ID_DEVELOPMENT),
						Junction::AccountId32 {
							network: NetworkId::Any,
							id: BOB.into(),
						}
					)
				)
				.into()
			),
			8_000_000_000,
		));

		assert_eq!(
			Tokens::free_balance(FungibleTokenId::Stable(0), &ALICE.into()),
			alice_initial_balance - transfer_amount
		);

		// Verify that the amount transferred is now part of the development account here
		assert_eq!(
			Tokens::free_balance(FungibleTokenId::Stable(0), &development_account()),
			transfer_amount
		);
	});

	Development::execute_with(|| {
		// Verify that BOB now has initial balance + amount transferred - fee
		assert_eq!(
			Tokens::free_balance(FungibleTokenId::Stable(0), &BOB.into()),
			bob_initial_balance + transfer_amount - kusd_fee()
		);
	});
}

#[test]
fn transfer_kar_to_development() {
	TestNet::reset();

	let alice_initial_balance = kar_amount(1000);
	let bob_initial_balance = kar_amount(1000);
	let transfer_amount = kar_amount(200);

	Karura::execute_with(|| {
		assert_ok!(Tokens::deposit(
			FungibleTokenId::NativeToken(2),
			&ALICE.into(),
			alice_initial_balance
		));

		assert_eq!(
			Tokens::free_balance(FungibleTokenId::NativeToken(2), &development_account()),
			0
		);
	});

	Development::execute_with(|| {
		assert_ok!(Tokens::deposit(
			FungibleTokenId::NativeToken(2),
			&BOB.into(),
			bob_initial_balance
		));
		assert_eq!(
			Tokens::free_balance(FungibleTokenId::NativeToken(2), &BOB.into()),
			bob_initial_balance,
		);

		assert_ok!(Tokens::deposit(
			FungibleTokenId::NativeToken(2),
			&karura_account().into(),
			bob_initial_balance
		));
	});

	Karura::execute_with(|| {
		assert_ok!(XTokens::transfer(
			Origin::signed(ALICE.into()),
			FungibleTokenId::NativeToken(2),
			transfer_amount,
			Box::new(
				MultiLocation::new(
					1,
					X2(
						Parachain(PARA_ID_DEVELOPMENT),
						Junction::AccountId32 {
							network: NetworkId::Any,
							id: BOB.into(),
						}
					)
				)
				.into()
			),
			8_000_000_000,
		));

		assert_eq!(
			Tokens::free_balance(FungibleTokenId::NativeToken(2), &ALICE.into()),
			alice_initial_balance - transfer_amount
		);

		// Verify that the amount transferred is now part of the development account here
		assert_eq!(
			Tokens::free_balance(FungibleTokenId::NativeToken(2), &development_account()),
			transfer_amount
		);
	});

	Development::execute_with(|| {
		// Verify that BOB now has initial balance + amount transferred - fee
		assert_eq!(
			Tokens::free_balance(FungibleTokenId::NativeToken(2), &BOB.into()),
			bob_initial_balance + transfer_amount - kar_fee()
		);
	});
}

#[test]
fn currency_id_convert_test() {
	use pioneer_runtime::FungibleTokenIdConvert;
	use sp_runtime::codec::Encode;
	use sp_runtime::traits::Convert as CurrencyConvert;

	let pioneer_native_location: MultiLocation = MultiLocation::new(
		1,
		X2(Parachain(2096), GeneralKey(FungibleTokenId::NativeToken(0).encode())),
	);

	let pioneer_mining_resource_location: MultiLocation = MultiLocation::new(
		1,
		X2(Parachain(2096), GeneralKey(FungibleTokenId::MiningResource(0).encode())),
	);

	assert_eq!(
		FungibleTokenId::NativeToken(0).encode(),
		vec![0, 0, 0, 0, 0, 0, 0, 0, 0]
	);

	assert_eq!(
		FungibleTokenId::MiningResource(0).encode(),
		vec![3, 0, 0, 0, 0, 0, 0, 0, 0]
	);

	assert_eq!(
		FungibleTokenIdConvert::convert(pioneer_native_location.clone()),
		Some(FungibleTokenId::NativeToken(0)),
	);

	assert_eq!(
		FungibleTokenIdConvert::convert(pioneer_mining_resource_location.clone()),
		Some(FungibleTokenId::MiningResource(0)),
	);

	Development::execute_with(|| {
		assert_eq!(
			FungibleTokenIdConvert::convert(FungibleTokenId::NativeToken(0)),
			Some(pioneer_native_location)
		)
	});

	Development::execute_with(|| {
		assert_eq!(
			FungibleTokenIdConvert::convert(FungibleTokenId::MiningResource(0)),
			Some(pioneer_mining_resource_location)
		)
	});
}

// The fee associated with transferring Native tokens
fn native_fee() -> Balance {
	let (_asset, fee) = NeerPerSecond::get();
	// This fee is varies between how fast local test executed.
	fee.div_euclid(10_000) * 4
}

// The fee associated with transferring KSM tokens
fn kusd_fee() -> Balance {
	let (_asset, fee) = KUsdPerSecond::get();
	// NOTE: it is possible that in different machines this value may differ. We shall see.
	fee.div_euclid(10_000) * 4
}

// The fee associated with transferring KSM tokens
fn kar_fee() -> Balance {
	let (_asset, fee) = KarPerSecond::get();
	// NOTE: it is possible that in different machines this value may differ. We shall see.
	fee.div_euclid(10_000) * 4
}
