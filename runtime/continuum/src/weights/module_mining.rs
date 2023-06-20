#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for module_mining.
pub struct WeightInfo<T>(PhantomData<T>);

impl<T: frame_system::Config> mining::WeightInfo for WeightInfo<T> {
	fn add_minting_origin() -> Weight {
		Weight::from_ref_time(42_200_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn remove_minting_origin() -> Weight {
		Weight::from_ref_time(25_500_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn update_round_length() -> Weight {
		Weight::from_ref_time(20_600_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn update_mining_issuance_config() -> Weight {
		Weight::from_ref_time(60_900_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn mint() -> Weight {
		Weight::from_ref_time(84_400_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn burn() -> Weight {
		Weight::from_ref_time(128_900_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn deposit() -> Weight {
		Weight::from_ref_time(163_900_000)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn withdraw() -> Weight {
		Weight::from_ref_time(52_100_000)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn pause_mining_round() -> Weight {
		Weight::from_ref_time(19_200_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn unpause_mining_round() -> Weight {
		Weight::from_ref_time(19_800_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
