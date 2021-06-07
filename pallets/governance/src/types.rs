use codec::{Encode, Decode};
use sp_runtime::{RuntimeDebug, DispatchError};
use sp_std::vec::Vec;
use primitives::{CountryId, AccountId,BlockNumber,ProposalId, ReferendumId,Balance};
use crate::*;


#[derive(Encode, Decode,  Clone, PartialEq, Eq, RuntimeDebug)]
pub enum VoteThreshold {
    StandardQualifiedMajority, // 72%+ 72%+ representation
    TwoThirdsSupermajority, // 66%+
    ThreeFifthsSupermajority, // 60%+
    ReinforcedQualifiedMajority, // 55%+ 65%+ representation
    AbsoluteMajority, // 50%+
    RelativeMajority, // Most votes
}

#[derive(Encode, Decode,  Clone, PartialEq, Eq, RuntimeDebug)]
pub enum CountryParameter {
    MaxProposals(u8),
    MaxParametersPerProposal(u8),
    SetReferendumReviewer(AccountId),
}

#[derive(Encode, Decode, Default, Clone, RuntimeDebug, PartialEq, Eq)]
pub struct ReferendumParameters<BlockNumber> {
    pub(crate) voting_threshold: Option<VoteThreshold>,
    pub(crate) min_proposal_launch_period: BlockNumber,//ProposalLaunchPeriod, // number of blocks
    pub(crate) voting_period: BlockNumber, // number of blocks
    pub(crate) enactment_period: BlockNumber, // number of blocks
    pub(crate) max_params_per_proposal: u8,
    pub(crate) max_proposals_per_country: u8,
}




#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Vote<Balance> {
   // pub(crate) who: AccountId,
    pub aye: bool,
    pub balance: Balance,
}

/// Tally Struct
#[derive(Encode, Decode,Default, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Tally<Balance> {
    pub(crate) ayes: Balance,
    pub(crate) nays: Balance,
    pub(crate) turnout: u64,
}


#[derive(Encode, Decode, Default,  Clone, PartialEq, Eq, RuntimeDebug)]
pub struct VotingRecord<Balance>  {
    pub(crate) votes: Vec<(ReferendumId,Vote<Balance>)>
}


#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct ProposalInfo<AccountId,BlockNumber,CountryParameter> {
    pub(crate) proposed_by: AccountId,
    pub(crate) parameters: Vec<CountryParameter>,
    pub(crate) description: Vec<u8>, // link to proposal description
    pub(crate) referendum_launch_block: BlockNumber,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct ReferendumStatus<BlockNumber> {
    pub(crate) end: BlockNumber,
    pub(crate) country: CountryId,
    pub(crate) proposal: ProposalId,
    pub(crate) tally: Tally<Balance>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum ReferendumInfo<BlockNumber> {
    Ongoing(ReferendumStatus<BlockNumber>),
    Finished{passed: bool, end: BlockNumber},
}





