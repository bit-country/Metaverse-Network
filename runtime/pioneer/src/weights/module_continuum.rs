#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for module_continuum.
pub struct WeightInfo<T>(PhantomData<T>);

impl<T: frame_system::Config> continuum::WeightInfo for WeightInfo<T> {
	fn set_allow_buy_now() -> Weight {
		Weight::from_parts(3_836_000, 646).saturating_add(T::DbWeight::get().writes(1))
	}
	fn set_max_bounds() -> Weight {
		Weight::from_parts(10_396_000, 919).saturating_add(T::DbWeight::get().writes(1))
	}
	fn issue_map_slot() -> Weight {
		Weight::from_parts(15_692_000, 5088)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn create_new_auction() -> Weight {
		Weight::from_parts(36_453_000, 20225)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	fn buy_map_spot() -> Weight {
		Weight::from_parts(172_450_000, 59161)
			.saturating_add(T::DbWeight::get().reads(11))
			.saturating_add(T::DbWeight::get().writes(8))
	}
	fn bid_map_spot() -> Weight {
		Weight::from_parts(67_164_000, 40870)
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(3))
	}
}
