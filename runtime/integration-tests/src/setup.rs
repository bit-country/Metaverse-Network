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

pub use codec::Encode;
use cumulus_test_relay_sproof_builder::RelayStateSproofBuilder;
use frame_support::traits::{GenesisBuild, OnFinalize, OnIdle, OnInitialize};
pub use frame_support::{assert_noop, assert_ok, traits::Currency};
pub use frame_system::RawOrigin;
pub use orml_traits::{location::RelativeLocations, Change, GetByKey, MultiCurrency};
pub use sp_core::H160;
use sp_io::hashing::keccak_256;
pub use sp_runtime::{
	traits::{AccountIdConversion, BadOrigin, BlakeTwo256, Convert, Hash, Zero},
	DispatchError, DispatchResult, FixedPointNumber, MultiAddress, Perbill, Permill,
};
pub use xcm::latest::prelude::*;

pub use pioneer_imports::*;
pub use primitives::{AccountId, BlockNumber, CurrencyId, FungibleTokenId};

pub fn dollar(d: u32) -> Balance {
	let d: Balance = d.into();
	d.saturating_mul(1_000_000_000_000_000_000)
}

mod pioneer_imports {
	pub use frame_support::{parameter_types, weights::Weight};
	pub use sp_runtime::{traits::AccountIdConversion, FixedPointNumber};

	pub use pioneer_runtime::{
		constants::parachains, create_x2_parachain_multilocation, AccountId, Balance, Balances, BlockNumber, Call,
		Currencies, Event, ExistentialDeposits, FungibleTokenIdConvert, MultiLocation, NetworkId, NftPalletId, Origin,
		OriginCaller, ParachainAccount, ParachainInfo, ParachainSystem, PolkadotXcm, Runtime, Scheduler, Session,
		System, Timestamp, TreasuryModuleAccount, TreasuryPalletId, Utility, Vesting, XTokens, XcmConfig, XcmExecutor,
	};
	use primitives::FungibleTokenId::*;
	use primitives::{CurrencyId, FungibleTokenId};

	// 0 => NEER
	// 1 => KSM
	// 2 => KAR
	// 3 => KUSD
	pub const NATIVE_CURRENCY: FungibleTokenId = NativeToken(0);
	pub const RELAY_CHAIN_CURRENCY: FungibleTokenId = FungibleToken(1);
	pub const PARA_CHAIN_CURRENCY: FungibleTokenId = FungibleToken(2);
	pub const STABLE_CURRENCY: FungibleTokenId = Stable(3);

	pub const NATIVE_CURRENCY_ID: CurrencyId = 0;
	pub const RELAY_CHAIN_CURRENCY_ID: CurrencyId = 1;
	pub const PARA_CHAIN_CURRENCY_ID: CurrencyId = 2;
	pub const STABLE_CURRENCY_ID: CurrencyId = 3;
}

const ORACLE1: [u8; 32] = [0u8; 32];
const ORACLE2: [u8; 32] = [1u8; 32];
const ORACLE3: [u8; 32] = [2u8; 32];
const ORACLE4: [u8; 32] = [3u8; 32];
const ORACLE5: [u8; 32] = [4u8; 32];

#[allow(dead_code)]
pub const DEFAULT: [u8; 32] = [0u8; 32];

pub const ALICE: [u8; 32] = [4u8; 32];
pub const BOB: [u8; 32] = [5u8; 32];
pub const CHARLIE: [u8; 32] = [6u8; 32];
pub const DAVE: [u8; 32] = [7u8; 32];

pub const INIT_TIMESTAMP: u64 = 30_000;
pub const BLOCK_TIME: u64 = 1000;

// pub fn run_to_block(n: u32) {
// 	while System::block_number() < n {
// 		Scheduler::on_finalize(System::block_number());
// 		System::set_block_number(System::block_number() + 1);
// 		Timestamp::set_timestamp((System::block_number() as u64 * BLOCK_TIME) + INIT_TIMESTAMP);
// 		Scheduler::on_initialize(System::block_number());
// 		Session::on_initialize(System::block_number());
// 		SessionManager::on_initialize(System::block_number());
// 		IdleScheduler::on_idle(System::block_number(), u64::MAX);
// 	}
// }

