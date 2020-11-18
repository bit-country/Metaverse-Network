// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use country::CountryOwner;
use frame_support::{decl_error, decl_event, decl_module, decl_storage, ensure, Parameter};
use frame_system::{self as system, ensure_signed};
use orml_traits::{
	MultiCurrency, MultiCurrencyExtended, MultiLockableCurrency, MultiReservableCurrency,
};
use primitives::{Balance, CountryId, CurrencyId};
use sp_runtime::{
	traits::{AtLeast32Bit, AtLeast32BitUnsigned, Hash, Member, One, StaticLookup, Zero},
	DispatchError, DispatchResult,
};
use sp_std::vec::Vec;
use unique_asset::AssetId;

/// The module configuration trait.
pub trait Trait: system::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	/// The arithmetic type of asset identifier.
	type TokenId: Parameter + AtLeast32Bit + Default + Copy;
	type CountryCurrency: MultiCurrencyExtended<
		Self::AccountId,
		CurrencyId = CurrencyId,
		Balance = Balance,
	>;
}
/// A wrapper for a token name.
pub type TokenName = Vec<u8>;
/// A wrapper for a ticker name.
pub type Ticker = Vec<u8>;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Token<Balance> {
	pub ticker: Ticker,
	pub total_supply: Balance,
}

decl_storage! {
	trait Store for Module<T: Trait> as Assets {
		CountryTokens get(fn get_country_token): map hasher(blake2_128_concat) CountryId => Option<CurrencyId>;
		/// The next asset identifier up for grabs.
		NextTokenId get(fn next_token_id): CurrencyId;
		/// Details of the token corresponding to the token id.
		/// (hash) -> Token details [returns Token struct]
		Tokens get(fn token_details): map hasher(blake2_128_concat) CurrencyId => Token<Balance>;
	}
}

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Transfer amount should be non-zero
		AmountZero,
		/// Account balance must be greater than or equal to the transfer amount
		BalanceLow,
		/// Balance should be non-zero
		BalanceZero,
		/// No permission to issue token
		NoPermissionTokenIssuance,
		/// Country Currency already issued for this country
		TokenAlreadyIssued,
		/// No available next token id
		NoAvailableTokenId,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;
		/// Issue a new class of fungible assets for country. There are, and will only ever be, `total`
		/// such assets and they'll all belong to the `origin` initially. It will have an
		/// identifier `TokenId` instance: this will be specified in the `Issued` event.
		// #[weight = 0]
		// fn issue(origin, country_id: AssetId, name: AssetName, ticker: Ticker ,#[compact] total: Balance) {
		// 	let origin = ensure_signed(origin)?;
		// 	//Check country ownership
		// 	let country_owner = <CountryOwner<T>>::get(country_id).ok_or("Not a country owner of this country")?;
		// 	ensure!(country_owner == origin, Error::<T>::NoPermissionTokenIssuance);

		// 	//Check if country already issued token
		// 	let country_token = Self::get_country_token(country_id);
		// 	ensure!(country_token.is_none(), Error::<T>::TokenAlreadyIssued);

		// 	let id = Self::next_asset_id();
		// 	<NextTokenId<T>>::mutate(|id| *id += One::one());

		// 	<Balances<T>>::insert((&id, &origin), total);
		// 	<TotalSupply<T>>::insert(&id, total);
		// 	<CountryTokens<T>>::insert(country_id,&id);
		// 	let new_token = Token{
		// 		name: name,
		// 		ticker: ticker,
		// 		total_supply: total,
		// 	};

		// 	<Tokens<T>>::insert(&id, new_token);

		// 	Self::deposit_event(RawEvent::Issued(id, origin, total, country_id));
		// }

		#[weight = 10_000]
		fn mint_token(origin, ticker: Ticker, country_id: CountryId, total_supply: Balance) -> DispatchResult{
			let who = ensure_signed(origin)?;
			let country_owner = <CountryOwner<T>>::get(country_id).ok_or("Country is not available")?;

			//Check ownership
			ensure!(<CountryOwner<T>>::contains_key(&country_id, &who), Error::<T>::NoPermissionTokenIssuance);

			//Generate new CurrencyId
			let currency_id = NextTokenId::mutate(|id| -> Result<CurrencyId, DispatchError>{

				let current_id =*id;
				*id = id.checked_add(One::one())
				.ok_or(Error::<T>::NoAvailableTokenId)?;

				Ok(current_id)
			})?;

			let token_info = Token{
				ticker,
				total_supply,
			};

			Tokens::insert(currency_id, token_info);
			CountryTokens::insert(country_id, currency_id);
			//ONly for country owner

			T::CountryCurrency::deposit(currency_id, &who, total_supply)?;

			Self::deposit_event(RawEvent::Issued(who, total_supply));

			Ok(())
		}
	}
}

decl_event! {
	pub enum Event<T> where
		<T as system::Trait>::AccountId,
		Balance = Balance,
		<T as Trait>::TokenId
	{
		/// Some assets were issued. \[asset_id, owner, total_supply\]
		Issued(AccountId, Balance),
		/// Some assets were transferred. \[asset_id, from, to, amount\]
		Transferred(TokenId, AccountId, AccountId, Balance),
		/// Some assets were destroyed. \[asset_id, owner, balance\]
		Destroyed(TokenId, AccountId, Balance),
	}
}

// // The main implementation block for the module.
// impl<T: Trait> Module<T> {
// 	// Public immutables

// 	/// Get the asset `id` balance of `who`.
// 	pub fn balance(id: T::TokenId, who: T::AccountId) -> Balance {
// 		<Balances<T>>::get((id, who))
// 	}

// 	/// Get the total supply of an asset `id`.
// 	pub fn total_supply(id: T::TokenId) -> Balance {
// 		<TotalSupply<T>>::get(id)
// 	}
// }
