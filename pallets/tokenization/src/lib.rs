// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use country::CountryOwner;
use frame_support::{decl_error, decl_event, decl_module, decl_storage, ensure, Parameter};
use frame_system::{self as system, ensure_signed};
use orml_traits::{
	MultiCurrency, MultiCurrencyExtended, MultiLockableCurrency, MultiReservableCurrency,
};
use sp_runtime::{traits::{
	AtLeast32Bit, AtLeast32BitUnsigned, Hash, Member, One, StaticLookup, Zero,
}, DispatchResult };
use sp_std::vec::Vec;
use primitives::Balance;
use unique_asset::AssetId;

/// The module configuration trait.
pub trait Trait: system::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	/// The arithmetic type of asset identifier.
	type TokenId: Parameter + AtLeast32Bit + Default + Copy;
	type CountryCurrency: MultiCurrency<Self::AccountId>;
}
/// A wrapper for a asset name.
pub type AssetName = Vec<u8>;
/// A wrapper for a ticker name.
pub type Ticker = Vec<u8>;

type CurrencyIdOf<T> = <<T as Trait>::CountryCurrency as MultiCurrency<
	<T as frame_system::Trait>::AccountId,
>>::CurrencyId;
type BalanceOf<T> = <<T as Trait>::CountryCurrency as MultiCurrency<
	<T as frame_system::Trait>::AccountId,
>>::Balance;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Token<Balance> {
	pub name: AssetName,
	pub ticker: Ticker,
	pub total_supply: Balance,
}

decl_storage! {
	trait Store for Module<T: Trait> as Assets {
		CountryTokens get(fn get_country_token): map hasher(blake2_128_concat) AssetId => Option<T::TokenId>;
		/// The number of units of assets held by any given account.
		Balances: map hasher(blake2_128_concat) (T::TokenId, T::AccountId) => Balance;
		/// The next asset identifier up for grabs.
		NextTokenId get(fn next_asset_id): T::TokenId;
		/// The total unit supply of an asset.
		/// TWOX-NOTE: `TokenId` is trusted, so this is safe.
		TotalSupply: map hasher(twox_64_concat) T::TokenId => Balance;
		/// Details of the token corresponding to the token id.
		/// (hash) -> Token details [returns Token struct]
		Tokens get(fn token_details): map hasher(blake2_128_concat) T::TokenId => Token<Balance>;
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
		fn mint_token(origin, currency_id: CurrencyIdOf<T>, balance: BalanceOf<T>) -> DispatchResult{
			let who = ensure_signed(origin)?;
			//Generate new CurrencyId
			T::CountryCurrency::deposit(currency_id, &who, balance)?;

			Self::deposit_event(RawEvent::Issued(who, balance));

			Ok(())
		}

		/// Move some assets from one holder to another.
		///
		/// # <weight>
		/// - `O(1)`
		/// - 1 static lookup
		/// - 2 storage mutations (codec `O(1)`).
		/// - 1 event.
		/// # </weight>
		#[weight = 0]
		fn transfer(origin,
			id: T::TokenId,
			target: <T::Lookup as StaticLookup>::Source,
			amount: Balance
		) {
			let origin = ensure_signed(origin)?;
			let origin_account = (id, origin.clone());
			let origin_balance = <Balances<T>>::get(&origin_account);
			let target = T::Lookup::lookup(target)?;
			ensure!(!amount.is_zero(), Error::<T>::AmountZero);
			ensure!(origin_balance >= amount, Error::<T>::BalanceLow);

			// Self::deposit_event(RawEvent::Transferred(id, origin, target.clone(), amount));
			<Balances<T>>::insert(origin_account, origin_balance - amount);
			<Balances<T>>::mutate((id, target), |balance| *balance += amount);
		}

		/// Destroy any assets of `id` owned by `origin`.
		///
		/// # <weight>
		/// - `O(1)`
		/// - 1 storage mutation (codec `O(1)`).
		/// - 1 storage deletion (codec `O(1)`).
		/// - 1 event.
		/// # </weight>
		#[weight = 0]
		fn destroy(origin, id: T::TokenId) {
			let origin = ensure_signed(origin)?;
			let balance = <Balances<T>>::take((id, &origin));
			ensure!(!balance.is_zero(), Error::<T>::BalanceZero);

			<TotalSupply<T>>::mutate(id, |total_supply| *total_supply -= balance);
			// Self::deposit_event(RawEvent::Destroyed(id, origin, balance));
		}
	}
}

decl_event! {
	pub enum Event<T> where
		<T as system::Trait>::AccountId,
		Balance = BalanceOf<T>,
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
