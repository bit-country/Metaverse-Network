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
		let id: u32 = ParachainInfo::parachain_id().into();

		assert_eq!(
			FungibleTokenIdConvert::<ParachainInfo>::convert(RELAY_CHAIN_CURRENCY),
			Some(MultiLocation::parent())
		);

		assert_eq!(
			FungibleTokenIdConvert::<ParachainInfo>::convert(NATIVE_CURRENCY),
			Some(MultiLocation::sibling_parachain_general_key(
				id,
				NATIVE_CURRENCY.encode()
			))
		);
		assert_eq!(
			FungibleTokenIdConvert::<ParachainInfo>::convert(STABLE_CURRENCY),
			Some(MultiLocation::sibling_parachain_general_key(
				id,
				STABLE_CURRENCY.encode()
			))
		);

		assert_eq!(
			FungibleTokenIdConvert::<ParachainInfo>::convert(MultiLocation::parent()),
			Some(RELAY_CHAIN_CURRENCY)
		);
		assert_eq!(
			FungibleTokenIdConvert::<ParachainInfo>::convert(MultiLocation::sibling_parachain_general_key(
				id,
				NATIVE_CURRENCY.encode()
			)),
			Some(NATIVE_CURRENCY)
		);
		assert_eq!(
			FungibleTokenIdConvert::<ParachainInfo>::convert(MultiLocation::sibling_parachain_general_key(
				id,
				STABLE_CURRENCY.encode()
			)),
			Some(STABLE_CURRENCY)
		);

		#[cfg(feature = "with-pioneer-runtime")]
		{
			assert_eq!(FungibleTokenIdConvert::<ParachainInfo>::convert(NATIVE_CURRENCY), None);
			assert_eq!(
				FungibleTokenIdConvert::<ParachainInfo>::convert(RELAY_CHAIN_CURRENCY),
				None
			);
			assert_eq!(
				FungibleTokenIdConvert::<ParachainInfo>::convert(PARA_CHAIN_CURRENCY),
				None
			);
			assert_eq!(FungibleTokenIdConvert::<ParachainInfo>::convert(STABLE_CURRENCY), None);

			assert_eq!(
				FungibleTokenIdConvert::<ParachainInfo>::convert(MultiLocation::sibling_parachain_general_key(
					id,
					PARA_CHAIN_CURRENCY_ID.encode()
				)),
				None
			);
			assert_eq!(
				FungibleTokenIdConvert::<ParachainInfo>::convert(MultiLocation::sibling_parachain_general_key(
					id,
					STABLE_CURRENCY_ID.encode()
				)),
				None
			);

			// assert_eq!(
			// 	FungibleTokenIdConvert::convert(MultiLocation::sibling_parachain_general_key(
			// 		parachains::karura::ID,
			// 		parachains::karura::KAR_KEY.to_vec()
			// 	)),
			// 	Some(BNC)
			// );
			// assert_eq!(
			// 	FungibleTokenIdConvert::convert(MultiLocation::sibling_parachain_general_key(
			// 		parachains::karura::ID,
			// 		parachains::karura::KUSD_KEY.to_vec()
			// 	)),
			// 	Some(VSKSM)
			// );

			let native_currency: MultiAsset = (
				MultiLocation::sibling_parachain_general_key(id, NATIVE_CURRENCY.encode()),
				1,
			)
				.into();

			assert_eq!(
				FungibleTokenIdConvert::<ParachainInfo>::convert(native_currency),
				Some(NATIVE_CURRENCY)
			);
		}
	});
}
//
// #[test]
// fn parachain_subaccounts_are_unique() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		let parachain: AccountId = ParachainInfo::parachain_id().into_account();
// 		assert_eq!(
// 			parachain,
// 			hex_literal::hex!["70617261d0070000000000000000000000000000000000000000000000000000"].into()
// 		);
//
// 		assert_eq!(
// 			create_x2_parachain_multilocation(0),
// 			MultiLocation::new(
// 				1,
// 				X1(Junction::AccountId32 {
// 					network: NetworkId::Any,
// 					id: hex_literal::hex!["d7b8926b326dd349355a9a7cca6606c1e0eb6fd2b506066b518c7155ff0d8297"].into(),
// 				})
// 			),
// 		);
// 		assert_eq!(
// 			create_x2_parachain_multilocation(1),
// 			MultiLocation::new(
// 				1,
// 				X1(Junction::AccountId32 {
// 					network: NetworkId::Any,
// 					id: hex_literal::hex!["74d37d762e06c6841a5dad64463a9afe0684f7e45245f6a7296ca613cca74669"].into(),
// 				})
// 			),
// 		);
// 	});
// }
