#![cfg(test)]

use sp_std::collections::btree_map::BTreeMap;
use core_primitives::{CollectionType, NftClassData, TokenType};
use frame_support::{construct_runtime, ord_parameter_types, parameter_types, PalletId};
use frame_system::EnsureSignedBy;
use primitives::{Attributes, ClassId, FungibleTokenId, GroupCollectionId, NftMetadata, TokenId};
use sp_runtime::{
	traits::{ConvertInto, IdentityLookup},
	BuildStorage, DispatchError, Perbill,
};
use sp_std::default::Default;
use sp_runtime::testing::H256;


use crate as nft_migration;
use super::*;

pub type AccountId = u128;
pub type Balance = u128;
pub type BlockNumber = u64;
pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 5;

fn test_attributes(x: u8) -> Attributes {
	let mut attr: Attributes = BTreeMap::new();
	attr.insert(vec![x, x + 5], vec![x, x + 10]);
	attr
}

ord_parameter_types! {
	pub const Alice: AccountId = ALICE;
}

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

pub struct MockNFTHandler;

impl NFTTrait<AccountId, Balance> for MockNFTHandler {
	type TokenId = TokenId;
	type ClassId = ClassId;

	fn check_ownership(who: &AccountId, asset_id: &(Self::ClassId, Self::TokenId)) -> Result<bool, DispatchError> {
		Ok(false)
	}

	fn check_collection_and_class(
		collection_id: GroupCollectionId,
		class_id: Self::ClassId,
	) -> Result<bool, DispatchError> {
		Ok(false)
	}
	fn get_nft_group_collection(_nft_collection: &Self::ClassId) -> Result<GroupCollectionId, DispatchError> {
		Ok(0)
	}

	fn is_stackable(_asset_id: (Self::ClassId, Self::TokenId)) -> Result<bool, DispatchError> {
		Ok(false)
	}

	fn create_token_class(
		sender: &AccountId,
		_metadata: NftMetadata,
		_attributes: Attributes,
		collection_id: GroupCollectionId,
		_token_type: TokenType,
		_collection_type: CollectionType,
		_royalty_fee: Perbill,
		_mint_limit: Option<u32>,
	) -> Result<ClassId, DispatchError> {
		Ok(0)
	}

	fn mint_token(
		sender: &AccountId,
		class_id: ClassId,
		_metadata: NftMetadata,
		_attributes: Attributes,
	) -> Result<TokenId, DispatchError> {
		Ok(1)
	}

	fn transfer_nft(_from: &AccountId, _to: &AccountId, _nft: &(Self::ClassId, Self::TokenId)) -> DispatchResult {
		Ok(())
	}

	fn check_item_on_listing(_class_id: Self::ClassId, _token_id: Self::TokenId) -> Result<bool, DispatchError> {
		Ok(true)
	}

	fn burn_nft(_account: &AccountId, _nft: &(Self::ClassId, Self::TokenId)) -> DispatchResult {
		Ok(())
	}
	fn is_transferable(_nft: &(Self::ClassId, Self::TokenId)) -> Result<bool, DispatchError> {
		Ok(true)
	}

	fn get_class_fund(_class_id: &Self::ClassId) -> AccountId {
		0
	}

	fn get_nft_detail(_asset_id: (Self::ClassId, Self::TokenId)) -> Result<NftClassData<Balance>, DispatchError> {
		let new_data = NftClassData {
			deposit: 0,
			attributes: test_attributes(1),
			token_type: TokenType::Transferable,
			collection_type: CollectionType::Collectable,
			is_locked: false,
			royalty_fee: Perbill::from_percent(0u32),
			mint_limit: None,
			total_minted_tokens: 0u32,
		};
		Ok(new_data)
	}

	fn set_lock_collection(_class_id: Self::ClassId, _is_locked: bool) -> sp_runtime::DispatchResult {
		Ok(())
	}

	fn set_lock_nft(_token_id: (Self::ClassId, Self::TokenId), _is_locked: bool) -> sp_runtime::DispatchResult {
		Ok(())
	}

	fn get_nft_class_detail(_class_id: Self::ClassId) -> Result<NftClassData<Balance>, DispatchError> {
		let new_data = NftClassData {
			deposit: 0,
			attributes: test_attributes(1),
			token_type: TokenType::Transferable,
			collection_type: CollectionType::Collectable,
			is_locked: false,
			royalty_fee: Perbill::from_percent(0u32),
			mint_limit: None,
			total_minted_tokens: 0u32,
		};
		Ok(new_data)
	}

	fn get_total_issuance(_class_id: Self::ClassId) -> Result<Self::TokenId, DispatchError> {
		Ok(1u64)
	}

	fn get_asset_owner(_asset_id: &(Self::ClassId, Self::TokenId)) -> Result<AccountId, DispatchError> {
		Ok(ALICE)
	}

	fn mint_token_with_id(
		sender: &AccountId,
		class_id: Self::ClassId,
		_token_id: Self::TokenId,
		_metadata: core_primitives::NftMetadata,
		_attributes: core_primitives::Attributes,
	) -> Result<Self::TokenId, DispatchError> {
		Ok(2)
	}

	fn get_free_stackable_nft_balance(_who: &AccountId, _asset_id: &(Self::ClassId, Self::TokenId)) -> Balance {
		0u128
	}

	fn reserve_stackable_nft_balance(
		_who: &AccountId,
		_asset_id: &(Self::ClassId, Self::TokenId),
		_amount: Balance,
	) -> DispatchResult {
		Ok(())
	}

	fn unreserve_stackable_nft_balance(
		_who: &AccountId,
		_asset_id: &(Self::ClassId, Self::TokenId),
		_amount: Balance,
	) -> sp_runtime::DispatchResult {
		Ok(())
	}

	fn transfer_stackable_nft(
		_sender: &AccountId,
		_to: &AccountId,
		_nft: &(Self::ClassId, Self::TokenId),
		_amount: Balance,
	) -> sp_runtime::DispatchResult {
		Ok(())
	}
}

impl Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type NFTSource = MockNFTHandler;
	type MigrationOrigin = EnsureSignedBy<Alice, AccountId>;
}

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		NftMigration: nft_migration::{Pallet, Call, Storage, Event<T>},
	}
);

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

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
			balances: vec![(ALICE, 100000), (BOB, 100000)],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

pub fn last_event() -> RuntimeEvent {
	frame_system::Pallet::<Runtime>::events()
		.pop()
		.expect("Event expected")
		.event
}

fn next_block() {
	NftMigration::on_finalize(System::block_number());
	System::set_block_number(System::block_number() + 1);
	NftMigration::on_initialize(System::block_number());
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		next_block();
	}
}
