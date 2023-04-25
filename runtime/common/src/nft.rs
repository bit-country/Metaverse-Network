use core_primitives::{Attributes, CollectionType, NftMetadata, TokenType};
use evm_mapping::AddressMapping as EvmMapping;
use evm_mapping::EvmAddressMapping;
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
use sp_runtime::Perbill;
use sp_std::{marker::PhantomData, prelude::*};

use codec::{DecodeAll, Encode};
use precompile_utils::data::{Address, EvmData, EvmDataWriter};
use precompile_utils::handle::PrecompileHandleExt;
use precompile_utils::modifier::FunctionModifier;
use precompile_utils::prelude::RuntimeHelper;
use precompile_utils::{succeed, EvmResult};
use primitives::evm::{Erc20Mapping, Output};
use primitives::{evm, Balance, ClassId, GroupCollectionId, TokenId};

#[precompile_utils_macro::generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
	GetNftMetadata = "getNftMetadata(uint256,uint256)",
	GetNftAddress = "getNftAddress(uint256,uint256)",
	GetAssetOwner = "getAssetOwner(uint256,uint256)",
	GetClassFundBalance = "getClassFundBalance(address,uint256)",
	CreateClass = "createClass(address,bytes32,uint256,unit256,uint256)",
	MintNfts = "mintNfts(address,uint256,bytes32,uint256)",
	TransferNft = "transferNft(address,uint256,uint256)",
	BurnNft = "burnNft(address,uint256,uint256)",
	WithdrawFromClassFund = "withdrawFromClassFund(address,uint256)",
}

//Alias for the Balance type for the provided Runtime and Instance.
//pub type BalanceOf<Runtime> = <<Runtime as nft::Config>::Currency as BasicCurrencyTrait<
//<Runtime as frame_system::Config>::AccountId,>>::Balance;

//Alias for the ClassId type for the provided Runtime and Instance.
pub type ClassIdOf<Runtime> = <Runtime as orml_nft::Config>::ClassId;

//Alias for the TokenId type for the provided Runtime and Instance.
pub type TokenIdOf<Runtime> = <Runtime as orml_nft::Config>::TokenId;

/// The `Nft` impl precompile.
///
///
/// `input` data starts with `action`, `class_id`, and `token_id`.
///
///
/// Actions:
/// - Get NFT asset info.
/// - Get asset owner.
/// - Get class fund balance.
/// - Create class.
/// - Mint NFTs.
/// - Transfer NFT. Rest `input` bytes: `from`, `to`, and (`class_id`, `token_id`).
/// - Burn NFT. Rest `input` bytes: `from`, and (`class_id`, `token_id`).
/// - Withdraw from class fund. Rest `input` bytes: `(`class_id`, `token_id`).
pub struct NftPrecompile<Runtime>(PhantomData<Runtime>);

impl<Runtime> Default for NftPrecompile<Runtime> {
	fn default() -> Self {
		Self(PhantomData)
	}
}

#[cfg(test)]
impl<Runtime> PrecompileSet for NftPrecompile<Runtime>
where
	Runtime: nft_pallet::Config + orml_nft::Config + pallet_evm::Config + frame_system::Config + evm_mapping::Config,
	Runtime: Erc20Mapping,
	U256: From<
		<<Runtime as nft_pallet::Config>::Currency as frame_support::traits::Currency<
			<Runtime as frame_system::Config>::AccountId,
		>>::Balance,
	>,
	ClassIdOf<Runtime>: TryFrom<U256> + Into<ClassId> + EvmData,
	TokenIdOf<Runtime>: TryFrom<U256> + Into<<Runtime as orml_nft::Config>::TokenId> + EvmData,
	//BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + EvmData,
	<<Runtime as frame_system::Config>::Call as Dispatchable>::Origin: OriginTrait,
{
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<EvmResult<PrecompileOutput>> {
		let result = {
			let selector = match handle.read_selector() {
				Ok(selector) => selector,
				Err(e) => return Some(Err(e)),
			};

			if let Err(err) = handle.check_function_modifier(match selector {
				Action::CreateClass
				| Action::MintNfts
				| Action::BurnNft
				| Action::TransferNft
				| Action::WithdrawFromClassFund => FunctionModifier::NonPayable,
				_ => FunctionModifier::View,
			}) {
				return Some(Err(err));
			}

			match selector {
				// Local and Foreign common
				Action::GetNftMetadata => Self::nft_metadata(handle),
				Action::GetNftAddress => Self::nft_address(handle),
				Action::GetAssetOwner => Self::nft_owner(handle),
				Action::GetClassFundBalance => Self::class_fund_balance(handle),
				Action::CreateClass => Self::create_class(handle),
				Action::MintNfts => Self::mint_nfts(handle),
				Action::TransferNft => Self::transfer_nft(handle),
				Action::BurnNft => Self::burn_nft(handle),
				Action::WithdrawFromClassFund => Self::withdraw_class_funds(handle),
			}
		};
		Some(result)
	}

	fn is_precompile(&self, address: H160) -> bool {
		todo!()
	}
}

