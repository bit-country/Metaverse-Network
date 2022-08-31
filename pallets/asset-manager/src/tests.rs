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

// This pallet use The Open Runtime Module Library (ORML) which is a community maintained collection
// of Substrate runtime modules. Thanks to all contributors of orml.
// Ref: https://github.com/open-web3-stack/open-runtime-module-library

#![cfg(test)]

use std::str::{from_utf8, FromStr};

use frame_support::{assert_noop, assert_ok};
use sp_core::H160;

use mock::{AssetManager, CouncilAccount, Event, ExtBuilder, Origin, Runtime, System};
use primitives::TokenSymbol;

use super::*;

#[test]
fn register_foreign_asset_work() {
	ExtBuilder::default().build().execute_with(|| {
		let v0_location = VersionedMultiLocation::V0(xcm::v0::MultiLocation::X1(xcm::v0::Junction::Parachain(2096)));

		assert_ok!(AssetManager::register_foreign_asset(
			Origin::signed(CouncilAccount::get()),
			Box::new(v0_location.clone()),
			Box::new(AssetMetadata {
				name: b"TNEER".to_vec(),
				symbol: b"TNEER".to_vec(),
				decimals: 18,
				minimal_balance: 1,
			})
		));

		let location: MultiLocation = v0_location.try_into().unwrap();
		System::assert_last_event(Event::AssetManager(crate::Event::ForeignAssetRegistered {
			asset_id: 0,
			asset_address: location.clone(),
			metadata: AssetMetadata {
				name: b"TNEER".to_vec(),
				symbol: b"TNEER".to_vec(),
				decimals: 18,
				minimal_balance: 1,
			},
		}));

		assert_eq!(ForeignAssetLocations::<Runtime>::get(0), Some(location.clone()));
		assert_eq!(
			AssetMetadatas::<Runtime>::get(AssetIds::ForeignAssetId(0)),
			Some(AssetMetadata {
				name: b"TNEER".to_vec(),
				symbol: b"TNEER".to_vec(),
				decimals: 18,
				minimal_balance: 1,
			})
		);
		assert_eq!(
			LocationToFungibleTokenIds::<Runtime>::get(location),
			Some(FungibleTokenId::FungibleToken(0))
		);
	});
}

#[test]
fn register_foreign_asset_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let v0_location = VersionedMultiLocation::V0(xcm::v0::MultiLocation::X1(xcm::v0::Junction::Parachain(2096)));

		assert_ok!(AssetManager::register_foreign_asset(
			Origin::signed(CouncilAccount::get()),
			Box::new(v0_location.clone()),
			Box::new(AssetMetadata {
				name: b"TNEER".to_vec(),
				symbol: b"TNEER".to_vec(),
				decimals: 18,
				minimal_balance: 1,
			})
		));

		assert_noop!(
			AssetManager::register_foreign_asset(
				Origin::signed(CouncilAccount::get()),
				Box::new(v0_location.clone()),
				Box::new(AssetMetadata {
					name: b"TNEER".to_vec(),
					symbol: b"TNEER".to_vec(),
					decimals: 18,
					minimal_balance: 1,
				})
			),
			Error::<Runtime>::MultiLocationExisted
		);

		let location: MultiLocation = v0_location.try_into().unwrap();
		System::assert_last_event(Event::AssetManager(crate::Event::ForeignAssetRegistered {
			asset_id: 0,
			asset_address: location.clone(),
			metadata: AssetMetadata {
				name: b"TNEER".to_vec(),
				symbol: b"TNEER".to_vec(),
				decimals: 18,
				minimal_balance: 1,
			},
		}));

		assert_eq!(ForeignAssetLocations::<Runtime>::get(0), Some(location.clone()));
		assert_eq!(
			AssetMetadatas::<Runtime>::get(AssetIds::ForeignAssetId(0)),
			Some(AssetMetadata {
				name: b"TNEER".to_vec(),
				symbol: b"TNEER".to_vec(),
				decimals: 18,
				minimal_balance: 1,
			})
		);
		assert_eq!(
			LocationToFungibleTokenIds::<Runtime>::get(location),
			Some(FungibleTokenId::FungibleToken(0))
		);
	});
}

#[test]
fn versioned_multi_location_convert_work() {
	ExtBuilder::default().build().execute_with(|| {
		// v0
		let v0_location = VersionedMultiLocation::V0(xcm::v0::MultiLocation::X1(xcm::v0::Junction::Parachain(2096)));
		let location: MultiLocation = v0_location.try_into().unwrap();
		assert_eq!(
			location,
			MultiLocation {
				parents: 0,
				interior: xcm::v1::Junctions::X1(xcm::v1::Junction::Parachain(2096))
			}
		);

		// v1
		let v1_location = VersionedMultiLocation::V1(MultiLocation {
			parents: 0,
			interior: xcm::v1::Junctions::X1(xcm::v1::Junction::Parachain(2096)),
		});
		let location: MultiLocation = v1_location.try_into().unwrap();
		assert_eq!(
			location,
			MultiLocation {
				parents: 0,
				interior: xcm::v1::Junctions::X1(xcm::v1::Junction::Parachain(2096))
			}
		);

		// handle all of VersionedMultiLocation
		assert!(match location.into() {
			VersionedMultiLocation::V0 { .. } | VersionedMultiLocation::V1 { .. } => true,
		});
	});
}
