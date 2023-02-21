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

use codec::{Decode, Encode, HasCompact};
use frame_support::storage::{child, ChildTriePrefixIterator};
use frame_support::traits::{LockIdentifier, WithdrawReasons};
use frame_support::{
	ensure, log,
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement, LockableCurrency, ReservableCurrency},
	transactional, PalletId,
};
use frame_system::{ensure_signed, pallet_prelude::*};
use orml_traits::{DataFeeder, DataProvider, MultiCurrency, MultiReservableCurrency};
use sp_core::Encode as SPEncode;
use sp_io::hashing::keccak_256;
use sp_runtime::traits::{BlockNumberProvider, CheckedAdd, CheckedMul, Hash as Hasher, Saturating};
use sp_runtime::{
	traits::{AccountIdConversion, One, Zero},
	ArithmeticError, DispatchError, Perbill, SaturatedConversion,
};
use sp_std::{collections::btree_map::BTreeMap, prelude::*, vec::Vec};

use core_primitives::NFTTrait;
use core_primitives::*;
pub use pallet::*;
use primitives::{
	estate::Estate, CampaignId, CampaignInfo, CampaignInfoV1, CampaignInfoV2, EstateId, Hash, RewardType, TrieIndex,
};
use primitives::{Balance, ClaimId, ClassId, FungibleTokenId, NftId};
pub use weights::WeightInfo;

