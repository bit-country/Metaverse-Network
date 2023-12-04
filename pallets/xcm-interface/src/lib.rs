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
pub use pallet::*;
use primitives::bounded::Rate;
use primitives::{ClassId, EraIndex, FungibleTokenId, PoolId, Ratio, StakingRound, TokenId};
pub use weights::WeightInfo;

pub type QueueId = u32;
//#[cfg(feature = "runtime-benchmarks")]
//pub mod benchmarking;

const BOOSTING_ID: LockIdentifier = *b"bc/boost";

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo)]
	pub enum XcmInterfaceOperation {
		// XTokens
		XtokensTransfer,
		// Spp
		WithdrawUnbonded,
		BondExtra,
		Unbond,
		// Parachain fee with location info
		ParachainFee(Box<MultiLocation>),
	}

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
		type XcmTransfer: XcmTransfer<Self::AccountId, Balance, CurrencyId>;

		/// Origin represented Governance
		type GovernanceOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;

		/// Self parachain location.
		#[pallet::constant]
		type SelfLocation: Get<MultiLocation>;

		/// Convert AccountId to MultiLocation to build XCM message.
		type AccountIdToMultiLocation: Convert<Self::AccountId, MultiLocation>;
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
	impl<T: Config> Pallet<T> {}
}
