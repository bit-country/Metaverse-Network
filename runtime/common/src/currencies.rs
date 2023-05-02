use frame_support::pallet_prelude::Get;
use frame_support::traits::{Currency, OriginTrait};
use orml_traits::{BasicCurrency, MultiCurrency as MultiCurrencyTrait};
use pallet_evm::Context;
use pallet_evm::{
	AddressMapping, ExitRevert, ExitSucceed, Precompile, PrecompileFailure, PrecompileHandle, PrecompileOutput,
	PrecompileResult, PrecompileSet,
};
use sp_core::{H160, U256};
use sp_runtime::traits::{AccountIdConversion, Dispatchable, Zero};
use sp_std::{marker::PhantomData, prelude::*};

use evm_mapping::AddressMapping as EvmMapping;
use evm_mapping::EvmAddressMapping;
use precompile_utils::data::{Address, EvmData, EvmDataWriter};
use precompile_utils::handle::PrecompileHandleExt;
use precompile_utils::modifier::FunctionModifier;
use precompile_utils::prelude::RuntimeHelper;
use precompile_utils::{succeed, EvmResult};
use primitives::evm::{Erc20Mapping, Output};
use primitives::{evm, Balance, FungibleTokenId, AssetIds, AssetMetadata};

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
pub type BalanceOf<Runtime> = <<Runtime as currencies_pallet::Config>::MultiSocialCurrency as MultiCurrencyTrait<
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

impl<Runtime> Default for MultiCurrencyPrecompile<Runtime> {
	fn default() -> Self {
		Self(PhantomData)
	}
}
#[cfg(test)]
impl<Runtime> PrecompileSet for MultiCurrencyPrecompile<Runtime>
where
	Runtime: currencies_pallet::Config + pallet_evm::Config + frame_system::Config + evm_mapping::Config + asset_manager::Config,
	Runtime: Erc20Mapping,
	currencies_pallet::Pallet<Runtime>:
		MultiCurrencyTrait<Runtime::AccountId, CurrencyId = FungibleTokenId, Balance = Balance>,
	U256: From<
		<<Runtime as currencies_pallet::Config>::MultiSocialCurrency as MultiCurrencyTrait<
			<Runtime as frame_system::Config>::AccountId,
		>>::Balance,
	>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + EvmData,
	<<Runtime as frame_system::Config>::Call as Dispatchable>::Origin: OriginTrait,
{
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<EvmResult<PrecompileOutput>> {
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
			return Some(result);
		}
		Some(Err(PrecompileFailure::Revert {
			exit_status: ExitRevert::Reverted,
			output: "invalid currency id".into(),
		}))
	}

	fn is_precompile(&self, address: H160) -> bool {
		todo!()
	}
}

