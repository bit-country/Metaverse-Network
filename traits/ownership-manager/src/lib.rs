#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use sp_runtime::{
    traits::{AtLeast32Bit, MaybeSerializeDeserialize},
    DispatchError, DispatchResult, RuntimeDebug,
};
use sp_std::vec::Vec;
use primitives::{AssetId, Balance, CountryId, CurrencyId, SocialTokenCurrencyId};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum TokenType {
    Transferable,
    BoundToAddress,
    Ownership(CountryId),
}

impl TokenType {
    pub fn is_transferable(&self) -> bool {
        match *self {
            TokenType::Transferable => true,
            TokenType::Ownership(country_id) => true,
            _ => false,
        }
    }
}

impl Default for TokenType {
    fn default() -> Self {
        TokenType::Transferable
    }
}

pub trait OwnershipTokenManager<AccountId> {
    fn mint_ownership_token(owner: &AccountId, country_id: &CountryId) -> Result<AssetId, DispatchError>;

    fn burn_ownership_token(owner: &AccountId, asset_id: &AssetId) -> DispatchResult;

    fn is_token_owner(who: &AccountId, asset_id: &AssetId) -> bool;
}
