// This file is part of Acala.

// Copyright (C) 2020-2022 Acala Foundation.
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

use crate::{Balance, Currencies, Metaverse, Nft, Runtime};
use frame_benchmarking::account;
use frame_support::traits::tokens::fungibles;
use frame_support::{assert_ok, traits::Contains};
use frame_system::RawOrigin;
use orml_traits::MultiCurrencyExtended;
use primitives::{AccountId, Balance, FungibleTokenId};
use sp_runtime::{
	traits::{SaturatedConversion, StaticLookup},
	DispatchResult,
};
use sp_std::prelude::*;

fn dollar(d: u32) -> Balance {
	let d: Balance = d.into();
	d.saturating_mul(1_000_000_000_000_000_000)
}
// pub fn lookup_of_account(who: AccountId) -> <<Runtime as frame_system::Config>::Lookup as
// StaticLookup>::Source { 	<Runtime as frame_system::Config>::Lookup::unlookup(who)
// }

pub fn set_balance(currency_id: FungibleTokenId, who: &AccountId, balance: Balance) {
	assert_ok!(<Currencies as MultiCurrencyExtended<_>>::update_balance(
		currency_id,
		who,
		balance.saturated_into()
	));
}


fn mint_NFT(caller: &AccountId) {
	//Nft::create_group(RawOrigin::Root.into(), vec![1], vec![1]);
	Nft::create_class(
		RawOrigin::Signed(caller).into(),
		vec![1],
		test_attributes(1),
		0u32.into(),
		TokenType::Transferable,
		CollectionType::Collectable,
		Perbill::from_percent(0u32),
	);
	Nft::mint(
		RawOrigin::Signed(caller).into(),
		0u32.into(),
		vec![1],
		test_attributes(1),
		3,
	);
}

fn create_metaverse_for_account(caller: &AccountId) {
	Metaverse::create_metaverse(
		RawOrigin::Signed(caller).into(),
		vec![1u8],
	);
}

fn test_attributes(x: u8) -> Attributes {
	let mut attr: Attributes = BTreeMap::new();
	attr.insert(vec![x, x + 5], vec![x, x + 10]);
	attr
}

// pub fn feed_price(prices: Vec<(CurrencyId, Price)>) -> DispatchResult {
// 	for i in 0..MinimumCount::get() {
// 		let oracle: AccountId = account("oracle", 0, i);
// 		if !OperatorMembershipAcala::contains(&oracle) {
// 			OperatorMembershipAcala::add_member(RawOrigin::Root.into(), oracle.clone())?;
// 		}
// 		AcalaOracle::feed_values(RawOrigin::Signed(oracle).into(), prices.to_vec())
// 			.map_or_else(|e| Err(e.error), |_| Ok(()))?;
// 	}
//
// 	Ok(())
// }
//
// pub fn set_balance_fungibles(currency_id: CurrencyId, who: &AccountId, balance: Balance) {
// 	assert_ok!(<orml_tokens::Pallet<Runtime> as fungibles::Mutate<AccountId>>::mint_into(currency_id,
// who, balance)); }
//
//pub fn dollar(currency_id: CurrencyId) -> Balance {
// 	if let Some(decimals) =
// module_asset_registry::EvmErc20InfoMapping::<Runtime>::decimals(currency_id) {
// 		10u128.saturating_pow(decimals.into())
// 	} else {
// 		panic!("{:?} not support decimals", currency_id);
// 	}
// }

#[cfg(test)]
pub mod tests {
	pub fn new_test_ext() -> sp_io::TestExternalities {
		frame_system::GenesisConfig::default()
			.build_storage::<crate::Runtime>()
			.unwrap()
			.into()
	}
}
