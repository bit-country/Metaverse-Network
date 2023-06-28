#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for module_currencies.
pub struct WeightInfo<T>(PhantomData<T>);

impl<T: frame_system::Config> currencies::WeightInfo for WeightInfo<T> {
	fn transfer() -> Weight {
		Weight::from_parts(26_192_000, 5206)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn transfer_native_currency() -> Weight {
		Weight::from_parts(24_570_000, 5206)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn update_balance() -> Weight {
		Weight::from_parts(16_394_000, 2603)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
