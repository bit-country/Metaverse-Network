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
		Weight::from_parts(56_713_000, 12368)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	fn create_nft_campaign() -> Weight {
		Weight::from_parts(58_468_000, 26918)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	fn claim_reward() -> Weight {
		Weight::from_parts(42_927_000, 13585)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn claim_reward_root() -> Weight {
		Weight::from_parts(51_031_000, 24831)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn claim_nft_reward() -> Weight {
		Weight::from_parts(53_120_000, 34950)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn claim_nft_reward_root() -> Weight {
		Weight::from_parts(79_172_000, 44835)
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn set_reward() -> Weight {
		Weight::from_parts(40_981_000, 14454)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn set_reward_root() -> Weight {
		Weight::from_parts(42_695_000, 19272)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn set_nft_reward() -> Weight {
		Weight::from_parts(44_297_000, 14919)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn set_nft_reward_root() -> Weight {
		Weight::from_parts(51_800_000, 23580)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	fn close_campaign() -> Weight {
		Weight::from_parts(68_700_000, 22503)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	fn close_nft_campaign() -> Weight {
		Weight::from_parts(61_489_000, 32753)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	fn cancel_campaign() -> Weight {
		Weight::from_parts(45_875_000, 7524)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn cancel_nft_campaign() -> Weight {
		Weight::from_parts(41_755_000, 13467)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn add_set_reward_origin() -> Weight {
		Weight::from_parts(29_035_000, 6566)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn remove_set_reward_origin() -> Weight {
		Weight::from_parts(21_305_000, 3510)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn on_finalize() -> Weight {
		Weight::from_parts(35_303_000, 6837).saturating_add(T::DbWeight::get().reads(2))
	}
}
