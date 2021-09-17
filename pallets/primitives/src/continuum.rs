use crate::{BitCountryId, SpotId};
use sp_runtime::DispatchError;

pub trait Continuum<AccountId> {
	fn transfer_spot(
		spot_id: SpotId,
		from: &AccountId,
		to: &(AccountId, BitCountryId),
	) -> Result<SpotId, DispatchError>;
}
