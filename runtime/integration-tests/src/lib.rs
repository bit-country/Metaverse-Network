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
mod purchase_nft_from_buy_now_listing;

#[cfg(feature = "with-pioneer-runtime")]
mod deploy_land_block_won_from_an_auction;
/*
#[cfg(feature = "with-pioneer-runtime")]
mod create_an_estate_and_add_or_remove_land_units_from_it;

#[cfg(feature = "with-pioneer-runtime")]
mod win_bundle_from_an_auction;
*/
