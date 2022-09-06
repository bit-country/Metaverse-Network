#![cfg(feature = "runtime-benchmarks")]

use super::*;
use sp_std::prelude::*;
use sp_std::vec;

#[allow(unused)]
pub use crate::Pallet as EvmMappingModule;
use crate::{Call, Config, EcdsaSignature};
// use crate::Mining as MiningModule;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
//use frame_support::traits::{Currency as PalletCurrency, Get};
use frame_system::{Pallet as System, RawOrigin};
use primitives::{Balance, EvmAddress};
use sp_runtime::traits::{AccountIdConversion, Lookup, StaticLookup, UniqueSaturatedInto};
use sp_io::hashing::keccak_256;

pub type AccountId = u128;

const SEED: u32 = 0;

fn dollar(d: u32) -> Balance {
	let d: Balance = d.into();
	d.saturating_mul(1_000_000_000_000_000_000)
}

fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	T::Currency::deposit_into_existing(&caller, dollar(100).unique_saturated_into());
	caller
}

fn alice() -> libsecp256k1::SecretKey {
	libsecp256k1::SecretKey::parse(&keccak_256(b"Alice")).unwrap()
}

fn bob() -> libsecp256k1::SecretKey {
	libsecp256k1::SecretKey::parse(&keccak_256(b"Bob")).unwrap()
}
/* 
fn bob_account_id() -> T::AccountId {
	let address = crate::Pallet::<T>::eth_address(&bob());
	let mut data = [0u8; 32];
	data[0..4].copy_from_slice(b"evm:");
	data[4..24].copy_from_slice(&address[..]);
	T::AccountId::from(Into::<[u8; 32]>::into(data))
}

fn funded_bob<T: Config>() -> T::AccountId {
	let caller: T::AccountId = bob_account_id();
	T::Currency::deposit_into_existing(&caller, dollar(100).unique_saturated_into());
	caller
}
*/

benchmarks! {
	claim_eth_account {
		let caller = funded_account::<T>("caller", 0);
		let eth = funded_account::<T>("eth", 1);
		//let bob = funded_bob::<T>();
		let evm_address = crate::Pallet::<T>::eth_address(&alice());
		let evm_signature = crate::Pallet::<T>::eth_sign(&alice(), &caller);
	}: _(RawOrigin::Signed(caller), evm_address, evm_signature)

	claim_default_account {
		let caller = funded_account::<T>("caller", 0);
	}: _(RawOrigin::Signed(caller))
}

impl_benchmark_test_suite!(Pallet, crate::benchmarking::tests::new_test_ext(), crate::mock::Test);