pub fn set_relaychain_block_number(number: BlockNumber) {
	ParachainSystem::on_initialize(number);

	let (relay_storage_root, proof) = RelayStateSproofBuilder::default().into_state_root_and_proof();

	assert_ok!(ParachainSystem::set_validation_data(
		Origin::none(),
		cumulus_primitives_parachain_inherent::ParachainInherentData {
			validation_data: cumulus_primitives_core::PersistedValidationData {
				parent_head: Default::default(),
				relay_parent_number: number,
				relay_parent_storage_root: relay_storage_root,
				max_pov_size: Default::default(),
			},
			relay_chain_state: proof,
			downward_messages: Default::default(),
			horizontal_messages: Default::default(),
		}
	));
}

pub struct ExtBuilder {
	balances: Vec<(AccountId, FungibleTokenId, Balance)>,
	parachain_id: u32,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			balances: vec![],
			parachain_id: 2000,
		}
	}
}

impl ExtBuilder {
	pub fn balances(mut self, balances: Vec<(AccountId, FungibleTokenId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	#[allow(dead_code)]
	pub fn parachain_id(mut self, parachain_id: u32) -> Self {
		self.parachain_id = parachain_id;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		// let evm_genesis_accounts = evm_genesis(vec![]);

		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		let native_currency_id = NATIVE_CURRENCY_ID;
		let existential_deposit = ExistentialDeposits::get(&NATIVE_CURRENCY);

		#[cfg(feature = "with-mandala-runtime")]
		GenesisBuild::<Runtime>::assimilate_storage(
			&ecosystem_renvm_bridge::GenesisConfig {
				ren_vm_public_key: hex_literal::hex!["4b939fc8ade87cb50b78987b1dda927460dc456a"],
			},
			&mut t,
		)
		.unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: self
				.balances
				.clone()
				.into_iter()
				.filter(|(_, currency_id, _)| *currency_id == NATIVE_CURRENCY)
				.map(|(account_id, _, initial_balance)| (account_id, initial_balance))
				// .chain(
				// 	get_all_module_accounts()
				// 		.iter()
				// 		.map(|x| (x.clone(), existential_deposit)),
				// )
				.collect::<Vec<_>>(),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		orml_tokens::GenesisConfig::<Runtime> {
			balances: self
				.balances
				.into_iter()
				.filter(|(_, currency_id, _)| *currency_id != RELAY_CHAIN_CURRENCY)
				.collect::<Vec<_>>(),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		// pallet_membership::GenesisConfig::<Runtime, pallet_membership::Instance5> {
		// 	members: vec![
		// 		AccountId::from(ORACLE1),
		// 		AccountId::from(ORACLE2),
		// 		AccountId::from(ORACLE3),
		// 		AccountId::from(ORACLE4),
		// 		AccountId::from(ORACLE5),
		// 	],
		// 	phantom: Default::default(),
		// }
		// .assimilate_storage(&mut t)
		// .unwrap();

		// module_evm::GenesisConfig::<Runtime> {
		// 	accounts: evm_genesis_accounts,
		// }
		// .assimilate_storage(&mut t)
		// .unwrap();

		// module_session_manager::GenesisConfig::<Runtime> { session_duration: 10 }
		// 	.assimilate_storage(&mut t)
		// 	.unwrap();

		<parachain_info::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
			&parachain_info::GenesisConfig {
				parachain_id: self.parachain_id.into(),
			},
			&mut t,
		)
		.unwrap();

		<pallet_xcm::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
			&pallet_xcm::GenesisConfig {
				safe_xcm_version: Some(2),
			},
			&mut t,
		)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

// pub fn alice_key() -> libsecp256k1::SecretKey {
// 	libsecp256k1::SecretKey::parse(&keccak_256(b"Alice")).unwrap()
// }
//
// pub fn bob_key() -> libsecp256k1::SecretKey {
// 	libsecp256k1::SecretKey::parse(&keccak_256(b"Bob")).unwrap()
// }

// pub fn alice() -> AccountId {
// 	// let address = EvmAccounts::eth_address(&alice_key());
// 	let mut data = [0u8; 32];
// 	// data[0..4].copy_from_slice(b"evm:");
// 	data[4..24].copy_from_slice(&address[..]);
// 	AccountId::from(Into::<[u8; 32]>::into(data))
// }
//
// pub fn bob() -> AccountId {
// 	// let address = EvmAccounts::eth_address(&bob_key());
// 	let mut data = [0u8; 32];
// 	// data[0..4].copy_from_slice(b"evm:");
// 	data[4..24].copy_from_slice(&address[..]);
// 	AccountId::from(Into::<[u8; 32]>::into(data))
// }
