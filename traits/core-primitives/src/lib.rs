#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{DispatchError, DispatchResult, RuntimeDebug};
use sp_std::vec::Vec;

use primitives::staking::RoundInfo;
use primitives::{
	AssetId, FungibleTokenId, GroupCollectionId, MetaverseId, UndeployedLandBlockId, UndeployedLandBlockType,
};

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

pub trait NFTTrait<AccountId> {
	/// Token identifier
	type TokenId;
	/// Token class identifier
	type ClassId;
	/// Check the ownership of this nft asset
	fn check_ownership(who: &AccountId, asset_id: &AssetId) -> Result<bool, DispatchError>;
	/// Get the detail of this nft
	fn get_nft_detail(asset_id: AssetId) -> Result<(GroupCollectionId, Self::ClassId, Self::TokenId), DispatchError>;
	/// Check if collection and class exist
	fn check_collection_and_class(
		collection_id: GroupCollectionId,
		class_id: Self::ClassId,
	) -> Result<bool, DispatchError>;
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
	/// kBIT and Land unit ratio
	pub ratio: u64,
	/// land staking reward percentage (4 decimals)
	pub staking_reward: u32,
	/// metaverse staking reward percentage (4 decimals)
	pub mining_reward: u32,
}

impl MiningResourceRateInfo {
	pub fn new(ratio: u64, staking_reward: u32, mining_reward: u32) -> MiningResourceRateInfo {
		MiningResourceRateInfo {
			ratio,
			staking_reward,
			mining_reward,
		}
	}

	/// kBIT and Land unit ratio
	pub fn set_ratio(&mut self, ratio: u64) {
		self.ratio = ratio;
	}

	/// Set staking reward
	pub fn set_staking_reward(&mut self, staking_reward: u32) {
		self.staking_reward = staking_reward;
	}

	/// Set mining reward
	pub fn set_mining_reward(&mut self, mining_reward: u32) {
		self.mining_reward = mining_reward;
	}
}
