#![cfg(feature = "runtime-benchmarks")]

use super::*;
use sp_std::prelude::*;
use sp_std::vec::*;

#[allow(unused)]
pub use crate::Pallet as NftMigrationModule;
use crate::{Call, Config};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::{Pallet as System, RawOrigin};
use sp_runtime::traits::{AccountIdConversion, Lookup, StaticLookup, UniqueSaturatedInto};

const SEED: u32 = 0;

benchmarks! {

	start_migration {
	}: _(RawOrigin::Root)

}
impl_benchmark_test_suite!(Pallet, crate::benchmarking::tests::new_test_ext(), crate::mock::Test);