//#[cfg(feature = "runtime-benchmarks")]
//pub mod benchmarking;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::traits::tokens::currency;
	use frame_support::traits::ExistenceRequirement::AllowDeath;
	use orml_traits::{rewards, MultiCurrencyExtended};
	use sp_runtime::traits::{CheckedAdd, CheckedSub, Saturating};
	use sp_runtime::ArithmeticError;

	use primitives::staking::RoundInfo;
	use primitives::{CampaignId, CampaignInfo, ClassId, NftId};

	use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	pub type TokenId = NftId;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The currency type
		type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>
			+ ReservableCurrency<Self::AccountId>;

		/// Multi-fungible token currency
		type FungibleTokenCurrency: MultiReservableCurrency<
			Self::AccountId,
			CurrencyId = FungibleTokenId,
			Balance = Balance,
		>;

		/// `PalletId` for the reward campaign pallet. An appropriate value could be
		/// `PalletId(*b"b/reward")`
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// The currency id of BIT
		#[pallet::constant]
		type MiningCurrencyId: Get<FungibleTokenId>;

		/// The minimum reward pool for a campaign
		#[pallet::constant]
		type MinimumRewardPool: Get<BalanceOf<Self>>;

		/// The amount to be held on deposit by the creator when creating new campaign.
		#[pallet::constant]
		type CampaignDeposit: Get<BalanceOf<Self>>;

		/// The minimum amount of blocks during which campaign rewards can be claimed.
		#[pallet::constant]
		type MinimumCampaignDuration: Get<Self::BlockNumber>;

		/// The minimum amount of blocks during which campaign rewards can be claimed.
		#[pallet::constant]
		type MinimumCampaignCoolingOffPeriod: Get<Self::BlockNumber>;

		/// The maximum amount of leaf nodes that could be passed when claiming reward
		#[pallet::constant]
		type MaxLeafNodes: Get<u64>;
		/// The max number of accounts that could be rewarded per extrinsic
		#[pallet::constant]
		type MaxSetRewardsListLength: Get<u64>;

		/// Accounts that can set rewards
		type AdminOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;

		/// NFT trait type that handler NFT implementation
		type NFTHandler: NFTTrait<Self::AccountId, BalanceOf<Self>, ClassId = ClassId, TokenId = TokenId>;

		/// Weight info
		type WeightInfo: WeightInfo;
	}

	/// Info of campaign.
	#[pallet::storage]
	#[pallet::getter(fn campaigns)]
	pub(super) type Campaigns<T: Config> = StorageMap<
		_,
		Twox64Concat,
		CampaignId,
		CampaignInfo<T::AccountId, BalanceOf<T>, T::BlockNumber, FungibleTokenId, ClassId, TokenId>,
	>;

	/// List of merkle roots for each campaign
	#[pallet::storage]
	#[pallet::getter(fn campaign_merkle_roots)]
	pub(super) type CampaignMerkleRoots<T: Config> = StorageMap<_, Twox64Concat, CampaignId, Vec<Hash>, ValueQuery>;

	/// List of indexes that can claim rewards for every campaign
	#[pallet::storage]
	#[pallet::getter(fn campaign_claim_indexes)]
	pub(super) type CampaignClaimIndexes<T: Config> =
		StorageMap<_, Twox64Concat, CampaignId, Vec<(T::AccountId, ClaimId)>, ValueQuery>;

	/// Tracker for the next available trie index
	#[pallet::storage]
	#[pallet::getter(fn next_trie_index)]
	pub(super) type NextTrieIndex<T> = StorageValue<_, u32, ValueQuery>;

	/// Tracker for the next available campaign index
	#[pallet::storage]
	#[pallet::getter(fn next_campaign_id)]
	pub(super) type NextCampaignId<T> = StorageValue<_, u32, ValueQuery>;

	/// Set reward origins
	#[pallet::storage]
	#[pallet::getter(fn set_reward_origins)]
	pub type SetRewardOrigins<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, (), OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New campaign created [campaign_id, account]
		NewRewardCampaignCreated(CampaignId, T::AccountId),
		/// Reward claimed [campaign_id, account, balance]
		RewardClaimed(CampaignId, T::AccountId, BalanceOf<T>),
		/// Reward claimed [campaign_id, account, assets]
		NftRewardClaimed(CampaignId, T::AccountId, Vec<(ClassId, TokenId)>),
		/// Set reward using merkle root [campaign_id, balance, hash]
		SetRewardRoot(CampaignId, BalanceOf<T>, Hash),
		/// Set NFT rewards using merkle root[campaign_id, hash]
		SetNftRewardRoot(CampaignId, Hash),
		/// Set reward [campaign_id, rewards_list]
		SetReward(CampaignId, Vec<(T::AccountId, BalanceOf<T>)>),
		/// Set reward [campaign_id, rewards_list]
		SetNftReward(CampaignId, Vec<(T::AccountId, Vec<(ClassId, TokenId)>)>),
		/// Reward campaign ended [campaign_id]
		RewardCampaignEnded(CampaignId),
		/// Reward campaign closed [campaign_id]
		RewardCampaignClosed(CampaignId),
		/// Reward campaign  root closed [campaign_id]
		RewardCampaignRootClosed(CampaignId),
		/// Reward campaign canceled [campaign_id]
		RewardCampaignCanceled(CampaignId),
		/// Set reward origin added [account]
		SetRewardOriginAdded(T::AccountId),
		/// Set reward origin removed [account]
		SetRewardOriginRemoved(T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Campaign has ended or not valid
		CampaignIsNotFound,
		/// No reward found in this account
		NoRewardFound,
		/// Campaign already expired
		CampaignExpired,
		/// Reward exceed the cap reward
		RewardExceedCap,
		/// Invalid reward account
		InvalidRewardAccount,
		/// Campaign reward pool is below the set minimum
		RewardPoolBelowMinimum,
		/// Campaign duration is below minimum
		CampaignDurationBelowMinimum,
		/// Campaign cooling-off duration is below minimum
		CoolingOffPeriodBelowMinimum,
		/// Campaign is still active
		CampaignStillActive,
		/// Not campaign creator
		NotCampaignCreator,
		/// Campaign period for setting rewards is over
		CampaignEnded,
		/// Reward origin already added
		SetRewardOriginAlreadyAdded,
		/// Reward origin does not exist
		SetRewardOriginDoesNotExist,
		/// Invalid set reward origin
		InvalidSetRewardOrigin,
		/// Invalid reward type
		InvalidRewardType,
		/// Cannot use an NFT token for a reward pool
		NoPermissionToUseNftInRewardPool,
		/// Nft token reward is already assigned
		NftTokenCannotBeRewarded,
		/// Invalid left NFT quantity
		InvalidNftQuantity,
		/// Invalid campaign type
		InvalidCampaignType,
		/// Reward is already set
		RewardAlreadySet,
		/// Reward leaf amount is larger then maximum
		InvalidRewardLeafAmount,
		/// Merkle root is not related to a campaign
		MerkleRootNotRelatedToCampaign,
		/// No merkle roots found
		NoMerkleRootsFound,
		/// Invalid merkle roots quantity
		InvalidMerkleRootsQuantity,
		/// The account is already rewarded for this campaign
		AccountAlreadyRewarded,
		/// Invalid total NFT rewards amount parameter
		InvalidTotalNftRewardAmountParameter,
		/// Rewards list size is above maximum permited size
		RewardsListSizeAboveMaximum,
		/// Arthimetic operation overflow
		ArithmeticOverflow,
		/// Invalid claim index value
		InvalidClaimIndex,
		/// Reward claim entry does not exist in the campaign claim index
		NoClaimIndexEntry,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new token-based campaign from parameters
		///
		/// The dispatch origin for this call must be _Signed_.
		/// - `creator`: the account for which the campaign is created.
		/// - `reward`: the total balance of the currency provided as reward.
		/// - `end`: the end block at which users can participate.
		/// - `cooling_off_duration`: the duriation (in blocks) of the period during which accounts
		///   can claim rewards.
		/// - `properties`: information relevant for the campaign.
		/// - `currency_id`: specify the type of currency which for the reward pool.
		///
		/// Emits `NewRewardCampaignCreated` if successful.
		#[pallet::weight(T::WeightInfo::create_campaign())]
		pub fn create_campaign(
			origin: OriginFor<T>,
			creator: T::AccountId,
			reward: BalanceOf<T>,
			end: T::BlockNumber,
			cooling_off_duration: T::BlockNumber,
			properties: Vec<u8>,
			currency_id: FungibleTokenId,
		) -> DispatchResult {
			let depositor = ensure_signed(origin)?;

			ensure!(
				end > frame_system::Pallet::<T>::block_number(),
				Error::<T>::CampaignDurationBelowMinimum
			);

			let campaign_duration = end.saturating_sub(frame_system::Pallet::<T>::block_number());

			ensure!(
				campaign_duration >= T::MinimumCampaignDuration::get(),
				Error::<T>::CampaignDurationBelowMinimum
			);

			ensure!(
				reward >= T::MinimumRewardPool::get(),
				Error::<T>::RewardPoolBelowMinimum
			);

			ensure!(
				cooling_off_duration >= T::MinimumCampaignCoolingOffPeriod::get(),
				Error::<T>::CoolingOffPeriodBelowMinimum
			);

			let trie_index = Self::next_trie_index();
			let next_trie_index = trie_index.checked_add(1).ok_or(ArithmeticError::Overflow)?;

			let deposit = T::CampaignDeposit::get();

			let campaign_id = Self::next_campaign_id();

			let fund_account = Self::fund_account_id(campaign_id);
			T::Currency::transfer(&depositor, &fund_account, deposit, AllowDeath)?;
			T::FungibleTokenCurrency::transfer(currency_id, &depositor, &fund_account, reward.saturated_into())?;

			let next_campaign_id = campaign_id.checked_add(1).ok_or(ArithmeticError::Overflow)?;

			Campaigns::<T>::insert(
				campaign_id,
				CampaignInfo {
					creator: creator.clone(),
					properties,
					end,
					cooling_off_duration,
					trie_index,
					reward: RewardType::FungibleTokens(currency_id, reward),
					claimed: RewardType::FungibleTokens(currency_id, Zero::zero()),
					cap: RewardType::FungibleTokens(currency_id, reward),
				},
			);

			let empty_root_vec: Vec<Hash> = Vec::new();
			let empty_acc_vec: Vec<(T::AccountId, ClaimId)> = Vec::new();
			CampaignMerkleRoots::<T>::insert(campaign_id, empty_root_vec);
			CampaignClaimIndexes::<T>::insert(campaign_id, empty_acc_vec);

			NextTrieIndex::<T>::put(next_trie_index);
			NextCampaignId::<T>::put(next_campaign_id);

			Self::deposit_event(Event::<T>::NewRewardCampaignCreated(campaign_id, creator));

			Ok(())
		}

		/// Create a new NFT-based campaign from parameters
		///
		/// The dispatch origin for this call must be _Signed_.
		/// - `creator`: the account for which the campaign is created.
		/// - `reward`: the pool of NFTs that will be provided as reward.
		/// - `end`: the end block at which users can participate.
		/// - `cooling_off_duration`: the duriation (in blocks) of the period during which accounts
		///   can claim rewards.
		/// - `properties`: information relevant for the campaign.
		///
		/// Emits `NewRewardCampaignCreated` if successful.
		#[pallet::weight(T::WeightInfo::create_campaign() * (1u64 + reward.len() as u64))]
		#[transactional]
		pub fn create_nft_campaign(
			origin: OriginFor<T>,
			creator: T::AccountId,
			reward: Vec<(ClassId, TokenId)>,
			end: T::BlockNumber,
			cooling_off_duration: T::BlockNumber,
			properties: Vec<u8>,
		) -> DispatchResult {
			let depositor = ensure_signed(origin)?;

			let campaign_duration = end.saturating_sub(frame_system::Pallet::<T>::block_number());

			ensure!(
				campaign_duration >= T::MinimumCampaignDuration::get(),
				Error::<T>::CampaignDurationBelowMinimum
			);

			ensure!(
				cooling_off_duration >= T::MinimumCampaignCoolingOffPeriod::get(),
				Error::<T>::CoolingOffPeriodBelowMinimum
			);

			ensure!(reward.len() > 0, Error::<T>::RewardPoolBelowMinimum);

			//ensure!(
			//	!reward.contains(&(0u32.into(), 0u64.into())),
			//	Error::<T>::CannotUseGenesisNftForReward
			//);

			let trie_index = Self::next_trie_index();
			let campaign_id = Self::next_campaign_id();
			let fund_account = Self::fund_account_id(campaign_id);

			for token in reward.clone() {
				ensure!(
					T::NFTHandler::check_ownership(&creator, &(token.0, token.1))?
						&& T::NFTHandler::is_transferable(&(token.0, token.1))?,
					Error::<T>::NoPermissionToUseNftInRewardPool
				);
				T::NFTHandler::set_lock_nft((token.0, token.1), true)?
			}

			let next_trie_index = trie_index.checked_add(1).ok_or(ArithmeticError::Overflow)?;
			let next_campaign_id = campaign_id.checked_add(1).ok_or(ArithmeticError::Overflow)?;

			T::Currency::transfer(&depositor, &fund_account, T::CampaignDeposit::get(), AllowDeath)?;

			Campaigns::<T>::insert(
				campaign_id,
				CampaignInfo {
					creator: creator.clone(),
					properties,
					end,
					cooling_off_duration,
					trie_index,
					reward: RewardType::NftAssets(reward.clone()),
					claimed: RewardType::NftAssets(Vec::new()),
					cap: RewardType::NftAssets(reward),
				},
			);

			let empty_root_vec: Vec<Hash> = Vec::new();
			let empty_acc_vec: Vec<(T::AccountId, ClaimId)> = Vec::new();
			CampaignMerkleRoots::<T>::insert(campaign_id, empty_root_vec);
			CampaignClaimIndexes::<T>::insert(campaign_id, empty_acc_vec);

			NextTrieIndex::<T>::put(next_trie_index);
			NextCampaignId::<T>::put(next_campaign_id);

			Self::deposit_event(Event::<T>::NewRewardCampaignCreated(campaign_id, creator));

			Ok(())
		}

		/// Claim reward set without merkle root for token-based campaign
		///
		/// The dispatch origin for this call must be _Signed_. This extrinsic only works if the
		/// account is rewarded for the campaign.
		/// - `campaign_id`: the ID of the campaign for which the account is claiming reward.
		///
		/// Emits `RewardClaimed` if successful.
		#[pallet::weight(T::WeightInfo::claim_reward())]
		pub fn claim_reward(origin: OriginFor<T>, id: CampaignId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let now = frame_system::Pallet::<T>::block_number();

			<Campaigns<T>>::try_mutate_exists(id, |campaign| -> DispatchResult {
				let mut campaign = campaign.as_mut().ok_or(Error::<T>::CampaignIsNotFound)?;

				ensure!(campaign.end < now, Error::<T>::CampaignStillActive);

				ensure!(
					campaign.end + campaign.cooling_off_duration >= now,
					Error::<T>::CampaignExpired
				);

				match campaign.claimed {
					RewardType::FungibleTokens(c, r) => {
						let fund_account = Self::fund_account_id(id);
						let (balance, _) = Self::reward_get(campaign.trie_index, &who);
						ensure!(balance > Zero::zero(), Error::<T>::NoRewardFound);
						// TO DO: Find account balance
						T::FungibleTokenCurrency::transfer(c, &fund_account, &who, balance.saturated_into())?;

						Self::reward_kill(campaign.trie_index, &who);

						campaign.claimed = RewardType::FungibleTokens(c, r.saturating_add(balance));
						Self::deposit_event(Event::<T>::RewardClaimed(id, who, balance));
						Ok(())
					}
					_ => Err(Error::<T>::InvalidCampaignType.into()),
				}
			})?;
			Ok(())
		}

		/// Claim reward set with merkle root for token-based campaign
		///
		/// The dispatch origin for this call must be _Signed_. This extrinsic only works if the
		/// account is rewarded for the campaign.
		/// - `campaign_id`: the ID of the campaign for which the account is claiming reward.
		/// - `balance`: the amount of tokens which the account will claim (required for
		///   merkle-proof calculation).
		/// - `leaf_nodes`: list of the merkle tree nodes required for merkle-proof calculation.
		///
		/// Emits `RewardClaimed` if successful.
		#[pallet::weight(T::WeightInfo::claim_reward_root()  * (1u64 + leaf_nodes.len() as u64))]
		#[transactional]
		pub fn claim_reward_root(
			origin: OriginFor<T>,
			id: CampaignId,
			claim_id: ClaimId,
			balance: BalanceOf<T>,
			leaf_nodes: Vec<Hash>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let now = frame_system::Pallet::<T>::block_number();

			<Campaigns<T>>::try_mutate_exists(id, |campaign| -> DispatchResult {
				let mut campaign = campaign.as_mut().ok_or(Error::<T>::CampaignIsNotFound)?;

				ensure!(campaign.end < now, Error::<T>::CampaignStillActive);

				ensure!(
					campaign.end + campaign.cooling_off_duration >= now,
					Error::<T>::CampaignExpired
				);

				<CampaignClaimIndexes<T>>::try_mutate(id, |campaign_claim_index| -> DispatchResult {
					//let mut new_claim_index = campaign_claim_index.clone();
					match campaign_claim_index.binary_search(&(who.clone(), claim_id)) {
						Ok(claim_index_entry_id) => {
							match campaign.claimed {
								RewardType::FungibleTokens(c, r) => {
									let fund_account = Self::fund_account_id(id);

									let merkle_root = Self::calculate_merkle_proof(&claim_id, &balance, &leaf_nodes)?;

									ensure!(
										Self::campaign_merkle_roots(id).contains(&merkle_root),
										Error::<T>::MerkleRootNotRelatedToCampaign
									);
									//ensure!(
									//	!Self::campaign_claimed_accounts_list(id).contains(&who),
									//	Error::<T>::NoRewardFound
									//s);

									campaign_claim_index.remove(claim_index_entry_id);

									let (root_balance, _) =
										Self::reward_get_root(campaign.trie_index, merkle_root.clone());
									// extra check in case the CampaignMerkleRoots storage is corrupted
									ensure!(root_balance > Zero::zero(), Error::<T>::NoRewardFound);
									T::FungibleTokenCurrency::transfer(
										c,
										&fund_account,
										&who,
										balance.saturated_into(),
									)?;

									campaign.claimed = RewardType::FungibleTokens(c, r.saturating_add(balance));
									Self::deposit_event(Event::<T>::RewardClaimed(id, who, balance));
								}
								_ => return Err(Error::<T>::InvalidCampaignType.into()),
							}
						}
						_ => return Err(Error::<T>::NoClaimIndexEntry.into()),
					}
					Ok(())
				})
			})?;
			Ok(())
		}

		/// Claim reward set without merkle root for NFT-based campaign
		///
		/// The dispatch origin for this call must be _Signed_. This extrinsic only works if the
		/// account is rewarded for the campaign.
		/// - `campaign_id`: the ID of the campaign for which the account is claiming reward.
		/// - `amount`: the amount of NFTs that the account is going to claim
		///
		/// Emits `RewardClaimed` if successful.
		#[pallet::weight(T::WeightInfo::claim_nft_reward() * (1u64 + amount))]
		#[transactional]
		pub fn claim_nft_reward(origin: OriginFor<T>, id: CampaignId, amount: u64) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let now = frame_system::Pallet::<T>::block_number();

			<Campaigns<T>>::try_mutate_exists(id, |campaign| -> DispatchResult {
				let mut campaign = campaign.as_mut().ok_or(Error::<T>::CampaignIsNotFound)?;

				ensure!(campaign.end < now, Error::<T>::CampaignStillActive);

				ensure!(
					campaign.end + campaign.cooling_off_duration >= now,
					Error::<T>::CampaignExpired
				);

				match campaign.reward.clone() {
					RewardType::NftAssets(reward) => match campaign.claimed.clone() {
						RewardType::NftAssets(claimed) => {
							let (tokens, _) = Self::reward_get_nft(campaign.trie_index, &who);
							ensure!(!tokens.is_empty(), Error::<T>::NoRewardFound);
							ensure!(tokens.len() as u64 == amount, Error::<T>::InvalidNftQuantity);

							let mut new_claimed = claimed.clone();

							for token in tokens.clone() {
								ensure!(
									reward.contains(&token) && !claimed.contains(&token),
									Error::<T>::NoRewardFound
								);
								T::NFTHandler::set_lock_nft((token.0, token.1), false)?;
								T::NFTHandler::transfer_nft(&campaign.creator, &who, &token)?;
								new_claimed.push(token);
							}

							campaign.claimed = RewardType::NftAssets(new_claimed);
							Self::reward_kill(campaign.trie_index, &who);

							Self::deposit_event(Event::<T>::NftRewardClaimed(id, who, tokens));
							Ok(())
						}
						_ => Err(Error::<T>::InvalidCampaignType.into()),
					},
					_ => Err(Error::<T>::InvalidCampaignType.into()),
				}
			})?;
			Ok(())
		}

		/// Claim reward set with merkle root for NFT-based campaign
		///
		/// The dispatch origin for this call must be _Signed_. This extrinsic only works if the
		/// account is rewarded for the campaign.
		/// - `campaign_id`: the ID of the campaign for which the account is claiming reward.
		/// - `tokens`: the list of NFTs which the account will claim (required for  merkle-proof
		///   calculation).
		/// - `leaf_nodes`: list of the merkle tree nodes required for  merkle-proof calculation.
		///
		/// Emits `RewardClaimed` if successful.
		#[pallet::weight(T::WeightInfo::claim_nft_reward_root() * (1u64 + tokens.len() as u64))]
		#[transactional]
		pub fn claim_nft_reward_root(
			origin: OriginFor<T>,
			id: CampaignId,
			claim_id: ClaimId,
			tokens: Vec<(ClassId, TokenId)>,
			leaf_nodes: Vec<Hash>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let now = frame_system::Pallet::<T>::block_number();

			<Campaigns<T>>::try_mutate_exists(id, |campaign| -> DispatchResult {
				let mut campaign = campaign.as_mut().ok_or(Error::<T>::CampaignIsNotFound)?;

				ensure!(campaign.end < now, Error::<T>::CampaignStillActive);

				ensure!(
					campaign.end + campaign.cooling_off_duration >= now,
					Error::<T>::CampaignExpired
				);

				ensure!(
					Self::campaign_claim_indexes(id).contains(&(who.clone(), claim_id)),
					Error::<T>::NoClaimIndexEntry
				);

				match campaign.reward.clone() {
					RewardType::NftAssets(reward) => match campaign.claimed.clone() {
						RewardType::NftAssets(claimed) => {
							let merkle_proof: Hash =
								Self::calculate_nft_rewards_merkle_proof(&claim_id.clone(), &tokens, &leaf_nodes)?;

							ensure!(
								Self::campaign_merkle_roots(id).contains(&merkle_proof),
								Error::<T>::MerkleRootNotRelatedToCampaign
							);

							let (tokens, _) = Self::reward_get_nft_root(campaign.trie_index, merkle_proof);
							ensure!(!tokens.is_empty(), Error::<T>::NoRewardFound);

							let mut new_claimed = claimed;
							for token in tokens.clone() {
								ensure!(
									reward.contains(&token) && !new_claimed.contains(&token),
									Error::<T>::NoRewardFound
								);

								T::NFTHandler::set_lock_nft((token.0, token.1), false)?;
								T::NFTHandler::transfer_nft(&campaign.creator, &who, &token)?;
								new_claimed.push(token);
							}

							campaign.claimed = RewardType::NftAssets(new_claimed);

							Self::deposit_event(Event::<T>::NftRewardClaimed(id, who, tokens));
							Ok(())
						}
						_ => Err(Error::<T>::InvalidCampaignType.into()),
					},
					_ => Err(Error::<T>::InvalidCampaignType.into()),
				}
			})?;
			Ok(())
		}

		/// Set reward for token-based campaign without using merkle root
		///
		/// The dispatch origin for this call must be _Signed_. This extrinsic only works if the
		/// origin got permission to set rewards.
		/// - `campaign_id`: the ID of the campaign for which the rewards will be set.
		/// - `rewards`: vector of account IDs and their's reward balances pairs.
		///
		/// Emits `SetReward` if successful.
		#[pallet::weight(T::WeightInfo::set_reward() * rewards.len() as u64)]
		#[transactional]
		pub fn set_reward(
			origin: OriginFor<T>,
			id: CampaignId,
			rewards: Vec<(T::AccountId, BalanceOf<T>)>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Self::is_set_reward_origin(&who), Error::<T>::InvalidSetRewardOrigin);
			ensure!(
				T::MaxSetRewardsListLength::get() >= rewards.len() as u64,
				Error::<T>::RewardsListSizeAboveMaximum
			);

			let now = frame_system::Pallet::<T>::block_number();

			<Campaigns<T>>::try_mutate_exists(id, |campaign| -> DispatchResult {
				let mut campaign = campaign.as_mut().ok_or(Error::<T>::CampaignIsNotFound)?;

				ensure!(
					campaign.end + campaign.cooling_off_duration >= now,
					Error::<T>::CampaignExpired
				);

				match campaign.cap {
					RewardType::FungibleTokens(c, b) => {
						let mut rewards_list: Vec<(T::AccountId, BalanceOf<T>)> = Vec::new();
						let mut total_amount: BalanceOf<T> = Zero::zero();
						for (to, amount) in rewards {
							total_amount = total_amount.saturating_add(amount);
							ensure!(total_amount <= b, Error::<T>::RewardExceedCap);

							let (balance, _) = Self::reward_get(campaign.trie_index, &to);
							ensure!(balance == Zero::zero(), Error::<T>::AccountAlreadyRewarded);

							Self::reward_put(campaign.trie_index, &to, &amount, &[]);
							rewards_list.push((to, amount));
						}
						campaign.cap = RewardType::FungibleTokens(c, b.saturating_sub(total_amount));
						Self::deposit_event(Event::<T>::SetReward(id, rewards_list));
						Ok(())
					}
					_ => Err(Error::<T>::InvalidCampaignType.into()),
				}
			})?;
			Ok(())
		}

		/// Set reward for token-based campaign using merkle root
		///
		/// The dispatch origin for this call must be _Signed_. This extrinsic only works if the
		/// origin got permission to set rewards.
		/// - `campaign_id`: the ID of the campaign for which the rewards will be set.
		/// - `total_amount`: the amount of tokens which will be rewarded.
		/// - `merkle_root`: the merkle root that will be used when claiming rewards.
		///
		/// Emits `SetReward` if successful.
		#[pallet::weight(T::WeightInfo::set_reward_root())]
		pub fn set_reward_root(
			origin: OriginFor<T>,
			id: CampaignId,
			total_amount: BalanceOf<T>,
			merkle_root: Hash,
			claim_index: Vec<(T::AccountId, ClaimId)>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Self::is_set_reward_origin(&who), Error::<T>::InvalidSetRewardOrigin);
			ensure!(claim_index.len() > 0, Error::<T>::InvalidClaimIndex);

			let now = frame_system::Pallet::<T>::block_number();

			<Campaigns<T>>::try_mutate_exists(id, |campaign| -> DispatchResult {
				let mut campaign = campaign.as_mut().ok_or(Error::<T>::CampaignIsNotFound)?;

				ensure!(
					campaign.end + campaign.cooling_off_duration >= now,
					Error::<T>::CampaignExpired
				);

				match campaign.cap {
					RewardType::FungibleTokens(c, b) => {
						ensure!(b >= total_amount, Error::<T>::RewardExceedCap);

						let (balance, _) = Self::reward_get_root(campaign.trie_index, merkle_root.clone());
						ensure!(balance == Zero::zero(), Error::<T>::RewardAlreadySet);

						campaign.cap = RewardType::FungibleTokens(c, b.saturating_sub(total_amount));

						<CampaignMerkleRoots<T>>::try_mutate_exists(id, |campaign_roots| -> DispatchResult {
							let mut campaign_roots_vec: Vec<Hash> = campaign_roots.clone().unwrap_or(Vec::new());
							campaign_roots_vec.push(merkle_root);
							campaign_roots.replace(campaign_roots_vec);
							Ok(())
						});

						<CampaignClaimIndexes<T>>::try_mutate_exists(id, |campaign_claim_index| -> DispatchResult {
							let mut campaign_claim_index_vec: Vec<(T::AccountId, ClaimId)> =
								campaign_claim_index.clone().unwrap_or(Vec::new());
							campaign_claim_index_vec.append(&mut claim_index.clone());
							campaign_claim_index.replace(campaign_claim_index_vec);
							Ok(())
						});

						Self::reward_put_root(campaign.trie_index, merkle_root.clone(), &total_amount, &[]);
						Self::deposit_event(Event::<T>::SetRewardRoot(id, total_amount, merkle_root));
						Ok(())
					}
					_ => Err(Error::<T>::InvalidCampaignType.into()),
				}
			})?;
			Ok(())
		}

		/// Set reward for NFT-based campaign without using merkle root
		///
		/// The dispatch origin for this call must be _Signed_. This extrinsic only works if the
		/// origin got permission to set rewards.
		/// - `campaign_id`: the ID of the campaign for which the rewards will be set.
		/// - `rewards`: vector of account IDs and their number of tokens that they will receive
		///   pairs.
		/// - `total_nfts_amount`: the total number of NFTs that will be rewrad.
		///
		/// Emits `SetReward` if successful.
		#[pallet::weight(T::WeightInfo::set_nft_reward() * total_nfts_amount)]
		#[transactional]
		pub fn set_nft_reward(
			origin: OriginFor<T>,
			id: CampaignId,
			rewards: Vec<(T::AccountId, u64)>,
			total_nfts_amount: u64,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Self::is_set_reward_origin(&who), Error::<T>::InvalidSetRewardOrigin);
			ensure!(
				T::MaxSetRewardsListLength::get() >= rewards.len() as u64,
				Error::<T>::RewardsListSizeAboveMaximum
			);

			let now = frame_system::Pallet::<T>::block_number();

			<Campaigns<T>>::try_mutate_exists(id, |campaign| -> DispatchResult {
				let mut campaign = campaign.as_mut().ok_or(Error::<T>::CampaignIsNotFound)?;

				ensure!(
					campaign.end + campaign.cooling_off_duration >= now,
					Error::<T>::CampaignExpired
				);

				match campaign.cap.clone() {
					RewardType::NftAssets(cap) => {
						let mut new_cap = cap.clone();
						let mut rewards_list: Vec<(T::AccountId, Vec<(ClassId, NftId)>)> = Vec::new();
						let mut tokens: Vec<(ClassId, TokenId)> = Vec::new();
						let mut total_amount_left: u64 = total_nfts_amount;
						for (to, amount) in rewards {
							let (t, _) = Self::reward_get_nft(campaign.trie_index, &to);
							ensure!(t.is_empty(), Error::<T>::AccountAlreadyRewarded);

							ensure!(
								total_amount_left >= amount,
								Error::<T>::InvalidTotalNftRewardAmountParameter
							);
							total_amount_left.saturating_sub(amount);

							for l in 0..amount {
								let token = new_cap.pop().ok_or(Error::<T>::RewardExceedCap)?;
								tokens.push(token);
							}
							Self::reward_put_nft(campaign.trie_index, &to, &tokens, &[]);
							rewards_list.push((to, tokens));
							tokens = Vec::new();
						}
						campaign.cap = RewardType::NftAssets(new_cap);
						Self::deposit_event(Event::<T>::SetNftReward(id, rewards_list));
						Ok(())
					}
					_ => Err(Error::<T>::InvalidCampaignType.into()),
				}
			})?;
			Ok(())
		}

		/// Set reward for NFT-based campaign using merkle root
		///
		/// The dispatch origin for this call must be _Signed_. This extrinsic only works if the
		/// origin got permission to set rewards.
		/// - `campaign_id`: the ID of the campaign for which the rewards will be set.
		/// - `merkle_root`: the merkle root that will be used when claiming rewards.
		///
		/// Emits `SetReward` if successful.
		#[pallet::weight(T::WeightInfo::set_nft_reward_root())]
		pub fn set_nft_reward_root(
			origin: OriginFor<T>,
			id: CampaignId,
			merkle_root: Hash,
			claim_index: Vec<(T::AccountId, ClaimId)>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Self::is_set_reward_origin(&who), Error::<T>::InvalidSetRewardOrigin);
			ensure!(claim_index.len() > 0, Error::<T>::InvalidClaimIndex);

			let now = frame_system::Pallet::<T>::block_number();

			<Campaigns<T>>::try_mutate_exists(id, |campaign| -> DispatchResult {
				let mut campaign = campaign.as_mut().ok_or(Error::<T>::CampaignIsNotFound)?;

				ensure!(
					campaign.end + campaign.cooling_off_duration >= now,
					Error::<T>::CampaignExpired
				);

				match campaign.cap.clone() {
					RewardType::NftAssets(cap) => {
						ensure!(Self::campaign_merkle_roots(id).is_empty(), Error::<T>::RewardAlreadySet);

						ensure!(!cap.is_empty(), Error::<T>::RewardExceedCap);

						Self::reward_put_nft_root(campaign.trie_index, merkle_root, &cap, &[]);

						let mut merkle_roots_vec: Vec<Hash> = Vec::new();
						merkle_roots_vec.push(merkle_root);
						<CampaignMerkleRoots<T>>::insert(id, merkle_roots_vec);
						<CampaignClaimIndexes<T>>::insert(id, claim_index);

						campaign.cap = RewardType::NftAssets(Vec::new());
						Self::deposit_event(Event::<T>::SetNftRewardRoot(id, merkle_root));
						Ok(())
					}
					_ => Err(Error::<T>::InvalidCampaignType.into()),
				}
			})?;
			Ok(())
		}

		/// Close token-based campaign  
		///
		/// The dispatch origin for this call must be _Signed_. This extrinsic only works for the
		/// campign creator.
		/// - `campaign_id`: the ID of the campaign for which the rewards will be set.
		/// - `merkle_roots_quanity`: the amount of merkle roots that were used for setting rewards.
		///
		/// Emits `RewardCampaignClosed` and/or `RewardCampaignRootClosed`  if successful.
		#[pallet::weight(T::WeightInfo::close_campaign() * (1u64 + merkle_roots_quantity))]
		#[transactional]
		pub fn close_campaign(origin: OriginFor<T>, id: CampaignId, merkle_roots_quantity: u64) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let now = frame_system::Pallet::<T>::block_number();

			let mut campaign = Self::campaigns(id).ok_or(Error::<T>::CampaignIsNotFound)?;

			ensure!(who == campaign.creator, Error::<T>::NotCampaignCreator);

			ensure!(
				campaign.end + campaign.cooling_off_duration < now,
				Error::<T>::CampaignStillActive
			);

			let fund_account = Self::fund_account_id(id);
			match campaign.reward {
				RewardType::FungibleTokens(_, r) => match campaign.claimed {
					RewardType::FungibleTokens(c, b) => {
						let unclaimed_balance = r.saturating_sub(b);
						T::Currency::transfer(&fund_account, &who, T::CampaignDeposit::get(), AllowDeath)?;
						T::FungibleTokenCurrency::transfer(c, &fund_account, &who, unclaimed_balance.saturated_into())?;

						Self::reward_kill(campaign.trie_index, &who);

						Campaigns::<T>::remove(id);
						Self::deposit_event(Event::<T>::RewardCampaignClosed(id));

						let merkle_roots = Self::campaign_merkle_roots(id);

						ensure!(
							merkle_roots.len() as u64 == merkle_roots_quantity,
							Error::<T>::InvalidMerkleRootsQuantity
						);

						for root in merkle_roots.clone() {
							Self::reward_kill_root(campaign.trie_index, &root);
						}

						if merkle_roots.len() as u64 > 0 {
							CampaignMerkleRoots::<T>::remove(id);
							CampaignClaimIndexes::<T>::remove(id);
							Self::deposit_event(Event::<T>::RewardCampaignRootClosed(id));
						}

						Ok(())
					}
					_ => Err(Error::<T>::InvalidCampaignType.into()),
				},
				_ => Err(Error::<T>::InvalidCampaignType.into()),
			}
		}

		/// Close NFT-based campaign  
		///
		/// The dispatch origin for this call must be _Signed_. This extrinsic only works for the
		/// campign creator.
		/// - `campaign_id`: the ID of the campaign for which the rewards will be set.
		/// - `left_nfts`: the amount of unclaimed NFTs in the reward pool.
		///
		/// Emits `RewardCampaignClosed` and/or `RewardCampaignRootClosed`  if successful.
		#[pallet::weight(T::WeightInfo::close_nft_campaign() * (1u64 + left_nfts))]
		#[transactional]
		pub fn close_nft_campaign(origin: OriginFor<T>, id: CampaignId, left_nfts: u64) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let now = frame_system::Pallet::<T>::block_number();

			let mut campaign = Self::campaigns(id).ok_or(Error::<T>::CampaignIsNotFound)?;

			ensure!(who == campaign.creator, Error::<T>::NotCampaignCreator);

			ensure!(
				campaign.end + campaign.cooling_off_duration < now,
				Error::<T>::CampaignStillActive
			);

			let fund_account = Self::fund_account_id(id);
			match campaign.reward {
				RewardType::NftAssets(reward) => match campaign.claimed {
					RewardType::NftAssets(claimed) => {
						ensure!(
							reward.len().saturating_sub(claimed.len()) as u64 == left_nfts,
							Error::<T>::InvalidNftQuantity
						);
						T::Currency::transfer(&fund_account, &who, T::CampaignDeposit::get(), AllowDeath)?;

						for token in reward {
							if !claimed.contains(&token) {
								T::NFTHandler::set_lock_nft((token.0, token.1), false)?
							}
						}

						Self::reward_kill(campaign.trie_index, &who);
						Campaigns::<T>::remove(id);
						Self::deposit_event(Event::<T>::RewardCampaignClosed(id));
						let roots_vec = Self::campaign_merkle_roots(id);
						CampaignMerkleRoots::<T>::remove(id);
						match roots_vec.get(0) {
							Some(mekrle_root_ref) => {
								CampaignClaimIndexes::<T>::remove(id);
								Self::reward_kill_root(campaign.trie_index, mekrle_root_ref);
								Self::deposit_event(Event::<T>::RewardCampaignRootClosed(id));
							}
							_ => {}
						}
						Ok(())
					}
					_ => Err(Error::<T>::InvalidCampaignType.into()),
				},
				_ => Err(Error::<T>::InvalidCampaignType.into()),
			}
		}

		/// Cancel token-based campaign  
		///
		/// The dispatch origin for this call must be _Signed_. This extrinsic only works if the
		/// origin got admin privilege.
		/// - `campaign_id`: the ID of the campaign for which the rewards will be set.
		///
		/// Emits `RewardCampaignCanceled` if successful.
		#[pallet::weight(T::WeightInfo::cancel_campaign())]
		pub fn cancel_campaign(origin: OriginFor<T>, id: CampaignId) -> DispatchResult {
			T::AdminOrigin::ensure_origin(origin)?;
			let now = frame_system::Pallet::<T>::block_number();

			let mut campaign = Self::campaigns(id).ok_or(Error::<T>::CampaignIsNotFound)?;

			ensure!(campaign.end > now, Error::<T>::CampaignEnded);

			let fund_account = Self::fund_account_id(id);

			match campaign.reward {
				RewardType::FungibleTokens(c, r) => {
					T::FungibleTokenCurrency::transfer(c, &fund_account, &campaign.creator, r.saturated_into())?;
					T::Currency::transfer(&fund_account, &campaign.creator, T::CampaignDeposit::get(), AllowDeath)?;
					Campaigns::<T>::remove(id);
					if !Self::campaign_claim_indexes(id).is_empty() {
						CampaignClaimIndexes::<T>::remove(id);
					}
					Self::deposit_event(Event::<T>::RewardCampaignCanceled(id));
					Ok(())
				}
				_ => Err(Error::<T>::InvalidCampaignType.into()),
			}
		}

		/// Cancel NFT-based campaign  
		///
		/// The dispatch origin for this call must be _Signed_. This extrinsic only works if the
		/// origin got admin privilege.
		/// - `campaign_id`: the ID of the campaign for which the rewards will be set.
		/// - `left_nfts`: the size of the NFT reward pool.
		///
		/// Emits `RewardCampaignCanceled` if successful.
		#[pallet::weight(T::WeightInfo::cancel_nft_campaign() * (1u64 + left_nfts))]
		pub fn cancel_nft_campaign(origin: OriginFor<T>, id: CampaignId, left_nfts: u64) -> DispatchResult {
			T::AdminOrigin::ensure_origin(origin)?;
			let now = frame_system::Pallet::<T>::block_number();

			let mut campaign = Self::campaigns(id).ok_or(Error::<T>::CampaignIsNotFound)?;

			ensure!(campaign.end > now, Error::<T>::CampaignEnded);

			let fund_account = Self::fund_account_id(id);

			match campaign.reward {
				RewardType::NftAssets(reward) => {
					ensure!(reward.len() as u64 == left_nfts, Error::<T>::InvalidNftQuantity);
					T::Currency::transfer(&fund_account, &campaign.creator, T::CampaignDeposit::get(), AllowDeath)?;
					for token in reward {
						T::NFTHandler::set_lock_nft((token.0, token.1), false)?;
					}
					Campaigns::<T>::remove(id);
					if !Self::campaign_claim_indexes(id).is_empty() {
						CampaignClaimIndexes::<T>::remove(id);
					}
					Self::deposit_event(Event::<T>::RewardCampaignCanceled(id));
					Ok(().into())
				}
				_ => Err(Error::<T>::InvalidCampaignType.into()),
			}
		}

		/// Allow account to set rewards
		///
		/// The dispatch origin for this call must be _Signed_. This extrinsic only works if the
		/// origin got admin privilege.
		/// - `account`: the account which will be allowed to set rewards.
		///
		/// Emits `SetRewardOriginAdded` if successful.
		#[pallet::weight(T::WeightInfo::add_set_reward_origin())]
		pub fn add_set_reward_origin(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
			T::AdminOrigin::ensure_origin(origin)?;

			ensure!(
				!Self::is_set_reward_origin(&account),
				Error::<T>::SetRewardOriginAlreadyAdded
			);

			SetRewardOrigins::<T>::insert(account.clone(), ());

			Self::deposit_event(Event::<T>::SetRewardOriginAdded(account));

			Ok(())
		}

		/// Remove permission  to set rewards for a given account
		///
		/// The dispatch origin for this call must be _Signed_. This extrinsic only works if the
		/// origin got admin privilege.
		/// - `account`: the account which won';t be allowed to set rewards.
		///
		/// Emits `SetRewardOriginRemoved` if successful.
		#[pallet::weight(T::WeightInfo::remove_set_reward_origin())]
		pub fn remove_set_reward_origin(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
			T::AdminOrigin::ensure_origin(origin)?;

			ensure!(
				Self::is_set_reward_origin(&account),
				Error::<T>::SetRewardOriginDoesNotExist
			);

			SetRewardOrigins::<T>::remove(account.clone());

			Self::deposit_event(Event::<T>::SetRewardOriginRemoved(account));

			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		/// Hook that is called every time a new block is finalized.
		fn on_finalize(block_number: T::BlockNumber) {
			for (id, info) in Campaigns::<T>::iter()
				.filter(|(_, campaign_info)| campaign_info.end == block_number)
				.collect::<Vec<_>>()
			{
				Self::end_campaign(id);
			}
		}

		/// Hook that is called every time the runtime is upgraded.
		fn on_runtime_upgrade() -> Weight {
			Self::upgrade_campaign_info_v3();
			0
		}
	}
}

