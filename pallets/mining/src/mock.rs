use frame_support::traits::Nothing;
use frame_support::{construct_runtime, ord_parameter_types, parameter_types};
use frame_system::EnsureSignedBy;
use orml_traits::parameter_type_with_key;
use sp_core::H256;
use sp_runtime::BuildStorage;
use sp_runtime::{
	traits::{AccountIdConversion, IdentityLookup},
	Perbill,
};

use primitives::estate::Estate;
use primitives::staking::MetaverseStakingTrait;

use primitives::{Amount, CurrencyId, EstateId, FungibleTokenId, RoundIndex, UndeployedLandBlockId};

use crate as mining;
use crate::Config;

use super::*;

pub type AccountId = u128;
pub type AuctionId = u64;
pub type Balance = u128;
pub type MetaverseId = u64;
pub type BlockNumber = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 5;
pub const DAVE: AccountId = 7;
pub const METAVERSE_ID: MetaverseId = 1;
pub const COUNTRY_ID_NOT_EXIST: MetaverseId = 1;
pub const NUUM: CurrencyId = 0;
pub const COUNTRY_FUND: FungibleTokenId = FungibleTokenId::FungibleToken(1);

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
	type RuntimeOrigin = RuntimeOrigin;
	type Nonce = u64;
	type Block = Block;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
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
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type MaxLocks = ();
	type WeightInfo = ();
	type MaxReserves = ();
	type ReserveIdentifier = ();
	type RuntimeHoldReason = ();
	type FreezeIdentifier = ();
	type MaxHolds = frame_support::traits::ConstU32<0>;
	type MaxFreezes = frame_support::traits::ConstU32<0>;
}
parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: FungibleTokenId| -> Balance {
		Default::default()
	};
}

parameter_types! {
	pub const MetaverseTreasuryPalletId: PalletId = PalletId(*b"bit/trsy");
	pub TreasuryModuleAccount: AccountId = MetaverseTreasuryPalletId::get().into_account_truncating();
	pub const MiningTreasuryPalletId: PalletId = PalletId(*b"bit/fund");
}

impl orml_tokens::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = FungibleTokenId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type CurrencyHooks = ();
	type MaxLocks = ();
	type ReserveIdentifier = [u8; 8];
	type MaxReserves = ();
	type DustRemovalWhitelist = Nothing;
}

pub type AdaptedBasicCurrency = currencies::BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;

parameter_types! {
	pub const GetNativeCurrencyId: FungibleTokenId = FungibleTokenId::NativeToken(0);
	pub const MiningCurrencyId: FungibleTokenId = FungibleTokenId::MiningResource(0);
}

impl currencies::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MultiSocialCurrency = Tokens;
	type NativeCurrency = AdaptedBasicCurrency;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type WeightInfo = ();
}

parameter_types! {
	pub const MinVestedTransfer: Balance = 100;
}

pub struct EstateHandler;

impl Estate<u128> for EstateHandler {
	fn transfer_estate(estate_id: EstateId, _from: &u128, _to: &u128) -> Result<EstateId, DispatchError> {
		Ok(estate_id)
	}

	fn transfer_landunit(
		coordinate: (i32, i32),
		_from: &u128,
		_to: &(u128, primitives::MetaverseId),
	) -> Result<(i32, i32), DispatchError> {
		Ok(coordinate)
	}

	fn transfer_undeployed_land_block(
		_who: &AccountId,
		_to: &AccountId,
		_undeployed_land_block_id: UndeployedLandBlockId,
	) -> Result<UndeployedLandBlockId, DispatchError> {
		Ok(2)
	}

	fn check_estate(_estate_id: EstateId) -> Result<bool, DispatchError> {
		Ok(true)
	}

	fn check_landunit(_metaverse_id: primitives::MetaverseId, _coordinate: (i32, i32)) -> Result<bool, DispatchError> {
		Ok(true)
	}

	fn check_undeployed_land_block(
		_owner: &AccountId,
		_undeployed_land_block_id: UndeployedLandBlockId,
	) -> Result<bool, DispatchError> {
		Ok(true)
	}

	fn get_total_land_units(_estate_id: Option<EstateId>) -> u64 {
		10
	}

	fn get_total_undeploy_land_units() -> u64 {
		10
	}

	fn check_estate_ownership(_owner: AccountId, _estate_id: EstateId) -> Result<bool, DispatchError> {
		Ok(false)
	}

	fn is_estate_leasor(_leasor: AccountId, _estate_id: EstateId) -> Result<bool, DispatchError> {
		Ok(false)
	}

	fn is_estate_leased(_estate_id: EstateId) -> Result<bool, DispatchError> {
		Ok(false)
	}
}

pub struct MetaverseStakingHandler;

impl MetaverseStakingTrait<u128> for MetaverseStakingHandler {
	fn update_staking_reward(_round: RoundIndex, _total_reward: u128) -> sp_runtime::DispatchResult {
		Ok(())
	}
}

parameter_types! {
	pub const TreasuryStakingReward: Perbill = Perbill::from_percent(1);
	pub StorageDepositFee: Balance = 1;
}

impl Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MiningCurrency = Currencies;
	type BitMiningTreasury = MiningTreasuryPalletId;
	type BitMiningResourceId = MiningCurrencyId;
	type EstateHandler = EstateHandler;
	type AdminOrigin = EnsureSignedBy<One, AccountId>;
	type MetaverseStakingHandler = MetaverseStakingHandler;
	type TreasuryStakingReward = TreasuryStakingReward;
	type WeightInfo = ();
	type NetworkTreasuryAccount = TreasuryModuleAccount;
	type StorageDepositFee = StorageDepositFee;
	type Currency = Balances;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Currencies: currencies::{ Pallet, Storage, Call, Event<T>},
		Tokens: orml_tokens::{ Pallet, Storage, Call, Event<T>},
		MiningModule: mining:: {Pallet, Call, Storage, Event<T>},
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
		let mut t = frame_system::GenesisConfig::<Runtime>::default()
			.build_storage()
			.unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: vec![(ALICE, 100000), (BOB, 1000), (TreasuryModuleAccount::get(), 100)],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

pub fn last_event() -> RuntimeEvent {
	frame_system::Module::<Runtime>::events()
		.pop()
		.expect("Event expected")
		.event
}
