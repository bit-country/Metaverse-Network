// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::useless_attribute)]

use frame_support::{pallet_prelude::*, transactional};
use frame_system::pallet_prelude::*;
use orml_traits::MultiCurrencyExtended;
use sp_arithmetic::traits::SaturatedConversion;
use sp_core::{H160, U256};
use sp_std::prelude::*;

pub use pallet::*;
use primitives::{Balance, FungibleTokenId};

pub type ResourceId = H160;
pub type ChainId = u8;
pub type DepositNonce = u64;

#[cfg(all(feature = "std", test))]
mod mock;

#[cfg(all(feature = "std", test))]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::traits::{Currency, ExistenceRequirement, LockableCurrency, ReservableCurrency};
	use frame_support::PalletId;
	use orml_traits::MultiCurrency;
	use sp_arithmetic::traits::Saturating;
	use sp_runtime::traits::{AccountIdConversion, CheckedDiv};

	use core_primitives::NFTTrait;
	use primitives::{Attributes, ClassId, NftMetadata, TokenId};

	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Specifies the origin check provided by the bridge for calls that can only be called by
		/// the bridge pallet
		type BridgeOrigin: EnsureOrigin<Self::Origin>;
		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>
			+ LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
		/// The nft handling mechanism.
		type NFTHandler: NFTTrait<Self::AccountId, BalanceOf<Self>, ClassId = ClassId, TokenId = TokenId>;
		/// Native currency
		type NativeCurrencyId: Get<FungibleTokenId>;
		/// The sovereign pallet
		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::error]
	pub enum Error<T> {
		/// Invalid transfer
		InvalidTransfer,
		/// Invalid command
		InvalidCommand,
		/// Invalid transaction payload
		InvalidPayload,
		/// Fee option is invalid
		InvalidFeeOption,
		/// No fee available
		FeeOptionsMissing,
		/// Resource id already exists
		ResourceTokenIdAlreadyExist,
		/// Resource id is not registered
		ResourceIdNotRegistered,
		/// NFT class id is not registered for associated resource id
		ClassIdIsNotRegistered,
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// [resource_id token_id]
		RegisterNewResourceTokenId(ResourceId, FungibleTokenId),
		/// [resource_id token_id]
		RemoveResourceTokenId(ResourceId, FungibleTokenId),
		/// [chainId, min_fee, fee_scale]
		FeeUpdated(ChainId, BalanceOf<T>, u32),
		/// FungibleTransfer is for relaying fungibles (dest_id, resource_id, amount,
		/// recipient, metadata)
		FungibleTransfer(ChainId, ResourceId, U256, Vec<u8>, Vec<u8>),
		/// Non-fungibletransfer is for relaying non-fungibles (dest_id, nonce, resource_id,
		/// collection_id, token_id, recipient, metadata)
		NonFungibleTransfer(ChainId, ResourceId, ClassId, TokenId, Vec<u8>, Vec<u8>),
		/// Register new NFT id with class id[resource_id class_id]
		RegisterNewResourceNftId(ResourceId, ClassId),
		/// Remove  NFT id with class id[resource_id class_id]
		RemoveResourceNftId(ResourceId, ClassId),
		/// Bridge in executed from foreign account to account with registered resource id
		/// [resource_id, class_id, token_id, H160 address, account_id]
		NonFungibleBridgeInExecuted(ResourceId, ClassId, TokenId, Vec<u8>, T::AccountId),
		/// Bridge out executed from account to foreign account registered resource id [resource_id,
		/// class_id, token_id, H160 address]
		NonFungibleBridgeOutExecuted(ResourceId, ClassId, TokenId, T::AccountId, Vec<u8>),
	}

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::storage]
	#[pallet::getter(fn resource_ids)]
	pub type ResourceIds<T: Config> = StorageMap<_, Twox64Concat, FungibleTokenId, ResourceId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn nft_resource_ids)]
	pub type NftResourceIds<T: Config> = StorageMap<_, Twox64Concat, ClassId, ResourceId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn bridge_fee)]
	pub type BridgeFee<T: Config> = StorageMap<_, Twox64Concat, ChainId, (BalanceOf<T>, u32), ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn currency_ids)]
	pub type CurrencyIds<T: Config> = StorageMap<_, Twox64Concat, ResourceId, FungibleTokenId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn class_ids)]
	pub type ClassIds<T: Config> = StorageMap<_, Twox64Concat, ResourceId, ClassId, OptionQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(195_000 + T::DbWeight::get().writes(1))]
		pub fn sudo_change_fee(
			origin: OriginFor<T>,
			min_fee: BalanceOf<T>,
			fee_scale: u32,
			dest_id: ChainId,
		) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(fee_scale <= 1000u32, Error::<T>::InvalidFeeOption);
			BridgeFee::<T>::insert(dest_id, (min_fee, fee_scale));
			Self::deposit_event(Event::FeeUpdated(dest_id, min_fee, fee_scale));
			Ok(())
		}

		/// Transfers some amount of the native token to some recipient on a (whitelisted)
		/// destination chain.
		#[pallet::weight(195_000 + T::DbWeight::get().writes(1))]
		pub fn transfer_native(
			origin: OriginFor<T>,
			amount: BalanceOf<T>,
			recipient: Vec<u8>,
			dest_id: ChainId,
		) -> DispatchResult {
			let source = ensure_signed(origin)?;
			let resource_id =
				Self::resource_ids(FungibleTokenId::NativeToken(0)).ok_or(Error::<T>::ResourceIdNotRegistered)?;
			let bridge_id = T::PalletId::get().into_account_truncating();
			ensure!(BridgeFee::<T>::contains_key(&dest_id), Error::<T>::FeeOptionsMissing);
			let (min_fee, fee_scale) = Self::bridge_fee(dest_id);
			let fee_estimated = amount.saturating_mul(fee_scale.into());
			let fee = if fee_estimated > min_fee {
				fee_estimated
			} else {
				min_fee
			};
			T::Currency::transfer(
				&source,
				&bridge_id,
				(amount + fee).into(),
				ExistenceRequirement::AllowDeath,
			)?;

			Self::deposit_event(Event::FungibleTransfer(
				dest_id,
				resource_id,
				U256::from(amount.saturated_into::<u128>()),
				recipient.clone(),
				recipient,
			));

			Ok(())
		}

		//
		// Executable calls. These can be triggered by a bridge transfer initiated on another chain
		//

		/// Execute NFT minting using bridge account as the source
		#[pallet::weight(195_000 + T::DbWeight::get().writes(1))]
		pub fn bridge_in_nft(
			origin: OriginFor<T>,
			from: Vec<u8>,
			to: T::AccountId,
			token_id: TokenId,
			resource_id: ResourceId,
			metadata: NftMetadata,
		) -> DispatchResult {
			T::BridgeOrigin::ensure_origin(origin.clone())?;

			let bridge_origin = ensure_signed(origin)?;
			// Get collection id from resource_id
			let class_id = Self::class_ids(resource_id).ok_or(Error::<T>::ClassIdIsNotRegistered)?;

			// Check if NFT does exists
			match T::NFTHandler::check_ownership(&bridge_origin, &(class_id, token_id)) {
				Ok(is_bridge_owned) => {
					if is_bridge_owned {
						if let Ok(_transfer_succeeded) =
							T::NFTHandler::transfer_nft(&bridge_origin, &to, &(class_id, token_id))
						{
							Self::deposit_event(Event::NonFungibleBridgeInExecuted(
								resource_id,
								class_id,
								token_id,
								from,
								to,
							));
						}
					}
					Ok(())
				}
				Err(err) => match err {
					DispatchError::Other(str) => {
						if str == "AssetInfoNotFound" {
							if let Ok(_mint_succeeded) =
								T::NFTHandler::mint_token_with_id(&to, class_id, token_id, metadata, Attributes::new())
							{
								Self::deposit_event(Event::NonFungibleBridgeInExecuted(
									resource_id,
									class_id,
									token_id,
									from,
									to,
								));
							};
						}
						Ok(())
					}
					_ => Err(err),
				},
			}
		}

		/// Executes a simple currency transfer using the bridge account as the source
		#[pallet::weight(195_000 + T::DbWeight::get().writes(1))]
		pub fn bridge_out_nft(
			origin: OriginFor<T>,
			to: Vec<u8>,
			token: (ClassId, TokenId),
			chain_id: ChainId,
		) -> DispatchResult {
			let source = ensure_signed(origin)?;

			let resource_id = Self::nft_resource_ids(token.0).ok_or(Error::<T>::ResourceIdNotRegistered)?;
			let bridge_id = T::PalletId::get().into_account_truncating();

			ensure!(BridgeFee::<T>::contains_key(&chain_id), Error::<T>::FeeOptionsMissing);
			let (min_fee, fee_scale) = Self::bridge_fee(chain_id);

			T::Currency::transfer(&source, &bridge_id, min_fee.into(), ExistenceRequirement::AllowDeath)?;

			T::NFTHandler::transfer_nft(&source, &bridge_id, &token)?;

			Self::deposit_event(Event::NonFungibleBridgeOutExecuted(
				resource_id,
				token.0,
				token.1,
				source,
				to,
			));

			Ok(())
		}

		/// Register new resource token id for bridge
		#[pallet::weight(195_000 + T::DbWeight::get().writes(1))]
		#[transactional]
		pub fn register_new_token_id(
			origin: OriginFor<T>,
			resource_id: ResourceId,
			currency_id: FungibleTokenId,
		) -> DispatchResult {
			T::BridgeOrigin::ensure_origin(origin)?;
			ensure!(
				!ResourceIds::<T>::contains_key(currency_id) && !CurrencyIds::<T>::contains_key(resource_id),
				Error::<T>::ResourceTokenIdAlreadyExist,
			);

			ResourceIds::<T>::insert(currency_id, resource_id);
			CurrencyIds::<T>::insert(resource_id, currency_id);
			Self::deposit_event(Event::RegisterNewResourceTokenId(resource_id, currency_id));
			Ok(())
		}

		#[pallet::weight(195_000 + T::DbWeight::get().writes(1))]
		#[transactional]
		pub fn remove_resource_token_id(origin: OriginFor<T>, resource_id: ResourceId) -> DispatchResult {
			T::BridgeOrigin::ensure_origin(origin)?;
			if let Some(currency_id) = CurrencyIds::<T>::take(resource_id) {
				ResourceIds::<T>::remove(currency_id);
				Self::deposit_event(Event::RemoveResourceTokenId(resource_id, currency_id));
			}
			CurrencyIds::<T>::remove(resource_id);
			Ok(())
		}

		/// Register new resource token id for bridge
		#[pallet::weight(195_000 + T::DbWeight::get().writes(1))]
		#[transactional]
		pub fn register_new_nft_resource_id(
			origin: OriginFor<T>,
			resource_id: ResourceId,
			class_id: ClassId,
		) -> DispatchResult {
			T::BridgeOrigin::ensure_origin(origin)?;
			ensure!(
				!NftResourceIds::<T>::contains_key(class_id) && !ClassIds::<T>::contains_key(resource_id),
				Error::<T>::ResourceTokenIdAlreadyExist,
			);

			NftResourceIds::<T>::insert(class_id, resource_id);
			ClassIds::<T>::insert(resource_id, class_id);
			Self::deposit_event(Event::RegisterNewResourceNftId(resource_id, class_id));
			Ok(())
		}

		#[pallet::weight(195_000 + T::DbWeight::get().writes(1))]
		#[transactional]
		pub fn remove_resource_nft_id(origin: OriginFor<T>, resource_id: ResourceId) -> DispatchResult {
			T::BridgeOrigin::ensure_origin(origin)?;

			if let Some(class_id) = ClassIds::<T>::take(resource_id) {
				NftResourceIds::<T>::remove(class_id);
				Self::deposit_event(Event::RemoveResourceNftId(resource_id, class_id));
			}
			ClassIds::<T>::remove(resource_id);

			Ok(())
		}
	}
}
