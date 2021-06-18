#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use sp_runtime::{
    traits::{AtLeast32Bit, MaybeSerializeDeserialize},
    DispatchError, DispatchResult, RuntimeDebug,
};
use sp_std::vec::Vec;
use primitives::{Balance, CountryId, CurrencyId};

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct CountryAssetData {
    pub image: Vec<u8>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Country<AccountId> {
    pub owner: AccountId,
    pub metadata: Vec<u8>,
    pub currency_id: CurrencyId,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct CountryFund<AccountId, Balance> {
    pub vault: AccountId,
    pub value: Balance,
    pub backing: Balance,
    pub currency_id: CurrencyId,
}

pub trait BCCountry<AccountId> {
    fn check_ownership(who: &AccountId, country_id: &CountryId) -> bool;

    fn get_country(country_id: CountryId) -> Option<Country<AccountId>>;    

    fn get_country_token(country_id: CountryId) -> Option<CurrencyId>;

    fn is_member(who: &AccountId, country_id: &CountryId) -> bool;
}
