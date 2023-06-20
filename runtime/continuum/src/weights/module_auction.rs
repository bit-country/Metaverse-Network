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
		Weight::from_ref_time(50_000_000)
			.saturating_add(T::DbWeight::get().reads(10))
			.saturating_add(T::DbWeight::get().writes(8))
	}
	fn create_new_buy_now() -> Weight {
		Weight::from_ref_time(49_000_000)
			.saturating_add(T::DbWeight::get().reads(10))
			.saturating_add(T::DbWeight::get().writes(8))
	}
	fn bid() -> Weight {
		Weight::from_ref_time(38_000_000)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	fn buy_now() -> Weight {
		Weight::from_ref_time(100_000_000)
			.saturating_add(T::DbWeight::get().reads(13))
			.saturating_add(T::DbWeight::get().writes(13))
	}
	fn cancel_listing() -> Weight {
		Weight::from_ref_time(38_000_000)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	fn authorise_metaverse_collection() -> Weight {
		Weight::from_ref_time(16_000_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn remove_authorise_metaverse_collection() -> Weight {
		Weight::from_ref_time(16_000_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn make_offer() -> Weight {
		Weight::from_ref_time(30_000_000)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn withdraw_offer() -> Weight {
		Weight::from_ref_time(23_000_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn accept_offer() -> Weight {
		Weight::from_ref_time(55_000_000)
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	fn on_finalize() -> Weight {
		Weight::from_ref_time(2_000_000)
	}
}
