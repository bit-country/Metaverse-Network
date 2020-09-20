#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, 
	StorageValue,
	StorageMap,
	dispatch::DispatchResult, 
	ensure,
	traits::Randomness
};
use sp_core::H256;
use frame_system::{self as system, ensure_signed};
use sp_runtime::traits::Hash;
use sp_std::vec::Vec;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Country<Hash> {
	id: Hash,
}

#[cfg(test)]
mod tests;

pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type RandomnessSource: Randomness<H256>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Country {

		pub CountryOwner get(fn owner_of): map hasher(blake2_128_concat) T::Hash => Option<T::AccountId>;
		pub Countries get(fn get_country): map hasher(blake2_128_concat) T::Hash => Country<T::Hash>;
		pub AllCountriesCount get(fn all_countries_count): u64;

		Init get(fn is_init): bool;

		Nonce get(fn nonce): u32;
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
	{
		Initialized(AccountId),
 		RandomnessConsumed(H256, H256),
	}

);

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Attempted to initialize the token after it had already been initialized.
		AlreadyInitialized,
		/// Attempted to transfer more funds than were available
		InsufficientFunds,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		#[weight = 10_000]
		fn create_country(origin) -> DispatchResult {

            let sender = ensure_signed(origin)?;
		   
			let nonce = Nonce::get();
			
			let random_str = Self::encode_and_update_nonce();

			let random_seed = T::RandomnessSource::random_seed();
			let random_result = T::RandomnessSource::random(&random_str);
			let random_hash = (random_seed, &sender, random_result).using_encoded(<T as system::Trait>::Hashing::hash);

			let new_country = Country{
				id: random_hash,
			};
			ensure!(!<Countries<T>>::contains_key(random_hash), "Country already exists");

			<CountryOwner<T>>::insert(random_hash, &sender);
			<Countries<T>>::insert(random_hash, new_country);

			let all_countries_count = Self::all_countries_count();

            let new_all_countries_count = all_countries_count.checked_add(1)
				.ok_or("Overflow adding a new country to total supply")?;
				
			AllCountriesCount::put(new_all_countries_count);	

			Self::deposit_event(RawEvent::RandomnessConsumed(random_seed, random_result));

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
