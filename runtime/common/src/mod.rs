pub mod mock;
//smod tests;

use frame_support::log;
use hex_literal::hex;
use sp_core::H160;
use pallet_evm::{
	precompiles::{
		Blake2F, Bn128Add, Bn128Mul, Bn128Pairing, ECRecover, ECRecoverPublicKey, Identity, IstanbulModexp, Modexp,
		Precompile, Ripemd160, Sha256, Sha3FIPS256, Sha3FIPS512,
	},
	runner::state::{PrecompileFailure, PrecompileResult, PrecompileSet},
	Context, ExitRevert,
};
use sp_std::{collections::btree_set::BTreeSet, marker::PhantomData};

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
	R: module_evm::Config,
	MultiCurrencyPrecompile<R>: Precompile,
{
    fn execute(
		&self,
		address: H160,
		input: &[u8],
		target_gas: Option<u64>,
		context: &Context,
		is_static: bool,
	) -> Option<PrecompileResult> {
		if !self.is_precompile(address) {
			return None;
		}

		// Filter known precompile addresses except Ethereum officials
		if address > ETH_PRECOMPILE_END && context.address != address {
			return Some(Err(PrecompileFailure::Revert {
				exit_status: ExitRevert::Reverted,
				output: "cannot be called with DELEGATECALL or CALLCODE".into(),
				cost: target_gas.unwrap_or_default(),
			}));
		}

		log::trace!(target: "evm", "Precompile begin, address: {:?}, input: {:?}, target_gas: {:?}, context: {:?}", address, input, target_gas, context);

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
}

#[test]
fn ensure_precompile_address_start() {
	use primitives::evm::PRECOMPILE_ADDRESS_START;
	assert_eq!(PRECOMPILE_ADDRESS_START, MULTI_CURRENCY);
}
