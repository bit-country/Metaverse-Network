// Unit testing for metaverse currency, metaverse treasury
#[cfg(test)]
use super::*;
use mock::{Event, *};
use primitives::{Balance};
use sp_std::vec::Vec;
use frame_support::{assert_noop, assert_ok};
use sp_runtime::AccountId32;
use sp_runtime::traits::BadOrigin;
use sp_core::blake2_256;

fn country_fund_account() -> AccountId {
    TokenizationModule::get_metaverse_fund_id(METAVERSE_ID)
}

fn get_country_fund_balance() -> Balance {
    match TokenizationModule::get_total_issuance(METAVERSE_ID) {
        Ok(balance) => balance,
        _ => 0
    }
}

#[test]
fn mint_social_token_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);        
        assert_eq!(get_country_fund_balance(), 0);

        assert_ok!(
			TokenizationModule::mint_token(
                origin,
                vec![1],
                METAVERSE_ID,
                400
            )
        );      

        assert_eq!(get_country_fund_balance(), 400);
        
        let event = mock::Event::tokenization(
            crate::Event::FungibleTokenIssued(1, ALICE, country_fund_account(),  400)
        );

        assert_eq!(last_event(), event);
    });
}

#[test]
fn mint_social_token_should_fail_for_non_owner() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(BOB);

        assert_noop!(
			TokenizationModule::mint_token(
                origin,
                vec![1],
                METAVERSE_ID,
                0
            ),
            Error::<Runtime>::NoPermissionTokenIssuance
        );        
    });
}

#[test]
fn mint_social_token_should_fail_if_already_exists() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);
        assert_ok!(
			TokenizationModule::mint_token(
                origin.clone(),
                vec![1],
                METAVERSE_ID,
                0
            )
        );        

        assert_noop!(
			TokenizationModule::mint_token(
                origin,
                vec![1],
                METAVERSE_ID,
                0
            ),
            Error::<Runtime>::FungibleTokenAlreadyIssued
        );        
    });
}

#[test]
fn country_treasury_pool_withdraw_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);

        assert_eq!(get_country_fund_balance(), 0);
        assert_ok!(
			TokenizationModule::mint_token(
                origin,
                vec![1],
                METAVERSE_ID,
                400
            )
        );
        assert_ok!(
			Currencies::deposit(
                COUNTRY_FUND,
                &TokenizationModule::get_metaverse_fund_id(METAVERSE_ID),
                400
            )
        );
        assert_eq!(get_country_fund_balance(), 800);
        assert_ok!(
            Currencies::withdraw(
                COUNTRY_FUND,
                &TokenizationModule::get_metaverse_fund_id(METAVERSE_ID),
                200
            )
        );
        assert_eq!(get_country_fund_balance(), 600);
    });
}

#[test]
fn country_treasury_pool_withdraw_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);
        assert_eq!(get_country_fund_balance(), 0);
        assert_ok!(
			TokenizationModule::mint_token(
                origin,
                vec![1],
                METAVERSE_ID,
                400
            )
        );
        assert_eq!(get_country_fund_balance(), 400);
        assert_noop!(
            Currencies::withdraw(
                COUNTRY_FUND,
                &ALICE,
                800
            ),
            orml_tokens::Error::<Runtime>::BalanceTooLow
        );
    });
}

#[test]
fn country_treasury_pool_transfer_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);
        assert_eq!(get_country_fund_balance(), 0);
        assert_ok!(
			TokenizationModule::mint_token(
                origin,
                vec![1],
                METAVERSE_ID,
                400
            )
        );
        assert_eq!(get_country_fund_balance(), 400);
        assert_ok!(
			Currencies::deposit(
                COUNTRY_FUND,
                &TokenizationModule::get_metaverse_fund_id(METAVERSE_ID),
                400
            )
        );
        assert_ok!(
			Currencies::transfer(
                Origin::signed(TokenizationModule::get_metaverse_fund_id(METAVERSE_ID)),
                ALICE,
                COUNTRY_FUND,
                100
            )
        );
        assert_eq!(Currencies::free_balance(COUNTRY_FUND, &ALICE), 500);
    });
}
