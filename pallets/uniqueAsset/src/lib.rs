#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{decl_error, decl_module, decl_storage, ensure, Parameter};
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, CheckedAdd, CheckedSub, Member, One, Zero,Printable},
	DispatchError, DispatchResult, RuntimeDebug,
	print
};
use sp_std::vec::Vec;

// mod mock;
// mod tests;

/// Token info
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct AssetInfo<AccountId, Data> {
	/// Asset owner
	pub owner: AccountId,
	/// Asset Properties
	pub data: Data,
}

pub trait Trait: frame_system::Trait {
	/// The Asset ID type
	type AssetId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy;
	/// The token properties type
	type AssetData: Parameter + Member;
}

decl_error! {
	/// Error for non-fungible-token module.
	pub enum Error for Module<T: Trait> {
		/// No available Asset ID
		NoAvailableAssetId,
		/// Token(ClassId, TokenId) not found
		AssetNotFound,
		/// The operator is not the owner of the token and has no permission
		NoPermission,
		/// Arithmetic calculation overflow
		NumOverflow,
		/// Can not destroy asset
		/// Total issuance is not 0
		CannotDestroyAsset,
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as NonFungibleToken {
		/// Next available token ID.
		pub NextAssetId get(fn next_asset_id): T::AssetId;
		/// Store asset info.
		///
		/// Returns `None` if token info not set or removed.
		pub Assets get(fn assets): map hasher(twox_64_concat) T::AssetId => Option<AssetInfo<<T as frame_system::Trait>::AccountId, <T as Trait>::AssetData>>;
		/// Token existence check by owner and class ID.
		pub AssetByOwner get(fn tokens_by_owner): double_map hasher(twox_64_concat) T::AccountId, hasher(twox_64_concat) T::AssetId => Option<()>;
		pub AssetsForAccount get(fn tokens_by_account): map hasher(twox_64_concat) T::AccountId => Vec<T::AssetId>;
		pub TotalAssetIssuance get(fn get_total_assets): u64;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
	}
}

impl<T: Trait> Module<T> {
	/// Transfer NFT(non fungible token) from `from` account to `to` account
	pub fn transfer(from: &T::AccountId, to: &T::AccountId, asset_id: T::AssetId) -> DispatchResult {
		if from == to {
			return Ok(());
		}

		AssetByOwner::<T>::try_mutate_exists(from, asset_id, |asset_by_owner| -> DispatchResult {
			//Ensure there is record of the asset id with account
			ensure!(asset_by_owner.take().is_some(), Error::<T>::NoPermission);
			AssetByOwner::<T>::insert(to, &asset_id, ());
			
			// AssetsForAccount::<T>::mutate(to, |assets| {
			// 	match assets.binary_search(&asset_id) {
			// 		Ok(_pos) => {} // should never happen
			// 		Err(pos) => assets.insert(pos, &asset),
			// 	}
			// });

			Assets::<T>::try_mutate_exists(asset_id, |asset_info| -> DispatchResult {
				let mut info = asset_info.as_mut().ok_or(Error::<T>::AssetNotFound)?;
				info.owner = to.clone();
				
				Ok(())
			})
		})
	}

	/// Mint NFT(non fungible token) to `owner`
	pub fn mint(
		owner: &T::AccountId,
		data: T::AssetData,
	) -> Result<T::AssetId, DispatchError> {
		NextAssetId::<T>::try_mutate(|id| -> Result<T::AssetId, DispatchError> {
			let asset_id = *id;
			*id = id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailableAssetId)?;

			let asset_info = AssetInfo {
				owner: owner.clone(),
				data,
			};

			Assets::<T>::insert(asset_id, asset_info);
			AssetByOwner::<T>::insert(owner, asset_id, ());
			AssetsForAccount::<T>::mutate(owner, |assets| {
				match assets.binary_search(&asset_id) {
					Ok(_pos) => {} // should never happen
					Err(pos) => assets.insert(pos, asset_id),
				}
			});

			let total_asset_count = Self::get_total_assets();

			let new_total_asset_count = total_asset_count.checked_add(1)
				.ok_or("Overflow adding a new count to total supply of asset")?;
				  
			TotalAssetIssuance::put(new_total_asset_count);	
			
			Ok(asset_id)
		})
	}

	/// Burn NFT(non fungible token) from `owner`
	pub fn burn(owner: &T::AccountId, asset: T::AssetId) -> DispatchResult {
		Assets::<T>::try_mutate_exists(asset, |asset_info| -> DispatchResult {
			ensure!(asset_info.take().is_some(), Error::<T>::AssetNotFound);

			AssetByOwner::<T>::try_mutate_exists(owner, asset ,|info| -> DispatchResult {
				ensure!(info.take().is_some(), Error::<T>::NoPermission);
				//TODO Do burn and reducee total supply

				Ok(())
			})
		})
	}
}