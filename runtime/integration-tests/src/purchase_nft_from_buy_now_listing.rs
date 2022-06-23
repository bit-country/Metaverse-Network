use crate::setup::*;

#[test]
fn test_list_nft() {
	ExtBuilder::default()
		.balances(vec![
			(
				AccountId::from(ALICE),
				NATIVE_CURRENCY,
				1_000 * dollar(NATIVE_CURRENCY),
			),
			(
				AccountId::from(BOB),
				NATIVE_CURRENCY,
				1_000 * dollar(NATIVE_CURRENCY),
			),
			(
				AccountId::from(CHARLIE),
				NATIVE_CURRENCY,
				1_000 * dollar(NATIVE_CURRENCY),
			),
		])
		.build()
		.execute_with(|| {
			let metadata = vec![1];
			assert_eq!(
				Balances::free_balance(AccountId::from(ALICE)),
				1_000 * dollar(NATIVE_CURRENCY)
			);
			assert_eq!(
				Balances::free_balance(AccountId::from(BOB)),
				1_000 * dollar(NATIVE_CURRENCY)
			);
			assert_eq!(
				Balances::free_balance(AccountId::from(CHARLIE)),
				1_000 * dollar(NATIVE_CURRENCY)
			);
            // create metaverse
            // create nft group
            // create nft class
            // mint nft
            // list nft as a buy now on a metaverse
            // buy the nft
		});
}
