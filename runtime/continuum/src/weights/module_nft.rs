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
		Weight::from_ref_time(15_762_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn create_class() -> Weight {
		Weight::from_ref_time(32_238_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	fn mint() -> Weight {
		Weight::from_ref_time(49_566_000)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(10))
	}
	fn mint_stackable_nft() -> Weight {
		Weight::from_ref_time(46_560_000)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(8))
	}
	fn transfer() -> Weight {
		Weight::from_ref_time(30_588_000)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn transfer_stackable_nft() -> Weight {
		Weight::from_ref_time(23_361_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn transfer_batch() -> Weight {
		Weight::from_ref_time(49_633_000)
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	fn sign_asset() -> Weight {
		Weight::from_ref_time(34_549_000)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn set_hard_limit() -> Weight {
		Weight::from_ref_time(14_025_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn withdraw_funds_from_class_fund() -> Weight {
		Weight::from_ref_time(27_002_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn force_update_total_issuance() -> Weight {
		Weight::from_ref_time(12_330_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
