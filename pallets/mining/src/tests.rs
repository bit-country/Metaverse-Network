use frame_support::{assert_noop, assert_ok};

use sp_runtime::Perbill;

use mock::*;
use primitives::Balance;

// Unit testing for metaverse currency, metaverse treasury
#[cfg(test)]
use super::*;

fn get_mining_balance() -> Balance {
	Currencies::total_issuance(MiningCurrencyId::get())
}

fn get_mining_balance_of(who: &AccountId) -> Balance {
	Currencies::free_balance(MiningCurrencyId::get(), who)
}

fn setup_minting_resource() -> DispatchResult {
	// Add ALICE as minting resource
	MiningModule::add_minting_origin(RuntimeOrigin::signed(ALICE), ALICE);
	Ok(())
}

#[test]
fn mint_mining_resource_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(setup_minting_resource());

		let origin = RuntimeOrigin::signed(ALICE);
		assert_eq!(get_mining_balance(), 0);

		assert_ok!(MiningModule::mint(origin, BOB, 1000));

		assert_eq!(get_mining_balance(), 1000);

		let event = mock::RuntimeEvent::MiningModule(crate::Event::MiningResourceMintedTo(BOB, 1000));

		assert_eq!(last_event(), event);
	});
}

#[test]
fn burn_mining_resource_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(setup_minting_resource());

		let origin = RuntimeOrigin::signed(ALICE);
		assert_eq!(get_mining_balance(), 0);

		assert_ok!(MiningModule::mint(origin.clone(), BOB, 1000));

		assert_eq!(get_mining_balance(), 1000);

		let _event = mock::RuntimeEvent::MiningModule(crate::Event::MiningResourceMintedTo(BOB, 1000));

		assert_ok!(MiningModule::burn(origin, BOB, 300));
		assert_eq!(get_mining_balance(), 700);
		assert_eq!(
			last_event(),
			mock::RuntimeEvent::MiningModule(crate::Event::MiningResourceBurnFrom(BOB, 300))
		);
	});
}

#[test]
fn withdraw_mining_resource_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(setup_minting_resource());

		let origin = RuntimeOrigin::signed(ALICE);
		assert_eq!(get_mining_balance(), 0);
		let treasury_id = MiningModule::bit_treasury_account_id();
		assert_ok!(MiningModule::mint(origin.clone(), treasury_id, 1000));
		assert_eq!(
			last_event(),
			mock::RuntimeEvent::MiningModule(crate::Event::MiningResourceMintedTo(treasury_id, 1000))
		);
		assert_eq!(get_mining_balance(), 1000);
		assert_eq!(get_mining_balance_of(&DAVE), 0);
		assert_ok!(MiningModule::withdraw(origin, DAVE, 300));
		assert_eq!(get_mining_balance_of(&DAVE), 300);

		let event = mock::RuntimeEvent::MiningModule(crate::Event::WithdrawMiningResource(DAVE, 300));

		assert_eq!(last_event(), event);
	});
}

#[test]
fn mint_mining_resource_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			MiningModule::mint(RuntimeOrigin::signed(ALICE), BOB, 1000),
			crate::Error::<Runtime>::NoPermission
		);
	})
}

#[test]
fn burn_mining_resource_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			MiningModule::burn(RuntimeOrigin::signed(ALICE), BOB, 1000),
			crate::Error::<Runtime>::NoPermission
		);
	})
}

#[test]
fn deposit_mining_resource_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			MiningModule::deposit(RuntimeOrigin::signed(ALICE), 1000),
			crate::Error::<Runtime>::BalanceLow
		);
	})
}

#[test]
fn withdraw_mining_resource_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			MiningModule::withdraw(RuntimeOrigin::signed(ALICE), BOB, 1000),
			crate::Error::<Runtime>::NoPermission
		);
	})
}

#[test]
fn deposit_mining_resource_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(setup_minting_resource());

		let origin = RuntimeOrigin::signed(ALICE);
		assert_eq!(get_mining_balance(), 0);
		let treasury_id = MiningModule::bit_treasury_account_id();
		assert_ok!(MiningModule::mint(origin.clone(), treasury_id, 1000));

		assert_eq!(get_mining_balance(), 1000);

		//Transfer to BOB 300
		assert_ok!(Currencies::transfer(
			RuntimeOrigin::signed(treasury_id),
			BOB,
			MiningCurrencyId::get(),
			300
		));
		//BOB balance now is 300
		assert_eq!(get_mining_balance_of(&BOB), 300);
		//BOB deposit to treasury so his hot wallet will be 100
		assert_ok!(MiningModule::deposit(RuntimeOrigin::signed(BOB), 100));
		assert_eq!(get_mining_balance_of(&BOB), 200);

		assert_eq!(get_mining_balance_of(&treasury_id), 800);

		let event = mock::RuntimeEvent::MiningModule(crate::Event::DepositMiningResource(BOB, 100));

		assert_eq!(last_event(), event);
	});
}

#[test]
fn update_mining_config_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		setup_minting_resource();
		let origin = RuntimeOrigin::signed(ALICE);
		let mining_config = MiningResourceRateInfo {
			rate: Perbill::from_percent(10),
			staking_reward: Perbill::from_percent(30),
			mining_reward: Perbill::from_percent(70),
		};
		assert_ok!(MiningModule::update_mining_issuance_config(
			origin,
			mining_config.clone()
		));

		assert_eq!(MiningModule::mining_ratio_config(), mining_config.clone());

		let event = mock::RuntimeEvent::MiningModule(crate::Event::MiningConfigUpdated(1, mining_config));

		assert_eq!(last_event(), event);
	});
}

#[test]
fn adding_and_removing_mining_origin_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MiningModule::add_minting_origin(RuntimeOrigin::signed(ALICE), BOB));
		assert_eq!(Balances::free_balance(BOB), 999);
		assert_ok!(MiningModule::remove_minting_origin(RuntimeOrigin::signed(ALICE), BOB));
		assert_eq!(Balances::free_balance(BOB), 1000);
	});
}
