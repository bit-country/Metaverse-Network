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
#![allow(clippy::unused_unit)]

use frame_support::{
	dispatch::{CallMetadata, GetCallMetadata},
	pallet_prelude::*,
	traits::{Contains, PalletInfoAccess},
	transactional,
};
use frame_system::pallet_prelude::*;
use sp_runtime::DispatchResult;
use sp_std::{prelude::*, vec::Vec};

pub use module::*;

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The origin which may set filter.
		type EmergencyOrigin: EnsureOrigin<Self::Origin>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Can not stop emergency call
		CannotStopEmergencyCall,
		/// invalid character encoding
		InvalidPalletAndFunction,
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// Stopped transaction
		EmergencyStopped {
			pallet_name_bytes: Vec<u8>,
			function_name_bytes: Vec<u8>,
		},
		/// Unstopped transaction
		EmergencyUnStopped {
			pallet_name_bytes: Vec<u8>,
			function_name_bytes: Vec<u8>,
		},
	}

	/// The paused transaction map
	///
	/// map (PalletNameBytes, FunctionNameBytes) => Option<()>
	#[pallet::storage]
	#[pallet::getter(fn emergency_stopped_pallets)]
	pub type EmergencyStoppedPallets<T: Config> = StorageMap<_, Twox64Concat, (Vec<u8>, Vec<u8>), (), OptionQuery>;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		#[transactional]
		pub fn emergency_stop(origin: OriginFor<T>, pallet_name: Vec<u8>, function_name: Vec<u8>) -> DispatchResult {
			T::EmergencyOrigin::ensure_origin(origin)?;

			// not allowed to pause calls of this pallet to ensure safe
			let pallet_name_string =
				sp_std::str::from_utf8(&pallet_name).map_err(|_| Error::<T>::InvalidPalletAndFunction)?;
			ensure!(
				pallet_name_string != <Self as PalletInfoAccess>::name(),
				Error::<T>::CannotStopEmergencyCall
			);

			EmergencyStoppedPallets::<T>::mutate_exists((pallet_name.clone(), function_name.clone()), |maybe_paused| {
				if maybe_paused.is_none() {
					*maybe_paused = Some(());
					Self::deposit_event(Event::EmergencyStopped {
						pallet_name_bytes: pallet_name,
						function_name_bytes: function_name,
					});
				}
			});
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		#[transactional]
		pub fn emergency_unstop(origin: OriginFor<T>, pallet_name: Vec<u8>, function_name: Vec<u8>) -> DispatchResult {
			T::EmergencyOrigin::ensure_origin(origin)?;
			if EmergencyStoppedPallets::<T>::take((&pallet_name, &function_name)).is_some() {
				Self::deposit_event(Event::EmergencyUnStopped {
					pallet_name_bytes: pallet_name,
					function_name_bytes: function_name,
				});
			};
			Ok(())
		}
	}
}

pub struct EmergencyStoppedFilter<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> Contains<T::Call> for EmergencyStoppedFilter<T>
where
	<T as frame_system::Config>::Call: GetCallMetadata,
{
	fn contains(call: &T::Call) -> bool {
		let CallMetadata {
			function_name,
			pallet_name,
		} = call.get_call_metadata();

		EmergencyStoppedPallets::<T>::contains_key((pallet_name.as_bytes(), function_name.as_bytes()))
	}
}
