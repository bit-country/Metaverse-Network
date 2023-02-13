use sp_runtime::DispatchError;

use crate::{MapSpotId, MetaverseId};

pub trait MapTrait<AccountId> {
    fn transfer_spot(
        spot_id: MapSpotId,
        from: AccountId,
        to: (AccountId, MetaverseId),
    ) -> Result<MapSpotId, DispatchError>;
}
