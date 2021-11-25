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

// Helper methods to compute the issuance rate for undeployed land.
use crate::pallet::{Config, Pallet};
use crate::{AllLandUnitsCount, TotalUndeployedLandUnit};
use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{Perbill, RuntimeDebug};

const SECONDS_PER_YEAR: u32 = 31557600;
const SECONDS_PER_BLOCK: u32 = 12;
const BLOCKS_PER_YEAR: u32 = SECONDS_PER_YEAR / SECONDS_PER_BLOCK;

fn rounds_per_year<T: Config>() -> u32 {
	let blocks_per_round = <Pallet<T>>::round().length;
	BLOCKS_PER_YEAR / blocks_per_round
}

#[warn(dead_code)]
fn get_annual_max_issuance<T: Config>(max_supply: u64, annual_percentage: u64) -> u64 {
	let total_land_unit_circulating = <AllLandUnitsCount<T>>::get();
	let total_undeployed_land_unit_circulating = <TotalUndeployedLandUnit<T>>::get();
	let circulating = total_land_unit_circulating.saturating_add(total_undeployed_land_unit_circulating);
	max_supply.saturating_sub(circulating).saturating_mul(annual_percentage)
}

/// Compute round issuance range from round inflation range and current total issuance
pub fn round_issuance_range<T: Config>(config: MintingRateInfo) -> Range<u64> {
	// Get total round per year
	let total_round_per_year = rounds_per_year::<T>();

	// Get total land unit circulating
	let total_land_unit_circulating = <AllLandUnitsCount<T>>::get();

	// Get total undeployed land unit circulating
	let total_undeployed_land_unit_circulating = <TotalUndeployedLandUnit<T>>::get();

	// Total circulating
	let circulating = total_land_unit_circulating.saturating_add(total_undeployed_land_unit_circulating);

	// Total annual minting percent
	let annual_percentage = Perbill::from_percent(config.annual as u32).deconstruct();

	// Round percentage minting rate
	let round_percentage = annual_percentage.checked_div(total_round_per_year).unwrap();

	// Convert to percentage
	let round_percentage_per_bill = Perbill::from_parts(round_percentage);

	// Return range - could implement more cases in the future.
	Range {
		min: round_percentage_per_bill * circulating,
		ideal: round_percentage_per_bill * circulating,
		max: round_percentage_per_bill * circulating,
	}
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Copy, Encode, Decode, Default, RuntimeDebug, TypeInfo)]
pub struct Range<T> {
	pub min: T,
	pub ideal: T,
	pub max: T,
}

impl<T: Ord> Range<T> {
	pub fn is_valid(&self) -> bool {
		self.max >= self.ideal && self.ideal >= self.min
	}
}

impl<T: Ord + Copy> From<T> for Range<T> {
	fn from(other: T) -> Range<T> {
		Range {
			min: other,
			ideal: other,
			max: other,
		}
	}
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, Default, RuntimeDebug, TypeInfo)]
pub struct MintingRateInfo {
	/// Number of metaverse expectations
	pub expect: Range<u64>,
	/// Annual minting range
	pub annual: u64,
	/// Max total supply
	pub max: u64,
}

impl MintingRateInfo {
	pub fn new<T: Config>(annual: u64, expect: Range<u64>, max: u64) -> MintingRateInfo {
		MintingRateInfo { expect, annual, max }
	}

	/// Set minting rate expectations
	pub fn set_expectations(&mut self, expect: Range<u64>) {
		self.expect = expect;
	}

	/// Set minting rate expectations
	pub fn set_max(&mut self, max: u64) {
		self.max = max;
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Compute round issuance range from round inflation range and current total issuance
	pub fn mock_round_issuance_per_year(
		config: MintingRateInfo,
		circulation: u64,
		total_round_per_year: u32,
	) -> Range<u64> {
		let annual_percentage = Perbill::from_percent(config.annual as u32).deconstruct();
		let round_percentage = annual_percentage.checked_div(total_round_per_year).unwrap();

		let round_percentage_per_bill = Perbill::from_parts(round_percentage);

		Range {
			min: round_percentage_per_bill * circulation,
			ideal: round_percentage_per_bill * circulation,
			max: round_percentage_per_bill * circulation,
		}
	}

	#[test]
	fn simple_round_issuance() {
		// 5% minting rate for 100_000 land unit = 100 minted over the year
		// let's assume there are 10 periods in a year
		// => mint 100 over 10 periods => 10 minted per period

		let mock_config: MintingRateInfo = MintingRateInfo {
			expect: Default::default(),
			annual: 5,
			max: 100_000,
		};

		let round_issuance = mock_round_issuance_per_year(mock_config, 2_000, 10);

		// make sure 10 land unit deploy per period
		assert_eq!(round_issuance.min, 10);
		assert_eq!(round_issuance.ideal, 10);
		assert_eq!(round_issuance.max, 10);
	}
}