impl<T: Config> Pallet<T> {
	/// The account ID of the fund pot.
	///
	/// This actually does computation. If you need to keep using it, then make sure you cache the
	/// value and only call this once.
	pub fn fund_account_id(id: CampaignId) -> T::AccountId {
		T::PalletId::get().into_sub_account_truncating(id)
	}

	/// Generate unique ChildInfo IDs
	pub fn id_from_index(index: TrieIndex) -> child::ChildInfo {
		let mut buf = Vec::new();
		buf.extend_from_slice(b"bcreward");
		buf.extend_from_slice(&index.encode()[..]);
		child::ChildInfo::new_default(T::Hashing::hash(&buf[..]).as_ref())
	}

	/// Add non-merke root reward for token-based campaigns.
	pub fn reward_put(index: TrieIndex, who: &T::AccountId, balance: &BalanceOf<T>, memo: &[u8]) {
		who.using_encoded(|b| child::put(&Self::id_from_index(index), b, &(balance, memo)));
	}

	/// Add merke root reward for token-based campaigns.
	pub fn reward_put_root(index: TrieIndex, merkle_root: Hash, balance: &BalanceOf<T>, memo: &[u8]) {
		merkle_root.using_encoded(|b| child::put(&Self::id_from_index(index), b, &(balance, memo)));
	}

