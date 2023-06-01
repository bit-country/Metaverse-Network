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
		Weight::from_parts(41_707_000, 0)
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(7 as u64))
	}
	fn create_nft_campaign() -> Weight {
		Weight::from_parts(43_668_000, 0)
			.saturating_add(T::DbWeight::get().reads(6 as u64))
			.saturating_add(T::DbWeight::get().writes(8 as u64))
	}
	fn claim_reward() -> Weight {
		Weight::from_parts(35_006_000, 0)
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	fn claim_reward_root() -> Weight {
		Weight::from_parts(42_272_000, 0)
			.saturating_add(T::DbWeight::get().reads(6 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	fn claim_nft_reward() -> Weight {
		Weight::from_parts(39_669_000, 0)
			.saturating_add(T::DbWeight::get().reads(7 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	fn claim_nft_reward_root() -> Weight {
		Weight::from_parts(44_661_000, 0)
			.saturating_add(T::DbWeight::get().reads(8 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	fn set_reward() -> Weight {
		Weight::from_parts(21_354_000, 0)
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	fn set_reward_root() -> Weight {
		Weight::from_parts(22_813_000, 0)
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	fn set_nft_reward() -> Weight {
		Weight::from_parts(22_278_000, 0)
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	fn set_nft_reward_root() -> Weight {
		Weight::from_parts(22_938_000, 0)
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	fn close_campaign() -> Weight {
		Weight::from_parts(50_279_000, 0)
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(7 as u64))
	}
	fn close_nft_campaign() -> Weight {
		Weight::from_parts(47_338_000, 0)
			.saturating_add(T::DbWeight::get().reads(6 as u64))
			.saturating_add(T::DbWeight::get().writes(8 as u64))
	}
	fn cancel_campaign() -> Weight {
		Weight::from_parts(38_746_000, 0)
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	fn cancel_nft_campaign() -> Weight {
		Weight::from_parts(31_672_000, 0)
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	fn add_set_reward_origin() -> Weight {
		Weight::from_parts(11_374_000, 0)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	fn remove_set_reward_origin() -> Weight {
		Weight::from_parts(11_973_000, 0)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	fn on_finalize() -> Weight {
		Weight::from_parts(13_598_000, 0).saturating_add(T::DbWeight::get().reads(2 as u64))
	}
}
