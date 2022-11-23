
use frame_support::pallet_prelude::Get;
use frame_support::traits::{Currency, OriginTrait};
use orml_traits::{BasicCurrency, MultiCurrency as MetaverseTrait};
use pallet_evm::{
	AddressMapping, ExitRevert, ExitSucceed, Precompile, PrecompileFailure, PrecompileHandle, PrecompileOutput,
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
use primitives::{evm, Balance, FungibleTokenId, MetaverseId}; 

use core_primitives::MetaverseMetadata;

#[precompile_utils_macro::generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
    GetMetaverse = "getMetaverse(uint256)",
    GetMetaverseOwner = "getMetaverseOwner(uint256)",
    GetMetaverseFundBalance = "getMetaverseFundBalance(uint256)",
	CreateMetaverse = "createMetaverse(address)",
    //TransferMetaverse = "transferMetaverse(address,uint256)",
    //WithdrawFromMetaverseFund = "withdrawFromMetaverseFund(uint256)",
    //UpdateMetaverseListingFee = "updateMetaverseListingFee(uint256,uint256)",
}

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
	metaverse::Pallet<Runtime>:
		MetaverseTrait<Runtime::AccountId>,
	U256: From<
		<<Runtime as metaverse::Config>::MetaverseTrait<
			<Runtime as frame_system::Config>::AccountId,
		>>,
	>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + EvmData,
	<<Runtime as frame_system::Config>::Call as Dispatchable>::Origin: OriginTrait,
{
    fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult {
		let address = handle.code_address();

		if let Some(metaverse_id) = Runtime::decode_evm_address(address) {
			log::debug!(target: "evm", "meatverse: meatevrse id: {:?}", metaverse_id);

            let result = {
				let selector = match handle.read_selector() {
					Ok(selector) => selector,
					Err(e) => return Err(e),
				};

				//if let Err(err) = handle.check_function_modifier(match selector {
				//	Action::Approve | Action::Transfer | Action::TransferFrom => FunctionModifier::NonPayable,
				//	_ => FunctionModifier::View,
				//}) {
				//	return Err(err);
				//}

				match selector {
					// Local and Foreign common
					Action::GetMetaverse => Self::metaverse_info(metaverse_id, handle),
                    Action::GetMetaverseOwner => Self::metaverse_info(metaverse_id, handle),
                    Action::GetMetaverseFundBalance => Self::fund_balance(metaverse_id, handle),
                    Action::CreateMetaverse => Self::create_metaverse(handle),
					//Action::TransferMetaverse => Self::transfer(metaverse_id, handle),
                    //Action::WithdrawFromMetaverseFund => Self::transfer(metaverse_id, handle),
                    //Action::UpdateMetaverseListingFee => Self::transfer(metaverse_id, handle),
				}
			};
		}
		Err(PrecompileFailure::Revert {
			exit_status: ExitRevert::Reverted,
			output: "invalid metaverse id".into(),
		})
    }
}

impl<Runtime> MeatversePrecompile<Runtime>
where
	Runtime: me::Config + pallet_evm::Config + frame_system::Config,
	metaverse::Pallet<Runtime>:
		MetaverseTrait<Runtime::AccountId, CurrencyId = FungibleTokenId, Balance = Balance>,
	U256: From<
        <<Runtime as metaverse::Config>::MetaverseTrait<
            <Runtime as frame_system::Config>::AccountId,
        >>,
    >,
    BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + EvmData,
{
	fn metaverse_info(metaverse_id: MetaverseId, handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Parse input
		let input = handle.read_input()?;
		input.expect_arguments(0)?;

		// Fetch info
		let metaverse_info = <metaverse::Pallet<Runtime> as MetaverseTrait<Runtime::AccountId>>::get_metaverse(metaverse_id);

		log::debug!(target: "evm", "metaverse_info: {:?}", metaverse_info);

		let encoded = Output::encode_uint(metaverse_info);
		// Build output.
		Ok(succeed(encoded))
	}

	fn fund_balance(metaverse_id: MetaverseId, handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Parse input of index 1 (owner)
		let mut input = handle.read_input()?;
		input.expect_arguments(1)?;

		let owner: H160 = input.read::<Address>()?.into();
		let who: Runtime::AccountId = Runtime::AddressMapping::into_account_id(owner);
         
        let metaverse_treasury = <metaverse::Pallet<Runtime> as MetaverseTrait<Runtime::AccountId>>::get_metaverse_treasury(metaverse_id);
		
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
        let metaverse_metadata: MetaverseMetadata = [u8; 5].to_vec();

        log::debug!(target: "evm", "create metaverse for: {:?}", who);

        <metaverse::Pallet<Runtime> as MetaverseTrait<Runtime::AccountId>>::create_metaverse(
			&who,
			metaverse_metadata,
		)
		.map_err(|e| PrecompileFailure::Revert {
			exit_status: ExitRevert::Reverted,
			output: Into::<&str>::into(e).as_bytes().to_vec(),
		})?;

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

/* 
	fn transfer(metaverse_id: MetaverseId, handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
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