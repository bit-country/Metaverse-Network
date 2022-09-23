// This file is part of Metaverse.Network & Bit.Country.

// Copyright (C) 2020-2022 Metaverse.Network & Bit.Country .
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
#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use frame_support::{
	traits::Get,
	weights::{constants::WEIGHT_PER_SECOND, Weight},
};
use sp_runtime::{FixedPointNumber, FixedU128};
use sp_std::{marker::PhantomData, prelude::*};
use xcm::latest::prelude::*;
use xcm_builder::TakeRevenue;
use xcm_executor::{
	traits::{DropAssets, WeightTrader},
	Assets,
};

use primitives::BuyWeightRate;

pub mod currencies;
pub mod precompiles;

/// Simple fee calculator that requires payment in a single fungible at a fixed rate.
///
/// - The `FixedRate` constant should be the concrete fungible ID and the amount of it
/// required for one second of weight.
/// - The `TakeRevenue` trait is used to collecting xcm execution fee.
/// - The `BuyWeightRate` trait is used to calculate ratio by location.
pub struct FixedRateOfAsset<FixedRate: Get<u128>, R: TakeRevenue, M: BuyWeightRate> {
	weight: Weight,
	amount: u128,
	ratio: FixedU128,
	multi_location: Option<MultiLocation>,
	_marker: PhantomData<(FixedRate, R, M)>,
}

impl<FixedRate: Get<u128>, R: TakeRevenue, M: BuyWeightRate> WeightTrader for FixedRateOfAsset<FixedRate, R, M> {
	fn new() -> Self {
		Self {
			weight: 0,
			amount: 0,
			ratio: Default::default(),
			multi_location: None,
			_marker: PhantomData,
		}
	}

	fn buy_weight(&mut self, weight: Weight, payment: Assets) -> Result<Assets, XcmError> {
		log::trace!(target: "xcm::weight", "buy_weight weight: {:?}, payment: {:?}", weight, payment);

		// only support first fungible assets now.
		let asset_id = payment
			.fungible
			.iter()
			.next()
			.map_or(Err(XcmError::TooExpensive), |v| Ok(v.0))?;

		if let AssetId::Concrete(ref multi_location) = asset_id {
			log::debug!(target: "xcm::weight", "buy_weight multi_location: {:?}", multi_location);

			if let Some(ratio) = M::calculate_rate(multi_location.clone()) {
				// The WEIGHT_PER_SECOND is non-zero.
				let weight_ratio = FixedU128::saturating_from_rational(weight as u128, WEIGHT_PER_SECOND as u128);
				let amount = ratio.saturating_mul_int(weight_ratio.saturating_mul_int(FixedRate::get()));

				let required = MultiAsset {
					id: asset_id.clone(),
					fun: Fungible(amount),
				};

				log::trace!(
					target: "xcm::weight", "buy_weight payment: {:?}, required: {:?}, fixed_rate: {:?}, ratio: {:?}, weight_ratio: {:?}",
					payment, required, FixedRate::get(), ratio, weight_ratio
				);
				let unused = payment
					.clone()
					.checked_sub(required)
					.map_err(|_| XcmError::TooExpensive)?;
				self.weight = self.weight.saturating_add(weight);
				self.amount = self.amount.saturating_add(amount);
				self.ratio = ratio;
				self.multi_location = Some(multi_location.clone());
				return Ok(unused);
			}
		}

		log::trace!(target: "xcm::weight", "no concrete fungible asset");
		Err(XcmError::TooExpensive)
	}

	fn refund_weight(&mut self, weight: Weight) -> Option<MultiAsset> {
		log::trace!(
			target: "xcm::weight", "refund_weight weight: {:?}, weight: {:?}, amount: {:?}, ratio: {:?}, multi_location: {:?}",
			weight, self.weight, self.amount, self.ratio, self.multi_location
		);
		let weight = weight.min(self.weight);
		let weight_ratio = FixedU128::saturating_from_rational(weight as u128, WEIGHT_PER_SECOND as u128);
		let amount = self
			.ratio
			.saturating_mul_int(weight_ratio.saturating_mul_int(FixedRate::get()));

		self.weight = self.weight.saturating_sub(weight);
		self.amount = self.amount.saturating_sub(amount);

		log::trace!(target: "xcm::weight", "refund_weight amount: {:?}", amount);
		if amount > 0 && self.multi_location.is_some() {
			Some(
				(
					self.multi_location.as_ref().expect("checked is non-empty; qed").clone(),
					amount,
				)
					.into(),
			)
		} else {
			None
		}
	}
}

impl<FixedRate: Get<u128>, R: TakeRevenue, M: BuyWeightRate> Drop for FixedRateOfAsset<FixedRate, R, M> {
	fn drop(&mut self) {
		log::trace!(target: "xcm::weight", "take revenue, weight: {:?}, amount: {:?}, multi_location: {:?}", self.weight, self.amount, self.multi_location);
		if self.amount > 0 && self.multi_location.is_some() {
			R::take_revenue(
				(
					self.multi_location.as_ref().expect("checked is non-empty; qed").clone(),
					self.amount,
				)
					.into(),
			);
		}
	}
}
