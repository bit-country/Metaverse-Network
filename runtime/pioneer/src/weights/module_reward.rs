// This default_weight is manually generated for UI integration testing purpose
// This bench_marking cli need to run to complete bench marking for all functions

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for module_nft.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> reward::WeightInfo for WeightInfo<T> {
	fn create_campaign() -> Weight {
		Weight::from_ref_time(41_707_000)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	fn create_nft_campaign() -> Weight {
		Weight::from_ref_time(43_668_000)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(8))
	}
	fn claim_reward() -> Weight {
		Weight::from_ref_time(35_006_000)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	fn claim_reward_root() -> Weight {
		Weight::from_ref_time(42_272_000)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	fn claim_nft_reward() -> Weight {
		Weight::from_ref_time(39_669_000)
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	fn claim_nft_reward_root() -> Weight {
		Weight::from_ref_time(44_661_000)
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn set_reward() -> Weight {
		Weight::from_ref_time(21_354_000)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn set_reward_root() -> Weight {
		Weight::from_ref_time(22_813_000)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	fn set_nft_reward() -> Weight {
		Weight::from_ref_time(22_278_000)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn set_nft_reward_root() -> Weight {
		Weight::from_ref_time(22_938_000)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	fn close_campaign() -> Weight {
		Weight::from_ref_time(50_279_000)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	fn close_nft_campaign() -> Weight {
		Weight::from_ref_time(47_338_000)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(8))
	}
	fn cancel_campaign() -> Weight {
		Weight::from_ref_time(38_746_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn cancel_nft_campaign() -> Weight {
		Weight::from_ref_time(31_672_000)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	fn add_set_reward_origin() -> Weight {
		Weight::from_ref_time(11_374_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn remove_set_reward_origin() -> Weight {
		Weight::from_ref_time(11_973_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn on_finalize() -> Weight {
		Weight::from_ref_time(13_598_000).saturating_add(T::DbWeight::get().reads(2))
	}
}
