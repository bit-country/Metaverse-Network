use orml_traits::MultiCurrency as MultiCurrencyTrait;
use pallet_evm::{PrecompileHandle, PrecompileSet};
use primitives::{Balance, FungibleTokenId};
use sp_core::H160;
use sp_std::{marker::PhantomData, prelude::*};

/// The `MultiCurrency` impl precompile.
///
///
/// `input` data starts with `action` and `currency_id`.
///
/// Actions:
/// - Query total issuance.
/// - Query balance. Rest `input` bytes: `account_id`.
/// - Transfer. Rest `input` bytes: `from`, `to`, `amount`.
pub struct MultiCurrencyPrecompile<Runtime>(PhantomData<Runtime>);

impl<Runtime> PrecompileSet for MultiCurrencyPrecompile<Runtime>
where
	Runtime: currencies::Config + pallet_evm::Config,
	currencies::Pallet<Runtime>:
		MultiCurrencyTrait<Runtime::AccountId, CurrencyId = FungibleTokenId, Balance = Balance>,
{
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<evm::executor::stack::executor::PrecompileResult> {
		todo!()
	}

	fn is_precompile(&self, address: H160) -> bool {
		todo!()
	}
}
