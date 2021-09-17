// Unit testing for bitcountry currency, bitcountry treasury
#[cfg(test)]
use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};
use primitives::Balance;
use sp_core::blake2_256;
use sp_runtime::traits::BadOrigin;
use sp_runtime::AccountId32;
use sp_std::vec::Vec;

fn get_mining_balance() -> Balance {
	Currencies::total_issuance(MiningCurrencyId::get())
}

fn get_mining_balance_of(who: &AccountId) -> Balance {
	Currencies::free_balance(MiningCurrencyId::get(), who)
}

fn setup_minting_resource() -> DispatchResult {
	//Add ALICE as minting resource
	MiningModule::add_minting_origin(Origin::signed(ALICE), ALICE);
	Ok(())
}

#[test]
fn mint_mining_resource_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(setup_minting_resource());

		let origin = Origin::signed(ALICE);
		assert_eq!(get_mining_balance(), 0);

		assert_ok!(MiningModule::mint(origin, 1000));

		assert_eq!(get_mining_balance(), 1000);

		let event = mock::Event::mining(crate::Event::MiningResourceMinted(1000));

		assert_eq!(last_event(), event);
	});
}

#[test]
fn burn_mining_resource_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(setup_minting_resource());

		let origin = Origin::signed(ALICE);
		assert_eq!(get_mining_balance(), 0);

		assert_ok!(MiningModule::mint(origin.clone(), 1000));

		assert_eq!(get_mining_balance(), 1000);

		let event = mock::Event::mining(crate::Event::MiningResourceMinted(1000));

		assert_ok!(MiningModule::burn(origin, 300));
		assert_eq!(get_mining_balance(), 700);
		assert_eq!(
			last_event(),
			mock::Event::mining(crate::Event::MiningResourceBurned(300))
		);
	});
}

#[test]
fn withdraw_mining_resource_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(setup_minting_resource());

		let origin = Origin::signed(ALICE);
		assert_eq!(get_mining_balance(), 0);

		assert_ok!(MiningModule::mint(origin.clone(), 1000));

		assert_eq!(get_mining_balance(), 1000);
		assert_eq!(get_mining_balance_of(&BOB), 0);
		assert_ok!(MiningModule::withdraw(origin, BOB, 300));
		assert_eq!(get_mining_balance_of(&BOB), 300);

		let event = mock::Event::mining(crate::Event::WithdrawMiningResource(BOB, 300));

		assert_eq!(last_event(), event);
	});
}

#[test]
fn mint_mining_resource_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			MiningModule::mint(Origin::signed(ALICE), 1000),
			crate::Error::<Runtime>::NoPermission
		);
	})
}

#[test]
fn burn_mining_resource_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			MiningModule::burn(Origin::signed(ALICE), 1000),
			crate::Error::<Runtime>::NoPermission
		);
	})
}

#[test]
fn deposit_mining_resource_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			MiningModule::deposit(Origin::signed(ALICE), BOB, 1000),
			crate::Error::<Runtime>::NoPermission
		);
	})
}

#[test]
fn withdraw_mining_resource_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			MiningModule::withdraw(Origin::signed(ALICE), BOB, 1000),
			crate::Error::<Runtime>::NoPermission
		);
	})
}

#[test]
fn deposit_mining_resource_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(setup_minting_resource());

		let origin = Origin::signed(ALICE);
		assert_eq!(get_mining_balance(), 0);

		assert_ok!(MiningModule::mint(origin.clone(), 1000));

		assert_eq!(get_mining_balance(), 1000);
		let treasury_id = MiningModule::bit_mining_resource_account_id();
		//Transfer to BOB 300
		assert_ok!(Currencies::transfer(
			Origin::signed(treasury_id),
			BOB,
			MiningCurrencyId::get(),
			300
		));
		//BOB balance now is 300
		assert_eq!(get_mining_balance_of(&BOB), 300);
		//BOB deposit to treasury so his hot wallet will be 100 - only ALICE can call this function
		assert_ok!(MiningModule::deposit(origin, BOB, 100));
		assert_eq!(get_mining_balance_of(&BOB), 200);

		assert_eq!(get_mining_balance_of(&treasury_id), 800);

		let event = mock::Event::mining(crate::Event::DepositMiningResource(BOB, 100));

		assert_eq!(last_event(), event);
	});
}
