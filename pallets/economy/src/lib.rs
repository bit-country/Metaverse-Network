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
use primitives::{
	AssetId, Balance, ClassId, DomainId, ElementId, FungibleTokenId, MetaverseId, NftId, PowerAmount, RoundIndex,
};
//pub use weights::WeightInfo;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

//pub mod weights;

/// A record for basic element info. i.e. price, compositions and rules
#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ElementInfo {
	/// Power price for the element
	power_price: PowerAmount,
	/// The tuple of other element index -> required amount
	compositions: Vec<(ElementId, u128)>,
}

/// A record for basic element info. i.e. price, compositions and rules
#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct OrderInfo<BlockNumber> {
	/// Power price for the element
	power_amount: PowerAmount,
	/// The tuple of other element index -> required amount
	bit_amount: Balance,
	/// Target block number that order can be fullfilled
	target: BlockNumber,
	/// Commission amount
	commission_fee: Balance,
}

#[frame_support::pallet]
pub mod pallet {
	use orml_traits::MultiCurrencyExtended;
	use sp_runtime::traits::{CheckedAdd, CheckedSub, Saturating};
	use sp_runtime::ArithmeticError;

	use primitives::staking::RoundInfo;
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
		#[pallet::constant]
		type EconomyTreasury: Get<PalletId>;
		#[pallet::constant]
		type MiningCurrencyId: Get<FungibleTokenId>;
		#[pallet::constant]
		type MinimumStake: Get<BalanceOf<Self>>;
		#[pallet::constant]
		type PowerAmountPerBlock: Get<PowerAmount>;
	}

	#[pallet::storage]
	#[pallet::getter(fn get_bit_power_exchange_rate)]
	pub(super) type BitPowerExchangeRate<T: Config> = StorageValue<_, Balance, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_element_index)]
	pub type ElementIndex<T: Config> = StorageMap<_, Twox64Concat, ElementId, ElementInfo>;

	#[pallet::storage]
	#[pallet::getter(fn get_elements_by_account)]
	pub type ElementBalance<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, ElementId, u64, ValueQuery>;

	/// Authorize power generator collection with associated commission
	#[pallet::storage]
	#[pallet::getter(fn get_authorized_generator_collection)]
	pub type AuthorizedGeneratorCollection<T: Config> =
		StorageMap<_, Twox64Concat, (GroupCollectionId, ClassId), (), OptionQuery>;

	/// Authorize power generator collection with associated commission
	#[pallet::storage]
	#[pallet::getter(fn get_authorized_distributor_collection)]
	pub type AuthorizedDistributorCollection<T: Config> =
		StorageMap<_, Twox64Concat, (GroupCollectionId, ClassId), (), OptionQuery>;

	/// Specific NFT commission for power conversion
	#[pallet::storage]
	#[pallet::getter(fn get_power_conversion_commission)]
	pub type EconomyCommission<T: Config> = StorageMap<_, Twox64Concat, (ClassId, TokenId), Perbill, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_buy_power_by_user_request_queue)]
	pub type BuyPowerByUserRequestQueue<T: Config> =
		StorageDoubleMap<_, Twox64Concat, (ClassId, TokenId), Twox64Concat, T::AccountId, OrderInfo<T::BlockNumber>>;

	#[pallet::storage]
	#[pallet::getter(fn get_buy_power_by_distributor_request_queue)]
	pub type BuyPowerByDistributorRequestQueue<T: Config> =
		StorageDoubleMap<_, Twox64Concat, (ClassId, TokenId), Twox64Concat, T::AccountId, OrderInfo<T::BlockNumber>>;

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

	/// Self-staking exit queue info
	/// This will keep track of stake exits queue, unstake only allows after 1 round
	#[pallet::storage]
	#[pallet::getter(fn staking_exit_queue)]
	pub type ExitQueue<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Twox64Concat, RoundIndex, BalanceOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn total_stake)]
	/// Total native token locked in this pallet
	type TotalStake<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		PowerGeneratorCollectionAuthorized(GroupCollectionId, ClassId),
		PowerDistributorCollectionAuthorized(GroupCollectionId, ClassId),
		BuyPowerOrderByUserHasAddedToQueue(T::AccountId, PowerAmount, (ClassId, TokenId)),
		BuyPowerOrderByUserExecuted(T::AccountId, PowerAmount, (ClassId, TokenId)),
		BuyPowerOrderByDistributorHasAddedToQueue(T::AccountId, PowerAmount, (ClassId, TokenId)),
		BuyPowerOrderByDistributorExecuted(T::AccountId, PowerAmount, (ClassId, TokenId)),
		BuyPowerOrderByGeneratorToNetworkExecuted(T::AccountId, PowerAmount, (ClassId, TokenId)),
		ElementMinted(T::AccountId, u32, u64),
		MiningResourceBurned(Balance),
		SelfStakedToEconomy101(T::AccountId, BalanceOf<T>),
		SelfStakingRemovedFromEconomy101(T::AccountId, BalanceOf<T>),
		BitPowerExchangeRateUpdated(Balance),
		UnstakedAmountWithdrew(T::AccountId, BalanceOf<T>),
		SetPowerBalance(T::AccountId, PowerAmount),
		CommissionUpdated((ClassId, TokenId), Perbill),
		CancelPowerConversionRequest((ClassId, TokenId), T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		// Power generator collection already authorized
		PowerGeneratorCollectionAlreadyAuthorized,
		// Power distributor collection already authorized
		PowerDistributorCollectionAlreadyAuthorized,
		// NFT asset does not exist
		NFTAssetDoesNotExist,
		// NFT class does not exist
		NFTClassDoesNotExist,
		// NFT collection does not exist
		NFTCollectionDoesNotExist,
		// No permission
		NoPermission,
		// No permission to buy power
		NoPermissionToBuyPower,
		// No permission to execute buy power order
		NoPermissionToExecuteBuyPowerOrder,
		// No authorization
		NoPermissionToExecuteGeneratingPowerOrder,
		// No authorization
		NoAuthorization,
		// Element does not exist
		ElementDoesNotExist,
		// Number of element is invalid
		InvalidNumberOfElements,
		// Insufficient power balance
		AccountHasNoPowerBalance,
		// Insufficient balance to mint element
		InsufficientBalanceToMintElement,
		// Insufficient balance to distribute power
		InsufficientBalanceToDistributePower,
		// Insufficient balance to generate power
		InsufficientBalanceToGeneratePower,
		// Insufficient balance to buy power
		InsufficientBalanceToBuyPower,
		// Power amount is zero
		PowerAmountIsZero,
		// Power distributor queue does not exist
		PowerDistributionQueueDoesNotExist,
		// Power generator queue does not exist
		PowerGenerationQueueDoesNotExist,
		// Power generator is not authorized
		PowerGenerationIsNotAuthorized,
		// Power distributor is not authorized
		PowerDistributorIsNotAuthorized,
		// Not enough free balance for staking
		InsufficientBalanceForStaking,
		// Unstake amount greater than staked amount
		UnstakeAmountExceedStakedAmount,
		// Has scheduled exit staking, only stake after queue exit
		ExitQueueAlreadyScheduled,
		// Stake amount below minimum staking required
		StakeBelowMinimum,
		// Withdraw future round
		WithdrawFutureRound,
		// Exit queue does not exist
		ExitQueueDoesNotExit,
		// Unstaked amount is zero
		UnstakeAmountIsZero,
		// Request already exists
		RequestAlreadyExist,
		// Order has not reach target
		NotReadyToExecute,
		// Order needs to reach target before cancelling
		OrderIsNotReadyForCancel,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set bit power exchange rate
		/// BIT price per Power, accept decimal
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		#[transactional]
		pub fn set_bit_power_exchange_rate(origin: OriginFor<T>, rate: Balance) -> DispatchResultWithPostInfo {
			// Only root can authorize
			ensure_root(origin)?;

			BitPowerExchangeRate::<T>::set(rate);

			Self::deposit_event(Event::<T>::BitPowerExchangeRateUpdated(rate));

			Ok(().into())
		}

		/// Set power balance for NFTs
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		#[transactional]
		pub fn set_power_balance(
			origin: OriginFor<T>,
			beneficiary: (ClassId, TokenId),
			amount: PowerAmount,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			let account_id = T::EconomyTreasury::get().into_sub_account(beneficiary);
			PowerBalance::<T>::insert(&account_id, amount);

			Self::deposit_event(Event::<T>::SetPowerBalance(account_id, amount));

			Ok(().into())
		}

		/// Authorize a NFT collector for power generator
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		#[transactional]
		pub fn authorize_power_generator_collection(
			origin: OriginFor<T>,
			collection_id: GroupCollectionId,
			class_id: ClassId,
		) -> DispatchResultWithPostInfo {
			// Only root can authorize
			ensure_root(origin)?;

			// Check that NFT collection is not authorized already
			ensure!(
				!AuthorizedGeneratorCollection::<T>::contains_key((collection_id, &class_id)),
				Error::<T>::PowerGeneratorCollectionAlreadyAuthorized
			);

			// Check if NFT class exist and match the specified collection
			ensure!(
				T::NFTHandler::check_collection_and_class(collection_id, class_id)?,
				Error::<T>::NFTCollectionDoesNotExist
			);

			AuthorizedGeneratorCollection::<T>::insert((collection_id, &class_id), ());

			Self::deposit_event(Event::<T>::PowerGeneratorCollectionAuthorized(
				collection_id,
				class_id.clone(),
			));

			Ok(().into())
		}

		/// Authorize a NFT collector for power distributor
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		#[transactional]
		pub fn authorize_power_distributor_collection(
			origin: OriginFor<T>,
			collection_id: GroupCollectionId,
			class_id: ClassId,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			// Check that NFT collection is not authorized already
			ensure!(
				!AuthorizedDistributorCollection::<T>::contains_key((collection_id, &class_id)),
				Error::<T>::PowerDistributorCollectionAlreadyAuthorized
			);

			// Check if NFT class exist and match the specified collection
			ensure!(
				T::NFTHandler::check_collection_and_class(collection_id, class_id)?,
				Error::<T>::NFTCollectionDoesNotExist
			);

			AuthorizedDistributorCollection::<T>::insert((collection_id, &class_id), ());

			Self::deposit_event(Event::<T>::PowerDistributorCollectionAuthorized(
				collection_id,
				class_id.clone(),
			));

			Ok(().into())
		}

		/// Enable user to buy mining power with specific distributor NFT
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		#[transactional]
		pub fn buy_power_by_user(
			origin: OriginFor<T>,
			power_amount: PowerAmount,
			distributor_nft_id: (ClassId, TokenId),
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(power_amount > 0, Error::<T>::PowerAmountIsZero);

			ensure!(
				!BuyPowerByUserRequestQueue::<T>::contains_key(&distributor_nft_id, &who),
				Error::<T>::RequestAlreadyExist
			);

			// Get NFT details
			let group_distributor_nft = T::NFTHandler::get_nft_group_collection(&distributor_nft_id.0)?;

			// Ensure distributor NFT is authorized
			ensure!(
				AuthorizedDistributorCollection::<T>::contains_key((group_distributor_nft, distributor_nft_id.0)),
				Error::<T>::NoPermissionToBuyPower
			);

			let commission = match EconomyCommission::<T>::get(distributor_nft_id) {
				Some(cm) => cm,
				None => Perbill::from_percent(0),
			};

			// Convert to bit by using global exchange rate
			let (bit_amount, commission_fee) = Self::convert_power_to_bit(power_amount.into(), commission);
			ensure!(
				T::FungibleTokenCurrency::can_reserve(T::MiningCurrencyId::get(), &who, bit_amount),
				Error::<T>::InsufficientBalanceToBuyPower
			);

			// Reserve BIT
			T::FungibleTokenCurrency::reserve(T::MiningCurrencyId::get(), &who, bit_amount);

			let target_block = Self::get_target_execution_order(power_amount)?;

			// Add key if does not exist
			BuyPowerByUserRequestQueue::<T>::insert(
				distributor_nft_id,
				who.clone(),
				OrderInfo {
					power_amount,
					bit_amount,
					target: target_block,
					commission_fee,
				},
			);

			Self::deposit_event(Event::<T>::BuyPowerOrderByUserHasAddedToQueue(
				who.clone(),
				power_amount,
				distributor_nft_id,
			));

			Ok(().into())
		}

		/// Execute user's mining power buying order
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		#[transactional]
		pub fn execute_buy_power_order(
			origin: OriginFor<T>,
			distributor_nft_id: (ClassId, TokenId),
			beneficiary: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Get asset detail
			// Check nft is part of distributor collection
			let group_distributor_nft_detail = T::NFTHandler::get_nft_group_collection(&distributor_nft_id.0)?;
			// Ensure distributor NFT is authorized
			ensure!(
				AuthorizedDistributorCollection::<T>::contains_key((
					group_distributor_nft_detail,
					distributor_nft_id.0
				)),
				Error::<T>::NoAuthorization
			);

			// Check if executor is the owner of the Distributor NFT
			ensure!(
				T::NFTHandler::check_nft_ownership(&who, &distributor_nft_id)?,
				Error::<T>::NoPermissionToExecuteBuyPowerOrder
			);

			// Process queue and delete if queue has been proceeded
			let buy_power_by_user_request =
				Self::get_buy_power_by_user_request_queue(&distributor_nft_id, &beneficiary)
					.ok_or(Error::<T>::PowerDistributionQueueDoesNotExist)?;

			ensure!(
				Self::check_target_execution(buy_power_by_user_request.target),
				Error::<T>::NotReadyToExecute
			);

			let power_amount = buy_power_by_user_request.power_amount;
			let bit_amount = buy_power_by_user_request.bit_amount;

			// Unreserve BIT
			T::FungibleTokenCurrency::unreserve(T::MiningCurrencyId::get(), &beneficiary, bit_amount);

			// Burn BIT
			Self::do_burn(&beneficiary, bit_amount)?;

			// Get distributor NFT account id
			let nft_account_id: T::AccountId = T::EconomyTreasury::get().into_sub_account(distributor_nft_id);
			// Deposit commission to NFT operator
			T::FungibleTokenCurrency::deposit(
				T::MiningCurrencyId::get(),
				&nft_account_id,
				buy_power_by_user_request.commission_fee,
			);

			// Transfer power amount
			Self::distribute_power_by_operator(power_amount, &beneficiary, distributor_nft_id)?;

			BuyPowerByUserRequestQueue::<T>::remove(distributor_nft_id, beneficiary);

			Ok(().into())
		}

		/// Enable distributor to buy mining power with specific generator NFT
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		#[transactional]
		pub fn buy_power_by_distributor(
			origin: OriginFor<T>,
			generator_nft_id: (ClassId, TokenId),
			distributor_nft_id: (ClassId, TokenId),
			power_amount: PowerAmount,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Get distributor NFT account id
			let distributor_nft_account_id: T::AccountId =
				T::EconomyTreasury::get().into_sub_account(distributor_nft_id);

			ensure!(
				!BuyPowerByDistributorRequestQueue::<T>::contains_key(&generator_nft_id, &distributor_nft_account_id),
				Error::<T>::RequestAlreadyExist
			);

			// Ensure buy power by distributor only called by distributor nft owner
			// Check nft is part of distributor collection
			let group_distributor_nft_detail = T::NFTHandler::get_nft_group_collection(&distributor_nft_id.0)?;
			// Ensure distributor NFT is authorized
			ensure!(
				AuthorizedDistributorCollection::<T>::contains_key((
					group_distributor_nft_detail,
					distributor_nft_id.0
				)),
				Error::<T>::NoPermissionToBuyPower
			);

			// Check if origin is the owner of the Distributor NFT
			ensure!(
				T::NFTHandler::check_nft_ownership(&who, &distributor_nft_id)?,
				Error::<T>::NoPermissionToBuyPower
			);

			// Check nft is part of generator collection
			let group_generator_nft_detail = T::NFTHandler::get_nft_group_collection(&generator_nft_id.0)?;
			// Ensure generator NFT is authorized
			ensure!(
				AuthorizedGeneratorCollection::<T>::contains_key((group_generator_nft_detail, generator_nft_id.0)),
				Error::<T>::PowerGenerationIsNotAuthorized
			);

			let commission = match EconomyCommission::<T>::get(generator_nft_id) {
				Some(cm) => cm,
				None => Perbill::from_percent(0),
			};

			// Convert to bit by using global exchange rate
			let (bit_amount, commission_fee) = Self::convert_power_to_bit(power_amount.into(), commission);

			ensure!(
				T::FungibleTokenCurrency::can_reserve(
					T::MiningCurrencyId::get(),
					&distributor_nft_account_id,
					bit_amount
				),
				Error::<T>::InsufficientBalanceToBuyPower
			);

			// Reserve BIT
			T::FungibleTokenCurrency::reserve(T::MiningCurrencyId::get(), &distributor_nft_account_id, bit_amount);

			let target_block = Self::get_target_execution_order(power_amount)?;

			// Add key if does not exist
			BuyPowerByDistributorRequestQueue::<T>::insert(
				generator_nft_id,
				distributor_nft_account_id.clone(),
				OrderInfo {
					power_amount,
					bit_amount,
					target: target_block,
					commission_fee,
				},
			);

			Self::deposit_event(Event::<T>::BuyPowerOrderByDistributorHasAddedToQueue(
				distributor_nft_account_id.clone(),
				power_amount,
				generator_nft_id,
			));

			Ok(().into())
		}

		/// Execute distributor's mining power buying order
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		#[transactional]
		pub fn execute_generate_power_order(
			origin: OriginFor<T>,
			generator_nft_id: (ClassId, TokenId),
			beneficiary: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Ensure executor is holding generator NFT
			// Check nft is part of generator collection
			let group_generator_nft_detail = T::NFTHandler::get_nft_group_collection(&generator_nft_id.0)?;
			// Ensure generator NFT is authorized
			ensure!(
				AuthorizedGeneratorCollection::<T>::contains_key((group_generator_nft_detail, generator_nft_id.0)),
				Error::<T>::PowerGenerationIsNotAuthorized
			);

			// Check if origin is the owner of the Distributor NFT
			ensure!(
				T::NFTHandler::check_nft_ownership(&who, &generator_nft_id)?,
				Error::<T>::NoPermissionToExecuteGeneratingPowerOrder
			);

			let buy_power_by_distributor_request =
				Self::get_buy_power_by_distributor_request_queue(&generator_nft_id, &beneficiary)
					.ok_or(Error::<T>::PowerGenerationQueueDoesNotExist)?;

			let power_amount = buy_power_by_distributor_request.power_amount;
			let bit_amount = buy_power_by_distributor_request.bit_amount;

			// Unreserve BIT
			T::FungibleTokenCurrency::unreserve(T::MiningCurrencyId::get(), &beneficiary, bit_amount);

			// Burn BIT
			Self::do_burn(&beneficiary, bit_amount)?;

			// Get distributor NFT account id
			let nft_account_id: T::AccountId = T::EconomyTreasury::get().into_sub_account(generator_nft_id);
			// Deposit commission to NFT operator
			T::FungibleTokenCurrency::deposit(
				T::MiningCurrencyId::get(),
				&nft_account_id,
				buy_power_by_distributor_request.commission_fee,
			);

			// Transfer power amount
			Self::generate_power_by_operator(power_amount, &beneficiary, generator_nft_id)?;

			BuyPowerByDistributorRequestQueue::<T>::remove(&generator_nft_id, &beneficiary);

			Ok(().into())
		}

		/// Mint Element
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		#[transactional]
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

		/// Stake native token to staking ledger for mining power calculation
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		#[transactional]
		pub fn stake(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
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

			// Update staking info
			let mut staked_balance = StakingInfo::<T>::get(&who);
			let total = staked_balance.checked_add(&amount).ok_or(ArithmeticError::Overflow)?;

			ensure!(total >= T::MinimumStake::get(), Error::<T>::StakeBelowMinimum);

			T::Currency::reserve(&who, amount)?;

			StakingInfo::<T>::insert(&who, total);

			let new_total_staked = TotalStake::<T>::get().saturating_add(amount);
			<TotalStake<T>>::put(new_total_staked);

			let current_round = T::RoundHandler::get_current_round_info();

			Self::deposit_event(Event::SelfStakedToEconomy101(who, amount));

			Ok(().into())
		}

		/// Stake native token to staking ledger for mining power calculation
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn unstake(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Ensure amount is greater than zero
			ensure!(!amount.is_zero(), Error::<T>::UnstakeAmountIsZero);

			// Update staking info
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

			Ok(().into())
		}

		/// Stake native token to staking ledger for mining power calculation
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn withdraw_unreserved(origin: OriginFor<T>, round_index: RoundIndex) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let current_round = T::RoundHandler::get_current_round_info();

			ensure!(round_index <= current_round.current, Error::<T>::WithdrawFutureRound);

			// Get user exit queue
			let exit_balance = ExitQueue::<T>::get(&who, round_index).ok_or(Error::<T>::ExitQueueDoesNotExit)?;

			ExitQueue::<T>::remove(&who, round_index);
			T::Currency::unreserve(&who, exit_balance);

			Self::deposit_event(Event::<T>::UnstakedAmountWithdrew(who, exit_balance));

			Ok(().into())
		}

		/// Get more power from the network from specific generator NFT
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		#[transactional]
		pub fn get_more_power_by_generator(
			origin: OriginFor<T>,
			generator_nft_id: (ClassId, TokenId),
			power_amount: PowerAmount,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Get distributor NFT account id
			let generator_nft_account_id: T::AccountId = T::EconomyTreasury::get().into_sub_account(generator_nft_id);

			// Ensure get power by generator only called by generator nft owner
			// Check nft is part of generator collection
			let group_generator_nft_detail = T::NFTHandler::get_nft_group_collection(&generator_nft_id.0)?;
			// Ensure generator NFT is authorized
			ensure!(
				AuthorizedGeneratorCollection::<T>::contains_key((group_generator_nft_detail, generator_nft_id.0)),
				Error::<T>::PowerGenerationIsNotAuthorized
			);

			// Check if origin is the owner of the Distributor NFT
			ensure!(
				T::NFTHandler::check_nft_ownership(&who, &generator_nft_id)?,
				Error::<T>::NoPermissionToBuyPower
			);

			// Convert to bit by using global exchange rate - no commission applied
			let (bit_amount, _commission_fee) =
				Self::convert_power_to_bit(power_amount.into(), Perbill::from_percent(0));

			ensure!(
				T::FungibleTokenCurrency::can_reserve(
					T::MiningCurrencyId::get(),
					&generator_nft_account_id,
					bit_amount
				),
				Error::<T>::InsufficientBalanceToBuyPower
			);

			// Burn BIT
			T::FungibleTokenCurrency::withdraw(T::MiningCurrencyId::get(), &generator_nft_account_id, bit_amount);

			// Update Power Balance
			Self::distribute_power_by_network(power_amount.into(), &generator_nft_account_id);

			Self::deposit_event(Event::<T>::BuyPowerOrderByGeneratorToNetworkExecuted(
				generator_nft_account_id.clone(),
				power_amount,
				generator_nft_id,
			));

			Ok(().into())
		}

		/// update commission of power distributor / generator
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		#[transactional]
		pub fn update_commission(
			origin: OriginFor<T>,
			nft_id: (ClassId, TokenId),
			commission: Perbill,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Check if origin is the owner of the NFT
			ensure!(
				T::NFTHandler::check_nft_ownership(&who, &nft_id)?,
				Error::<T>::NoPermission
			);

			// Check nft is part of generator collection
			let group_generator_nft_detail = T::NFTHandler::get_nft_group_collection(&nft_id.0)?;
			// Ensure NFT is authorized
			ensure!(
				AuthorizedGeneratorCollection::<T>::contains_key((group_generator_nft_detail, nft_id.0))
					|| AuthorizedDistributorCollection::<T>::contains_key((group_generator_nft_detail, nft_id.0)),
				Error::<T>::NoAuthorization
			);

			EconomyCommission::<T>::insert((nft_id.0, &nft_id.1), commission.clone());

			Self::deposit_event(Event::<T>::CommissionUpdated(nft_id.clone(), commission));

			Ok(().into())
		}

		/// update commission of power distributor / generator
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		#[transactional]
		pub fn withdraw_mining_resource(
			origin: OriginFor<T>,
			nft_id: (ClassId, TokenId),
			amount: Balance,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Check if origin is the owner of the NFT
			ensure!(
				T::NFTHandler::check_nft_ownership(&who, &nft_id)?,
				Error::<T>::NoPermission
			);

			// Get NFT account id
			let nft_account_id: T::AccountId = T::EconomyTreasury::get().into_sub_account(nft_id);

			T::FungibleTokenCurrency::transfer(T::MiningCurrencyId::get(), &nft_account_id, &who, amount);
			Ok(().into())
		}

		/// Cancel queue order of power distributor
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		#[transactional]
		pub fn cancel_user_queue_order(origin: OriginFor<T>, nft_id: (ClassId, TokenId)) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(
				BuyPowerByUserRequestQueue::<T>::contains_key(nft_id, &who),
				Error::<T>::PowerGenerationQueueDoesNotExist
			);

			let order_info = BuyPowerByUserRequestQueue::<T>::get(nft_id, &who)
				.ok_or(Error::<T>::PowerDistributionQueueDoesNotExist)?;

			let current_block_number = <frame_system::Pallet<T>>::current_block_number();

			ensure!(
				order_info.target < current_block_number,
				Error::<T>::OrderIsNotReadyForCancel
			);

			T::FungibleTokenCurrency::unreserve(T::MiningCurrencyId::get(), &who, order_info.bit_amount);

			BuyPowerByUserRequestQueue::<T>::remove(nft_id, &who);

			Self::deposit_event(Event::CancelPowerConversionRequest(nft_id, who));

			Ok(().into())
		}

		/// Cancel queue order of user
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		#[transactional]
		pub fn cancel_distributor_queue_order(
			origin: OriginFor<T>,
			nft_id: (ClassId, TokenId),
			receiver_nft_id: (ClassId, TokenId),
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Check if origin is the owner of the NFT
			ensure!(
				T::NFTHandler::check_nft_ownership(&who, &receiver_nft_id)?,
				Error::<T>::NoPermission
			);

			// Nft account id
			// Get receiver NFT account id
			let receiver_nft_account_id: T::AccountId = T::EconomyTreasury::get().into_sub_account(receiver_nft_id);

			// Check if queue exists
			ensure!(
				BuyPowerByDistributorRequestQueue::<T>::contains_key(nft_id, &receiver_nft_account_id),
				Error::<T>::PowerGenerationQueueDoesNotExist
			);

			let current_block_number = <frame_system::Pallet<T>>::current_block_number();

			let order_info = BuyPowerByDistributorRequestQueue::<T>::get(nft_id, &receiver_nft_account_id)
				.ok_or(Error::<T>::PowerGenerationQueueDoesNotExist)?;

			ensure!(
				order_info.target < current_block_number,
				Error::<T>::OrderIsNotReadyForCancel
			);

			T::FungibleTokenCurrency::unreserve(
				T::MiningCurrencyId::get(),
				&receiver_nft_account_id,
				order_info.bit_amount,
			);

			BuyPowerByDistributorRequestQueue::<T>::remove(nft_id, &receiver_nft_account_id);

			Self::deposit_event(Event::CancelPowerConversionRequest(nft_id, receiver_nft_account_id));

			Ok(().into())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_runtime_upgrade() -> Weight {
			Self::upgrade_order_info_data_v2();
			0
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn economy_pallet_account_id() -> T::AccountId {
		T::EconomyTreasury::get().into_account()
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

	fn distribute_power_by_operator(
		power_amount: PowerAmount,
		beneficiary: &T::AccountId,
		distributor_nft_id: (ClassId, TokenId),
	) -> DispatchResult {
		// Convert distributor NFT to accountId
		let distributor_nft_account_id: T::AccountId = T::EconomyTreasury::get().into_sub_account(distributor_nft_id);

		let mut distributor_power_balance = PowerBalance::<T>::get(distributor_nft_account_id.clone());
		ensure!(
			distributor_power_balance > power_amount,
			Error::<T>::InsufficientBalanceToDistributePower
		);
		distributor_power_balance = distributor_power_balance
			.checked_sub(power_amount)
			.ok_or(ArithmeticError::Underflow)?;

		let mut user_power_balance = PowerBalance::<T>::get(beneficiary.clone());
		user_power_balance = user_power_balance
			.checked_add(power_amount)
			.ok_or(ArithmeticError::Overflow)?;

		PowerBalance::<T>::insert(distributor_nft_account_id.clone(), distributor_power_balance);
		PowerBalance::<T>::insert(beneficiary.clone(), user_power_balance);

		Self::deposit_event(Event::<T>::BuyPowerOrderByUserExecuted(
			beneficiary.clone(),
			power_amount,
			distributor_nft_id,
		));

		Ok(())
	}

	fn generate_power_by_operator(
		power_amount: PowerAmount,
		beneficiary: &T::AccountId,
		generator_nft_id: (ClassId, TokenId),
	) -> DispatchResult {
		// Convert generator NFT to accountId
		let generator_nft_account_id: T::AccountId = T::EconomyTreasury::get().into_sub_account(generator_nft_id);

		let mut generator_power_balance = PowerBalance::<T>::get(generator_nft_account_id.clone());
		ensure!(
			generator_power_balance > power_amount,
			Error::<T>::InsufficientBalanceToGeneratePower
		);
		generator_power_balance = generator_power_balance
			.checked_sub(power_amount)
			.ok_or(ArithmeticError::Underflow)?;

		let mut distributor_power_balance = PowerBalance::<T>::get(beneficiary.clone());
		distributor_power_balance = distributor_power_balance
			.checked_add(power_amount)
			.ok_or(ArithmeticError::Overflow)?;

		PowerBalance::<T>::insert(generator_nft_account_id.clone(), generator_power_balance);
		PowerBalance::<T>::insert(beneficiary.clone(), distributor_power_balance);

		Self::deposit_event(Event::<T>::BuyPowerOrderByDistributorExecuted(
			beneficiary.clone(),
			power_amount,
			generator_nft_id,
		));

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

	fn upgrade_order_info_data_v2() -> Weight {
		log::info!("Start upgrading order info data v2");
		let mut num_order_queue_classes = 0;

		BuyPowerByUserRequestQueue::<T>::translate(|_k, _k2, order_info: OrderInfo<T::BlockNumber>| {
			num_order_queue_classes += 1;

			Some(OrderInfo {
				power_amount: order_info.power_amount,
				bit_amount: order_info.bit_amount,
				target: T::BlockNumber::zero(),
				commission_fee: 0,
			})
		});

		log::info!("Classes upgraded: {}", num_order_queue_classes);

		0
	}
}
