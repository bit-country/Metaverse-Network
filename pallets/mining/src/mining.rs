// This file is part of Bit.Country.

// Copyright (C) 2020-2021 Bit.Country.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::traits::Zero;
use sp_runtime::{Perbill, RuntimeDebug};

use core_primitives::{MiningRange, MiningResourceRateInfo};
use primitives::estate::Estate;

// Helper methods to compute the issuance rate for undeployed land.
use crate::pallet::{Config, Pallet};

const SECONDS_PER_YEAR: u32 = 31557600;
const SECONDS_PER_BLOCK: u32 = 12;
const BLOCKS_PER_YEAR: u32 = SECONDS_PER_YEAR / SECONDS_PER_BLOCK;

fn rounds_per_year<T: Config>() -> u32 {
	let blocks_per_round = <Pallet<T>>::round().length;
	BLOCKS_PER_YEAR / blocks_per_round
}

/// Compute round issuance range from round inflation range and current total issuance
pub fn round_issuance_range<T: Config>(config: MiningResourceRateInfo) -> MiningRange<u64> {
	// Get total round per year
	// Initial minting ratio per land unit
	let minting_ratio = config.ratio;
	// Get total deployed land unit circulating
	let total_land_unit_circulating = T::EstateHandler::get_total_land_units();

	let issuance_per_round = total_land_unit_circulating
		.checked_mul(minting_ratio)
		.unwrap_or(Zero::zero());

	let staking_allocation = issuance_per_round
		.checked_mul(config.staking_reward.into())
		.unwrap_or(issuance_per_round)
		.checked_div(10000u64)
		.unwrap();

	let mining_allocation = issuance_per_round
		.checked_mul(config.mining_reward.into())
		.unwrap_or(issuance_per_round)
		.checked_div(10000u64)
		.unwrap();

	// Return range - could implement more cases in the future.
	MiningRange {
		min: issuance_per_round,
		ideal: issuance_per_round,
		max: issuance_per_round,
		staking_allocation,
		mining_allocation,
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Compute round issuance range from round inflation range and current total issuance
	pub fn mock_round_issuance_per_year(
		config: MiningResourceRateInfo,
		land_unit_circulation: u64,
	) -> MiningRange<u64> {
		let issuance_per_round = land_unit_circulation.checked_mul(config.ratio).unwrap_or(Zero::zero());

		let staking_allocation = issuance_per_round
			.checked_mul(config.staking_reward.into())
			.unwrap_or(issuance_per_round)
			.checked_div(10000u64)
			.unwrap();

		let mining_allocation = issuance_per_round
			.checked_mul(config.mining_reward.into())
			.unwrap_or(issuance_per_round)
			.checked_div(10000u64)
			.unwrap();

		// Return range - could implement more cases in the future.
		MiningRange {
			min: issuance_per_round,
			ideal: issuance_per_round,
			max: issuance_per_round,
			staking_allocation,
			mining_allocation,
		}
	}

	#[test]
	fn simple_round_issuance() {
		// 10 BIT/Land unit minting ratio for 2_000 land unit = 2_000_000 minted over the year
		// let's assume there are 10 periods in a year
		// => mint 2_000_000 over 10 periods => 20_000 minted per period

		let mock_config: MiningResourceRateInfo = MiningResourceRateInfo {
			ratio: 10,
			staking_reward: 2000,
			mining_reward: 8000,
		};

		let round_issuance = mock_round_issuance_per_year(mock_config, 2_000);

		// make sure 20_000 land unit deploy per period
		assert_eq!(round_issuance.min, 20_000);
		assert_eq!(round_issuance.ideal, 20_000);
		assert_eq!(round_issuance.max, 20_000);
		assert_eq!(round_issuance.staking_allocation, 4_000);
		assert_eq!(round_issuance.mining_allocation, 16_000);
	}
}
