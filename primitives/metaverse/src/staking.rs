use codec::{Decode, Encode};
use sp_runtime::DispatchResult;
use sp_std::vec::Vec;

use crate::{RoundIndex, RuntimeDebug, TypeInfo};

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

impl<B: Copy + sp_std::ops::Add<Output=B> + sp_std::ops::Sub<Output=B> + From<u32> + PartialOrd> RoundInfo<B> {
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

impl<B: Copy + sp_std::ops::Add<Output=B> + sp_std::ops::Sub<Output=B> + From<u32> + PartialOrd> Default
for RoundInfo<B>
{
    fn default() -> RoundInfo<B> {
        RoundInfo::new(1u32, 1u32.into(), 20u32)
    }
}

pub trait MetaverseStakingTrait<Balance> {
    fn update_staking_reward(round: RoundIndex, total_reward: Balance) -> DispatchResult;
}
