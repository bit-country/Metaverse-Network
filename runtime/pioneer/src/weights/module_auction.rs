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
		Weight::from_parts(61_964_000, 54013)
			.saturating_add(T::DbWeight::get().reads(9))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	fn create_new_buy_now() -> Weight {
		Weight::from_parts(60_960_000, 54013)
			.saturating_add(T::DbWeight::get().reads(9))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	fn bid() -> Weight {
		Weight::from_parts(57_509_000, 17254)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	fn buy_now() -> Weight {
		Weight::from_parts(119_730_000, 74974)
			.saturating_add(T::DbWeight::get().reads(13))
			.saturating_add(T::DbWeight::get().writes(12))
	}
	fn cancel_listing() -> Weight {
		Weight::from_parts(48_570_000, 28348)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	fn authorise_metaverse_collection() -> Weight {
		Weight::from_parts(22_121_000, 7564)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn remove_authorise_metaverse_collection() -> Weight {
		Weight::from_parts(23_031_000, 7670)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn make_offer() -> Weight {
		Weight::from_parts(36_729_000, 15443)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn withdraw_offer() -> Weight {
		Weight::from_parts(29_063_000, 6612)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn accept_offer() -> Weight {
		Weight::from_parts(68_958_000, 31537)
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	fn on_finalize() -> Weight {
		Weight::from_parts(5_727_000, 0)
	}
}
