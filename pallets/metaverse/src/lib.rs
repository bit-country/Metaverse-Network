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
	PalletId,
};
use frame_system::{ensure_signed, pallet_prelude::*};
use orml_traits::MultiCurrency;
use sp_runtime::traits::Saturating;
use sp_runtime::{
	traits::{AccountIdConversion, One, Zero},
	DispatchError, Perbill,
};
use sp_std::{collections::btree_map::BTreeMap, prelude::*};

use core_primitives::*;
use core_primitives::{MetaverseInfo, MetaverseInfoV1, MetaverseTrait};
pub use pallet::*;
use primitives::staking::MetaverseStakingTrait;
use primitives::{ClassId, FungibleTokenId, MetaverseId, RoundIndex, TokenId};
pub use weights::WeightInfo;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;

/// A record for total rewards and total amount staked for an era
#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct MetaverseStakingSnapshot<Balance> {
	/// Total amount of rewards for a staking round
	rewards: Balance,
	/// Total staked amount for a staking round
	staked: Balance,
}

const LOCK_STAKING: LockIdentifier = *b"stakelok";
const ESTATE_CLASS_ROYALTY_FEE: u32 = 5;
const LAND_CLASS_ROYALTY_FEE: u32 = 10;

/// Storing the reward detail of metaverse that store the list of stakers for each metaverse
/// This will be used to reward metaverse owner and the stakers.
#[derive(Clone, PartialEq, Encode, Decode, Default, RuntimeDebug, TypeInfo)]
pub struct MetaverseStakingPoints<AccountId: Ord, Balance: HasCompact> {
	/// Total staked amount.
	total: Balance,
	/// The map of stakers and the amount they staked.
	stakers: BTreeMap<AccountId, Balance>,
	/// Accrued and claimed rewards on this metaverse for both metaverse owner and stakers
	claimed_rewards: Balance,
}

#[frame_support::pallet]
pub mod pallet {
	use orml_traits::MultiCurrencyExtended;
	use sp_runtime::traits::{CheckedAdd, Saturating};
	use sp_runtime::ArithmeticError;

	use primitives::staking::RoundInfo;
	use primitives::RoundIndex;

