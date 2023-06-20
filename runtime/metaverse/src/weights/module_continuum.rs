#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for module_continuum.
pub struct WeightInfo<T>(PhantomData<T>);

impl<T: frame_system::Config> continuum::WeightInfo for WeightInfo<T> {
	fn set_allow_buy_now() -> Weight {
		Weight::from_ref_time(3_000_000).saturating_add(T::DbWeight::get().writes(1))
	}
	fn set_max_bounds() -> Weight {
		Weight::from_ref_time(20_600_000).saturating_add(T::DbWeight::get().writes(1))
	}
	fn issue_map_slot() -> Weight {
		Weight::from_ref_time(49_500_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn create_new_auction() -> Weight {
		Weight::from_ref_time(174_400_000)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	fn buy_map_spot() -> Weight {
		Weight::from_ref_time(303_100_000)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	fn bid_map_spot() -> Weight {
		Weight::from_ref_time(207_400_000)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(2))
	}
}
