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

use frame_support::pallet_prelude::*;
use sp_runtime::traits::Verify;

use crate::*;

impl<T: Config> Pallet<T> {
	/// Validates the signature of the given data with the provided signer's account ID.
	///
	/// # Errors
	///
	/// This function returns a [`WrongSignature`](crate::Error::WrongSignature) error if the
	/// signature is invalid or the verification process fails.
	pub fn validate_signature(
		data: &Vec<u8>,
		signature: &T::OffchainSignature,
		signer: &T::AccountId,
	) -> DispatchResult {
		if signature.verify(&**data, &signer) {
			return Ok(());
		}

		// NOTE: for security reasons modern UIs implicitly wrap the data requested to sign into
		// <Bytes></Bytes>, that's why we support both wrapped and raw versions.
		let prefix = b"<Bytes>";
		let suffix = b"</Bytes>";
		let mut wrapped: Vec<u8> = Vec::with_capacity(data.len() + prefix.len() + suffix.len());
		wrapped.extend(prefix);
		wrapped.extend(data);
		wrapped.extend(suffix);

		ensure!(signature.verify(&*wrapped, &signer), Error::<T, I>::WrongSignature);

		Ok(())
	}
}
