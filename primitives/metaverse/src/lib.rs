// This file is part of Metaverse.Network & Bit.Country.

// Copyright (C) 2020-2022 Metaverse.Network & Bit.Country .
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

use codec::{Decode, Encode, HasCompact, MaxEncodedLen};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::traits::AtLeast32Bit;
use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
use sp_runtime::RuntimeDebug;
use sp_runtime::{
	generic,
	traits::{BlakeTwo256, IdentifyAccount, Verify},
	MultiSignature,
};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::vec::Vec;

pub mod continuum;
pub mod dex;
pub mod estate;
pub mod staking;

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
pub type DigestItem = generic::DigestItem;
/// Header type.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// Block ID.
pub type BlockId = generic::BlockId<Block>;
/// Country Id
pub type MetaverseId = u64;
/// Amount for transaction type
pub type Amount = i128;
/// Currency Id type
pub type CurrencyId = u32;
/// Group collection id type
pub type GroupCollectionId = u64;
/// AssetId for all NFT and FT
pub type AssetId = u64;
/// Collection Id of NFT
pub type ClassId = u32;
/// Nft Id of NFT
pub type NftId = u64;
/// AuctionId
pub type AuctionId = u64;
/// SpotId
pub type SpotId = u64;
/// MapSpotId
pub type MapSpotId = (i32, i32);
/// ProposalId
pub type ProposalId = u64;
/// ReferendumId
pub type ReferendumId = u64;
/// LandId
pub type LandId = u64;
/// EstateId
pub type EstateId = u64;
/// Social Token Id type
pub type TokenId = u64;
/// Undeployed LandBlock Id type
pub type UndeployedLandBlockId = u128;
/// Staking Round index
pub type RoundIndex = u32;
/// Domain Id
pub type DomainId = u32;
/// Element Id
pub type ElementId = u32;
/// Mining Power Amount
pub type PowerAmount = u64;
/// Nonce
pub type Nonce = u32;
/// Evm Address.
pub type EvmAddress = sp_core::H160;
/// NFT Metadata
pub type NftMetadata = Vec<u8>;
/// NFT Attributes
pub type Attributes = BTreeMap<Vec<u8>, Vec<u8>>;

/// Land Token Class Id
pub const LAND_CLASS_ID: ClassId = 15;
/// Estate Token Class Id
pub const ESTATE_CLASS_ID: ClassId = 16;

/// Public item id for auction
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ItemId<Balance> {
	NFT(ClassId, TokenId),
	Spot(MapSpotId, MetaverseId),
	Metaverse(MetaverseId),
	Block(u64),
	Bundle(Vec<(ClassId, TokenId, Balance)>),
	UndeployedLandBlock(UndeployedLandBlockId),
}

impl<Balance: AtLeast32Bit + Copy> ItemId<Balance> {
	pub fn is_map_spot(&self) -> bool {
		matches!(self, ItemId::Spot(_, _))
	}

	pub fn get_map_spot_detail(&self) -> Option<(&MapSpotId, &MetaverseId)> {
		match self {
			ItemId::Spot(spot_id, metaverse_id) => Some((spot_id, metaverse_id)),
			_ => None,
		}
	}
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, MaxEncodedLen, PartialOrd, Ord, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum FungibleTokenId {
	NativeToken(TokenId),
	FungibleToken(TokenId),
	DEXShare(TokenId, TokenId),
	MiningResource(TokenId),
	Stable(TokenId), // kUSD
}

impl FungibleTokenId {
	pub fn is_native_token_currency_id(&self) -> bool {
		matches!(self, FungibleTokenId::NativeToken(_))
	}

	pub fn is_social_token_currency_id(&self) -> bool {
		matches!(self, FungibleTokenId::FungibleToken(_))
	}

	pub fn is_dex_share_social_token_currency_id(&self) -> bool {
		matches!(self, FungibleTokenId::DEXShare(_, _))
	}

	pub fn is_mining_resource_currency(&self) -> bool {
		matches!(self, FungibleTokenId::MiningResource(_))
	}

	pub fn split_dex_share_social_token_currency_id(&self) -> Option<(Self, Self)> {
		match self {
			FungibleTokenId::DEXShare(token_currency_id_0, token_currency_id_1) => Some((
				FungibleTokenId::NativeToken(*token_currency_id_0),
				FungibleTokenId::FungibleToken(*token_currency_id_1),
			)),
			_ => None,
		}
	}

