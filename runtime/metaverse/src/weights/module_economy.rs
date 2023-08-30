// This default_weight is manually generated for UI integration testing purpose
// This bench_marking cli need to run to complete bench marking for all functions

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for module_economy.
pub struct WeightInfo<T>(PhantomData<T>);

impl<T: frame_system::Config> economy::WeightInfo for WeightInfo<T> {
	// Storage: Economy BitPowerExchangeRate (r:0 w:1)
	// Proof Skipped: Economy BitPowerExchangeRate (max_values: Some(1), max_size: None, mode: Measured)
	fn set_bit_power_exchange_rate() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `919`
		//  Estimated: `919`
		// Minimum execution time: 11_685 nanoseconds.
		Weight::from_parts(12_108_000, 919).saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Economy PowerBalance (r:0 w:1)
	// Proof Skipped: Economy PowerBalance (max_values: None, max_size: None, mode: Measured)
	fn set_power_balance() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `919`
		//  Estimated: `919`
		// Minimum execution time: 12_284 nanoseconds.
		Weight::from_parts(12_785_000, 919).saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Mining Round (r:1 w:0)
	// Proof Skipped: Mining Round (max_values: Some(1), max_size: None, mode: Measured)
	// Storage: Economy ExitQueue (r:1 w:0)
	// Proof Skipped: Economy ExitQueue (max_values: None, max_size: None, mode: Measured)
	// Storage: Economy StakingInfo (r:1 w:1)
	// Proof Skipped: Economy StakingInfo (max_values: None, max_size: None, mode: Measured)
	// Storage: Economy TotalStake (r:1 w:1)
	// Proof Skipped: Economy TotalStake (max_values: Some(1), max_size: None, mode: Measured)
	fn stake_a() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1496`
		//  Estimated: `11924`
		// Minimum execution time: 29_836 nanoseconds.
		Weight::from_parts(30_929_000, 11924)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Mining Round (r:1 w:0)
	// Proof Skipped: Mining Round (max_values: Some(1), max_size: None, mode: Measured)
	// Storage: Economy EstateExitQueue (r:1 w:0)
	// Proof Skipped: Economy EstateExitQueue (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate Estates (r:1 w:0)
	// Proof Skipped: Estate Estates (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate EstateOwner (r:1 w:0)
	// Proof Skipped: Estate EstateOwner (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Tokens (r:1 w:0)
	// Proof Skipped: OrmlNFT Tokens (max_values: None, max_size: None, mode: Measured)
	// Storage: Economy EstateStakingInfo (r:1 w:1)
	// Proof Skipped: Economy EstateStakingInfo (max_values: None, max_size: None, mode: Measured)
	// Storage: Economy TotalEstateStake (r:1 w:1)
	// Proof Skipped: Economy TotalEstateStake (max_values: Some(1), max_size: None, mode: Measured)
	fn stake_b() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2144`
		//  Estimated: `28373`
		// Minimum execution time: 45_093 nanoseconds.
		Weight::from_parts(48_615_000, 28373)
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: Economy StakingInfo (r:1 w:1)
	// Proof Skipped: Economy StakingInfo (max_values: None, max_size: None, mode: Measured)
	// Storage: Mining Round (r:1 w:0)
	// Proof Skipped: Mining Round (max_values: Some(1), max_size: None, mode: Measured)
	// Storage: Economy ExitQueue (r:1 w:1)
	// Proof Skipped: Economy ExitQueue (max_values: None, max_size: None, mode: Measured)
	// Storage: Economy TotalStake (r:1 w:1)
	// Proof Skipped: Economy TotalStake (max_values: Some(1), max_size: None, mode: Measured)
	fn unstake_a() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1233`
		//  Estimated: `10872`
		// Minimum execution time: 22_126 nanoseconds.
		Weight::from_parts(23_896_000, 10872)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: Estate Estates (r:1 w:0)
	// Proof Skipped: Estate Estates (max_values: None, max_size: None, mode: Measured)
	// Storage: Economy EstateStakingInfo (r:1 w:1)
	// Proof Skipped: Economy EstateStakingInfo (max_values: None, max_size: None, mode: Measured)
	// Storage: Mining Round (r:1 w:0)
	// Proof Skipped: Mining Round (max_values: Some(1), max_size: None, mode: Measured)
	// Storage: Economy EstateExitQueue (r:1 w:1)
	// Proof Skipped: Economy EstateExitQueue (max_values: None, max_size: None, mode: Measured)
	// Storage: Economy TotalEstateStake (r:1 w:1)
	// Proof Skipped: Economy TotalEstateStake (max_values: Some(1), max_size: None, mode: Measured)
	fn unstake_b() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1489`
		//  Estimated: `15860`
		// Minimum execution time: 31_005 nanoseconds.
		Weight::from_parts(32_916_000, 15860)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: Estate Estates (r:1 w:0)
	// Proof Skipped: Estate Estates (max_values: None, max_size: None, mode: Measured)
	// Storage: Estate EstateOwner (r:1 w:0)
	// Proof Skipped: Estate EstateOwner (max_values: None, max_size: None, mode: Measured)
	// Storage: OrmlNFT Tokens (r:1 w:0)
	// Proof Skipped: OrmlNFT Tokens (max_values: None, max_size: None, mode: Measured)
	// Storage: Economy EstateStakingInfo (r:1 w:1)
	// Proof Skipped: Economy EstateStakingInfo (max_values: None, max_size: None, mode: Measured)
	// Storage: Mining Round (r:1 w:0)
	// Proof Skipped: Mining Round (max_values: Some(1), max_size: None, mode: Measured)
	// Storage: Economy TotalEstateStake (r:1 w:1)
	// Proof Skipped: Economy TotalEstateStake (max_values: Some(1), max_size: None, mode: Measured)
	// Storage: Economy EstateExitQueue (r:0 w:1)
	// Proof Skipped: Economy EstateExitQueue (max_values: None, max_size: None, mode: Measured)
	fn unstake_new_estate_owner() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1914`
		//  Estimated: `24288`
		// Minimum execution time: 36_727 nanoseconds.
		Weight::from_parts(38_216_000, 24288)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: Economy ExitQueue (r:1 w:1)
	// Proof Skipped: Economy ExitQueue (max_values: None, max_size: None, mode: Measured)
	fn withdraw_unreserved() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1568`
		//  Estimated: `4043`
		// Minimum execution time: 24_239 nanoseconds.
		Weight::from_parts(25_134_000, 4043)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
