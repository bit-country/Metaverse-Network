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
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

use primitives::SpotId;

/// Struct of every Continuum vote
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Vote<AccountId> {
	pub nay: bool,
	pub who: AccountId,
}

/// Keep track of voting activities of an account
#[derive(Encode, Decode, Clone, Eq, PartialEq, Default, RuntimeDebug, TypeInfo)]
pub struct Voting<AccountId> {
	pub votes: Vec<(SpotId, AccountVote<AccountId>)>,
}

/// A vote for a referendum of a particular account.
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum AccountVote<AccountId> {
	/// A standard continuum vote
	Standard { vote: Vote<AccountId> },
}

impl<AccountId> AccountVote<AccountId> {
	pub fn vote_who(self) -> Vote<AccountId> {
		match self {
			AccountVote::Standard { vote } => vote,
		}
	}
}