	use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The currency type
		type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>
			+ ReservableCurrency<Self::AccountId>;
		/// The multicurrencies type
		type MultiCurrency: MultiCurrencyExtended<
			Self::AccountId,
			CurrencyId = FungibleTokenId,
			Balance = BalanceOf<Self>,
		>;
		#[pallet::constant]
		type MetaverseTreasury: Get<PalletId>;
		#[pallet::constant]
		type MaxMetaverseMetadata: Get<u32>;
		/// Minimum contribution
		#[pallet::constant]
		type MinContribution: Get<BalanceOf<Self>>;
		/// Origin to add new metaverse
		type MetaverseCouncil: EnsureOrigin<Self::Origin>;
		/// Mininum deposit for registering a metaverse
		type MetaverseRegistrationDeposit: Get<BalanceOf<Self>>;
		/// Mininum staking amount
		type MinStakingAmount: Get<BalanceOf<Self>>;
		/// Maximum amount of stakers per metaverse
		type MaxNumberOfStakersPerMetaverse: Get<u32>;
		/// Weight implementation for estate extrinsics
		type WeightInfo: WeightInfo;
		/// NFT handler required for minting classes for lands and estates when creating a metaverse
		type NFTHandler: NFTTrait<Self::AccountId, BalanceOf<Self>, ClassId = ClassId, TokenId = TokenId>;
	}

	#[pallet::storage]
	#[pallet::getter(fn next_metaverse_id)]
	pub type NextMetaverseId<T: Config> = StorageValue<_, MetaverseId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_metaverse)]
	pub type Metaverses<T: Config> = StorageMap<_, Twox64Concat, MetaverseId, MetaverseInfo<T::AccountId>>;

	#[pallet::storage]
	#[pallet::getter(fn get_metaverse_owner)]
	pub type MetaverseOwner<T: Config> = StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, MetaverseId, ()>;

	#[pallet::storage]
	#[pallet::getter(fn all_metaverse_count)]
	pub(super) type AllMetaversesCount<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_freezing_metaverse)]
	pub(super) type FreezedMetaverses<T: Config> = StorageMap<_, Twox64Concat, MetaverseId, (), OptionQuery>;

	/// Metaverse staking related storage

	/// Staking round info
	#[pallet::storage]
	#[pallet::getter(fn staking_round)]
	/// Current round index and next round scheduled transition
	pub type Round<T: Config> = StorageValue<_, RoundInfo<T::BlockNumber>, ValueQuery>;

	/// Registered metaverse for staking
	#[pallet::storage]
	#[pallet::getter(fn get_registered_metaverse)]
	pub(crate) type RegisteredMetaverse<T: Config> =
		StorageMap<_, Blake2_128Concat, MetaverseId, T::AccountId, OptionQuery>;

	/// Metaverse Staking snapshot per staking round
	#[pallet::storage]
	#[pallet::getter(fn get_metaverse_staking_snapshots)]
	pub(crate) type MetaverseStakingSnapshots<T: Config> =
		StorageMap<_, Blake2_128Concat, RoundIndex, MetaverseStakingSnapshot<BalanceOf<T>>>;

	/// Stores amount staked and stakers for individual metaverse per staking round
	#[pallet::storage]
	#[pallet::getter(fn get_metaverse_stake_per_round)]
	pub(crate) type MetaverseRoundStake<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		MetaverseId,
		Twox64Concat,
		RoundIndex,
		MetaverseStakingPoints<T::AccountId, BalanceOf<T>>,
	>;

	/// Keep track of staking info of individual staker
	#[pallet::storage]
	#[pallet::getter(fn staking_info)]
	pub(crate) type StakingInfo<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewMetaverseCreated(MetaverseId, T::AccountId),
		TransferredMetaverse(MetaverseId, T::AccountId, T::AccountId),
		MetaverseFreezed(MetaverseId),
		MetaverseDestroyed(MetaverseId),
		MetaverseUnfreezed(MetaverseId),
		MetaverseMintedNewCurrency(MetaverseId, FungibleTokenId),
		NewMetaverseRegisteredForStaking(MetaverseId, T::AccountId),
		MetaverseStaked(T::AccountId, MetaverseId, BalanceOf<T>),
		MetaverseUnstaked(T::AccountId, MetaverseId, BalanceOf<T>),
		MetaverseStakingRewarded(T::AccountId, MetaverseId, RoundIndex, BalanceOf<T>),
		MetaverseListingFeeUpdated(MetaverseId, Perbill),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Metaverse info not found
		MetaverseInfoNotFound,
		/// Metaverse Id not found
		MetaverseIdNotFound,
		/// No permission
		NoPermission,
		/// No available Metaverse id
		NoAvailableMetaverseId,
		/// Fungible token already issued
		FungibleTokenAlreadyIssued,
		/// Max metadata exceed
		MaxMetadataExceeded,
		/// Contribution is insufficient
		InsufficientContribution,
		/// Only frozen metaverse can be destroy
		OnlyFrozenMetaverseCanBeDestroyed,
		/// Already registered for staking
		AlreadyRegisteredForStaking,
		/// Metaverse is not registered for staking
		NotRegisteredForStaking,
		/// Not enough balance to stake
		NotEnoughBalanceToStake,
		/// Maximum amount of allowed stakers per metaverse
		MaximumAmountOfStakersPerMetaverse,
		/// Minimum staking balance is not met
		MinimumStakingAmountRequired,
		/// Exceed staked amount
		InsufficientBalanceToUnstake,
		/// Metaverse Staking Info not found
		MetaverseStakingInfoNotFound,
		/// Reward has been paid
		MetaverseStakingAlreadyPaid,
		/// Metaverse has no stake
		MetaverseHasNoStake,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::create_metaverse())]
		pub fn create_metaverse(origin: OriginFor<T>, metadata: MetaverseMetadata) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let metaverse_id = Self::do_create_metaverse(&who, metadata)?;
			Self::deposit_event(Event::<T>::NewMetaverseCreated(metaverse_id, who));

			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::transfer_metaverse())]
		pub fn transfer_metaverse(
			origin: OriginFor<T>,
			to: T::AccountId,
			metaverse_id: MetaverseId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			// Get owner of the metaverse
			MetaverseOwner::<T>::try_mutate_exists(
				&who,
				&metaverse_id,
				|metaverse_by_owner| -> DispatchResultWithPostInfo {
					// Ensure there is record of the metaverse owner with metaverse id, account
					// id and delete them
					ensure!(metaverse_by_owner.is_some(), Error::<T>::NoPermission);

					if who == to {
						// No change needed
						return Ok(().into());
					}

					*metaverse_by_owner = None;
					MetaverseOwner::<T>::insert(to.clone(), metaverse_id.clone(), ());

					Metaverses::<T>::try_mutate_exists(&metaverse_id, |metaverse| -> DispatchResultWithPostInfo {
						let mut metaverse_record = metaverse.as_mut().ok_or(Error::<T>::NoPermission)?;
						metaverse_record.owner = to.clone();
						Self::deposit_event(Event::<T>::TransferredMetaverse(metaverse_id, who.clone(), to.clone()));

						Ok(().into())
					})
				},
			)
		}

		#[pallet::weight(T::WeightInfo::freeze_metaverse())]
		pub fn freeze_metaverse(origin: OriginFor<T>, metaverse_id: MetaverseId) -> DispatchResultWithPostInfo {
			// Only Council can freeze a metaverse
			T::MetaverseCouncil::ensure_origin(origin)?;

			Metaverses::<T>::try_mutate(metaverse_id, |maybe_metaverse| {
				let metaverse_info = maybe_metaverse.as_mut().ok_or(Error::<T>::MetaverseInfoNotFound)?;

				metaverse_info.is_frozen = true;

				Self::deposit_event(Event::<T>::MetaverseFreezed(metaverse_id));

				Ok(().into())
			})
		}

		#[pallet::weight(T::WeightInfo::unfreeze_metaverse())]
		pub fn unfreeze_metaverse(origin: OriginFor<T>, metaverse_id: MetaverseId) -> DispatchResultWithPostInfo {
			// Only Council can freeze a metaverse
			T::MetaverseCouncil::ensure_origin(origin)?;

			Metaverses::<T>::try_mutate(metaverse_id, |maybe_metaverse| {
				let metaverse_info = maybe_metaverse.as_mut().ok_or(Error::<T>::MetaverseInfoNotFound)?;

				metaverse_info.is_frozen = false;

				Self::deposit_event(Event::<T>::MetaverseUnfreezed(metaverse_id));

				Ok(().into())
			})
		}

		#[pallet::weight(T::WeightInfo::destroy_metaverse())]
		pub fn destroy_metaverse(origin: OriginFor<T>, metaverse_id: MetaverseId) -> DispatchResultWithPostInfo {
			// Only Council can destroy a metaverse
			T::MetaverseCouncil::ensure_origin(origin)?;

			let metaverse_info = Metaverses::<T>::get(metaverse_id).ok_or(Error::<T>::MetaverseInfoNotFound)?;

			ensure!(metaverse_info.is_frozen, Error::<T>::OnlyFrozenMetaverseCanBeDestroyed);

			MetaverseOwner::<T>::remove(metaverse_info.owner, &metaverse_id);
			Metaverses::<T>::remove(&metaverse_id);
			Self::deposit_event(Event::<T>::MetaverseDestroyed(metaverse_id));
			Ok(().into())
		}

		/// Register metaverse for staking
		/// only metaverse owner can register for staking
		#[pallet::weight(T::WeightInfo::register_metaverse())]
		pub fn register_metaverse(origin: OriginFor<T>, metaverse_id: MetaverseId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let metaverse_info = Self::get_metaverse(metaverse_id).ok_or(Error::<T>::MetaverseInfoNotFound)?;
			ensure!(metaverse_info.owner == who, Error::<T>::NoPermission);

			ensure!(
				!RegisteredMetaverse::<T>::contains_key(&metaverse_id),
				Error::<T>::AlreadyRegisteredForStaking
			);

			T::Currency::reserve(&who, T::MetaverseRegistrationDeposit::get())?;

			RegisteredMetaverse::<T>::insert(metaverse_id.clone(), who.clone());

			Self::deposit_event(Event::<T>::NewMetaverseRegisteredForStaking(metaverse_id, who));

			Ok(().into())
		}

		/// Lock up and stake balance of the origin account.
		/// New stake will be applied at the beginning of the next round.
		#[pallet::weight(T::WeightInfo::stake())]
		pub fn stake(
			origin: OriginFor<T>,
			metaverse_id: MetaverseId,
			#[pallet::compact] value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Check that metaverse is registered for staking.
			ensure!(
				RegisteredMetaverse::<T>::contains_key(&metaverse_id),
				Error::<T>::NotRegisteredForStaking
			);

			// Get the staking ledger or create an entry if it doesn't exist.
			let mut staking_info = Self::staking_info(&who);

			// Ensure that staker has enough balance to stake.
			let free_balance = T::Currency::free_balance(&who).saturating_sub(T::MinStakingAmount::get());

			// Remove already locked funds from the free balance
			let available_balance = free_balance.saturating_sub(staking_info);
			let stake_amount = value.min(available_balance);
			ensure!(stake_amount > Zero::zero(), Error::<T>::NotEnoughBalanceToStake);

			// Get the latest round staking point info or create it if metaverse hasn't been staked yet so far.
			let current_staking_round: RoundInfo<T::BlockNumber> = Self::staking_round();

			if !MetaverseRoundStake::<T>::contains_key(&metaverse_id, current_staking_round.current) {
				let stakers: BTreeMap<T::AccountId, BalanceOf<T>> = BTreeMap::new();

				let new_metaverse_stake_per_round: MetaverseStakingPoints<T::AccountId, BalanceOf<T>> =
					MetaverseStakingPoints {
						total: 0u32.into(),
						claimed_rewards: 0u32.into(),
						stakers: stakers,
					};

				// Update staked information for contract in current round
				MetaverseRoundStake::<T>::insert(
					metaverse_id.clone(),
					current_staking_round.current,
					new_metaverse_stake_per_round,
				);
			}

			// Get staking info of metaverse and current round
			let mut metaverse_stake_per_round: MetaverseStakingPoints<T::AccountId, BalanceOf<T>> =
				Self::get_metaverse_stake_per_round(&metaverse_id, current_staking_round.current)
					.ok_or(Error::<T>::MetaverseStakingInfoNotFound)?;

			// Ensure that we can add additional staker for the metaverse.
			ensure!(
				metaverse_stake_per_round.stakers.len() < T::MaxNumberOfStakersPerMetaverse::get() as usize,
				Error::<T>::MaximumAmountOfStakersPerMetaverse
			);
			// Increment ledger and total staker value for a metaverse.
			staking_info = staking_info
				.checked_add(&stake_amount)
				.ok_or(ArithmeticError::Overflow)?;

			let individual_staker = metaverse_stake_per_round.stakers.entry(who.clone()).or_default();
			*individual_staker = individual_staker
				.checked_add(&stake_amount)
				.ok_or(ArithmeticError::Overflow)?;

			ensure!(
				*individual_staker >= T::MinStakingAmount::get(),
				Error::<T>::MinimumStakingAmountRequired,
			);

			// Update total staked value in current round
			MetaverseStakingSnapshots::<T>::mutate(current_staking_round.current, |may_be_staking_snapshot| {
				if let Some(snapshot) = may_be_staking_snapshot {
					snapshot.staked = snapshot.staked.saturating_add(stake_amount)
				}
			});

			// Update staking info of origin
			Self::update_staking_info(&who, staking_info);

			// Update staked information for contract in current round
			MetaverseRoundStake::<T>::insert(
				metaverse_id.clone(),
				current_staking_round.current,
				metaverse_stake_per_round,
			);

			Self::deposit_event(Event::<T>::MetaverseStaked(who.clone(), metaverse_id, stake_amount));
			Ok(().into())
		}

		/// Unstake and withdraw balance of the origin account.
		/// If user unstake below minimum staking amount, the entire staking of that origin will be
		/// removed Unstake will on be kicked off from the begining of the next round.
		#[pallet::weight(T::WeightInfo::unstake_and_withdraw())]
		pub fn unstake_and_withdraw(
			origin: OriginFor<T>,
			metaverse_id: MetaverseId,
			#[pallet::compact] value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Check that metaverse is registered for staking.
			ensure!(
				RegisteredMetaverse::<T>::contains_key(&metaverse_id),
				Error::<T>::NotRegisteredForStaking
			);

			// Get the latest round staking point info.
			let current_staking_round: RoundInfo<T::BlockNumber> = Self::staking_round();

			// Get staking info of metaverse and current round
			let mut metaverse_stake_per_round: MetaverseStakingPoints<T::AccountId, BalanceOf<T>> =
				Self::get_metaverse_stake_per_round(&metaverse_id, current_staking_round.current)
					.ok_or(Error::<T>::MetaverseStakingInfoNotFound)?;

			ensure!(
				metaverse_stake_per_round.stakers.contains_key(&who),
				Error::<T>::NoPermission
			);

			let staked_amount = metaverse_stake_per_round.stakers[&who];

			ensure!(value <= staked_amount, Error::<T>::InsufficientBalanceToUnstake);

			let remaining = staked_amount.saturating_sub(value);
			let amount_to_unstake = if remaining < T::MinStakingAmount::get() {
				// Remaining amount below minimum, remove all staked amount
				metaverse_stake_per_round.stakers.remove(&who);
				staked_amount
			} else {
				metaverse_stake_per_round.stakers.insert(who.clone(), remaining);
				value
			};

			let staking_info = Self::staking_info(&who);
			Self::update_staking_info(&who, staking_info.saturating_sub(amount_to_unstake));

			// Update total staked value in current round
			MetaverseStakingSnapshots::<T>::mutate(current_staking_round.current, |may_be_staking_snapshot| {
				if let Some(snapshot) = may_be_staking_snapshot {
					snapshot.staked = snapshot.staked.saturating_sub(amount_to_unstake)
				}
			});

			metaverse_stake_per_round.total = metaverse_stake_per_round.total.saturating_sub(amount_to_unstake);
			// Update staked information for contract in current round
			MetaverseRoundStake::<T>::insert(
				metaverse_id.clone(),
				current_staking_round.current,
				metaverse_stake_per_round,
			);

			Self::deposit_event(Event::<T>::MetaverseUnstaked(
				who.clone(),
				metaverse_id,
				amount_to_unstake,
			));

			Ok(().into())
		}

		/// Pay staker reward of the round per metaverse
		/// Get all
		#[pallet::weight(T::WeightInfo::unstake_and_withdraw())]
		pub fn pay_staker(
			origin: OriginFor<T>,
			metaverse_id: MetaverseId,
			round: RoundIndex,
		) -> DispatchResultWithPostInfo {
			let _ = ensure_signed(origin);
			// Get staking info of metaverse and current round
			let mut metaverse_stake_per_round: MetaverseStakingPoints<T::AccountId, BalanceOf<T>> =
				Self::get_metaverse_stake_per_round(&metaverse_id, round)
					.ok_or(Error::<T>::MetaverseStakingInfoNotFound)?;

			ensure!(
				metaverse_stake_per_round.claimed_rewards.is_zero(),
				Error::<T>::MetaverseStakingAlreadyPaid
			);

			ensure!(
				!metaverse_stake_per_round.stakers.is_empty(),
				Error::<T>::MetaverseHasNoStake
			);

			// Get total reward from staking snapshot - which updated every mining round.

			// Update total staked value in current round - for accounting purpose
			let metaverse_staking_snapshot =
				MetaverseStakingSnapshots::<T>::get(round).ok_or(Error::<T>::MetaverseStakingInfoNotFound)?;

			let mut total_rewward_per_metaverse: BalanceOf<T> = Default::default();

			for (staker, staked_amount) in &metaverse_stake_per_round.stakers {
				let ratio = Perbill::from_rational(*staked_amount, metaverse_stake_per_round.total);
				let staking_reward = ratio * metaverse_staking_snapshot.rewards;

				let balance_staking_reward = TryInto::<BalanceOf<T>>::try_into(staking_reward).unwrap_or_default();

				total_rewward_per_metaverse = total_rewward_per_metaverse
					.checked_add(&balance_staking_reward)
					.ok_or(ArithmeticError::Overflow)?;
				Self::deposit_event(Event::<T>::MetaverseStakingRewarded(
					staker.clone(),
					metaverse_id.clone(),
					round,
					staking_reward,
				));

				T::MultiCurrency::deposit(FungibleTokenId::MiningResource(0), staker, staking_reward);
			}

			metaverse_stake_per_round.claimed_rewards = total_rewward_per_metaverse;
			<MetaverseRoundStake<T>>::insert(&metaverse_id, round, metaverse_stake_per_round);
			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::update_metaverse_listing_fee())]
		pub fn update_metaverse_listing_fee(
			origin: OriginFor<T>,
			metaverse_id: MetaverseId,
			new_listing_fee: Perbill,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			Self::do_update_metaverse_listing_fee(&who, &metaverse_id, new_listing_fee)?;
			Self::deposit_event(Event::<T>::MetaverseListingFeeUpdated(metaverse_id, new_listing_fee));

			Ok(().into())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}

