#![cfg(test)]

#[cfg(any(feature = "with-metaverse-runtime", feature = "with-pioneer-runtime"))]
mod setup;

#[cfg(feature = "with-metaverse-runtime")]
mod xcm_transfers;

#[cfg(any(feature = "with-metaverse-runtime", feature = "with-pioneer-runtime"))]
mod weights;

#[cfg(any(feature = "with-metaverse-runtime", feature = "with-pioneer-runtime"))]
mod relaychain;

#[cfg(any(feature = "with-metaverse-runtime"))]
mod purchase_nft_from_buy_now_listing;

/*
#[cfg(any(
	feature = "with-metaverse-runtime",
	feature = "with-pioneer-runtime"
))]
mod win_and_deploy_land_block;

#[cfg(any(
	feature = "with-metaverse-runtime",
	feature = "with-pioneer-runtime"
))]
mod create_an_estate;

#[cfg(any(
	feature = "with-metaverse-runtime",
	feature = "with-pioneer-runtime"
))]
mod win_nft_auction;
*/
