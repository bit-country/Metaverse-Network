use codec::{Decode, Encode};
use primitives::SpotId;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

/// Struct of every Continuum vote
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
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
