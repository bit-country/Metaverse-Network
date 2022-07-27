#![cfg_attr(not(feature = "std"), no_std)]

use orml_traits::MultiCurrency as MultiCurrencyTrait;
use pallet_evm::{ExitSucceed, PrecompileHandle, PrecompileOutput, PrecompileResult, PrecompileSet};
use sp_core::{H160, U256};
use sp_std::{marker::PhantomData, prelude::*};

use precompile_utils::data::{EvmData, EvmDataWriter};
use precompile_utils::handle::PrecompileHandleExt;
use precompile_utils::modifier::FunctionModifier;
use precompile_utils::prelude::RuntimeHelper;
use precompile_utils::{succeed, EvmResult};
use primitives::evm::{Erc20Mapping, Output};
use primitives::{evm, Balance, FungibleTokenId};

#[precompile_utils_macro::generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
	TotalSupply = "totalSupply()",
	BalanceOf = "balanceOf(address)",
	Allowance = "allowance(address,address)",
	Transfer = "transfer(address,uint256)",
	Approve = "approve(address,uint256)",
	TransferFrom = "transferFrom(address,address,uint256)",
	Name = "name()",
	Symbol = "symbol()",
	Decimals = "decimals()",
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
	U256: From<
		<<Runtime as currencies::Config>::MultiSocialCurrency as MultiCurrencyTrait<
			<Runtime as frame_system::Config>::AccountId,
		>>::Balance,
	>,
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
					Action::BalanceOf => Self::total_supply(currency_id, handle),
					Action::Allowance => Self::total_supply(currency_id, handle),
					Action::Transfer => Self::total_supply(currency_id, handle),
					Action::Approve => Self::total_supply(currency_id, handle),
					Action::TransferFrom => Self::total_supply(currency_id, handle),
					Action::Name => Self::total_supply(currency_id, handle),
					Action::Symbol => Self::total_supply(currency_id, handle),
					Action::Decimals => Self::total_supply(currency_id, handle),
				}
			};
			return Some(result);
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
	currencies::Pallet<Runtime>:
		MultiCurrencyTrait<Runtime::AccountId, CurrencyId = FungibleTokenId, Balance = Balance>,
	U256: From<
		<<Runtime as currencies::Config>::MultiSocialCurrency as MultiCurrencyTrait<
			<Runtime as frame_system::Config>::AccountId,
		>>::Balance,
	>,
{
	fn total_supply(currency_id: FungibleTokenId, handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Parse input
		let input = handle.read_input()?;
		input.expect_arguments(0)?;

		// Fetch info
		let total_issuance = <Runtime as currencies::Config>::MultiSocialCurrency::total_issuance(currency_id);

		log::debug!(target: "evm", "multicurrency: total issuance: {:?}", total_issuance);

		let encoded = Output::encode_uint(total_issuance);
		// Build output.
		Ok(succeed(encoded))
	}
}
