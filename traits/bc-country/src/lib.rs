#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use sp_runtime::{
    traits::{AtLeast32Bit, MaybeSerializeDeserialize},
    DispatchError, DispatchResult, RuntimeDebug,
};
use sp_std::vec::Vec;
use primitives::{AccountId, AssetId, Balance, CountryId, CurrencyId, SocialTokenCurrencyId};

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct CountryAssetData {
    pub image: Vec<u8>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Country<AccountId> {
    pub ownership_id: OwnershipId<AccountId>,
    pub metadata: Vec<u8>,
    pub currency_id: SocialTokenCurrencyId,
}

impl<AccountId> Country<AccountId> {
    pub fn is_tokenized(&self) -> bool {
        if let OwnershipId::Token(_) = self.ownership_id {
            true 
        } else {
            false
        }
    }
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct CountryFund<AccountId, Balance> {
    pub vault: AccountId,
    pub value: Balance,
    pub backing: Balance,
    pub currency_id: SocialTokenCurrencyId,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum OwnershipId<AccountId> {
    Standard(AccountId),
    Token(AssetId),
}

pub trait BCCountry<AccountId> {
    fn check_ownership(owner: &AccountId, country_id: &CountryId) -> bool;

    fn check_ownership_given_country(owner: &AccountId, country: &Country<AccountId>) -> bool;

    fn get_country(country_id: CountryId) -> Option<Country<AccountId>>;

    fn get_country_token(country_id: CountryId) -> Option<SocialTokenCurrencyId>;

    fn update_country_token(country_id: CountryId, currency_id: SocialTokenCurrencyId) -> DispatchResult;

    fn transfer_ownership(from: &AccountId, to: &AccountId, country_id: CountryId) -> DispatchResult;
}
