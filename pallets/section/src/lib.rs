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
use country:: {CountryOwner};
use unique_asset;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Section<Hash> {
	id: Hash,
	block_id: Hash,
}

#[cfg(test)]
mod tests;

pub trait Trait: system::Trait + country::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type BlockRandomnessSource: Randomness<H256>;
}

decl_storage! {
	trait Store for Module<T: Trait> as SectionModule {

		pub SectionOwner get(fn get_section_owner): map hasher(blake2_128_concat) T::Hash => Option<T::AccountId>;
		pub Sections get(fn get_section): map hasher(blake2_128_concat) T::Hash => Section<T::Hash>;
		pub AllSectionCount get(fn all_section_count): u64;

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
		BlockRandomnessSource(H256, H256),
	}

);

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Attempted to initialize the token after it had already been initialized.
		AlreadyInitialized,
		//No permission section issuance
		NoPermissionSectionIssuance,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		// #[weight = 10_000]
		// fn create_section(origin, block_id: T::Hash, country_id: T::Hash) -> DispatchResult {

        //     let sender = ensure_signed(origin)?;
		   
		// 	let nonce = Nonce::get();
			
		// 	let random_str = Self::encode_and_update_nonce();

		// 	let random_seed = T::RandomnessSource::random_seed();
		// 	let random_result = T::RandomnessSource::random(&random_str);
		// 	let random_hash = (random_seed, &sender, random_result).using_encoded(<T as system::Trait>::Hashing::hash);

		// 	//Check country ownership
		// 	let country_owner = <CountryOwner<T>>::get(country_id).ok_or("Not a country owner of this country")?;
		// 	ensure!(country_owner == sender, Error::<T>::NoPermissionSectionIssuance);

		// 	let new_section = Section{
		// 		id: random_hash,
		// 		block_id: block_id,
		// 	};
		// 	ensure!(!<Sections<T>>::contains_key(random_hash), "Section already exists");

		// 	<SectionOwner<T>>::insert(random_hash, &sender);
		// 	<Sections<T>>::insert(random_hash, new_section);

		// 	let all_section_count = Self::all_section_count();

        //     let new_all_section_count = all_section_count.checked_add(1)
		// 		.ok_or("Overflow adding a new section to total supply")?;
				
		// 	AllSectionCount::put(new_all_section_count);	

		// 	Self::deposit_event(RawEvent::BlockRandomnessSource(random_seed, random_result));

        //     Ok(())
		// }				
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
