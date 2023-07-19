#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for module_mining.
pub struct WeightInfo<T>(PhantomData<T>);

impl<T: frame_system::Config> mining::WeightInfo for WeightInfo<T> {
	fn add_minting_origin() -> Weight {
		Weight::from_parts(47_273_000, 8027)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn remove_minting_origin() -> Weight {
		Weight::from_parts(22_628_000, 8101)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn update_round_length() -> Weight {
		Weight::from_parts(7_119_000, 571)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn update_mining_issuance_config() -> Weight {
		Weight::from_parts(7_988_000, 647)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn mint() -> Weight {
		Weight::from_parts(23_623_000, 8043)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn burn() -> Weight {
		Weight::from_parts(24_990_000, 8043)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn deposit() -> Weight {
		Weight::from_parts(32_866_000, 10398)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn withdraw() -> Weight {
		Weight::from_parts(31_275_000, 10997)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn pause_mining_round() -> Weight {
		Weight::from_parts(20_520_000, 1142)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn unpause_mining_round() -> Weight {
		Weight::from_parts(22_937_000, 1190)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
