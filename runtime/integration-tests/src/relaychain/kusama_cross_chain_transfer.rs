// This file is part of Bit.Country.

// Copyright (C) 2020-2021 Bit.Country.
// SPDX-License-Identifier: Apache-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Cross-chain transfer tests within Kusama network.

use frame_support::assert_ok;
use orml_traits::MultiCurrency;
use xcm::v0::OriginKind::Native;
use xcm_emulator::TestExt;

use module_relaychain::RelayChainCallBuilder;
use module_support::CallBuilder;
use pioneer_runtime::TreasuryModuleAccount;

use crate::relaychain::kusama_test_net::*;
use crate::setup::*;

#[test]
fn transfer_from_relay_chain() {
	KusamaNet::execute_with(|| {
		assert_eq!(
			kusama_runtime::Balances::free_balance(&AccountId::from(ALICE)),
			2002_u128.saturating_mul(1_000_000_000_000_000_000)
		);
	});

	KusamaNet::execute_with(|| {
		assert_ok!(kusama_runtime::XcmPallet::reserve_transfer_assets(
			kusama_runtime::Origin::signed(ALICE.into()),
			Box::new(Parachain(2000).into().into()),
			Box::new(
				Junction::AccountId32 {
					id: BOB,
					network: NetworkId::Any
				}
				.into()
				.into()
			),
			Box::new((Here, dollar(RELAY_CHAIN_CURRENCY_ID)).into()),
			0
		));
	});

	Pioneer::execute_with(|| {
		assert_eq!(
			Tokens::free_balance(RELAY_CHAIN_CURRENCY, &AccountId::from(BOB)),
			999_999_996_000_000_000
		);
	});
}

#[test]
fn transfer_to_relay_chain() {
	Pioneer::execute_with(|| {
		assert_ok!(Tokens::deposit(RELAY_CHAIN_CURRENCY, &AccountId::from(ALICE), 10*dollar(RELAY_CHAIN_CURRENCY_ID)));

		assert_ok!(XTokens::transfer(
			Origin::signed(ALICE.into()),
			RELAY_CHAIN_CURRENCY,
			dollar(RELAY_CHAIN_CURRENCY_ID),
			Box::new(
				MultiLocation::new(
					1,
					X1(Junction::AccountId32 {
						id: BOB,
						network: NetworkId::Any,
					})
				)
				.into()
			),
			4_000_000_000
		));
	});

	KusamaNet::execute_with(|| {
		assert_eq!(
			kusama_runtime::Balances::free_balance(&AccountId::from(BOB)),
			999_999_999_893_333_340
		);
	});
}

#[test]
fn transfer_to_sibling() {
	TestNet::reset();

	fn neer_reserve_account() -> AccountId {
		use sp_runtime::traits::AccountIdConversion;
		polkadot_parachain::primitives::Sibling::from(2000).into_account()
	}

	Pioneer::execute_with(|| {
		assert_ok!(Tokens::deposit(PARA_CHAIN_CURRENCY, &AccountId::from(ALICE), 100_000_000_000_000));
	});

	Sibling::execute_with(|| {
		assert_ok!(Tokens::deposit(PARA_CHAIN_CURRENCY, &neer_reserve_account(), 100_000_000_000_000));
	});

	Pioneer::execute_with(|| {
		assert_eq!(Tokens::free_balance(PARA_CHAIN_CURRENCY, &AccountId::from(ALICE)), 100_000_000_000_000);

		assert_ok!(XTokens::transfer(
			Origin::signed(ALICE.into()),
			PARA_CHAIN_CURRENCY,
			10_000_000_000_000,
			Box::new(
				MultiLocation::new(
					1,
					X2(
						Parachain(2001),
						Junction::AccountId32 {
							network: NetworkId::Any,
							id: BOB.into(),
						}
					)
				)
				.into()
			),
			1_000_000_000,
		));

		assert_eq!(Tokens::free_balance(PARA_CHAIN_CURRENCY, &AccountId::from(ALICE)), 90_000_000_000_000);
	});

	Sibling::execute_with(|| {
		assert_eq!(Tokens::free_balance(PARA_CHAIN_CURRENCY, &neer_reserve_account()), 100_000_000_000_000);

		// Check if token received correctly
		assert_eq!(Tokens::free_balance(PARA_CHAIN_CURRENCY, &AccountId::from(BOB)), 10_000_000_000_000);
		// assert_eq!(Currencies::free_balance(PARA_CHAIN_CURRENCY, &AccountId::from(BOB)), 10_000_000_000_000);

		assert_ok!(XTokens::transfer(
			Origin::signed(BOB.into()),
			PARA_CHAIN_CURRENCY,
			5_000_000_000_000,
			Box::new(
				MultiLocation::new(
					1,
					X2(
						Parachain(2000),
						Junction::AccountId32 {
							network: NetworkId::Any,
							id: ALICE.into(),
						}
					)
				)
				.into()
			),
			1_000_000_000,
		));

		assert_eq!(Tokens::free_balance(PARA_CHAIN_CURRENCY, &AccountId::from(BOB)), 5_000_000_000_000);
	});

	Pioneer::execute_with(|| {
		assert_eq!(Tokens::free_balance(PARA_CHAIN_CURRENCY, &AccountId::from(ALICE)), 95_000_000_000_000);
	});
}