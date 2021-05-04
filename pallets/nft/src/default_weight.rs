// This default_weight is manually generated for UI integration testing purpose 
// This bench_marking cli need to run to complete bench marking for all functions 

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

/// Weight functions needed for module_nft.
pub trait WeightInfo {
    fn mint(i: u32) -> Weight;
}

impl WeightInfo for () {
    fn mint(i: u32) -> Weight {
        (456_053_000 as Weight)
            .saturating_add((29_136_000 as Weight).saturating_mul(i as Weight))
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
            .saturating_add(DbWeight::get().writes((2 as Weight).saturating_mul(i as Weight)))
    }
}