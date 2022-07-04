#![cfg(test)]

#[cfg(any(feature = "with-metaverse-runtime", feature = "with-pioneer-runtime"))]
mod setup;

#[cfg(feature = "with-pioneer-runtime")]
mod xcm_transfers;

#[cfg(any(feature = "with-metaverse-runtime", feature = "with-pioneer-runtime"))]
mod weights;

#[cfg(feature = "with-pioneer-runtime")]
mod relaychain;

#[cfg(feature = "with-pioneer-runtime")]
mod nft;

#[cfg(feature = "with-pioneer-runtime")]
mod estate;
