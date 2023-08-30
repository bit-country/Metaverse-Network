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
		Weight::from_parts(47_856_000, 12368)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	fn create_nft_campaign() -> Weight {
		Weight::from_parts(51_776_000, 26330)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	fn claim_reward() -> Weight {
		Weight::from_parts(43_780_000, 13249)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn claim_reward_root() -> Weight {
		Weight::from_parts(50_524_000, 24159)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn claim_nft_reward() -> Weight {
		Weight::from_parts(52_709_000, 33438)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn claim_nft_reward_root() -> Weight {
		Weight::from_parts(118_897_000, 41895)
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn set_reward() -> Weight {
		Weight::from_parts(30_996_000, 13950)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn set_reward_root() -> Weight {
		Weight::from_parts(31_147_000, 18600)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn set_nft_reward() -> Weight {
		Weight::from_parts(32_853_000, 14163)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn set_nft_reward_root() -> Weight {
		Weight::from_parts(32_100_000, 17525)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn close_campaign() -> Weight {
		Weight::from_parts(67_727_000, 21663)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	fn close_nft_campaign() -> Weight {
		Weight::from_parts(61_669_000, 30653)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	fn cancel_campaign() -> Weight {
		Weight::from_parts(46_849_000, 7440)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn cancel_nft_campaign() -> Weight {
		Weight::from_parts(42_579_000, 13131)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	fn add_set_reward_origin() -> Weight {
		Weight::from_parts(15_236_000, 3436)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn remove_set_reward_origin() -> Weight {
		Weight::from_parts(15_816_000, 3510)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn on_finalize() -> Weight {
		Weight::from_parts(19_256_000, 6753).saturating_add(T::DbWeight::get().reads(2))
	}
}
