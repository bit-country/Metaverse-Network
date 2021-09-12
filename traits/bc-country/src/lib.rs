#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use sp_runtime::{
    traits::{AtLeast32Bit, MaybeSerializeDeserialize},
    DispatchError, DispatchResult, RuntimeDebug,
};
use sp_std::vec::Vec;
use primitives::{Balance, BitCountryId, CurrencyId, FungibleTokenId};

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct BitCountryAssetData {
    pub image: Vec<u8>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct BitCountryStruct<AccountId> {
    pub owner: AccountId,
    pub metadata: Vec<u8>,
    pub currency_id: FungibleTokenId,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct BitCountryFund<AccountId, Balance> {
    pub vault: AccountId,
    pub value: Balance,
    pub backing: Balance,
    pub currency_id: FungibleTokenId,
}

pub trait BitCountryTrait<AccountId> {
    fn check_ownership(who: &AccountId, bitcountry_id: &BitCountryId) -> bool;

    fn get_bitcountry(bitcountry_id: BitCountryId) -> Option<BitCountryStruct<AccountId>>;

    fn get_bitcountry_token(bitcountry_id: BitCountryId) -> Option<FungibleTokenId>;

    fn update_bitcountry_token(bitcountry_id: BitCountryId, currency_id: FungibleTokenId) -> Result<(), DispatchError>;
}
