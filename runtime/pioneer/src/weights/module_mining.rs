#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for module_mining.
pub struct WeightInfo<T>(PhantomData<T>);

impl<T: frame_system::Config> mining::WeightInfo for WeightInfo<T> {
	fn add_minting_origin() -> Weight {
		Weight::from_parts(8_665_000, 2551)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn remove_minting_origin() -> Weight {
		Weight::from_parts(9_396_000, 2625)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn update_round_length() -> Weight {
		Weight::from_parts(7_040_000, 571)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn update_mining_issuance_config() -> Weight {
		Weight::from_parts(8_115_000, 647)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn mint() -> Weight {
		Weight::from_parts(22_826_000, 8043)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn burn() -> Weight {
		Weight::from_parts(24_969_000, 8043)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn deposit() -> Weight {
		Weight::from_parts(31_798_000, 10398)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn withdraw() -> Weight {
		Weight::from_parts(29_933_000, 10997)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn pause_mining_round() -> Weight {
		Weight::from_parts(8_074_000, 1142)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn unpause_mining_round() -> Weight {
		Weight::from_parts(8_775_000, 1190)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
