#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_module, decl_storage, decl_event,
    dispatch::{DispatchError, DispatchResult}, ensure,
    traits::{
        Currency, Get,
        Imbalance, OnUnbalanced,
    },
};
use sp_runtime::RuntimeDebug;
use sp_std::{
    collections::btree_set::BTreeSet,
    prelude::*,
};
use frame_system::{self as system};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

pub trait Trait: system::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    /// The currency mechanism.
    type Currency: Currency<Self::AccountId>;
}

decl_event!(
    pub enum Event<T> where Balance = BalanceOf<T>
    {
		Deposit(Balance),
    }
);