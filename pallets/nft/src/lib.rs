// This pallet use The Open Runtime Module Library (ORML) which is a community maintained collection of Substrate runtime modules.
// Thanks to all contributors of orml.
// https://github.com/open-web3-stack/open-runtime-module-library

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    ensure, dispatch::{DispatchResult, DispatchResultWithPostInfo},
    traits::{Currency, ExistenceRequirement, Get, ReservableCurrency},
    pallet_prelude::*,
};
use primitives::Balance;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use frame_system::pallet_prelude::*;
use orml_nft::Pallet as NftModule;
use primitives::{AssetId, GroupCollectionId};
use sp_runtime::RuntimeDebug;
use sp_runtime::{
    traits::{AccountIdConversion, One},
    DispatchError, ModuleId,
};
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;

pub mod default_weight;

pub use default_weight::WeightInfo;

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
pub struct NftGroupCollectionData<AccountId> {
    pub name: Vec<u8>,
    pub owner: AccountId,
    // Metadata from ipfs
    pub properties: Vec<u8>,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct NftClassData<Balance> {
    //Minimum balance to create a collection of Asset
    pub deposit: Balance,
    // Metadata from ipfs
    pub properties: Vec<u8>,
    pub token_type: TokenType,
    pub collection_type: CollectionType,
    pub total_supply: u64,
    pub initial_supply: u64,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct NftAssetData<Balance> {
    //Deposit balance to create each token
    pub deposit: Balance,
    pub name: Vec<u8>,
    pub description: Vec<u8>,
    pub properties: Vec<u8>,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum TokenType {
    Transferable,
    BoundToAddress,
}

impl TokenType {
    pub fn is_transferable(&self) -> bool {
        match *self {
            TokenType::Transferable => true,
            _ => false,
        }
    }
}

impl Default for TokenType {
    fn default() -> Self {
        TokenType::Transferable
    }
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CollectionType {
    Collectable,
    Wearable,
    Executable,
}

//Collection extension for fast retrieval
impl CollectionType {
    pub fn is_collectable(&self) -> bool {
        match *self {
            CollectionType::Collectable => true,
            _ => false,
        }
    }

    pub fn is_executable(&self) -> bool {
        match *self {
            CollectionType::Executable => true,
            _ => false,
        }
    }

    pub fn is_wearable(&self) -> bool {
        match *self {
            CollectionType::Wearable => true,
            _ => false,
        }
    }
}

impl Default for CollectionType {
    fn default() -> Self {
        CollectionType::Collectable
    }
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::config]
    pub trait Config:
    frame_system::Config +
    orml_nft::Config<
        TokenData=NftAssetData<BalanceOf<Self>>,
        ClassData=NftClassData<BalanceOf<Self>>,
    >
    {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// The minimum balance to create class
        #[pallet::constant]
        type CreateClassDeposit: Get<BalanceOf<Self>>;
        /// The minimum balance to create token
        #[pallet::constant]
        type CreateAssetDeposit: Get<BalanceOf<Self>>;
        // Currency type for reserve/unreserve balance
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
        //NFT Module Id
        #[pallet::constant]
        type ModuleId: Get<ModuleId>;
        // Weight info
        type WeightInfo: WeightInfo;
    }

    type ClassIdOf<T> = <T as orml_nft::Config>::ClassId;
    type TokenIdOf<T> = <T as orml_nft::Config>::TokenId;
    type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::storage]
    #[pallet::getter(fn get_asset)]
    pub(super) type Assets<T: Config> =
    StorageMap<_, Blake2_128Concat, AssetId, (ClassIdOf<T>, TokenIdOf<T>), OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_assets_by_owner)]
    pub(super) type AssetsByOwner<T: Config> =
    StorageMap<_, Blake2_128Concat, T::AccountId, Vec<AssetId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_group_collection)]
    pub(super) type GroupCollections<T: Config> =
    StorageMap<_, Blake2_128Concat, GroupCollectionId, NftGroupCollectionData<T::AccountId>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_class_collection)]
    pub(super) type ClassDataCollection<T: Config> =
    StorageMap<_, Blake2_128Concat, ClassIdOf<T>, GroupCollectionId, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn next_group_collection_id)]
    pub(super) type NextGroupCollectionId<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn all_nft_collection_count)]
    pub(super) type AllNftGroupCollection<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_class_type)]
    pub(super) type ClassDataType<T: Config> = StorageMap<_, Blake2_128Concat, ClassIdOf<T>, TokenType, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn next_asset_id)]
    pub(super) type NextAssetId<T: Config> = StorageValue<_, AssetId, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    #[pallet::metadata(
    < T as frame_system::Config >::AccountId = "AccountId",
    ClassIdOf < T > = "ClassId",
    TokenIdOf < T > = "TokenId",
    )]
    pub enum Event<T: Config> {
        //New NFT Group Collection created
        NewNftCollectionCreated(<T as frame_system::Config>::AccountId, GroupCollectionId),
        //New NFT Collection/Class created
        NewNftClassCreated(<T as frame_system::Config>::AccountId, ClassIdOf<T>),
        //Emit event when new nft minted - show the first and last asset mint
        NewNftMinted(AssetId, AssetId, <T as frame_system::Config>::AccountId, ClassIdOf<T>, u32),
        //Successfully transfer NFT
        TransferedNft(<T as frame_system::Config>::AccountId, <T as frame_system::Config>::AccountId, TokenIdOf<T>),
        //Signed on NFT
        SignedNft(TokenIdOf<T>, <T as frame_system::Config>::AccountId),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Attempted to initialize the bitcountry after it had already been initialized.
        AlreadyInitialized,
        //Asset Info not found
        AssetInfoNotFound,
        //Asset Id not found
        AssetIdNotFound,
        //No permission
        NoPermission,
        //No available collection id
        NoAvailableCollectionId,
        //Collection id is not exist
        CollectionIsNotExist,
        //Class Id not found
        ClassIdNotFound,
        //Non Transferable
        NonTransferable,
        //Invalid quantity
        InvalidQuantity,
        //No available asset id
        NoAvailableAssetId,
        //Asset Id is already exist
        AssetIdAlreadyExist,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn create_group(origin: OriginFor<T>, name: Vec<u8>, properties: Vec<u8>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            let next_group_collection_id = Self::do_create_group_collection(&sender, name.clone(), properties.clone())?;

            let collection_data = NftGroupCollectionData {
                owner: sender.clone(),
                name,
                properties,
            };

            GroupCollections::<T>::insert(next_group_collection_id, collection_data);

            let all_collection_count = Self::all_nft_collection_count();
            let new_all_nft_collection_count = all_collection_count.checked_add(One::one())
                .ok_or("Overflow adding a new collection to total collection")?;

            AllNftGroupCollection::<T>::put(new_all_nft_collection_count);

            Self::deposit_event(Event::<T>::NewNftCollectionCreated(sender, next_group_collection_id));
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn create_class(origin: OriginFor<T>, metadata: Vec<u8>, properties: Vec<u8>, collection_id: GroupCollectionId, token_type: TokenType, collection_type: CollectionType) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let next_class_id = NftModule::<T>::next_class_id();

            let collection_info = Self::get_group_collection(collection_id).ok_or(Error::<T>::CollectionIsNotExist)?;

            ensure!(sender == collection_info.owner, Error::<T>::NoPermission);
            //Class fund
            let class_fund: T::AccountId = T::ModuleId::get().into_sub_account(next_class_id);

            // Secure deposit of token class owner -- TODO - support customise deposit
            let class_deposit = T::CreateClassDeposit::get();
            // Transfer fund to pot
            <T as Config>::Currency::transfer(&sender, &class_fund, class_deposit, ExistenceRequirement::KeepAlive)?;

            <T as Config>::Currency::reserve(&class_fund, <T as Config>::Currency::free_balance(&class_fund))?;

            let class_data = NftClassData
            {
                deposit: class_deposit,
                properties,
                token_type,
                collection_type,
                total_supply: Default::default(),
                initial_supply: Default::default(),
            };

            NftModule::<T>::create_class(&sender, metadata, class_data)?;
            ClassDataCollection::<T>::insert(next_class_id, collection_id);

            Self::deposit_event(Event::<T>::NewNftClassCreated(sender, next_class_id));

            Ok(().into())
        }


        #[pallet::weight(< T as Config >::WeightInfo::mint(* quantity))]
        pub fn mint(origin: OriginFor<T>, class_id: ClassIdOf<T>, name: Vec<u8>, description: Vec<u8>, metadata: Vec<u8>, quantity: u32) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            ensure!(quantity >= 1, Error::<T>::InvalidQuantity);
            let class_info = NftModule::<T>::classes(class_id).ok_or(Error::<T>::ClassIdNotFound)?;
            ensure!(sender == class_info.owner, Error::<T>::NoPermission);

            let deposit = T::CreateAssetDeposit::get();
            let class_fund: T::AccountId = T::ModuleId::get().into_sub_account(class_id);
            let total_deposit = deposit * Into::<BalanceOf<T>>::into(quantity);

            <T as Config>::Currency::transfer(&sender, &class_fund, total_deposit, ExistenceRequirement::KeepAlive)?;
            <T as Config>::Currency::reserve(&class_fund, total_deposit)?;

            let new_nft_data = NftAssetData {
                deposit,
                name,
                description,
                properties: metadata.clone(),
            };

            let mut new_asset_ids: Vec<AssetId> = Vec::new();

            for _ in 0..quantity {
                let asset_id = NextAssetId::<T>::try_mutate(|id| -> Result<AssetId, DispatchError> {
                    let current_id = *id;
                    *id = id.checked_add(One::one()).ok_or(Error::<T>::NoAvailableAssetId)?;

                    Ok(current_id)
                })?;

                new_asset_ids.push(asset_id);

                if AssetsByOwner::<T>::contains_key(&sender) {
                    AssetsByOwner::<T>::try_mutate(
                        &sender,
                        |asset_ids| -> DispatchResult {
                            // Check if the asset_id already in the owner
                            ensure!(!asset_ids.iter().any(|i| asset_id == *i), Error::<T>::AssetIdAlreadyExist);
                            asset_ids.push(asset_id);
                            Ok(())
                        },
                    )?;
                } else {
                    let mut assets = Vec::<AssetId>::new();
                    assets.push(asset_id);
                    AssetsByOwner::<T>::insert(&sender, assets)
                }

                let token_id = NftModule::<T>::mint(&sender, class_id, metadata.clone(), new_nft_data.clone())?;
                Assets::<T>::insert(asset_id, (class_id, token_id));
            }

            Self::deposit_event(Event::<T>::NewNftMinted(*new_asset_ids.first().unwrap(), *new_asset_ids.last().unwrap(), sender, class_id, quantity));

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn transfer(origin: OriginFor<T>, to: T::AccountId, asset_id: AssetId) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            //FIXME asset transfer should be reverted once it's locked in Auction
            let token_id = Self::do_transfer(&sender, &to, asset_id)?;

            Self::deposit_event(Event::<T>::TransferedNft(sender, to, token_id));

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn transfer_batch(origin: OriginFor<T>, tos: Vec<(T::AccountId, AssetId)>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            for (_i, x) in tos.iter().enumerate() {
                let item = &x;
                let owner = &sender.clone();

                let asset = Assets::<T>::get(item.1).ok_or(Error::<T>::AssetIdNotFound)?;

                let class_info = NftModule::<T>::classes(asset.0).ok_or(Error::<T>::ClassIdNotFound)?;
                let data = class_info.data;

                match data.token_type {
                    TokenType::Transferable => {
                        let asset_info = NftModule::<T>::tokens(asset.0, asset.1).ok_or(Error::<T>::AssetInfoNotFound)?;
                        ensure!(owner.clone() == asset_info.owner, Error::<T>::NoPermission);
                        Self::handle_asset_ownership_transfer(&owner, &item.0, item.1);
                        NftModule::<T>::transfer(&owner, &item.0, (asset.0, asset.1))?;
                        Self::deposit_event(Event::<T>::TransferedNft(owner.clone(), item.0.clone(), asset.1.clone()));
                    }
                    _ => ()
                };
            }

            Ok(().into())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}

impl<T: Config> Module<T> {
    fn do_create_group_collection(
        sender: &T::AccountId,
        name: Vec<u8>,
        properties: Vec<u8>,
    ) -> Result<GroupCollectionId, DispatchError> {
        let next_group_collection_id = NextGroupCollectionId::<T>::try_mutate(
            |collection_id| -> Result<GroupCollectionId, DispatchError> {
                let current_id = *collection_id;

                *collection_id = collection_id
                    .checked_add(One::one())
                    .ok_or(Error::<T>::NoAvailableCollectionId)?;

                Ok(current_id)
            },
        )?;

        let collection_data = NftGroupCollectionData::<T::AccountId> {
            name,
            owner: sender.clone(),
            properties,
        };

        <GroupCollections<T>>::insert(next_group_collection_id, collection_data);

        Ok(next_group_collection_id)
    }

    fn handle_asset_ownership_transfer(
        sender: &T::AccountId,
        to: &T::AccountId,
        asset_id: AssetId,
    ) -> DispatchResult {
        //Remove asset from sender
        AssetsByOwner::<T>::try_mutate(&sender, |asset_ids| -> DispatchResult {
            // Check if the asset_id already in the owner
            let asset_index = asset_ids.iter().position(|x| *x == asset_id).unwrap();
            asset_ids.remove(asset_index);

            Ok(())
        })?;

        //Insert asset to recipient

        if AssetsByOwner::<T>::contains_key(to) {
            AssetsByOwner::<T>::try_mutate(&to, |asset_ids| -> DispatchResult {
                // Check if the asset_id already in the owner
                ensure!(
                    asset_ids.iter().any(|i| asset_id == *i),
                    Error::<T>::AssetIdAlreadyExist
                );
                asset_ids.push(asset_id);
                Ok(())
            })?;
        } else {
            let mut asset_ids = Vec::<AssetId>::new();
            asset_ids.push(asset_id);
            AssetsByOwner::<T>::insert(&to, asset_ids);
        }

        Ok(())
    }

    pub fn do_transfer(
        sender: &T::AccountId,
        to: &T::AccountId,
        asset_id: AssetId) -> Result<<T as orml_nft::Config>::TokenId, DispatchError> {
        let asset = Assets::<T>::get(asset_id).ok_or(Error::<T>::AssetIdNotFound)?;

        let class_info = NftModule::<T>::classes(asset.0).ok_or(Error::<T>::ClassIdNotFound)?;
        let data = class_info.data;

        match data.token_type {
            TokenType::Transferable => {
                let check_ownership = Self::check_nft_ownership(&sender, &asset_id)?;
                ensure!(check_ownership, Error::<T>::NoPermission);

                Self::handle_asset_ownership_transfer(&sender, &to, asset_id);

                NftModule::<T>::transfer(&sender, &to, asset.clone())?;
                Ok(asset.1)
            }
            TokenType::BoundToAddress => Err(Error::<T>::NonTransferable.into())
        }
    }

    pub fn check_nft_ownership(
        sender: &T::AccountId,
        asset_id: &AssetId) -> Result<bool, DispatchError> {
        let asset = Assets::<T>::get(asset_id).ok_or(Error::<T>::AssetIdNotFound)?;
        let class_info = NftModule::<T>::classes(asset.0).ok_or(Error::<T>::ClassIdNotFound)?;
        let data = class_info.data;

        let asset_info = NftModule::<T>::tokens(asset.0, asset.1).ok_or(Error::<T>::AssetInfoNotFound)?;
        if sender == &asset_info.owner {
            return Ok(true);
        }

        return Ok(false);
    }
}
