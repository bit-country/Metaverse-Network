
/// This pallet use The Open Runtime Module Library (ORML) which is a community maintained collection of Substrate runtime modules. Thanks to all contributors of orml.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResultWithPostInfo,
    ensure,
    traits::{Currency, ExistenceRequirement, Get, Randomness, ReservableCurrency},
    weights::Weight,
    StorageMap, StorageValue,
};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use frame_system::ensure_signed;
use orml_nft::Module as NftModule;
use primitives::CollectionId;
use sp_runtime::RuntimeDebug;
use sp_runtime::{
    traits::{AccountIdConversion, One},
    DispatchError, ModuleId,
};
use sp_std::vec::Vec;

mod default_weight;

pub trait WeightInfo {
    fn mint(i: u32) -> Weight;
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
pub struct NftCollectionData<AccountId> {
    pub name: Vec<u8>,
    pub owner: AccountId,
    // Metadata from ipfs
    pub properties: Vec<u8>,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct NftAssetData {
    pub name: Vec<u8>,
    pub description: Vec<u8>,
    pub properties: Vec<u8>,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum TokenType {
    Transferrable,
    BoundToAddress,
}

impl TokenType {
    pub fn is_transferrable(&self) -> bool {
        match *self {
            TokenType::Transferrable => true,
            _ => false,
        }
    }
}

impl Default for TokenType {
    fn default() -> Self {
        TokenType::Transferrable
    }
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct NftClassData<Balance> {
    //Minimum balance to create a collection of Asset
    pub deposit: Balance,
    // Metadata from ipfs
    pub properties: Vec<u8>,
    pub token_type: TokenType,
}

#[cfg(test)]
mod tests;

pub trait Config:
    orml_nft::Config<TokenData = NftAssetData, ClassData = NftClassData<BalanceOf<Self>>>
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type Randomness: Randomness<Self::Hash>;
    /// The minimum balance to create class
    type CreateClassDeposit: Get<BalanceOf<Self>>;
    /// The minimum balance to create token
    type CreateAssetDeposit: Get<BalanceOf<Self>>;
    // Currency type for reserve/unreserve balance
    type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
    //NFT Module Id
    type ModuleId: Get<ModuleId>;
    // Weight info
    type WeightInfo: WeightInfo;
}

type ClassIdOf<T> = <T as orml_nft::Config>::ClassId;
type TokenIdOf<T> = <T as orml_nft::Config>::TokenId;
type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

decl_storage! {
    trait Store for Module<T: Config> as NftAsset {

        pub Collections get(fn get_collection): map hasher(blake2_128_concat) CollectionId => Option<NftCollectionData<T::AccountId>>;
        pub ClassDataCollection get(fn get_class_collection): map hasher(blake2_128_concat) ClassIdOf<T> => CollectionId;
        pub NextCollectionId get(fn next_collection_id): u64;
        pub AllNftCollection get(fn all_nft_collection_count): u64;
        pub ClassDataType get(fn get_class_type): map hasher(blake2_128_concat) ClassIdOf<T> => TokenType;

    }
}

decl_event!(
    pub enum Event<T>
    where
        <T as frame_system::Config>::AccountId,
        ClassId = ClassIdOf<T>,
        AssetId = TokenIdOf<T>,
    {
        NewNftCollectionCreated(AccountId, CollectionId),
        NewNftClassCreated(AccountId, ClassId),
        NewNftMinted(AccountId, ClassId, u32),
        TransferedNft(AccountId, AccountId, AssetId),
        SignedNft(AssetId, AccountId),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        /// Attempted to initialize the country after it had already been initialized.
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
        //Non transferrable
        NonTransferrable,
        //Invalid quantity
        InvalidQuantity
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = 10_000]
        fn create_collection(origin, name: Vec<u8>, properties: Vec<u8>) -> DispatchResultWithPostInfo{

            let sender = ensure_signed(origin)?;

            let next_collection_id = Self::do_create_collection(&sender, name.clone(), properties.clone())?;

            let collection_data = NftCollectionData {
                owner: sender.clone(),
                name,
                properties,
            };

            Collections::<T>::insert(next_collection_id, collection_data);

            let all_collection_count = Self::all_nft_collection_count();
            let new_all_nft_collection_count = all_collection_count.checked_add(One::one())
                .ok_or("Overflow adding a new collection to total collection")?;

            AllNftCollection::put(new_all_nft_collection_count);

            Self::deposit_event(RawEvent::NewNftCollectionCreated(sender, next_collection_id));
            Ok(().into())
        }

        #[weight = 10_000]
        fn create_class(origin, metadata: Vec<u8>, properties: Vec<u8>, collection_id: CollectionId, token_type: TokenType) -> DispatchResultWithPostInfo{

            let sender = ensure_signed(origin)?;
            let next_class_id = NftModule::<T>::next_class_id();

            let collection_info = Self::get_collection(collection_id).ok_or(Error::<T>::CollectionIsNotExist)?;

            ensure!(sender == collection_info.owner, Error::<T>::NoPermission);
            //Class fund
            let class_fund: T::AccountId = T::ModuleId::get().into_sub_account(next_class_id);

            // Secure deposit of token class owner -- support customise deposit
            let class_deposit = T::CreateClassDeposit::get();
            // Transfer fund to pot
            <T as Config>::Currency::transfer(&sender, &class_fund, class_deposit, ExistenceRequirement::KeepAlive)?;

            <T as Config>::Currency::reserve(&class_fund, <T as Config>::Currency::free_balance(&class_fund))?;

            let class_data = NftClassData { deposit: class_deposit, properties, token_type };

            NftModule::<T>::create_class(&sender, metadata, class_data)?;
            ClassDataCollection::<T>::insert(next_class_id, collection_id);

            Self::deposit_event(RawEvent::NewNftClassCreated(sender, next_class_id));

            Ok(().into())
        }

        #[weight = <T as Config>::WeightInfo::mint(*quantity)]
        fn mint(origin, class_id: ClassIdOf<T>, name: Vec<u8>, description: Vec<u8>, metadata: Vec<u8>, quantity: u32) -> DispatchResultWithPostInfo {

            let sender = ensure_signed(origin)?;

            ensure!(quantity >= 1, Error::<T>::InvalidQuantity);
            let class_info = NftModule::<T>::classes(class_id).ok_or(Error::<T>::ClassIdNotFound)?;
            ensure!(sender == class_info.owner, Error::<T>::NoPermission);

            let deposit = T::CreateAssetDeposit::get();
            let class_fund: T::AccountId = T::ModuleId::get().into_sub_account(class_id);
            let total_deposit = deposit * Into::<BalanceOf<T>>::into(quantity);
            <T as Config>::Currency::transfer(&sender, &class_fund, total_deposit, ExistenceRequirement::KeepAlive)?;
            <T as Config>::Currency::reserve(&class_fund, total_deposit)?;

            //Global Identifier -  todo
            // let nft_uid = Self::random_value(&sender);

            let new_nft_data = NftAssetData {
                name,
                description,
                properties: metadata.clone(),
            };

            for _ in 0..quantity{
                NftModule::<T>::mint(&sender, class_id, metadata.clone(), new_nft_data.clone())?;
            }

            Self::deposit_event(RawEvent::NewNftMinted(sender, class_id, quantity));

            Ok(().into())
        }

        #[weight = 100_000]
        fn transfer(origin,  to: T::AccountId, asset: (ClassIdOf<T>, TokenIdOf<T>)) -> DispatchResultWithPostInfo {

            let sender = ensure_signed(origin)?;

            let class_info = NftModule::<T>::classes(asset.0).ok_or(Error::<T>::ClassIdNotFound)?;
            let data = class_info.data;

            match data.token_type {
                TokenType::Transferrable => {
                    let asset_info = NftModule::<T>::tokens(asset.0, asset.1).ok_or(Error::<T>::AssetInfoNotFound)?;
                    ensure!(sender == asset_info.owner, Error::<T>::NoPermission);

                    NftModule::<T>::transfer(&sender, &to, asset)?;

                    Self::deposit_event(RawEvent::TransferedNft(sender, to, asset.1));

                    Ok(().into())
                }
                TokenType::BoundToAddress => Err(Error::<T>::NonTransferrable.into())
            }
        }

        // #[weight = 100_000]
        // fn sign(origin, asset_id: TokenIdOf<T>) -> DispatchResult {

        //     let sender = ensure_signed(origin)?;
        //     <NftAssets<T>>::try_mutate_exists(asset_id, |asset_data| -> DispatchResult {
        //         let mut asset = asset_data.as_mut().ok_or(Error::<T>::AssetInfoNotFound)?;

        //         match asset.supporters.binary_search(&sender) {
        //             Ok(_pos) => {} // should never happen
        //             Err(pos) => asset.supporters.insert(pos, sender.clone()),
        //         }

        //         Self::deposit_event(RawEvent::SignedNft(asset_id, sender));

        //         Ok(())
        //     })
        // }
    }
}

impl<T: Config> Module<T> {
    // fn random_value(sender: &T::AccountId) -> [u8; 16] {
    //     let payload = (
    //         T::Randomness::random_seed(),
    //         &sender,
    //         <frame_system::Module<T>>::extrinsic_index(),
    //     );

    //     payload.using_encoded(blake2_128)
    // }

    fn do_create_collection(
        sender: &T::AccountId,
        name: Vec<u8>,
        properties: Vec<u8>,
    ) -> Result<CollectionId, DispatchError> {
        let next_collection_id =
            NextCollectionId::try_mutate(|collection_id| -> Result<CollectionId, DispatchError> {
                let current_id = *collection_id;

                *collection_id = collection_id
                    .checked_add(One::one())
                    .ok_or(Error::<T>::NoAvailableCollectionId)?;

                Ok(current_id)
            })?;

        let collection_data = NftCollectionData::<T::AccountId> {
            name,
            owner: sender.clone(),
            properties,
        };

        <Collections<T>>::insert(next_collection_id, collection_data);

        Ok(next_collection_id)
    }
}
