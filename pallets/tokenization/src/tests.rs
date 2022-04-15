use frame_support::{assert_noop, assert_ok};
use sp_core::blake2_256;
use sp_runtime::traits::BadOrigin;
use sp_runtime::AccountId32;
use sp_std::vec::Vec;

use mock::{Event, *};
use primitives::Balance;

// Unit testing for metaverse currency, metaverse treasury
#[cfg(test)]
use super::*;

fn metaverse_fund_account() -> AccountId {
	TokenizationModule::get_metaverse_fund_id(METAVERSE_ID)
}

fn get_metaverse_fund_balance() -> Balance {
	match TokenizationModule::get_total_issuance(METAVERSE_ID) {
		Ok(balance) => balance,
		_ => 0,
	}
}

#[test]
fn mint_social_token_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::root();
		assert_eq!(get_metaverse_fund_balance(), 0);

		assert_ok!(TokenizationModule::mint_token(
			origin,
			vec![1],
			METAVERSE_ID,
			400,
			(3, 10),
			10,
			ALICE
		));

		assert_eq!(get_metaverse_fund_balance(), 400);

		let event = mock::Event::TokenizationModule(crate::Event::FungibleTokenIssued(
			FungibleTokenId::FungibleToken(1),
			ALICE,
			metaverse_fund_account(),
			400,
			METAVERSE_ID,
		));

		assert_eq!(last_event(), event);
	});
}

#[test]
fn mint_social_token_should_fail_for_non_owner() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::root();

		assert_noop!(
			TokenizationModule::mint_token(origin, vec![1], METAVERSE_ID, 0, (3, 10), 10, BOB),
			Error::<Runtime>::NoPermissionTokenIssuance
		);
	});
}

#[test]
fn mint_social_token_should_fail_if_already_exists() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::root();
		assert_ok!(TokenizationModule::mint_token(
			origin.clone(),
			vec![1],
			METAVERSE_ID,
			100,
			(3, 10),
			10,
			ALICE
		));

		assert_noop!(
			TokenizationModule::mint_token(origin, vec![1], METAVERSE_ID, 100, (3, 10), 10, ALICE),
			Error::<Runtime>::FungibleTokenAlreadyIssued
		);
	});
}

#[test]
fn metaverse_treasury_pool_withdraw_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::root();

		assert_eq!(get_metaverse_fund_balance(), 0);
		assert_ok!(TokenizationModule::mint_token(
			origin,
			vec![1],
			METAVERSE_ID,
			400,
			(3, 10),
			10,
			ALICE
		));
		assert_ok!(Currencies::deposit(
			METAVERSE_FUND,
			&TokenizationModule::get_metaverse_fund_id(METAVERSE_ID),
			400
		));
		assert_eq!(get_metaverse_fund_balance(), 800);
		assert_ok!(Currencies::withdraw(
			METAVERSE_FUND,
			&TokenizationModule::get_metaverse_fund_id(METAVERSE_ID),
			200
		));
		assert_eq!(get_metaverse_fund_balance(), 600);
	});
}

#[test]
fn metaverse_treasury_pool_withdraw_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::root();
		assert_eq!(get_metaverse_fund_balance(), 0);
		assert_ok!(TokenizationModule::mint_token(
			origin,
			vec![1],
			METAVERSE_ID,
			400,
			(3, 10),
			10,
			ALICE
		));
		assert_eq!(get_metaverse_fund_balance(), 400);
		assert_noop!(
			Currencies::withdraw(METAVERSE_FUND, &ALICE, 800,),
			orml_tokens::Error::<Runtime>::BalanceTooLow
		);
	});
}

#[test]
fn metaverse_treasury_pool_transfer_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::root();
		assert_eq!(get_metaverse_fund_balance(), 0);
		assert_ok!(TokenizationModule::mint_token(
			origin,
			vec![1],
			METAVERSE_ID,
			400,
			(3, 10),
			10,
			ALICE
		));
		assert_eq!(get_metaverse_fund_balance(), 400);
		assert_ok!(Currencies::deposit(
			METAVERSE_FUND,
			&TokenizationModule::get_metaverse_fund_id(METAVERSE_ID),
			400
		));
		assert_ok!(Currencies::transfer(
			Origin::signed(TokenizationModule::get_metaverse_fund_id(METAVERSE_ID)),
			ALICE,
			METAVERSE_FUND,
			100
		));
		assert_eq!(Currencies::free_balance(METAVERSE_FUND, &ALICE), 380); // 120 has been vested
	});
}
