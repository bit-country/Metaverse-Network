// This file is part of Metaverse.Network & Bit.Country.

// Copyright (C) 2020-2022 Metaverse.Network & Bit.Country .
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Relay chain and parachains emulation.
use crate::setup::*;
use cumulus_primitives_core::ParaId;
use frame_support::traits::GenesisBuild;
use polkadot_primitives::v1::{BlockNumber, MAX_CODE_SIZE, MAX_POV_SIZE};
use polkadot_runtime_parachains::configuration::HostConfiguration;
use sp_runtime::traits::AccountIdConversion;
use xcm_emulator::{decl_test_network, decl_test_parachain, decl_test_relay_chain};

use core_traits::FungibleTokenId;
use pioneer_runtime::AccountId;

decl_test_relay_chain! {
	pub struct KusamaNet {
		Runtime = kusama_runtime::Runtime,
		XcmConfig = kusama_runtime::xcm_config::XcmConfig,
		new_ext = kusama_ext(),
	}
}

decl_test_parachain! {
	pub struct Development {
		Runtime = pioneer_runtime::Runtime,
		Origin = pioneer_runtime::Origin,
		XcmpMessageHandler = pioneer_runtime::XcmpQueue,
		DmpMessageHandler = pioneer_runtime::DmpQueue,
		new_ext = para_ext(PARA_ID_DEVELOPMENT),
	}
}

decl_test_parachain! {
	pub struct Sibling {
		Runtime = pioneer_runtime::Runtime,
		Origin = pioneer_runtime::Origin,
		XcmpMessageHandler = pioneer_runtime::XcmpQueue,
		DmpMessageHandler = pioneer_runtime::DmpQueue,
		new_ext = para_ext(PARA_ID_SIBLING),
	}
}

decl_test_parachain! {
	pub struct Karura {
		Runtime = pioneer_runtime::Runtime,
		Origin = pioneer_runtime::Origin,
		XcmpMessageHandler = pioneer_runtime::XcmpQueue,
		DmpMessageHandler = pioneer_runtime::DmpQueue,
		new_ext = para_ext(PARA_ID_KARURA),
	}
}
/*
decl_test_parachain! {
	pub struct Statemine {
		Runtime = statemine_runtime::Runtime,
		Origin = statemine_runtime::Origin,
		XcmpMessageHandler = statemine_runtime::XcmpQueue,
		DmpMessageHandler = statemine_runtime::DmpQueue,
		new_ext = para_ext(PARA_ID_STATEMINE),
	}
}
*/
decl_test_network! {
	pub struct TestNet {
		relay_chain = KusamaNet,
		parachains = vec![
			// Be sure to use `PARA_ID_STATEMINE`
			// (PARA_ID_STATEMINE, Statemine),
			// Be sure to use `PARA_ID_KARURA`
			(2000, Karura),
			// Be sure to use `PARA_ID_DEVELOPMENT`
			(2096, Development),
			// Be sure to use `PARA_ID_SIBLING`
			(3096, Sibling),
		],
	}
}

fn default_parachains_host_configuration() -> HostConfiguration<BlockNumber> {
	HostConfiguration {
		minimum_validation_upgrade_delay: 5,
		validation_upgrade_cooldown: 5u32,
		validation_upgrade_delay: 5,
		code_retention_period: 1200,
		max_code_size: MAX_CODE_SIZE,
		max_pov_size: MAX_POV_SIZE,
		max_head_data_size: 32 * 1024,
		group_rotation_frequency: 20,
		chain_availability_period: 4,
		thread_availability_period: 4,
		max_upward_queue_count: 8,
		max_upward_queue_size: 1024 * 1024,
		max_downward_message_size: 1024,
		ump_service_total_weight: 4 * 1_000_000_000,
		max_upward_message_size: 50 * 1024,
		max_upward_message_num_per_candidate: 5,
		hrmp_sender_deposit: 0,
		hrmp_recipient_deposit: 0,
		hrmp_channel_max_capacity: 8,
		hrmp_channel_max_total_size: 8 * 1024,
		hrmp_max_parachain_inbound_channels: 4,
		hrmp_max_parathread_inbound_channels: 4,
		hrmp_channel_max_message_size: 1024 * 1024,
		hrmp_max_parachain_outbound_channels: 4,
		hrmp_max_parathread_outbound_channels: 4,
		hrmp_max_message_num_per_candidate: 5,
		dispute_period: 6,
		no_show_slots: 2,
		n_delay_tranches: 25,
		needed_approvals: 2,
		relay_vrf_modulo_samples: 2,
		zeroth_delay_tranche_width: 0,
		..Default::default()
	}
}

pub fn kusama_ext() -> sp_io::TestExternalities {
	use kusama_runtime::{Runtime, System};

	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
		.unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![
			(AccountId::from(ALICE), native_amount(10000)),
			(ParaId::from(PARA_ID_DEVELOPMENT).into_account(), native_amount(10000)),
			(ParaId::from(PARA_ID_SIBLING).into_account(), native_amount(10000)),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	polkadot_runtime_parachains::configuration::GenesisConfig::<Runtime> {
		config: default_parachains_host_configuration(),
	}
	.assimilate_storage(&mut t)
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

pub fn para_ext(parachain_id: u32) -> sp_io::TestExternalities {
	ExtBuilder::default()
		.balances(vec![
			(
				AccountId::from(ALICE),
				FungibleTokenId::NativeToken(0),
				native_amount(10000),
			),
			(
				AccountId::from(BOB),
				FungibleTokenId::NativeToken(0),
				native_amount(10000),
			),
			(
				AccountId::from(ALICE),
				FungibleTokenId::NativeToken(1),
				ksm_amount(10000),
			),
			(
				pioneer_runtime::TreasuryModuleAccount::get(),
				FungibleTokenId::NativeToken(1),
				ksm_amount(10000),
			),
		])
		.parachain_id(parachain_id)
		.build()
}
