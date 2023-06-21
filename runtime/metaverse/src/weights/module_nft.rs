// This default_weight is manually generated for UI integration testing purpose
// This bench_marking cli need to run to complete bench marking for all functions

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for module_nft.
pub struct WeightInfo<T>(PhantomData<T>);

impl<T: frame_system::Config> nft::WeightInfo for WeightInfo<T> {
	fn create_group() -> Weight {
		Weight::from_parts(12_179_000, 1317)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn create_class() -> Weight {
		Weight::from_parts(27_797_000, 7417)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	fn mint() -> Weight {
		Weight::from_parts(54_185_000, 23976)
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(10))
	}
	fn mint_stackable_nft() -> Weight {
		Weight::from_parts(41_846_000, 15931)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(8))
	}
	fn transfer() -> Weight {
		Weight::from_parts(32_122_000, 17187)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn transfer_stackable_nft() -> Weight {
		Weight::from_parts(28_373_000, 11790)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn transfer_batch() -> Weight {
		Weight::from_parts(112_127_000, 25680)
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	fn sign_asset() -> Weight {
		Weight::from_parts(38_782_000, 11890)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn set_hard_limit() -> Weight {
		Weight::from_parts(15_065_000, 2757)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn withdraw_funds_from_class_fund() -> Weight {
		Weight::from_parts(25_576_000, 8268)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn force_update_total_issuance() -> Weight {
		Weight::from_parts(11_211_000, 2757)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
