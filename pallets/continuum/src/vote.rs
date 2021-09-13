use crate::ContinuumSpotTally;
use codec::{Decode, Encode, EncodeLike, Input, Output};
use frame_support::sp_runtime::traits::AccountIdConversion;
use primitives::SpotId;
use sp_runtime::{
    traits::{Saturating, Zero},
    RuntimeDebug,
};
use sp_std::{convert::TryFrom, prelude::*, result::Result};
// use crate::mock::AccountId;

/// Struct of every Continuum vote
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Vote<AccountId> {
    pub nay: bool,
    pub who: AccountId,
}

impl<AccountId: From<u32> + Default> Default for Vote<AccountId> {
    fn default() -> Self {
        Vote {
            nay: false,
            who: Default::default(),
        }
    }
}

/// Keep track of voting activities of an account
#[derive(Encode, Decode, Clone, Eq, PartialEq, Default, RuntimeDebug)]
pub struct Voting<AccountId> {
    pub votes: Vec<(SpotId, AccountVote<AccountId>)>,
}

/// A vote for a referendum of a particular account.
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum AccountVote<AccountId> {
    /// A standard continuum vote
    Standard { vote: Vote<AccountId> },
}

// impl<AccountId: From<u32> + Default> Default for AccountVote<AccountId> {
//     fn default() -> Self {
//         AccountVote::Standard { vote: Default::default() }
//     }
// }

impl<AccountId> AccountVote<AccountId> {
    pub fn vote_who(self) -> Vote<AccountId> {
        match self {
            AccountVote::Standard { vote } => vote,
        }
    }
}
