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
use sp_runtime::traits::{BlockNumberProvider, CheckedAdd, CheckedMul, Hash, Saturating};
use sp_runtime::{
	traits::{AccountIdConversion, One, Zero},
	ArithmeticError, DispatchError, Perbill,
};
use sp_std::{collections::btree_map::BTreeMap, prelude::*, vec::Vec};

use core_primitives::NFTTrait;
use core_primitives::*;
pub use pallet::*;
use primitives::{estate::Estate, CampaignId, EstateId, TrieIndex};
use primitives::{AssetId, Balance, ClassId, DomainId, FungibleTokenId, MetaverseId, NftId, PowerAmount, RoundIndex};
pub use weights::WeightInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::traits::ExistenceRequirement::AllowDeath;
	use orml_traits::MultiCurrencyExtended;
	use sp_runtime::traits::{CheckedAdd, CheckedSub, Saturating};
	use sp_runtime::ArithmeticError;

	use primitives::staking::RoundInfo;
	use primitives::{CampaignId, CampaignInfo, ClassId, GroupCollectionId, NftId};

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

		/// Weight info
		type WeightInfo: WeightInfo;
	}

	/// Info of campaign.
	#[pallet::storage]
	#[pallet::getter(fn campaigns)]
	pub(super) type Campaigns<T: Config> =
		StorageMap<_, Twox64Concat, CampaignId, CampaignInfo<T::AccountId, BalanceOf<T>, T::BlockNumber>>;

	/// Tracker for the next available trie index
	#[pallet::storage]
	#[pallet::getter(fn next_trie_index)]
	pub(super) type NextTrieIndex<T> = StorageValue<_, u32, ValueQuery>;

	/// Tracker for the next available campaign index
	#[pallet::storage]
	#[pallet::getter(fn next_campaign_id)]
	pub(super) type NextCampaignId<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New campaign created [campaign_id, account]
		NewRewardCampaignCreated(CampaignId, T::AccountId),
		/// Reward claimed [campaign_id, account, balance]
		RewardClaimed(CampaignId, T::AccountId, BalanceOf<T>),
		/// Set Reward [campaign_id, account, balance]
		SetReward(CampaignId, T::AccountId, BalanceOf<T>),
		/// New campaign ended [campaign_id]
		RewardCampaignEnded(CampaignId),
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
		/// Campaign end block is before the current block
		CampaignEndBlockBeforeCurrentBlock,
		/// Campaign reward pool is below the set minimum
		RewardPoolBelowMinimum,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::unstake_b())]
		pub fn create_campaign(
			origin: OriginFor<T>,
			creator: T::AccountId,
			reward: BalanceOf<T>,
			end: T::BlockNumber,
		) -> DispatchResult {
			let depositor = ensure_signed(origin)?;
			let now = frame_system::Pallet::<T>::block_number();

			ensure!(now < end, Error::<T>::CampaignEndBlockBeforeCurrentBlock);

			ensure!(
				reward >= T::MinimumRewardPool::get(),
				Error::<T>::RewardPoolBelowMinimum
			);

			let trie_index = Self::next_trie_index();
			let next_trie_index = trie_index.checked_add(1).ok_or(ArithmeticError::Overflow)?;

			let deposit = T::CampaignDeposit::get();

			let campaign_id = Self::next_campaign_id();

			let fund_account = Self::fund_account_id(campaign_id);
			T::Currency::transfer(&depositor, &fund_account, deposit, AllowDeath)?;

			let next_campaign_id = campaign_id.checked_add(1).ok_or(ArithmeticError::Overflow)?;

			Campaigns::<T>::insert(
				campaign_id,
				CampaignInfo {
					creator: creator.clone(),
					reward,
					claimed: Zero::zero(),
					end,
					cap: reward,
					trie_index,
				},
			);

			NextTrieIndex::<T>::put(next_trie_index);
			NextCampaignId::<T>::put(next_campaign_id);

			Self::deposit_event(Event::<T>::NewRewardCampaignCreated(next_campaign_id, creator));

			Ok(())
		}

		#[pallet::weight(T::WeightInfo::unstake_b())]
		pub fn claim_reward(origin: OriginFor<T>, id: CampaignId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let now = frame_system::Pallet::<T>::block_number();

			let mut campaign = Self::campaigns(id).ok_or(Error::<T>::CampaignIsNotFound)?;
			let fund_account = Self::fund_account_id(id);

			let (balance, _) = Self::reward_get(campaign.trie_index, &who);
			ensure!(balance > Zero::zero(), Error::<T>::NoRewardFound);

			T::Currency::transfer(&fund_account, &who, balance, AllowDeath)?;

			Self::reward_kill(campaign.trie_index, &who);
			campaign.claimed = campaign.claimed.saturating_add(balance);

			Campaigns::<T>::insert(id, &campaign);
			Self::deposit_event(Event::<T>::RewardClaimed(id, who, balance));

			Ok(())
		}

		#[pallet::weight(T::WeightInfo::unstake_b())]
		pub fn set_reward(
			origin: OriginFor<T>,
			id: CampaignId,
			to: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			// Ensure root for now
			// We need to move to SetRewardOrigin similar to MiningOrigin
			let who = ensure_root(origin)?;
			let now = frame_system::Pallet::<T>::block_number();

			let mut campaign = Self::campaigns(id).ok_or(Error::<T>::CampaignIsNotFound)?;

			ensure!(campaign.end > now, Error::<T>::CampaignExpired);

			ensure!(amount < campaign.cap, Error::<T>::RewardExceedCap);

			Self::reward_put(campaign.trie_index, &to, &amount, &[]);

			Self::deposit_event(Event::<T>::SetReward(id, to, amount));

			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_finalize(block_number: T::BlockNumber) {
			for (id, info) in Campaigns::<T>::iter()
				.filter(|(_, campaign_info)| campaign_info.end <= block_number)
				.collect::<Vec<_>>()
			{
				Self::end_campaign(id);
			}
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

	pub fn reward_get(index: TrieIndex, who: &T::AccountId) -> (BalanceOf<T>, Vec<u8>) {
		who.using_encoded(|b| child::get_or_default::<(BalanceOf<T>, Vec<u8>)>(&Self::id_from_index(index), b))
	}

	pub fn reward_kill(index: TrieIndex, who: &T::AccountId) {
		who.using_encoded(|b| child::kill(&Self::id_from_index(index), b));
	}

	pub fn campaign_reward_iterator(
		index: TrieIndex,
	) -> ChildTriePrefixIterator<(T::AccountId, (BalanceOf<T>, Vec<u8>))> {
		ChildTriePrefixIterator::<_>::with_prefix_over_key::<Identity>(&Self::id_from_index(index), &[])
	}

	fn end_campaign(campaign_id: CampaignId) -> DispatchResult {
		Self::deposit_event(Event::<T>::RewardCampaignEnded(campaign_id));
		Ok(())
	}
}