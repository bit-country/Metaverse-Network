#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};

#[test]
// Private new_auction should work
fn create_new_auction_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(NftAuctionModule::new_auction(BOB, 100, 0, None), 0);
    });
}