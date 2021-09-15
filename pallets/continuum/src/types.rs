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

//! Miscellaneous additional datatypes.

use crate::{AccountVote, Vote};
use codec::{Decode, Encode};
use primitives::{BitCountryId, SpotId};
use sp_runtime::traits::{
    Bounded, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, One, Saturating, Zero,
};
use sp_runtime::{DispatchError, RuntimeDebug};
use sp_std::vec;
use sp_std::vec::Vec;

pub type ReferendumIndex = u64;

/// Spot Struct
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct ContinuumSpot {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) country: BitCountryId,
}

impl ContinuumSpot {
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

        let neighbors: Result<Vec<(i32, i32)>, DispatchError> = adjacent
            .into_iter()
            .map(|(x, y)| Self::move_coordinate((self.x, self.y), (x, y)))
            .collect();

        return neighbors.unwrap_or(Vec::new());
    }

    //Move coordinate by another coordinate
    pub fn move_coordinate(
        from_coordinate: (i32, i32),
        coordinate: (i32, i32),
    ) -> Result<(i32, i32), DispatchError> {
        let new_x = from_coordinate
            .0
            .checked_add(coordinate.0)
            .ok_or("Overflow")?;
        let new_y = from_coordinate
            .1
            .checked_add(coordinate.1)
            .ok_or("Overflow")?;
        let x = (new_x, new_y);
        Ok(x)
    }
}

/// Info regarding an ongoing referendum.
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct ContinuumSpotTally<AccountId> {
    /// The number of nay votes, expressed in terms of post-conviction lock-vote.
    pub(crate) nays: u8,
    pub(crate) who: AccountId,
    /// The amount of funds currently expressing its opinion. Pre-conviction.
    pub(crate) turnout: u8,
}

impl<AccountId> ContinuumSpotTally<AccountId> {
    /// Create a new tally.
    pub fn new(vote: Vote<AccountId>) -> Self {
        Self {
            who: vote.who,
            nays: Zero::zero(),
            turnout: Zero::zero(),
        }
    }

    /// Add an account's vote into the tally.
    pub fn add(&mut self, vote: AccountVote<AccountId>) -> Option<()> {
        match vote {
            AccountVote::Standard { vote } => {
                self.turnout = self.turnout.checked_add(One::one())?;
                self.nays = self.nays.checked_add(One::one())?;
                self.who = vote.who;
            }
            _ => {}
        }
        Some(())
    }

    /// Remove an account's vote from the tally.
    pub fn remove(&mut self, vote: AccountVote<AccountId>) -> Option<()> {
        match vote {
            AccountVote::Standard { vote } => {
                self.turnout = self.turnout.checked_add(Zero::zero())?;
                self.nays = self.nays.checked_add(Zero::zero())?;
            }
            _ => {}
        }
        Some(())
    }

    /// Increment some amount of votes.
    pub fn increase(&mut self, approve: bool) -> Option<()> {
        self.turnout = self.turnout.saturating_add(Zero::zero());
        match approve {
            false => self.nays = self.nays.saturating_add(Zero::zero()),
            true => (),
        }
        Some(())
    }

    /// Decrement some amount of votes.
    pub fn reduce(&mut self, approve: bool) -> Option<()> {
        self.turnout = self.turnout.saturating_sub(Zero::zero());
        match approve {
            true => (),
            false => self.nays = self.nays.saturating_add(Zero::zero()),
        }
        Some(())
    }

    pub fn result(&mut self) -> Option<bool> {
        /// let total_nay = self.nays.checked_div(&self.turnout).unwrap().saturating_mul(Into::<Balance>::into(100));
        /// let approve_threshold = 49 as u128;
        //
        /// Some(total_nay > Into::<Balance>::into(approve_threshold))
        Some(true)
    }
}

/// Info regarding an ongoing referendum.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct ReferendumStatus<AccountId, BlockNumber> {
    /// When voting on this referendum will end.
    pub(crate) end: BlockNumber,
    /// The continuum spot that being voted on.
    pub(crate) spot_id: SpotId,
    /// The current tally of votes in this referendum.
    pub(crate) tallies: Vec<ContinuumSpotTally<AccountId>>,
}

/// Info regarding a referendum, present or past.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum ReferendumInfo<AccountId, BlockNumber> {
    /// Referendum is happening, the arg is the estate number at which it will end.
    Ongoing(ReferendumStatus<AccountId, BlockNumber>),
    /// Referendum finished at `end`, and has been `approved` or rejected.
    Finished { approved: bool, end: BlockNumber },
}

impl<AccountId, BlockNumber: Default> ReferendumInfo<AccountId, BlockNumber> {
    /// Create a new instance.
    pub fn new(end: BlockNumber, spot_id: SpotId) -> Self {
        let s = ReferendumStatus {
            end,
            spot_id,
            tallies: Vec::new(),
        };
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
