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
		(187_900_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(7 as Weight))
	}
	fn create_nft_campaign() -> Weight {
		(106_500_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(8 as Weight))
	}
	fn claim_reward() -> Weight {
		(82_600_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	fn claim_reward_root() -> Weight {
		(189_300_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	fn claim_nft_reward() -> Weight {
		(94_400_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(7 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	fn claim_nft_reward_root() -> Weight {
		(107_300_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(9 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn set_reward() -> Weight {
		(47_100_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn set_reward_root() -> Weight {
		(107_600_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
	}
	fn set_nft_reward() -> Weight {
		(61_500_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn set_nft_reward_root() -> Weight {
		(61_700_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
	}
	fn close_campaign() -> Weight {
		(123_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(7 as Weight))
	}
	fn close_nft_campaign() -> Weight {
		(115_900_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(9 as Weight))
	}
	fn cancel_campaign() -> Weight {
		(98_800_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn cancel_nft_campaign() -> Weight {
		(119_800_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	fn add_set_reward_origin() -> Weight {
		(24_200_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn remove_set_reward_origin() -> Weight {
		(47_600_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn on_finalize() -> Weight {
		(29_500_000 as Weight).saturating_add(T::DbWeight::get().reads(2 as Weight))
	}
}
