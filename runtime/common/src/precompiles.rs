use crate::currencies::MultiCurrencyPrecompile;
use hex_literal::hex;
use pallet_evm::{ExitRevert, Precompile, PrecompileFailure, PrecompileHandle, PrecompileResult, PrecompileSet};
use pallet_evm_precompile_blake2::Blake2F;
use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_dispatch::Dispatch;
use pallet_evm_precompile_ed25519::Ed25519Verify;
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};
use sp_core::H160;
use sp_std::fmt::Debug;
use sp_std::marker::PhantomData;

pub const MULTI_CURRENCY: H160 = H160(hex!("0000000000000000000000000000000000000400"));
/// The PrecompileSet installed in the Metaverse runtime.
#[derive(Debug, Default, Clone, Copy)]
pub struct MetaverseNetworkPrecompiles<R>(PhantomData<(R)>);

impl<R> MetaverseNetworkPrecompiles<R> {
	pub fn new() -> Self {
		Self(Default::default())
	}
}

/// The following distribution has been decided for the precompiles
/// 0-1023: Ethereum Mainnet Precompiles
/// 1024-2047 Precompiles that are not in Ethereum Mainnet
impl<R> PrecompileSet for MetaverseNetworkPrecompiles<R>
where
	R: pallet_evm::Config,
	MultiCurrencyPrecompile<R>: Precompile,
	Dispatch<R>: Precompile,
{
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
		let address = handle.code_address();

		if self.is_precompile(address) && address > hash(9) && handle.context().address != address {
			return Some(Err(PrecompileFailure::Revert {
				exit_status: ExitRevert::Reverted,
				output: b"cannot be called with DELEGATECALL or CALLCODE".to_vec(),
			}));
		}

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
	}

	fn is_precompile(&self, address: H160) -> bool {
		todo!()
	}
}

fn hash(a: u64) -> H160 {
	H160::from_low_u64_be(a)
}
