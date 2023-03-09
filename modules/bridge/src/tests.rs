#![cfg(test)]

use std::str::{from_utf8, FromStr};

use frame_support::{assert_noop, assert_ok};
use sp_core::H160;

use mock::{Balances, BridgeModule, Event, ExtBuilder, One, Origin, Runtime, System, Tokens};
use primitives::evm::{CurrencyIdType, EvmAddress, H160_POSITION_CURRENCY_ID_TYPE, H160_POSITION_TOKEN};
use primitives::FungibleTokenId::FungibleToken;
use primitives::{TokenId, TokenSymbol};

use crate::mock::{ALICE, BOB};

use super::*;

#[test]
fn bridge_out_nft_works() {
	ExtBuilder::default().build().execute_with(|| {
		let resource_id = H160::from_str("0x0000000000000000000200000000000000000000")
			.ok()
			.unwrap();
		assert_ok!(BridgeModule::sudo_change_fee(Origin::root(), 100, 1, 0));
		assert_ok!(BridgeModule::register_new_nft_resource_id(
			Origin::root(),
			resource_id,
			0
		));
		assert_eq!(Balances::free_balance(ALICE), 100000);
		assert_ok!(BridgeModule::bridge_out_nft(Origin::signed(ALICE), vec![1], (0, 1), 0));
	})
}

#[test]
fn bridge_in_nft_works() {
	ExtBuilder::default().build().execute_with(|| {
		let resource_id = H160::from_str("0x0000000000000000000200000000000000000000")
			.ok()
			.unwrap();
		assert_ok!(BridgeModule::sudo_change_fee(Origin::root(), 100, 1, 0));
		assert_ok!(BridgeModule::register_new_nft_resource_id(
			Origin::root(),
			resource_id,
			0
		));
		assert_ok!(BridgeModule::add_bridge_origin(Origin::root(), ALICE));

		assert_eq!(Balances::free_balance(ALICE), 100000);
		assert_ok!(BridgeModule::bridge_in_nft(
			Origin::signed(ALICE),
			vec![1],
			BOB,
			5,
			resource_id,
			vec![1]
		));
	})
}
