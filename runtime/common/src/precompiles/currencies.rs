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

#![cfg_attr(not(feature = "std"), no_std)]
//use crate::WeightToGas;
use frame_support::pallet_prelude::Get;
use frame_support::traits::{Currency, OriginTrait};
use orml_traits::{BasicCurrency, MultiCurrency as MultiCurrencyTrait};
use pallet_evm::{
	AddressMapping, Context, ExitRevert, ExitSucceed, Precompile, PrecompileFailure, PrecompileHandle, PrecompileOutput,
	PrecompileResult, PrecompileSet,
};
use sp_core::{H160, U256};
use sp_runtime::traits::Dispatchable;
use sp_std::{marker::PhantomData, prelude::*};

use precompile_utils::data::{Address, EvmData, EvmDataWriter};
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

/// Alias for the Balance type for the provided Runtime and Instance.
pub type BalanceOf<Runtime> = <<Runtime as currencies::Config>::MultiSocialCurrency as MultiCurrencyTrait<
	<Runtime as frame_system::Config>::AccountId,
>>::Balance;

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

impl<Runtime> Precompile for MultiCurrencyPrecompile<Runtime>
where
	Runtime: currencies::Config + pallet_evm::Config + frame_system::Config,
	Runtime: Erc20Mapping,
	currencies::Pallet<Runtime>:
		MultiCurrencyTrait<Runtime::AccountId, CurrencyId = FungibleTokenId, Balance = Balance>,
	U256: From<
		<<Runtime as currencies::Config>::MultiSocialCurrency as MultiCurrencyTrait<
			<Runtime as frame_system::Config>::AccountId,
		>>::Balance,
	>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + EvmData,
	<<Runtime as frame_system::Config>::Call as Dispatchable>::Origin: OriginTrait,
{
	fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult {
		let address = handle.code_address();

		if let Some(currency_id) = Runtime::decode_evm_address(address) {
			log::debug!(target: "evm", "multicurrency: currency id: {:?}", currency_id);

			let result = {
				let selector = match handle.read_selector() {
					Ok(selector) => selector,
					Err(e) => return Err(e),
				};

				if let Err(err) = handle.check_function_modifier(match selector {
					Action::Approve | Action::Transfer | Action::TransferFrom => FunctionModifier::NonPayable,
					_ => FunctionModifier::View,
				}) {
					return Err(err);
				}

				match selector {
					// Local and Foreign common
					Action::TotalSupply => Self::total_supply(currency_id, handle),
					Action::BalanceOf => Self::balance_of(currency_id, handle),
					Action::Allowance => Self::total_supply(currency_id, handle),
					Action::Transfer => Self::transfer(currency_id, handle),
					Action::Approve => Self::total_supply(currency_id, handle),
					Action::TransferFrom => Self::total_supply(currency_id, handle),
					Action::Name => Self::total_supply(currency_id, handle),
					Action::Symbol => Self::total_supply(currency_id, handle),
					Action::Decimals => Self::total_supply(currency_id, handle),
				}
			};
		}
		Err(PrecompileFailure::Revert {
			exit_status: ExitRevert::Reverted,
			output: "invalid currency id".into(),
		})
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
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + EvmData,
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

	fn balance_of(currency_id: FungibleTokenId, handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Parse input of index 1 (owner)
		let mut input = handle.read_input()?;
		input.expect_arguments(1)?;

		let owner: H160 = input.read::<Address>()?.into();
		let who: Runtime::AccountId = Runtime::AddressMapping::into_account_id(owner);
		// Fetch info
		let balance = if currency_id == <Runtime as currencies::Config>::GetNativeCurrencyId::get() {
			<Runtime as currencies::Config>::NativeCurrency::free_balance(&who)
		} else {
			<Runtime as currencies::Config>::MultiSocialCurrency::free_balance(currency_id, &who)
		};

		log::debug!(target: "evm", "multicurrency: who: {:?} balance: {:?}", who ,balance);

		let encoded = Output::encode_uint(balance);
		// Build output.
		Ok(succeed(encoded))
	}

	fn transfer(currency_id: FungibleTokenId, handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_log_costs_manual(3, 32)?;

		// Parse input of index 1 (owner)
		let mut input = handle.read_input()?;
		input.expect_arguments(2)?;

		let to: H160 = input.read::<Address>()?.into();
		let amount = input.read::<BalanceOf<Runtime>>()?;

		// Build call info
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let to = Runtime::AddressMapping::into_account_id(to);

		log::debug!(target: "evm", "multicurrency: transfer from: {:?}, to: {:?}, amount: {:?}", origin, to, amount);

		<currencies::Pallet<Runtime> as MultiCurrencyTrait<Runtime::AccountId>>::transfer(
			currency_id,
			&origin,
			&to,
			amount.try_into().ok().unwrap(),
		)
		.map_err(|e| PrecompileFailure::Revert {
			exit_status: ExitRevert::Reverted,
			output: Into::<&str>::into(e).as_bytes().to_vec(),
		})?;

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::precompile::mock::{
		alice, bob, erc20_address_not_exists, neer_evm_address, new_test_ext, nuum_evm_address, Balances, Test,
	};
	use frame_support::assert_noop;
	use hex_literal::hex;

	type MultiCurrencyPrecompile = crate::precompile::currencies::MultiCurrencyPrecompile<Test>;

	#[test]
	fn handles_invalid_currency_id() {
		new_test_ext().execute_with(|| {
			// call with not exists erc20
			let context = Context {
				address: Default::default(),
				caller: erc20_address_not_exists(),
				apparent_value: Default::default(),
			};

			// symbol() -> 0x95d89b41
			let input = hex! {"
				95d89b41
			"};

			assert_noop!(
				MultiCurrencyPrecompile::execute(&input, Some(10_000), &context, false),
				PrecompileFailure::Revert {
					exit_status: ExitRevert::Reverted,
					output: "invalid currency id".into(),
				}
			);
		});
	}

	#[test]
	fn name_works() {
		new_test_ext().execute_with(|| {
			let mut context = Context {
				address: Default::default(),
				caller: Default::default(),
				apparent_value: Default::default(),
			};

			// name() -> 0x06fdde03
			let input = hex! {"
				06fdde03
			"};

			// Token
			context.caller = neer_evm_address();

			let expected_output = hex! {"
				0000000000000000000000000000000000000000000000000000000000000020
				0000000000000000000000000000000000000000000000000000000000000005
				4163616c61000000000000000000000000000000000000000000000000000000
			"};

			let resp = MultiCurrencyPrecompile::execute(&input, None, &context, false).unwrap();
			assert_eq!(resp.exit_status, ExitSucceed::Returned);
			assert_eq!(resp.output, expected_output.to_vec());
		});
	}

	#[test]
	fn decimals_works() {
		new_test_ext().execute_with(|| {
			let mut context = Context {
				address: Default::default(),
				caller: Default::default(),
				apparent_value: Default::default(),
			};

			// decimals() -> 0x313ce567
			let input = hex! {"
				313ce567
			"};

			// Token
			context.caller = neer_evm_address();

			let expected_output = hex! {"
				00000000000000000000000000000000 0000000000000000000000000000000c
			"};

			let resp = MultiCurrencyPrecompile::execute(&input, None, &context, false).unwrap();
			assert_eq!(resp.exit_status, ExitSucceed::Returned);
			assert_eq!(resp.output, expected_output.to_vec());
		});
	}

	#[test]
	fn total_supply_works() {
		new_test_ext().execute_with(|| {
			let mut context = Context {
				address: Default::default(),
				caller: Default::default(),
				apparent_value: Default::default(),
			};

			// totalSupply() -> 0x18160ddd
			let input = hex! {"
				18160ddd
			"};

			// Token
			context.caller = neer_evm_address();

			// 2_000_000_000
			let expected_output = hex! {"
				00000000000000000000000000000000 00000000000000000000000077359400
			"};

			let resp = MultiCurrencyPrecompile::execute(&input, None, &context, false).unwrap();
			assert_eq!(resp.exit_status, ExitSucceed::Returned);
			assert_eq!(resp.output, expected_output.to_vec());
		});
	}

	#[test]
	fn balance_of_works() {
		new_test_ext().execute_with(|| {
			let mut context = Context {
				address: Default::default(),
				caller: Default::default(),
				apparent_value: Default::default(),
			};

			// balanceOf(address) -> 0x70a08231
			// account
			let input = hex! {"
				70a08231
				000000000000000000000000 1000000000000000000000000000000000000001
			"};

			// Token
			context.caller = neer_evm_address();

			// INITIAL_BALANCE = 1_000_000_000_000
			let expected_output = hex! {"
				00000000000000000000000000000000 0000000000000000000000e8d4a51000
			"};

			let resp = MultiCurrencyPrecompile::execute(&input, None, &context, false).unwrap();
			assert_eq!(resp.exit_status, ExitSucceed::Returned);
			assert_eq!(resp.output, expected_output.to_vec());
		})
	}

	#[test]
	fn transfer_works() {
		new_test_ext().execute_with(|| {
			let mut context = Context {
				address: Default::default(),
				caller: Default::default(),
				apparent_value: Default::default(),
			};

			// transfer(address,address,uint256) -> 0xbeabacc8
			// from
			// to
			// amount
			let input = hex! {"
				beabacc8
				000000000000000000000000 1000000000000000000000000000000000000001
				000000000000000000000000 1000000000000000000000000000000000000002
				00000000000000000000000000000000 00000000000000000000000000000001
			"};

			let from_balance = Balances::free_balance(alice());
			let to_balance = Balances::free_balance(bob());

			// Token
			context.caller = neer_evm_address();

			let resp = MultiCurrencyPrecompile::execute(&input, None, &context, false).unwrap();
			assert_eq!(resp.exit_status, ExitSucceed::Returned);
			assert_eq!(resp.output, [0u8; 0].to_vec());

			assert_eq!(Balances::free_balance(alice()), from_balance - 1);
			assert_eq!(Balances::free_balance(bob()), to_balance + 1);
		})
	}
}
