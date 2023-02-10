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
use frame_support::traits::{LockIdentifier, WithdrawReasons};
use frame_support::{
	ensure, log,
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement, LockableCurrency, ReservableCurrency},
	transactional, PalletId,
};
use frame_system::{ensure_signed, pallet_prelude::*};
use orml_traits::{DataFeeder, DataProvider, MultiCurrency, MultiReservableCurrency};
use sp_runtime::traits::{BlockNumberProvider, CheckedAdd, CheckedMul, Saturating};
use sp_runtime::{
	traits::{AccountIdConversion, One, Zero},
	ArithmeticError, DispatchError, Perbill,
};
use sp_std::{collections::btree_map::BTreeMap, prelude::*, vec::Vec};

use core_primitives::NFTTrait;
use core_primitives::*;
pub use pallet::*;
use primitives::{estate::Estate, EstateId};
use primitives::{AssetId, Balance, ClassId, DomainId, FungibleTokenId, MetaverseId, NftId, PowerAmount, RoundIndex};
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
	use orml_traits::MultiCurrencyExtended;
	use sp_runtime::traits::{CheckedAdd, CheckedSub, Saturating};
	use sp_runtime::ArithmeticError;

	use primitives::staking::{RoundInfo, Bond};
	use primitives::{ClassId, GroupCollectionId, NftId};

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

		/// NFT handler
		type NFTHandler: NFTTrait<Self::AccountId, BalanceOf<Self>, ClassId = ClassId, TokenId = TokenId>;

		/// Round handler
		type RoundHandler: RoundTrait<Self::BlockNumber>;

		/// Estate handler
		type EstateHandler: Estate<Self::AccountId>;

		/// Economy treasury fund
		#[pallet::constant]
		type EconomyTreasury: Get<PalletId>;

		/// The currency id of BIT
		#[pallet::constant]
		type MiningCurrencyId: Get<FungibleTokenId>;

		/// The minimum stake required for staking
		#[pallet::constant]
		type MinimumStake: Get<BalanceOf<Self>>;

		/// The Power Amount per block
		#[pallet::constant]
		type PowerAmountPerBlock: Get<PowerAmount>;

		/// Weight info
		type WeightInfo: WeightInfo;
	}

	/// BIT to power exchange rate
	#[pallet::storage]
	#[pallet::getter(fn get_bit_power_exchange_rate)]
	pub(super) type BitPowerExchangeRate<T: Config> = StorageValue<_, Balance, ValueQuery>;

	/// Power balance of user
	#[pallet::storage]
	#[pallet::getter(fn get_power_balance)]
	pub type PowerBalance<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, PowerAmount, ValueQuery>;

	/// TBD Accept domain
	#[pallet::storage]
	#[pallet::getter(fn get_accepted_domain)]
	pub type AcceptedDomain<T: Config> = StorageMap<_, Twox64Concat, DomainId, ()>;

	/// Self-staking info
	#[pallet::storage]
	#[pallet::getter(fn get_staking_info)]
	pub type StakingInfo<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	/// Estate-staking info
	#[pallet::storage]
	#[pallet::getter(fn get_estate_staking_info)]
	pub type EstateStakingInfo<T: Config> = StorageMap<_, Twox64Concat, EstateId, Bond<T::AccountId, BalanceOf<T>>, OptionQuery>;

	/// Self-staking exit queue info
	/// This will keep track of stake exits queue, unstake only allows after 1 round
	#[pallet::storage]
	#[pallet::getter(fn staking_exit_queue)]
	pub type ExitQueue<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Twox64Concat, RoundIndex, BalanceOf<T>, OptionQuery>;

	/// Total native token locked in this pallet
	#[pallet::storage]
	#[pallet::getter(fn total_stake)]
	type TotalStake<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	/// Total native token locked estate staking pallet
	#[pallet::storage]
	#[pallet::getter(fn total_estate_stake)]
	type TotalEstateStake<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Mining resource burned [amount]
		MiningResourceBurned(Balance),
		/// Self staking to economy 101 [staker, amount]
		SelfStakedToEconomy101(T::AccountId, BalanceOf<T>),
		/// Estate staking to economy 101 [staker, estate_id, amount]
		EstateStakedToEconomy101(T::AccountId, EstateId, BalanceOf<T>),
		/// Self staking removed from economy 101 [staker, amount]
		SelfStakingRemovedFromEconomy101(T::AccountId, BalanceOf<T>),
		/// Estate staking remoed from economy 101 [staker, estate_id, amount]
		EstateStakingRemovedFromEconomy101(T::AccountId, EstateId, BalanceOf<T>),
		/// New BIT to Power exchange rate has updated [amount]
		BitPowerExchangeRateUpdated(Balance),
		/// Unstaked amount has been withdrew after it's expired [account, rate]
		UnstakedAmountWithdrew(T::AccountId, BalanceOf<T>),
		/// Set power balance by sudo [account, power_amount]
		SetPowerBalance(T::AccountId, PowerAmount),
		/// Power conversion request has cancelled [(class_id, token_id), account]
		CancelPowerConversionRequest((ClassId, TokenId), T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// NFT asset does not exist
		NFTAssetDoesNotExist,
		/// NFT class does not exist
		NFTClassDoesNotExist,
		/// NFT collection does not exist
		NFTCollectionDoesNotExist,
		/// No permission
		NoPermission,
		/// No authorization
		NoAuthorization,
		/// Insufficient power balance
		AccountHasNoPowerBalance,
		/// Power amount is zero
		PowerAmountIsZero,
		/// Not enough free balance for staking
		InsufficientBalanceForStaking,
		/// Unstake amount greater than staked amount
		UnstakeAmountExceedStakedAmount,
		/// Has scheduled exit staking, only stake after queue exit
		ExitQueueAlreadyScheduled,
		/// Stake amount below minimum staking required
		StakeBelowMinimum,
		/// Withdraw future round
		WithdrawFutureRound,
		/// Exit queue does not exist
		ExitQueueDoesNotExit,
		/// Unstaked amount is zero
		UnstakeAmountIsZero,
		/// Request already exists
		RequestAlreadyExist,
		/// Order has not reach target
		NotReadyToExecute,
		/// Staker is not estate owner
		StakerNotEstateOwner,
		/// Staking estate does not exist
		StakeEstateDoesNotExist,
		/// Stake is not previous owner
		StakerNotPreviousOwner,
		/// No funds staked at estate
		NoFundsStakedAtEstate,
		/// Previous owner still stakes at estate
		PreviousOwnerStillStakesAtEstate,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set bit power exchange rate
		///
		/// The dispatch origin for this call must be _Root_.
		///
		/// `rate`: exchange rate of bit to power. input is BIT price per power
		///
		/// Emit `BitPowerExchangeRateUpdated` event if successful
		#[pallet::weight(T::WeightInfo::set_bit_power_exchange_rate())]
		#[transactional]
		pub fn set_bit_power_exchange_rate(origin: OriginFor<T>, rate: Balance) -> DispatchResultWithPostInfo {
			// Only root can authorize
			ensure_root(origin)?;

			BitPowerExchangeRate::<T>::set(rate);

			Self::deposit_event(Event::<T>::BitPowerExchangeRateUpdated(rate));

			Ok(().into())
		}

		/// Set power balance for specific NFT
		///
		/// The dispatch origin for this call must be _Root_.
		///
		/// `beneficiary`: NFT account that receives power
		/// `amount`: amount of power
		///
		/// Emit `SetPowerBalance` event if successful
		#[pallet::weight(T::WeightInfo::set_power_balance())]
		#[transactional]
		pub fn set_power_balance(
			origin: OriginFor<T>,
			beneficiary: (ClassId, TokenId),
			amount: PowerAmount,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			let account_id = T::EconomyTreasury::get().into_sub_account_truncating(beneficiary);
			PowerBalance::<T>::insert(&account_id, amount);

			Self::deposit_event(Event::<T>::SetPowerBalance(account_id, amount));

			Ok(().into())
		}

		/// Stake native token to staking ledger to receive build material every round
		///
		/// The dispatch origin for this call must be _Signed_.
		///
		/// `amount`: the stake amount
		///
		/// Emit `SelfStakedToEconomy101` event or `EstateStakedToEconomy101` event if successful
		#[pallet::weight(
			if estate.is_some() {
				T::WeightInfo::stake_b()
			} else {
				T::WeightInfo::stake_a()
			}
		)]
		#[transactional]
		pub fn stake(
			origin: OriginFor<T>,
			amount: BalanceOf<T>,
			estate: Option<EstateId>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Check if user has enough balance for staking
			ensure!(
				T::Currency::free_balance(&who) >= amount,
				Error::<T>::InsufficientBalanceForStaking
			);

			let current_round = T::RoundHandler::get_current_round_info();
			// Check if user already in exit queue
			ensure!(
				!ExitQueue::<T>::contains_key(&who, current_round.current),
				Error::<T>::ExitQueueAlreadyScheduled
			);

			match estate {
				None => {
					let mut staked_balance = StakingInfo::<T>::get(&who);
					let total = staked_balance.checked_add(&amount).ok_or(ArithmeticError::Overflow)?;

					ensure!(total >= T::MinimumStake::get(), Error::<T>::StakeBelowMinimum);

					T::Currency::reserve(&who, amount)?;

					StakingInfo::<T>::insert(&who, total);

					let new_total_staked = TotalStake::<T>::get().saturating_add(amount);
					<TotalStake<T>>::put(new_total_staked);

					let current_round = T::RoundHandler::get_current_round_info();
					Self::deposit_event(Event::SelfStakedToEconomy101(who, amount));
				}
				Some(estate_id) => {
					ensure!(
						T::EstateHandler::check_estate(estate_id.clone())?,
						Error::<T>::StakeEstateDoesNotExist
					);
					ensure!(
						T::EstateHandler::check_estate_ownership(who.clone(), estate_id.clone())?,
						Error::<T>::StakerNotEstateOwner
					);

					let mut staked_balance: BalanceOf<T> = Zero::zero();
					let staking_bond_value = EstateStakingInfo::<T>::get(estate_id);
					match staking_bond_value {
						Some(staking_bond) => {
							ensure!(
								staking_bond.staker != who.clone(),
								Error::<T>::PreviousOwnerStillStakesAtEstate
							);
							staked_balance = staking_bond.amount;
						}
						_ => {}
					}
					
					let total = staked_balance.checked_add(&amount).ok_or(ArithmeticError::Overflow)?;

					ensure!(total >= T::MinimumStake::get(), Error::<T>::StakeBelowMinimum);

					T::Currency::reserve(&who, amount)?;

					let new_staking_bond = Bond {
						staker: who.clone(),
						amount: total,
					};

					EstateStakingInfo::<T>::insert(&estate_id, new_staking_bond);

					let new_total_staked = TotalEstateStake::<T>::get().saturating_add(amount);
					<TotalEstateStake<T>>::put(new_total_staked);

					let current_round = T::RoundHandler::get_current_round_info();
					Self::deposit_event(Event::EstateStakedToEconomy101(who, estate_id, amount));
				}
			}

			Ok(().into())
		}

		/// Unstake native token from staking ledger. The unstaked amount able to redeem from the
		/// next round
		///
		/// The dispatch origin for this call must be _Signed_.
		///
		/// `amount`: the stake amount
		///
		/// Emit `SelfStakingRemovedFromEconomy101` event or `EstateStakingRemovedFromEconomy101`
		/// event if successful
		#[pallet::weight(
			if estate.is_some() {
				T::WeightInfo::unstake_b()
			} else {
				T::WeightInfo::unstake_a()
			}
		)]
		pub fn unstake(
			origin: OriginFor<T>,
			amount: BalanceOf<T>,
			estate: Option<EstateId>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Ensure amount is greater than zero
			ensure!(!amount.is_zero(), Error::<T>::UnstakeAmountIsZero);

			match estate {
				None => {
					let mut staked_balance = StakingInfo::<T>::get(&who);
					ensure!(amount <= staked_balance, Error::<T>::UnstakeAmountExceedStakedAmount);

					let remaining = staked_balance.checked_sub(&amount).ok_or(ArithmeticError::Underflow)?;

					let amount_to_unstake = if remaining < T::MinimumStake::get() {
						// Remaining amount below minimum, remove all staked amount
						staked_balance
					} else {
						amount
					};

					let current_round = T::RoundHandler::get_current_round_info();
					let next_round = current_round.current.saturating_add(One::one());

					// Check if user already in exit queue of the current
					ensure!(
						!ExitQueue::<T>::contains_key(&who, next_round),
						Error::<T>::ExitQueueAlreadyScheduled
					);

					// This exit queue will be executed by exit_staking extrinsics to unreserved token
					ExitQueue::<T>::insert(&who, next_round.clone(), amount_to_unstake);

					// Update staking info of user immediately
					// Remove staking info
					if amount_to_unstake == staked_balance {
						StakingInfo::<T>::remove(&who);
					} else {
						StakingInfo::<T>::insert(&who, remaining);
					}

					let new_total_staked = TotalStake::<T>::get().saturating_sub(amount_to_unstake);
					<TotalStake<T>>::put(new_total_staked);

					Self::deposit_event(Event::SelfStakingRemovedFromEconomy101(who, amount));
				}
				Some(estate_id) => {
					ensure!(
						T::EstateHandler::check_estate(estate_id.clone())?,
						Error::<T>::StakeEstateDoesNotExist
					);

					let mut staked_balance = Zero::zero();
					let staking_bond_value = EstateStakingInfo::<T>::get(estate_id);
					match staking_bond_value {
						Some(staking_bond) => {
							ensure!(
								staking_bond.staker == who.clone(),
								Error::<T>::NoFundsStakedAtEstate
							);
							staked_balance = staking_bond.amount;
						}
						_=> {}
					}
					ensure!(amount <= staked_balance, Error::<T>::UnstakeAmountExceedStakedAmount);

					let remaining = staked_balance.checked_sub(&amount).ok_or(ArithmeticError::Underflow)?;

					let amount_to_unstake = if remaining < T::MinimumStake::get() {
						// Remaining amount below minimum, remove all staked amount
						staked_balance
					} else {
						amount
					};

					let current_round = T::RoundHandler::get_current_round_info();
					let next_round = current_round.current.saturating_add(One::one());

					// This exit queue will be executed by exit_staking extrinsics to unreserved token
					ExitQueue::<T>::insert(&who, next_round.clone(), amount_to_unstake);

					// Update staking info of user immediately
					// Remove staking info
					if amount_to_unstake == staked_balance {
						EstateStakingInfo::<T>::remove(&estate_id);
					} else {
						let new_staking_bond = Bond {
							staker: who.clone(),
							amount: remaining,
						};
						EstateStakingInfo::<T>::insert(&estate_id, new_staking_bond);
					}

					let new_total_staked = TotalEstateStake::<T>::get().saturating_sub(amount_to_unstake);
					<TotalEstateStake<T>>::put(new_total_staked);

					Self::deposit_event(Event::EstateStakingRemovedFromEconomy101(who, estate_id, amount));
				}
			}

			Ok(().into())
		}

		/// Unstake native token (staked by previous owner) from staking ledger.
		///
		/// The dispatch origin for this call must be _Signed_. Works if the origin is the estate owner and the previous owner got staked funds
		///
		/// `estate_id`: the estate ID which funds are going to be unstaked
		///
		/// Emit `EstateStakingRemovedFromEconomy101` event if successful
		#[pallet::weight(T::WeightInfo::unstake_b())]
		pub fn unstake_new_estate_owner(origin: OriginFor<T>, estate_id: EstateId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(
				T::EstateHandler::check_estate(estate_id.clone())?,
				Error::<T>::StakeEstateDoesNotExist
			);

			ensure!(
				T::EstateHandler::check_estate_ownership(who.clone(), estate_id.clone())?,
				Error::<T>::StakerNotEstateOwner
			);

			let staking_bond_value = EstateStakingInfo::<T>::get(estate_id);
			match staking_bond_value {
				Some(staking_info) => {
					ensure!(
						staking_info.staker.clone() != who.clone(),
						Error::<T>::StakerNotPreviousOwner
					);
					let staked_balance = staking_info.amount;
		
					let current_round = T::RoundHandler::get_current_round_info();
					let next_round = current_round.current.saturating_add(One::one());
		
					// This exit queue will be executed by exit_staking extrinsics to unreserved token
					ExitQueue::<T>::insert(&staking_info.staker, next_round.clone(), staked_balance);
					EstateStakingInfo::<T>::remove(&estate_id);
		
					let new_total_staked = TotalEstateStake::<T>::get().saturating_sub(staked_balance);
					<TotalEstateStake<T>>::put(new_total_staked);
		
					Self::deposit_event(Event::EstateStakingRemovedFromEconomy101(who, estate_id, staked_balance));
					Ok(().into())
				}
				None => {
					Err(Error::<T>::StakeEstateDoesNotExist.into())
				}
			}
		}
			

		/// Withdraw unstaked token from unstaking queue. The unstaked amount will be unreserved and
		/// become transferrable
		///
		/// The dispatch origin for this call must be _Signed_.
		///
		/// `round_index`: the round index that user can unstake.
		///
		/// Emit `UnstakedAmountWithdrew` event if successful
		#[pallet::weight(T::WeightInfo::withdraw_unreserved())]
		pub fn withdraw_unreserved(origin: OriginFor<T>, round_index: RoundIndex) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Get user exit queue
			let exit_balance = ExitQueue::<T>::get(&who, round_index).ok_or(Error::<T>::ExitQueueDoesNotExit)?;

			ExitQueue::<T>::remove(&who, round_index);
			T::Currency::unreserve(&who, exit_balance);

			Self::deposit_event(Event::<T>::UnstakedAmountWithdrew(who, exit_balance));

			Ok(().into())
		}

		/// Force unstake native token from staking ledger. The unstaked amount able to redeem
		/// immediately
		///
		///
		/// The dispatch origin for this call must be _Root_.
		///
		/// `amount`: the stake amount
		/// `who`: the address of staker
		///
		/// Emit `SelfStakingRemovedFromEconomy101` event or `EstateStakingRemovedFromEconomy101`
		/// event if successful
		#[pallet::weight(
			if estate.is_some() {
				T::WeightInfo::unstake_b()
			} else {
				T::WeightInfo::unstake_a()
			}
		)]
		pub fn force_unstake(
			origin: OriginFor<T>,
			amount: BalanceOf<T>,
			who: T::AccountId,
			estate: Option<EstateId>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			// Ensure amount is greater than zero
			ensure!(!amount.is_zero(), Error::<T>::UnstakeAmountIsZero);

			match estate {
				None => {
					let mut staked_balance = StakingInfo::<T>::get(&who);
					ensure!(amount <= staked_balance, Error::<T>::UnstakeAmountExceedStakedAmount);

					let remaining = staked_balance.checked_sub(&amount).ok_or(ArithmeticError::Underflow)?;

					let amount_to_unstake = if remaining < T::MinimumStake::get() {
						// Remaining amount below minimum, remove all staked amount
						staked_balance
					} else {
						amount
					};

					// Update staking info of user immediately
					// Remove staking info
					if amount_to_unstake == staked_balance {
						StakingInfo::<T>::remove(&who);
					} else {
						StakingInfo::<T>::insert(&who, remaining);
					}

					let new_total_staked = TotalStake::<T>::get().saturating_sub(amount_to_unstake);
					<TotalStake<T>>::put(new_total_staked);

					T::Currency::unreserve(&who, amount_to_unstake);

					Self::deposit_event(Event::UnstakedAmountWithdrew(who.clone(), amount_to_unstake));
					Self::deposit_event(Event::SelfStakingRemovedFromEconomy101(who, amount));
				}
				Some(estate_id) => {
					ensure!(
						T::EstateHandler::check_estate(estate_id.clone())?,
						Error::<T>::StakeEstateDoesNotExist
					);
					let mut staked_balance: BalanceOf<T> = Zero::zero();
					let staking_bond_value = EstateStakingInfo::<T>::get(estate_id);
					match staking_bond_value {
						Some(staking_bond) => {
							ensure!(
								staking_bond.staker == who.clone(),
								Error::<T>::NoFundsStakedAtEstate
							);
							staked_balance = staking_bond.amount;
						}
						_ => {}
					}
					ensure!(amount <= staked_balance, Error::<T>::UnstakeAmountExceedStakedAmount);

					let remaining = staked_balance.checked_sub(&amount).ok_or(ArithmeticError::Underflow)?;

					let amount_to_unstake = if remaining < T::MinimumStake::get() {
						// Remaining amount below minimum, remove all staked amount
						staked_balance
					} else {
						amount
					};

					// Update staking info of user immediately
					// Remove staking info
					if amount_to_unstake == staked_balance {
						EstateStakingInfo::<T>::remove(&estate_id);
					} else {
						let new_staking_bond = Bond {
							staker: who.clone(),
							amount: remaining
						};
						EstateStakingInfo::<T>::insert(&estate_id, new_staking_bond);
					}

					let new_total_staked = TotalStake::<T>::get().saturating_sub(amount_to_unstake);
					<TotalStake<T>>::put(new_total_staked);

					T::Currency::unreserve(&who, amount_to_unstake);

					Self::deposit_event(Event::UnstakedAmountWithdrew(who.clone(), amount_to_unstake));
					Self::deposit_event(Event::EstateStakingRemovedFromEconomy101(who, estate_id, amount));
				}
			}
			Ok(().into())
		}

		/// Force unreserved unstake native token from staking ledger. The unstaked amount able to
		/// unreserve immediately
		///
		///
		/// The dispatch origin for this call must be _Root_.
		///
		/// `amount`: the stake amount
		/// `who`: the address of staker
		///
		/// Emit `SelfStakingRemovedFromEconomy101` event if successful
		#[pallet::weight(T::WeightInfo::unstake_b())]
		pub fn force_unreserved_staking(
			origin: OriginFor<T>,
			amount: BalanceOf<T>,
			who: T::AccountId,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			// Ensure amount is greater than zero
			ensure!(!amount.is_zero(), Error::<T>::UnstakeAmountIsZero);

			// Update staking info
			let mut staked_reserved_balance = T::Currency::reserved_balance(&who);
			ensure!(
				amount <= staked_reserved_balance,
				Error::<T>::UnstakeAmountExceedStakedAmount
			);

			T::Currency::unreserve(&who, amount);

			Ok(().into())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}