	/// Add non-merke root reward for NFT-based campaigns.
	pub fn reward_put_nft(index: TrieIndex, who: &T::AccountId, tokens: &Vec<(ClassId, TokenId)>, memo: &[u8]) {
		who.using_encoded(|b| child::put(&Self::id_from_index(index), b, &(tokens, memo)));
	}

	/// Add merke root reward for NFT-based campaigns.
	pub fn reward_put_nft_root(index: TrieIndex, merkle_root: Hash, tokens: &Vec<(ClassId, TokenId)>, memo: &[u8]) {
		merkle_root.using_encoded(|b| child::put(&Self::id_from_index(index), b, &(tokens, memo)));
	}

	/// Get the balance for an account rewarded in a token-based campaigns.
	pub fn reward_get(index: TrieIndex, who: &T::AccountId) -> (BalanceOf<T>, Vec<u8>) {
		who.using_encoded(|b| child::get_or_default::<(BalanceOf<T>, Vec<u8>)>(&Self::id_from_index(index), b))
	}

	/// Get a merkle root for a token-based campaigns.
	pub fn reward_get_root(index: TrieIndex, merkle_root: Hash) -> (BalanceOf<T>, Vec<u8>) {
		merkle_root.using_encoded(|b| child::get_or_default::<(BalanceOf<T>, Vec<u8>)>(&Self::id_from_index(index), b))
	}

