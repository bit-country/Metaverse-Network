// This file is part of Metaverse.Network & Bit.Country.

// Copyright (C) 2020-2022 Metaverse.Network & Bit.Country .
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// mod tests;
pub mod mock;

use frame_support::log;
use hex_literal::hex;
use pallet_evm::{
	Context, ExitRevert, Precompile, PrecompileFailure, PrecompileHandle, PrecompileResult, PrecompileSet,
};
use pallet_evm_precompile_blake2::Blake2F;
use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_dispatch::Dispatch;
use pallet_evm_precompile_ed25519::Ed25519Verify;
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};
use sp_core::H160;
use sp_std::{collections::btree_set::BTreeSet, fmt::Debug, marker::PhantomData};

pub mod currencies;


pub use currencies::MultiCurrencyPrecompile;

pub const ECRECOVER: H160 = H160(hex!("0000000000000000000000000000000000000001"));
pub const SHA256: H160 = H160(hex!("0000000000000000000000000000000000000002"));
pub const RIPEMD: H160 = H160(hex!("0000000000000000000000000000000000000003"));
pub const IDENTITY: H160 = H160(hex!("0000000000000000000000000000000000000004"));
pub const MODEXP: H160 = H160(hex!("0000000000000000000000000000000000000005"));
pub const BN_ADD: H160 = H160(hex!("0000000000000000000000000000000000000006"));
pub const BN_MUL: H160 = H160(hex!("0000000000000000000000000000000000000007"));
pub const BN_PAIRING: H160 = H160(hex!("0000000000000000000000000000000000000008"));
pub const BLAKE2F: H160 = H160(hex!("0000000000000000000000000000000000000009"));

pub const ETH_PRECOMPILE_END: H160 = BLAKE2F;

pub const ECRECOVER_PUBLICKEY: H160 = H160(hex!("0000000000000000000000000000000000000080"));
pub const SHA3_256: H160 = H160(hex!("0000000000000000000000000000000000000081"));
pub const SHA3_512: H160 = H160(hex!("0000000000000000000000000000000000000082"));

pub const MULTI_CURRENCY: H160 = H160(hex!("0000000000000000000000000000000000000400"));
/// The PrecompileSet installed in the Metaverse runtime.
#[derive(Debug, Default, Clone, Copy)]
pub struct MetaverseNetworkPrecompiles<R>(PhantomData<(R)>);

impl<R> MetaverseNetworkPrecompiles<R> {
	pub fn new() -> Self {
		Self(Default::default())
	}
}

pub fn target_gas_limit(target_gas: Option<u64>) -> Option<u64> {
	target_gas.map(|x| x.saturating_div(10).saturating_mul(9)) // 90%
}

pub struct AllPrecompiles<R> {
	active: BTreeSet<H160>,
	_marker: PhantomData<R>,
}

impl<R> AllPrecompiles<R>
where
	R: pallet_evm::Config,
{
	pub fn continuum() -> Self {
		Self {
			active: BTreeSet::from([
				ECRECOVER,
				SHA256,
				RIPEMD,
				IDENTITY,
				MODEXP,
				BN_ADD,
				BN_MUL,
				BN_PAIRING,
				BLAKE2F,
				// Non-standard precompile starts with 128
				ECRECOVER_PUBLICKEY,
				SHA3_256,
				SHA3_512,
				MULTI_CURRENCY,
			]),
			_marker: Default::default(),
		}
	}

	pub fn pioneer() -> Self {
		Self {
			active: BTreeSet::from([
				ECRECOVER,
				SHA256,
				RIPEMD,
				IDENTITY,
				MODEXP,
				BN_ADD,
				BN_MUL,
				BN_PAIRING,
				BLAKE2F,
				// Non-standard precompile starts with 128
				ECRECOVER_PUBLICKEY,
				SHA3_256,
				SHA3_512,
				MULTI_CURRENCY,
			]),
			_marker: Default::default(),
		}
	}

	pub fn metaverse() -> Self {
		Self {
			active: BTreeSet::from([
				ECRECOVER,
				SHA256,
				RIPEMD,
				IDENTITY,
				MODEXP,
				BN_ADD,
				BN_MUL,
				BN_PAIRING,
				BLAKE2F,
				// Non-standard precompile starts with 128
				ECRECOVER_PUBLICKEY,
				SHA3_256,
				SHA3_512,
				MULTI_CURRENCY,
			]),
			_marker: Default::default(),
		}
	}
}

