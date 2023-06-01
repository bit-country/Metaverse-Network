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
		Weight::from_parts(15_762_000, 0)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	fn create_class() -> Weight {
		Weight::from_parts(32_238_000, 0)
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	fn mint() -> Weight {
		Weight::from_parts(49_566_000, 0)
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(10 as u64))
	}
	fn mint_stackable_nft() -> Weight {
		Weight::from_parts(46_560_000, 0)
			.saturating_add(T::DbWeight::get().reads(6 as u64))
			.saturating_add(T::DbWeight::get().writes(8 as u64))
	}
	fn transfer() -> Weight {
		Weight::from_parts(30_588_000, 0)
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	fn transfer_stackable_nft() -> Weight {
		Weight::from_parts(23_361_000, 0)
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	fn transfer_batch() -> Weight {
		Weight::from_parts(49_633_000, 0)
			.saturating_add(T::DbWeight::get().reads(8 as u64))
			.saturating_add(T::DbWeight::get().writes(6 as u64))
	}
	fn sign_asset() -> Weight {
		Weight::from_parts(34_549_000, 0)
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	fn set_hard_limit() -> Weight {
		Weight::from_parts(14_025_000, 0)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	fn withdraw_funds_from_class_fund() -> Weight {
		Weight::from_parts(27_002_000, 0)
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	fn force_update_total_issuance() -> Weight {
		Weight::from_parts(12_330_000, 0)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
}
