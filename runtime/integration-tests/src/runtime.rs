// This file is part of Bit.Country.

// Copyright (C) 2020-2021 Bit.Country.
// SPDX-License-Identifier: Apache-2.0

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

use crate::setup::*;

#[test]
fn fungible_token_id_convert() {
    ExtBuilder::default().build().execute_with(|| {
        let para_id: u32 = 2096u32;;

        // Convert FungibleTokenId
        assert_eq!(
            FungibleTokenIdConvert::convert(RELAY_CHAIN_CURRENCY),
            Some(MultiLocation::parent())
        );

        assert_eq!(
            FungibleTokenIdConvert::convert(NATIVE_CURRENCY),
            Some(MultiLocation::sibling_parachain_general_key(
                para_id,
                NATIVE_CURRENCY.encode(),
			))
        );

        assert_eq!(
            FungibleTokenIdConvert::convert(PARA_CHAIN_CURRENCY),
            Some(MultiLocation::sibling_parachain_general_key(
				parachains::karura::ID,
				parachains::karura::KAR_KEY.to_vec(),
			))
        );

        assert_eq!(
            FungibleTokenIdConvert::convert(STABLE_CURRENCY),
            Some(MultiLocation::sibling_parachain_general_key(
				parachains::karura::ID,
				parachains::karura::KUSD_KEY.to_vec(),
			))
        );

        assert_eq!(
            FungibleTokenIdConvert::convert(PARA_CHAIN_CURRENCY_NONE), None
        );

        // Convert MultiLocation
        assert_eq!(
            FungibleTokenIdConvert::convert(MultiLocation::parent()),
            Some(RELAY_CHAIN_CURRENCY)
        );

        assert_eq!(
            FungibleTokenIdConvert::convert(MultiLocation::sibling_parachain_general_key(
                para_id,
                NATIVE_CURRENCY.encode(),
			)),
            Some(NATIVE_CURRENCY)
        );

        assert_eq!(
            FungibleTokenIdConvert::convert(MultiLocation::sibling_parachain_general_key(
				parachains::karura::ID,
				parachains::karura::KAR_KEY.to_vec(),
			)),
            Some(PARA_CHAIN_CURRENCY)
        );

        assert_eq!(
        	FungibleTokenIdConvert::convert(MultiLocation::sibling_parachain_general_key(
        		parachains::karura::ID,
        		parachains::karura::KUSD_KEY.to_vec()
        	)),
        	Some(STABLE_CURRENCY)
        );

		//////////////////
        assert_eq!(
            FungibleTokenIdConvert::convert(MultiLocation::sibling_parachain_general_key(
                para_id,
                STABLE_CURRENCY.encode(),
			)),
            None
        );

        assert_eq!(
            FungibleTokenIdConvert::convert(MultiLocation::sibling_parachain_general_key(
                para_id,
                PARA_CHAIN_CURRENCY_ID.encode(),
			)),
            None
        );

        assert_eq!(
            FungibleTokenIdConvert::convert(MultiLocation::sibling_parachain_general_key(
                para_id,
                STABLE_CURRENCY_ID.encode(),
			)),
            None
        );

        // Convert MultiAsset
        let native_currency: MultiAsset = (
            MultiLocation::sibling_parachain_general_key(para_id, NATIVE_CURRENCY.encode()),
            1,
        ).into();

        assert_eq!(
            FungibleTokenIdConvert::convert(native_currency),
            Some(NATIVE_CURRENCY)
        );
    });
}