	/// Get the balance for an account rewarded in a NFT-based campaigns.
	pub fn reward_get_nft(index: TrieIndex, who: &T::AccountId) -> (Vec<(ClassId, TokenId)>, Vec<u8>) {
		who.using_encoded(|b| {
			child::get_or_default::<(Vec<(ClassId, TokenId)>, Vec<u8>)>(&Self::id_from_index(index), b)
		})
	}

	/// Get the merkle root for a NFT-based campaigns.
	pub fn reward_get_nft_root(index: TrieIndex, merkle_root: Hash) -> (Vec<(ClassId, TokenId)>, Vec<u8>) {
		merkle_root.using_encoded(|b| {
			child::get_or_default::<(Vec<(ClassId, TokenId)>, Vec<u8>)>(&Self::id_from_index(index), b)
		})
	}

	/// Close a non-merkle proof based campaign.
	pub fn reward_kill(index: TrieIndex, who: &T::AccountId) {
		who.using_encoded(|b| child::kill(&Self::id_from_index(index), b));
	}

	/// Close a merkle proof based campaign.
	pub fn reward_kill_root(index: TrieIndex, merkle_root: &Hash) {
		merkle_root.using_encoded(|b| child::kill(&Self::id_from_index(index), b));
	}

	/// Child trie iterator for token-based campaign.
	pub fn campaign_reward_iterator(
		index: TrieIndex,
	) -> ChildTriePrefixIterator<(T::AccountId, (BalanceOf<T>, Vec<u8>))> {
		ChildTriePrefixIterator::<_>::with_prefix_over_key::<Identity>(&Self::id_from_index(index), &[])
	}

