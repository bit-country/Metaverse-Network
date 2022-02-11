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
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use sp_runtime::{
	traits::{AccountIdConversion, One, Zero},
	DispatchError, Perbill,
};
use sp_std::{collections::btree_map::BTreeMap, prelude::*, vec::Vec};

use bc_primitives::*;
use nft::Pallet as NFTModule;
pub use pallet::*;
use primitives::{AssetId, Balance, DomainId, ElementId, FungibleTokenId, MetaverseId, PowerAmount, RoundIndex};
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
	power_price: PowerAmount,
	/// The tuple of other element index -> required amount
	compositions: Vec<(ElementId, u128)>,
}

#[frame_support::pallet]
pub mod pallet {
	use orml_traits::MultiCurrencyExtended;
	use sp_runtime::traits::{CheckedAdd, Saturating};
	use sp_runtime::ArithmeticError;

	use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	pub type ClassIdOf<T> = <T as orml_nft::Config>::ClassId;

	#[pallet::config]
	pub trait Config: frame_system::Config
    // + orml_nft::Config
    + nft::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// The currency type
        type Currency: LockableCurrency<Self::AccountId, Moment=Self::BlockNumber>
        + ReservableCurrency<Self::AccountId>;
        /// Multi-fungible token currency
        type FungibleTokenCurrency: MultiReservableCurrency<
            Self::AccountId,
            CurrencyId=FungibleTokenId,
            Balance=Balance,
        >;
        #[pallet::constant]
        type EconomyTreasury: Get<PalletId>;
        // #[pallet::constant]
        // type MaxMetaverseMetadata: Get<u32>;
        /// Minimum contribution
        #[pallet::constant]
        type MinContribution: Get<BalanceOf<Self>>;

