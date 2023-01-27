// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::useless_attribute)]

use frame_support::{pallet_prelude::*, transactional};
use frame_system::pallet_prelude::*;
use orml_traits::MultiCurrencyExtended;
use sp_arithmetic::traits::SaturatedConversion;
use sp_core::U256;
use sp_std::prelude::*;

pub use pallet::*;
use primitives::{Balance, FungibleTokenId};

pub type ResourceId = [u8; 32];
pub type ChainId = u8;
pub type DepositNonce = u64;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::PalletId;
	use orml_traits::MultiCurrency;
	use sp_runtime::traits::AccountIdConversion;

	use core_primitives::NFTTrait;
	use primitives::{ClassId, TokenId};

	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Specifies the origin check provided by the bridge for calls that can only be called by
		/// the bridge pallet
		type BridgeOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;
		/// The currency mechanism.
		type Currency: MultiCurrencyExtended<Self::AccountId, CurrencyId = FungibleTokenId, Balance = Balance>;
		/// The nft handling mechanism.
		type NFTHandler: NFTTrait<Self::AccountId, Balance, ClassId = ClassId, TokenId = TokenId>;
		/// Native currency
		type NativeCurrencyId: Get<FungibleTokenId>;
		/// The sovereign pallet
		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidTransfer,
		InvalidCommand,
		InvalidPayload,
		InvalidFeeOption,
		FeeOptionsMissing,
		ResourceTokenIdAlreadyExist,
		ResourceIdNotRegistered,
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// [resource_id token_id]
		RegisterNewResourceTokenId(ResourceId, FungibleTokenId),
		/// [resource_id token_id]
		RemoveResourceTokenId(ResourceId, FungibleTokenId),
		/// [chainId, min_fee, fee_scale]
		FeeUpdated(ChainId, Balance, u32),
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
	pub type BridgeFee<T: Config> = StorageMap<_, Twox64Concat, ChainId, (Balance, u32), ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn currency_ids)]
	pub type CurrencyIds<T: Config> = StorageMap<_, Twox64Concat, ResourceId, FungibleTokenId, OptionQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(195_000 + T::DbWeight::get().writes(1))]
		pub fn sudo_change_fee(
			origin: OriginFor<T>,
			min_fee: Balance,
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
			amount: Balance,
			recipient: Vec<u8>,
			dest_id: ChainId,
		) -> DispatchResult {
			let source = ensure_signed(origin)?;
			//			ensure!(
			//				<bridge::Module<T>>::chain_whitelisted(dest_id),
			//				Error::<T>::InvalidTransfer
			//			);

			let resource_id =
				Self::resource_ids(FungibleTokenId::NativeToken(0)).ok_or(Error::<T>::ResourceIdNotRegistered)?;
			let bridge_id = T::PalletId::get().into_account_truncating();
			ensure!(BridgeFee::<T>::contains_key(&dest_id), Error::<T>::FeeOptionsMissing);
			let (min_fee, fee_scale) = Self::bridge_fee(dest_id);
			let fee_estimated = amount
				.saturating_mul(fee_scale.into())
				.checked_div(1000u32.into())
				.ok_or("Overflow")?;
			let fee = if fee_estimated > min_fee {
				fee_estimated
			} else {
				min_fee
			};
			T::Currency::transfer(
				FungibleTokenId::NativeToken(0),
				&source,
				&bridge_id,
				(amount + fee).into(),
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

		/// Executes a simple currency transfer using the bridge account as the source
		#[pallet::weight(195_000 + T::DbWeight::get().writes(1))]
		pub fn transfer(origin: OriginFor<T>, to: T::AccountId, amount: Balance, _rid: ResourceId) -> DispatchResult {
			let source = T::BridgeOrigin::ensure_origin(origin)?;
			T::Currency::transfer(FungibleTokenId::NativeToken(0), &source, &to, amount.into())?;
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
			ensure_root(origin)?;
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
			ensure_root(origin)?;
			if let Some(currency_id) = CurrencyIds::<T>::take(resource_id) {
				ResourceIds::<T>::remove(currency_id);
				Self::deposit_event(Event::RemoveResourceTokenId(resource_id, currency_id));
			}
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
			ensure_root(origin)?;
			ensure!(
				!NftResourceIds::<T>::contains_key(class_id),
				Error::<T>::ResourceTokenIdAlreadyExist,
			);

			NftResourceIds::<T>::insert(class_id, resource_id);
			Self::deposit_event(Event::RegisterNewResourceNftId(resource_id, class_id));
			Ok(())
		}

		#[pallet::weight(195_000 + T::DbWeight::get().writes(1))]
		#[transactional]
		pub fn remove_resource_nft_id(origin: OriginFor<T>, class_id: ClassId) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(
				NftResourceIds::<T>::contains_key(class_id),
				Error::<T>::ResourceIdNotRegistered,
			);

			NftResourceIds::<T>::remove(class_id);
			if let Some(resource_id) = NftResourceIds::<T>::take(class_id) {
				NftResourceIds::<T>::remove(class_id);
				Self::deposit_event(Event::RemoveResourceNftId(resource_id, class_id));
			}
			Ok(())
		}
	}
}
