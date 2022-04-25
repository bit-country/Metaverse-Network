use crate::{MetaverseId, SpotId};
use sp_runtime::DispatchError;

pub trait Continuum<AccountId> {
	fn transfer_spot(spot_id: SpotId, from: &AccountId, to: &(AccountId, MetaverseId))
		-> Result<SpotId, DispatchError>;
}
