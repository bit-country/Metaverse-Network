// Copyright (C) 2019-2021 Liebi Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use codec::{Decode, Encode};
pub use cumulus_primitives_core::ParaId;
use frame_support::{
	sp_runtime::traits::{CheckedConversion, Convert},
	traits::Get,
};
use orml_traits::location::Reserve;
use polkadot_parachain::primitives::Sibling;
use primitives::{Amount, FungibleTokenId, TokenSymbol};
use sp_std::{convert::TryFrom, marker::PhantomData};
use xcm::latest::prelude::*;
// use xcm_builder::{AccountId32Aliases, NativeAsset, ParentIsDefault, SiblingParachainConvertsVia};
// use xcm_executor::traits::{FilterAssetLocation, MatchesFungible};

use crate::constants::parachains;

fn native_currency_location(id: FungibleTokenId, para_id: ParaId) -> MultiLocation {
	MultiLocation::new(1, X2(Parachain(para_id.into()), GeneralKey(id.encode())))
}

/// **************************************
// Below is for the network of Kusama.
/// **************************************

pub struct FungibleTokenIdConvert<T>(sp_std::marker::PhantomData<T>);
impl<T: Get<ParaId>> Convert<FungibleTokenId, Option<MultiLocation>> for FungibleTokenIdConvert<T> {
	fn convert(id: FungibleTokenId) -> Option<MultiLocation> {
		use FungibleTokenId::{DEXShare, FungibleToken, MiningResource, NativeToken, Stable};
		match id {
			// KSM
			FungibleToken(1) => Some(MultiLocation::parent()),
			// NEER
			NativeToken(0) => Some(native_currency_location(id, T::get())),
			// Karura currencyId types
			FungibleToken(2) => Some(MultiLocation::new(
				1,
				X2(
					Parachain(parachains::karura::ID),
					GeneralKey(parachains::karura::KAR_KEY.to_vec()),
				),
			)),
			Stable(3) => Some(MultiLocation::new(
				1,
				X2(
					Parachain(parachains::karura::ID),
					GeneralKey(parachains::karura::KUSD_KEY.to_vec()),
				),
			)),
			_ => None,
		}
	}
}

impl<T: Get<ParaId>> Convert<MultiLocation, Option<FungibleTokenId>> for FungibleTokenIdConvert<T> {
	fn convert(location: MultiLocation) -> Option<FungibleTokenId> {
		use FungibleTokenId::{DEXShare, FungibleToken, MiningResource, NativeToken, Stable};
		use TokenSymbol::*;
		// TODO: use TokenSymbol enum
		// 0 => NEER
		// 1 => KSM
		// 2 => KAR
		// 3 => KUSD

		if location == MultiLocation::parent() {
			return Some(FungibleToken(1));
		}
		match location {
			MultiLocation { parents, interior } if parents == 1 => match interior {
				X2(Parachain(id), GeneralKey(key)) if ParaId::from(id) == T::get() => {
					// decode the general key
					if let Ok(currency_id) = FungibleTokenId::decode(&mut &key[..]) {
						match currency_id {
							NativeToken(0) | FungibleToken(1) => Some(currency_id),
							_ => None,
						}
					} else {
						None
					}
				}
				X2(Parachain(id), GeneralKey(key)) if id == parachains::karura::ID => {
					if key == parachains::karura::KAR_KEY.to_vec() {
						Some(FungibleToken(2))
					} else if key == parachains::karura::KUSD_KEY.to_vec() {
						Some(Stable(3))
					} else {
						None
					}
				}
				_ => None,
			},
			MultiLocation { parents, interior } if parents == 0 => match interior {
				X1(GeneralKey(key)) => {
					// decode the general key
					if let Ok(currency_id) = FungibleTokenId::decode(&mut &key[..]) {
						match currency_id {
							NativeToken(0) | FungibleToken(1) => Some(currency_id),
							_ => None,
						}
					} else {
						None
					}
				}
				_ => None,
			},
			_ => None,
		}
	}
}

impl<T: Get<ParaId>> Convert<MultiAsset, Option<FungibleTokenId>> for FungibleTokenIdConvert<T> {
	fn convert(asset: MultiAsset) -> Option<FungibleTokenId> {
		if let MultiAsset {
			id: Concrete(location), ..
		} = asset
		{
			Self::convert(location)
		} else {
			None
		}
	}
}
