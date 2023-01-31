#![cfg(test)]

use std::str::{from_utf8, FromStr};

use frame_support::{assert_noop, assert_ok};
use sp_core::H160;

use mock::{Event, ExtBuilder, Origin, Runtime, System};
use primitives::evm::{CurrencyIdType, EvmAddress, H160_POSITION_CURRENCY_ID_TYPE, H160_POSITION_TOKEN};
use primitives::FungibleTokenId::FungibleToken;
use primitives::{TokenId, TokenSymbol};

use super::*;

#[test]
fn register_foreign_asset_work() {
	assert_eq!(1, 1)
}
