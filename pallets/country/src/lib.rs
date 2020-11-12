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
use sp_runtime::{traits::Hash, RuntimeDebug};
use sp_std::vec::Vec;
use unique_asset::AssetId;
use primitives::{AccountId, CountryId}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct CountryAssetData {
	pub image: Vec<u8>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Country{
	pub owner: AccountId;
	pub metadata: Vec<u8>;
	pub token_address: AccountId;
}

#[cfg(test)]
mod tests;

pub trait Trait: system::Trait + nft::Trait + unique_asset::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type RandomnessSource: Randomness<H256>;
	/// Convert between CountryAssetData and unique_asset::Trait::AssetData
	type ConvertCountryData: IsType<<Self as unique_asset::Trait>::AssetData>
		+ IsType<nft::NftAssetData<Self::AccountId>>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Country {

		pub NextCountryId get(fn next_country_id): CountryId;
		pub Countries get(fn get_country): map hasher(blake2_128_concat) CountryId => Country;
		pub CountryOwner get(fn get_country_owner): map hasher(blake2_128_concat) CountryId => Option<T::AccountId>;
		pub AllCountriesCount get(fn all_countries_count): u64;

		Init get(fn is_init): bool;

		Nonce get(fn nonce): u32;
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = AccountId,
		CountryId = CountryId,
	{
		Initialized(AccountId),
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
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		#[weight = 10_000]
		fn create_country(origin, metadata: Vec<u8>) -> DispatchResult<CountryId, DispatchError> {

			// let sender = ensure_signed(origin)?;
			// // let nonce = Nonce::get();
			// // let random_str = Self::encode_and_update_nonce();
			// // let random_seed = T::RandomnessSource::random_seed();
			// // let random_result = T::RandomnessSource::random(&random_str);
			// // let random_hash = (random_seed, &sender, random_result).using_encoded(<T as system::Trait>::Hashing::hash);
			// let new_country_data = <T as nft::Trait>::NftAssetData {
			// 	name: country_name,
			// 	description: country_name,
			// 	properties: image,
			// };


			// let country_data = new_country_data.clone();
			// let new_country_data = T::ConvertCountryData::from(new_country_data);
			// let new_country_data = Into::<<T as unique_asset::Trait>::AssetData>::into(new_country_data);

			// // ensure!(!<Countries<T>>::contains_key(random_hash), "Country hash id already exists");

			// //Create country and mint nft asset token
			// let new_asset_id = unique_asset::Module::<T>::mint(&sender, new_country_data.clone())?;
			// <CountryOwner<T>>::insert(new_asset_id, &sender);

			// <Countries<T>>::insert(&new_asset_id, country_data);

			// let all_countries_count = Self::all_countries_count();

			  // let new_all_countries_count = all_countries_count.checked_add(1)
			// 	.ok_or("Overflow adding a new country to total supply")?;

			// AllCountriesCount::put(new_all_countries_count);
			// Self::deposit_event(RawEvent::NewCountryCreated(new_asset_id));

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
}
