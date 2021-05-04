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

use codec::{Decode, Encode};
use sp_runtime::{
    generic,
    traits::{BlakeTwo256, IdentifyAccount, Verify},
    MultiSignature, RuntimeDebug,
};

use sp_std::{
    convert::{Into, TryFrom, TryInto},
    prelude::*,
};

use xcm::v0::{Junction, NetworkId, MultiLocation};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

/// Opaque block header type.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Opaque block type.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// Opaque block identifier type.
pub type BlockId = generic::BlockId<Block>;
/// An index to a block.
pub type BlockNumber = u32;
/// Alias to 512-bit hash when used in the context of a transaction signature on
/// the chain.
pub type Signature = MultiSignature;
/// Alias to the public key used for this chain, actually a `MultiSigner`. Like
/// the signature, this also isn't a fixed size when encoded, as different
/// cryptos have different size public keys.
pub type AccountPublic = <Signature as Verify>::Signer;
/// Alias to the opaque account ID type for this chain, actually a
/// `AccountId32`. This is always 32 bytes.
pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;
/// Balance of an account.
pub type Balance = u128;
/// Country Id
pub type CountryId = u64;
/// Country Currency Id
pub type CountryCurrencyId = u64;
// Collection Type Id
pub type CollectionId = u64;
// Amount type
pub type Amount = i128;
/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;
/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;
/// An instant or duration in time.
pub type Moment = u64;
/// Counter for the number of eras that have passed.
pub type EraIndex = u32;
/// Auction ID
pub type AuctionId = u32;
pub type Index = u32;
/// Group collection id type
pub type GroupCollectionId = u64;
/// AssetId for all NFT and FT
pub type AssetId = u64;
/// SpotId
pub type SpotId = u64;

/// Public item id for auction
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ItemId {
    NFT(AssetId),
    Spot(u64, CountryId),
    Country(CountryId),
    Block(u64),
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum TokenSymbol {
    NUUM = 0,
    AUSD = 1,
    ACA = 2,
    DOT = 3,
}

impl TryFrom<u8> for TokenSymbol {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(TokenSymbol::NUUM),
            1 => Ok(TokenSymbol::AUSD),
            2 => Ok(TokenSymbol::ACA),
            3 => Ok(TokenSymbol::DOT),
            _ => Err(()),
        }
    }
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CurrencyId {
    NUUM = 0,
    AUSD,
    ACA,
    DOT,
    LAMI,
}

impl TryFrom<Vec<u8>> for CurrencyId {
    type Error = ();
    fn try_from(v: Vec<u8>) -> Result<CurrencyId, ()> {
        match v.as_slice() {
            b"NUUM" => Ok(CurrencyId::NUUM),
            b"AUSD" => Ok(CurrencyId::AUSD),
            b"ACA" => Ok(CurrencyId::ACA),
            b"DOT" => Ok(CurrencyId::DOT),
            _ => Err(()),
        }
    }
}

impl From<CurrencyId> for MultiLocation {
    fn from(id: CurrencyId) -> Self {
        match id {
            CurrencyId::NUUM => Junction::Parent.into(),
            CurrencyId::AUSD => (
                Junction::Parent,
                Junction::Parachain { id: 666 },
                Junction::GeneralKey("AUSD".into()),
            )
                .into(),
            _ => Junction::Parent.into()
        }
    }
}
