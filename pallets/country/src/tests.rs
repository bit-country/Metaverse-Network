//! Unit tests for the country module.
use super::*;
use mock::*;

use frame_support::{assert_noop, assert_ok, dispatch};
use sp_runtime::traits::BadOrigin;
use sp_core::blake2_256;

#[test]
fn create_country_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(CountryModule::create_country(Origin::signed(ALICE), vec![1]));
		assert_eq!(
			CountryModule::get_country(&COUNTRY_ID),
			Some(Country{
				owner: ALICE,
				metadata: vec![1],
				currency_id: Default::default()
			})
		);
		let event = TestEvent::country(RawEvent::NewCountryCreated(COUNTRY_ID));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn create_country_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			CountryModule::create_country(
				Origin::none(),
				vec![1],
			),
			dispatch::DispatchError::BadOrigin
		);
	});
}

#[test]
fn transfer_country_should_work() {
	ExtBuilder::default().build().execute_with( || {
		assert_ok!(CountryModule::create_country(Origin::signed(ALICE), vec![1]));
		assert_ok!(CountryModule::transfer_country(Origin::signed(ALICE), BOB, COUNTRY_ID));
		let event = TestEvent::country(RawEvent::TransferredCountry(COUNTRY_ID, ALICE, BOB));
		assert_eq!(last_event(), event);
		//Make sure 2 ways transfer works
		assert_ok!(CountryModule::transfer_country(Origin::signed(BOB), ALICE, COUNTRY_ID));
		let event = TestEvent::country(RawEvent::TransferredCountry(COUNTRY_ID, BOB, ALICE));
		assert_eq!(last_event(), event);
	})
}

#[test]
fn transfer_country_should_fail() {
	ExtBuilder::default().build().execute_with( || {
		assert_ok!(CountryModule::create_country(Origin::signed(ALICE), vec![1]));
		assert_noop!(CountryModule::transfer_country(Origin::signed(BOB), ALICE, COUNTRY_ID), Error::<Runtime>::NoPermission);
	})
}

#[test]
fn freeze_country_should_work() {
	ExtBuilder::default().build().execute_with( || {
		assert_ok!(CountryModule::create_country(Origin::signed(ALICE), vec![1]));
		assert_ok!(CountryModule::freeze_country(Origin::root(), COUNTRY_ID));
		let event = TestEvent::country(RawEvent::CountryFreezed(COUNTRY_ID));
		assert_eq!(last_event(), event);
	})
}

#[test]
fn freeze_country_should_fail() {
	ExtBuilder::default().build().execute_with( || {
		assert_ok!(CountryModule::create_country(Origin::signed(ALICE), vec![1]));
		//Country owner tries to freeze their own country
		assert_noop!(CountryModule::freeze_country(Origin::signed(ALICE), COUNTRY_ID), dispatch::DispatchError::BadOrigin);
	})
}

#[test]
fn unfreeze_country_should_work() {
	ExtBuilder::default().build().execute_with( || {
		assert_ok!(CountryModule::create_country(Origin::signed(ALICE), vec![1]));
		assert_ok!(CountryModule::freeze_country(Origin::root(), COUNTRY_ID));
		let event = TestEvent::country(RawEvent::CountryFreezed(COUNTRY_ID));
		assert_eq!(last_event(), event);
		assert_ok!(CountryModule::unfreeze_country(Origin::root(), COUNTRY_ID));
		let event = TestEvent::country(RawEvent::CountryUnFreezed(COUNTRY_ID));
		assert_eq!(last_event(), event);
	})
}

#[test]
fn destroy_country_should_work() {
	ExtBuilder::default().build().execute_with( || {
		assert_ok!(CountryModule::create_country(Origin::signed(ALICE), vec![1]));
		assert_ok!(CountryModule::destroy_country(Origin::root(), COUNTRY_ID));
		let event = TestEvent::country(RawEvent::CountryDestroyed(COUNTRY_ID));
		assert_eq!(last_event(), event);
	})
}

#[test]
fn destroy_country_without_root_should_fail() {
	ExtBuilder::default().build().execute_with( || {
		assert_ok!(CountryModule::create_country(Origin::signed(ALICE), vec![1]));
		assert_noop!(CountryModule::destroy_country(Origin::signed(ALICE), COUNTRY_ID), dispatch::DispatchError::BadOrigin);
	})
}

#[test]
fn destroy_country_with_no_id_should_fail() {
	ExtBuilder::default().build().execute_with( || {
		assert_ok!(CountryModule::create_country(Origin::signed(ALICE), vec![1]));
		assert_noop!(CountryModule::destroy_country(Origin::root(), COUNTRY_ID_NOT_EXIST), Error::<Runtime>::CountryInfoNotFound);
	})
}