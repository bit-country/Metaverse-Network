#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::DispatchResult,
	ensure,
	traits::{Currency, Get, IsType, Randomness, ReservableCurrency},
	StorageMap, StorageValue,
};
use frame_system::{self as system, ensure_signed};

use primitives::Balance;
use sp_core::H256;
use sp_runtime::ModuleId;
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;
use unique_asset::{AssetId, CollectionId};

const MODULE_ID: ModuleId = ModuleId(*b"bcc/bNFT");

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct NftAssetData<AccountId> {
	pub name: Vec<u8>,
	pub description: Vec<u8>,
	pub properties: Vec<u8>,
	pub supporters: Vec<AccountId>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct NftCollectionData<Balance> {
	//Minimum balance to create a collection of Asset
	pub deposit: Balance,
	// Metadata from ipfs
	pub properties: Vec<u8>,
}

#[cfg(test)]
mod tests;

pub trait Trait: system::Trait + unique_asset::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type RandomnessSource: Randomness<H256>;
	/// Convert between NftCollectionData and unique_asset::Trait::CollectionData
	type ConvertNftCollectionData: IsType<<Self as unique_asset::Trait>::CollectionData>
		+ IsType<NftCollectionData<BalanceOf<Self>>>;
	/// Convert between NftAssetData and unique_asset::Trait::AssetData
	type ConvertNftData: IsType<<Self as unique_asset::Trait>::AssetData>
		+ IsType<NftAssetData<Self::AccountId>>;
	/// The minimum balance to create class
	type CreateCollectionDeposit: Get<BalanceOf<Self>>;
	type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

decl_storage! {
	trait Store for Module<T: Trait> as Country {

		pub NftAssets get(fn get_nft_asset): map hasher(blake2_128_concat) AssetId => Option<NftAssetData<T::AccountId>>;
		pub NftOwner get(fn get_nft_owner): map hasher(blake2_128_concat) AssetId => T::AccountId;
		pub AllNftCount get(fn all_nft_count): u64;

		Init get(fn is_init): bool;
		Nonce get(fn nonce): u32;
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
		AssetId = AssetId,
		CollectionId = CollectionId,
	{
		NewNftCollectionCreated(AccountId, CollectionId),
		NewNftCreated(AssetId),
		TransferedNft(AccountId, AccountId, AssetId),
		SignedNft(AssetId, AccountId),
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
		fn create_collection(origin, metadata: Vec<u8>, properties: Vec<u8>) -> DispatchResult {

			let sender = ensure_signed(origin)?;
			let next_collection_id = unique_asset::Module::<T>::next_collection_id();
			//Secure deposit of collection owner
			let deposit = T::CreateCollectionDeposit::get();
			//TODO Reserve the fund
			<T as Trait>::Currency::reserve(&sender, deposit.clone())?;
			let collection_data = NftCollectionData { deposit, properties };
			let collection_data = T::ConvertNftCollectionData::from(collection_data);
			let collection_data = Into::<<T as unique_asset::Trait>::CollectionData>::into(collection_data);

			unique_asset::Module::<T>::create_collection(&sender, metadata, collection_data)?;

			Self::deposit_event(RawEvent::NewNftCollectionCreated(sender, next_collection_id));
			Ok(())
		}

		#[weight = 10_000]
		fn mint(origin, collection_id: CollectionId ,name: Vec<u8>, description: Vec<u8>, properties: Vec<u8>) -> DispatchResult {

			let sender = ensure_signed(origin)?;
			let new_nft_data = NftAssetData {
				name: name,
				description: description,
				properties: properties,
				supporters: Vec::<T::AccountId>::new()
			};

			let nft_data = new_nft_data.clone();
			let new_nft_data = T::ConvertNftData::from(new_nft_data);
			let new_nft_data = Into::<<T as unique_asset::Trait>::AssetData>::into(new_nft_data);

			//Create new nft token
			match unique_asset::Module::<T>::mint(&sender, collection_id ,new_nft_data.clone()){
				Ok(id) => {
					<NftOwner<T>>::insert(&id, &sender);

					<NftAssets<T>>::insert(&id, nft_data.clone());

					let all_nft_count = Self::all_nft_count();

					let new_all_nft_count = all_nft_count.checked_add(1)
						.ok_or("Overflow adding a new nft to total supply")?;

					AllNftCount::put(new_all_nft_count);
					Self::deposit_event(RawEvent::NewNftCreated(id));
				},
				Err(error) => {}
			};
			Ok(())
		}
		#[weight = 100_000]
		fn transfer(origin,  to: T::AccountId, asset: (CollectionId, AssetId)) -> DispatchResult {

			let sender = ensure_signed(origin)?;
			//Get owner of the country
			// let owner = Self::owner_of(country_id).ok_or("No country owner of this country")?;
			// ensure!(owner == sender, "You are not the owner of the country");

			let asset_info = unique_asset::Module::<T>::assets(asset.0, asset.1).ok_or(Error::<T>::AssetInfoNotFound)?;

			ensure!(sender == asset_info.owner, Error::<T>::NoPermission);

			unique_asset::Module::<T>::transfer_from(sender.clone(), to.clone(), asset.0.clone(), asset.1.clone())?;

			NftOwner::<T>::try_mutate_exists(asset.1, |asset_by_owner| -> DispatchResult {
				//Ensure there is record of the asset id with account
				ensure!(asset_by_owner.take().is_some(), Error::<T>::NoPermission);
				NftOwner::<T>::insert(&asset.1, &to);
				Self::deposit_event(RawEvent::TransferedNft(sender, to, asset.1));

				Ok(())
			})
		}

		#[weight = 100_000]
		fn sign(origin, asset_id: AssetId) -> DispatchResult {

			let sender = ensure_signed(origin)?;

			<NftAssets<T>>::try_mutate_exists(asset_id, |asset_data| -> DispatchResult {
				let mut asset = asset_data.as_mut().ok_or(Error::<T>::AssetInfoNotFound)?;

				match asset.supporters.binary_search(&sender) {
					Ok(_pos) => {} // should never happen
					Err(pos) => asset.supporters.insert(pos, sender.clone()),
				}

				Self::deposit_event(RawEvent::SignedNft(asset_id, sender));

				Ok(())
			})
		}
	}
}
