// This pallet use The Open Runtime Module Library (ORML) which is a community maintained collection of Substrate runtime modules.
// Thanks to all contributors of orml.
// Ref: https://github.com/open-web3-stack/open-runtime-module-library
#![cfg_attr(not(feature = "std"), no_std)]

use codec::FullCodec;
use codec::{Decode, Encode};
use sp_runtime::{
    traits::{AtLeast32Bit, MaybeSerializeDeserialize},
    DispatchError, DispatchResult, RuntimeDebug,
};
use sp_std::{
    cmp::{Eq, PartialEq},
    fmt::Debug,
};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use primitives::AccountId;

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum BlindBoxType {
    KSM,
    NUUM,
    MainnetNFTHat1,
    MainnetNFTJacket1,
    MainnetNFTPants1,
    MainnetNFTShoes1,
    MainnetNFTHat2,
    MainnetNFTJacket2,
    MainnetNFTPants2,
    MainnetNFTShoes2,
    CollectableNFT
}

#[cfg_attr(feature = "std", derive(PartialEq, Eq))]
#[derive(Encode, Decode, Clone, RuntimeDebug)]
pub struct BlindBoxRewardItem<AccountId> {
    pub recipient: AccountId,
    /// amount for blindbox
    pub amount: u32,
    /// BlindBox type
    pub blindbox_type: BlindBoxType
}

