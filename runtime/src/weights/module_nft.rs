#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for module_nft.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> nft::WeightInfo for WeightInfo<T> {
    fn mint(i: u32) -> Weight {
        (1_701_000 as Weight)
            // Standard Error: 5_000
            .saturating_add((18_283_000 as Weight).saturating_mul(i as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(i as Weight)))
    }
}
