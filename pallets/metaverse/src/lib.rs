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
use sp_runtime::traits::{CheckedSub, Saturating};
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

// #[cfg(feature = "runtime-benchmarks")]
// pub mod benchmarking;

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
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
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
		/// The metaverse treasury pallet
		#[pallet::constant]
		type MetaverseTreasury: Get<PalletId>;
		/// The  maximum metaverse metadata size
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
	/// Track the next metaverse ID.
	pub type NextMetaverseId<T: Config> = StorageValue<_, MetaverseId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_metaverse)]
	/// Stores metaverses' informaion.
	pub type Metaverses<T: Config> = StorageMap<_, Twox64Concat, MetaverseId, MetaverseInfo<T::AccountId>>;

	#[pallet::storage]
	#[pallet::getter(fn get_metaverse_owner)]
	/// Stores metaverses owned by each account.
	pub type MetaverseOwner<T: Config> = StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, MetaverseId, ()>;

	#[pallet::storage]
	#[pallet::getter(fn all_metaverse_count)]
	/// Track the total amount of metaverses.
	pub(super) type AllMetaversesCount<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_freezing_metaverse)]
	/// Stores frozen metaverses.
	pub(super) type FreezedMetaverses<T: Config> = StorageMap<_, Twox64Concat, MetaverseId, (), OptionQuery>;

	/// Metaverse staking related storage

	#[pallet::storage]
	#[pallet::getter(fn staking_round)]
	/// Tracks current staking round index and next round scheduled transition.
	pub type Round<T: Config> = StorageValue<_, RoundInfo<T::BlockNumber>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_registered_metaverse)]
	/// Stores metaverses registered for staking.
	pub(crate) type RegisteredMetaverse<T: Config> =
		StorageMap<_, Blake2_128Concat, MetaverseId, T::AccountId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_metaverse_staking_snapshots)]
	/// Stores metaverse staking snapshot for a staking round.
	pub(crate) type MetaverseStakingSnapshots<T: Config> =
		StorageMap<_, Blake2_128Concat, RoundIndex, MetaverseStakingSnapshot<BalanceOf<T>>>;

	#[pallet::storage]
	#[pallet::getter(fn get_metaverse_stake_per_round)]
	/// Stores amount staked and stakers for individual metaverse per staking round.
	pub(crate) type MetaverseRoundStake<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		MetaverseId,
		Twox64Concat,
		RoundIndex,
		MetaverseStakingPoints<T::AccountId, BalanceOf<T>>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn staking_info)]
	/// Stores staking info of individual stakers.
	pub(crate) type StakingInfo<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Successfully created a metaverse
		NewMetaverseCreated(MetaverseId, T::AccountId),
		/// Successfully transferred a metaverse
		TransferredMetaverse(MetaverseId, T::AccountId, T::AccountId),
		/// Successfully frozen a metaverse
		MetaverseFreezed(MetaverseId),
		/// Successfully destroyed a metaverse
		MetaverseDestroyed(MetaverseId),
		/// Successfully unfreezed a metaverse
		MetaverseUnfreezed(MetaverseId),
		/// Successfully minted new metaverse currency
		MetaverseMintedNewCurrency(MetaverseId, FungibleTokenId),
		/// Successfully registerred a metaverse for staking
		NewMetaverseRegisteredForStaking(MetaverseId, T::AccountId),
		/// Successfully staked funds to a metaverse
		MetaverseStaked(T::AccountId, MetaverseId, BalanceOf<T>),
		/// Successfully unstaked funds from a metaverse
		MetaverseUnstaked(T::AccountId, MetaverseId, BalanceOf<T>),
		/// Successfully payed rewards for a metaverse staking round
		MetaverseStakingRewarded(T::AccountId, MetaverseId, RoundIndex, BalanceOf<T>),
		/// Successfully updated the local marketplace listing fee for a metaverse
		MetaverseListingFeeUpdated(MetaverseId, Perbill),
		/// Successfully withdrawn funds from a metaverse treasury fund
		MetaverseTreasuryFundsWithdrawn(MetaverseId),
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
		/// Listing fee exceed threshold
		MetaverseListingFeeExceedThreshold,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a metaverse.
		///
		/// The dispatch origin for this call must be _Signed_.
		/// - `metadata': the metadata of the new metaverse
		///
		/// Emits `NewMetaverseCreated` if successful.
		#[pallet::weight(T::WeightInfo::create_metaverse())]
		pub fn create_metaverse(origin: OriginFor<T>, metadata: MetaverseMetadata) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let metaverse_id = Self::do_create_metaverse(&who, metadata)?;
			Self::deposit_event(Event::<T>::NewMetaverseCreated(metaverse_id, who));
			Ok(().into())
		}

		/// Transfer a metaverse.
		///
		/// The dispatch origin for this call must be _Signed_.
		/// Only metaverse owner can perform this call.
		/// - 'to': the account which will receive the transferred metaverse
		/// - `metaverse_id': the metaverse ID which will be transferred
		///
		/// Emits `TransferredMetaverse` if successful.
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

		/// Freeze an existing meataverse.
		///
		/// The dispatch origin for this call must be _Signed_.
		/// Only metaverse council can perform this call.
		/// - `metaverse_id`: the metaverse ID which will be freezed
		///
		/// Emits `MetaverseFreezed` if successful.
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

		/// Unfreeze existing frozen meataverse.
		///
		/// The dispatch origin for this call must be _Signed_.
		/// Only metaverse council can perform this call.
		/// - `metaverse_id`: the metaverse ID which will be unfreezed
		///
		/// Emits `MetaverseUnfreezed` if successful.
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

		/// Destroy a frozen meataverse.
		///
		/// The dispatch origin for this call must be _Signed_.
		/// Only metaverse owner can perform this call
		/// - `metaverse_id`: the metaverse ID which will be destroyed
		///
		/// Emits `MetaverseDestroyed` if successful.
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

		/// Updates the meatverse's local marketplace listing fee
		///
		/// The dispatch origin for this call must be _Signed_.
		/// Only metaverse owner can withdraw funds.
		/// - `metaverse_id`: the meatverse ID which fees will be updated
		/// - 'new_listng_fee': the updated metaverse's local marketplace listing fee
		///
		/// Emits `MetaverseListingFeeUpdated` if successful.
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

		/// Withdraws funds from metaverse treasury fund
		///
		/// The dispatch origin for this call must be _Signed_.
		/// Only metaverse owner can withdraw funds.
		/// - `metaverse_id`: the meatverse ID of the local treasury which funds will be withdrawn
		///
		/// Emits `MetaverseTreasuryFundsWithdrawn` if successful.
		#[pallet::weight(T::WeightInfo::withdraw_from_metaverse_fund())]
		pub fn withdraw_from_metaverse_fund(
			origin: OriginFor<T>,
			metaverse_id: MetaverseId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(Self::check_ownership(&who, &metaverse_id), Error::<T>::NoPermission);
			let metaverse_fund_account = T::MetaverseTreasury::get().into_sub_account(metaverse_id);

			// Balance minus existential deposit
			let metaverse_fund_balance = <T as Config>::Currency::free_balance(&metaverse_fund_account)
				.checked_sub(&<T as Config>::Currency::minimum_balance())
				.ok_or(ArithmeticError::Underflow)?;
			<T as Config>::Currency::transfer(
				&metaverse_fund_account,
				&who,
				metaverse_fund_balance,
				ExistenceRequirement::KeepAlive,
			)?;

			Self::deposit_event(Event::<T>::MetaverseTreasuryFundsWithdrawn(metaverse_id));

			Ok(().into())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}

