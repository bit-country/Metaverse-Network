#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use primitives::{FungibleTokenId, MetaverseId};
use sp_runtime::{DispatchError, RuntimeDebug};
use sp_std::vec::Vec;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct MetaverseAssetData {
	pub image: Vec<u8>,
}

pub type MetaverseMetadata = Vec<u8>;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct MetaverseInfo<AccountId> {
	pub owner: AccountId,
	pub metadata: MetaverseMetadata,
	pub currency_id: FungibleTokenId,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct MetaverseFund<AccountId, Balance> {
	pub vault: AccountId,
	pub value: Balance,
	pub backing: Balance,
	pub currency_id: FungibleTokenId,
}

pub trait MetaverseTrait<AccountId> {
	fn check_ownership(who: &AccountId, metaverse_id: &MetaverseId) -> bool;

	fn get_metaverse(metaverse_id: MetaverseId) -> Option<MetaverseInfo<AccountId>>;

	fn get_metaverse_token(metaverse_id: MetaverseId) -> Option<FungibleTokenId>;

	fn update_metaverse_token(metaverse_id: MetaverseId, currency_id: FungibleTokenId) -> Result<(), DispatchError>;
}

pub trait MetaverseLandTrait<AccountId> {
	fn get_user_land_units(who: &AccountId, metaverse_id: &MetaverseId) -> Vec<(i32, i32)>;

	fn is_user_own_metaverse_land(who: &AccountId, metaverse_id: &MetaverseId) -> bool;
}
