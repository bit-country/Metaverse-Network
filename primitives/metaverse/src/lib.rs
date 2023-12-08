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
use sp_runtime::RuntimeDebug;
use sp_runtime::{
	generic,
	traits::{BlakeTwo256, IdentifyAccount, Verify},
	MultiSignature,
};
use sp_runtime::{FixedU128, OpaqueExtrinsic as UncheckedExtrinsic};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::prelude::*;
use sp_std::vec::Vec;
use xcm::v3::MultiLocation;

pub mod bounded;
pub mod continuum;
pub mod estate;
pub mod evm;
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
/// Number of era on relaychain
pub type EraIndex = u32;
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
/// Weight ratio
pub type Ratio = FixedU128;
/// Trie index
pub type TrieIndex = u32;
/// Campaign index
pub type CampaignId = u32;
/// Index used for claim rewrads for merkle root campaigns
pub type ClaimId = u64;
/// Pool Id to keep track of each pool
pub type PoolId = u32;

/// Land Token Class Id
pub const LAND_CLASS_ID: ClassId = 15;
/// Estate Token Class Id
pub const ESTATE_CLASS_ID: ClassId = 16;

/// Public item id for auction
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ItemId<Balance> {
	NFT(ClassId, TokenId),
	StackableNFT(ClassId, TokenId, Balance),
	Spot(MapSpotId, MetaverseId),
	Metaverse(MetaverseId),
	Block(u64),
	Estate(EstateId),
	LandUnit((i32, i32), MetaverseId),
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
	Stable(TokenId),
}

impl FungibleTokenId {
	pub fn is_native_token_currency_id(&self) -> bool {
		matches!(self, FungibleTokenId::NativeToken(_))
	}

	pub fn is_social_token_currency_id(&self) -> bool {
		matches!(self, FungibleTokenId::FungibleToken(_))
	}

	pub fn is_mining_resource_currency(&self) -> bool {
		matches!(self, FungibleTokenId::MiningResource(_))
	}

	pub fn decimals(&self) -> u8 {
		match self {
			FungibleTokenId::NativeToken(0) => 18, // Native token
			FungibleTokenId::NativeToken(1) | FungibleTokenId::NativeToken(2) | FungibleTokenId::Stable(0) => 12, // KSM
			FungibleTokenId::MiningResource(0) => 18,
			_ => 18,
		}
	}
}

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo)]
pub enum AssetIds {
	Erc20(EvmAddress),
	StableAssetId(TokenId),
	ForeignAssetId(TokenId),
	NativeAssetId(FungibleTokenId),
}

pub trait BuyWeightRate {
	fn calculate_rate(location: MultiLocation) -> Option<Ratio>;
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, MaxEncodedLen, PartialOrd, Ord, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct NftOffer<Balance, BlockNumber> {
	/// Offer amount
	pub amount: Balance,
	/// Offer expiry block
	pub end_block: BlockNumber,
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

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo)]
pub struct AssetMetadata<Balance> {
	pub name: Vec<u8>,
	pub symbol: Vec<u8>,
	pub decimals: u8,
	pub minimal_balance: Balance,
}

/// A mapping between AssetId and AssetMetadata.
pub trait ForeignAssetIdMapping<ForeignAssetId, MultiLocation, AssetMetadata> {
	/// Returns the AssetMetadata associated with a given `AssetIds`.
	fn get_asset_metadata(asset_ids: AssetIds) -> Option<AssetMetadata>;
	/// Returns the MultiLocation associated with a given ForeignAssetId.
	fn get_multi_location(foreign_asset_id: ForeignAssetId) -> Option<MultiLocation>;
	/// Returns the CurrencyId associated with a given MultiLocation.
	fn get_currency_id(multi_location: MultiLocation) -> Option<FungibleTokenId>;
}
#[derive(Eq, PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum RewardType<FungibleTokenId, Balance, ClassId, TokenId> {
	FungibleTokens(FungibleTokenId, Balance),
	NftAssets(Vec<(ClassId, TokenId)>),
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[codec(dumb_trait_bound)]
pub struct CampaignInfoV1<AccountId, Balance, BlockNumber> {
	/// The creator account who created this campaign.
	pub creator: AccountId,
	/// The total reward amount.
	pub reward: Balance,
	/// The total claimed amount.
	pub claimed: Balance,
	/// Block number this campaign need to end
	pub end: BlockNumber,
	/// A hard-cap on the each reward amount that may be contributed.
	pub cap: Balance,
	/// Duration of the period during which rewards can be claimed.
	pub cooling_off_duration: BlockNumber,
	/// Index used for the child trie of this fund
	pub trie_index: TrieIndex,
}