impl<Runtime> Precompile for MultiCurrencyPrecompile<Runtime>
where
	Runtime: currencies_pallet::Config + pallet_evm::Config + frame_system::Config + evm_mapping::Config + asset_manager::Config,
	Runtime: Erc20Mapping,
	currencies_pallet::Pallet<Runtime>:
		MultiCurrencyTrait<Runtime::AccountId, CurrencyId = FungibleTokenId, Balance = Balance>,
	U256: From<
		<<Runtime as currencies_pallet::Config>::MultiSocialCurrency as MultiCurrencyTrait<
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
					Action::Allowance => Self::not_supported(currency_id, handle),
					Action::Transfer => Self::transfer(currency_id, handle),
					Action::Approve => Self::not_supported(currency_id, handle),
					Action::TransferFrom => Self::not_supported(currency_id, handle),
					Action::Name => Self::name(currency_id, handle),
					Action::Symbol => Self::symbol(currency_id, handle),
					Action::Decimals => Self::decimals(currency_id, handle),
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
	Runtime: currencies_pallet::Config + pallet_evm::Config + frame_system::Config + evm_mapping::Config + asset_manager::Config,
	currencies_pallet::Pallet<Runtime>:
		MultiCurrencyTrait<Runtime::AccountId, CurrencyId = FungibleTokenId, Balance = Balance>,
	U256: From<
		<<Runtime as currencies_pallet::Config>::MultiSocialCurrency as MultiCurrencyTrait<
			<Runtime as frame_system::Config>::AccountId,
		>>::Balance,
	>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + EvmData,
{
	fn not_supported(currency_id: FungibleTokenId, handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		Err(PrecompileFailure::Revert {
			exit_status: ExitRevert::Reverted,
			output: "not supported".as_bytes().to_vec(),
		})
	}

	fn name(currency_id: FungibleTokenId, handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Parse input
		let input = handle.read_input()?;
		input.expect_arguments(0)?;

		let asset_metadata_result = <asset_manager::Pallet<Runtime>>::asset_metadatas(AssetIds::NativeAssetId(currency_id));
		match asset_metadata_result{ 
			Some(asset_metadata) => {
				log::debug!(target: "evm", "multicurrency: name: {:?}", asset_metadata.name);

				let encoded = Output::encode_bytes(asset_metadata.name.as_slice());
				// Build output.
				Ok(succeed(encoded))
			}
			None => Err(PrecompileFailure::Error {
				exit_status: pallet_evm::ExitError::Other("Non-existing asset.".into()),
			})
		}
	}

	fn symbol(currency_id: FungibleTokenId, handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Parse input
		let input = handle.read_input()?;
		input.expect_arguments(0)?;

		let asset_metadata_result = <asset_manager::Pallet<Runtime>>::asset_metadatas(AssetIds::NativeAssetId(currency_id));

		match asset_metadata_result{ 
			Some(asset_metadata) => {
				log::debug!(target: "evm", "multicurrency: symbol: {:?}", asset_metadata.symbol);

				let encoded = Output::encode_bytes(asset_metadata.symbol.as_slice());
				// Build output.
				Ok(succeed(encoded))
			}
			None => Err(PrecompileFailure::Error {
				exit_status: pallet_evm::ExitError::Other("Non-existing asset.".into()),
			})
		}
	}

	fn decimals(currency_id: FungibleTokenId, handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Parse input
		let input = handle.read_input()?;
		input.expect_arguments(0)?;

		let asset_metadata_result = <asset_manager::Pallet<Runtime>>::asset_metadatas(AssetIds::NativeAssetId(currency_id));
		
		match asset_metadata_result{ 
			Some(asset_metadata) => {
				log::debug!(target: "evm", "multicurrency: decimals: {:?}", asset_metadata.decimals);

				let encoded = Output::encode_uint(asset_metadata.decimals.into());
				// Build output.
				Ok(succeed(encoded))
			}
			None => Err(PrecompileFailure::Error {
				exit_status: pallet_evm::ExitError::Other("Non-existing asset.".into()),
			})
		}
		
	}

	fn total_supply(currency_id: FungibleTokenId, handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Parse input
		let input = handle.read_input()?;
		input.expect_arguments(0)?;

		let total_issuance = if currency_id == <Runtime as currencies_pallet::Config>::GetNativeCurrencyId::get() {
			<Runtime as currencies_pallet::Config>::NativeCurrency::total_issuance()
		} else {
			<Runtime as currencies_pallet::Config>::MultiSocialCurrency::total_issuance(currency_id)
		};

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

		let who: Runtime::AccountId = <Runtime as evm_mapping::Config>::AddressMapping::get_account_id(&owner);
		// Fetch info
		let balance = if currency_id == <Runtime as currencies_pallet::Config>::GetNativeCurrencyId::get() {
			<Runtime as currencies_pallet::Config>::NativeCurrency::free_balance(&who)
		} else {
			<Runtime as currencies_pallet::Config>::MultiSocialCurrency::free_balance(currency_id, &who)
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
		let origin = <Runtime as evm_mapping::Config>::AddressMapping::get_account_id(&handle.context().caller);
		let to = <Runtime as evm_mapping::Config>::AddressMapping::get_account_id(&to);

		log::debug!(target: "evm", "multicurrency: transfer from: {:?}, to: {:?}, amount: {:?}", origin, to, amount);

		<currencies_pallet::Pallet<Runtime> as MultiCurrencyTrait<Runtime::AccountId>>::transfer(
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
