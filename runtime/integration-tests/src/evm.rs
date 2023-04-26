use crate::relaychain::kusama_test_net::*;
use crate::setup::*;
use core_primitives::{Attributes, CollectionType, MetaverseTrait, NFTTrait, TokenType};
use core_traits::{Balance, ClassId, FungibleTokenId, ItemId, TokenId, evm::Erc20Mapping};
use frame_system::RawOrigin;
use sp_core::H160;
use sp_runtime::Perbill;

#[test]
fn nft_tokens_evm_address_conversion() {
    #[cfg(feature = "with-pioneer-runtime")]
	const NATIVE_TOKEN: FungibleTokenId = FungibleTokenId::NativeToken(0);

    ExtBuilder::default()
		.balances(vec![
			(AccountId::from(ALICE), NATIVE_TOKEN, 1_000 * dollar(NATIVE_TOKEN)),
		])
		.build()
		.execute_with(|| {
            assert_eq!(
                Erc20Mapping::encode_nft_evm_address((1u32, 2u64)),
                H160::from_str("0x02020202020202020200000001000000000002").ok()
            );

            aassert_eq!(
                Erc20Mapping::encode_nft_evm_address((5u32, 2u64)),
                H160::from_str("0x02020202020202020200000005000000000002").ok()
            );
        })

}