use crate::{AccountId, RoundIndex, RuntimeDebug, TypeInfo};
use codec::{Decode, Encode, HasCompact};
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
/// The current round index and transition information
pub struct RoundInfo<BlockNumber> {
	/// Current round index
	pub current: RoundIndex,
	/// The first block of the current round
	pub first: BlockNumber,
	/// The length of the current round in number of blocks
	pub length: u32,
}

#[derive(Default, Encode, Decode, RuntimeDebug, TypeInfo)]
/// Snapshot of estate state at the start of the round for which they are selected
pub struct StakeSnapshot<AccountId, Balance> {
	pub stakers: Vec<Bond<AccountId, Balance>>,
	pub total_bond: Balance,
}

#[derive(Default, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Bond<AccountId, Balance> {
	pub staker: AccountId,
	pub amount: Balance,
}

/// A record for total rewards and total amount staked for an era
#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct StakingSnapshot<Balance> {
	/// Total amount of rewards for a staking round
	pub rewards: Balance,
	/// Total staked amount for a staking round
	pub staked: Balance,
}

/// Storing the reward detail of estate that store the list of stakers for each estate
/// This will be used to reward estate owner and the stakers.
#[derive(Clone, PartialEq, Encode, Decode, Default, RuntimeDebug, TypeInfo)]
pub struct StakingPoints<AccountId: Ord, Balance: HasCompact> {
	/// Total staked amount.
	pub total: Balance,
	/// The map of stakers and the amount they staked.
	pub stakers: BTreeMap<AccountId, Balance>,
	/// Accrued and claimed rewards on this estate for both estate owner and stakers
	pub claimed_rewards: Balance,
}

impl<B: Copy + sp_std::ops::Add<Output = B> + sp_std::ops::Sub<Output = B> + From<u32> + PartialOrd> RoundInfo<B> {
	pub fn new(current: RoundIndex, first: B, length: u32) -> RoundInfo<B> {
		RoundInfo { current, first, length }
	}
	/// Check if the round should be updated
	pub fn should_update(&self, now: B) -> bool {
		now - self.first >= self.length.into()
	}
	/// New round
	pub fn update(&mut self, now: B) {
		self.current += 1u32;
		self.first = now;
	}
}

impl<B: Copy + sp_std::ops::Add<Output = B> + sp_std::ops::Sub<Output = B> + From<u32> + PartialOrd> Default
	for RoundInfo<B>
{
	fn default() -> RoundInfo<B> {
		RoundInfo::new(1u32, 1u32.into(), 20u32)
	}
}