impl<T: Config> Pallet<T> {
	fn new_metaverse(owner: &T::AccountId, metadata: MetaverseMetadata) -> Result<MetaverseId, DispatchError> {
		let metaverse_id = NextMetaverseId::<T>::try_mutate(|id| -> Result<MetaverseId, DispatchError> {
			let current_id = *id;
			*id = id.checked_add(One::one()).ok_or(Error::<T>::NoAvailableMetaverseId)?;
			Ok(current_id)
		})?;
		let land_class_id = Self::mint_metaverse_land_class(owner, metaverse_id)?;
		let estate_class_id = Self::mint_metaverse_estate_class(owner, metaverse_id)?;
		let metaverse_info = MetaverseInfo {
			owner: owner.clone(),
			currency_id: FungibleTokenId::NativeToken(0),
			metadata,
			is_frozen: false,
			land_class_id,
			estate_class_id,
			listing_fee: Perbill::from_percent(0u32),
		};

		Metaverses::<T>::insert(metaverse_id, metaverse_info);

		Ok(metaverse_id)
	}

	fn do_create_metaverse(who: &T::AccountId, metadata: MetaverseMetadata) -> Result<MetaverseId, DispatchError> {
		ensure!(
			metadata.len() as u32 <= T::MaxMetaverseMetadata::get(),
			Error::<T>::MaxMetadataExceeded
		);

		ensure!(
			T::Currency::free_balance(&who) >= T::MinContribution::get(),
			Error::<T>::InsufficientContribution
		);

		T::Currency::transfer(
			&who,
			&Self::account_id(),
			T::MinContribution::get(),
			ExistenceRequirement::KeepAlive,
		)?;
		let metaverse_id = Self::new_metaverse(&who, metadata)?;

		MetaverseOwner::<T>::insert(who.clone(), metaverse_id, ());

		let total_metaverse_count = Self::all_metaverse_count();
		let new_total_metaverse_count = total_metaverse_count
			.checked_add(One::one())
			.ok_or("Overflow adding new count to new_total_metaverse_count")?;
		AllMetaversesCount::<T>::put(new_total_metaverse_count);
		Ok(metaverse_id)
	}

