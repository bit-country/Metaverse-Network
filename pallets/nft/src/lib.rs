#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchResult, DispatchResultWithPostInfo},
    ensure,
    traits::{Currency, Get, IsType, Randomness, ReservableCurrency},
    weights::Weight,
    StorageMap, StorageValue,
};
use frame_system::{self as system, ensure_signed};
use orml_nft::Module as NftModule;
use orml_traits::{BasicCurrency, BasicReservableCurrency};
use primitives::{AccountId, Balance, CollectionId};
use sp_core::H256;
// use sp_io::hashing::blake2_128;
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

const MODULE_ID: ModuleId = ModuleId(*b"bcc/bNFT");

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct NftCollectionData<AccountId> {
    pub name: Vec<u8>,
    pub owner: AccountId,
    // Metadata from ipfs
    pub properties: Vec<u8>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct NftAssetData {
    pub name: Vec<u8>,
    pub description: Vec<u8>,
    pub properties: Vec<u8>,
}

#[derive(Encode, Decode, Debug, Clone, Eq, PartialEq)]
pub enum TokenType {
    Transferrable,
    BoundToAddress,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct NftClassData {
    //Minimum balance to create a collection of Asset
    pub deposit: Balance,
    // Metadata from ipfs
    pub properties: Vec<u8>,
    pub token_type: TokenType,
}

#[cfg(test)]
mod tests;

pub trait Trait: orml_nft::Trait<TokenData = NftAssetData, ClassData = NftClassData> {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Randomness: Randomness<Self::Hash>;
    /// The minimum balance to create class
    type CreateClassDeposit: Get<Balance>;
    /// The minimum balance to create token
    type CreateAssetDeposit: Get<Balance>;
    // Currency type for reserve/unreserve balance
    type Currency: BasicReservableCurrency<Self::AccountId, Balance = Balance>;
    //NFT Module Id
    type ModuleId: Get<ModuleId>;
    // Weight info
    type WeightInfo: WeightInfo;
}

type ClassIdOf<T> = <T as orml_nft::Trait>::ClassId;
type TokenIdOf<T> = <T as orml_nft::Trait>::TokenId;
type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

decl_storage! {
    trait Store for Module<T: Trait> as NftAsset {

        // pub NftAssets get(fn get_nft_asset): map hasher(blake2_128_concat) TokenId => Option<NftAssetData<T::AccountId>>;
        // pub NftOwner get(fn get_nft_owner): map hasher(blake2_128_concat) AssetId => T::AccountId;
        pub Collections get(fn get_collection): map hasher(blake2_128_concat) CollectionId => Option<NftCollectionData<T::AccountId>>;
        pub TokenDataCollection get(fn get_token_collection): double_map hasher(twox_64_concat) ClassIdOf<T>, hasher(twox_64_concat) TokenIdOf<T> => Option<CollectionId>;
        pub NextCollectionId get(fn next_collection_id): u64;
        pub AllNftCount get(fn all_nft_count): u64;
        pub AllNftCollection get(fn all_nft_collection_count): u64;

        Init get(fn is_init): bool;
        // Nonce get(fn nonce): u32;
    }
}

decl_event!(
    pub enum Event<T>
    where
        <T as frame_system::Trait>::AccountId,
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
    pub enum Error for Module<T: Trait> {
        /// Attempted to initialize the country after it had already been initialized.
        AlreadyInitialized,
        //Asset Info not found
        AssetInfoNotFound,
        //Asset Id not found
        AssetIdNotFound,
        //No permission
        NoPermission,
        NoAvailableCollectionId,
        //Class Id not found
        ClassIdNotFound,
        NonTransferrable,
        InvalidQuantity
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = 10_000]
        fn create_collection(origin, name: Vec<u8>, properties: Vec<u8>) -> DispatchResultWithPostInfo{

            let sender = ensure_signed(origin)?;

            let next_collection_id = Self::do_create_collection(&sender, name, properties)?;

            Self::deposit_event(RawEvent::NewNftCollectionCreated(sender, next_collection_id));
            Ok(().into())
        }

        #[weight = 10_000]
        fn create_class(origin, metadata: Vec<u8>, properties: Vec<u8>, token_type: TokenType) -> DispatchResultWithPostInfo{

            let sender = ensure_signed(origin)?;
            let next_class_id = NftModule::<T>::next_class_id();
            //Class fund
            let class_fund: T::AccountId = T::ModuleId::get().into_sub_account(next_class_id);

            //Secure deposit of token class owner -- support customise deposit
            let class_deposit = T::CreateClassDeposit::get();
            //Transfer fund to pot
            <T as Trait>::Currency::transfer(&sender, &class_fund, class_deposit)?;

            <T as Trait>::Currency::reserve(&class_fund, <T as Trait>::Currency::free_balance(&class_fund))?;

            let class_data = NftClassData { deposit: class_deposit, properties, token_type };

            NftModule::<T>::create_class(&sender, metadata, class_data)?;

            Self::deposit_event(RawEvent::NewNftClassCreated(sender, next_class_id));

            Ok(().into())
        }

        #[weight = <T as Trait>::WeightInfo::mint(*quantity)]
        fn mint(origin, class_id: ClassIdOf<T>, name: Vec<u8>, description: Vec<u8>, metadata: Vec<u8>, quantity: u32) -> DispatchResultWithPostInfo {

            let sender = ensure_signed(origin)?;
            ensure!(quantity >= 1, Error::<T>::InvalidQuantity);
            let class_info = NftModule::<T>::classes(class_id).ok_or(Error::<T>::ClassIdNotFound)?;
            ensure!(sender == class_info.owner, Error::<T>::NoPermission);
            let deposit = T::CreateAssetDeposit::get();
            let class_fund: T::AccountId = T::ModuleId::get().into_sub_account(class_id);
            let total_deposit = deposit * (quantity as u128);
            <T as Trait>::Currency::transfer(&sender, &class_fund, total_deposit)?;
            <T as Trait>::Currency::reserve(&class_fund, total_deposit)?;

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

impl<T: Trait> Module<T> {
    fn random_value(sender: &T::AccountId) -> [u8; 16] {
        let payload = (
            T::Randomness::random_seed(),
            &sender,
            <frame_system::Module<T>>::extrinsic_index(),
        );

        payload.using_encoded(blake2_128)
    }

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
