use codec::{Decode, Encode};
use scale_info::{prelude::vec::Vec, TypeInfo};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::DispatchError;
use sp_runtime::{Perbill, RuntimeDebug};

use crate::UndeployedLandBlockId;
use crate::{EstateId, MetaverseId, TokenId};

pub trait Estate<AccountId> {
	fn transfer_estate(estate_id: EstateId, from: &AccountId, to: &AccountId) -> Result<EstateId, DispatchError>;

	fn transfer_landunit(
		coordinate: (i32, i32),
		from: &AccountId,
		to: &(AccountId, MetaverseId),
	) -> Result<(i32, i32), DispatchError>;

	fn transfer_undeployed_land_block(
		who: &AccountId,
		to: &AccountId,
		undeployed_land_block_id: UndeployedLandBlockId,
	) -> Result<UndeployedLandBlockId, DispatchError>;

	fn check_estate(estate_id: EstateId) -> Result<bool, DispatchError>;

	fn check_landunit(metaverse_id: MetaverseId, coordinate: (i32, i32)) -> Result<bool, DispatchError>;

	fn check_undeployed_land_block(
		owner: &AccountId,
		undeployed_land_block: UndeployedLandBlockId,
	) -> Result<bool, DispatchError>;

	fn get_total_land_units() -> u64;

	fn get_total_undeploy_land_units() -> u64;
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct EstateInfo {
	/// Metaverse Ids
	pub metaverse_id: MetaverseId,
	/// Land Units
	pub land_units: Vec<(i32, i32)>,
}

#[derive(Eq, PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum OwnerId<AccountId, TokenId> {
	Account(AccountId),
	Token(TokenId),
}

#[derive(Eq, PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum LandUnitStatus<AccountId> {
	NonExisting,
	Existing(AccountId),
}