	/// Child trie iterator for NFT-based campaign.
	pub fn campaign_nft_reward_iterator(
		index: TrieIndex,
	) -> ChildTriePrefixIterator<(T::AccountId, (Vec<(ClassId, TokenId)>, Vec<u8>))> {
		ChildTriePrefixIterator::<_>::with_prefix_over_key::<Identity>(&Self::id_from_index(index), &[])
	}

	/// Internal calculation of a merkle proof for a token-based campaign.
	pub fn calculate_merkle_proof(
		claim_id: &ClaimId,
		balance: &BalanceOf<T>,
		leaf_nodes: &Vec<Hash>,
	) -> Result<Hash, DispatchError> {
		ensure!(
			leaf_nodes.len() as u64 <= T::MaxLeafNodes::get(),
			Error::<T>::InvalidRewardLeafAmount
		);

		// Hash the pair of AccountId and Balance
		let mut leaf: Vec<u8> = claim_id.encode();
		leaf.extend(balance.encode());

		Self::build_merkle_proof(leaf, leaf_nodes)
	}

	/// Internal calculation of the merkle proof for NFT-based campaign.
	pub fn calculate_nft_rewards_merkle_proof(
		claim_id: &ClaimId,
		tokens: &Vec<(ClassId, TokenId)>,
		leaf_nodes: &Vec<Hash>,
	) -> Result<Hash, DispatchError> {
		ensure!(
			leaf_nodes.len() as u64 <= T::MaxLeafNodes::get(),
			Error::<T>::InvalidRewardLeafAmount
		);

		// Hash the pair of AccountId and list of (ClassId, TokenId)
		let mut leaf: Vec<u8> = claim_id.encode();
		for token in tokens.clone() {
			leaf.extend(token.encode());
		}

		Self::build_merkle_proof(leaf, leaf_nodes)
	}

