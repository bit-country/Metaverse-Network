pub use codec::{Decode, Encode};
use cumulus_test_relay_sproof_builder::RelayStateSproofBuilder;
use frame_support::traits::{GenesisBuild, OnFinalize, OnIdle, OnInitialize};
pub use frame_support::{assert_noop, assert_ok, traits::Currency};
pub use frame_system::RawOrigin;

pub use cumulus_pallet_parachain_system::RelaychainBlockNumberProvider;
pub use orml_traits::{location::RelativeLocations, Change, GetByKey, MultiCurrency};

pub use primitives::currency::*;
pub use sp_core::H160;
use sp_io::hashing::keccak_256;
pub use sp_runtime::{
	traits::{AccountIdConversion, BadOrigin, BlakeTwo256, Convert, Hash, Zero},
	DispatchError, DispatchResult, FixedPointNumber, MultiAddress, Perbill, Permill,
};

use core_traits::{Balance, FungibleTokenId};

/// Accounts
pub const ALICE: [u8; 32] = [4u8; 32];
pub const BOB: [u8; 32] = [5u8; 32];
pub const FRED: [u8; 32] = [6u8; 32];

/// Parachain Ids
pub const PARA_ID_DEVELOPMENT: u32 = 2096;
pub const PARA_ID_SIBLING: u32 = 3096;
pub const PARA_ID_KARURA: u32 = 2000;
pub const PARA_ID_STATEMINE: u32 = 1000;

#[cfg(feature = "with-metaverse-runtime")]
pub use mod metaverse_imports::*;
#[cfg(feature = "with-metaverse-runtime")]
mod metaverse_imports {
	pub use metaverse_runtime::xcm_config::*;
	pub use metaverse_runtime::{AccountId, Origin, Runtime, System};
	pub use sp_runtime::traits::AccountIdConversion;
	use sp_runtime::Percent;
	pub use xcm_executor::XcmExecutor;
	pub const NATIVE_TOKEN_SYMBOL: TokenSymbol = TokenSymbol::AlphaNEER;
}
#[cfg(feature = "with-pioneer-runtime")]
pub use mod pioneer_imports::*;
#[cfg(feature = "with-pioneer-runtime")]
mod pioneer_imports {
	pub use pioneer_runtime::{AccountId, Origin, Runtime, System};
	pub use sp_runtime::traits::AccountIdConversion;
	use sp_runtime::Percent;
	pub const NATIVE_TOKEN_SYMBOL: TokenSymbol = TokenSymbol::NEER;
}

pub struct ExtBuilder {
	balances: Vec<(AccountId, FungibleTokenId, Balance)>,
	parachain_id: u32,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			balances: vec![],
			parachain_id: PARA_ID_DEVELOPMENT,
		}
	}
}

impl ExtBuilder {
	pub fn balances(mut self, balances: Vec<(AccountId, FungibleTokenId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	pub fn parachain_id(mut self, parachain_id: u32) -> Self {
		self.parachain_id = parachain_id;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();
		let native_currency_id = pioneer_runtime::GetNativeCurrencyId::get();
		pallet_balances::GenesisConfig::<Runtime> {
			balances: self
				.balances
				.clone()
				.into_iter()
				.filter(|(_, currency_id, _)| *currency_id == native_currency_id)
				.map(|(account_id, _, initial_balance)| (account_id, initial_balance))
				.collect::<Vec<_>>(),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		orml_tokens::GenesisConfig::<Runtime> {
			balances: self
				.balances
				.into_iter()
				.filter(|(_, currency_id, _)| *currency_id != native_currency_id)
				.collect::<Vec<_>>(),
		}
		.assimilate_storage(&mut t)
		.unwrap();

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

pub fn native_amount(amount: Balance) -> Balance {
	amount * dollar(FungibleTokenId::NativeToken(0))
}

pub fn bit_amount(amount: Balance) -> Balance {
	amount * dollar(FungibleTokenId::MiningResource(0))
}

pub fn kusd_amount(amount: Balance) -> Balance {
	amount * dollar(FungibleTokenId::Stable(0))
}

pub fn kar_amount(amount: Balance) -> Balance {
	amount * dollar(FungibleTokenId::NativeToken(2))
}

pub fn ksm_amount(amount: Balance) -> Balance {
	amount * dollar(FungibleTokenId::NativeToken(1))
}

pub fn dollar(currency_id: FungibleTokenId) -> Balance {
	10u128.saturating_pow(currency_id.decimals().into())
}

pub fn sibling_account() -> AccountId {
	parachain_account(PARA_ID_SIBLING.into())
}

pub fn karura_account() -> AccountId {
	parachain_account(PARA_ID_KARURA.into())
}

pub fn development_account() -> AccountId {
	parachain_account(PARA_ID_DEVELOPMENT.into())
}

fn parachain_account(id: u32) -> AccountId {
	use sp_runtime::traits::AccountIdConversion;

	polkadot_parachain::primitives::Sibling::from(id).into_account()
}