        /// Mininum staking amount
        type MinStakingAmount: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type MiningCurrencyId: Get<FungibleTokenId>;
    }

	#[pallet::storage]
	#[pallet::getter(fn get_element_index)]
	pub type ElementIndex<T: Config> = StorageMap<_, Twox64Concat, ElementId, ElementInfo>;

	#[pallet::storage]
	#[pallet::getter(fn get_elements_by_account)]
	pub type ElementBalance<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, ElementId, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_authorized_generator_collection)]
	pub type AuthorizedGeneratorCollection<T: Config> = StorageMap<_, Twox64Concat, ClassIdOf<T>, (), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_authorized_distributor_collection)]
	pub type AuthorizedDistributorCollection<T: Config> = StorageMap<_, Twox64Concat, ClassIdOf<T>, (), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_buy_power_by_user_request_queue)]
	pub type BuyPowerByUserRequestQueue<T: Config> =
		StorageMap<_, Twox64Concat, AssetId, Vec<(T::AccountId, PowerAmount)>>;

	#[pallet::storage]
	#[pallet::getter(fn get_buy_power_by_distributor_request_queue)]
	pub type BuyPowerByDistributorRequestQueue<T: Config> =
		StorageMap<_, Twox64Concat, AssetId, Vec<(T::AccountId, PowerAmount)>>;

	#[pallet::storage]
	#[pallet::getter(fn get_power_balance)]
	pub type PowerBalance<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, PowerAmount, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_accepted_domain)]
	pub type AcceptedDomain<T: Config> = StorageMap<_, Twox64Concat, DomainId, ()>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		PowerGeneratorCollectionAuthorized(ClassIdOf<T>),
		PowerDistributorCollectionAuthorized(ClassIdOf<T>),
		BuyPowerOrderByUserHasAddedToQueue(T::AccountId, PowerAmount, AssetId),
		BuyPowerOrderByUserExecuted(T::AccountId, PowerAmount, AssetId),
		BuyPowerOrderByDistributorHasAddedToQueue(T::AccountId, PowerAmount, AssetId),
		BuyPowerOrderByDistributorExecuted(T::AccountId, PowerAmount, AssetId),
		ElementMinted(T::AccountId, u32, u64),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Power generator collection already authorized
		PowerGeneratorCollectionAlreadyAuthorized,
		/// Power distributor collection already authorized
		PowerDistributorCollectionAlreadyAuthorized,
		NFTAssetDoesNotExist,
		NFTClassDoesNotExist,
		NoPermissionToBuyMiningPower,
		DistributorNftDoesNotExist,
		GeneratorNftDoesNotExist,
		AccountIdDoesNotExistInBuyOrderQueue,
		DistributorAccountIdDoesNotExistInBuyOrderQueue,
		ElementDoesNotExist,
		InvalidNumberOfElements,
		AccountHasNoPowerBalance,
		InsufficientBalanceToMintElement,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Authorize a NFT collector for power generator
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn authorize_power_generator_collection(
			origin: OriginFor<T>,
			class_id: ClassIdOf<T>,
		) -> DispatchResultWithPostInfo {
			// Only root can authorize
			ensure_root(origin)?;

			// Check that NFT collection is not authorized already
			ensure!(
				!AuthorizedGeneratorCollection::<T>::contains_key(&class_id),
				Error::<T>::PowerGeneratorCollectionAlreadyAuthorized
			);

			// TODO: check if NFT collection exist
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

			// TODO: check if NFT collection exist
			AuthorizedDistributorCollection::<T>::insert(&class_id, ());

			Self::deposit_event(Event::<T>::PowerDistributorCollectionAuthorized(class_id.clone()));

			Ok(().into())
		}

		/// Enable user to buy mining power with specific distributor NFT
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn buy_power_by_user(
			origin: OriginFor<T>,
			power_amount: PowerAmount,
			distributor_nft_id: AssetId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Check nft is part of distributor collection
			// Get asset detail
			let asset = NFTModule::<T>::get_asset(distributor_nft_id).ok_or(Error::<T>::NFTAssetDoesNotExist)?;
			// Check ownership
			let class_id = asset.0;
			let class_info = orml_nft::Pallet::<T>::classes(class_id).ok_or(Error::<T>::NFTClassDoesNotExist)?;

			ensure!(
				AuthorizedDistributorCollection::<T>::contains_key(class_id),
				Error::<T>::NoPermissionToBuyMiningPower
			);

			// TBD: Get NFT attribute. Convert power amount to the correct bit amount
			let bit_amount: Balance = 100u32.into();

			// Add key if does not exist
			if !BuyPowerByUserRequestQueue::<T>::contains_key(distributor_nft_id) {
				let user_amounts: Vec<(T::AccountId, PowerAmount)> = Vec::new();
				BuyPowerByUserRequestQueue::<T>::insert(distributor_nft_id, user_amounts)
			}

			// Mutate BuyPowerByUserRequestQueue, add to queue
			BuyPowerByUserRequestQueue::<T>::try_mutate_exists(distributor_nft_id, |maybe_user_power_amount| {
				// Append account and power amount info to BuyPowerByUserRequestQueue
				let user_power_amount_by_distributor_nft = maybe_user_power_amount
					.as_mut()
					.ok_or(Error::<T>::DistributorNftDoesNotExist)?;
				user_power_amount_by_distributor_nft.push((who.clone(), power_amount));

				// Reserve BIT
				T::FungibleTokenCurrency::reserve(T::MiningCurrencyId::get(), &who, bit_amount);

				Self::deposit_event(Event::<T>::BuyPowerOrderByUserHasAddedToQueue(
					who.clone(),
					power_amount,
					distributor_nft_id,
				));

				Ok(().into())
			})
		}

		/// Execute user's mining power buying order
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn execute_buy_power_order(
			origin: OriginFor<T>,
			distributor_nft_id: AssetId,
			beneficiary: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Get asset detail
			let asset = NFTModule::<T>::get_asset(distributor_nft_id).ok_or(Error::<T>::NFTAssetDoesNotExist)?;
			// Check ownership
			let class_info = orml_nft::Pallet::<T>::classes(asset.0).ok_or(Error::<T>::NFTClassDoesNotExist)?;
			let class_info_data = class_info.data;
			let token_info =
				orml_nft::Pallet::<T>::tokens(asset.0, asset.1).ok_or(Error::<T>::NoPermissionToBuyMiningPower)?;
			ensure!(who == token_info.owner, Error::<T>::NoPermissionToBuyMiningPower);

			// Mutate BuyPowerByUserRequestQueue, add to queue
			BuyPowerByUserRequestQueue::<T>::try_mutate_exists(distributor_nft_id, |maybe_user_power_amount| {
				// Remove account and power amount info to BuyPowerByUserRequestQueue
				let power_amount_by_user = maybe_user_power_amount
					.as_mut()
					.ok_or(Error::<T>::DistributorNftDoesNotExist)?;

				// Remove from queue
				let index = power_amount_by_user
					.iter()
					.position(|x| x.0.clone() == beneficiary.clone())
					.ok_or(Error::<T>::AccountIdDoesNotExistInBuyOrderQueue)?;

				let (account_id, power_amount) = power_amount_by_user.remove(index);

				// TODO: convert power_amount to bit_amount, or read from struct.
				let bit_amount: Balance = 10u32.into();

				// Unreserve BIT
				T::FungibleTokenCurrency::unreserve(T::MiningCurrencyId::get(), &beneficiary, bit_amount);

				// Burn BIT

				// Transfer power amount
				Self::distribute_power_by_operator(power_amount, beneficiary.clone(), distributor_nft_id);

				Self::deposit_event(Event::<T>::BuyPowerOrderByUserExecuted(
					beneficiary.clone(),
					power_amount,
					distributor_nft_id,
				));

				Ok(().into())
			})
		}

		/// Enable distributor to buy mining power with specific generator NFT
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn buy_power_by_distributor(
			origin: OriginFor<T>,
			generator_nft_id: AssetId,
			distributor_nft_id: AssetId,
			power_amount: PowerAmount,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Check nft is part of distributor collection
			// Get asset detail
			let asset = NFTModule::<T>::get_asset(generator_nft_id).ok_or(Error::<T>::NFTAssetDoesNotExist)?;
			// Check ownership
			let class_id = asset.0;
			let class_info = orml_nft::Pallet::<T>::classes(class_id).ok_or(Error::<T>::NFTClassDoesNotExist)?;

			ensure!(
				AuthorizedGeneratorCollection::<T>::contains_key(class_id),
				Error::<T>::NoPermissionToBuyMiningPower
			);

			// TBD: Get NFT attribute of generator NFT. Convert power amount to the correct bit amount
			let bit_amount: Balance = 100u32.into();

			// Add key if does not exist
			if !BuyPowerByDistributorRequestQueue::<T>::contains_key(generator_nft_id) {
				let distributor_amounts: Vec<(T::AccountId, PowerAmount)> = Vec::new();
				BuyPowerByDistributorRequestQueue::<T>::insert(generator_nft_id, distributor_amounts)
			}

			// Mutate BuyPowerByUserRequestQueue, add to queue
			BuyPowerByDistributorRequestQueue::<T>::try_mutate_exists(
				generator_nft_id,
				|maybe_distributor_power_amount| {
					// Append account and power amount info to BuyPowerByUserRequestQueue
					let distributor_power_amount_by_generator_nft = maybe_distributor_power_amount
						.as_mut()
						.ok_or(Error::<T>::DistributorNftDoesNotExist)?;
					distributor_power_amount_by_generator_nft.push((who.clone(), power_amount));

					// Convert distributor NFT to accountId
					let distributor_nft_account_id: T::AccountId =
						T::EconomyTreasury::get().into_sub_account(distributor_nft_id);

					// Reserve BIT
					T::FungibleTokenCurrency::reserve(
						T::MiningCurrencyId::get(),
						&distributor_nft_account_id,
						bit_amount,
					);

					let power_amount: PowerAmount = 100;
					Self::deposit_event(Event::<T>::BuyPowerOrderByDistributorHasAddedToQueue(
						distributor_nft_account_id.clone(),
						power_amount,
						generator_nft_id,
					));

					Ok(().into())
				},
			)
		}

		/// Execute distributor's mining power buying order
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn execute_generate_power_order(
			origin: OriginFor<T>,
			generator_nft_id: AssetId,
			beneficiary: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Convert generator NFT to accountId
			let generator_nft_account_id: T::AccountId = T::EconomyTreasury::get().into_sub_account(generator_nft_id);

			// Get asset detail
			let asset = NFTModule::<T>::get_asset(generator_nft_id).ok_or(Error::<T>::NFTAssetDoesNotExist)?;
			// Check ownership
			let class_info = orml_nft::Pallet::<T>::classes(asset.0).ok_or(Error::<T>::NFTClassDoesNotExist)?;
			let class_info_data = class_info.data;
			let token_info =
				orml_nft::Pallet::<T>::tokens(asset.0, asset.1).ok_or(Error::<T>::NoPermissionToBuyMiningPower)?;
			ensure!(who == token_info.owner, Error::<T>::NoPermissionToBuyMiningPower);

			// Mutate BuyPowerByUserRequestQueue, add to queue
			BuyPowerByDistributorRequestQueue::<T>::try_mutate_exists(
				generator_nft_id,
				|maybe_distributor_power_amount| {
					// Remove account and power amount info to BuyPowerByDistributorRequestQueue
					let user_power_amount_by_distributor_nft = maybe_distributor_power_amount
						.as_mut()
						.ok_or(Error::<T>::GeneratorNftDoesNotExist)?;

					// Remove from queue
					let index = user_power_amount_by_distributor_nft
						.iter()
						.position(|x| x.0.clone() == beneficiary.clone())
						.ok_or(Error::<T>::DistributorAccountIdDoesNotExistInBuyOrderQueue)?;

					let (account_id, power_amount) = user_power_amount_by_distributor_nft.remove(index);

					// TODO: convert power_amount to bit_amount, or read from struct.
					let bit_amount: Balance = 10u32.into();

					// Unreserve BIT
					T::FungibleTokenCurrency::unreserve(T::MiningCurrencyId::get(), &beneficiary, bit_amount);

					// Burn BIT

					// Transfer power amount
					Self::generate_power_by_operator(power_amount, beneficiary.clone(), generator_nft_id);

					Self::deposit_event(Event::<T>::BuyPowerOrderByDistributorExecuted(
						beneficiary.clone(),
						power_amount,
						generator_nft_id,
					));

					Ok(().into())
				},
			)
		}

		/// Mint Element
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn mint_element(
			origin: OriginFor<T>,
			element_index: ElementId,
			number_of_element: u64,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(!number_of_element.is_zero(), Error::<T>::InvalidNumberOfElements);

			let element_info = ElementIndex::<T>::get(element_index).ok_or(Error::<T>::ElementDoesNotExist)?;

			let power_cost: PowerAmount = number_of_element
				.checked_mul(element_info.power_price)
				.ok_or(ArithmeticError::Overflow)?;

			let mut power_balance = PowerBalance::<T>::get(&who);
			ensure!(power_balance > power_cost, Error::<T>::InsufficientBalanceToMintElement);

			// Update PowerBalance
			power_balance = power_balance
				.checked_sub(power_cost)
				.ok_or(ArithmeticError::Underflow)?;
			PowerBalance::<T>::insert(&who, power_balance);

			// Update ElementBalance
			let mut element_balance = ElementBalance::<T>::get(who.clone(), element_index);
			element_balance = element_balance
				.checked_add(number_of_element)
				.ok_or(ArithmeticError::Overflow)?;
			ElementBalance::<T>::insert(who.clone(), element_index, element_balance);

			Self::deposit_event(Event::<T>::ElementMinted(who.clone(), element_index, number_of_element));

			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn distribute_power_by_operator(
		power_amount: PowerAmount,
		beneficiary: T::AccountId,
		distributor_nft_id: AssetId,
	) -> DispatchResultWithPostInfo {
		Ok(().into())
	}

	fn generate_power_by_operator(
		power_amount: PowerAmount,
		beneficiary: T::AccountId,
		generator_nft_id: AssetId,
	) -> DispatchResultWithPostInfo {
		Ok(().into())
	}
}