	/// The account ID of the treasury pot.
	///
	/// This actually does computation. If you need to keep using it, then make sure you cache the
	/// value and only call this once.
	pub fn account_id() -> T::AccountId {
		T::MetaverseTreasury::get().into_account()
	}

	/// Update staking info of origin
	fn update_staking_info(who: &T::AccountId, staking_info: BalanceOf<T>) {
		if staking_info.is_zero() {
			StakingInfo::<T>::remove(&who);
			T::Currency::remove_lock(LOCK_STAKING, &who);
		} else {
			T::Currency::set_lock(LOCK_STAKING, &who, staking_info, WithdrawReasons::all());
			StakingInfo::<T>::insert(who, staking_info);
		}
	}

	fn mint_metaverse_land_class(sender: &T::AccountId, metaverse_id: MetaverseId) -> Result<ClassId, DispatchError> {
		// Pre-mint class for lands
		let mut land_class_attributes = Attributes::new();
		land_class_attributes.insert("MetaverseId:".as_bytes().to_vec(), "MetaverseId:".as_bytes().to_vec());
		land_class_attributes.insert("Category:".as_bytes().to_vec(), "Lands".as_bytes().to_vec());
		let land_class_metadata: NftMetadata = metaverse_id.to_be_bytes().to_vec();
		let class_owner: T::AccountId = T::MetaverseTreasury::get().into_account();
		T::NFTHandler::create_token_class(
			&class_owner,
			land_class_metadata,
			land_class_attributes,
			0,
			TokenType::Transferable,
			CollectionType::Collectable,
			Perbill::from_percent(LAND_CLASS_ROYALTY_FEE),
			None,
		)
	}

