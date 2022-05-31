use frame_benchmarking::account;
use frame_support::traits::tokens::fungibles;
use frame_support::{assert_ok, traits::Contains};
use frame_system::RawOrigin;
use orml_traits::MultiCurrencyExtended;
use sp_runtime::Perbill;
use sp_runtime::{
	traits::{SaturatedConversion, StaticLookup},
	DispatchResult,
};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::prelude::*;

use core_primitives::{Attributes, CollectionType, TokenType};
use primitives::estate::EstateInfo;
use primitives::{AccountId, Balance, FungibleTokenId, UndeployedLandBlockType};

use crate::{Auction, Balances, Currencies, Estate, Metaverse, Nft, Runtime};

const SEED: u32 = 0;
const METAVERSE_ID: u64 = 1;

pub fn dollar(d: u32) -> Balance {
	let d: Balance = d.into();
	d.saturating_mul(1_000_000_000_000_000_000)
}
//pub fn lookup_of_account(who: AccountId)
//-> <<Runtime as frame_system::Config>::Lookup as StaticLookup>::Source {
//	<Runtime as frame_system::Config>::Lookup::unlookup(who)
//}

pub fn set_balance(currency_id: FungibleTokenId, who: &AccountId, balance: Balance) {
	assert_ok!(<Currencies as MultiCurrencyExtended<_>>::update_balance(
		currency_id,
		who,
		balance.saturated_into()
	));
}

pub fn mint_NFT(caller: &AccountId, class_id: u32) {
	assert_ok!(Nft::create_class(
		RawOrigin::Signed(caller.clone()).into(),
		vec![1],
		test_attributes(1),
		0u32.into(),
		TokenType::Transferable,
		CollectionType::Collectable,
		Perbill::from_percent(0u32),
	));
	assert_ok!(Nft::mint(
		RawOrigin::Signed(caller.clone()).into(),
		class_id,
		vec![3],
		test_attributes(3),
		1,
	));
}

pub fn create_nft_group() {
	assert_ok!(Nft::create_group(RawOrigin::Root.into(), vec![1], vec![1]));
}

pub fn create_land_and_estate_groups() {
	assert_ok!(Nft::create_group(RawOrigin::Root.into(), vec![1], vec![1]));
}

pub fn get_estate_info(lands: Vec<(i32, i32)>) -> EstateInfo {
	return EstateInfo {
		metaverse_id: METAVERSE_ID,
		land_units: lands,
	};
}

pub fn issue_new_undeployed_land_block(n: u32) -> Result<bool, &'static str> {
	let caller: AccountId = account("caller", 0, SEED);
	set_balance(FungibleTokenId::NativeToken(0), &caller, 10000);
	assert_ok!(Estate::issue_undeployed_land_blocks(
		RawOrigin::Root.into(),
		caller,
		n,
		100,
		UndeployedLandBlockType::Transferable,
	));

	Ok(true)
}

pub fn create_metaverse_for_account(caller: &AccountId) {
	create_land_and_estate_groups();
	assert_ok!(Metaverse::create_metaverse(
		RawOrigin::Signed(caller.clone()).into(),
		vec![1u8]
	));
}

pub fn test_attributes(x: u8) -> Attributes {
	let mut attr: Attributes = BTreeMap::new();
	attr.insert(vec![x, x + 5], vec![x, x + 10]);
	attr
}

#[cfg(test)]
pub mod tests {
	pub fn new_test_ext() -> sp_io::TestExternalities {
		frame_system::GenesisConfig::default()
			.build_storage::<crate::Runtime>()
			.unwrap()
			.into()
	}
}