impl<R> PrecompileSet for AllPrecompiles<R>
where
	R: pallet_evm::Config,
	MultiCurrencyPrecompile<R>: Precompile,
	Dispatch<R>: Precompile,
{
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult>
/*
    fn execute(
		&self,
		address: H160,
		input: &[u8],
		target_gas: Option<u64>,
		context: &Context,
		is_static: bool,
	) -> Option<PrecompileResult> */ {
		let address = handle.code_address();
		if !self.is_precompile(address) {
			return None;
		}

		if self.is_precompile(address) && address > hash(9) && handle.context().address != address {
			return Some(Err(PrecompileFailure::Revert {
				exit_status: ExitRevert::Reverted,
				output: b"cannot be called with DELEGATECALL or CALLCODE".to_vec(),
			}));
		}

		//log::trace!(target: "evm", "Precompile begin, address: {:?}, input: {:?}, target_gas: {:?},
		// context: {:?}", address, input, target_gas, context);

		match address {
			// Ethereum precompiles :
			a if a == hash(1) => Some(ECRecover::execute(handle)),
			a if a == hash(2) => Some(Sha256::execute(handle)),
			a if a == hash(3) => Some(Ripemd160::execute(handle)),
			a if a == hash(4) => Some(Identity::execute(handle)),
			a if a == hash(5) => Some(Modexp::execute(handle)),
			a if a == hash(6) => Some(Bn128Add::execute(handle)),
			a if a == hash(7) => Some(Bn128Mul::execute(handle)),
			a if a == hash(8) => Some(Bn128Pairing::execute(handle)),
			a if a == hash(9) => Some(Blake2F::execute(handle)),
			// nor Ethereum precompiles :
			a if a == hash(1024) => Some(Sha3FIPS256::execute(handle)),
			a if a == hash(1025) => Some(Dispatch::<R>::execute(handle)),
			a if a == hash(1026) => Some(ECRecoverPublicKey::execute(handle)),
			a if a == hash(1027) => Some(Ed25519Verify::execute(handle)),
			// Metaverse Network precompiles (starts from 0x5000):
			// If the address matches asset prefix, the we route through the asset precompile set
			a if a == MULTI_CURRENCY => Some(MultiCurrencyPrecompile::<R>::execute(handle)),
			// Default
			_ => None,
		}
		/*
			// https://github.com/ethereum/go-ethereum/blob/9357280fce5c5d57111d690a336cca5f89e34da6/core/vm/contracts.go#L83
			let result = if address == ECRECOVER {
				Some(ECRecover::execute(input, target_gas, context, is_static))
			} else if address == SHA256 {
				Some(Sha256::execute(input, target_gas, context, is_static))
			} else if address == RIPEMD {
				Some(Ripemd160::execute(input, target_gas, context, is_static))
			} else if address == IDENTITY {
				Some(Identity::execute(input, target_gas, context, is_static))
			} else if address == MODEXP {
				if R::config().increase_state_access_gas {
					Some(Modexp::execute(input, target_gas, context, is_static))
				} else {
					Some(IstanbulModexp::execute(input, target_gas, context, is_static))
				}
			} else if address == BN_ADD {
				Some(Bn128Add::execute(input, target_gas, context, is_static))
			} else if address == BN_MUL {
				Some(Bn128Mul::execute(input, target_gas, context, is_static))
			} else if address == BN_PAIRING {
				Some(Bn128Pairing::execute(input, target_gas, context, is_static))
			} else if address == BLAKE2F {
				Some(Blake2F::execute(input, target_gas, context, is_static))
			}
			// Non-standard precompile starts with 128
			else if address == ECRECOVER_PUBLICKEY {
				Some(ECRecoverPublicKey::execute(input, target_gas, context, is_static))
			} else if address == SHA3_256 {
				Some(Sha3FIPS256::execute(input, target_gas, context, is_static))
			} else if address == SHA3_512 {
				Some(Sha3FIPS512::execute(input, target_gas, context, is_static))
			}
			else {
				if !pallet_evm::Pallet::<R>::is_contract(&context.caller) {
					log::debug!(target: "evm", "Caller is not a system contract: {:?}", context.caller);
					return Some(Err(PrecompileFailure::Revert {
						exit_status: ExitRevert::Reverted,
						output: "Caller is not a system contract".into(),
						cost: target_gas.unwrap_or_default(),
					}));
				}

				if address == MULTI_CURRENCY {
					Some(MultiCurrencyPrecompile::<R>::execute(
						input, target_gas, context, is_static,
					))
				} else {
					None
				}
			};

			log::trace!(target: "evm", "Precompile end, address: {:?}, input: {:?}, target_gas: {:?}, context: {:?}, result: {:?}", address, input, target_gas, context, result);
			if let Some(Err(PrecompileFailure::Revert { ref output, .. })) = result {
				log::debug!(target: "evm", "Precompile failed: {:?}", core::str::from_utf8(output));
			};
			 result
		}
		*/
	}

	fn is_precompile(&self, address: H160) -> bool {
		if address != hash(1) && address != hash(2) && address != hash(3) 
			&& address != hash(4) && address != hash(5) && address != hash(6) 
			&& address != hash(7) && address != hash(8) && address != hash(9) 
			&& address != hash(1024) && address != hash(1025) && address != hash(1026) 
			&& address != hash(1027) && address != MULTI_CURRENCY {
				return false;
		}
		true
	}
}

fn hash(a: u64) -> H160 {
	H160::from_low_u64_be(a)
}

#[test]
fn ensure_precompile_address_start() {
	use primitives::evm::PRECOMPILE_ADDRESS_START;
	assert_eq!(PRECOMPILE_ADDRESS_START, MULTI_CURRENCY);
}
