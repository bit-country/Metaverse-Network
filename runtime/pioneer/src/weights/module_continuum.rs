#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for module_continuum.
pub struct WeightInfo<T>(PhantomData<T>);

impl<T: frame_system::Config> continuum::WeightInfo for WeightInfo<T> {
	fn set_allow_buy_now() -> Weight {
		Weight::from_parts(6_650_000, 646).saturating_add(T::DbWeight::get().writes(1))
	}
	fn set_max_bounds() -> Weight {
		Weight::from_parts(25_462_000, 919).saturating_add(T::DbWeight::get().writes(1))
	}
	fn issue_map_slot() -> Weight {
		Weight::from_parts(24_533_000, 5088)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn create_new_auction() -> Weight {
		Weight::from_parts(85_653_000, 20225)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	fn buy_map_spot() -> Weight {
		Weight::from_parts(170_947_000, 59161)
			.saturating_add(T::DbWeight::get().reads(11))
			.saturating_add(T::DbWeight::get().writes(8))
	}
	fn bid_map_spot() -> Weight {
		Weight::from_parts(120_875_000, 44418)
			.saturating_add(T::DbWeight::get().reads(9))
			.saturating_add(T::DbWeight::get().writes(4))
	}
}
