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
		Weight::from_parts(36_400_000, 0)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	fn set_power_balance() -> Weight {
		Weight::from_parts(40_400_000, 0)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	fn stake_a() -> Weight {
		Weight::from_parts(85_100_000, 0)
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	fn stake_b() -> Weight {
		Weight::from_parts(109_700_000, 0)
			.saturating_add(T::DbWeight::get().reads(8 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	fn unstake_a() -> Weight {
		Weight::from_parts(59_500_000, 0)
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	fn unstake_b() -> Weight {
		Weight::from_parts(62_200_000, 0)
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	fn unstake_new_estate_owner() -> Weight {
		Weight::from_parts(81_200_000, 0)
			.saturating_add(T::DbWeight::get().reads(7 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	fn withdraw_unreserved() -> Weight {
		Weight::from_parts(76_400_000, 0)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
}