impl<T: Config> Pallet<T> {
	pub fn economy_pallet_account_id() -> T::AccountId {
		T::EconomyTreasury::get().into_account_truncating()
	}

	pub fn convert_power_to_bit(power_amount: Balance, commission: Perbill) -> (Balance, Balance) {
		let rate = Self::get_bit_power_exchange_rate();

		let mut bit_required = power_amount
			.checked_mul(rate)
			.ok_or(ArithmeticError::Overflow)
			.unwrap_or(Zero::zero());
		let commission_fee = commission * bit_required;
		(
			bit_required + commission_fee,
			TryInto::<Balance>::try_into(commission_fee).unwrap_or_default(),
		)
	}

	fn do_burn(who: &T::AccountId, amount: Balance) -> DispatchResult {
		if amount.is_zero() {
			return Ok(());
		}

		T::FungibleTokenCurrency::withdraw(T::MiningCurrencyId::get(), who, amount);

		Self::deposit_event(Event::<T>::MiningResourceBurned(amount));

		Ok(())
	}

	fn distribute_power_by_network(power_amount: PowerAmount, beneficiary: &T::AccountId) -> DispatchResult {
		let mut distributor_power_balance = PowerBalance::<T>::get(beneficiary);
		distributor_power_balance = distributor_power_balance
			.checked_add(power_amount)
			.ok_or(ArithmeticError::Overflow)?;

		PowerBalance::<T>::insert(beneficiary.clone(), power_amount);

		Ok(())
	}

	fn get_target_execution_order(power_amount: PowerAmount) -> Result<T::BlockNumber, DispatchError> {
		let current_block_number = <frame_system::Pallet<T>>::current_block_number();
		let target_block = if power_amount <= T::PowerAmountPerBlock::get() {
			let target_b = current_block_number
				.checked_add(&One::one())
				.ok_or(ArithmeticError::Overflow)?;
			target_b
		} else {
			let block_required = power_amount
				.checked_div(T::PowerAmountPerBlock::get())
				.ok_or(ArithmeticError::Overflow)?;

			let target_b = current_block_number
				.checked_add(&TryInto::<T::BlockNumber>::try_into(block_required).unwrap_or_default())
				.ok_or(ArithmeticError::Overflow)?;
			target_b
		};

		Ok(target_block)
	}

	fn check_target_execution(target: T::BlockNumber) -> bool {
		let current_block_number = <frame_system::Pallet<T>>::current_block_number();

		current_block_number >= target
	}
}
