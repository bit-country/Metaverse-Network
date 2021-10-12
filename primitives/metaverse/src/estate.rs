use crate::{MetaverseId, EstateId};
use sp_runtime::DispatchError;

pub trait Estate<AccountId> {
	fn transfer_estate(estate_id: EstateId, from: &AccountId, to: &AccountId)
		-> Result<EstateId, DispatchError>;

	fn transfer_landunit(coordinate: (i32, i32), from: &AccountId, to: &(AccountId, MetaverseId))
					   -> Result<(i32, i32), DispatchError>;

	fn check_estate(estate_id: EstateId) -> Result<bool, DispatchError>;

	fn check_landunit(metaverse_id: MetaverseId, coordinate: (i32, i32)) -> Result<bool, DispatchError>;
}
