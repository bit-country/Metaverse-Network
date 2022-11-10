// This file is part of Metaverse.Network & Bit.Country.

// Copyright (C) 2020-2022 Metaverse.Network & Bit.Country .
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use core::ops::Range;

use codec::{Decode, Encode, MaxEncodedLen};
use ethabi::Token;
use hex_literal::hex;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::{H160, U256};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

use crate::FungibleTokenId;

/// Evm Address.
pub type EvmAddress = sp_core::H160;

/// H160 CurrencyId Type enum
#[derive(
	Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord, TryFromPrimitive, IntoPrimitive, TypeInfo,
)]
#[repr(u8)]
pub enum CurrencyIdType {
	NativeToken = 1, // 0 is prefix of precompile and predeploy
	FungibleToken,
	MiningResource,
}

/// A mapping between FungibleTokenId and Erc20 address.
/// provide a way to encode/decode for FungibleTokenId;
pub trait Erc20Mapping {
	/// Encode the FungibleTokenId to EvmAddress.
	fn encode_evm_address(v: FungibleTokenId) -> Option<EvmAddress>;
	/// Decode the FungibleTokenId from EvmAddress.
	fn decode_evm_address(v: EvmAddress) -> Option<FungibleTokenId>;
}

#[rustfmt::skip]
/// FungibleCurrencyId to H160([u8; 20]) bit encoding rule.
///
/// Type occupies 1 byte, and data occupies 4 bytes(less than 4 bytes, right justified).
///
/// 0x0000000000000000000000000000000000000000
///    0 1 2 3 4 5 6 7 8 910111213141516171819 index
///   ^^^^^^^^^^^^^^^^^^                       System contract address prefix
///                     ^^                     CurrencyId Type: 1-NativeToken 2-FungibleToken 3-DexShare(No use)
///                                                             4-MiningResource
///                                         ^^ CurrencyId Type is 1-NativeToken
///                               ^^^^^^^^^^^^ CurrencyId Type is 1-NativeToken, NFT
///                       ^^^^^^^^             CurrencyId Type is 2-FungibleToken
///                               ^^^^^^^^^^^^ CurrencyId Type is 3-MiningResource

//pub const METAVERSE_CHAIN_ID: u64 = 2042;
//pub const PIONEER_CHAIN_ID: u64 = 137;

pub const H160_POSITION_CURRENCY_ID_TYPE: usize = 9;
pub const H160_POSITION_TOKEN: usize = 19;
pub const H160_POSITION_TOKEN_NFT: Range<usize> = 14..20;
pub const H160_POSITION_FUNGIBLE_TOKEN: Range<usize> = 10..13;
pub const H160_POSITION_MINING_RESOURCE: Range<usize> = 14..20;

/// Generate the EvmAddress from FungibleTokenId so that evm contracts can call the erc20 contract.
/// NOTE: Can not be used directly, need to check the erc20 is mapped.
impl TryFrom<FungibleTokenId> for EvmAddress {
	type Error = ();

	fn try_from(val: FungibleTokenId) -> Result<Self, Self::Error> {
		let mut address = [0u8; 20];
		match val {
			FungibleTokenId::NativeToken(token_id) => {
				address[H160_POSITION_CURRENCY_ID_TYPE] = CurrencyIdType::NativeToken.into();
				address[H160_POSITION_TOKEN] = token_id as u8;
			}
			//			FungibleTokenId::Dex(erc20) => {
			//				address[..].copy_from_slice(erc20.as_bytes());
			//			}
			FungibleTokenId::FungibleToken(token_id) => {
				address[H160_POSITION_CURRENCY_ID_TYPE] = CurrencyIdType::FungibleToken.into();
				address[H160_POSITION_TOKEN] = token_id as u8;
			}
			FungibleTokenId::MiningResource(token_id) => {
				address[H160_POSITION_CURRENCY_ID_TYPE] = CurrencyIdType::MiningResource.into();
				address[H160_POSITION_TOKEN] = token_id as u8;
			}
			FungibleTokenId::Stable(token_id) => {
				address[H160_POSITION_CURRENCY_ID_TYPE] = CurrencyIdType::FungibleToken.into();
				address[H160_POSITION_TOKEN] = token_id as u8;
			}
			_ => {}
		};

		Ok(EvmAddress::from_slice(&address))
	}
}

pub struct Output;

impl Output {
	pub fn encode_bool(b: bool) -> Vec<u8> {
		ethabi::encode(&[Token::Bool(b)])
	}

	pub fn encode_uint<T>(b: T) -> Vec<u8>
	where
		U256: From<T>,
	{
		ethabi::encode(&[Token::Uint(U256::from(b))])
	}

	pub fn encode_uint_tuple<T>(b: Vec<T>) -> Vec<u8>
	where
		U256: From<T>,
	{
		ethabi::encode(&[Token::Tuple(b.into_iter().map(U256::from).map(Token::Uint).collect())])
	}

	pub fn encode_uint_array<T>(b: Vec<T>) -> Vec<u8>
	where
		U256: From<T>,
	{
		ethabi::encode(&[Token::Array(b.into_iter().map(U256::from).map(Token::Uint).collect())])
	}

	pub fn encode_bytes(b: &[u8]) -> Vec<u8> {
		ethabi::encode(&[Token::Bytes(b.to_vec())])
	}

	pub fn encode_fixed_bytes(b: &[u8]) -> Vec<u8> {
		ethabi::encode(&[Token::FixedBytes(b.to_vec())])
	}

	pub fn encode_address(b: H160) -> Vec<u8> {
		ethabi::encode(&[Token::Address(b)])
	}

	pub fn encode_address_tuple(b: Vec<H160>) -> Vec<u8> {
		ethabi::encode(&[Token::Tuple(b.into_iter().map(Token::Address).collect())])
	}

	pub fn encode_address_array(b: Vec<H160>) -> Vec<u8> {
		ethabi::encode(&[Token::Array(b.into_iter().map(Token::Address).collect())])
	}
}
