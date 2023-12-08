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
use frame_support::traits::{ExistenceRequirement, LockIdentifier};
use frame_support::{
	dispatch::DispatchResult,
	ensure, log,
	traits::{Currency, Get},
	transactional, PalletId,
};
use frame_system::ensure_signed;
use frame_system::pallet_prelude::*;
use orml_traits::XcmTransfer;
use orml_traits::{MultiCurrency, RewardHandler};
use sp_runtime::traits::{
	BlockNumberProvider, Bounded, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, One, UniqueSaturatedInto,
};
use sp_runtime::{
	traits::{AccountIdConversion, Convert, Saturating, Zero},
	ArithmeticError, DispatchError, FixedPointNumber, Perbill, Permill, SaturatedConversion,
};
use xcm::{prelude::*, v3::Weight as XcmWeight};

use core_primitives::*;
use module_support::CallBuilder;
pub use pallet::*;
use primitives::{
	bounded::Rate, AccountId, Balance, ClassId, EraIndex, FungibleTokenId, PoolId, Ratio, StakingRound, TokenId,
};
use utils::SppAccountXcmHelper;

mod utils;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo)]
	pub enum XcmInterfaceOperation {
		// XTokens
		XtokensTransfer,
		// Spp
		PayoutReward,
		WithdrawUnbonded,
		BondExtra,
		Unbond,
		// Parachain fee with location info
		ParachainFee(Box<MultiLocation>),
	}

	/// The dest weight limit and fee for execution XCM msg sent by XcmInterface. Must be
	/// sufficient, otherwise the execution of XCM msg on relaychain will fail.
	///
	/// XcmDestWeightAndFee: map: XcmInterfaceOperation => (Weight, Balance)
	#[pallet::storage]
	#[pallet::getter(fn xcm_dest_weight_and_fee)]
	pub type XcmDestWeightAndFee<T: Config> =
		StorageMap<_, Twox64Concat, XcmInterfaceOperation, (Weight, Balance), ValueQuery>;

	#[pallet::pallet]
	#[pallet::generate_store(trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_xcm::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The currency id of the Staking asset
		#[pallet::constant]
		type StakingCurrencyId: Get<FungibleTokenId>;

		/// The account of parachain on the relaychain.
		#[pallet::constant]
		type ParachainAccount: Get<Self::AccountId>;

		/// The convert for convert sovereign subacocunt index to the MultiLocation where the
		/// staking currencies are sent to.
		type SovereignSubAccountLocationConvert: Convert<u16, MultiLocation>;

		/// The interface to Cross-chain transfer.
		type XcmTransfer: XcmTransfer<Self::AccountId, Balance, FungibleTokenId>;

		/// Origin represented Governance
		type GovernanceOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;

		/// Self parachain location.
		#[pallet::constant]
		type SelfLocation: Get<MultiLocation>;

		/// Convert AccountId to MultiLocation to build XCM message.
		type AccountIdToMultiLocation: Convert<Self::AccountId, MultiLocation>;

		/// The Call builder for communicating with RelayChain via XCM messaging.
		type RelayChainCallBuilder: CallBuilder<AccountId = Self::AccountId, Balance = Balance>;

		/// Unbonding slashing spans for unbonding on the relaychain.
		#[pallet::constant]
		type RelayChainUnbondingSlashingSpans: Get<EraIndex>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Xcm dest weight has been updated.
		XcmDestWeightUpdated {
			xcm_operation: XcmInterfaceOperation,
			new_xcm_dest_weight: XcmWeight,
		},
		/// Xcm dest weight has been updated.
		XcmFeeUpdated {
			xcm_operation: XcmInterfaceOperation,
			new_xcm_dest_weight: Balance,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The xcm operation have failed
		XcmFailed,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Sets the xcm_dest_weight and fee for XCM operation of XcmInterface.
		///
		/// Parameters:
		/// - `updates`: vec of tuple: (XcmInterfaceOperation, WeightChange, FeeChange).
		#[pallet::call_index(0)]
		#[pallet::weight({10_000_000})]
		pub fn update_xcm_dest_weight_and_fee(
			origin: OriginFor<T>,
			updates: Vec<(XcmInterfaceOperation, Option<Weight>, Option<Balance>)>,
		) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;

			for (operation, weight_change, fee_change) in updates {
				XcmDestWeightAndFee::<T>::mutate(&operation, |(weight, fee)| {
					if let Some(new_weight) = weight_change {
						*weight = new_weight;
						Self::deposit_event(Event::<T>::XcmDestWeightUpdated {
							xcm_operation: operation.clone(),
							new_xcm_dest_weight: new_weight,
						});
					}
					if let Some(new_fee) = fee_change {
						*fee = new_fee;
						Self::deposit_event(Event::<T>::XcmFeeUpdated {
							xcm_operation: operation.clone(),
							new_xcm_dest_weight: new_fee,
						});
					}
				});
			}

			Ok(())
		}
	}

	impl<T: Config> SppAccountXcmHelper<T::AccountId, Balance> for Pallet<T> {
		fn transfer_staking_to_sub_account(
			sender: &T::AccountId,
			sub_account_index: u16,
			amount: Balance,
		) -> DispatchResult {
			T::XcmTransfer::transfer(
				sender.clone(),
				T::StakingCurrencyId::get(),
				amount,
				T::SovereignSubAccountLocationConvert::convert(sub_account_index),
				Limited(Self::xcm_dest_weight_and_fee(XcmInterfaceOperation::XtokensTransfer).0),
			)
			.map(|_| ())
		}

		fn withdraw_unbonded_from_sub_account(sub_account_index: u16, amount: Balance) -> DispatchResult {
			let (xcm_dest_weight, xcm_fee) = Self::xcm_dest_weight_and_fee(XcmInterfaceOperation::WithdrawUnbonded);

			let xcm_message = T::RelayChainCallBuilder::finalize_multiple_calls_into_xcm_message(
				vec![
					(
						T::RelayChainCallBuilder::utility_as_derivative_call(
							T::RelayChainCallBuilder::staking_withdraw_unbonded(
								T::RelayChainUnbondingSlashingSpans::get(),
							),
							sub_account_index,
						),
						xcm_dest_weight,
					),
					(
						T::RelayChainCallBuilder::utility_as_derivative_call(
							T::RelayChainCallBuilder::balances_transfer_keep_alive(T::ParachainAccount::get(), amount),
							sub_account_index,
						),
						xcm_dest_weight,
					),
				],
				xcm_fee.saturating_mul(2),
			);

			let result = pallet_xcm::Pallet::<T>::send_xcm(Here, Parent, xcm_message);
			log::debug!(
				target: "xcm-interface",
				"subaccount {:?} send XCM to withdraw unbonded {:?}, result: {:?}",
				sub_account_index, amount, result
			);

			ensure!(result.is_ok(), Error::<T>::XcmFailed);
			Ok(())
		}

		fn bond_extra_on_sub_account(sub_account_index: u16, amount: Balance) -> DispatchResult {
			let (xcm_dest_weight, xcm_fee) = Self::xcm_dest_weight_and_fee(XcmInterfaceOperation::BondExtra);
			let xcm_message = T::RelayChainCallBuilder::finalize_call_into_xcm_message(
				T::RelayChainCallBuilder::utility_as_derivative_call(
					T::RelayChainCallBuilder::staking_bond_extra(amount),
					sub_account_index,
				),
				xcm_fee,
				xcm_dest_weight,
			);
			let result = pallet_xcm::Pallet::<T>::send_xcm(Here, Parent, xcm_message);
			log::debug!(
				target: "xcm-interface",
				"subaccount {:?} send XCM to bond {:?}, result: {:?}",
				sub_account_index, amount, result,
			);

			ensure!(result.is_ok(), Error::<T>::XcmFailed);
			Ok(())
		}

		fn unbond_on_sub_account(sub_account_index: u16, amount: Balance) -> DispatchResult {
			let (xcm_dest_weight, xcm_fee) = Self::xcm_dest_weight_and_fee(XcmInterfaceOperation::Unbond);
			let xcm_message = T::RelayChainCallBuilder::finalize_call_into_xcm_message(
				T::RelayChainCallBuilder::utility_as_derivative_call(
					T::RelayChainCallBuilder::staking_unbond(amount),
					sub_account_index,
				),
				xcm_fee,
				xcm_dest_weight,
			);
			let result = pallet_xcm::Pallet::<T>::send_xcm(Here, Parent, xcm_message);
			log::debug!(
				target: "xcm-interface",
				"subaccount {:?} send XCM to unbond {:?}, result: {:?}",
				sub_account_index, amount, result
			);

			ensure!(result.is_ok(), Error::<T>::XcmFailed);
			Ok(())
		}

		/// The fee of cross-chain transfer is deducted from the recipient.
		fn get_xcm_transfer_fee() -> Balance {
			Self::xcm_dest_weight_and_fee(XcmInterfaceOperation::XtokensTransfer).1
		}

		/// The fee of parachain transfer.
		fn get_parachain_fee(location: MultiLocation) -> Balance {
			Self::xcm_dest_weight_and_fee(XcmInterfaceOperation::ParachainFee(Box::new(location))).1
		}
	}
}