	/// Internal merkle proof calculation out of leaf node and vector of hashes of relevant leaf
	/// nodes and branches
	fn build_merkle_proof(raw_leaf: Vec<u8>, proof_nodes: &Vec<Hash>) -> Result<Hash, DispatchError> {
		let mut proof: Hash = keccak_256(&keccak_256(&raw_leaf)).into();

		for leaf_node in proof_nodes {
			proof = Self::sorted_hash_of(&proof, leaf_node);
		}

		Ok(proof)
	}

	/// Internal emit of end campaign event
	fn end_campaign(campaign_id: CampaignId) -> DispatchResult {
		Self::deposit_event(Event::<T>::RewardCampaignEnded(campaign_id));
		Ok(())
	}

	/// Internal check if an account is allowed to set rewards.
	pub fn is_set_reward_origin(who: &T::AccountId) -> bool {
		let set_reward_origin = Self::set_reward_origins(who);
		set_reward_origin == Some(())
	}

	/// Internal merkle hash calculation from two hashes
	pub fn sorted_hash_of(a: &Hash, b: &Hash) -> Hash {
		let mut h: Vec<u8> = Vec::with_capacity(64);

		if a < b {
			h.extend_from_slice(a.as_ref());
			h.extend_from_slice(b.as_ref());
		} else {
			h.extend_from_slice(b.as_ref());
			h.extend_from_slice(a.as_ref());
		}

		keccak_256(&h).into()
	}
	/*
		/// Internal update of campaign info to v2
		pub fn upgrade_campaign_info_v2() -> Weight {
			log::info!("Start upgrade_campaign_info_v2");
			let mut upgraded_campaign_items = 0;

			Campaigns::<T>::translate(
				|k, campaign_info_v1: CampaignInfoV1<T::AccountId, BalanceOf<T>, T::BlockNumber>| {
					upgraded_campaign_items += 1;

					let v2: CampaignInfo<T::AccountId, BalanceOf<T>, T::BlockNumber> = CampaignInfo {
						creator: campaign_info_v1.creator,
						properties: Vec::<u8>::new(),
						reward: campaign_info_v1.reward,
						claimed: campaign_info_v1.claimed,
						end: campaign_info_v1.end,
						cap: campaign_info_v1.cap,
						cooling_off_duration: campaign_info_v1.cooling_off_duration,
						trie_index: campaign_info_v1.trie_index,
					};
					Some(v2)
				},
			);
			log::info!("{} campaigns upgraded:", upgraded_campaign_items);
			0
		}
	*/

