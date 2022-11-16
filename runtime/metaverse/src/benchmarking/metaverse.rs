use super::utils::{create_nft_group, dollar, set_balance, set_metaverse_treasury_initial_balance};
#[allow(unused)]
use crate::{Balances, Event, LocalMetaverseFundPalletId, Metaverse, MetaverseNetworkTreasuryPalletId, MinContribution, Nft, Runtime};
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
	LocalMetaverseFundPalletId::get().into_sub_account_truncating(metaverse_id)
}

runtime_benchmarks! {
	{ Runtime, metaverse }

	create_metaverse{
		create_nft_group();
		let caller: AccountId = account("caller", 0, SEED);
		set_balance(CURRENCY_ID, &caller, dollar(10000));
		set_metaverse_treasury_initial_balance();
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
		set_balance(CURRENCY_ID, &caller, dollar(1000));
		let target: AccountId = account("target", 1, SEED);
		set_balance(CURRENCY_ID, &target, dollar(1000));

		create_nft_group();
		set_metaverse_treasury_initial_balance();
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
		set_balance(CURRENCY_ID, &caller, dollar(1000));
		let target: AccountId = account("target", 1, SEED);
		set_balance(CURRENCY_ID, &target, dollar(1000));
		create_nft_group();
		set_metaverse_treasury_initial_balance();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1]);
	}: _(RawOrigin::Root, 0)
	verify {
		let metaverse = Metaverse::get_metaverse(0);
		match metaverse {
			Some(a) => {
				assert_eq!(a.is_frozen, true);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	}

	unfreeze_metaverse{
		let caller: AccountId = account("caller", 0, SEED);
		set_balance(CURRENCY_ID, &caller, dollar(1000));
		let target: AccountId = account("target", 1, SEED);
		set_balance(CURRENCY_ID, &target, dollar(1000));

		create_nft_group();
		set_metaverse_treasury_initial_balance();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1]);
		Metaverse::freeze_metaverse(RawOrigin::Root.into(), 0);
	}: _(RawOrigin::Root, 0)
	verify {
		let metaverse = Metaverse::get_metaverse(0);
		match metaverse {
			Some(a) => {
				// Verify details of Metaverse
				assert_eq!(a.is_frozen, false);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	}

	destroy_metaverse{
		let caller: AccountId = account("caller", 0, SEED);
		set_balance(CURRENCY_ID, &caller, dollar(1000));
		let target: AccountId = account("target", 1, SEED);
		set_balance(CURRENCY_ID, &target, dollar(1000));

		create_nft_group();
		set_metaverse_treasury_initial_balance();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1]);
		Metaverse::freeze_metaverse(RawOrigin::Root.into(), 0);
	}: _(RawOrigin::Root, 0)
	verify {
		assert_eq!(Metaverse::get_metaverse(0), None);
	}

	update_metaverse_listing_fee {
		let caller: AccountId = account("caller", 0, SEED);
		set_balance(CURRENCY_ID, &caller, dollar(1000));
		create_nft_group();
		set_metaverse_treasury_initial_balance();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1]);
	}: _(RawOrigin::Signed(caller.clone()), 0, Perbill::from_percent(1u32))
	verify {
		let metaverse_info = Metaverse::get_metaverse(0);
		match metaverse_info {
			Some(v) => {
				assert_eq!(v.listing_fee, Perbill::from_percent(1u32));
			}
			_ => {
				assert_eq!(0,1);
			}
		}
	}
 
	withdraw_from_metaverse_fund{
		let caller: AccountId = account("caller", 0, SEED);
		set_balance(CURRENCY_ID, &caller, dollar(1000));
		create_nft_group();
		set_metaverse_treasury_initial_balance();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1]);
		let metaverse_fund: AccountId = get_metaverse_fund(0u32.into());
		set_balance(CURRENCY_ID, &metaverse_fund, dollar(1000));
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
