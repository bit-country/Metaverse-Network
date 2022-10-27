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
use primitives::{Balance, ClassId, FungibleTokenId, NftId};
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
	pub(super) type CampaignMerkleRoots<T: Config> = StorageMap<_, Twox64Concat, CampaignId, Vec<Hash>>;

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
		/// Set reward [campaign_id, account, balance]
		SetReward(CampaignId, T::AccountId, BalanceOf<T>),
		/// Set reward using merkle root [campaign_id, balance, hash]
		SetRewardRoot(CampaignId, BalanceOf<T>, Hash),
		/// Set NFT reward [campaign_id, account, asset]
		SetNftReward(CampaignId, T::AccountId, (ClassId, TokenId)),
		/// Set NFT rewards using merkle root[campaign_id, hash]
		SetNftRewardRoot(CampaignId, Hash),
		/// Reward campaign ended [campaign_id]
		RewardCampaignEnded(CampaignId),
		/// Reward campaign closed [campaign_id]
		RewardCampaignClosed(CampaignId),
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
		InvalidLeftNftQuantity,
		/// Invalid campaign type
		InvalidCampaignType,
		/// Cannot use genesis nft for reward
		CannotUseGenesisNftForReward,
		/// Reward is already set
		RewardAlreadySet,
		/// Reward leaf amount is larger then maximum
		InvalidRewardLeafAmount,
		/// Merkle root is not related to a campaign
		MerkleRootNotRelatedToCampaign,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
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

			NextTrieIndex::<T>::put(next_trie_index);
			NextCampaignId::<T>::put(next_campaign_id);

			Self::deposit_event(Event::<T>::NewRewardCampaignCreated(campaign_id, creator));

			Ok(())
		}

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

			ensure!(
				!reward.contains(&(0u32.into(), 0u64.into())),
				Error::<T>::CannotUseGenesisNftForReward
			);

			let trie_index = Self::next_trie_index();
			let campaign_id = Self::next_campaign_id();
			let fund_account = Self::fund_account_id(campaign_id);

			for token in reward.clone() {
				ensure!(
					T::NFTHandler::check_ownership(&creator, &(token.0, token.1))?,
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

			NextTrieIndex::<T>::put(next_trie_index);
			NextCampaignId::<T>::put(next_campaign_id);

			Self::deposit_event(Event::<T>::NewRewardCampaignCreated(campaign_id, creator));

			Ok(())
		}

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

		#[pallet::weight(T::WeightInfo::claim_reward_root()  * (1u64 + leaf_nodes.len() as u64))]
		pub fn claim_reward_root(
			origin: OriginFor<T>,
			id: CampaignId,
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

				match campaign.claimed {
					RewardType::FungibleTokens(c, r) => {
						let fund_account = Self::fund_account_id(id);
						let merkle_root = Self::calculate_merkle_proof(&who, &balance, &leaf_nodes)?;
						ensure!(
							Self::campaign_merkle_roots(id).is_some()
								&& Self::campaign_merkle_roots(id).unwrap().contains(&merkle_root),
							Error::<T>::MerkleRootNotRelatedToCampaign
						);
						let (root_balance, _) = Self::reward_get_root(campaign.trie_index, merkle_root.clone());
						// extra check in case the CampaignMerkleRoots storage is corrupted
						ensure!(root_balance > Zero::zero(), Error::<T>::NoRewardFound);
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

		#[pallet::weight(T::WeightInfo::claim_nft_reward())]
		pub fn claim_nft_reward(origin: OriginFor<T>, id: CampaignId) -> DispatchResult {
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
							let (token, _) = Self::reward_get_nft(campaign.trie_index, &who);
							ensure!(
								reward.contains(&token) && !claimed.contains(&token),
								Error::<T>::NoRewardFound
							);
							T::NFTHandler::set_lock_nft((token.0, token.1), false)?;
							T::NFTHandler::transfer_nft(&campaign.creator, &who, &token)?;

							let mut new_claimed = claimed.clone();
							new_claimed.push(token);
							campaign.claimed = RewardType::NftAssets(new_claimed);

							Self::reward_kill(campaign.trie_index, &who);

							let mut token_vec: Vec<(ClassId, TokenId)> = Vec::new();
							token_vec.push(token);
							Self::deposit_event(Event::<T>::NftRewardClaimed(id, who, token_vec));
							Ok(())
						}
						_ => Err(Error::<T>::InvalidCampaignType.into()),
					},
					_ => Err(Error::<T>::InvalidCampaignType.into()),
				}
			})?;
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::claim_nft_reward_root() * (1u64 + tokens.len() as u64))]
		#[transactional]
		pub fn claim_nft_reward_root(
			origin: OriginFor<T>,
			id: CampaignId,
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

				match campaign.reward.clone() {
					RewardType::NftAssets(reward) => match campaign.claimed.clone() {
						RewardType::NftAssets(claimed) => {
							let merkle_proof: Hash =
								Self::calculate_nft_rewards_merkle_proof(&who, &tokens, &leaf_nodes)?;

							ensure!(
								Self::campaign_merkle_roots(id).is_some()
									&& Self::campaign_merkle_roots(id).unwrap().contains(&merkle_proof),
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

		#[pallet::weight(T::WeightInfo::set_reward())]
		pub fn set_reward(
			origin: OriginFor<T>,
			id: CampaignId,
			to: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Self::is_set_reward_origin(&who), Error::<T>::InvalidSetRewardOrigin);

			let now = frame_system::Pallet::<T>::block_number();

			<Campaigns<T>>::try_mutate_exists(id, |campaign| -> DispatchResult {
				let mut campaign = campaign.as_mut().ok_or(Error::<T>::CampaignIsNotFound)?;

				ensure!(
					campaign.end + campaign.cooling_off_duration >= now,
					Error::<T>::CampaignExpired
				);

				match campaign.cap {
					RewardType::FungibleTokens(c, b) => {
						ensure!(b >= amount, Error::<T>::RewardExceedCap);
						campaign.cap = RewardType::FungibleTokens(c, b.saturating_sub(amount));
						Self::reward_put(campaign.trie_index, &to, &amount, &[]);
						Self::deposit_event(Event::<T>::SetReward(id, to, amount));
						Ok(())
					}
					_ => Err(Error::<T>::InvalidCampaignType.into()),
				}
			})?;
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::set_reward_root())]
		pub fn set_reward_root(
			origin: OriginFor<T>,
			id: CampaignId,
			total_amount: BalanceOf<T>,
			merkle_root: Hash,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Self::is_set_reward_origin(&who), Error::<T>::InvalidSetRewardOrigin);

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

						Self::reward_put_root(campaign.trie_index, merkle_root.clone(), &total_amount, &[]);
						Self::deposit_event(Event::<T>::SetRewardRoot(id, total_amount, merkle_root));
						Ok(())
					}
					_ => Err(Error::<T>::InvalidCampaignType.into()),
				}
			})?;
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::set_nft_reward())]
		pub fn set_nft_reward(origin: OriginFor<T>, id: CampaignId, to: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Self::is_set_reward_origin(&who), Error::<T>::InvalidSetRewardOrigin);

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
						let token = new_cap.pop().ok_or(Error::<T>::RewardExceedCap)?;
						Self::reward_put_nft(campaign.trie_index, &to, &token, &[]);
						campaign.cap = RewardType::NftAssets(new_cap);
						Self::deposit_event(Event::<T>::SetNftReward(id, to, token));
						Ok(())
					}
					_ => Err(Error::<T>::InvalidCampaignType.into()),
				}
			})?;
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::set_nft_reward_root())]
		pub fn set_nft_reward_root(origin: OriginFor<T>, id: CampaignId, merkle_root: Hash) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Self::is_set_reward_origin(&who), Error::<T>::InvalidSetRewardOrigin);

			let now = frame_system::Pallet::<T>::block_number();

			<Campaigns<T>>::try_mutate_exists(id, |campaign| -> DispatchResult {
				let mut campaign = campaign.as_mut().ok_or(Error::<T>::CampaignIsNotFound)?;

				ensure!(
					campaign.end + campaign.cooling_off_duration >= now,
					Error::<T>::CampaignExpired
				);

				match campaign.cap.clone() {
					RewardType::NftAssets(cap) => {
						ensure!(Self::campaign_merkle_roots(id) == None, Error::<T>::RewardAlreadySet);

						ensure!(!cap.is_empty(), Error::<T>::RewardExceedCap);

						Self::reward_put_nft_root(campaign.trie_index, merkle_root, &cap, &[]);

						let mut merkle_roots_vec: Vec<Hash> = Vec::new();
						merkle_roots_vec.push(merkle_root);
						<CampaignMerkleRoots<T>>::insert(id, merkle_roots_vec);

						campaign.cap = RewardType::NftAssets(Vec::new());
						Self::deposit_event(Event::<T>::SetNftRewardRoot(id, merkle_root));
						Ok(())
					}
					_ => Err(Error::<T>::InvalidCampaignType.into()),
				}
			})?;
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::close_campaign())]
		pub fn close_campaign(origin: OriginFor<T>, id: CampaignId) -> DispatchResult {
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
						CampaignMerkleRoots::<T>::remove(id);
						Self::deposit_event(Event::<T>::RewardCampaignClosed(id));
						Ok(())
					}
					_ => Err(Error::<T>::InvalidCampaignType.into()),
				},
				_ => Err(Error::<T>::InvalidCampaignType.into()),
			}
		}

		#[pallet::weight(T::WeightInfo::close_nft_campaign() * (1u64 + left_nfts))]
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
							Error::<T>::InvalidLeftNftQuantity
						);
						T::Currency::transfer(&fund_account, &who, T::CampaignDeposit::get(), AllowDeath)?;

						for token in reward {
							if !claimed.contains(&token) {
								T::NFTHandler::set_lock_nft((token.0, token.1), false)?
							}
						}

						Self::reward_kill(campaign.trie_index, &who);
						Campaigns::<T>::remove(id);
						CampaignMerkleRoots::<T>::remove(id);
						Self::deposit_event(Event::<T>::RewardCampaignClosed(id));
						Ok(())
					}
					_ => Err(Error::<T>::InvalidCampaignType.into()),
				},
				_ => Err(Error::<T>::InvalidCampaignType.into()),
			}
		}

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
					Self::deposit_event(Event::<T>::RewardCampaignCanceled(id));
					Ok(())
				}
				_ => Err(Error::<T>::InvalidCampaignType.into()),
			}
		}

		#[pallet::weight(T::WeightInfo::cancel_nft_campaign() * (1u64 + left_nfts))]
		pub fn cancel_nft_campaign(origin: OriginFor<T>, id: CampaignId, left_nfts: u64) -> DispatchResult {
			T::AdminOrigin::ensure_origin(origin)?;
			let now = frame_system::Pallet::<T>::block_number();

			let mut campaign = Self::campaigns(id).ok_or(Error::<T>::CampaignIsNotFound)?;

			ensure!(campaign.end > now, Error::<T>::CampaignEnded);

			let fund_account = Self::fund_account_id(id);

			match campaign.reward {
				RewardType::NftAssets(reward) => {
					ensure!(reward.len() as u64 == left_nfts, Error::<T>::InvalidLeftNftQuantity);
					T::Currency::transfer(&fund_account, &campaign.creator, T::CampaignDeposit::get(), AllowDeath)?;
					for token in reward {
						T::NFTHandler::set_lock_nft((token.0, token.1), false)?;
					}
					Campaigns::<T>::remove(id);
					Self::deposit_event(Event::<T>::RewardCampaignCanceled(id));
					Ok(().into())
				}
				_ => Err(Error::<T>::InvalidCampaignType.into()),
			}
		}

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
		fn on_finalize(block_number: T::BlockNumber) {
			for (id, info) in Campaigns::<T>::iter()
				.filter(|(_, campaign_info)| campaign_info.end == block_number)
				.collect::<Vec<_>>()
			{
				Self::end_campaign(id);
			}
		}

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

	pub fn id_from_index(index: TrieIndex) -> child::ChildInfo {
		let mut buf = Vec::new();
		buf.extend_from_slice(b"bcreward");
		buf.extend_from_slice(&index.encode()[..]);
		child::ChildInfo::new_default(T::Hashing::hash(&buf[..]).as_ref())
	}

	pub fn reward_put(index: TrieIndex, who: &T::AccountId, balance: &BalanceOf<T>, memo: &[u8]) {
		who.using_encoded(|b| child::put(&Self::id_from_index(index), b, &(balance, memo)));
	}

	pub fn reward_put_root(index: TrieIndex, merkle_root: Hash, balance: &BalanceOf<T>, memo: &[u8]) {
		merkle_root.using_encoded(|b| child::put(&Self::id_from_index(index), b, &(balance, memo)));
	}

	pub fn reward_put_nft(index: TrieIndex, who: &T::AccountId, token: &(ClassId, TokenId), memo: &[u8]) {
		who.using_encoded(|b| child::put(&Self::id_from_index(index), b, &(token, memo)));
	}

	pub fn reward_put_nft_root(index: TrieIndex, merkle_root: Hash, tokens: &Vec<(ClassId, TokenId)>, memo: &[u8]) {
		merkle_root.using_encoded(|b| child::put(&Self::id_from_index(index), b, &(tokens, memo)));
	}

	pub fn reward_get(index: TrieIndex, who: &T::AccountId) -> (BalanceOf<T>, Vec<u8>) {
		who.using_encoded(|b| child::get_or_default::<(BalanceOf<T>, Vec<u8>)>(&Self::id_from_index(index), b))
	}

	pub fn reward_get_root(index: TrieIndex, merkle_root: Hash) -> (BalanceOf<T>, Vec<u8>) {
		merkle_root.using_encoded(|b| child::get_or_default::<(BalanceOf<T>, Vec<u8>)>(&Self::id_from_index(index), b))
	}

	pub fn reward_get_nft(index: TrieIndex, who: &T::AccountId) -> ((ClassId, TokenId), Vec<u8>) {
		who.using_encoded(|b| child::get_or_default::<((ClassId, TokenId), Vec<u8>)>(&Self::id_from_index(index), b))
	}

	pub fn reward_get_nft_root(index: TrieIndex, merkle_root: Hash) -> (Vec<(ClassId, TokenId)>, Vec<u8>) {
		merkle_root.using_encoded(|b| {
			child::get_or_default::<(Vec<(ClassId, TokenId)>, Vec<u8>)>(&Self::id_from_index(index), b)
		})
	}

	pub fn reward_kill(index: TrieIndex, who: &T::AccountId) {
		who.using_encoded(|b| child::kill(&Self::id_from_index(index), b));
	}

	pub fn reward_kill_root(index: TrieIndex, merkle_root: Hash) {
		merkle_root.using_encoded(|b| child::kill(&Self::id_from_index(index), b));
	}

	pub fn campaign_reward_iterator(
		index: TrieIndex,
	) -> ChildTriePrefixIterator<(T::AccountId, (BalanceOf<T>, Vec<u8>))> {
		ChildTriePrefixIterator::<_>::with_prefix_over_key::<Identity>(&Self::id_from_index(index), &[])
	}

	pub fn campaign_nft_reward_iterator(
		index: TrieIndex,
	) -> ChildTriePrefixIterator<(T::AccountId, ((ClassId, TokenId), Vec<u8>))> {
		ChildTriePrefixIterator::<_>::with_prefix_over_key::<Identity>(&Self::id_from_index(index), &[])
	}

	pub fn calculate_merkle_proof(
		who: &T::AccountId,
		balance: &BalanceOf<T>,
		leaf_nodes: &Vec<Hash>,
	) -> Result<Hash, DispatchError> {
		ensure!(
			leaf_nodes.len() as u64 <= T::MaxLeafNodes::get(),
			Error::<T>::InvalidRewardLeafAmount
		);

		// Hash the pair of AccountId and Balance
		let mut leaf: Vec<u8> = who.encode();
		leaf.extend(balance.encode());

		let leaf_hash: Hash = keccak_256(&leaf).into();

		leaf_nodes.iter().fold(leaf_hash.clone(), |acc, hash| {
			Self::sorted_hash_of(&Hash::from_slice(acc.as_ref()), hash)
		});
		Ok(leaf_hash)
	}

	pub fn calculate_nft_rewards_merkle_proof(
		who: &T::AccountId,
		tokens: &Vec<(ClassId, TokenId)>,
		leaf_nodes: &Vec<Hash>,
	) -> Result<Hash, DispatchError> {
		ensure!(
			leaf_nodes.len() as u64 <= T::MaxLeafNodes::get(),
			Error::<T>::InvalidRewardLeafAmount
		);

		// Hash the pair of AccountId and list of (ClassId, TokenId)
		let mut leaf: Vec<u8> = who.encode();
		for token in tokens.clone() {
			leaf.extend(token.0.encode());
			leaf.extend(token.1.encode());
		}

		let leaf_hash: Hash = keccak_256(&leaf).into();

		leaf_nodes.iter().fold(leaf_hash.clone(), |acc, hash| {
			Self::sorted_hash_of(&Hash::from_slice(acc.as_ref()), hash)
		});
		Ok(leaf_hash)
	}

	fn end_campaign(campaign_id: CampaignId) -> DispatchResult {
		Self::deposit_event(Event::<T>::RewardCampaignEnded(campaign_id));
		Ok(())
	}

	pub fn is_set_reward_origin(who: &T::AccountId) -> bool {
		let set_reward_origin = Self::set_reward_origins(who);
		set_reward_origin == Some(())
	}

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
