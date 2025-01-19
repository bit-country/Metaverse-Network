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
		Weight::from_parts(12_117_000, 1317)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn create_class() -> Weight {
		Weight::from_parts(37_465_000, 7417)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	fn mint() -> Weight {
		Weight::from_parts(69_706_000, 23976)
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(10))
	}
	fn mint_stackable_nft() -> Weight {
		Weight::from_parts(111_946_000, 15931)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(8))
	}
	fn transfer() -> Weight {
		Weight::from_parts(33_226_000, 17187)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn transfer_stackable_nft() -> Weight {
		Weight::from_parts(28_946_000, 11790)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn transfer_batch() -> Weight {
		Weight::from_parts(63_183_000, 25680)
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	fn sign_asset() -> Weight {
		Weight::from_parts(115_994_000, 14763)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	fn set_hard_limit() -> Weight {
		Weight::from_parts(23_425_000, 2757)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn withdraw_funds_from_class_fund() -> Weight {
		Weight::from_parts(61_491_000, 8268)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn force_update_total_issuance() -> Weight {
		Weight::from_parts(27_207_000, 2757)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn mint_pre_signed() -> Weight {
		Weight::from_parts(91_000_000, 19036)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(6))
	}
}
