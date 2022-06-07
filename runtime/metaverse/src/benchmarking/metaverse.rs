use super::utils::{create_land_and_estate_group, dollar, set_balance};
#[allow(unused)]
use crate::{Balances, Event, Metaverse, MetaverseNetworkTreasuryPalletId, MinContribution, Nft, Runtime};
use core_primitives::MetaverseInfo;
use frame_benchmarking::{account, whitelisted_caller};
use frame_support::assert_ok;
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;
use orml_benchmarking::runtime_benchmarks;
use primitives::{AccountId, Balance, ClassId, FungibleTokenId, GroupCollectionId, MetaverseId};
use sp_runtime::traits::{AccountIdConversion, StaticLookup, UniqueSaturatedInto};
use sp_runtime::Perbill;
use sp_std::{collections::btree_map::BTreeMap, prelude::*, vec};

const SEED: u32 = 0;
const CURRENCY_ID: FungibleTokenId = FungibleTokenId::NativeToken(0);

const MAX_BOUND: (i32, i32) = (-100, 100);
const COORDINATE_IN_1: (i32, i32) = (-10, 10);
const COORDINATE_IN_2: (i32, i32) = (-5, 5);
const COORDINATE_OUT: (i32, i32) = (0, 101);
const COORDINATE_IN_AUCTION: (i32, i32) = (99, 99);

fn get_metaverse_fund(metaverse_id: MetaverseId) -> AccountId {
	MetaverseNetworkTreasuryPalletId::get().into_sub_account(metaverse_id)
}

runtime_benchmarks! {
	{ Runtime, metaverse }
	create_metaverse{
		let caller: AccountId = account("caller", 0, SEED);
		set_balance(CURRENCY_ID, &caller, dollar(10));
	}: _(RawOrigin::Signed(caller.clone()), vec![1])
	verify {
		let metaverse = Metaverse::get_metaverse(0);
		match metaverse {
			Some(a) => {
				assert_eq!(a.owner, caller.clone());
				assert_eq!(a.is_frozen, false);
				assert_eq!(a.metadata, vec![1]);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	}

	transfer_metaverse{
		let caller: AccountId = account("caller", 0, SEED);
		set_balance(CURRENCY_ID, &caller, dollar(10));
		let target: AccountId = account("caller", 0, SEED);
		set_balance(CURRENCY_ID, &target, dollar(10));

		create_land_and_estate_group();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1]);
	}: _(RawOrigin::Signed(caller.clone()), target.clone(), 0)
	verify {
		let metaverse = Metaverse::get_metaverse(0);
		match metaverse {
		Some(a) => {
				assert_eq!(a.owner, target.clone());
				assert_eq!(a.is_frozen, false);
				assert_eq!(a.metadata, vec![1]);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	}

	freeze_metaverse{
		let caller: AccountId = account("caller", 0, SEED);
		set_balance(CURRENCY_ID, &caller, dollar(10));
		let target: AccountId = account("caller", 0, SEED);
		set_balance(CURRENCY_ID, &target, dollar(10));
		create_land_and_estate_group();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1]);
	}: _(RawOrigin::Root, 0)

	unfreeze_metaverse{
		let caller: AccountId = account("caller", 0, SEED);
		set_balance(CURRENCY_ID, &caller, dollar(10));
		let target: AccountId = account("caller", 0, SEED);
		set_balance(CURRENCY_ID, &target, dollar(10));

		create_land_and_estate_group();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1]);
		Metaverse::freeze_metaverse(RawOrigin::Root.into(), 0);
	}: _(RawOrigin::Root, 0)

	destroy_metaverse{
		let caller: AccountId = account("caller", 0, SEED);
		set_balance(CURRENCY_ID, &caller, dollar(10));
		let target: AccountId = account("caller", 0, SEED);
		set_balance(CURRENCY_ID, &target, dollar(10));

		create_land_and_estate_group();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1]);
		Metaverse::freeze_metaverse(RawOrigin::Root.into(), 0);
	}: _(RawOrigin::Root, 0)
	verify {
		assert_eq!(Metaverse::get_metaverse(0), None);
	}

	register_metaverse{
		let caller: AccountId = account("caller", 0, SEED);
		set_balance(CURRENCY_ID, &caller, dollar(10));
		let target: AccountId = account("caller", 0, SEED);
		set_balance(CURRENCY_ID, &target, dollar(10));

		create_land_and_estate_group();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1]);
	}: _(RawOrigin::Signed(caller.clone()), 0)
	verify {
		let metaverse = Metaverse::get_registered_metaverse(0);
		match metaverse {
			Some(a) => {
				assert_eq!(1, 1);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	}

	stake{
		let caller: AccountId = account("caller", 0, SEED);
		set_balance(CURRENCY_ID, &caller, dollar(10));
		let target: AccountId = account("caller", 0, SEED);
		set_balance(CURRENCY_ID, &target, dollar(10));

		create_land_and_estate_group();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1]);
		Metaverse::register_metaverse(RawOrigin::Signed(caller.clone()).into(), 0);
	}: _(RawOrigin::Signed(caller.clone()), 0, dollar(2))
	verify {
		let staking_info = Metaverse::staking_info(caller);
		assert_eq!(staking_info, dollar(2));
	}

	unstake_and_withdraw{
		let caller: AccountId = account("caller", 0, SEED);
		set_balance(CURRENCY_ID, &caller, dollar(10));
		let target: AccountId = account("caller", 0, SEED);
		set_balance(CURRENCY_ID, &target, dollar(10));
		create_land_and_estate_group();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1]);
		Metaverse::register_metaverse(RawOrigin::Signed(caller.clone()).into(), 0);
		Metaverse::stake(RawOrigin::Signed(caller.clone()).into(), 0, dollar(2));
	}: _(RawOrigin::Signed(caller.clone()), 0, 1u32.into())
	verify {
		let staking_info = Metaverse::staking_info(caller);
		assert_eq!(staking_info, 1999999999999999999);
	}

	update_metaverse_listing_fee {
		let caller: AccountId = account("caller", 0, SEED);
		set_balance(CURRENCY_ID, &caller, dollar(10));
		create_land_and_estate_group();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1]);
		Metaverse::register_metaverse(RawOrigin::Signed(caller.clone()).into(), 0);
	}: _(RawOrigin::Signed(caller.clone()), 0, Perbill::from_percent(1u32))

	withdraw_funds_from_metaverse_fund{
		let caller: AccountId = account("caller", 0, SEED);
		set_balance(CURRENCY_ID, &caller, dollar(10));
		create_land_and_estate_group();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1]);
		let metaverse_fund: AccountId = get_metaverse_fund(0u32.into());
		Balances::make_free_balance_be(&metaverse_fund, dollar(100).unique_saturated_into());
	}: _(RawOrigin::Signed(caller), 0u32.into())
	verify {
		assert_eq!(Balances::free_balance(&metaverse_fund), 1u32.into());
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::benchmarking::utils::tests::new_test_ext;
	use orml_benchmarking::impl_benchmark_test_suite;

	impl_benchmark_test_suite!(new_test_ext(),);
}