	/// Internal update of campaign info to v3
	pub fn upgrade_campaign_info_v3() -> Weight {
		log::info!("Start upgrade_campaign_info_v3");
		let mut upgraded_campaign_items = 0;

		Campaigns::<T>::translate(
			|k, campaign_info_v2: CampaignInfoV2<T::AccountId, BalanceOf<T>, T::BlockNumber>| {
				upgraded_campaign_items += 1;

				let v3_reward = RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), campaign_info_v2.reward);
				let v3_claimed = RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), campaign_info_v2.claimed);
				let v3_cap = RewardType::FungibleTokens(FungibleTokenId::NativeToken(0), campaign_info_v2.cap);

				let v3: CampaignInfo<T::AccountId, BalanceOf<T>, T::BlockNumber, FungibleTokenId, ClassId, TokenId> =
					CampaignInfo {
						creator: campaign_info_v2.creator,
						properties: campaign_info_v2.properties,
						end: campaign_info_v2.end,
						cooling_off_duration: campaign_info_v2.cooling_off_duration,
						trie_index: campaign_info_v2.trie_index,
						reward: v3_reward,
						claimed: v3_claimed,
						cap: v3_cap,
					};
				Some(v3)
			},
		);
		log::info!("{} campaigns upgraded:", upgraded_campaign_items);
		0
	}
}
