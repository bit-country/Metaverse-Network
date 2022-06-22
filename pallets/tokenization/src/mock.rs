use frame_support::pallet_prelude::{GenesisBuild, Hooks, MaybeSerializeDeserialize};
use frame_support::sp_runtime::traits::AtLeast32Bit;
use frame_support::traits::Nothing;
use frame_support::{construct_runtime, ord_parameter_types, parameter_types, traits::EnsureOrigin, weights::Weight};
use frame_system::{EnsureRoot, EnsureSignedBy};
use orml_traits::parameter_type_with_key;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{AccountIdConversion, IdentityLookup},
	Perbill,
};

use primitives::FungibleTokenId::FungibleToken;
use primitives::{Amount, ClassId, CurrencyId, FungibleTokenId};

use crate as tokenization;
use crate::{Config, Module};

use super::*;

pub type AccountId = u128;
pub type AuctionId = u64;
pub type Balance = u128;
pub type MetaverseId = u64;
pub type BlockNumber = u64;

pub const ALICE: AccountId = 4;
pub const BOB: AccountId = 5;
pub const METAVERSE_ID: MetaverseId = 1;
pub const METAVERSE_ID_NOT_EXIST: MetaverseId = 1;
pub const NUUM: CurrencyId = 0;
pub const METAVERSE_FUND: FungibleTokenId = FungibleTokenId::FungibleToken(1);

pub const ALICE_METAVERSE_ID: MetaverseId = 1;
pub const BOB_METAVERSE_ID: MetaverseId = 2;
pub const GENERAL_METAVERSE_FUND: AccountId = 102;

ord_parameter_types! {
	pub const One: AccountId = ALICE;
}

// Configure a mock runtime to test the pallet.

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: u32 = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
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
	type BaseCallFilter = frame_support::traits::Everything;
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
	type AccountStore = System;
	type MaxLocks = ();
	type WeightInfo = ();
	type MaxReserves = ();
	type ReserveIdentifier = ();
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: FungibleTokenId| -> Balance {
		Default::default()
	};
}

parameter_types! {
	pub const MetaverseTreasuryPalletId: PalletId = PalletId(*b"bit/trsy");
	pub TreasuryModuleAccount: AccountId = MetaverseTreasuryPalletId::get().into_account();
	pub const MetaverseFundPalletId: PalletId = PalletId(*b"bit/fund");
}

impl orml_tokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = FungibleTokenId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = orml_tokens::TransferDust<Runtime, TreasuryModuleAccount>;
	type MaxLocks = ();
	type ReserveIdentifier = [u8; 8];
	type MaxReserves = ();
	type DustRemovalWhitelist = Nothing;
}

pub type AdaptedBasicCurrency = currencies::BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;

pub struct MetaverseInfoSource {}

impl MetaverseTrait<AccountId> for MetaverseInfoSource {
	fn create_metaverse(who: &AccountId, metadata: MetaverseMetadata) -> MetaverseId {
		1u64
	}

	fn check_ownership(who: &AccountId, metaverse_id: &MetaverseId) -> bool {
		match *who {
			ALICE => true,
			_ => false,
		}
	}

	fn get_metaverse(metaverse_id: u64) -> Option<MetaverseInfo<u128>> {
		None
	}

	fn get_metaverse_token(metaverse_id: u64) -> Option<FungibleTokenId> {
		None
	}

	fn update_metaverse_token(metaverse_id: u64, currency_id: FungibleTokenId) -> Result<(), DispatchError> {
		Ok(())
	}

	fn get_metaverse_land_class(metaverse_id: MetaverseId) -> Result<ClassId, DispatchError> {
		Ok(15u32)
	}

	fn get_metaverse_estate_class(metaverse_id: MetaverseId) -> Result<ClassId, DispatchError> {
		Ok(16u32)
	}

	fn get_metaverse_marketplace_listing_fee(metaverse_id: MetaverseId) -> Result<Perbill, DispatchError> {
		Ok(Perbill::from_percent(1u32))
	}

	fn get_metaverse_treasury(metaverse_id: MetaverseId) -> AccountId {
		GENERAL_METAVERSE_FUND
	}

	fn get_network_treasury() -> AccountId {
		GENERAL_METAVERSE_FUND
	}

	fn check_if_metaverse_estate(
		metaverse_id: primitives::MetaverseId,
		class_id: &ClassId,
	) -> Result<bool, DispatchError> {
		if class_id == &15u32 || class_id == &16u32 {
			return Ok(true);
		}
		return Ok(false);
	}
}

pub struct DEXManager {}

impl SwapManager<AccountId, FungibleTokenId, Balance> for DEXManager {
	fn add_liquidity(
		who: &AccountId,
		token_id_a: FungibleTokenId,
		token_id_b: FungibleTokenId,
		max_amount_a: Balance,
		max_amount_b: Balance,
	) -> DispatchResult {
		Ok(())
	}
}

parameter_types! {
	pub const SwapPalletId: PalletId = PalletId(*b"bit/swap");
	pub const SwapFee: (u32, u32) = (1, 20); //0.005%
}

impl swap::Config for Runtime {
	type Event = Event;
	type PalletId = SwapPalletId;
	type FungibleTokenCurrency = Tokens;
	type NativeCurrency = Balances;
	type GetSwapFee = SwapFee;
}

parameter_types! {
	pub const GetNativeCurrencyId: FungibleTokenId = FungibleTokenId::NativeToken(0);
}

impl currencies::Config for Runtime {
	type Event = Event;
	type MultiSocialCurrency = Tokens;
	type NativeCurrency = AdaptedBasicCurrency;
	type GetNativeCurrencyId = GetNativeCurrencyId;
}

parameter_types! {
	pub const MinVestedTransfer: Balance = 100;
}

impl Config for Runtime {
	type Event = Event;
	type TokenId = u64;
	type MetaverseMultiCurrency = Currencies;
	type FungibleTokenTreasury = MetaverseFundPalletId;
	type MetaverseInfoSource = MetaverseInfoSource;
	type LiquidityPoolManager = SwapModule;
	type MinVestedTransfer = MinVestedTransfer;
	type VestedTransferOrigin = EnsureSignedBy<One, AccountId>;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Currencies: currencies::{ Pallet, Storage, Call, Event<T>},
		Tokens: orml_tokens::{ Pallet, Storage, Call, Event<T>},
		SwapModule: swap::{Pallet, Call ,Storage, Event<T>},
		TokenizationModule: tokenization:: {Pallet, Call, Storage, Event<T>},
	}
);

pub struct ExtBuilder;

impl Default for ExtBuilder {
	fn default() -> Self {
		ExtBuilder
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: vec![(ALICE, 100000)],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

pub fn last_event() -> Event {
	frame_system::Module::<Runtime>::events()
		.pop()
		.expect("Event expected")
		.event
}
