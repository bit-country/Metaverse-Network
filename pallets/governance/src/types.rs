use crate::*;
use codec::{Decode, Encode};
use primitives::{MetaverseId, ProposalId, ReferendumId};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{Bounded, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, IntegerSquareRoot, Saturating, Zero},
	RuntimeDebug,
};
use sp_std::convert::TryFrom;
use sp_std::ops::{Add, Div, Mul, Rem};
use sp_std::vec::Vec;

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
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

impl<AccountId, Balance, BlockNumber> PreimageStatus<AccountId, Balance, BlockNumber> {
	fn to_missing_expiry(self) -> Option<BlockNumber> {
		match self {
			PreimageStatus::Missing(expiry) => Some(expiry),
			_ => None,
		}
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum VoteThreshold {
	SuperMajorityApprove,
	SuperMajorityAgainst,
	RelativeMajority, /* Most votes */
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
fn compare_rationals<T: Zero + Mul<T, Output = T> + Div<T, Output = T> + Rem<T, Output = T> + Ord + Copy>(
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
			return true;
		}
		if q2 < q1 {
			return false;
		}
		let r1 = n1 % d1;
		let r2 = n2 % d2;
		if r2.is_zero() {
			return false;
		}
		if r1.is_zero() {
			return true;
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
			return false;
		}
		match *self {
			VoteThreshold::SuperMajorityApprove => {
				compare_rationals(tally.nays, sqrt_voters, tally.ayes, sqrt_electorate)
			}
			VoteThreshold::SuperMajorityAgainst => {
				compare_rationals(tally.nays, sqrt_electorate, tally.ayes, sqrt_voters)
			}
			VoteThreshold::RelativeMajority => tally.ayes > tally.nays,
		}
	}
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub struct ReferendumParameters<BlockNumber> {
	pub(crate) voting_threshold: Option<VoteThreshold>,
	pub(crate) min_proposal_launch_period: BlockNumber, // number of blocks
	pub(crate) voting_period: BlockNumber,              // number of blocks
	pub(crate) enactment_period: BlockNumber,           // number of blocks
	pub(crate) local_vote_locking_period: BlockNumber,  // number of blocks
	pub(crate) max_proposals_per_metaverse: u8,
}

impl<BlockNumber: From<u32>> Default for ReferendumParameters<BlockNumber> {
	fn default() -> Self {
		ReferendumParameters {
			voting_threshold: Some(VoteThreshold::RelativeMajority),
			min_proposal_launch_period: 15u32.into(),
			voting_period: 100u32.into(),
			enactment_period: 10u32.into(),
			local_vote_locking_period: 28u32.into(),
			max_proposals_per_metaverse: 20,
		}
	}
}

/// Amount of votes and capital placed in delegation for an account.
#[derive(Encode, Decode, Default, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct Delegations<Balance> {
	/// The number of votes (this is post-conviction).
	pub votes: Balance,
	/// The amount of raw capital, used for the turnout.
	pub capital: Balance,
}

/// A value denoting the strength of conviction of a vote.
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, TypeInfo)]
pub enum Conviction {
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

impl Default for Conviction {
	fn default() -> Self {
		Conviction::None
	}
}

impl From<Conviction> for u8 {
	fn from(c: Conviction) -> u8 {
		match c {
			Conviction::None => 0,
			Conviction::Locked1x => 1,
			Conviction::Locked2x => 2,
			Conviction::Locked3x => 3,
			Conviction::Locked4x => 4,
			Conviction::Locked5x => 5,
			Conviction::Locked6x => 6,
		}
	}
}

impl TryFrom<u8> for Conviction {
	type Error = ();
	fn try_from(i: u8) -> Result<Conviction, ()> {
		Ok(match i {
			0 => Conviction::None,
			1 => Conviction::Locked1x,
			2 => Conviction::Locked2x,
			3 => Conviction::Locked3x,
			4 => Conviction::Locked4x,
			5 => Conviction::Locked5x,
			6 => Conviction::Locked6x,
			_ => return Err(()),
		})
	}
}

impl Conviction {
	/// The amount of time (in number of periods) that our conviction implies a successful voter's
	/// balance should be locked for.
	pub fn lock_periods(self) -> u32 {
		match self {
			Conviction::None => 0,
			Conviction::Locked1x => 1,
			Conviction::Locked2x => 2,
			Conviction::Locked3x => 4,
			Conviction::Locked4x => 8,
			Conviction::Locked5x => 16,
			Conviction::Locked6x => 32,
		}
	}

