// This file is part of Substrate.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
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

//! Miscellaneous additional datatypes.

use codec::{Encode, Decode};
use sp_runtime::RuntimeDebug;
use sp_runtime::traits::{Zero, Bounded, CheckedAdd, CheckedSub, CheckedMul, CheckedDiv, Saturating};
use crate::{Vote, AccountVote, Conviction};
use frame_support::sp_runtime::traits::One;

pub type ReferendumIndex = u64;

/// Spot Struct
pub struct ContinuumSpot<AccountId> {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

impl<AccountId: From<u32> + Zero + Copy + CheckedAdd + CheckedSub + CheckedMul + CheckedDiv + Bounded +
Saturating> ContinuumSpot<AccountId> {
    pub fn find_neighbour(&self) -> Vec<(i32, i32)> {
        let adjacent = vec![
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];

        let neighbors: Vec<(i32, i32)> = adjacent
            .into_iter()
            .map(|(x, y)| Self::move_coordinate((self.x, self.y), (x, y)))
            .collect();

        return neighbors;
    }

    //Move coordinate by another coordinate
    pub fn move_coordinate(from_coordinate: (i32, i32), coordinate: (i32, i32)) -> (i32, i32) {
        (from_coordinate.0.checked_add(coordinate.0)?, from_coordinate.1.checked_add(coordinate.1)?)
    }
}

/// Info regarding an ongoing referendum.
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct ContinuumSpotTally<AccountId, Balance> {
    /// The number of nay votes, expressed in terms of post-conviction lock-vote.
    pub(crate) nays: Balance,
    pub(crate) who: AccountId,
    /// The amount of funds currently expressing its opinion. Pre-conviction.
    pub(crate) turnout: Balance,
}

impl<
    AccountId: From<u32>,
    Balance: From<u8> + Zero + Copy + CheckedAdd + CheckedSub + CheckedMul + CheckedDiv + Bounded +
    Saturating
> ContinuumSpotTally<AccountId, Balance> {
    /// Create a new tally.
    pub fn new(
        vote: Vote<AccountId>
    ) -> Self {
        Self {
            who: vote.who,
            nays: Zero::zero(),
            turnout: One::one(),
        }
    }

    /// Add an account's vote into the tally.
    pub fn add(
        &mut self,
        vote: AccountVote<AccountId>,
    ) -> Option<()> {
        match vote {
            AccountVote::Standard { vote } => {
                self.turnout = self.turnout.checked_add(One::one())?;
                self.nays = self.nays.checked_add(&One::one())?;
                self.who = vote.who;
            }
            _ => {}
        }
        Some(())
    }

    /// Remove an account's vote from the tally.
    pub fn remove(
        &mut self,
        vote: AccountVote<AccountId>,
    ) -> Option<()> {
        match vote {
            AccountVote::Standard { vote } => {
                self.turnout = self.turnout.checked_sub(&One::one())?;
                self.nays = self.nays.checked_sub(&One::one())?;
            }
            _ => {}
        }
        Some(())
    }

    /// Increment some amount of votes.
    pub fn increase(&mut self, approve: bool) -> Option<()> {
        self.turnout = self.turnout.saturating_add(One::one());
        match approve {
            false => self.nays = self.nays.saturating_add(One::one()),
            true => ()
        }
        Some(())
    }

    /// Decrement some amount of votes.
    pub fn reduce(&mut self, approve: bool) -> Option<()> {
        self.turnout = self.turnout.saturating_sub(One::one());
        match approve {
            true => (),
            false => self.nays = self.nays.saturating_add(One::one()),
        }
        Some(())
    }
}

/// Info regarding an ongoing referendum.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct ReferendumStatus<AccountId, BlockNumber, Hash, Balance> {
    /// When voting on this referendum will end.
    pub(crate) end: BlockNumber,
    /// The continuum spot that being voted on.
    pub(crate) spot_id: SpotId,
    /// The current tally of votes in this referendum.
    pub(crate) tallies: Vec<ContinuumSpotTally<AccountId, Balance>>,
}

/// Info regarding a referendum, present or past.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum ReferendumInfo<AccountId, BlockNumber, Hash, Balance> {
    /// Referendum is happening, the arg is the block number at which it will end.
    Ongoing(ReferendumStatus<AccountId, BlockNumber, Hash, Balance>),
    /// Referendum finished at `end`, and has been `approved` or rejected.
    Finished { approved: bool, end: BlockNumber },
}

impl<AccountId, BlockNumber, Hash, Balance: Default> ReferendumInfo<AccountId, BlockNumber, Hash, Balance> {
    /// Create a new instance.
    pub fn new(
        end: BlockNumber,
        spot_id: SpotId,
        // threshold: VoteThreshold,
        delay: BlockNumber,
    ) -> Self {
        let s = ReferendumStatus { end, spot_id, delay, tallies: Vec::new() };
        ReferendumInfo::Ongoing(s)
    }
}

/// Whether an `unvote` operation is able to make actions that are not strictly always in the
/// interest of an account.
pub enum UnvoteScope {
    /// Permitted to do everything.
    Any,
    /// Permitted to do only the changes that do not need the owner's permission.
    OnlyExpired,
}
