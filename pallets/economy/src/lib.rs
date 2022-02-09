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
use frame_support::traits::{LockIdentifier, WithdrawReasons};
use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement, LockableCurrency, ReservableCurrency},
	PalletId,
};
use frame_system::{ensure_signed, pallet_prelude::*};
use orml_nft::Pallet as NftModule;
use orml_traits::MultiCurrency;
use sp_runtime::{
	traits::{AccountIdConversion, One, Zero},
	DispatchError, Perbill,
};
use sp_std::{collections::btree_map::BTreeMap, prelude::*};

use bc_primitives::*;
use bc_primitives::{MetaverseInfo, MetaverseTrait};
pub use pallet::*;
use primitives::{AssetId, FungibleTokenId, MetaverseId, RoundIndex};
pub use weights::WeightInfo;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;

/// A record for basic element info. i.e. price, compositions and rules
#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ElementInfo {
	/// Power price for the element
	power_price: u32,
	/// The tuple of other element index -> required amount
	compositions: Vec<(u32, u128)>,
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

	pub type ClassIdOf<T> = <T as orml_nft::Config>::ClassId;

	#[pallet::config]
	pub trait Config: frame_system::Config + orml_nft::Config {
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
	}

	#[pallet::storage]
	#[pallet::getter(fn total_staked)]
	pub type TotalStaked<T: Config> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_element_index)]
	pub type ElementIndex<T: Config> = StorageMap<_, Twox64Concat, u32, ElementInfo>;

	#[pallet::storage]
	#[pallet::getter(fn get_authorized_generator_collection)]
	pub type AuthorizedGeneratorCollection<T: Config> = StorageMap<_, Twox64Concat, ClassIdOf<T>, (), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_authorized_distributor_collection)]
	pub type AuthorizedDistributorCollection<T: Config> = StorageMap<_, Twox64Concat, ClassIdOf<T>, (), OptionQuery>;

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

	/// Keep track of staking info of individual staker
	#[pallet::storage]
	#[pallet::getter(fn staking_info)]
	pub(crate) type StakingInfo<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		PowerGeneratorCollectionAuthorized(ClassIdOf<T>),
		PowerDistributorCollectionAuthorized(ClassIdOf<T>),
		/* TransferredMetaverse(MetaverseId, T::AccountId, T::AccountId),
		 * MetaverseFreezed(MetaverseId),
		 * MetaverseDestroyed(MetaverseId),
		 * MetaverseUnfreezed(MetaverseId),
		 * MetaverseMintedNewCurrency(MetaverseId, FungibleTokenId),
		 * NewMetaverseRegisteredForStaking(MetaverseId, T::AccountId),
		 * MetaverseStaked(T::AccountId, MetaverseId, BalanceOf<T>),
		 * MetaverseUnstaked(T::AccountId, MetaverseId, BalanceOf<T>),
		 * MetaverseStakingRewarded(T::AccountId, MetaverseId, RoundIndex, BalanceOf<T>), */
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Power generator collection already authorized
		PowerGeneratorCollectionAlreadyAuthorized,
		/// Power distributor collection already authorized
		PowerDistributorCollectionAlreadyAuthorized,
		/* /// Metaverse Id not found
		 * MetaverseIdNotFound,
		 * /// No permission
		 * NoPermission,
		 * /// No available Metaverse id
		 * NoAvailableMetaverseId,
		 * /// Fungible token already issued
		 * FungibleTokenAlreadyIssued,
		 * /// Max metadata exceed
		 * MaxMetadataExceeded,
		 * /// Contribution is insufficient
		 * InsufficientContribution,
		 * /// Only frozen metaverse can be destroy
		 * OnlyFrozenMetaverseCanBeDestroyed,
		 * /// Already registered for staking
		 * AlreadyRegisteredForStaking,
		 * /// Metaverse is not registered for staking
		 * NotRegisteredForStaking,
		 * /// Not enough balance to stake
		 * NotEnoughBalanceToStake,
		 * /// Maximum amount of allowed stakers per metaverse
		 * MaximumAmountOfStakersPerMetaverse,
		 * /// Minimum staking balance is not met
		 * MinimumStakingAmountRequired,
		 * /// Exceed staked amount
		 * InsufficientBalanceToUnstake,
		 * /// Metaverse Staking Info not found
		 * MetaverseStakingInfoNotFound,
		 * /// Reward has been paid
		 * MetaverseStakingAlreadyPaid,
		 * /// Metaverse has no stake
		 * MetaverseHasNoStake, */
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Authorize a NFT collector for power generator
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn authorize_power_generator_collection(
			origin: OriginFor<T>,
			class_id: ClassIdOf<T>,
		) -> DispatchResultWithPostInfo {
			// Only Council can create a metaverse
			ensure_root(origin)?;

			// Check that NFT collection is not authorized already
			ensure!(
				!AuthorizedGeneratorCollection::<T>::contains_key(&class_id),
				Error::<T>::PowerGeneratorCollectionAlreadyAuthorized
			);

			AuthorizedGeneratorCollection::<T>::insert(&class_id, ());

			Self::deposit_event(Event::<T>::PowerGeneratorCollectionAuthorized(class_id.clone()));

			Ok(().into())
		}

		/// Authorize a NFT collector for power distributor
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn authorize_power_distributor_collection(
			origin: OriginFor<T>,
			class_id: ClassIdOf<T>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			// Check that NFT collection is not authorized already
			ensure!(
				!AuthorizedDistributorCollection::<T>::contains_key(&class_id),
				Error::<T>::PowerDistributorCollectionAlreadyAuthorized
			);

			AuthorizedDistributorCollection::<T>::insert(&class_id, ());

			Self::deposit_event(Event::<T>::PowerDistributorCollectionAuthorized(class_id.clone()));

			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn buy_power_by_user(
			origin: OriginFor<T>,
			power_amount: u32,
			distributor_nft_id: AssetId,
		) -> DispatchResultWithPostInfo {
			// Only Council can freeze a metaverse
			T::MetaverseCouncil::ensure_origin(origin)?;

			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn execute_buy_power_order(
			origin: OriginFor<T>,
			distributor_nft_id: AssetId,
			beneficiary: T::AccountId,
		) -> DispatchResultWithPostInfo {
			// Only Council can freeze a metaverse
			T::MetaverseCouncil::ensure_origin(origin)?;

			Ok(().into())
		}

		/// Register metaverse for staking
		/// only metaverse owner can register for staking
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn buy_power_by_distributor(
			origin: OriginFor<T>,
			generator_nft_id: AssetId,
			distributor_nft_id: AssetId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Ok(().into())
		}

		/// Lock up and stake balance of the origin account.
		/// New stake will be applied at the beginning of the next round.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn execute_generate_power_order(
			origin: OriginFor<T>,
			generator_nft_id: AssetId,
			beneficiary: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Ok(().into())
		}

		/// Unstake and withdraw balance of the origin account.
		/// If user unstake below minimum staking amount, the entire staking of that origin will be
		/// removed Unstake will on be kicked off from the begining of the next round.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn mint_element(
			origin: OriginFor<T>,
			element_index: u32,
			number_of_element: u32,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn distribute_power_by_operator(
		power_amount: u128,
		beneficiary: T::AccountId,
		distributor_nft_id: AssetId,
	) -> DispatchResultWithPostInfo {
		// Get staking info of metaverse and current round

		Ok(().into())
	}

	fn generate_power_by_operator(
		power_amount: u128,
		generator_nft_id: AssetId,
		distributor_nft_id: AssetId,
	) -> DispatchResultWithPostInfo {
		// Get staking info of metaverse and current round

		Ok(().into())
	}
}
