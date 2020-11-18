#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use sp_runtime::{
    generic,
    traits::{BlakeTwo256, IdentifyAccount, Verify},
    MultiSignature, RuntimeDebug,
};

/// An index to a block.
pub type BlockNumber = u32;
/// Alias to 512-bit hash when used in the context of a transaction signature on
/// the chain.
pub type Signature = MultiSignature;
/// Alias to the public key used for this chain, actually a `MultiSigner`. Like
/// the signature, this also isn't a fixed size when encoded, as different
/// cryptos have different size public keys.
pub type AccountPublic = <Signature as Verify>::Signer;
/// Alias to the opaque account ID type for this chain, actually a
/// `AccountId32`. This is always 32 bytes.
pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;
/// Balance of an account.
pub type Balance = u128;
/// Country Id
pub type CountryId = u64;
pub type Amount = i128;
pub type CurrencyId = u32;
