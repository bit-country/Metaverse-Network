// This default_weight is manually generated for UI integration testing purpose
// This bench_marking cli need to run to complete bench marking for all functions

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for module_nft.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> nft::WeightInfo for WeightInfo<T> {
	fn mint() -> Weight {
		(1_621_000 as Weight)
			// Standard Error: 5_000
			.saturating_add((21_976_000 as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
			.saturating_add(T::DbWeight::get().writes((2 as Weight)))
	}

	fn create_group() -> Weight {
		(60_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn create_class() -> Weight {
		(225_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	fn transfer() -> Weight {
		(176_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
	}
	fn transfer_batch() -> Weight {
		(447_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(8 as Weight))
			.saturating_add(T::DbWeight::get().writes(9 as Weight))
			.saturating_add(T::DbWeight::get().writes((2 as Weight)))
	}
	fn sign_asset() -> Weight {
		(47_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}
