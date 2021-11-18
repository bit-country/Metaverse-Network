use crate::AssetId;
use sp_runtime::DispatchError;

pub trait NftTrait<AccountId, Balance, TokenId> {
	fn do_transfer_with_loyalty(
		sender: AccountId,
		to: &AccountId,
		asset_id: AssetId,
		loyalty_fee: Balance,
	) -> Result<TokenId, DispatchError>;
}
