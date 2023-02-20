use codec::{Decode, Encode};
use scale_info::{prelude::vec::Vec, TypeInfo};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::DispatchError;
use sp_runtime::RuntimeDebug;

use crate::UndeployedLandBlockId;
use crate::{EstateId, MetaverseId};

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

	fn get_total_land_units(estate_id: Option<EstateId>) -> u64;

	fn get_total_undeploy_land_units() -> u64;

	fn check_estate_ownership(owner: AccountId, estate_id: EstateId) -> Result<bool, DispatchError>;

	fn is_estate_leasor(leasor: AccountId, estate_id: EstateId) -> Result<bool, DispatchError>;

	fn is_estate_leased(estate_id: EstateId) -> Result<bool, DispatchError>;
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
pub enum OwnerId<AccountId, ClassId, TokenId> {
	Account(AccountId),
	Token(ClassId, TokenId),
}

#[derive(Eq, PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum LandUnitStatus<AccountId> {
	NonExisting,
	Existing(AccountId),
	NonExistingWithEstate,
	RemovedFromEstate,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct LeaseContract<Balance, BlockNumber> {
	/// Price per block
	pub price_per_block: Balance,
	/// Lease duration (in number of blocks)
	pub duration: u32,
	/// Lease end block (equivalent to expiry block if offer is not accepted)
	pub end_block: BlockNumber,
	/// Lease start block (equal to end block + 1 if the offer is not accepted)
	pub start_block: BlockNumber,
	/// Unclaimed rent balance
	pub unclaimed_rent: Balance,
}
