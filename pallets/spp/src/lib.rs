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

use frame_support::pallet_prelude::*;
use frame_support::{
	dispatch::DispatchResult,
	ensure,
	traits::{Currency, Get},
	transactional, PalletId,
};
use frame_system::ensure_signed;
use frame_system::pallet_prelude::*;
use orml_traits::MultiCurrency;
use sp_runtime::traits::{BlockNumberProvider, CheckedAdd, CheckedDiv, CheckedSub};
use sp_runtime::{
	traits::{AccountIdConversion, Convert, Saturating, Zero},
	ArithmeticError, DispatchError, SaturatedConversion,
};

use core_primitives::*;
pub use pallet::*;
use primitives::{ClassId, EraIndex, FungibleTokenId, StakingRound, TokenId};
pub use weights::WeightInfo;

pub type QueueId = u32;
//#[cfg(feature = "runtime-benchmarks")]
//pub mod benchmarking;

#[cfg(test)]
mod mock;
mod utils;

#[cfg(test)]
mod tests;

pub mod weights;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::traits::{Currency, LockableCurrency, ReservableCurrency};
	use orml_traits::{MultiCurrency, MultiReservableCurrency};
	use sp_core::U256;
	use sp_runtime::traits::{BlockNumberProvider, CheckedAdd, CheckedMul, CheckedSub, UniqueSaturatedInto};
	use sp_runtime::Permill;

	use primitives::{PoolId, StakingRound};

	use crate::utils::{BoostInfo, BoostingRecord, PoolInfo};

	use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ orml_rewards::Config<
			Share = BalanceOf<Self>,
			Balance = BalanceOf<Self>,
			PoolId = PoolId,
			CurrencyId = FungibleTokenId,
		>
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Currency type
		type Currency: Currency<Self::AccountId>
			+ ReservableCurrency<Self::AccountId>
			+ LockableCurrency<Self::AccountId>;
		/// Multi currencies type that handles different currency type in auction
		type MultiCurrency: MultiReservableCurrency<
			Self::AccountId,
			CurrencyId = FungibleTokenId,
			Balance = BalanceOf<Self>,
		>;

		/// Weight implementation for estate extrinsics
		type WeightInfo: WeightInfo;

		/// Minimum staking balance
		#[pallet::constant]
		type MinimumStake: Get<BalanceOf<Self>>;

		/// Delay of staking reward payment (in number of rounds)
		#[pallet::constant]
		type RewardPaymentDelay: Get<u32>;

		/// Default max bound for each metaverse mapping system, this could change through proposal
		type DefaultMaxBound: Get<(i32, i32)>;

		/// Network fee charged on pool creation
		#[pallet::constant]
		type NetworkFee: Get<BalanceOf<Self>>;

		/// Storage deposit free charged when saving data into the blockchain.
		/// The fee will be unreserved after the storage is freed.
		#[pallet::constant]
		type StorageDepositFee: Get<BalanceOf<Self>>;

		/// Allows converting block numbers into balance
		type BlockNumberToBalance: Convert<Self::BlockNumber, BalanceOf<Self>>;

		/// Block number provider for the relaychain.
		type RelayChainBlockNumber: BlockNumberProvider<BlockNumber = BlockNumberFor<Self>>;

		#[pallet::constant]
		type PoolAccount: Get<PalletId>;

		#[pallet::constant]
		type MaximumQueue: Get<u32>;

		type CurrencyIdConversion: CurrencyIdManagement;

		/// Origin represented Governance
		type GovernanceOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;
	}

	pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::storage]
	#[pallet::getter(fn next_class_id)]
	pub type NextPoolId<T: Config> = StorageValue<_, PoolId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn fees)]
	pub type Fees<T: Config> = StorageValue<_, (Permill, Permill), ValueQuery>;

	/// Keep track of Pool detail
	#[pallet::storage]
	#[pallet::getter(fn pool)]
	pub type Pool<T: Config> = StorageMap<_, Twox64Concat, PoolId, PoolInfo<T::AccountId>, OptionQuery>;

	/// Pool ledger that keeps track of Pool id and balance of base currency
	#[pallet::storage]
	#[pallet::getter(fn pool_ledger)]
	pub type PoolLedger<T: Config> = StorageMap<_, Twox64Concat, PoolId, BalanceOf<T>, ValueQuery>;

	/// Network ledger that keep track of all staking across all pools
	#[pallet::storage]
	#[pallet::getter(fn network_ledger)]
	pub type NetworkLedger<T: Config> = StorageMap<_, Twox64Concat, FungibleTokenId, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn minimum_redeem)]
	pub type MinimumRedeem<T: Config> = StorageMap<_, Twox64Concat, FungibleTokenId, BalanceOf<T>, ValueQuery>;

	/// Keep track of each staking round, how many items in queue need to be redeem
	#[pallet::storage]
	#[pallet::getter(fn staking_round_redeem_requests)]
	pub type StakingRoundRedeemQueue<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		StakingRound,
		Blake2_128Concat,
		FungibleTokenId,
		(BalanceOf<T>, BoundedVec<QueueId, T::MaximumQueue>, FungibleTokenId),
		OptionQuery,
	>;

	/// Keep track of user ledger that how many queue items that needs to be unlocked
	#[pallet::storage]
	#[pallet::getter(fn user_redeem_requests)]
	pub type UserCurrencyRedeemQueue<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		FungibleTokenId,
		(BalanceOf<T>, BoundedVec<QueueId, T::MaximumQueue>),
		OptionQuery,
	>;

	/// Keep track of queue item as well as account that locked amount of currency can be redeemed
	#[pallet::storage]
	#[pallet::getter(fn currency_redeem_requests)]
	pub type CurrencyRedeemQueue<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		FungibleTokenId,
		Blake2_128Concat,
		QueueId,
		(T::AccountId, BalanceOf<T>, StakingRound),
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn unlock_duration)]
	pub type UnlockDuration<T: Config> = StorageMap<_, Twox64Concat, FungibleTokenId, StakingRound>;

	#[pallet::storage]
	#[pallet::getter(fn current_staking_round)]
	pub type CurrentStakingRound<T: Config> = StorageMap<_, Twox64Concat, FungibleTokenId, StakingRound>;

	/// The current era of relaychain
	///
	/// RelayChainCurrentEra : EraIndex
	#[pallet::storage]
	#[pallet::getter(fn relay_chain_current_era)]
	pub type RelayChainCurrentEra<T: Config> = StorageValue<_, EraIndex, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn last_staking_round)]
	pub type LastStakingRound<T: Config> = StorageMap<_, Twox64Concat, FungibleTokenId, StakingRound, ValueQuery>;

	/// The relaychain block number of last staking round
	#[pallet::storage]
	#[pallet::getter(fn last_era_updated_block)]
	pub type LastEraUpdatedBlock<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

	/// The internal of relaychain block number between era.
	#[pallet::storage]
	#[pallet::getter(fn update_era_frequency)]
	pub type UpdateEraFrequency<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn queue_next_id)]
	pub type QueueNextId<T: Config> = StorageMap<_, Twox64Concat, FungibleTokenId, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn iteration_limit)]
	pub type IterationLimit<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn boosting_record)]
	/// Store boosting records for each account
	pub type BoostingOf<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, BoostingRecord<BalanceOf<T>, T::BlockNumber>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New pool created
		PoolCreated {
			from: T::AccountId,
			pool_id: PoolId,
			currency_id: FungibleTokenId,
		},
		/// Deposited
		Deposited {
			from: T::AccountId,
			pool_id: PoolId,
			amount: BalanceOf<T>,
		},
		/// Redeemed
		Redeemed {
			from: T::AccountId,
			pool_id: PoolId,
			amount: BalanceOf<T>,
		},
		/// Redeemed success
		RedeemSuccess {
			queue_id: QueueId,
			currency_id: FungibleTokenId,
			to: T::AccountId,
			token_amount: BalanceOf<T>,
		},
		/// Current era updated
		CurrentEraUpdated { new_era_index: EraIndex },
		/// Last era updated
		LastEraUpdated { last_era_block: BlockNumberFor<T> },
		/// Update era frequency
		UpdateEraFrequency { frequency: BlockNumberFor<T> },
		/// Boosted successful
		Boosted {
			booster: T::AccountId,
			pool_id: PoolId,
			boost_info: BoostInfo<BalanceOf<T>>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// No permission
		NoPermission,
		/// Currency is not supported
		CurrencyIsNotSupported,
		/// No available next pool id
		NoAvailablePoolId,
		/// Pool doesn't exists
		PoolDoesNotExist,
		/// Overflow
		Overflow,
		/// Below minimum redemption
		BelowMinimumRedeem,
		/// No current staking round
		NoCurrentStakingRound,
		/// Unexpected
		Unexpected,
		/// Too many redeems
		TooManyRedeems,
		/// Arthimetic Overflow
		ArithmeticOverflow,
		/// Token type is not supported
		NotSupportTokenType,
		/// Unlock duration not found
		UnlockDurationNotFound,
		/// Staking round not found
		StakingRoundNotFound,
		/// Staking round redeem queue not found
		StakingRoundRedeemNotFound,
		/// User currency redeem queue not found
		UserCurrencyRedeemQueueNotFound,
		/// Redeem queue per currency not found
		CurrencyRedeemQueueNotFound,
		/// The last era updated block is invalid
		InvalidLastEraUpdatedBlock,
		/// Fail to process redeem requests
		FailedToProcessRedemption,
		/// Insufficient Fund
		InsufficientFund,
		/// Error while adding new boost
		MaxVotesReached,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
			let era_number = Self::get_era_index(T::RelayChainBlockNumber::current_block_number());
			if !era_number.is_zero() {
				let _ = Self::update_current_era(era_number).map_err(|err| err).ok();
			}

			T::WeightInfo::on_initialize()
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::mint_land())]
		pub fn create_pool(
			origin: OriginFor<T>,
			currency_id: FungibleTokenId,
			max_nft_reward: u32,
			commission: Permill,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Ensure currency_id supported
			ensure!(
				currency_id == FungibleTokenId::NativeToken(0) || currency_id == FungibleTokenId::NativeToken(1),
				Error::<T>::CurrencyIsNotSupported
			);

			// TODO Check commission below threshold

			// Collect pool creation fee
			Self::collect_pool_creation_fee(&who, currency_id)?;

			// Next pool id
			let next_pool_id = NextPoolId::<T>::try_mutate(|id| -> Result<PoolId, DispatchError> {
				let current_id = *id;
				*id = id.checked_add(1u32).ok_or(Error::<T>::NoAvailablePoolId)?;
				Ok(current_id)
			})?;

			let new_pool = PoolInfo {
				creator: who.clone(),
				commission,
				currency_id,
				max: max_nft_reward,
			};

			// Add tuple class_id, currency_id
			Pool::<T>::insert(next_pool_id, new_pool);

			// Emit event for pool creation
			Self::deposit_event(Event::PoolCreated {
				from: who,
				pool_id: next_pool_id,
				currency_id,
			});
			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::mint_land())]
		pub fn deposit(origin: OriginFor<T>, pool_id: PoolId, amount: BalanceOf<T>) -> DispatchResult {
			// Ensure user is signed
			let who = ensure_signed(origin)?;
			// Check if pool exists
			let pool_instance = Pool::<T>::get(pool_id).ok_or(Error::<T>::PoolDoesNotExist)?;

			// Get currencyId from pool detail
			let currency_id = pool_instance.currency_id;

			// Get network ledger balance from currency id
			let network_ledger_balance = Self::network_ledger(currency_id);

			// Collect deposit fee for protocol
			// Assuming there's a function `collect_deposit_fee` that deducts a fee for deposits.
			let amount_after_fee = Self::collect_deposit_fee(&who, currency_id, amount)?;

			let r_currency_id = T::CurrencyIdConversion::convert_to_rcurrency(currency_id)
				.map_err(|_| Error::<T>::CurrencyIsNotSupported)?;
			// Calculate rAmount as receipt of amount locked. The formula based on rAmount = (amount * rAmount
			// total issuance)/network ledger balance
			let r_amount_total_issuance = T::MultiCurrency::total_issuance(r_currency_id);
			let mut r_amount = amount_after_fee;
			// This will apply for subsequence deposits - the first deposit r_amount = amount_after_fee
			if network_ledger_balance != BalanceOf::<T>::zero() {
				r_amount = U256::from(amount_after_fee.saturated_into::<u128>())
					.saturating_mul(r_amount_total_issuance.saturated_into::<u128>().into())
					.checked_div(network_ledger_balance.saturated_into::<u128>().into())
					.ok_or(Error::<T>::ArithmeticOverflow)?
					.as_u128()
					.saturated_into();
			}

			// Deposit rAmount to user using T::MultiCurrency::deposit
			T::MultiCurrency::deposit(r_currency_id, &who, r_amount)?;

			// Update this specific pool ledger to keep track of pool balance
			PoolLedger::<T>::mutate(&pool_id, |pool| -> Result<(), Error<T>> {
				*pool = pool
					.checked_add(&amount_after_fee)
					.ok_or(Error::<T>::ArithmeticOverflow)?;
				Ok(())
			})?;

			NetworkLedger::<T>::mutate(&currency_id, |pool| -> Result<(), Error<T>> {
				*pool = pool
					.checked_add(&amount_after_fee)
					.ok_or(Error::<T>::ArithmeticOverflow)?;
				Ok(())
			})?;
			// Transfer amount to PoolAccount using T::MultiCurrency::transfer
			// Assuming `PoolAccount` is an associated type that represents the pool's account ID or a method to
			// get it.
			T::MultiCurrency::transfer(
				currency_id,
				&who,
				&T::PoolAccount::get().into_account_truncating(),
				amount_after_fee,
			)?;

			// Emit deposit event
			Self::deposit_event(Event::Deposited {
				from: who,
				pool_id,
				amount,
			});
			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::mint_land())]
		pub fn redeem(
			origin: OriginFor<T>,
			pool_id: PoolId,
			v_currency_id: FungibleTokenId,
			r_amount: BalanceOf<T>,
		) -> DispatchResult {
			// Ensure user is signed
			let who = ensure_signed(origin)?;
			ensure!(
				r_amount >= MinimumRedeem::<T>::get(v_currency_id),
				Error::<T>::BelowMinimumRedeem
			);

			let currency_id = T::CurrencyIdConversion::convert_to_currency(v_currency_id)
				.map_err(|_| Error::<T>::NotSupportTokenType)?;

			// Check if pool exists
			let pool_instance = Pool::<T>::get(pool_id).ok_or(Error::<T>::PoolDoesNotExist)?;

			ensure!(
				currency_id == pool_instance.currency_id,
				Error::<T>::CurrencyIsNotSupported
			);

			// Get network ledger balance from currency id
			let network_ledger_balance = Self::network_ledger(currency_id);

			// Collect deposit fee for protocol
			let amount_after_fee = Self::collect_redeem_fee(&who, v_currency_id, r_amount)?;
			let r_amount = amount_after_fee;
			// Calculate rAmount as receipt of amount locked. The formula based on rAmount = (amount * rAmount
			// total issuance)/network ledger balance
			let r_amount_total_issuance = T::MultiCurrency::total_issuance(v_currency_id);
			let currency_amount = U256::from(r_amount.saturated_into::<u128>())
				.saturating_mul(network_ledger_balance.saturated_into::<u128>().into())
				.checked_div(r_amount_total_issuance.saturated_into::<u128>().into())
				.ok_or(Error::<T>::ArithmeticOverflow)?
				.as_u128()
				.saturated_into();

			// Check current staking era - only failed when there is no current staking era
			// Staking era get checked and updated every blocks
			match CurrentStakingRound::<T>::get(currency_id) {
				Some(staking_round) => {
					// Calculate the staking duration to be locked
					let new_staking_round = Self::calculate_next_staking_round(
						Self::unlock_duration(currency_id).ok_or(Error::<T>::UnlockDurationNotFound)?,
						staking_round,
					)?;
					// Burn currency
					T::MultiCurrency::withdraw(v_currency_id, &who, amount_after_fee)?;

					// Update pool ledger
					PoolLedger::<T>::mutate(&pool_id, |pool| -> Result<(), Error<T>> {
						*pool = pool
							.checked_sub(&currency_amount)
							.ok_or(Error::<T>::ArithmeticOverflow)?;
						Ok(())
					})?;

					// Get current queue_id
					let next_queue_id = Self::queue_next_id(currency_id);

					// Add request into network currency redeem queue
					CurrencyRedeemQueue::<T>::insert(
						&currency_id,
						&next_queue_id,
						(&who, currency_amount, &new_staking_round),
					);

					// Handle ledger of user and currency - user,currency: total_amount_unlocked, vec![queue_id]
					// Check if you already has any redeem requests
					if UserCurrencyRedeemQueue::<T>::get(&who, &currency_id).is_some() {
						// Add new queue id into the list
						UserCurrencyRedeemQueue::<T>::mutate(&who, &currency_id, |value| -> Result<(), Error<T>> {
							//
							if let Some((amount_need_unlocked, existing_queue)) = value {
								existing_queue
									.try_push(next_queue_id)
									.map_err(|_| Error::<T>::TooManyRedeems)?;

								*amount_need_unlocked = amount_need_unlocked
									.checked_add(&currency_amount)
									.ok_or(Error::<T>::ArithmeticOverflow)?;
							};
							Ok(())
						})?;
					} else {
						let mut new_queue = BoundedVec::<QueueId, T::MaximumQueue>::default();
						new_queue
							.try_push(next_queue_id)
							.map_err(|_| Error::<T>::TooManyRedeems)?;
						UserCurrencyRedeemQueue::<T>::insert(&who, &currency_id, (currency_amount, new_queue));
					}

					// Handle ledger of staking round - executed by hooks on every block - staking_round,currency:
					// total_amount_unlocked, vec![queue_id], currency

					// Check if there any existing claim of the next staking round
					if let Some((_, _, _token_id)) = StakingRoundRedeemQueue::<T>::get(&new_staking_round, &currency_id)
					{
						StakingRoundRedeemQueue::<T>::mutate(
							&new_staking_round,
							&currency_id,
							|value| -> Result<(), Error<T>> {
								// Add new queue item
								if let Some((amount_need_unlocked, existing_queue, _token_id)) = value {
									existing_queue
										.try_push(next_queue_id)
										.map_err(|_| Error::<T>::TooManyRedeems)?;
									*amount_need_unlocked = amount_need_unlocked
										.checked_add(&currency_amount)
										.ok_or(Error::<T>::ArithmeticOverflow)?;
								};
								Ok(())
							},
						)?;
					} else {
						let mut new_queue = BoundedVec::<QueueId, T::MaximumQueue>::default();
						new_queue
							.try_push(next_queue_id)
							.map_err(|_| Error::<T>::TooManyRedeems)?;

						StakingRoundRedeemQueue::<T>::insert(
							&new_staking_round,
							&currency_id,
							(currency_amount, new_queue, currency_id),
						);
					}
				}
				None => return Err(Error::<T>::NoCurrentStakingRound.into()),
			}

			QueueNextId::<T>::mutate(&currency_id, |queue_id| -> Result<(), Error<T>> {
				*queue_id = queue_id.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
				Ok(())
			})?;

			// Emit deposit event
			Self::deposit_event(Event::Redeemed {
				from: who,
				pool_id,
				amount: r_amount,
			});
			Ok(().into())
		}

		/// This function only for governance origin to execute when starting the protocol or
		/// changes of era duration.
		#[pallet::weight(< T as Config >::WeightInfo::mint_land())]
		pub fn update_era_config(
			origin: OriginFor<T>,
			last_era_updated_block: Option<BlockNumberFor<T>>,
			frequency: Option<BlockNumberFor<T>>,
			last_staking_round: StakingRound,
		) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;

			if let Some(change) = frequency {
				UpdateEraFrequency::<T>::put(change);
				Self::deposit_event(Event::<T>::UpdateEraFrequency { frequency: change });
			}

			if let Some(change) = last_era_updated_block {
				let update_era_frequency = UpdateEraFrequency::<T>::get();
				let current_relay_chain_block = T::RelayChainBlockNumber::current_block_number();
				if !update_era_frequency.is_zero() {
					ensure!(
						change > current_relay_chain_block.saturating_sub(update_era_frequency)
							&& change <= current_relay_chain_block,
						Error::<T>::InvalidLastEraUpdatedBlock
					);

					LastEraUpdatedBlock::<T>::put(change);
					LastStakingRound::<T>::insert(FungibleTokenId::NativeToken(1), last_staking_round);
					Self::deposit_event(Event::<T>::LastEraUpdated { last_era_block: change });
				}
			}

			Ok(())
		}

		/// This function allow reward voting for the pool
		#[pallet::weight(< T as Config >::WeightInfo::mint_land())]
		pub fn boost(origin: OriginFor<T>, pool_id: PoolId, vote: BoostInfo<BalanceOf<T>>) -> DispatchResult {
			// Ensure user is signed
			let who = ensure_signed(origin)?;

			// Ensure user has balance to vote
			ensure!(
				vote.balance <= T::Currency::free_balance(&who),
				Error::<T>::InsufficientFund
			);

			// Check if pool exists
			let pool_instance = Pool::<T>::get(pool_id).ok_or(Error::<T>::PoolDoesNotExist)?;
			// Convert boost conviction into shares
			let vote_conviction = vote.conviction.lock_periods();
			// Calculate lock period from UnlockDuration block number x conviction
			let current_block: T::BlockNumber = <frame_system::Pallet<T>>::block_number();

			let mut unlock_at = current_block.saturating_add(UpdateEraFrequency::<T>::get());
			let mut total_balance = vote.balance;
			if !vote_conviction.is_zero() {
				unlock_at.saturating_mul(vote_conviction.into());
				total_balance.saturating_mul(vote_conviction.into());
			}
			// Locked token

			BoostingOf::<T>::try_mutate(who.clone(), |voting| -> DispatchResult {
				let votes = &mut voting.votes;
				match votes.binary_search_by_key(&pool_id, |i| i.0) {
					Ok(i) => {
						// User already boosted, this is adding up their boosting weight
						votes[i]
							.1
							.add(total_balance.clone())
							.ok_or(Error::<T>::ArithmeticOverflow)?;
						voting
							.prior
							.accumulate(unlock_at, votes[i].1.balance.saturating_add(total_balance))
					}
					Err(i) => {
						votes.insert(i, (pool_id, vote.clone()));
						voting.prior.accumulate(unlock_at, total_balance);
					}
				}
				Ok(())
			})?;
			// Add shares into the rewards pool
			<orml_rewards::Pallet<T>>::add_share(&who, &pool_id, total_balance.unique_saturated_into());
			// Emit Boosted event
			Self::deposit_event(Event::<T>::Boosted {
				booster: who.clone(),
				pool_id,
				boost_info: vote.clone(),
			});

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn calculate_next_staking_round(a: StakingRound, b: StakingRound) -> Result<StakingRound, DispatchError> {
		let result = match a {
			StakingRound::Era(era_a) => match b {
				StakingRound::Era(era_b) => {
					StakingRound::Era(era_a.checked_add(era_b).ok_or(Error::<T>::ArithmeticOverflow)?)
				}
				_ => return Err(Error::<T>::Unexpected.into()),
			},
			StakingRound::Round(round_a) => match b {
				StakingRound::Round(round_b) => {
					StakingRound::Round(round_a.checked_add(round_b).ok_or(Error::<T>::ArithmeticOverflow)?)
				}
				_ => return Err(Error::<T>::Unexpected.into()),
			},
			StakingRound::Epoch(epoch_a) => match b {
				StakingRound::Epoch(epoch_b) => {
					StakingRound::Epoch(epoch_a.checked_add(epoch_b).ok_or(Error::<T>::ArithmeticOverflow)?)
				}
				_ => return Err(Error::<T>::Unexpected.into()),
			},
			StakingRound::Hour(hour_a) => match b {
				StakingRound::Hour(hour_b) => {
					StakingRound::Hour(hour_a.checked_add(hour_b).ok_or(Error::<T>::ArithmeticOverflow)?)
				}
				_ => return Err(Error::<T>::Unexpected.into()),
			},
		};

		Ok(result)
	}

	pub fn collect_deposit_fee(
		who: &T::AccountId,
		currency_id: FungibleTokenId,
		amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let (deposit_rate, _redeem_rate) = Fees::<T>::get();

		let deposit_fee = deposit_rate * amount;
		let amount_exclude_fee = amount.checked_sub(&deposit_fee).ok_or(Error::<T>::ArithmeticOverflow)?;
		T::MultiCurrency::transfer(
			currency_id,
			who,
			&T::PoolAccount::get().into_account_truncating(),
			deposit_fee,
		)?;

		return Ok(amount_exclude_fee);
	}

	pub fn collect_redeem_fee(
		who: &T::AccountId,
		currency_id: FungibleTokenId,
		amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let (_mint_rate, redeem_rate) = Fees::<T>::get();
		let redeem_fee = redeem_rate * amount;
		let amount_exclude_fee = amount.checked_sub(&redeem_fee).ok_or(Error::<T>::ArithmeticOverflow)?;
		T::MultiCurrency::transfer(
			currency_id,
			who,
			&T::PoolAccount::get().into_account_truncating(),
			redeem_fee,
		)?;

		return Ok(amount_exclude_fee);
	}

	pub fn collect_pool_creation_fee(who: &T::AccountId, currency_id: FungibleTokenId) -> DispatchResult {
		let pool_fee = T::NetworkFee::get();
		T::MultiCurrency::transfer(
			currency_id,
			who,
			&T::PoolAccount::get().into_account_truncating(),
			pool_fee,
		)
	}

	fn handle_update_staking_round(era_index: EraIndex, currency: FungibleTokenId) -> DispatchResult {
		let last_staking_round = StakingRound::Era(era_index as u32);
		let unlock_duration = match UnlockDuration::<T>::get(currency) {
			Some(StakingRound::Era(unlock_duration_era)) => unlock_duration_era,
			Some(StakingRound::Round(unlock_duration_round)) => unlock_duration_round,
			Some(StakingRound::Epoch(unlock_duration_epoch)) => unlock_duration_epoch,
			Some(StakingRound::Hour(unlock_duration_hour)) => unlock_duration_hour,
			_ => 0,
		};

		let current_staking_round = era_index;

		// Check current staking round queue with last staking round if there is any pending redeem requests
		if let Some((_total_locked, existing_queue, currency_id)) =
			StakingRoundRedeemQueue::<T>::get(last_staking_round.clone(), currency)
		{
			for queue_id in existing_queue.iter().take(Self::iteration_limit() as usize) {
				if let Some((account, unlock_amount, staking_round)) =
					CurrencyRedeemQueue::<T>::get(currency_id, queue_id)
				{
					let pool_account_balance =
						T::MultiCurrency::free_balance(currency_id, &T::PoolAccount::get().into_account_truncating());
					if pool_account_balance != BalanceOf::<T>::zero() {
						Self::update_queue_request(
							currency_id,
							account,
							queue_id,
							unlock_amount,
							pool_account_balance,
							staking_round,
						)
						.ok();
					}
				}
			}
		} else {
			LastStakingRound::<T>::mutate(currency, |last_staking_round| -> Result<(), Error<T>> {
				match last_staking_round {
					StakingRound::Era(era) => {
						if current_staking_round + unlock_duration > *era {
							*era = era.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
						}
						Ok(())
					}
					StakingRound::Round(round) => {
						if current_staking_round + unlock_duration > *round {
							*round = round.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
						}
						Ok(())
					}
					StakingRound::Epoch(epoch) => {
						if current_staking_round + unlock_duration > *epoch {
							*epoch = epoch.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
						}
						Ok(())
					}
					StakingRound::Hour(hour) => {
						if current_staking_round + unlock_duration > *hour {
							*hour = hour.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
						}
						Ok(())
					}
					_ => Ok(()),
				}
			})?;
		};

		Ok(())
	}

	#[transactional]
	fn update_queue_request(
		currency_id: FungibleTokenId,
		account: T::AccountId,
		queue_id: &QueueId,
		mut unlock_amount: BalanceOf<T>,
		pool_account_balance: BalanceOf<T>,
		staking_round: StakingRound,
	) -> DispatchResult {
		// Get minimum balance of currency
		let ed = T::MultiCurrency::minimum_balance(currency_id);
		let mut account_to_send = account.clone();

		// If unlock amount less than existential deposit, to avoid error kill the process, transfer the
		// unlock_amount to pool address instead
		if unlock_amount < ed {
			let receiver_balance = T::MultiCurrency::total_balance(currency_id, &account);

			// Check if even after receiving unlock amount, account still below ED then transfer fund to
			// PoolAccount
			let receiver_balance_after = receiver_balance
				.checked_add(&unlock_amount)
				.ok_or(ArithmeticError::Overflow)?;
			if receiver_balance_after < ed {
				account_to_send = T::PoolAccount::get().into_account_truncating();
			}
		}

		// If pool account balance greater than unlock amount
		if pool_account_balance >= unlock_amount {
			// Transfer amount from PoolAccount to users
			T::MultiCurrency::transfer(
				currency_id,
				&T::PoolAccount::get().into_account_truncating(),
				&account_to_send,
				unlock_amount,
			)?;

			// Remove currency redeem queue
			CurrencyRedeemQueue::<T>::remove(&currency_id, &queue_id);

			// Edit staking round redeem queue with locked amount
			StakingRoundRedeemQueue::<T>::mutate_exists(
				&staking_round,
				&currency_id,
				|value| -> Result<(), Error<T>> {
					if let Some((total_locked_origin, existing_queue, _)) = value {
						// If total locked == unlock_amount, then set value to zero
						if total_locked_origin == &unlock_amount {
							*value = None;
							return Ok(());
						}
						// Otherwise, deduct unlock amount
						*total_locked_origin = total_locked_origin
							.checked_sub(&unlock_amount)
							.ok_or(Error::<T>::ArithmeticOverflow)?;
						// Only keep items that not with processed queue_id
						existing_queue.retain(|x| x != queue_id);
					} else {
						return Err(Error::<T>::StakingRoundRedeemNotFound);
					}
					Ok(())
				},
			)?;

			UserCurrencyRedeemQueue::<T>::mutate_exists(&account, &currency_id, |value| -> Result<(), Error<T>> {
				if let Some((total_locked_origin, existing_queue)) = value {
					if total_locked_origin == &unlock_amount {
						*value = None;
						return Ok(());
					}
					existing_queue.retain(|x| x != queue_id);
					*total_locked_origin = total_locked_origin
						.checked_sub(&unlock_amount)
						.ok_or(Error::<T>::ArithmeticOverflow)?;
				} else {
					return Err(Error::<T>::UserCurrencyRedeemQueueNotFound);
				}
				Ok(())
			})?;
		} else {
			// When pool account balance less than amount need to be unlocked then use pool remaining balance as
			// unlock amount
			unlock_amount = pool_account_balance;
			T::MultiCurrency::transfer(
				currency_id,
				&T::PoolAccount::get().into_account_truncating(),
				&account_to_send,
				unlock_amount,
			)?;

			CurrencyRedeemQueue::<T>::mutate_exists(&currency_id, &queue_id, |value| -> Result<(), Error<T>> {
				if let Some((_, total_locked_origin, _)) = value {
					if total_locked_origin == &unlock_amount {
						*value = None;
						return Ok(());
					}
					*total_locked_origin = total_locked_origin
						.checked_sub(&unlock_amount)
						.ok_or(Error::<T>::ArithmeticOverflow)?;
				} else {
					return Err(Error::<T>::CurrencyRedeemQueueNotFound);
				}
				Ok(())
			})?;

			StakingRoundRedeemQueue::<T>::mutate_exists(
				&staking_round,
				&currency_id,
				|value| -> Result<(), Error<T>> {
					if let Some((total_locked_origin, _existing_queue, _)) = value {
						if total_locked_origin == &unlock_amount {
							*value = None;
							return Ok(());
						}
						*total_locked_origin = total_locked_origin
							.checked_sub(&unlock_amount)
							.ok_or(Error::<T>::ArithmeticOverflow)?;
					} else {
						return Err(Error::<T>::StakingRoundRedeemNotFound);
					}
					Ok(())
				},
			)?;

			UserCurrencyRedeemQueue::<T>::mutate_exists(&account, &currency_id, |value| -> Result<(), Error<T>> {
				if let Some((total_locked_origin, _existing_queue)) = value {
					if total_locked_origin == &unlock_amount {
						*value = None;
						return Ok(());
					}

					*total_locked_origin = total_locked_origin
						.checked_sub(&unlock_amount)
						.ok_or(Error::<T>::ArithmeticOverflow)?;
				} else {
					return Err(Error::<T>::UserCurrencyRedeemQueueNotFound);
				}
				Ok(())
			})?;
		}

		pool_account_balance
			.checked_sub(&unlock_amount)
			.ok_or(Error::<T>::ArithmeticOverflow)?;

		NetworkLedger::<T>::mutate(&currency_id, |pool| -> Result<(), Error<T>> {
			*pool = pool.checked_sub(&unlock_amount).ok_or(Error::<T>::ArithmeticOverflow)?;
			Ok(())
		})?;

		Self::deposit_event(Event::RedeemSuccess {
			queue_id: *queue_id,
			currency_id,
			to: account_to_send,
			token_amount: unlock_amount,
		});
		Ok(())
	}

	pub fn get_era_index(relaychain_block_number: BlockNumberFor<T>) -> EraIndex {
		relaychain_block_number
			.checked_sub(&Self::last_era_updated_block())
			.and_then(|n| n.checked_div(&Self::update_era_frequency()))
			.and_then(|n| TryInto::<EraIndex>::try_into(n).ok())
			.unwrap_or_else(Zero::zero)
	}

	fn handle_redeem_requests(era_index: EraIndex) -> DispatchResult {
		for currency in CurrentStakingRound::<T>::iter_keys() {
			Self::handle_update_staking_round(era_index, currency)?;
		}
		Ok(())
	}

	#[transactional]
	pub fn update_current_era(era_index: EraIndex) -> DispatchResult {
		let previous_era = Self::relay_chain_current_era();
		let new_era = previous_era.saturating_add(era_index);

		RelayChainCurrentEra::<T>::put(new_era);
		LastEraUpdatedBlock::<T>::put(T::RelayChainBlockNumber::current_block_number());
		Self::handle_redeem_requests(new_era)?;
		Self::deposit_event(Event::<T>::CurrentEraUpdated { new_era_index: new_era });
		Ok(())
	}

	pub fn get_pool_account() -> T::AccountId {
		T::PoolAccount::get().into_account_truncating()
	}
}
