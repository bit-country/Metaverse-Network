#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for module_auction.
pub struct WeightInfo<T>(PhantomData<T>);

impl<T: frame_system::Config> auction::WeightInfo for WeightInfo<T> {
	fn create_new_auction() -> Weight {
		(50_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(10 as Weight))
			.saturating_add(T::DbWeight::get().writes(8 as Weight))
	}
	fn create_new_buy_now() -> Weight {
		(49_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(10 as Weight))
			.saturating_add(T::DbWeight::get().writes(8 as Weight))
	}
	fn bid() -> Weight {
		(38_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}
	fn buy_now() -> Weight {
		(100_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(13 as Weight))
			.saturating_add(T::DbWeight::get().writes(13 as Weight))
	}
	fn cancel_listing() -> Weight {
		(38_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(7 as Weight))
	}
	fn authorise_metaverse_collection() -> Weight {
		(16_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn remove_authorise_metaverse_collection() -> Weight {
		(16_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn make_offer() -> Weight {
		(30_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn withdraw_offer() -> Weight {
		(23_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn accept_offer() -> Weight {
		(55_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(7 as Weight))
			.saturating_add(T::DbWeight::get().writes(7 as Weight))
	}
	fn on_finalize() -> Weight {
		(2_000_000 as Weight)
	}
}