impl<Runtime> Precompile for NftPrecompile<Runtime>
where
	Runtime: nft_pallet::Config + orml_nft::Config + pallet_evm::Config + frame_system::Config + evm_mapping::Config,
	Runtime: Erc20Mapping,
	U256: From<
		<<Runtime as nft_pallet::Config>::Currency as frame_support::traits::Currency<
			<Runtime as frame_system::Config>::AccountId,
		>>::Balance,
	>,
	ClassIdOf<Runtime>: TryFrom<U256> + Into<ClassId> + EvmData,
	TokenIdOf<Runtime>: TryFrom<U256> + Into<<Runtime as orml_nft::Config>::TokenId> + EvmData,
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
				Action::CreateClass
				| Action::MintNfts
				| Action::BurnNft
				| Action::TransferNft
				| Action::WithdrawFromClassFund => FunctionModifier::NonPayable,
				_ => FunctionModifier::View,
			}) {
				return Err(err);
			}

			match selector {
				// Local and Foreign common
				Action::GetNftMetadata => Self::nft_metadata(handle),
				Action::GetNftAddress => Self::nft_address(handle),
				Action::GetAssetOwner => Self::nft_owner(handle),
				Action::GetClassFundBalance => Self::class_fund_balance(handle),
				Action::CreateClass => Self::create_class(handle),
				Action::MintNfts => Self::mint_nfts(handle),
				Action::TransferNft => Self::transfer_nft(handle),
				Action::BurnNft => Self::burn_nft(handle),
				Action::WithdrawFromClassFund => Self::withdraw_class_funds(handle),
			}
		};
		result
	}
}

