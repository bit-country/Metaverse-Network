// This default_weight is manually generated for UI integration testing purpose
// This bench_marking cli need to run to complete bench marking for all functions

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for module_economy.
pub struct WeightInfo<T>(PhantomData<T>);

impl<T: frame_system::Config> economy::WeightInfo for WeightInfo<T> {
	fn stake_a() -> Weight {
		Weight::from_parts(52_209_000, 4929)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn stake_b() -> Weight {
		Weight::from_parts(71_491_000, 5545)
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn stake_on_innovation() -> Weight {
		Weight::from_parts(56_832_000, 4929)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	fn unstake_a() -> Weight {
		Weight::from_parts(32_069_000, 4698)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn unstake_b() -> Weight {
		Weight::from_parts(47_341_000, 4921)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn unstake_new_estate_owner() -> Weight {
		Weight::from_parts(53_229_000, 5314)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn unstake_on_innovation() -> Weight {
		Weight::from_parts(46_719_000, 4811)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	fn withdraw_unreserved() -> Weight {
		Weight::from_parts(58_043_000, 5001)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
