#![cfg(feature = "runtime-benchmarks")]

use super::*;
use sp_std::prelude::*;
use sp_std::vec::*;

#[allow(unused)]
pub use crate::Pallet as EmergencyModule;
use crate::{Call, Config};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::{Pallet as System, RawOrigin};
use sp_runtime::traits::{AccountIdConversion, Lookup, StaticLookup, UniqueSaturatedInto};

pub type AccountId = u128;

const SEED: u32 = 0;

benchmarks! {
	emergency_stop{
		let pallet_name: Vec<u8> = "pallet_estate".as_bytes().to_vec();
		let function_name: Vec<u8> = "mint_land".as_bytes().to_vec();
	}: _(RawOrigin::Root, pallet_name, function_name)

	emergency_unstop {
		let pallet_name: Vec<u8> = "pallet_estate".as_bytes().to_vec();
		let function_name: Vec<u8> = "mint_land".as_bytes().to_vec();
		crate::Pallet::<T>::emergency_stop(RawOrigin::Root.into(), pallet_name.clone(), function_name.clone());
	}: _(RawOrigin::Root, pallet_name, function_name)

}
impl_benchmark_test_suite!(Pallet, crate::benchmarking::tests::new_test_ext(), crate::mock::Test);