	fn mint_metaverse_estate_class(sender: &T::AccountId, metaverse_id: MetaverseId) -> Result<ClassId, DispatchError> {
		// Pre-mint class for estates
		let mut estate_class_attributes = Attributes::new();
		estate_class_attributes.insert("MetaverseId:".as_bytes().to_vec(), metaverse_id.to_be_bytes().to_vec());
		estate_class_attributes.insert("Category:".as_bytes().to_vec(), "Estates".as_bytes().to_vec());
		let estate_class_metadata: NftMetadata = metaverse_id.to_be_bytes().to_vec();
		let class_owner: T::AccountId = T::MetaverseTreasury::get().into_account();
		T::NFTHandler::create_token_class(
			&class_owner,
			estate_class_metadata,
			estate_class_attributes,
			0,
			TokenType::Transferable,
			CollectionType::Collectable,
			Perbill::from_percent(ESTATE_CLASS_ROYALTY_FEE),
			None,
		)
	}

	fn do_update_metaverse_listing_fee(
		who: &T::AccountId,
		metaverse_id: &MetaverseId,
		new_listing_fee: Perbill,
	) -> Result<(), DispatchError> {
		ensure!(Self::check_ownership(who, metaverse_id), Error::<T>::NoPermission);

		Metaverses::<T>::try_mutate(metaverse_id, |metaverse_info| -> DispatchResult {
			let t = metaverse_info.as_mut().ok_or(Error::<T>::MetaverseInfoNotFound)?;
			t.listing_fee = new_listing_fee;
			Ok(())
		})
	}

