#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::upper_case_acronyms)]

use crate::FungibleTokenId;
use codec::{Decode, Encode};
use sp_runtime::FixedU128;
use sp_runtime::RuntimeDebug;

pub type Price = FixedU128;
pub type ExchangeRate = FixedU128;
pub type Ratio = FixedU128;
pub type Rate = FixedU128;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct TradingPair(pub FungibleTokenId, pub FungibleTokenId);

impl TradingPair {
    pub fn new(currency_id_a: FungibleTokenId, currency_id_b: FungibleTokenId) -> Self {
        if currency_id_a > currency_id_b {
            TradingPair(currency_id_b, currency_id_a)
        } else {
            TradingPair(currency_id_a, currency_id_b)
        }
    }

    pub fn from_token_currency_ids(
        currency_id_0: FungibleTokenId,
        currency_id_1: FungibleTokenId,
    ) -> Option<Self> {
        if currency_id_0.is_native_token_currency_id()
            && currency_id_1.is_social_token_currency_id()
        {
            Some(TradingPair(currency_id_0, currency_id_1))
        } else if currency_id_0.is_social_token_currency_id()
            && currency_id_1.is_native_token_currency_id()
        {
            Some(TradingPair(currency_id_1, currency_id_0))
        } else {
            None
        }
    }

    pub fn get_dex_share_social_currency_id(&self) -> Option<FungibleTokenId> {
        FungibleTokenId::join_dex_share_social_currency_id(self.0, self.1)
    }
}
