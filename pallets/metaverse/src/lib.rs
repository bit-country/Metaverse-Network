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

use codec::{Decode, Encode, HasCompact};
use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement, LockableCurrency, ReservableCurrency},
	PalletId,
};
use frame_system::{ensure_signed, pallet_prelude::*};
use sp_runtime::{
	traits::{AccountIdConversion, One, Zero},
	DispatchError,
};
use sp_std::{collections::btree_map::BTreeMap, prelude::*};

use bc_primitives::*;
use bc_primitives::{MetaverseInfo, MetaverseTrait};
pub use pallet::*;
use primitives::{FungibleTokenId, MetaverseId};
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
	use sp_runtime::traits::Saturating;
	use sp_runtime::ArithmeticError;

	use primitives::staking::RoundInfo;
	use primitives::RoundIndex;

	use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(trait Store)]
	pub struct Pallet<T>(PhantomData<T>);

	type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The currency type
		type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>
			+ ReservableCurrency<Self::AccountId>;
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
		/// Weight implementation for estate extrinsics
		type WeightInfo: WeightInfo;
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

	/// Keep track of staking ledger of individual staker
	#[pallet::storage]
	#[pallet::getter(fn staking_ledger)]
	pub(crate) type StakingLedger<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

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
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::create_metaverse())]
		pub fn create_metaverse(
			origin: OriginFor<T>,
			owner: T::AccountId,
			metadata: MetaverseMetadata,
		) -> DispatchResultWithPostInfo {
			// Only Council can create a metaverse
			T::MetaverseCouncil::ensure_origin(origin)?;

			ensure!(
				metadata.len() as u32 <= T::MaxMetaverseMetadata::get(),
				Error::<T>::MaxMetadataExceeded
			);

			ensure!(
				T::Currency::free_balance(&owner) >= T::MinContribution::get(),
				Error::<T>::InsufficientContribution
			);

			T::Currency::transfer(
				&owner,
				&Self::account_id(),
				T::MinContribution::get(),
				ExistenceRequirement::KeepAlive,
			)?;

			let metaverse_id = Self::new_metaverse(&owner, metadata)?;

			MetaverseOwner::<T>::insert(owner.clone(), metaverse_id, ());

			let total_metaverse_count = Self::all_metaverse_count();
			let new_total_metaverse_count = total_metaverse_count
				.checked_add(One::one())
				.ok_or("Overflow adding new count to new_total_metaverse_count")?;
			AllMetaversesCount::<T>::put(new_total_metaverse_count);
			Self::deposit_event(Event::<T>::NewMetaverseCreated(metaverse_id.clone(), owner));

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
		#[pallet::weight(10_000)]
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
		#[pallet::weight(100_000)]
		pub fn stake(
			origin: OriginFor<T>,
			metaverse_id: MetaverseId,
			#[pallet::compact] value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let staker = ensure_signed(origin)?;

			// Check that metaverse is registered for staking.
			ensure!(
				RegisteredMetaverse::<T>::contains_key(&metaverse_id),
				Error::<T>::NotRegisteredForStaking
			);

			// Get the staking ledger or create an entry if it doesn't exist.
			let mut staking_ledger = Self::staking_ledger(&staker);

			// Ensure that staker has enough balance to stake.
			let free_balance = T::Currency::free_balance(&staker).saturating_sub(T::MinStakingAmount::get());

			// Remove already locked funds from the free balance
			let available_balance = free_balance.saturating_sub(staking_ledger);
			let value_to_stake = value.min(available_balance);
			ensure!(value_to_stake > Zero::zero(), Error::<T>::NotEnoughBalanceToStake);

			// Get the latest round staking point info or create it if metaverse hasn't been staked yet so far.
			let current_staking_round = Self::staking_round();
			// get staking info of metaverse and current round

			// Ensure that we can add additional staker for the metaverse.

			// Increment ledger and total staker value for a metaverse.
			// Update total staked value in current round
			// Update ledger and payee
			// Update staked information for contract in current round
			Ok(().into())
		}

		/// Unstake and withdraw balance of the origin account.
		/// If user unstake below minimum staking amount, the entire staking of that origin will be
		/// removed Unstake will on be kicked off from the begining of the next round.
		#[pallet::weight(100_000)]
		pub fn unstake_and_withdraw(
			origin: OriginFor<T>,
			metaverse_id: MetaverseId,
			#[pallet::compact] value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let staker = ensure_signed(origin)?;

			// Check that metaverse is registered for staking.
			ensure!(
				RegisteredMetaverse::<T>::contains_key(&metaverse_id),
				Error::<T>::NotRegisteredForStaking
			);

			// Get the staking ledger or create an entry if it doesn't exist.
			let mut staking_ledger = Self::staking_ledger(&staker);

			// Ensure that staker has enough balance to stake.
			let free_balance = T::Currency::free_balance(&staker).saturating_sub(T::MinStakingAmount::get());

			// Remove already locked funds from the free balance
			let available_balance = free_balance.saturating_sub(staking_ledger);
			let value_to_stake = value.min(available_balance);
			ensure!(value_to_stake > Zero::zero(), Error::<T>::NotEnoughBalanceToStake);

			// Get the latest round staking point info or create it if metaverse hasn't been staked yet so far.
			let current_staking_round = Self::staking_round();
			// get staking info of metaverse and current round

			// Ensure that we can add additional staker for the metaverse.

			// Increment ledger and total staker value for a metaverse.
			// Update total staked value in current round
			// Update ledger and payee
			// Update staked information for contract in current round
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

		let metaverse_info = MetaverseInfo {
			owner: owner.clone(),
			currency_id: FungibleTokenId::NativeToken(0),
			metadata,
			is_frozen: false,
		};

		Metaverses::<T>::insert(metaverse_id, metaverse_info);

		Ok(metaverse_id)
	}

	/// The account ID of the treasury pot.
	///
	/// This actually does computation. If you need to keep using it, then make sure you cache the
	/// value and only call this once.
	pub fn account_id() -> T::AccountId {
		T::MetaverseTreasury::get().into_account()
	}
}

impl<T: Config> MetaverseTrait<T::AccountId> for Pallet<T> {
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
}
