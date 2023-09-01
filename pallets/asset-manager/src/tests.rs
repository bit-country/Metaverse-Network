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

use mock::{AssetManager, CouncilAccount, ExtBuilder, Runtime, RuntimeEvent, RuntimeOrigin, System};
use primitives::evm::{CurrencyIdType, EvmAddress, H160_POSITION_CURRENCY_ID_TYPE, H160_POSITION_TOKEN};
use primitives::FungibleTokenId::FungibleToken;
use primitives::{TokenId, TokenSymbol};

use super::*;

#[test]
fn register_foreign_asset_work() {
	ExtBuilder::default().build().execute_with(|| {
		let v2_location = VersionedMultiLocation::V2(xcm::v2::MultiLocation::new(
			0,
			xcm::v2::Junctions::X1(xcm::v2::Junction::Parachain(2096)),
		));

		assert_ok!(AssetManager::register_foreign_asset(
			RuntimeOrigin::signed(CouncilAccount::get()),
			Box::new(v2_location.clone()),
			Box::new(AssetMetadata {
				name: b"TNEER".to_vec(),
				symbol: b"TNEER".to_vec(),
				decimals: 18,
				minimal_balance: 1,
			})
		));

		let location: MultiLocation = v2_location.try_into().unwrap();
		System::assert_last_event(RuntimeEvent::AssetManager(crate::Event::ForeignAssetRegistered {
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
		let v2_location = VersionedMultiLocation::V2(xcm::v2::MultiLocation::new(
			0,
			xcm::v2::Junctions::X1(xcm::v2::Junction::Parachain(2096)),
		));

		assert_ok!(AssetManager::register_foreign_asset(
			RuntimeOrigin::signed(CouncilAccount::get()),
			Box::new(v2_location.clone()),
			Box::new(AssetMetadata {
				name: b"TNEER".to_vec(),
				symbol: b"TNEER".to_vec(),
				decimals: 18,
				minimal_balance: 1,
			})
		));

		assert_noop!(
			AssetManager::register_foreign_asset(
				RuntimeOrigin::signed(CouncilAccount::get()),
				Box::new(v2_location.clone()),
				Box::new(AssetMetadata {
					name: b"TNEER".to_vec(),
					symbol: b"TNEER".to_vec(),
					decimals: 18,
					minimal_balance: 1,
				})
			),
			Error::<Runtime>::MultiLocationExisted
		);

		let location: MultiLocation = v2_location.try_into().unwrap();
		System::assert_last_event(RuntimeEvent::AssetManager(crate::Event::ForeignAssetRegistered {
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
		// v2
		let v2_location = VersionedMultiLocation::V2(xcm::v2::MultiLocation::new(
			0,
			xcm::v2::Junctions::X1(xcm::v2::Junction::Parachain(2096)),
		));
		let location: MultiLocation = v2_location.try_into().unwrap();
		assert_eq!(
			location,
			MultiLocation {
				parents: 0,
				interior: xcm::v3::Junctions::X1(xcm::v3::Junction::Parachain(2096))
			}
		);

		// v3
		let v3_location = VersionedMultiLocation::V3(MultiLocation {
			parents: 0,
			interior: xcm::v3::Junctions::X1(xcm::v3::Junction::Parachain(2096)),
		});
		let location: MultiLocation = v3_location.try_into().unwrap();
		assert_eq!(
			location,
			MultiLocation {
				parents: 0,
				interior: xcm::v3::Junctions::X1(xcm::v3::Junction::Parachain(2096))
			}
		);

		// handle all of VersionedMultiLocation
		assert!(match location.into() {
			VersionedMultiLocation::V2 { .. } | VersionedMultiLocation::V3 { .. } => true,
		});
	});
}

#[test]
fn evm_decode_address_works() {
	ExtBuilder::default().build().execute_with(|| {
		let neer_evm_address = EvmAddress::try_from(FungibleTokenId::NativeToken(0)).ok();
		let nuum_evm_address = EvmAddress::try_from(FungibleTokenId::NativeToken(1)).ok();
		let bit_evm_address = EvmAddress::try_from(FungibleTokenId::MiningResource(0)).ok();
		let fungile_asset_one_evm_address = EvmAddress::try_from(FungibleTokenId::FungibleToken(0)).ok();

		let address = EvmAddress::try_from(FungibleTokenId::MiningResource(5)).unwrap();

		let currency_id = match CurrencyIdType::try_from(address[H160_POSITION_CURRENCY_ID_TYPE])
			.ok()
			.unwrap()
		{
			CurrencyIdType::NativeToken => address[H160_POSITION_TOKEN]
				.try_into()
				.map(FungibleTokenId::NativeToken)
				.ok(),
			CurrencyIdType::MiningResource => address[H160_POSITION_TOKEN]
				.try_into()
				.map(FungibleTokenId::MiningResource)
				.ok(),
			CurrencyIdType::FungibleToken => address[H160_POSITION_TOKEN]
				.try_into()
				.map(FungibleTokenId::FungibleToken)
				.ok(),
		};

		assert_eq!(currency_id.unwrap(), FungibleTokenId::MiningResource(5));

		assert_eq!(
			neer_evm_address,
			H160::from_str("0x0000000000000000000100000000000000000000").ok()
		);
		assert_eq!(
			fungile_asset_one_evm_address,
			H160::from_str("0x0000000000000000000200000000000000000000").ok()
		);
		assert_eq!(
			nuum_evm_address,
			H160::from_str("0x0000000000000000000100000000000000000001").ok()
		);

		assert_eq!(
			bit_evm_address,
			H160::from_str("0x0000000000000000000300000000000000000000").ok()
		);

		assert_eq!(neer_evm_address.unwrap().to_fixed_bytes()[0..9], [0u8; 9])
	})
}

#[test]
fn ensure_precompile_addresses_are_correct() {
	ExtBuilder::default().build().execute_with(|| {
		let currency_precompile_address = [0u8; 20];
		let metaverse_precompile_address = [
			1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
		];
		let nft_precompile_address = [
			2u8, 2u8, 2u8, 2u8, 2u8, 2u8, 2u8, 2u8, 2u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
		];
		let auction_precompile_address = [
			3u8, 3u8, 3u8, 3u8, 3u8, 3u8, 3u8, 3u8, 3u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
		];

		assert_eq!(
			H160::from_slice(&currency_precompile_address),
			H160::from_str("0x0000000000000000000000000000000000000000")
				.ok()
				.unwrap()
		);
		assert_eq!(
			H160::from_slice(&metaverse_precompile_address),
			H160::from_str("0x0101010101010101010000000000000000000000")
				.ok()
				.unwrap()
		);
		assert_eq!(
			H160::from_slice(&nft_precompile_address),
			H160::from_str("0x0202020202020202020000000000000000000000")
				.ok()
				.unwrap()
		);

		assert_eq!(
			H160::from_slice(&auction_precompile_address),
			H160::from_str("0x0303030303030303030000000000000000000000")
				.ok()
				.unwrap()
		);
	})
}
