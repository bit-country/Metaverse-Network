// Unit testing for country currency, country treasury
use super::*;
use mock::*;
use frame_support::{assert_noop, assert_ok, dispatch};
use sp_runtime::traits::BadOrigin;
use sp_core::blake2_256;
#[test]
fn country_treasury_pool_should_work() {
    ExtBuilder::default().build().execute_with(|| {        
        assert_eq!(TokenizationModule::total_issuance(COUNTRY_FUND), 0);
		assert_ok!(
			Currencies::deposit(
                COUNTRY_FUND,
                &TokenizationModule::get_country_fund_id(COUNTRY_ID),
                400
            )
        );
        assert_eq!(TokenizationModule::total_issuance(COUNTRY_FUND), 400);
	});
}
#[test]
fn country_treasury_pool_withdraw_should_work() {
    ExtBuilder::default().build().execute_with(|| {        
        assert_eq!(TokenizationModule::total_issuance(COUNTRY_FUND), 0);
		assert_ok!(
			Currencies::deposit(
                COUNTRY_FUND,
                &TokenizationModule::get_country_fund_id(COUNTRY_ID),
                400
            )
        );
        assert_eq!(TokenizationModule::total_issuance(COUNTRY_FUND), 400);
        assert_ok!(
            Currencies::withdraw(
                COUNTRY_FUND,
                &TokenizationModule::get_country_fund_id(COUNTRY_ID),
                200
            )
        );
        assert_eq!(TokenizationModule::total_issuance(COUNTRY_FUND), 200);
	});
}
#[test]
fn country_treasury_pool_withdraw_should_fail() {
    ExtBuilder::default().build().execute_with(|| {        
        assert_eq!(TokenizationModule::total_issuance(COUNTRY_FUND), 0);
		assert_ok!(
			Currencies::deposit(
                COUNTRY_FUND,
                &TokenizationModule::get_country_fund_id(COUNTRY_ID),
                400
            )
        );
        assert_eq!(TokenizationModule::total_issuance(COUNTRY_FUND), 400);
        assert_noop!(
            Currencies::withdraw(
                COUNTRY_FUND,
                &ALICE,
                200
            ),
            orml_tokens::Error::<Runtime>::BalanceTooLow
        );
	});
}
#[test]
fn country_treasury_pool_transfer_should_work() {
    ExtBuilder::default().build().execute_with(|| {        
        assert_eq!(TokenizationModule::total_issuance(COUNTRY_FUND), 0);
		assert_ok!(
			Currencies::deposit(
                COUNTRY_FUND,
                &TokenizationModule::get_country_fund_id(COUNTRY_ID),
                400
            )
        );
        assert_eq!(TokenizationModule::total_issuance(COUNTRY_FUND), 400);
        assert_ok!(
			Currencies::transfer(
                Origin::signed(TokenizationModule::get_country_fund_id(COUNTRY_ID)),
                ALICE,
                COUNTRY_FUND,
                100
            )
        );
        assert_eq!(Currencies::free_balance(COUNTRY_FUND, &ALICE), 100);
	});
}
