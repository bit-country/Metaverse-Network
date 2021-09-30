use codec::{Encode, Decode};
use sp_runtime::{RuntimeDebug,traits::{Bounded, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, Saturating, Zero,One,Hash,IntegerSquareRoot}};
use sp_std::vec::Vec;
use primitives::{CountryId, ProposalId, ReferendumId,AccountId,Balance};
use frame_support::traits::CurrencyToVote;
use sp_std::ops::{Add, Div, Mul, Rem};
use crate::*;


#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub enum PreimageStatus<AccountId, Balance, BlockNumber> {
	/// The preimage is imminently needed at the argument.
	Missing(BlockNumber),
	/// The preimage is available.
	Available {
		data: Vec<u8>,
        does_update_jury: bool,
		provider: AccountId,
		deposit: Balance,
		since: BlockNumber,
		/// None if it's not imminent.
		expiry: Option<BlockNumber>,
	},
}

impl<AccountId, Balance, BlockNumber> PreimageStatus<AccountId, Balance, BlockNumber> {
	fn to_missing_expiry(self) -> Option<BlockNumber> {
		match self {
			PreimageStatus::Missing(expiry) => Some(expiry),
			_ => None,
		}
	}
}



#[derive(Encode, Decode,  Clone, PartialEq, Eq, RuntimeDebug)]
pub enum VoteThreshold {
    SuperMajorityApprove,
    SuperMajorityAgainst,
    RelativeMajority, // Most votes
    /* to be enabled later
    StandardQualifiedMajority, // 72%+ 72%+ representation
    TwoThirdsSupermajority, // 66%+
    ThreeFifthsSupermajority, // 60%+
    ReinforcedQualifiedMajority, // 55%+ 65%+ representation
    */
}

pub trait ReferendumApproved<Balance> {
	/// Given a `tally` of votes and a total size of `electorate`, this returns `true` if the
	/// overall outcome is in favor of approval according to `self`'s threshold method.
	fn is_referendum_approved(&self, tally: Tally<Balance>, electorate: Balance) -> bool;
}

/// Return `true` iff `n1 / d1 < n2 / d2`. `d1` and `d2` may not be zero.
fn compare_rationals<
	T: Zero + Mul<T, Output = T> + Div<T, Output = T> + Rem<T, Output = T> + Ord + Copy,
>(
	mut n1: T,
	mut d1: T,
	mut n2: T,
	mut d2: T,
) -> bool {
	// Uses a continued fractional representation for a non-overflowing compare.
	// Detailed at https://janmr.com/blog/2014/05/comparing-rational-numbers-without-overflow/.
	loop {
		let q1 = n1 / d1;
		let q2 = n2 / d2;
		if q1 < q2 {
			return true
		}
		if q2 < q1 {
			return false
		}
		let r1 = n1 % d1;
		let r2 = n2 % d2;
		if r2.is_zero() {
			return false
		}
		if r1.is_zero() {
			return true
		}
		n1 = d2;
		n2 = d1;
		d1 = r2;
		d2 = r1;
	}
}


