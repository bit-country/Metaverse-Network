#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{decl_error, decl_event, decl_module, decl_storage, ensure, Parameter};
use frame_system::{self as system, ensure_signed};
use sp_runtime::{
	print,
	traits::{AtLeast32BitUnsigned, CheckedAdd, CheckedSub, Member, One, Printable, Zero},
	DispatchError, DispatchResult, RuntimeDebug,
};
use sp_std::vec::Vec;

// mod mock;
// mod tests;

pub type AssetId = u64;
pub type RentId = u64;

/// Token info
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct AssetInfo<AccountId, Data> {
	/// Asset owner
	pub owner: AccountId,
	/// Asset Properties
	pub data: Data,
}

/// Rental info
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct RentalInfo<AccountId, BlockNumber, Balance> {
	/// Rental beneficial
	pub owner: AccountId,
	/// Rent start
	pub start: BlockNumber,
	/// Rent end
	pub end: Option<BlockNumber>,
	//Price per block
	pub price_per_block: Balance,
}

pub trait Trait: frame_system::Trait + pallet_balances::Trait {
	/// The Asset ID type
	// type AssetId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy;
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
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

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
		AssetId = AssetId,
	{
		NewAssetCreated(AssetId),
		TransferedAsset(AccountId, AccountId, AssetId),
		NewAssetRented(AssetId, RentId),
	}
);

decl_storage! {
	trait Store for Module<T: Trait> as NonFungibleToken {
		/// Next available token ID.
		pub NextAssetId get(fn next_asset_id): AssetId;
		/// Store asset info.
		///
		/// Returns `None` if token info not set or removed.
		pub Assets get(fn assets): map hasher(twox_64_concat) AssetId => Option<AssetInfo<<T as frame_system::Trait>::AccountId, <T as Trait>::AssetData>>;
		/// Token existence check by owner and class ID.
		pub AssetByOwner get(fn tokens_by_owner): double_map hasher(twox_64_concat) T::AccountId, hasher(twox_64_concat) AssetId => Option<()>;
		pub AssetsForAccount get(fn tokens_by_account): map hasher(twox_64_concat) T::AccountId => Vec<AssetId>;
		pub TotalAssetIssuance get(fn get_total_assets): u64;

		//Rental Mapping
		pub NextRentId get(fn next_rent_id): RentId;
		//Check if asset id and rent id is valid
		pub AssetRent get(fn get_asset_rent): double_map hasher(twox_64_concat) AssetId, hasher(twox_64_concat) RentId => Option<()>;
		//Get AssetId is currently on rent
		pub AssetForRent get(fn asset_for_rent): map hasher(twox_64_concat) RentId => AssetId;
		//Get AssetId is renting
		pub AssetByRent get(fn asset_by_rent): map hasher(twox_64_concat) RentId => AssetId;
		//Get rent info by id
		pub Rents get(fn get_rent_info): map hasher(twox_64_concat) RentId => Option<RentalInfo<<T as frame_system::Trait>::AccountId, <T as frame_system::Trait>::BlockNumber, T::Balance>>;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		#[weight = 10_000]
		pub fn transfer(origin, to: T::AccountId ,asset_id: AssetId) -> DispatchResult{
			let from = ensure_signed(origin)?;

			ensure!(AssetByOwner::<T>::contains_key(&from, &asset_id), Error::<T>::NoPermission);

			Self::transfer_from(from, to, asset_id);

			Ok(())
		}

		#[weight = 10_000]
		pub fn rent_asset(origin, source_asset_id: AssetId, consume_asset_id: AssetId, start: T::BlockNumber, end: T::BlockNumber, price: T::Balance) -> DispatchResult{
			let owner = ensure_signed(origin)?;
			//TODO Check asset ownership
			//TODO Check not in auction
			NextRentId::try_mutate(|id| -> DispatchResult {
				let rent_id = *id;
				*id = id
					.checked_add(One::one())
					.ok_or(Error::<T>::NoAvailableAssetId)?;
				let rent_info = RentalInfo {
					owner: owner.clone(),
					start: start,
					end: Some(end),
					price_per_block: price,
				};

				Rents::<T>::insert(rent_id, rent_info);

				AssetRent::insert(rent_id, source_asset_id, ());
				AssetByRent::insert(rent_id, consume_asset_id);
				AssetForRent::insert(rent_id, source_asset_id);

				Ok(())
			})
		}
	}
}

impl<T: Trait> Module<T> {
	/// Mint NFT(non fungible token) to `owner`
	pub fn mint(owner: &T::AccountId, data: T::AssetData) -> Result<AssetId, DispatchError> {
		NextAssetId::try_mutate(|id| -> Result<AssetId, DispatchError> {
			let asset_id = *id;
			*id = id
				.checked_add(One::one())
				.ok_or(Error::<T>::NoAvailableAssetId)?;

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

			let new_total_asset_count = total_asset_count
				.checked_add(1)
				.ok_or("Overflow adding a new count to total supply of asset")?;

			TotalAssetIssuance::put(new_total_asset_count);
			Ok(asset_id)
		})
	}

	/// Burn NFT(non fungible token) from `owner`
	pub fn burn(owner: &T::AccountId, asset: AssetId) -> DispatchResult {
		Assets::<T>::try_mutate_exists(asset, |asset_info| -> DispatchResult {
			ensure!(asset_info.take().is_some(), Error::<T>::AssetNotFound);

			AssetByOwner::<T>::try_mutate_exists(owner, asset, |info| -> DispatchResult {
				ensure!(info.take().is_some(), Error::<T>::NoPermission);
				//TODO Do burn and reducee total supply

				Ok(())
			})
		})
	}

	/// Transfer NFT(non fungible token) from `from` account to `to` account
	pub fn transfer_from(
		from: T::AccountId,
		to: T::AccountId,
		asset_id: AssetId,
	) -> DispatchResult {
		if from == to {
			return Ok(());
		}

		AssetByOwner::<T>::try_mutate_exists(from, asset_id, |asset_by_owner| -> DispatchResult {
			//Ensure there is record of the asset id with account and delete them
			ensure!(asset_by_owner.take().is_some(), Error::<T>::NoPermission);
			AssetByOwner::<T>::insert(&to, &asset_id, ());
			AssetsForAccount::<T>::mutate(&to, |assets| {
				match assets.binary_search(&asset_id) {
					Ok(_pos) => {} // should never happen
					Err(pos) => assets.insert(pos, asset_id.clone()),
				}
			});

			Assets::<T>::try_mutate_exists(asset_id, |asset_info| -> DispatchResult {
				let mut info = asset_info.as_mut().ok_or(Error::<T>::AssetNotFound)?;
				info.owner = to.clone();
				Ok(())
			})
		})
	}
}
