use frame_support::assert_noop;
use hex_literal::hex;
use sp_core::{ByteArray, H160, U256};

use precompile_utils::data::{Address, EvmDataWriter};
use precompile_utils::testing::*;
use primitives::FungibleTokenId;

use crate::currencies::Action;
use crate::mock::*;
use orml_traits::BasicCurrency;
use orml_traits::MultiCurrency;
use pallet_evm::AddressMapping;
//use currencies_pallet::MultiSocialCurrency;

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

// Nft Precompile Tests
