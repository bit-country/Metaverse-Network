#![cfg(test)]

use codec::Encode;
use frame_support::dispatch::DispatchError;
use frame_support::traits::{EqualPrivilegeOnly, Nothing};
use frame_support::{construct_runtime, ord_parameter_types, parameter_types};
use frame_support::{pallet_prelude::Hooks, weights::Weight, PalletId};
use frame_system::{EnsureRoot, EnsureSignedBy};
use orml_traits::parameter_type_with_key;
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{AccountIdConversion, BlakeTwo256, Hash, IdentityLookup},
	Perbill,
};
use sp_std::collections::btree_map::BTreeMap;

use metaverse_primitive::{
	Attributes, CollectionType, MetaverseInfo as MetaversePrimitiveInfo, MetaverseLandTrait, MetaverseMetadata,
	MetaverseTrait, NFTTrait, NftClassData, NftMetadata, TokenType,
};
use primitives::{Amount, ClassId, FungibleTokenId, GroupCollectionId, TokenId};

use crate as governance;

use super::*;

parameter_types! {
	pub const BlockHashCount: u32 = 256;
	pub const MaximumBlockWeight: u32 = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
}

pub type AccountId = u64;
pub type Balance = u64;
pub type CountryId = u64;
pub type BlockNumber = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const ALICE_COUNTRY_ID: CountryId = 1;
pub const BOB_COUNTRY_ID: CountryId = 2;
pub const PROPOSAL_BLOCK: BlockNumber = 12;
pub const PROPOSAL_DESCRIPTION: [u8; 2] = [1, 2];
pub const REFERENDUM_PARAMETERS: ReferendumParameters<BlockNumber> = ReferendumParameters {
	voting_threshold: Some(VoteThreshold::RelativeMajority),
	min_proposal_launch_period: 12,
	voting_period: 5,
	enactment_period: 10,
	local_vote_locking_period: 30,
	max_proposals_per_metaverse: 10,
};

pub const VOTE_FOR: Vote<Balance> = Vote {
	aye: true,
	balance: 10,
	conviction: Conviction::None,
};

pub const VOTE_AGAINST: Vote<Balance> = Vote {
	aye: false,
	balance: 10,
	conviction: Conviction::None,
};

pub const CLASS_FUND_ID: AccountId = 123;
pub const BENEFICIARY_ID: AccountId = 99;
pub const ASSET_ID_1: TokenId = 101;
pub const ASSET_ID_2: TokenId = 100;
pub const ASSET_CLASS_ID: ClassId = 5;
pub const ASSET_TOKEN_ID: TokenId = 6;
pub const ASSET_COLLECTION_ID: GroupCollectionId = 7;

pub const ALICE_METAVERSE_ID: MetaverseId = 1;
pub const GENERAL_METAVERSE_FUND: AccountId = 102;

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

parameter_types! {
	pub MaximumSchedulerWeight: Weight = 128;
}
impl pallet_scheduler::Config for Runtime {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type MaxScheduledPerBlock = ();
	type WeightInfo = ();
	type PreimageProvider = ();
	type NoPreimagePostponement = ();
}

pub struct MetaverseInfo {}

impl MetaverseTrait<AccountId> for MetaverseInfo {
	fn create_metaverse(who: &AccountId, metadata: MetaverseMetadata) -> MetaverseId {
		1u64
	}

	fn check_ownership(who: &AccountId, country_id: &CountryId) -> bool {
		match *who {
			ALICE => *country_id == ALICE_COUNTRY_ID,
			BOB => *country_id == BOB_COUNTRY_ID,
			_ => false,
		}
	}

	fn get_metaverse(_metaverse_id: u64) -> Option<MetaversePrimitiveInfo<AccountId>> {
		None
	}

	fn get_metaverse_token(_metaverse_id: u64) -> Option<FungibleTokenId> {
		None
	}