	pub fn join_dex_share_social_currency_id(currency_id_0: Self, currency_id_1: Self) -> Option<Self> {
		match (currency_id_0, currency_id_1) {
			(
				FungibleTokenId::NativeToken(token_currency_id_0),
				FungibleTokenId::FungibleToken(token_currency_id_1),
			) => Some(FungibleTokenId::DEXShare(token_currency_id_0, token_currency_id_1)),
			(
				FungibleTokenId::FungibleToken(token_currency_id_0),
				FungibleTokenId::NativeToken(token_currency_id_1),
			) => Some(FungibleTokenId::DEXShare(token_currency_id_1, token_currency_id_0)),
			_ => None,
		}
	}

	pub fn decimals(&self) -> u8 {
		match self {
			FungibleTokenId::NativeToken(0) => 18, // Native token
			FungibleTokenId::NativeToken(1) | FungibleTokenId::NativeToken(2) | FungibleTokenId::Stable(0) => 12, // KSM KAR KUSD
			FungibleTokenId::MiningResource(0) => 18,
			_ => 18,
		}
	}
}

/// App-specific crypto used for reporting equivocation/misbehavior in BABE and
/// GRANDPA. Any rewards for misbehavior reporting will be paid out to this
/// account.
//pub mod report {
//	use frame_system::offchain::AppCrypto;
//	use sp_core::crypto::{key_types, KeyTypeId};
//
//	use super::{Signature, Verify};
//
//	/// Key type for the reporting module. Used for reporting BABE and GRANDPA
//	/// equivocations.
//	pub const KEY_TYPE: KeyTypeId = key_types::REPORTING;
//
//	mod app {
//		use sp_application_crypto::{app_crypto, sr25519};
//
//		app_crypto!(sr25519, super::KEY_TYPE);
//	}
//
//	/// Identity of the equivocation/misbehavior reporter.
//	pub type ReporterId = app::Public;
//
//	/// An `AppCrypto` type to allow submitting signed transactions using the reporting
//	/// application key as signer.
//	pub struct ReporterAppCrypto;
//
//	impl AppCrypto<<Signature as Verify>::Signer, Signature> for ReporterAppCrypto {
//		type RuntimeAppPublic = ReporterId;
//		type GenericSignature = sp_core::sr25519::Signature;
//		type GenericPublic = sp_core::sr25519::Public;
//	}
//}

/// The vesting schedule.
///
/// Benefits would be granted gradually, `per_period` amount every `period`
/// of blocks after `start`.
#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo)]
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

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum UndeployedLandBlockType {
	Transferable,
	BoundToAddress,
}

impl UndeployedLandBlockType {
	pub fn is_transferable(&self) -> bool {
		match *self {
			UndeployedLandBlockType::Transferable => true,
			_ => false,
		}
	}
}

impl Default for UndeployedLandBlockType {
	fn default() -> Self {
		UndeployedLandBlockType::Transferable
	}
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo)]
// #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct UndeployedLandBlock<AccountId> {
	/// id of undeploy land block
	pub id: UndeployedLandBlockId,
	/// Number of land units in this undeployed land block
	pub number_land_units: u32,
	/// Type of undeployed land block type
	pub undeployed_land_block_type: UndeployedLandBlockType,
	/// The owner of this asset.
	pub owner: AccountId,
	/// The approved co-owned of this asset, if one is set.
	pub approved: Option<AccountId>,
	/// Whether the undeployed land block is locked
	pub is_locked: bool,
}

// create_currency_id! {
// Represent a Token symbol with 8 bit
// Bit 8 : 0 for Pokladot Ecosystem, 1 for Kusama Ecosystem
// Bit 7 : Reserved
// Bit 6 - 1 : The token ID
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[repr(u8)]
pub enum TokenSymbol {
	// 0 => NEER
	// 1 => KSM
	// 2 => KAR
	// 3 => KUSD
	NEER = 0,
	// NEER("NEER Token", 18) = 10,
	KSM = 1,
	// KSM("Kusama", 12) = 4,
	KAR = 2,
	// KAR("Karura", 12) = 6,
	KUSD = 3, // KUSD("Karura Dollar", 12) = 2,
}
// }

impl Default for TokenSymbol {
	fn default() -> Self {
		Self::NEER
	}
}
