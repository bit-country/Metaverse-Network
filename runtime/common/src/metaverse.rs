
use frame_support::pallet_prelude::Get;
use frame_support::traits::{Currency, OriginTrait};
use frame_system::RawOrigin;
use orml_traits::{BasicCurrency, MultiCurrency};
use pallet_evm::{
	AddressMapping, ExitRevert, ExitSucceed, Precompile, PrecompileFailure, PrecompileHandle, PrecompileOutput,
	PrecompileResult, PrecompileSet,
};
use sp_core::{H160, U256};
use sp_runtime::traits::{AccountIdConversion, Dispatchable};
use sp_std::{marker::PhantomData, prelude::*};
use core_primitives::MetaverseMetadata;

use precompile_utils::data::{Address, EvmData, EvmDataWriter};
use precompile_utils::handle::PrecompileHandleExt;
use precompile_utils::modifier::FunctionModifier;
use precompile_utils::prelude::RuntimeHelper;
use precompile_utils::{succeed, EvmResult};
use primitives::evm::{Erc20Mapping, Output};
use primitives::{evm, Balance, MetaverseId}; 

#[precompile_utils_macro::generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
    GetMetaverse = "getMetaverse()",
    //GetMetaverseOwner = "getMetaverseOwner()",
    GetMetaverseFundBalance = "getMetaverseFundBalance()",
	CreateMetaverse = "createMetaverse()",
	WithdrawFromMetaverseFund = "withdrawFromMetaverseFund()",
    //TransferMetaverse = "transferMetaverse()",
    //UpdateMetaverseListingFee = "updateMetaverseListingFee()",
}

//Alias for the Balance type for the provided Runtime and Instance.
//pub type BalanceOf<Runtime> = <<Runtime as metaverse::Config>::Currency as Trait>::Balance;

/// The `Metaverse` impl precompile.
///
///
/// `input` data starts with `action` and `metaverse_id`.
///
/// Actions:
/// - Get metaverse info.
/// - Get metaverse owner.
/// - Get metaverse fund balance.
/// - Create metaverse.
/// - Transfer metaverse. Rest `input` bytes: `from`, `to`, `meatverse_id`.
/// - Withdraw from metaverse fund. Rest `input` bytes: `meatverse_id`.
/// - Update metaverse listing fee. Rest `input` bytes: `meatverse_id`, `new_fee`.
pub struct MetaversePrecompile<Runtime>(PhantomData<Runtime>);

impl<Runtime> Precompile for MetaversePrecompile<Runtime>
where
	Runtime: metaverse::Config + pallet_evm::Config + frame_system::Config,
	Runtime: Erc20Mapping,
	U256: From<<<Runtime as metaverse::Config>::Currency as frame_support::traits::Currency<<Runtime as frame_system::Config>::AccountId>>::Balance>,
	//BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + EvmData,
	<<Runtime as frame_system::Config>::Call as Dispatchable>::Origin: OriginTrait,
{
    fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult {
		let result = {
			let selector = match handle.read_selector() {
				Ok(selector) => selector,
				Err(e) => return Err(e),
			};

			if let Err(err) = handle.check_function_modifier(match selector {
				Action::CreateMetaverse| Action::WithdrawFromMetaverseFund => FunctionModifier::NonPayable,
				_ => FunctionModifier::View,
			}) {
				return Err(err);
			}

			match selector {
				// Local and Foreign common
				Action::GetMetaverse => Self::metaverse_info(handle),
				//Action::GetMetaverseOwner => Self::metaverse_info(handle),
				Action::GetMetaverseFundBalance => Self::fund_balance(handle),
				Action::CreateMetaverse => Self::create_metaverse(handle),
				Action::WithdrawFromMetaverseFund => Self::withdraw_funds(handle),
				//Action::TransferMetaverse => Self::transfer(metaverse_id, handle),
				//Action::UpdateMetaverseListingFee => Self::transfer(metaverse_id, handle),
			}
		};
		Err(PrecompileFailure::Revert {
			exit_status: ExitRevert::Reverted,
			output: "invalid metaverse action".into(),
		})
    }
}