	fn update_metaverse_token(_metaverse_id: u64, _currency_id: FungibleTokenId) -> Result<(), DispatchError> {
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

pub struct MetaverseLandInfo {}

impl MetaverseLandTrait<AccountId> for MetaverseLandInfo {
	fn get_user_land_units(_who: &u64, _metaverse_id: &u64) -> Vec<(i32, i32)> {
		Vec::default()
	}

	fn is_user_own_metaverse_land(who: &u64, metaverse_id: &u64) -> bool {
		match *metaverse_id {
			ALICE_COUNTRY_ID => *who == ALICE,
			BOB_COUNTRY_ID => *who == ALICE || *who == BOB,
			_ => false,
		}
	}
	fn check_landunit(metaverse_id: MetaverseId, coordinate: (i32, i32)) -> Result<bool, DispatchError> {
		Ok(false)
	}
}

parameter_types! {
	pub const DefaultVotingPeriod: u32 = 10;
	pub const DefaultEnactmentPeriod: u32 = 2;
	pub const DefaultProposalLaunchPeriod: u32 = 15;
	pub const DefaultMaxParametersPerProposal: u8 = 3;
	pub const DefaultLocalVoteLockingPeriod: u32 = 10;
	pub const DefaultMaxProposalsPerMetaverse: u8 = 20;
	pub const OneBlock: BlockNumber = 1;
	pub const MinimumProposalDeposit: Balance = 50;
	pub const DefaultPreimageByteDeposit: Balance = 1;
}

ord_parameter_types! {
	pub const One: AccountId = 1;
	pub const Two: AccountId = 2;
}

fn test_attributes(x: u8) -> Attributes {
	let mut attr: Attributes = BTreeMap::new();
	attr.insert(vec![x, x + 5], vec![x, x + 10]);
	attr
}

pub struct MockNFTHandler;

impl NFTTrait<AccountId, Balance> for MockNFTHandler {
	type TokenId = TokenId;
	type ClassId = ClassId;

	fn check_ownership(who: &AccountId, asset_id: &(Self::ClassId, Self::TokenId)) -> Result<bool, DispatchError> {
		let nft_value = *asset_id;
		if (*who == ALICE && (nft_value.1 == 1 || nft_value.1 == 3))
			|| (*who == BOB && (nft_value.1 == 2 || nft_value.1 == 4))
			|| (*who == BENEFICIARY_ID && (nft_value.1 == 100 || nft_value.1 == 101))
		{
			return Ok(true);
		}
		Ok(false)
	}

	fn check_collection_and_class(
		collection_id: GroupCollectionId,
		class_id: Self::ClassId,
	) -> Result<bool, DispatchError> {
		if class_id == ASSET_CLASS_ID && collection_id == ASSET_COLLECTION_ID {
			return Ok(true);
		}
		Ok(false)
	}
	fn get_nft_group_collection(nft_collection: &Self::ClassId) -> Result<GroupCollectionId, DispatchError> {
		Ok(ASSET_COLLECTION_ID)
	}

	fn create_token_class(
		sender: &AccountId,
		metadata: NftMetadata,
		attributes: Attributes,
		collection_id: GroupCollectionId,
		token_type: TokenType,
		collection_type: CollectionType,
		royalty_fee: Perbill,
		mint_limit: Option<u32>,
	) -> Result<ClassId, DispatchError> {
		match *sender {
			ALICE => {
				if collection_id == 0 {
					Ok(0)
				} else if collection_id == 1 {
					Ok(1)
				} else {
					Ok(2)
				}
			}
			BOB => Ok(3),
			BENEFICIARY_ID => Ok(ASSET_CLASS_ID),
			_ => Ok(100),
		}
	}

	fn mint_token(
		sender: &AccountId,
		class_id: ClassId,
		metadata: NftMetadata,
		attributes: Attributes,
	) -> Result<TokenId, DispatchError> {
		match *sender {
			ALICE => Ok(1),
			BOB => Ok(2),
			BENEFICIARY_ID => {
				if class_id == 15 {
					return Ok(ASSET_ID_1);
				} else if class_id == 16 {
					return Ok(ASSET_ID_2);
				} else {
					return Ok(200);
				}
			}
			_ => {
				if class_id == 0 {
					return Ok(1000);
				} else {
					return Ok(1001);
				}
			}
		}
	}

	fn transfer_nft(from: &AccountId, to: &AccountId, nft: &(Self::ClassId, Self::TokenId)) -> DispatchResult {
		Ok(())
	}

	fn check_item_on_listing(class_id: Self::ClassId, token_id: Self::TokenId) -> Result<bool, DispatchError> {
		Ok(true)
	}

	fn burn_nft(account: &AccountId, nft: &(Self::ClassId, Self::TokenId)) -> DispatchResult {
		Ok(())
	}
	fn is_transferable(nft: &(Self::ClassId, Self::TokenId)) -> Result<bool, DispatchError> {
		Ok(true)
	}

	fn get_class_fund(class_id: &Self::ClassId) -> AccountId {
		CLASS_FUND_ID
	}

	fn get_nft_detail(asset_id: (Self::ClassId, Self::TokenId)) -> Result<NftClassData<Balance>, DispatchError> {
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

	fn set_lock_collection(class_id: Self::ClassId, is_locked: bool) -> sp_runtime::DispatchResult {
		todo!()
	}

	fn set_lock_nft(token_id: (Self::ClassId, Self::TokenId), is_locked: bool) -> sp_runtime::DispatchResult {
		todo!()
	}
}

parameter_types! {
	pub const MetaverseFundPalletId: PalletId = PalletId(*b"bit/fund");
	pub const MaxTokenMetadata: u32 = 1024;
	pub const MinContribution: Balance = 1;
	pub const MaxNumberOfStakersPerMetaverse: u32 = 512;
}

impl pallet_metaverse::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type MultiCurrency = Currencies;
	type MetaverseTreasury = MetaverseFundPalletId;
	type MaxMetaverseMetadata = MaxTokenMetadata;
	type MinContribution = MinContribution;
	type MetaverseCouncil = EnsureSignedBy<One, AccountId>;
	type MetaverseRegistrationDeposit = MinContribution;
	type MinStakingAmount = MinContribution;
	type MaxNumberOfStakersPerMetaverse = MaxNumberOfStakersPerMetaverse;
	type WeightInfo = ();
	type NFTHandler = MockNFTHandler;
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum ProposalType {
	Any,
	JustTransfer,
}

impl Default for ProposalType {
	fn default() -> Self {
		Self::JustTransfer
	}
}

impl InstanceFilter<Call> for ProposalType {
	fn filter(&self, c: &Call) -> bool {
		match self {
			ProposalType::Any => true,
			ProposalType::JustTransfer => matches!(c, Call::Metaverse(..)),
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		self == &ProposalType::Any || self == o
	}
}

impl Config for Runtime {
	type DefaultVotingPeriod = DefaultVotingPeriod;
	type DefaultEnactmentPeriod = DefaultEnactmentPeriod;
	type DefaultProposalLaunchPeriod = DefaultProposalLaunchPeriod;
	type DefaultMaxProposalsPerMetaverse = DefaultMaxProposalsPerMetaverse;
	type DefaultLocalVoteLockingPeriod = DefaultLocalVoteLockingPeriod;
	type Event = Event;
	type DefaultPreimageByteDeposit = DefaultPreimageByteDeposit;
	type MinimumProposalDeposit = MinimumProposalDeposit;
	type OneBlock = OneBlock;
	type Currency = Balances;
	type Slash = ();
	type MetaverseInfo = MetaverseInfo;
	type PalletsOrigin = OriginCaller;
	type Proposal = Call;
	type Scheduler = Scheduler;
	type MetaverseLandInfo = MetaverseLandInfo;
	type MetaverseCouncil = EnsureSignedBy<One, AccountId>;
	type ProposalType = ProposalType;
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: FungibleTokenId| -> Balance {
		Default::default()
	};
}

parameter_types! {
	pub const MetaverseTreasuryPalletId: PalletId = PalletId(*b"bit/trsy");
	pub TreasuryModuleAccount: AccountId = MetaverseTreasuryPalletId::get().into_account();
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

parameter_types! {
	pub const NativeCurrencyId: FungibleTokenId = FungibleTokenId::NativeToken(0);
	pub const MiningCurrencyId: FungibleTokenId = FungibleTokenId::MiningResource(0);
}

impl currencies::Config for Runtime {
	type Event = Event;
	type MultiSocialCurrency = Tokens;
	type NativeCurrency = AdaptedBasicCurrency;
	type GetNativeCurrencyId = NativeCurrencyId;
}

pub type GovernanceModule = Pallet<Runtime>;

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
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>},
		Governance: governance::{Pallet, Call ,Storage, Event<T>},
		Currencies: currencies::{ Pallet, Storage, Call, Event<T>},
		Tokens: orml_tokens::{Pallet, Call, Storage, Config<T>, Event<T>},
		Metaverse: pallet_metaverse::{Pallet, Call ,Storage, Event<T>}
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
		self.build_with_block_number(1)
	}

	pub fn build_with_block_number(self, block_number: u64) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: vec![(ALICE, 100000), (BOB, 500)],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(block_number));
		ext
	}
}

