#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, 
	StorageValue,
	StorageMap,
	dispatch::DispatchResult, 
	ensure,
	traits::{Get, IsType, Randomness},
};
use sp_core::H256;
use frame_system::{self as system, ensure_signed};
use sp_runtime::{
	traits::Hash,
	RuntimeDebug,
};
use sp_std::vec::Vec;
use unique_asset::{AssetId};

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct NftAssetData {
	pub name: Vec<u8>,
	pub description: Vec<u8>,
	pub properties: Vec<u8>,
}

#[cfg(test)]
mod tests;

pub trait Trait: system::Trait + unique_asset::Trait {
	// type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type RandomnessSource: Randomness<H256>;
	/// Convert between TokenData and orml_nft::Trait::TokenData
	type ConvertNftData: IsType<<Self as unique_asset::Trait>::AssetData> + IsType<NftAssetData>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Country {

		pub NftAssets get(fn get_nft_asset): map hasher(blake2_128_concat) AssetId => Option<NftAssetData>;
		pub NftOwner get(fn get_nft_owner): map hasher(blake2_128_concat) AssetId => T::AccountId; 
		pub AllNftCount get(fn all_nft_count): u64;

		Init get(fn is_init): bool;

		Nonce get(fn nonce): u32;
	}
}

// decl_event!(
// 	pub enum Event<T>
// 	where
// 		// AccountId = <T as system::Trait>::AccountId,
// 		AssetId = AssetId,
// 	{
// 		NewNftCreated(AssetId),
// 		// TransferedNft(AccountId, AccountId, AssetId),
// 	}

// );

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
		// fn deposit_event() = default;

		#[weight = 10_000]
		fn mint(origin, name: Vec<u8>, description: Vec<u8>, properties: Vec<u8>) -> DispatchResult {

			let sender = ensure_signed(origin)?;
			
			let new_nft_data = NftAssetData {
				name: name,
				description: description,
				properties: properties,
			};

			let nft_data = new_nft_data.clone();
			
			let new_nft_data = T::ConvertNftData::from(new_nft_data);
			let new_nft_data = Into::<<T as unique_asset::Trait>::AssetData>::into(new_nft_data);

			//Create new nft token
			match unique_asset::Module::<T>::mint(&sender, new_nft_data.clone()){
				Ok(id) => {
					<NftOwner<T>>::insert(&id, &sender);

					NftAssets::insert(&id, nft_data.clone());		

					let all_nft_count = Self::all_nft_count();

					let new_all_nft_count = all_nft_count.checked_add(1)
						.ok_or("Overflow adding a new nft to total supply")?;
				
					AllNftCount::put(new_all_nft_count);	
				},
				Err(error) => {}
			};

			// Self::deposit_event(RawEvent::NewNftCreated(&new_asset_id));

      		Ok(())
		}			
		
		#[weight = 100_000]
		fn transfer(origin,  to: T::AccountId, asset_id: AssetId) -> DispatchResult {

            let sender = ensure_signed(origin)?;
		   
			//Get owner of the country
			// let owner = Self::owner_of(country_id).ok_or("No country owner of this country")?;
			// ensure!(owner == sender, "You are not the owner of the country");

			let asset_info = unique_asset::Module::<T>::assets(asset_id).ok_or(Error::<T>::AssetInfoNotFound)?;

			ensure!(sender == asset_info.owner, Error::<T>::NoPermission);

			unique_asset::Module::<T>::transfer_from(sender, to, asset_id)?;

			// NftOwner::<T>::try_mutate_exists(asset_id, |asset_by_owner| -> DispatchResult {
			// 	//Ensure there is record of the asset id with account
			// 	ensure!(asset_by_owner.take().is_some(), Error::<T>::NoPermission);
			// 	NftOwner::<T>::insert(&asset_id, &to);
	
			// 	Self::deposit_event(RawEvent::TransferedNft(sender, to, asset_id));

			// 	Ok(())
			// })

			Ok(())
		}		
	}
}
