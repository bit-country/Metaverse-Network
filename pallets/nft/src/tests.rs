#[cfg(test)]
use super::*;
use mock::*;

use frame_support::{assert_noop, assert_ok};

fn free_balance(who: &AccountId) -> Balance {
    <Runtime as Config>::Currency::free_balance(who)
}

fn reserved_balance(who: &AccountId) -> Balance {
    <Runtime as Config>::Currency::reserved_balance(who)
}

fn class_id_account() -> AccountId {
    <Runtime as Config>::ModuleId::get().into_sub_account(CLASS_ID)
}

#[test]
fn create_class_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(NftModule::create_class(
			Origin::signed(ALICE),
			vec![1],
            vec![1],
            TokenType::Transferrable,
            CollectionType::Collectable,
		));
        let event = Event::nft(crate::Event::CreatedClass(class_id_account(), CLASS_ID));
        assert_eq!(last_event(), event);

        assert_eq!(
            reserved_balance(&class_id_account()),
            <Runtime as Config>::CreateClassDeposit::get()
        );
    });
}