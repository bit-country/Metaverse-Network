#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for module_currencies.
pub struct WeightInfo<T>(PhantomData<T>);

impl<T: frame_system::Config> currencies::WeightInfo for WeightInfo<T> {
	fn transfer() -> Weight {
		Weight::from_ref_time(43_400_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn transfer_native_currency() -> Weight {
		Weight::from_ref_time(42_500_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn update_balance() -> Weight {
		Weight::from_ref_time(68_700_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
