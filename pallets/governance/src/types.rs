use crate::*;
use codec::{Decode, Encode};
use primitives::{AccountId, MetaverseId, ProposalId, ReferendumId};
use sp_runtime::{traits::One, RuntimeDebug};
use sp_std::vec::Vec;

#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub enum PreimageStatus<AccountId, Balance, BlockNumber> {
	/// The preimage is imminently needed at the argument.
	Missing(BlockNumber),
	/// The preimage is available.
	Available {
		data: Vec<u8>,
		provider: AccountId,
		deposit: Balance,
		since: BlockNumber,
		/// None if it's not imminent.
		expiry: Option<BlockNumber>,
	},
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum VoteThreshold {
	StandardQualifiedMajority,   // 72%+ 72%+ representation
	TwoThirdsSupermajority,      // 66%+
	ThreeFifthsSupermajority,    // 60%+
	ReinforcedQualifiedMajority, // 55%+ 65%+ representation
	RelativeMajority,            // Most votes
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum MetaverseParameter {
	MaxProposals(u8),
	SetReferendumJury(AccountId),
}

#[derive(Encode, Decode, Default, Clone, RuntimeDebug, PartialEq, Eq)]
pub struct ReferendumParameters<BlockNumber> {
	pub(crate) voting_threshold: Option<VoteThreshold>,
	pub(crate) min_proposal_launch_period: BlockNumber, // number of blocks
	pub(crate) voting_period: BlockNumber,              // number of block
	pub(crate) enactment_period: BlockNumber,           // number of blocks
	pub(crate) max_proposals_per_metaverse: u8,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Vote {
	pub(crate) aye: bool,
}

/// Tally Struct
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Tally {
	pub(crate) ayes: u32,
	pub(crate) nays: u32,
	pub(crate) turnout: u32,
}

impl Tally {
	/// Add an account's vote into the tally.
	pub fn add(&mut self, vote: Vote) -> Option<()> {
		match vote.aye {
			true => self.ayes = self.ayes.checked_add(One::one())?,
			false => self.nays = self.nays.checked_add(One::one())?,
		}
		self.turnout = self.ayes.checked_add(One::one())?;
		Some(())
	}

	/// Add an account's vote into the tally.
	pub fn remove(&mut self, vote: Vote) -> Option<()> {
		match vote.aye {
			true => self.ayes = self.ayes.checked_sub(One::one())?,
			false => self.nays = self.nays.checked_sub(One::one())?,
		}
		self.turnout = self.ayes.checked_sub(One::one())?;
		Some(())
	}
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct VotingRecord {
	pub(crate) votes: Vec<(ReferendumId, Vote)>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct ProposalInfo<AccountId, BlockNumber, Hash> {
	pub(crate) proposed_by: AccountId,
	pub(crate) hash: Hash,
	pub(crate) description: Vec<u8>, // link to proposal description
	pub(crate) referendum_launch_block: BlockNumber,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct ReferendumStatus<BlockNumber> {
	pub(crate) end: BlockNumber,
	pub(crate) metaverse: MetaverseId,
	pub(crate) proposal: ProposalId,
	pub(crate) tally: Tally,
	pub(crate) threshold: Option<VoteThreshold>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum ReferendumInfo<BlockNumber> {
	Ongoing(ReferendumStatus<BlockNumber>),
	Finished { passed: bool, end: BlockNumber },
}
