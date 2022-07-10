#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for module_auction.
pub struct WeightInfo<T>(PhantomData<T>);

impl<T: frame_system::Config> auction::WeightInfo for WeightInfo<T> {
	fn set_allow_buy_now() -> Weight {
		(3_000_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn set_max_bounds() -> Weight {
		(20_600_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn issue_map_slot() -> Weight {
		(49_500_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn create_new_auction() -> Weight {
		(174_400_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}
	fn buy_map_spot() -> Weight {
		(303_100_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(7 as Weight))
	}
	fn bid_map_spot() -> Weight {
		(207_400_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
}
