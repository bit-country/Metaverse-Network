#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::upper_case_acronyms)]

use crate::SocialTokenCurrencyId;

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct TradingPair(pub SocialTokenCurrencyId, pub SocialTokenCurrencyId);

impl TradingPair {
    pub fn new(currency_id_a: SocialTokenCurrencyId, currency_id_b: SocialTokenCurrencyId) -> Self {
        if currency_id_a > currency_id_b {
            TradingPair(currency_id_b, currency_id_a)
        } else {
            TradingPair(currency_id_a, currency_id_b)
        }
    }

    pub fn from_token_currency_ids(currency_id_0: SocialTokenCurrencyId, currency_id_1: SocialTokenCurrencyId) -> Option<Self> {
        match currency_id_0.is_social_token_currency_id() && currency_id_1.is_social_token_currency_id() {
            true if currency_id_0 > currency_id_1 => Some(TradingPair(currency_id_1, currency_id_0)),
            true if currency_id_0 < currency_id_1 => Some(TradingPair(currency_id_0, currency_id_1)),
            _ => None,
        }
    }

    pub fn get_dex_share_currency_id(&self) -> Option<SocialTokenCurrencyId> {
        SocialTokenCurrencyId::join_dex_share_social_currency_id(self.0, self.1)
    }
}
