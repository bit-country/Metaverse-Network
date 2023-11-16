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

use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{
	traits::{Bounded, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, IntegerSquareRoot, Saturating, Zero},
	Permill, RuntimeDebug,
};

use primitives::FungibleTokenId;

// Helper methods to compute the issuance rate for undeployed land.

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct PoolInfo<AccountId> {
	pub creator: AccountId,
	pub commission: Permill,
	/// Currency id of the pool
	pub currency_id: FungibleTokenId,
	/// Max nft rewards
	pub max: u32,
}

/// Amount of votes and capital placed in delegation for an account.
#[derive(Encode, Decode, Default, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct BoostingDelegations<Balance> {
	/// The number of votes (this is post-conviction).
	pub votes: Balance,
	/// The amount of raw capital, used for the turnout.
	pub capital: Balance,
}

/// A value denoting the strength of conviction of a vote.
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, TypeInfo)]
pub enum BoostingConviction {
	/// 0.1x votes, unlocked.
	None,
	/// 1x votes, locked for an enactment period following a successful vote.
	Locked1x,
	/// 2x votes, locked for 2x enactment periods following a successful vote.
	Locked2x,
	/// 3x votes, locked for 4x...
	Locked3x,
	/// 4x votes, locked for 8x...
	Locked4x,
	/// 5x votes, locked for 16x...
	Locked5x,
	/// 6x votes, locked for 32x...
	Locked6x,
}

impl Default for BoostingConviction {
	fn default() -> Self {
		BoostingConviction::None
	}
}

impl From<BoostingConviction> for u8 {
	fn from(c: BoostingConviction) -> u8 {
		match c {
			BoostingConviction::None => 0,
			BoostingConviction::Locked1x => 1,
			BoostingConviction::Locked2x => 2,
			BoostingConviction::Locked3x => 3,
			BoostingConviction::Locked4x => 4,
			BoostingConviction::Locked5x => 5,
			BoostingConviction::Locked6x => 6,
		}
	}
}

impl TryFrom<u8> for BoostingConviction {
	type Error = ();
	fn try_from(i: u8) -> Result<BoostingConviction, ()> {
		Ok(match i {
			0 => BoostingConviction::None,
			1 => BoostingConviction::Locked1x,
			2 => BoostingConviction::Locked2x,
			3 => BoostingConviction::Locked3x,
			4 => BoostingConviction::Locked4x,
			5 => BoostingConviction::Locked5x,
			6 => BoostingConviction::Locked6x,
			_ => return Err(()),
		})
	}
}

impl BoostingConviction {
	/// The amount of time (in number of periods) that our conviction implies a successful voter's
	/// balance should be locked for.
	pub fn lock_periods(self) -> u32 {
		match self {
			BoostingConviction::None => 0,
			BoostingConviction::Locked1x => 1,
			BoostingConviction::Locked2x => 2,
			BoostingConviction::Locked3x => 4,
			BoostingConviction::Locked4x => 8,
			BoostingConviction::Locked5x => 16,
			BoostingConviction::Locked6x => 32,
		}
	}

	/// The votes of a voter of the given `balance` with our conviction.
	pub fn votes<B: From<u8> + Zero + Copy + CheckedMul + CheckedDiv + Bounded>(
		self,
		capital: B,
	) -> BoostingDelegations<B> {
		let votes = match self {
			BoostingConviction::None => capital.checked_div(&10u8.into()).unwrap_or_else(Zero::zero),
			x => capital.checked_mul(&u8::from(x).into()).unwrap_or_else(B::max_value),
		};
		BoostingDelegations { votes, capital }
	}
}

impl Bounded for BoostingConviction {
	fn min_value() -> Self {
		BoostingConviction::None
	}
	fn max_value() -> Self {
		BoostingConviction::Locked6x
	}
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct BoostInfo<Balance> {
	pub(crate) balance: Balance,
	pub(crate) conviction: BoostingConviction,
}

impl<Balance: Saturating> BoostInfo<Balance> {
	/// Returns `Some` of the lock periods that the account is locked for, assuming that the
	/// referendum passed if `approved` is `true`.
	pub fn get_locked_period(self) -> (u32, Balance) {
		return (self.conviction.lock_periods(), self.balance);
	}
}