/// Information on a funding effort for a pre-existing parachain. We assume that the parachain ID
/// is known as it's used for the key of the storage item for which this is the value (`Funds`).
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[codec(dumb_trait_bound)]
pub struct CampaignInfoV2<AccountId, Balance, BlockNumber> {
	/// The creator account who created this campaign.
	pub creator: AccountId,
	/// The campaign info properties.
	pub properties: Vec<u8>,
	/// The total reward amount.
	pub reward: Balance,
	/// The total claimed amount.
	pub claimed: Balance,
	/// Block number this campaign need to end
	pub end: BlockNumber,
	/// A hard-cap on the each reward amount that may be contributed.
	pub cap: Balance,
	/// Duration of the period during which rewards can be claimed.
	pub cooling_off_duration: BlockNumber,
	/// Index used for the child trie of this fund
	pub trie_index: TrieIndex,
}

/// Information on a funding effort for a pre-existing parachain. We assume that the parachain ID
/// is known as it's used for the key of the storage item for which this is the value (`Funds`).
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[codec(dumb_trait_bound)]
pub struct CampaignInfo<AccountId, Balance, BlockNumber, FungibleTokenId, ClassId, TokenId> {
	/// The creator account who created this campaign.
	pub creator: AccountId,
	/// The campaign info properties.
	pub properties: Vec<u8>,
	/// Block number this campaign need to end
	pub end: BlockNumber,
	/// Duration of the period during which rewards can be claimed.
	pub cooling_off_duration: BlockNumber,
	/// Index used for the child trie of this fund
	pub trie_index: TrieIndex,
	/// The total reward amount.
	pub reward: RewardType<FungibleTokenId, Balance, ClassId, TokenId>,
	/// The total claimed amount.
	pub claimed: RewardType<FungibleTokenId, Balance, ClassId, TokenId>,
	/// A hard-cap on the each reward amount that may be contributed.
	pub cap: RewardType<FungibleTokenId, Balance, ClassId, TokenId>,
}
// For multiple time calculation type
#[derive(Encode, Decode, Clone, RuntimeDebug, Eq, TypeInfo, MaxEncodedLen)]
pub enum StakingRound {
	Era(#[codec(compact)] u32),
	Round(#[codec(compact)] u32),
	Epoch(#[codec(compact)] u32),
	Hour(#[codec(compact)] u32),
}

impl Default for StakingRound {
	fn default() -> Self {
		StakingRound::Era(0u32)
	}
}

impl PartialEq for StakingRound {
	fn eq(&self, other: &Self) -> bool {
		match (&self, other) {
			(Self::Era(a), Self::Era(b)) => a.eq(b),
			(Self::Round(a), Self::Round(b)) => a.eq(b),
			(Self::Epoch(a), Self::Epoch(b)) => a.eq(b),
			(Self::Hour(a), Self::Hour(b)) => a.eq(b),
			_ => false,
		}
	}
}

impl Ord for StakingRound {
	fn cmp(&self, other: &Self) -> sp_std::cmp::Ordering {
		match (&self, other) {
			(Self::Era(a), Self::Era(b)) => a.cmp(b),
			(Self::Round(a), Self::Round(b)) => a.cmp(b),
			(Self::Epoch(a), Self::Epoch(b)) => a.cmp(b),
			(Self::Hour(a), Self::Hour(b)) => a.cmp(b),
			_ => sp_std::cmp::Ordering::Less,
		}
	}
}

impl PartialOrd for StakingRound {
	fn partial_cmp(&self, other: &Self) -> Option<sp_std::cmp::Ordering> {
		match (&self, other) {
			(Self::Era(a), Self::Era(b)) => Some(a.cmp(b)),
			(Self::Round(a), Self::Round(b)) => Some(a.cmp(b)),
			(Self::Epoch(a), Self::Epoch(b)) => Some(a.cmp(b)),
			(Self::Hour(a), Self::Hour(b)) => Some(a.cmp(b)),
			_ => None,
		}
	}
}
