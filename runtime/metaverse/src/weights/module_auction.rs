// This default_weight is manually generated for UI integration testing purpose
// This bench_marking cli need to run to complete bench marking for all functions

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for module_auction.
pub struct WeightInfo<T>(PhantomData<T>);

impl<T: frame_system::Config> auction::WeightInfo for WeightInfo<T> {
	fn create_new_auction() -> Weight {
		Weight::from_parts(83_359_000, 54013)
			.saturating_add(T::DbWeight::get().reads(9))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	fn create_new_buy_now() -> Weight {
		Weight::from_parts(60_938_000, 54013)
			.saturating_add(T::DbWeight::get().reads(9))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	fn bid() -> Weight {
		Weight::from_parts(52_851_000, 17254)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	fn buy_now() -> Weight {
		Weight::from_parts(117_267_000, 74974)
			.saturating_add(T::DbWeight::get().reads(13))
			.saturating_add(T::DbWeight::get().writes(12))
	}
	fn cancel_listing() -> Weight {
		Weight::from_parts(76_276_000, 28348)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	fn authorise_metaverse_collection() -> Weight {
		Weight::from_parts(35_078_000, 7564)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn remove_authorise_metaverse_collection() -> Weight {
		Weight::from_parts(24_963_000, 7670)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn make_offer() -> Weight {
		Weight::from_parts(38_021_000, 15443)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn withdraw_offer() -> Weight {
		Weight::from_parts(29_056_000, 6612)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn accept_offer() -> Weight {
		Weight::from_parts(69_244_000, 31537)
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	fn on_finalize() -> Weight {
		Weight::from_parts(6_116_000, 0)
	}
}