pub fn last_event() -> Event {
	frame_system::Pallet::<Runtime>::events()
		.pop()
		.expect("Event expected")
		.event
}

fn next_block() {
	GovernanceModule::on_finalize(System::block_number());
	System::set_block_number(System::block_number() + 1);
	GovernanceModule::on_initialize(System::block_number());
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		next_block();
	}
}

pub fn set_balance_proposal(value: u64) -> Vec<u8> {
	Call::Balances(pallet_balances::Call::set_balance {
		who: BOB,
		new_free: value,
		new_reserved: 100,
	})
	.encode()
}

pub fn set_freeze_metaverse_proposal(value: u64) -> Vec<u8> {
	Call::Metaverse(pallet_metaverse::Call::freeze_metaverse { metaverse_id: value }).encode()
}

pub fn set_balance_proposal_hash(value: u64) -> H256 {
	BlakeTwo256::hash(&set_balance_proposal(value)[..])
}

pub fn set_freeze_metaverse_proposal_hash(value: u64) -> H256 {
	BlakeTwo256::hash(&set_freeze_metaverse_proposal(value)[..])
}

pub fn add_preimage(hash: H256) {
	let preimage_status = PreimageStatus::Available {
		data: set_balance_proposal(4),
		provider: ALICE,
		deposit: 200,
		since: 1,
		/// None if it's not imminent.
		expiry: Some(150),
	};
	Preimages::<Runtime>::insert(BOB_COUNTRY_ID, hash, preimage_status);
}

