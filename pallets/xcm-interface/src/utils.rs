use frame_support::dispatch::DispatchResult;
use xcm::prelude::MultiLocation;

pub trait SppAccountXcmHelper<AccountId, Balance> {
	/// Cross-chain transfer staking currency to sub account on relaychain.
	fn transfer_staking_to_sub_account(sender: &AccountId, sub_account_index: u16, amount: Balance) -> DispatchResult;
	/// Send XCM message to the relaychain for sub account to withdraw_unbonded staking currency and
	/// send it back.
	fn withdraw_unbonded_from_sub_account(sub_account_index: u16, amount: Balance) -> DispatchResult;
	/// Send XCM message to the relaychain for sub account to bond extra.
	fn bond_extra_on_sub_account(sub_account_index: u16, amount: Balance) -> DispatchResult;
	/// Send XCM message to the relaychain for sub account to unbond.
	fn unbond_on_sub_account(sub_account_index: u16, amount: Balance) -> DispatchResult;
	/// The fee of cross-chain transfer is deducted from the recipient.
	fn get_xcm_transfer_fee() -> Balance;
	/// The fee of parachain
	fn get_parachain_fee(location: MultiLocation) -> Balance;
}
