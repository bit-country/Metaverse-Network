#![cfg(feature = "runtime-benchmarks")]

use super::*;
use sp_std::prelude::*;
use sp_std::vec;

#[allow(unused)]
pub use crate::Pallet as CurrencyModule;
use crate::{Call, Config};
// use crate::Mining as MiningModule;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
//use frame_support::traits::{Currency as PalletCurrency, Get};
use frame_system::{Pallet as System, RawOrigin};
use orml_traits::{BasicCurrencyExtended as NativeCurrency, MultiCurrencyExtended as MultiSocialCurrency};
use primitives::{Balance, FungibleTokenId};
use sp_runtime::traits::{AccountIdConversion, Lookup, StaticLookup, UniqueSaturatedInto};

pub type AccountId = u128;

const SEED: u32 = 0;

const ALICE: AccountId = 1;
const CURRENCY_ID: FungibleTokenId = FungibleTokenId::NativeToken(0);

fn dollar(d: u32) -> Balance {
	let d: Balance = d.into();
	d.saturating_mul(1_000_000_000_000_000_000)
}

fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	T::NativeCurrency::update_balance(&caller, dollar(100).unique_saturated_into());
	caller
}

benchmarks! {
	transfer{
		let caller = funded_account::<T>("caller", 0);
		let to = funded_account::<T>("to", 0);
		let to_lookup = T::Lookup::unlookup(to.clone());
	}: _(RawOrigin::Signed(caller), to_lookup, CURRENCY_ID, 1u32.into())

	transfer_native_currency{
		let caller = funded_account::<T>("caller", 0);
		let to = funded_account::<T>("to", 0);
		let to_lookup = T::Lookup::unlookup(to.clone());
	}: _(RawOrigin::Signed(caller), to_lookup, 1u32.into())

	update_balance{
		let caller = funded_account::<T>("caller", 0);
		let to = funded_account::<T>("to", 0);
		let to_lookup = T::Lookup::unlookup(to.clone());
	}: _(RawOrigin::Root, to_lookup, CURRENCY_ID, 1u32.into())
}

impl_benchmark_test_suite!(Pallet, crate::benchmarking::tests::new_test_ext(), crate::mock::Test);
