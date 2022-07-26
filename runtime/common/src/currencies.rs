#![cfg_attr(not(feature = "std"), no_std)]

use orml_traits::MultiCurrency as MultiCurrencyTrait;
use pallet_evm::{PrecompileHandle, PrecompileOutput, PrecompileResult, PrecompileSet};
use sp_core::H160;
use sp_std::{marker::PhantomData, prelude::*};

use precompile_utils::data::EvmDataWriter;
use precompile_utils::handle::PrecompileHandleExt;
use precompile_utils::modifier::FunctionModifier;
use precompile_utils::prelude::RuntimeHelper;
use precompile_utils::{
	keccak256, succeed, Address, Bytes, EvmData, EvmDataWriter, EvmResult, FunctionModifier, LogExt, LogsBuilder,
	PrecompileHandleExt, RuntimeHelper,
};
use primitives::evm::Erc20Mapping;
use primitives::{evm, Balance, FungibleTokenId};

#[precompile_utils_macro::generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
	QueryName = "name()",
	QuerySymbol = "symbol()",
	QueryDecimals = "decimals()",
	QueryTotalIssuance = "totalSupply()",
	QueryBalance = "balanceOf(address)",
	Transfer = "transfer(address,address,uint256)",
}

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
	Runtime: Erc20Mapping,
	currencies::Pallet<Runtime>:
		MultiCurrencyTrait<Runtime::AccountId, CurrencyId = FungibleTokenId, Balance = Balance>,
{
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
		let address = handle.code_address();

		if let Some(currency_id) = Runtime::decode_evm_address(address) {
			log::debug!(target: "evm", "multicurrency: currency id: {:?}", currency_id);

			let result = {
				let selector = match handle.read_selector() {
					Ok(selector) => selector,
					Err(e) => return Some(Err(e)),
				};

				if let Err(err) = handle.check_function_modifier(match selector {
					Action::Approve | Action::Transfer | Action::TransferFrom => FunctionModifier::NonPayable,
					_ => FunctionModifier::View,
				}) {
					return Some(Err(err));
				}

				match selector {
					// Local and Foreign common
					Action::TotalSupply => Self::total_supply(currency_id, handle),
					//					Action::BalanceOf => Self::balance_of(currency_id, handle),
					//					Action::Allowance => Self::allowance(asset_id, handle),
					//					Action::Approve => Self::approve(asset_id, handle),
					//					Action::Transfer => Self::transfer(currency_id, handle),
					//					Action::TransferFrom => Self::transfer_from(currency_id, handle),
				};
			};

			Some(result)
		}
		None
	}

	fn is_precompile(&self, address: H160) -> bool {
		todo!()
	}
}

impl<Runtime> MultiCurrencyPrecompile<Runtime>
where
	Runtime: currencies::Config + pallet_evm::Config + frame_system::Config,
{
	fn total_supply(currency_id: FungibleTokenId, handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Parse input
		let input = handle.read_input()?;
		input.expect_arguments(0)?;

		// Fetch info
		let total_issuance = currencies::Pallet::<Runtime>::total_issuance(currency_id).into();

		// Build output
		Ok(succeed(EvmDataWriter::new().write(total_issuance).build()))
	}
}
