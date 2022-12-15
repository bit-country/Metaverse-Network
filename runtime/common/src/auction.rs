use frame_support::pallet_prelude::Get;
use frame_support::traits::{Currency, OriginTrait};
use orml_traits::{BasicCurrency, MultiCurrency as MultiCurrencyTrait};
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
use primitives::{evm, Balance, FungibleTokenId, ItemId, AuctionId};

#[precompile_utils_macro::generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
	AuctionInfo = "getAuctionInfo(uint256)",
    AuctionItem = "getAuctionItem(uint256)",
    CreateNftAuction = "createNftAuction(uint256,uint256,uint256,uint256,uint256,uint256)",
    Bid = "bid(uint256,uint256)",
    FinalizeAuction = "finalizeAuction(uint256)",
    CreateNftBuyNow = "createNftBuyNow(uint256,uint256,uint256,uint256,uint256,uint256)",
    BuyNow = "buyNow(uint256,uint256)",
    CancelListing = "cancelListing(uint256)",
    MakeOffer = "makeOffer(uint256,uint256,uint256)",
    AcceptOffer = "acceptOffer(uint256,uint256,account)",
    WithdrawOffer = "withrawOffer(uint256,uint256)",
}

/// Alias for the Balance type for the provided Runtime and Instance.
pub type BalanceOf<Runtime> = <<Runtime as auction::Config>::FungibleTokenCurrency as MultiCurrencyTrait<
	<Runtime as frame_system::Config>::AccountId,
>>::Balance;

/// The `Auction` impl precompile.
///
///
/// Actions
/// - Query auction info. Rest `input` bytes: `auction_id`.
/// - Query auction item. Rest `input` bytes: `auction_id`.
/// - Create auction for an NFT. Rest `input` bytes: `class_id`, `token_id`, `value`, `end_time`, `metaverse_id`, `currency_id`.
/// - Bid on auction. Rest `input` bytes: `auction_id`, `value`.
/// - Finalize auction. Rest `input` bytes: `auction_id`.
/// - Create buy now for an NFT. Rest `input` bytes: `class_id`, `token_id`, `value`, `end_time`, `metaverse_id`, `currency_id`.
/// - Buy a buy now listing. Rest `input` bytes: `auction_id`, `value`.
/// - Cancel auction or buy now listing. Rest `input` bytes: `auction_id`.
/// - Make offer for an NFT. Rest `input` bytes: `class_id`, `token_id`, `value`.
/// - Accept offer for an NFT. Rest `input` bytes: `class_id`, `token_id`, `account_id`.
/// - Withdraw offer for an NFT. Rest `input` bytes: `class_id`, `token_id`.
pub struct AuctionPrecompile<Runtime>(PhantomData<Runtime>);