	pub fn upgrade_metaverse_info_v2() -> Weight {
		log::info!("Start upgrade_metaverse_info_v2");
		let mut num_metaverse_items = 0;

		let default_land_class_id = TryInto::<ClassId>::try_into(0u32).unwrap_or_default();
		let default_estate_class_id = TryInto::<ClassId>::try_into(1u32).unwrap_or_default();

		Metaverses::<T>::translate(|_k, metaverse_info_v1: MetaverseInfoV1<T::AccountId>| {
			num_metaverse_items += 1;
			let v2: MetaverseInfo<T::AccountId> = MetaverseInfo {
				owner: metaverse_info_v1.owner,
				metadata: metaverse_info_v1.metadata,
				currency_id: metaverse_info_v1.currency_id,
				is_frozen: false,
				listing_fee: Perbill::from_percent(0u32),
				land_class_id: default_land_class_id,
				estate_class_id: default_estate_class_id,
			};
			Some(v2)
		});

		log::info!("{} metaverses upgraded:", num_metaverse_items);
		0
	}
}

impl<T: Config> MetaverseTrait<T::AccountId> for Pallet<T> {
	fn create_metaverse(who: &T::AccountId, metadata: MetaverseMetadata) -> MetaverseId {
		Self::do_create_metaverse(who, metadata).unwrap_or_default()
	}