impl<Runtime> MetaversePrecompile<Runtime>
where
	Runtime: metaverse::Config + pallet_evm::Config + frame_system::Config,
	U256: From<<<Runtime as metaverse::Config>::Currency as frame_support::traits::Currency<<Runtime as frame_system::Config>::AccountId>>::Balance>,
   // BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + EvmData,
{
	fn metaverse_info(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Parse input of 1 (metaverse_id)
		let mut input = handle.read_input()?;
		input.expect_arguments(1)?;

		let metaverse_id: MetaverseId = input.read::<MetaverseId>()?.into();

		// Fetch info
		let metaverse_info_result = <metaverse::Pallet<Runtime>>::get_metaverse(metaverse_id);

		match metaverse_info_result
		{
			Some(metaverse_info) =>  {
				log::debug!(target: "evm", "metaverse_info: {:?}", metaverse_info);
				let encoded = Output::encode_bytes(&metaverse_info.metadata);
				// Build output.
				Ok(succeed(encoded))
			}
			None => {
				Err(PrecompileFailure::Revert {
					exit_status: ExitRevert::Reverted,
					output: "invalid metaverse id".into(),
				})
			}
		}
		
	}

	fn fund_balance(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Parse input of index 1 (metaverse_id)
		let mut input = handle.read_input()?;
		input.expect_arguments(2)?;

		let metaverse_id: MetaverseId = input.read::<MetaverseId>()?.into();

		let owner: H160 = input.read::<Address>()?.into();
		let who: Runtime::AccountId = Runtime::AddressMapping::into_account_id(owner);
         
        let metaverse_treasury = <Runtime as metaverse::Config>::MetaverseTreasury::get().into_sub_account_truncating(metaverse_id);
		
        // Fetch info
		let balance = <Runtime as metaverse::Config>::Currency::free_balance(&metaverse_treasury);

		log::debug!(target: "evm", "metaverse: {:?} fund balance: {:?}", metaverse_id, balance);

		let encoded = Output::encode_uint(balance);
		// Build output.
		Ok(succeed(encoded))
	}

    fn create_metaverse(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Parse input of index 1 (owner)
		let mut input = handle.read_input()?;
		input.expect_arguments(2)?;

        // Build call info
        let owner: H160 = input.read::<Address>()?.into();
		let who: Runtime::AccountId = Runtime::AddressMapping::into_account_id(owner);
		let metaverse_metadata: MetaverseMetadata = input.read::<MetaverseMetadata>()?.into();

        log::debug!(target: "evm", "create metaverse for: {:?}", who);

        <metaverse::Pallet<Runtime>>::create_metaverse(
			RawOrigin::Signed(who).into(),
			metaverse_metadata
		)
		.map_err(|e| PrecompileFailure::Revert {
			exit_status: ExitRevert::Reverted,
			output: Into::<&str>::into(e).as_bytes().to_vec(),
		})?;

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn withdraw_funds(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Parse input of index 1 (metaverse_id)
		let mut input = handle.read_input()?;
		input.expect_arguments(2)?;

		let metaverse_id: MetaverseId = input.read::<MetaverseId>()?.into();

        // Build call info
        let owner: H160 = input.read::<Address>()?.into();
		let who: Runtime::AccountId = Runtime::AddressMapping::into_account_id(owner);

        log::debug!(target: "evm", "withdraw funds from {:?} treasury", metaverse_id);

		<metaverse::Pallet<Runtime>>::withdraw_from_metaverse_fund(
			RawOrigin::Signed(who).into(),
			metaverse_id,
		)
		.map_err(|e| PrecompileFailure::Revert {
			exit_status: ExitRevert::Reverted,
			output: Into::<&str>::into(e).as_bytes().to_vec(),
		})?;

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

/* 
	fn transfer(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_log_costs_manual(3, 32)?;

		// Parse input of index 1 (owner)
		let mut input = handle.read_input()?;
		input.expect_arguments(2)?;

		let to: H160 = input.read::<Address>()?.into();
		let amount = input.read::<BalanceOf<Runtime>>()?;

		// Build call info
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let to = Runtime::AddressMapping::into_account_id(to);

		log::debug!(target: "evm", "meatverse: transfer from: {:?}, to: {:?}, amount: {:?}", origin, to, amount);

		<metaverse::Pallet<Runtime> as MetaverseTrait<Runtime::AccountId>>::transfer(
			metaverse_id,
			&origin,
			&to,
		)
		.map_err(|e| PrecompileFailure::Revert {
			exit_status: ExitRevert::Reverted,
			output: Into::<&str>::into(e).as_bytes().to_vec(),
		})?;

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}
*/
}