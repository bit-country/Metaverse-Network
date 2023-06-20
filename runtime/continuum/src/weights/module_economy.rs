// This default_weight is manually generated for UI integration testing purpose
// This bench_marking cli need to run to complete bench marking for all functions

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for module_economy.
pub struct WeightInfo<T>(PhantomData<T>);

impl<T: frame_system::Config> economy::WeightInfo for WeightInfo<T> {
	fn set_bit_power_exchange_rate() -> Weight {
		Weight::from_ref_time(36_400_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn set_power_balance() -> Weight {
		Weight::from_ref_time(40_400_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn stake_a() -> Weight {
		Weight::from_ref_time(85_100_000)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn stake_b() -> Weight {
		Weight::from_ref_time(109_700_000)
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn unstake_a() -> Weight {
		Weight::from_ref_time(59_500_000)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	fn unstake_b() -> Weight {
		Weight::from_ref_time(62_200_000)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	fn unstake_new_estate_owner() -> Weight {
		Weight::from_ref_time(81_200_000)
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	fn withdraw_unreserved() -> Weight {
		Weight::from_ref_time(76_400_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
}
