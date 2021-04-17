use sp_std::{prelude::*, result::Result, convert::TryFrom};
use codec::{Encode, EncodeLike, Decode, Output, Input};
use sp_runtime::{RuntimeDebug, traits::{Saturating, Zero}};
use crate::{Conviction, ReferendumIndex, Delegations, ContinuumSpotTally};
use crate::mock::AccountId;

/// Struct of every Continuum vote
#[derive(Copy, Clone, Eq, PartialEq, Default, RuntimeDebug)]
pub struct Vote<AccountId> {
    pub nay: bool,
    pub who: AccountId,
}

/// Keep track of voting activities of an account
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Voting<Balance, AccountId, BlockNumber> {
    pub votes: Vec<(SpotId, AccountVote<T::AccountId>)>
}

/// A vote for a referendum of a particular account.
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum AccountVote<AccountId> {
    /// A standard continuum vote
    Standard { vote: Vote<AccountId> },
}

impl<AccountId: From<u32>> AccountVote<AccountId> {
    pub fn vote_who(&self) -> AccountId {
        match self {
            AccountVote::Standard { vote } => &vote.who
        }
    }
}



