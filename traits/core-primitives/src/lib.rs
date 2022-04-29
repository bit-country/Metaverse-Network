#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{DispatchError, DispatchResult, Perbill, RuntimeDebug};
use sp_std::{collections::btree_map::BTreeMap, prelude::*, vec::Vec};

use primitives::staking::RoundInfo;
use primitives::{
	AssetId, ClassId, FungibleTokenId, GroupCollectionId, ItemId, MetaverseId, TokenId, UndeployedLandBlockId,
	UndeployedLandBlockType,
};

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum TokenType {
	Transferable,
	BoundToAddress,
}

impl TokenType {
	pub fn is_transferable(&self) -> bool {
		match *self {
			TokenType::Transferable => true,
			_ => false,
		}
	}
}

impl Default for TokenType {
	fn default() -> Self {
		TokenType::Transferable
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CollectionType {
	Collectable,
	Wearable,
	Executable(Vec<u8>),
}

// Collection extension for fast retrieval
impl CollectionType {
	pub fn is_collectable(&self) -> bool {
		match *self {
			CollectionType::Collectable => true,
			_ => false,
		}
	}

	pub fn is_executable(&self) -> bool {
		match *self {
			CollectionType::Executable(_) => true,
			_ => false,
		}
	}

	pub fn is_wearable(&self) -> bool {
		match *self {
			CollectionType::Wearable => true,
			_ => false,
		}
	}
}

impl Default for CollectionType {
	fn default() -> Self {
		CollectionType::Collectable
	}
}

pub type NftMetadata = Vec<u8>;
pub type Attributes = BTreeMap<Vec<u8>, Vec<u8>>;

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub struct NftGroupCollectionData {
	pub name: NftMetadata,
	// Metadata from ipfs
	pub properties: NftMetadata,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct NftClassData<Balance> {
	// Minimum balance to create a collection of Asset
	pub deposit: Balance,
	pub attributes: Attributes,
	pub token_type: TokenType,
	pub collection_type: CollectionType,
	pub is_locked: bool,
	pub royalty_fee: Perbill,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct NftClassDataV1<Balance> {
	// Minimum balance to create a collection of Asset
	pub deposit: Balance,
	pub attributes: Attributes,
	pub token_type: TokenType,
	pub collection_type: CollectionType,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct NftAssetData<Balance> {
	// Deposit balance to create each token
	pub deposit: Balance,
	pub attributes: Attributes,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct MetaverseAssetData {
	pub image: Vec<u8>,
}

pub type MetaverseMetadata = Vec<u8>;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct MetaverseInfo<AccountId> {
	/// The owner of this metaverse
	pub owner: AccountId,
	/// The metadata of this metaverse
	pub metadata: MetaverseMetadata,
	/// The currency use in this metaverse
	pub currency_id: FungibleTokenId,
	/// Whether the metaverse can be transferred or not.
	pub is_frozen: bool,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct MetaverseFund<AccountId, Balance> {
	/// The fund account of this metaverse
	pub vault: AccountId,
	/// The fund balance of this metaverse
	pub value: Balance,
	/// The amount of native token backing of this metaverse
	pub backing: Balance,
	/// The currency use in this fund
	pub currency_id: FungibleTokenId,
}

pub trait MetaverseTrait<AccountId> {
	/// Check the ownership of this metaverse
	fn check_ownership(who: &AccountId, metaverse_id: &MetaverseId) -> bool;
	/// Get the detail of this metaverse
	fn get_metaverse(metaverse_id: MetaverseId) -> Option<MetaverseInfo<AccountId>>;
	/// Get metaverse token detail
	fn get_metaverse_token(metaverse_id: MetaverseId) -> Option<FungibleTokenId>;
	/// Update metaverse token, this only use once per metaverse
	fn update_metaverse_token(metaverse_id: MetaverseId, currency_id: FungibleTokenId) -> Result<(), DispatchError>;
	/// Get the land class for a specific metaverse
	fn get_metaverse_land_class(metaverse_id: MetaverseId) -> ClassId;
	/// Get the estate class for a specific metaverse
	fn get_metaverse_estate_class(metaverse_id: MetaverseId) -> ClassId;
	/// Get metaverse marketplace listing fee
	fn get_metaverse_marketplace_listing_fee(metaverse_id: MetaverseId) -> Perbill;
}

pub trait MetaverseLandTrait<AccountId> {
	/// Get Land units owned by account
	fn get_user_land_units(who: &AccountId, metaverse_id: &MetaverseId) -> Vec<(i32, i32)>;
	/// Check if this user own the metaverse
	fn is_user_own_metaverse_land(who: &AccountId, metaverse_id: &MetaverseId) -> bool;
}

pub trait UndeployedLandBlocksTrait<AccountId> {
	fn issue_undeployed_land_blocks(
		beneficiary: &AccountId,
		number_of_land_block: u32,
		number_land_units_per_land_block: u32,
		undeployed_land_block_type: UndeployedLandBlockType,
	) -> Result<Vec<UndeployedLandBlockId>, DispatchError>;

	fn transfer_undeployed_land_block(
		who: &AccountId,
		to: &AccountId,
		undeployed_land_block_id: UndeployedLandBlockId,
	) -> Result<UndeployedLandBlockId, DispatchError>;

	fn burn_undeployed_land_block(
		undeployed_land_block_id: UndeployedLandBlockId,
	) -> Result<UndeployedLandBlockId, DispatchError>;

	fn freeze_undeployed_land_block(
		undeployed_land_block_id: UndeployedLandBlockId,
	) -> Result<UndeployedLandBlockId, DispatchError>;
}

pub trait NFTTrait<AccountId, Balance> {
	/// Token identifier
	type TokenId;
	/// Token class identifier
	type ClassId;
	/// Check the ownership of this nft asset
	fn check_ownership(who: &AccountId, asset_id: &(Self::ClassId, Self::TokenId)) -> Result<bool, DispatchError>;
	/// Check the ownership of this nft tuple
	fn check_nft_ownership(who: &AccountId, nft: &(Self::ClassId, Self::TokenId)) -> Result<bool, DispatchError>;
	/// Get the detail of this nft
	fn get_nft_detail(asset_id: (Self::ClassId, Self::TokenId)) -> Result<NftClassData<Balance>, DispatchError>;
	/// Get the detail of this nft
	fn get_nft_group_collection(nft_collection: &Self::ClassId) -> Result<GroupCollectionId, DispatchError>;
	/// Check if collection and class exist
	fn check_collection_and_class(
		collection_id: GroupCollectionId,
		class_id: Self::ClassId,
	) -> Result<bool, DispatchError>;
	/// Create NFT token class
	fn create_token_class(
		sender: &AccountId,
		metadata: NftMetadata,
		attributes: Attributes,
		collection_id: GroupCollectionId,
		token_type: TokenType,
		collection_type: CollectionType,
		royalty_fee: Perbill,
	) -> Result<ClassId, DispatchError>;
	/// Mint NFT token
	fn mint_token(
		sender: &AccountId,
		class_id: ClassId,
		metadata: NftMetadata,
		attributes: Attributes,
	) -> Result<TokenId, DispatchError>;
	/// Burn nft
	fn burn_nft(account: &AccountId, nft: &(Self::ClassId, Self::TokenId)) -> DispatchResult;
	/// Check if item is on listing
	fn check_item_on_listing(class_id: Self::ClassId, token_id: Self::TokenId) -> Result<bool, DispatchError>;
	/// transfer nft
	fn transfer_nft(sender: &AccountId, to: &AccountId, nft: &(Self::ClassId, Self::TokenId)) -> DispatchResult;
	/// Is Nft transferable
	fn is_transferable(nft: &(Self::ClassId, Self::TokenId)) -> Result<bool, DispatchError>;
	/// Get collection account fund
	fn get_class_fund(class_id: &Self::ClassId) -> AccountId;
}

pub trait RoundTrait<BlockNumber> {
	fn get_current_round_info() -> RoundInfo<BlockNumber>;
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Copy, Encode, Decode, Default, RuntimeDebug, TypeInfo)]
pub struct MiningRange<T> {
	pub min: T,
	pub ideal: T,
	pub max: T,
	pub staking_allocation: T,
	pub mining_allocation: T,
}

impl<T: Ord> MiningRange<T> {
	pub fn is_valid(&self) -> bool {
		self.max >= self.ideal && self.ideal >= self.min
	}
}

impl<T: Ord + Copy> From<T> for MiningRange<T> {
	fn from(other: T) -> MiningRange<T> {
		MiningRange {
			min: other,
			ideal: other,
			max: other,
			staking_allocation: other,
			mining_allocation: other,
		}
	}
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, Default, RuntimeDebug, TypeInfo)]
pub struct MiningResourceRateInfo {
	/// annual inflation rate
	pub rate: Perbill,
	/// land staking reward percentage (4 decimals)
	pub staking_reward: Perbill,
	/// metaverse staking reward percentage (4 decimals)
	pub mining_reward: Perbill,
}

impl MiningResourceRateInfo {
	pub fn new(rate: Perbill, staking_reward: Perbill, mining_reward: Perbill) -> MiningResourceRateInfo {
		MiningResourceRateInfo {
			rate,
			staking_reward,
			mining_reward,
		}
	}

	/// kBIT and Land unit ratio
	pub fn set_rate(&mut self, rate: Perbill) {
		self.rate = rate;
	}

	/// Set staking reward
	pub fn set_staking_reward(&mut self, staking_reward: Perbill) {
		self.staking_reward = staking_reward;
	}

	/// Set mining reward
	pub fn set_mining_reward(&mut self, mining_reward: Perbill) {
		self.mining_reward = mining_reward;
	}
}
