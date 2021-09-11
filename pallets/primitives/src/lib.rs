// This file is part of Bit.Country.

// Copyright (C) 2020-2021 Bit.Country.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]


use sp_runtime::{
    generic, traits::{Verify, BlakeTwo256, IdentifyAccount}, OpaqueExtrinsic, MultiSignature,
};
use codec::{Decode, Encode, HasCompact};
use sp_runtime::RuntimeDebug;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::traits::AtLeast32Bit;

pub mod continuum;
pub mod dex;

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them.
pub type AccountIndex = u32;

/// Balance of an account.
pub type Balance = u128;

/// Type used for expressing timestamp.
pub type Moment = u64;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// A timestamp: milliseconds since the unix epoch.
/// `u64` is enough to represent a duration of half a billion years, when the
/// time scale is milliseconds.
pub type Timestamp = u64;
/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;
/// Header type.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type.
pub type Block = generic::Block<Header, OpaqueExtrinsic>;
/// Block ID.
pub type BlockId = generic::BlockId<Block>;
/// Country Id
pub type BitCountryId = u64;
/// Amount for transaction type
pub type Amount = i128;
/// Currency Id type
pub type CurrencyId = u32;
/// Group collection id type
pub type GroupCollectionId = u64;
/// AssetId for all NFT and FT
pub type AssetId = u64;
/// AuctionId
pub type AuctionId = u64;
/// SpotId
pub type SpotId = u64;
/// LandId
pub type LandId = u64;
/// Social Token Id type
pub type TokenId = u64;

/// Public item id for auction
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ItemId {
    NFT(AssetId),
    Spot(u64, BitCountryId),
    Country(BitCountryId),
    Block(u64),
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum FungibleTokenId {
    NativeToken(TokenId),
    SocialToken(TokenId),
    DEXShare(TokenId, TokenId),
    MiningResource(TokenId),
}

impl FungibleTokenId {
    pub fn is_native_token_currency_id(&self) -> bool {
        matches!(self, FungibleTokenId::NativeToken(_))
    }

    pub fn is_social_token_currency_id(&self) -> bool {
        matches!(self, FungibleTokenId::SocialToken(_))
    }

    pub fn is_dex_share_social_token_currency_id(&self) -> bool {
        matches!(self, FungibleTokenId::DEXShare(_, _))
    }

    pub fn is_mining_resource_currency(&self) -> bool {
        matches!(self, FungibleTokenId::MiningResource(_))
    }

    pub fn split_dex_share_social_token_currency_id(&self) -> Option<(Self, Self)> {
        match self {
            FungibleTokenId::DEXShare(token_currency_id_0, token_currency_id_1) => {
                Some((FungibleTokenId::NativeToken(*token_currency_id_0), FungibleTokenId::SocialToken(*token_currency_id_1)))
            }
            _ => None,
        }
    }

    pub fn join_dex_share_social_currency_id(currency_id_0: Self, currency_id_1: Self) -> Option<Self> {
        match (currency_id_0, currency_id_1) {
            (FungibleTokenId::NativeToken(token_currency_id_0), FungibleTokenId::SocialToken(token_currency_id_1)) => {
                Some(FungibleTokenId::DEXShare(token_currency_id_0, token_currency_id_1))
            }
            (FungibleTokenId::SocialToken(token_currency_id_0), FungibleTokenId::NativeToken(token_currency_id_1)) => {
                Some(FungibleTokenId::DEXShare(token_currency_id_1, token_currency_id_0))
            }
            _ => None,
        }
    }
}

/// App-specific crypto used for reporting equivocation/misbehavior in BABE and
/// GRANDPA. Any rewards for misbehavior reporting will be paid out to this
/// account.
pub mod report {
    use super::{Signature, Verify};
    use frame_system::offchain::AppCrypto;
    use sp_core::crypto::{key_types, KeyTypeId};

    /// Key type for the reporting module. Used for reporting BABE and GRANDPA
    /// equivocations.
    pub const KEY_TYPE: KeyTypeId = key_types::REPORTING;

    mod app {
        use sp_application_crypto::{app_crypto, sr25519};
        app_crypto!(sr25519, super::KEY_TYPE);
    }

    /// Identity of the equivocation/misbehavior reporter.
    pub type ReporterId = app::Public;

    /// An `AppCrypto` type to allow submitting signed transactions using the reporting
    /// application key as signer.
    pub struct ReporterAppCrypto;

    impl AppCrypto<<Signature as Verify>::Signer, Signature> for ReporterAppCrypto {
        type RuntimeAppPublic = ReporterId;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }
}

/// The vesting schedule.
///
/// Benefits would be granted gradually, `per_period` amount every `period`
/// of blocks after `start`.
#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug)]
pub struct VestingSchedule<BlockNumber, Balance: HasCompact> {
    /// Vesting token
    pub token: FungibleTokenId,
    /// Vesting starting block
    pub start: BlockNumber,
    /// Number of blocks between vest
    pub period: BlockNumber,
    /// Number of vest
    pub period_count: u32,
    /// Amount of tokens to release per vest
    #[codec(compact)]
    pub per_period: Balance,
}

impl<BlockNumber: AtLeast32Bit + Copy, Balance: AtLeast32Bit + Copy> VestingSchedule<BlockNumber, Balance> {
    /// Returns the end of all periods, `None` if calculation overflows.
    pub fn end(&self) -> Option<BlockNumber> {
        // period * period_count + start
        self.period
            .checked_mul(&self.period_count.into())?
            .checked_add(&self.start)
    }

    /// Returns all locked amount, `None` if calculation overflows.
    pub fn total_amount(&self) -> Option<Balance> {
        self.per_period.checked_mul(&self.period_count.into())
    }

    /// Returns locked amount for a given `time`.
    ///
    /// Note this func assumes schedule is a valid one(non-zero period and
    /// non-overflow total amount), and it should be guaranteed by callers.
    pub fn locked_amount(&self, time: BlockNumber) -> Balance {
        // full = (time - start) / period
        // unrealized = period_count - full
        // per_period * unrealized
        let full = time
            .saturating_sub(self.start)
            .checked_div(&self.period)
            .expect("ensured non-zero period; qed");
        let unrealized = self.period_count.saturating_sub(full.unique_saturated_into());
        self.per_period
            .checked_mul(&unrealized.into())
            .expect("ensured non-overflow total amount; qed")
    }
}

