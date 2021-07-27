#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use sp_runtime::{
    traits::{AtLeast32Bit, MaybeSerializeDeserialize},
    DispatchError, DispatchResult, RuntimeDebug,
};
use primitives::{CountryId, LandId};
use sp_std::vec::Vec;

pub trait BCLand<AccountId,CountryId> {
    fn get_owner_lands(who: &AccountId) -> Vec<LandId>;

    fn get_lands_in_country(country_id: &CountryId) -> Vec<LandId>;
}