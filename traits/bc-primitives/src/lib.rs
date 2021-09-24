#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use primitives::{Balance, CurrencyId, FungibleTokenId, MetaverseId};
use sp_runtime::{
	traits::{AtLeast32Bit, MaybeSerializeDeserialize},
	DispatchError, DispatchResult, RuntimeDebug,
};
use sp_std::vec::Vec;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct MetaverseAssetData {
	pub image: Vec<u8>,
}

pub type Metadata = Vec<u8>;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct MetaverseInfo<AccountId> {
	pub owner: AccountId,
	pub metadata: Metadata,
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
