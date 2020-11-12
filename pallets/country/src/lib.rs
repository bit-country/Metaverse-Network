#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::DispatchResult,
	ensure,
	traits::{Get, IsType, Randomness},
	StorageMap, StorageValue,
};
use frame_system::{self as system, ensure_signed};
use nft;
use sp_core::H256;
use sp_runtime::{
	traits::{Hash, One},
	DispatchError, RuntimeDebug,
};
use sp_std::vec::Vec;
use unique_asset::AssetId;
use primitives::{CountryId};

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct CountryAssetData {
	pub image: Vec<u8>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Country<AccountId> {
	pub owner: AccountId,
	pub metadata: Vec<u8>,
	pub token_address: AccountId,
}

#[cfg(test)]
mod tests;

pub trait Trait: system::Trait {
	type Event: From<Event> + Into<<Self as system::Trait>::Event>;
	type RandomnessSource: Randomness<H256>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Country {

		pub NextCountryId get(fn next_country_id): CountryId;
		pub Countries get(fn get_country): map hasher(twox_64_concat) CountryId => Option<Country<T::AccountId>>;
		pub CountryOwner get(fn get_country_owner): map hasher(twox_64_concat) CountryId => Option<T::AccountId>;
		pub AllCountriesCount get(fn all_countries_count): u64;

		Init get(fn is_init): bool;

		Nonce get(fn nonce): u32;
	}
}

decl_event!(
	pub enum Event {
		NewCountryCreated(CountryId),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Attempted to initialize the country after it had already been initialized.
		AlreadyInitialized,
		//Asset Info not found
		AssetInfoNotFound,
		//Asset Id not found
		AssetIdNotFound,
		//No permission
		NoPermission,
		//No available country id
		NoAvailableCountryId,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		#[weight = 10_000]
		fn create_country(origin, metadata: Vec<u8>) -> DispatchResult {

			let owner = ensure_signed(origin)?;

			let country_id = Self::new_country(&owner, metadata)?;

			CountryOwner::<T>::insert(country_id, owner);

			let total_country_count = Self::all_countries_count();

			let new_total_country_count = total_country_count.checked_add(One::one()).ok_or("Overflow adding new count to total country")?;

			AllCountriesCount::put(new_total_country_count);

			Self::deposit_event(Event::NewCountryCreated(country_id.clone()));

			Ok(())
		}

		#[weight = 100_000]
		fn transfer_country(origin,  to: T::AccountId, country_id: T::Hash, asset_id: AssetId) -> DispatchResult {

			// let sender = ensure_signed(origin)?;
			// //Get owner of the country
			// // let owner = Self::owner_of(country_id).ok_or("No country owner of this country")?;
			// // ensure!(owner == sender, "You are not the owner of the country");

			// let asset_info = unique_asset::Module::<T>::assets(asset_id).ok_or(Error::<T>::AssetInfoNotFound)?;

			// ensure!(sender == asset_info.owner, Error::<T>::NoPermission);

			// unique_asset::Module::<T>::transfer(&sender, &to, asset_id)?;
			// //TODO Emit transfer event
			// Self::deposit_event(RawEvent::TransferedCountry(sender, to, asset_id));
			  Ok(())
		}
	}
}

impl<T: Trait> Module<T> {
	/// Reads the nonce from storage, increments the stored nonce, and returns
	/// the encoded nonce to the caller.
	fn encode_and_update_nonce() -> Vec<u8> {
		let nonce = Nonce::get();
		Nonce::put(nonce.wrapping_add(1));
		nonce.encode()
	}

	fn new_country(owner: &T::AccountId, metadata: Vec<u8>) -> Result<CountryId, DispatchError> {
		let country_id = NextCountryId::try_mutate(|id| -> Result<CountryId, DispatchError>{
			let current_id = *id;
			*id = id.checked_add(One::one()).ok_or(Error::<T>::NoAvailableCountryId)?;
			Ok(current_id)
		})?;

		let country_info = Country {
			owner: owner.clone(),
			token_address: Default::default(),
			metadata,
		};

		Countries::<T>::insert(country_id, country_info);

		Ok(country_id)
	}
}
