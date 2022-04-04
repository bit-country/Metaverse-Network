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
use orml_traits::arithmetic::{CheckedDiv, CheckedMul};
use orml_traits::MultiCurrency;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::traits::{Saturating, Zero};
use sp_runtime::{ArithmeticError, Perbill, RuntimeDebug};

use core_primitives::{MiningRange, MiningResourceRateInfo};
use primitives::{estate::Estate, Balance, FungibleTokenId};

// Helper methods to compute the issuance rate for undeployed land.
use crate::pallet::{Config, Pallet};

const SECONDS_PER_YEAR: u32 = 31557600;
const SECONDS_PER_BLOCK: u32 = 12;
const BLOCKS_PER_YEAR: u32 = SECONDS_PER_YEAR / SECONDS_PER_BLOCK;

fn rounds_per_year<T: Config>() -> u32 {
	let blocks_per_round = <Pallet<T>>::round().length;
	BLOCKS_PER_YEAR / blocks_per_round
}

pub fn convert_annual_to_round<T: Config>(annual: Perbill) -> Perbill {
	let rounds = rounds_per_year::<T>();
	annual / rounds
}

/// Compute round issuance range from round inflation range and current total issuance
pub fn round_issuance_range<T: Config>(config: MiningResourceRateInfo) -> MiningRange<Balance> {
	// Get total round per year
	// Annual inflation rate
	let annual_rate = config.rate;
	// Get total deployed land unit circulating

	// Get total token supply
	let total_circulation_supply = T::MiningCurrency::total_issuance(FungibleTokenId::MiningResource(0));

	let rate_per_round = convert_annual_to_round::<T>(annual_rate);

	let issuance_per_round = rate_per_round * total_circulation_supply;

	let staking_allocation = config.staking_reward * issuance_per_round;

	let mining_allocation = config.mining_reward * issuance_per_round;

	// Return range - could implement more cases in the future.
	MiningRange {
		min: issuance_per_round.into(),
		ideal: issuance_per_round.into(),
		max: issuance_per_round.into(),
		staking_allocation: staking_allocation.into(),
		mining_allocation: mining_allocation.into(),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Compute round issuance range from round inflation range and current total issuance
	pub fn mock_round_issuance_per_year(
		config: MiningResourceRateInfo,
		mock_total_mining_resource_circulation: Balance,
	) -> MiningRange<Balance> {
		let issuance_per_round = mock_total_mining_resource_circulation
			.checked_mul(config.rate)
			.unwrap_or(Zero::zero());
		let rate_per_round = convert_annual_to_round::<T>(config.rate);

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
			rate: Perbill::from_percent(10),
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
