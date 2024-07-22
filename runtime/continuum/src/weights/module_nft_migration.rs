#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for module_nft_migration.
pub struct WeightInfo<T>(PhantomData<T>);

impl<T: frame_system::Config> nft_migration::WeightInfo for WeightInfo<T> {
	fn start_migration() -> Weight {
		Weight::from_parts(14_373_000, 1491)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
