// This file is part of Metaverse.Network & Bit.Country.

// The evm-mapping pallet is inspired by evm mapping designed by AcalaNetwork

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

#![cfg(test)]

use codec::Encode;
use frame_support::dispatch::DispatchError;
use frame_support::traits::{ConstU32, EqualPrivilegeOnly, Everything, Nothing};
use frame_support::{construct_runtime, ord_parameter_types, parameter_types};
use frame_support::{pallet_prelude::Hooks, weights::Weight, PalletId};
use frame_system::{EnsureRoot, EnsureSignedBy};
use orml_traits::parameter_type_with_key;
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, Hash, IdentityLookup},
};

use primitives::{Amount, Balance, FungibleTokenId};

use crate as evm_account;
use crate::mock::secp_utils::eth;

use super::*;

mod pallet_evm_mapping {
	pub use super::super::*;
}

pub type AccountId = AccountId32;
pub type BlockNumber = u64;

pub const ALICE: AccountId = AccountId32::new([0u8; 32]);
pub const BOB: AccountId = AccountId32::new([1u8; 32]);

parameter_types! {
	pub const BlockHashCount: u32 = 250;
}
impl frame_system::Config for Runtime {
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Call = Call;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type BlockWeights = ();
	type BlockLength = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = Everything;
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = frame_system::Pallet<Runtime>;
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: FungibleTokenId| -> Balance {
		Default::default()
	};
}

impl orml_tokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = FungibleTokenId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = ();
	type MaxLocks = ();
	type ReserveIdentifier = [u8; 8];
	type MaxReserves = ();
	type DustRemovalWhitelist = Nothing;
	type OnNewTokenAccount = ();
	type OnKilledTokenAccount = ();
}

parameter_types! {
	pub const GetNativeCurrencyId: FungibleTokenId = FungibleTokenId::NativeToken(0);
}

impl orml_currencies::Config for Runtime {
	type MultiCurrency = Tokens;
	type NativeCurrency = AdaptedBasicCurrency;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type WeightInfo = ();
}

pub type AdaptedBasicCurrency = orml_currencies::BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

impl Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type ChainId = ();
	type AddressMapping = EvmAddressMapping<Runtime>;
	type TransferAll = Currencies;
}

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		EVMMapping: pallet_evm_mapping::{Pallet, Call ,Storage, Event<T>},
		Tokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>},
		Currencies: orml_currencies::{Pallet, Call},
	}
);

#[cfg(any(test, feature = "runtime-benchmarks"))]
pub mod secp_utils {
	use super::*;

	pub fn public(secret: &libsecp256k1::SecretKey) -> libsecp256k1::PublicKey {
		libsecp256k1::PublicKey::from_secret_key(secret)
	}

	pub fn eth(secret: &libsecp256k1::SecretKey) -> EvmAddress {
		let mut res = EvmAddress::default();
		res.0
			.copy_from_slice(&keccak_256(&public(secret).serialize()[1..65])[12..]);
		res
	}

	pub fn sig<T: Config>(secret: &libsecp256k1::SecretKey, what: &[u8], extra: &[u8]) -> EcdsaSignature {
		let msg = keccak_256(&<super::Pallet<T>>::ethereum_signable_message(
			&to_ascii_hex(what)[..],
			extra,
		));
		let (sig, recovery_id) = libsecp256k1::sign(&libsecp256k1::Message::parse(&msg), secret);
		let mut r = [0u8; 65];
		r[0..64].copy_from_slice(&sig.serialize()[..]);
		r[64] = recovery_id.serialize();
		EcdsaSignature(r)
	}
}

pub struct ExtBuilder();

impl Default for ExtBuilder {
	fn default() -> Self {
		Self()
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: vec![(bob_account_id(), 100000)],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

pub fn alice() -> libsecp256k1::SecretKey {
	libsecp256k1::SecretKey::parse(&keccak_256(b"Alice")).unwrap()
}

pub fn bob() -> libsecp256k1::SecretKey {
	libsecp256k1::SecretKey::parse(&keccak_256(b"Bob")).unwrap()
}

// Substrate account of bob which derive from eth address of bob
pub fn bob_account_id() -> AccountId {
	let address = eth(&bob());
	let mut data = [0u8; 32];
	data[0..4].copy_from_slice(b"evm:");
	data[4..24].copy_from_slice(&address[..]);
	AccountId32::from(Into::<[u8; 32]>::into(data))
}