	/// The votes of a voter of the given `balance` with our conviction.
	pub fn votes<B: From<u8> + Zero + Copy + CheckedMul + CheckedDiv + Bounded>(self, capital: B) -> Delegations<B> {
		let votes = match self {
			Conviction::None => capital.checked_div(&10u8.into()).unwrap_or_else(Zero::zero),
			x => capital.checked_mul(&u8::from(x).into()).unwrap_or_else(B::max_value),
		};
		Delegations { votes, capital }
	}
}

impl Bounded for Conviction {
	fn min_value() -> Self {
		Conviction::None
	}
	fn max_value() -> Self {
		Conviction::Locked6x
	}
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct Vote<Balance> {
	pub(crate) aye: bool,
	pub(crate) balance: Balance,
	pub(crate) conviction: Conviction,
}

impl<Balance: Saturating> Vote<Balance> {
	/// Returns `Some` of the lock periods that the account is locked for, assuming that the
	/// referendum passed iff `approved` is `true`.
	pub fn locked_if(self, approved: bool) -> Option<(u32, Balance)> {
		// winning side: can only be removed after the lock period ends.
		if self.aye == approved {
			return Some((self.conviction.lock_periods(), self.balance));
		}
		None
	}
}
/// Tally Struct
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct Tally<Balance> {
	pub(crate) ayes: Balance,
	pub(crate) nays: Balance,
	pub(crate) turnout: Balance,
}
impl<Balance: From<u8> + Zero + Copy + CheckedAdd + CheckedSub + CheckedMul + CheckedDiv + Bounded + Saturating>
	Tally<Balance>
{
	/// Add an account's vote into the tally.
	pub fn add(&mut self, vote: Vote<Balance>) -> Option<()> {
		match vote.aye {
			true => self.ayes = self.ayes.checked_add(&vote.conviction.votes(vote.balance).votes)?,
			false => self.nays = self.nays.checked_add(&vote.conviction.votes(vote.balance).votes)?,
		}
		self.turnout = self.turnout.checked_add(&vote.conviction.votes(vote.balance).votes)?;
		Some(())
	}

	/// Add an account's vote into the tally.
	pub fn remove(&mut self, vote: Vote<Balance>) -> Option<()> {
		match vote.aye {
			true => self.ayes = self.ayes.checked_sub(&vote.conviction.votes(vote.balance).votes)?,
			false => self.nays = self.nays.checked_sub(&vote.conviction.votes(vote.balance).votes)?,
		}
		self.turnout = self.turnout.checked_sub(&vote.conviction.votes(vote.balance).votes)?;
		Some(())
	}
}
/// A "prior" lock, i.e. a lock for some now-forgotten reason.
#[derive(Encode, Decode, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, TypeInfo)]
pub struct PriorLock<BlockNumber, Balance>(BlockNumber, Balance);

impl<BlockNumber: Ord + Copy + Zero, Balance: Ord + Copy + Zero> PriorLock<BlockNumber, Balance> {
	/// Accumulates an additional lock.
	pub fn accumulate(&mut self, until: BlockNumber, amount: Balance) {
		self.0 = self.0.max(until);
		self.1 = self.1.max(amount);
	}

	pub fn locked(&self) -> Balance {
		self.1
	}

	pub fn rejig(&mut self, now: BlockNumber) {
		if now >= self.0 {
			self.0 = Zero::zero();
			self.1 = Zero::zero();
		}
	}
}
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct VotingRecord<Balance, BlockNumber> {
	pub(crate) votes: Vec<(ReferendumId, Vote<Balance>)>,
	pub(crate) prior: PriorLock<BlockNumber, Balance>,
}

impl<Balance: Saturating + Ord + Zero + Copy, BlockNumber: Ord + Copy + Zero> VotingRecord<Balance, BlockNumber> {
	pub fn rejig(&mut self, now: BlockNumber) {
		self.prior.rejig(now);
	}

	/// The amount of this account's balance that much currently be locked due to voting.
	pub fn locked_balance(&self) -> Balance {
		self.votes
			.iter()
			.map(|i| i.1.balance)
			.fold(self.prior.locked(), |a, i| a.max(i))
	}
}
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct ProposalInfo<AccountId, BlockNumber, Hash> {
	pub(crate) proposed_by: AccountId,
	pub(crate) hash: Hash,
	pub(crate) title: Vec<u8>,
	pub(crate) referendum_launch_block: BlockNumber,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct ReferendumStatus<BlockNumber, Balance, Hash> {
	pub(crate) end: BlockNumber,
	pub(crate) metaverse: MetaverseId,
	pub(crate) proposal: ProposalId,
	pub(crate) tally: Tally<Balance>,
	pub(crate) title: Vec<u8>,
	pub(crate) threshold: VoteThreshold,
	pub(crate) proposal_hash: Hash,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum ReferendumInfo<BlockNumber, Balance, Hash> {
	Ongoing(ReferendumStatus<BlockNumber, Balance, Hash>),
	Finished {
		title: Vec<u8>,
		passed: bool,
		end: BlockNumber,
	},
}
