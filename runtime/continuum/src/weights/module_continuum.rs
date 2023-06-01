#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for module_continuum.
pub struct WeightInfo<T>(PhantomData<T>);

impl<T: frame_system::Config> continuum::WeightInfo for WeightInfo<T> {
	fn set_allow_buy_now() -> Weight {
		Weight::from_parts(3_000_000, 0).saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	fn set_max_bounds() -> Weight {
		Weight::from_parts(20_600_000, 0).saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	fn issue_map_slot() -> Weight {
		Weight::from_parts(49_500_000, 0)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	fn create_new_auction() -> Weight {
		Weight::from_parts(174_400_000, 0)
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(6 as u64))
	}
	fn buy_map_spot() -> Weight {
		Weight::from_parts(303_100_000, 0)
			.saturating_add(T::DbWeight::get().reads(6 as u64))
			.saturating_add(T::DbWeight::get().writes(7 as u64))
	}
	fn bid_map_spot() -> Weight {
		Weight::from_parts(207_400_000, 0)
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
}
