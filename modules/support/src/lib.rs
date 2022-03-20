// This file is part of Bit.Country.

// Copyright (C) 2020-2021 Bit.Country.
// SPDX-License-Identifier: Apache-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::upper_case_acronyms)]

use codec::{Decode, Encode, FullCodec};
use frame_support::pallet_prelude::{DispatchClass, Pays, Weight};
use primitives::CurrencyId;
use sp_core::H160;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, CheckedDiv, MaybeSerializeDeserialize},
	transaction_validity::TransactionValidityError,
	DispatchError, DispatchResult, FixedU128, RuntimeDebug,
};
use sp_std::{
	cmp::{Eq, PartialEq},
	fmt::Debug,
	prelude::*,
};

use xcm::latest::prelude::*;

pub trait CallBuilder {
	type AccountId: FullCodec;
	type Balance: FullCodec;
	type RelayChainCall: FullCodec;

	/// Execute multiple calls in a batch.
	/// Param:
	/// - calls: List of calls to be executed
	fn utility_batch_call(calls: Vec<Self::RelayChainCall>) -> Self::RelayChainCall;

	/// Execute a call, replacing the `Origin` with a sub-account.
	///  params:
	/// - call: The call to be executed. Can be nested with `utility_batch_call`
	/// - index: The index of sub-account to be used as the new origin.
	fn utility_as_derivative_call(call: Self::RelayChainCall, index: u16) -> Self::RelayChainCall;

	/// Bond extra on relay-chain.
	///  params:
	/// - amount: The amount of staking currency to bond.
	fn staking_bond_extra(amount: Self::Balance) -> Self::RelayChainCall;

	/// Unbond on relay-chain.
	///  params:
	/// - amount: The amount of staking currency to unbond.
	fn staking_unbond(amount: Self::Balance) -> Self::RelayChainCall;

	/// Withdraw unbonded staking on the relay-chain.
	///  params:
	/// - num_slashing_spans: The number of slashing spans to withdraw from.
	fn staking_withdraw_unbonded(num_slashing_spans: u32) -> Self::RelayChainCall;

	/// Transfer Staking currency to another account, disallowing "death".
	///  params:
	/// - to: The destination for the transfer
	/// - amount: The amount of staking currency to be transferred.
	fn balances_transfer_keep_alive(to: Self::AccountId, amount: Self::Balance) -> Self::RelayChainCall;

	/// Wrap the final calls into the Xcm format.
	///  params:
	/// - call: The call to be executed
	/// - extra_fee: Extra fee (in staking currency) used for buy the `weight` and `debt`.
	/// - weight: the weight limit used for XCM.
	/// - debt: the weight limit used to process the `call`.
	fn finalize_call_into_xcm_message(call: Self::RelayChainCall, extra_fee: Self::Balance, weight: Weight) -> Xcm<()>;
}
//
// /// Dispatchable tasks
// pub trait DispatchableTask {
// 	fn dispatch(self, weight: Weight) -> TaskResult;
// }
//
// #[cfg(feature = "std")]
// impl DispatchableTask for () {
// 	fn dispatch(self, _weight: Weight) -> TaskResult {
// 		unimplemented!()
// 	}
// }