impl<
		Balance: IntegerSquareRoot
            + Zero
			+ Ord
			+ Add<Balance, Output = Balance>
			+ Mul<Balance, Output = Balance>
			+ Div<Balance, Output = Balance>
			+ Rem<Balance, Output = Balance>
			+ Copy,
	> ReferendumApproved<Balance> for VoteThreshold
{
	fn is_referendum_approved(&self, tally: Tally<Balance>, electorate: Balance) -> bool {
		let sqrt_voters = tally.turnout.integer_sqrt();
		let sqrt_electorate = electorate.integer_sqrt();
		if sqrt_voters.is_zero() {
			return false
		}
		match *self {
			VoteThreshold::SuperMajorityApprove =>
				compare_rationals(tally.nays, sqrt_voters, tally.ayes, sqrt_electorate),
			VoteThreshold::SuperMajorityAgainst =>
				compare_rationals(tally.nays, sqrt_electorate, tally.ayes, sqrt_voters),
			VoteThreshold::RelativeMajority => tally.ayes > tally.nays,
		}

	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum CountryParameter {
    MaxProposals(u8),
    SetReferendumJury(AccountId),
}

#[derive(Encode, Decode,Default, Clone, RuntimeDebug, PartialEq, Eq)]
pub struct ReferendumParameters<BlockNumber> {
    pub(crate) voting_threshold: Option<VoteThreshold>,
    pub(crate) min_proposal_launch_period: BlockNumber,// number of blocks
    pub(crate) voting_period: BlockNumber, // number of blocks
    pub(crate) enactment_period: BlockNumber, // number of blocks
    pub(crate) local_vote_locking_period: BlockNumber, // number of blocks
    pub(crate) max_proposals_per_country: u8,
}
/*
impl<BlockNumber: From<u32> + Default> Default for ReferendumParameters<BlockNumber>{
    fn default() -> Self {
        ReferendumParameters {
            voting_threshold: Some(VoteThreshold::RelativeMajority),
            min_proposal_launch_period: T::Pallet::DefaultProposalLaunchPeriod::get(),
            voting_period:  T::DefaultVotingPeriod::get(), 
            enactment_period:  T::DefaultEnactmentPeriod::get(), 
          //  max_params_per_proposal:  T::DefaultMaxParametersPerProposal::get(),
            max_proposals_per_country: T::DefaultMaxProposalsPerCountry::get(),
        }
    }
}
*/
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Vote<Balance> {
    pub(crate) aye: bool,
    pub(crate) balance: Balance,
}

/// Tally Struct
#[derive(Encode, Decode,Default, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Tally<Balance> {
    pub(crate) ayes: Balance,
    pub(crate) nays: Balance,
    pub(crate) turnout: Balance,
}

impl <
    Balance: From<u8>
        + Zero
        + Copy
        + CheckedAdd
        + CheckedSub
        + CheckedMul
        + CheckedDiv
        + Bounded
        + Saturating,
> Tally<Balance> {

    /// Add an account's vote into the tally.
    pub fn add(
		&mut self,
		vote: Vote<Balance>,
	) -> Option<()> {
        match vote.aye {
            true => self.ayes = self.ayes.checked_add(&vote.balance)?,
            false => self.nays = self.nays.checked_add(&vote.balance)?,
        }
        self.turnout = self.turnout.checked_add(&vote.balance)?;
		Some(())
	}

    /// Add an account's vote into the tally.
    pub fn remove(
		&mut self,
		vote: Vote<Balance>,
	) -> Option<()> {
        match vote.aye {
            true => self.ayes = self.ayes.checked_sub(&vote.balance)?,
            false => self.nays = self.nays.checked_sub(&vote.balance)?,
        }
        self.turnout = self.turnout.checked_sub(&vote.balance)?;
		Some(())
	}


}


#[derive(Encode, Decode, Default,  Clone, PartialEq, Eq, RuntimeDebug)]
pub struct VotingRecord<Balance> {
    pub(crate) votes: Vec<(ReferendumId,Vote<Balance>)>
}


#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct ProposalInfo<AccountId,BlockNumber,Hash> {
    pub(crate) proposed_by: AccountId,
    pub(crate) hash: Hash,
    pub(crate) description: Vec<u8>, // link to proposal description
    pub(crate) referendum_launch_block: BlockNumber,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct ReferendumStatus<BlockNumber,Balance> {
    pub(crate) end: BlockNumber,
    pub(crate) country: CountryId,
    pub(crate) proposal: ProposalId,
    pub(crate) tally: Tally<Balance>,
    pub(crate) threshold: VoteThreshold,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum ReferendumInfo<BlockNumber,Balance> {
    Ongoing(ReferendumStatus<BlockNumber,Balance>),
    Finished{passed: bool, end: BlockNumber},
}