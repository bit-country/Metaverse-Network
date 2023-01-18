#![cfg(feature = "runtime-benchmarks")]

// use sp_runtime::traits::AccountIdConversion;
// module benchmarking
pub mod auction;
pub mod continuum;
pub mod economy;
pub mod estate;
pub mod governance;
pub mod metaverse;
pub mod reward;
pub mod utils;

// pub fn get_vesting_account() -> super::AccountId {
// 	super::TreasuryPalletId::get().into_account_truncating()
// }