	fn check_ownership(who: &T::AccountId, metaverse_id: &MetaverseId) -> bool {
		Self::get_metaverse_owner(who, metaverse_id) == Some(())
	}

	fn get_metaverse(metaverse_id: MetaverseId) -> Option<MetaverseInfo<T::AccountId>> {
		Self::get_metaverse(metaverse_id)
	}

	fn get_metaverse_token(metaverse_id: MetaverseId) -> Option<FungibleTokenId> {
		if let Some(country) = Self::get_metaverse(metaverse_id) {
			return Some(country.currency_id);
		}
		None
	}

	fn update_metaverse_token(metaverse_id: MetaverseId, currency_id: FungibleTokenId) -> Result<(), DispatchError> {
		Metaverses::<T>::try_mutate_exists(&metaverse_id, |metaverse| {
			let mut metaverse_record = metaverse.as_mut().ok_or(Error::<T>::NoPermission)?;

			ensure!(
				metaverse_record.currency_id == FungibleTokenId::NativeToken(0),
				Error::<T>::FungibleTokenAlreadyIssued
			);

			metaverse_record.currency_id = currency_id.clone();
			Self::deposit_event(Event::<T>::MetaverseMintedNewCurrency(metaverse_id, currency_id));
			Ok(())
		})
	}