pub fn add_freeze_metaverse_preimage(hash: H256) {
	let preimage_status = PreimageStatus::Available {
		data: set_freeze_metaverse_proposal(1),
		provider: ALICE,
		deposit: 200,
		since: 1,
		/// None if it's not imminent.
		expiry: Some(150),
	};
	Preimages::<Runtime>::insert(BOB_COUNTRY_ID, hash, preimage_status);
}

pub fn add_freeze_metaverse_preimage_alice(hash: H256) {
	let preimage_status = PreimageStatus::Available {
		data: set_freeze_metaverse_proposal(1),
		provider: ALICE,
		deposit: 200,
		since: 1,
		/// None if it's not imminent.
		expiry: Some(150),
	};
	Preimages::<Runtime>::insert(ALICE_COUNTRY_ID, hash, preimage_status);
}

pub fn add_metaverse_preimage(hash: H256) {
	let preimage_status = PreimageStatus::Available {
		data: set_freeze_metaverse_proposal(0),
		provider: ALICE,
		deposit: 200,
		since: 1,
		/// None if it's not imminent.
		expiry: Some(150),
	};
	Preimages::<Runtime>::insert(BOB_COUNTRY_ID, hash, preimage_status);
}

pub fn add_out_of_scope_proposal(preimage_hash: H256) {
	let proposal_info = ProposalInfo {
		proposed_by: ALICE,
		hash: preimage_hash,
		title: PROPOSAL_DESCRIPTION.to_vec(),
		referendum_launch_block: PROPOSAL_BLOCK,
	};
	Proposals::<Runtime>::insert(BOB_COUNTRY_ID, 0, proposal_info);
}
