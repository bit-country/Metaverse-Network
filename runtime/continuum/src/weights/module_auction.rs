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
		Weight::from_parts(61_885_000, 57373)
			.saturating_add(T::DbWeight::get().reads(9))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	fn create_new_buy_now() -> Weight {
		Weight::from_parts(74_722_000, 61706)
			.saturating_add(T::DbWeight::get().reads(10))
			.saturating_add(T::DbWeight::get().writes(8))
	}
	fn bid() -> Weight {
		Weight::from_parts(75_918_000, 25151)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	fn buy_now() -> Weight {
		Weight::from_parts(121_030_000, 78754)
			.saturating_add(T::DbWeight::get().reads(13))
			.saturating_add(T::DbWeight::get().writes(12))
	}
	fn cancel_listing() -> Weight {
		Weight::from_parts(50_823_000, 30028)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	fn authorise_metaverse_collection() -> Weight {
		Weight::from_parts(23_046_000, 7564)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn remove_authorise_metaverse_collection() -> Weight {
		Weight::from_parts(23_607_000, 7670)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn make_offer() -> Weight {
		Weight::from_parts(38_079_000, 15443)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn withdraw_offer() -> Weight {
		Weight::from_parts(28_621_000, 6612)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn accept_offer() -> Weight {
		Weight::from_parts(72_454_000, 31537)
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	fn on_finalize() -> Weight {
		Weight::from_parts(7_467_000, 0)
	}
}