	fn get_metaverse_land_class(metaverse_id: MetaverseId) -> Result<ClassId, DispatchError> {
		let metaverse_info = Self::get_metaverse(metaverse_id).ok_or(Error::<T>::MetaverseInfoNotFound)?;
		Ok(TryInto::<ClassId>::try_into(metaverse_info.land_class_id).unwrap_or_default())
	}

	fn get_metaverse_estate_class(metaverse_id: MetaverseId) -> Result<ClassId, DispatchError> {
		let metaverse_info = Self::get_metaverse(metaverse_id).ok_or(Error::<T>::MetaverseInfoNotFound)?;
		Ok(TryInto::<ClassId>::try_into(metaverse_info.estate_class_id).unwrap_or_default())
	}

	fn get_metaverse_marketplace_listing_fee(metaverse_id: MetaverseId) -> Result<Perbill, DispatchError> {
		let metaverse_info = Metaverses::<T>::get(metaverse_id).ok_or(Error::<T>::MetaverseInfoNotFound)?;

		Ok(metaverse_info.listing_fee)
	}

	fn get_metaverse_treasury(metaverse_id: MetaverseId) -> T::AccountId {
		return T::MetaverseTreasury::get().into_account();
	}

	fn get_network_treasury() -> T::AccountId {
		return T::MetaverseTreasury::get().into_account();
	}
}

impl<T: Config> MetaverseStakingTrait<BalanceOf<T>> for Pallet<T> {
	fn update_staking_reward(round: RoundIndex, total_reward: BalanceOf<T>) -> DispatchResult {
		// Update total reward value of current round - for reward distribution
		MetaverseStakingSnapshots::<T>::mutate(round, |may_be_staking_snapshot| {
			if let Some(snapshot) = may_be_staking_snapshot {
				snapshot.rewards = total_reward
			}
		});

		Ok(())
	}
}