impl<Runtime> NftPrecompile<Runtime>
where
	Runtime: nft_pallet::Config + orml_nft::Config + pallet_evm::Config + frame_system::Config + evm_mapping::Config,
	Runtime: Erc20Mapping,
	U256: From<
		<<Runtime as nft_pallet::Config>::Currency as frame_support::traits::Currency<
			<Runtime as frame_system::Config>::AccountId,
		>>::Balance,
	>,
	ClassIdOf<Runtime>: TryFrom<U256> + Into<ClassId> + EvmData,
	TokenIdOf<Runtime>: TryFrom<U256> + Into<<Runtime as orml_nft::Config>::TokenId> + EvmData,
	//BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + EvmData,
{
	fn nft_metadata(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Parse input of 2 (class_id, token_id)
		let mut input = handle.read_input()?;
		input.expect_arguments(2)?;

		let class_id = input.read::<ClassIdOf<Runtime>>()?.into();
		let token_id = input.read::<TokenIdOf<Runtime>>()?.into();

		// Fetch info
		let nft_info_result = <orml_nft::Pallet<Runtime>>::tokens::<ClassIdOf<Runtime>, TokenIdOf<Runtime>>(
			class_id.into(),
			token_id.into(),
		);

		match nft_info_result {
			Some(nft_info) => {
				log::debug!(target: "evm", "NFT asset metadata: {:?}", nft_info);
				let encoded = Output::encode_bytes(&nft_info.metadata);
				// Build output.
				Ok(succeed(encoded))
			}
			None => Err(PrecompileFailure::Error {
				exit_status: pallet_evm::ExitError::Other("Non-existing NFT.".into()),
			}),
		}
	}

	fn nft_address(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Parse input of 2 (class_id, token_id)
		let mut input = handle.read_input()?;
		input.expect_arguments(2)?;

		let class_id = input.read::<ClassIdOf<Runtime>>()?.into();
		let token_id = input.read::<TokenId>()?.into();

		// Fetch info
		let nft_address_result = Runtime::encode_nft_evm_address((class_id, token_id));

		match nft_address_result {
			Some(nft_address) => {
				log::debug!(target: "evm", "NFT asset address: {:?}", nft_address);
				let encoded = Output::encode_address(nft_address);
				// Build output.
				Ok(succeed(encoded))
			}
			None => Err(PrecompileFailure::Error {
				exit_status: pallet_evm::ExitError::Other("Non-existing NFT asset EVM address.".into()),
			}),
		}
	}

	fn nft_owner(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Parse input of 2 (class_id, token_id)
		let mut input = handle.read_input()?;
		input.expect_arguments(2)?;

		let class_id = input.read::<ClassIdOf<Runtime>>()?.into();
		let token_id = input.read::<TokenIdOf<Runtime>>()?.into();

		// Fetch info
		let nft_info_result = <orml_nft::Pallet<Runtime>>::tokens::<ClassIdOf<Runtime>, TokenIdOf<Runtime>>(
			class_id.into(),
			token_id.into(),
		);

		match nft_info_result {
			Some(nft_info) => {
				log::debug!(target: "evm", "Nft asset info: {:?}", nft_info);

				let evm_address =
					<Runtime as evm_mapping::Config>::AddressMapping::get_or_create_evm_address(&nft_info.owner);
				let encoded = Output::encode_address(evm_address);
				// Build output.
				Ok(succeed(encoded))

				/*
				let evm_address_output = <Runtime as evm_mapping::Config>::AddressMapping::get_evm_address(&nft_info.owner);
				match evm_address_output {
					Some(evm_address) => {
						let encoded = Output::encode_address(evm_address);
						// Build output.
						Ok(succeed(encoded))
					}
					None => Err(PrecompileFailure::Error {
						exit_status: pallet_evm::ExitError::Other("Invalid nft asset owner EVM address".into()),
					}),
				}
				*/
			}
			None => Err(PrecompileFailure::Error {
				exit_status: pallet_evm::ExitError::Other("Non-existing NFT.".into()),
			}),
		}
	}

	fn class_fund_balance(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Parse input of index 1 (class_id)
		let mut input = handle.read_input()?;
		input.expect_arguments(1)?;

		let class_id = input.read::<ClassIdOf<Runtime>>()?.into();

		let class_treasury = <Runtime as nft_pallet::Config>::PalletId::get().into_sub_account_truncating(class_id);

		// Fetch info
		let balance = <Runtime as nft_pallet::Config>::Currency::free_balance(&class_treasury);

		log::debug!(target: "evm", "class: ({:?}, ) fund balance: {:?}", class_id, balance);

		let encoded = Output::encode_uint(balance);
		// Build output.
		Ok(succeed(encoded))
	}

	fn create_class(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let who = <Runtime as evm_mapping::Config>::AddressMapping::get_account_id(&handle.context().caller);

		// Parse input
		let mut input = handle.read_input()?;
		input.expect_arguments(4)?;

		let class_collection_id: GroupCollectionId = input.read::<GroupCollectionId>()?.into();
		let class_metadata: NftMetadata = input.read::<NftMetadata>()?.into();
		let mut class_attributes: Attributes = Attributes::new();
		class_attributes.insert("Chain:".as_bytes().to_vec(), "EVM".as_bytes().to_vec());
		class_attributes.insert("Metadata:".as_bytes().to_vec(), class_metadata.clone());

		let class_token_type = TokenType::Transferable;
		let class_collection_type = CollectionType::Collectable;

		let class_royalty_fee: Perbill = Perbill::from_percent(input.read::<u32>()?.into());
		let class_mint_limit: u32 = input.read::<u32>()?.into();

		log::debug!(target: "evm", "create class for: {:?}", who);

		<nft_pallet::Pallet<Runtime>>::create_class(
			RawOrigin::Signed(who).into(),
			class_metadata,
			class_attributes,
			class_collection_id,
			class_token_type,
			class_collection_type,
			class_royalty_fee,
			Some(class_mint_limit),
		)
		.map_err(|e| PrecompileFailure::Revert {
			exit_status: ExitRevert::Reverted,
			output: Into::<&str>::into(e).as_bytes().to_vec(),
		})?;

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn mint_nfts(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let who = <Runtime as evm_mapping::Config>::AddressMapping::get_account_id(&handle.context().caller);

		// Parse input of index 1 (owner)
		let mut input = handle.read_input()?;
		input.expect_arguments(3)?;

		let nft_class_id = input.read::<ClassIdOf<Runtime>>()?.into();
		let nft_metadata: NftMetadata = input.read::<NftMetadata>()?.into();
		let mut nft_attributes: Attributes = Attributes::new();
		nft_attributes.insert("Chain:".as_bytes().to_vec(), "EVM".as_bytes().to_vec());
		nft_attributes.insert("Metadata:".as_bytes().to_vec(), nft_metadata.clone());
		let nft_quantity: u32 = input.read::<u32>()?.into();

		log::debug!(target: "evm", "create class for: {:?}", who);

		<nft_pallet::Pallet<Runtime>>::mint(
			RawOrigin::Signed(who).into(),
			nft_class_id.into(),
			nft_metadata,
			nft_attributes,
			nft_quantity,
		)
		.map_err(|e| PrecompileFailure::Revert {
			exit_status: ExitRevert::Reverted,
			output: Into::<&str>::into(e).as_bytes().to_vec(),
		})?;

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn transfer_nft(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Parse input of index 1 (owner)
		let mut input = handle.read_input()?;
		input.expect_arguments(3)?;

		// Build call info
		let to: H160 = input.read::<Address>()?.into();

		let class_id = input.read::<ClassIdOf<Runtime>>()?.into();
		let token_id = input.read::<TokenIdOf<Runtime>>()?.into();

		let origin = <Runtime as evm_mapping::Config>::AddressMapping::get_account_id(&handle.context().caller);
		let to = <Runtime as evm_mapping::Config>::AddressMapping::get_account_id(&to);

		log::debug!(target: "evm", "nft transfer from: {:?}, to: {:?}, token: ({:?}, {:?})", origin, to, class_id, token_id);

		<nft_pallet::Pallet<Runtime>>::transfer(RawOrigin::Signed(origin).into(), to, (class_id.into(), token_id))
			.map_err(|e| PrecompileFailure::Revert {
				exit_status: ExitRevert::Reverted,
				output: Into::<&str>::into(e).as_bytes().to_vec(),
			})?;

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn burn_nft(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Parse input of index 1 (owner)
		let mut input = handle.read_input()?;
		input.expect_arguments(2)?;

		// Build call info
		let who = <Runtime as evm_mapping::Config>::AddressMapping::get_account_id(&handle.context().caller);
		//let who: Runtime::AccountId = <Runtime as
		// pallet_evm::Config>::AddressMapping::into_account_id(owner);
		let class_id = input.read::<ClassIdOf<Runtime>>()?.into();
		let token_id = input.read::<TokenIdOf<Runtime>>()?.into();

		log::debug!(target: "evm", "burn nft asset: ({:?}, {:?})", class_id, token_id);

		<nft_pallet::Pallet<Runtime>>::burn(RawOrigin::Signed(who).into(), (class_id.into(), token_id)).map_err(
			|e| PrecompileFailure::Revert {
				exit_status: ExitRevert::Reverted,
				output: Into::<&str>::into(e).as_bytes().to_vec(),
			},
		)?;

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn withdraw_class_funds(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Parse input of index 1 (class_id)
		let mut input = handle.read_input()?;
		input.expect_arguments(2)?;

		let class_id = input.read::<ClassIdOf<Runtime>>()?.into();

		// Build call info
		let owner: H160 = input.read::<Address>()?.into();
		let who = <Runtime as evm_mapping::Config>::AddressMapping::get_account_id(&handle.context().caller);

		log::debug!(target: "evm", "withdraw funds from class {:?} fund", class_id);

		<nft_pallet::Pallet<Runtime>>::withdraw_funds_from_class_fund(RawOrigin::Signed(who).into(), class_id.into())
			.map_err(|e| PrecompileFailure::Revert {
			exit_status: ExitRevert::Reverted,
			output: Into::<&str>::into(e).as_bytes().to_vec(),
		})?;

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}
}
