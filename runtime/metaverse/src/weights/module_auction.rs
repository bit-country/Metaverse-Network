#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for module_auction.
pub struct WeightInfo<T>(PhantomData<T>);

impl<T: frame_system::Config> auction::WeightInfo for WeightInfo<T> {
	fn create_new_auction() -> Weight {
		Weight::from_parts(50_000_000, 0)
			.saturating_add(T::DbWeight::get().reads(10 as u64))
			.saturating_add(T::DbWeight::get().writes(8 as u64))
	}
	fn create_new_buy_now() -> Weight {
		Weight::from_parts(49_000_000, 0)
			.saturating_add(T::DbWeight::get().reads(10 as u64))
			.saturating_add(T::DbWeight::get().writes(8 as u64))
	}
	fn bid() -> Weight {
		Weight::from_parts(38_000_000, 0)
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(6 as u64))
	}
	fn buy_now() -> Weight {
		Weight::from_parts(100_000_000, 0)
			.saturating_add(T::DbWeight::get().reads(13 as u64))
			.saturating_add(T::DbWeight::get().writes(13 as u64))
	}
	fn cancel_listing() -> Weight {
		Weight::from_parts(38_000_000, 0)
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(7 as u64))
	}
	fn authorise_metaverse_collection() -> Weight {
		Weight::from_parts(16_000_000, 0)
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	fn remove_authorise_metaverse_collection() -> Weight {
		Weight::from_parts(16_000_000, 0)
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	fn make_offer() -> Weight {
		Weight::from_parts(30_000_000, 0)
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	fn withdraw_offer() -> Weight {
		Weight::from_parts(23_000_000, 0)
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	fn accept_offer() -> Weight {
		Weight::from_parts(55_000_000, 0)
			.saturating_add(T::DbWeight::get().reads(7 as u64))
			.saturating_add(T::DbWeight::get().writes(7 as u64))
	}
	fn on_finalize() -> Weight {
		Weight::from_parts(2_000_000, 0)
	}
}