impl<T: Config> Pallet<T> {
	/// Internal new metaverse generation
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

	/// Internal new metaverse creation
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
		//log::info!("Created Metaverse  with Id {:?}", metaverse_id);
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

	/// Minting of a land class for the metaverse
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

	/// Minting of an estate class for the metaverse
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

	/// Internal update of a metaverse listing fee
	fn do_update_metaverse_listing_fee(
		who: &T::AccountId,
		metaverse_id: &MetaverseId,
		new_listing_fee: Perbill,
	) -> Result<(), DispatchError> {
		ensure!(Self::check_ownership(who, metaverse_id), Error::<T>::NoPermission);
		ensure!(
			new_listing_fee <= Perbill::from_percent(25u32),
			Error::<T>::MetaverseListingFeeExceedThreshold
		);

		Metaverses::<T>::try_mutate(metaverse_id, |metaverse_info| -> DispatchResult {
			let t = metaverse_info.as_mut().ok_or(Error::<T>::MetaverseInfoNotFound)?;
			t.listing_fee = new_listing_fee;
			Ok(())
		})
	}

	/// Internal update of metaverse info to v2
	pub fn upgrade_metaverse_info_v2() -> Weight {
		log::info!("Start upgrade_metaverse_info_v2");
		let mut upgraded_metaverse_items = 0;

		let default_land_class_id = TryInto::<ClassId>::try_into(0u32).unwrap_or_default();
		let default_estate_class_id = TryInto::<ClassId>::try_into(1u32).unwrap_or_default();

		Metaverses::<T>::translate(|k, metaverse_info_v1: MetaverseInfoV1<T::AccountId>| {
			upgraded_metaverse_items += 1;

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
		log::info!("{} metaverses upgraded:", upgraded_metaverse_items);
		0
	}

	/// Internal update of metaverse info to v3
	pub fn upgrade_metaverse_info_v3() -> Weight {
		log::info!("Start upgrade_metaverse_info_v3");
		let mut upgraded_metaverse_items = 0;
		let mut total_metaverse_items = 0;

		Metaverses::<T>::translate(|k, metaverse_info_v1: MetaverseInfoV1<T::AccountId>| {
			total_metaverse_items += 1;
			let new_land_class_id = Self::mint_metaverse_land_class(&metaverse_info_v1.owner, k).unwrap_or_default();
			let new_estate_class_id =
				Self::mint_metaverse_estate_class(&metaverse_info_v1.owner, k).unwrap_or_default();

			upgraded_metaverse_items += 1;

			let v3: MetaverseInfo<T::AccountId> = MetaverseInfo {
				owner: metaverse_info_v1.owner,
				metadata: metaverse_info_v1.metadata,
				currency_id: metaverse_info_v1.currency_id,
				is_frozen: false,
				listing_fee: Perbill::from_percent(0u32),
				land_class_id: new_land_class_id,
				estate_class_id: new_estate_class_id,
			};
			Some(v3)
		});
		log::info!("{} metaverses in total:", total_metaverse_items);
		log::info!("{} metaverses upgraded:", upgraded_metaverse_items);
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
		return T::MetaverseTreasury::get().into_sub_account(metaverse_id);
	}

	fn get_network_treasury() -> T::AccountId {
		return T::MetaverseTreasury::get().into_account();
	}

	fn check_if_metaverse_estate(metaverse_id: MetaverseId, class_id: &ClassId) -> Result<bool, DispatchError> {
		let metaverse_info = Self::get_metaverse(metaverse_id).ok_or(Error::<T>::MetaverseInfoNotFound)?;

		Ok(class_id == &metaverse_info.land_class_id || class_id == &metaverse_info.estate_class_id)
	}

	fn check_if_metaverse_has_any_land(metaverse_id: MetaverseId) -> Result<bool, DispatchError> {
		let metaverse_info = Self::get_metaverse(metaverse_id).ok_or(Error::<T>::MetaverseInfoNotFound)?;

		let land_unit_class_info = T::NFTHandler::get_nft_class_detail(metaverse_info.land_class_id)?;

		let estate_class_info = T::NFTHandler::get_nft_class_detail(metaverse_info.estate_class_id)?;

		Ok(land_unit_class_info.total_minted_tokens > 0 || estate_class_info.total_minted_tokens > 0)
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
