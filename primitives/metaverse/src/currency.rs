// create_currency_id! {
// 	// Represent a Token symbol with 8 bit
// 	// Bit 8 : 0 for Pokladot Ecosystem, 1 for Kusama Ecosystem
// 	// Bit 7 : Reserved
// 	// Bit 6 - 1 : The token ID
// 	#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord, TypeInfo)]
// 	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
// 	#[repr(u8)]
// 	pub enum TokenSymbol {
// 		ASG("Asgard", 12) = 0,
// 		BNC("Bifrost", 12) = 1,
// 		KUSD("Karura Dollar", 12) = 2,
// 		DOT("Polkadot", 10) = 3,
// 		KSM("Kusama", 12) = 4,
// 		ETH("Ethereum", 18) = 5,
// 		KAR("Karura", 12) = 6,
// 		ZLK("Zenlink Network Token", 18) = 7,
// 		PHA("Phala Native Token", 12) = 8,
// 		RMRK("RMRK Token",10) = 9,
// 		NEER("Pioneer Token",10) = 10,
// 	}
// }
//
// impl Default for TokenSymbol {
//     fn default() -> Self {
//         Self::BNC
//     }
// }
//
// /// Currency ID, it might be extended with more variants in the future.
// #[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord, TypeInfo)]
// #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
// #[non_exhaustive]
// pub enum CurrencyId {
//     Native(TokenSymbol),
//     VToken(TokenSymbol),
//     Token(TokenSymbol),
//     Stable(TokenSymbol),
//     VSToken(TokenSymbol),
//     VSBond(TokenSymbol, ParaId, LeasePeriod, LeasePeriod),
//     // [currency1 Tokensymbol, currency1 TokenType, currency2 TokenSymbol, currency2 TokenType]
//     LPToken(TokenSymbol, u8, TokenSymbol, u8),
// }
//
// impl Default for CurrencyId {
//     fn default() -> Self {
//         Self::Native(Default::default())
//     }
// }
//
// impl From<TokenSymbol> for CurrencyId {
//     fn from(symbol: TokenSymbol) -> Self {
//         Self::Token(symbol)
//     }
// }
