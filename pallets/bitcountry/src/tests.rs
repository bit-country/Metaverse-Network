// This file is part of Bit.Country.

// Copyright (C) 2020-2021 Bit.Country.
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

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};
use sp_core::blake2_256;
use sp_runtime::traits::BadOrigin;
use pallet_nft::CollectionType;
use ownership_manager::TokenType;

fn init_nft_collection(owner: Origin) {
    assert_ok!(Nft::create_group(
        Origin::root(),
        vec![1],
        vec![1],
    ));
    assert_ok!(Nft::create_class(
        owner.clone(),
        vec![1],        
        COLLECTION_ID,
        TokenType::Transferable,
        CollectionType::Collectable,
    ));
    // assert_ok!(Nft::mint(
    //     owner.clone(),
    //     CLASS_ID,
    //     vec![1],
    //     vec![1],
    //     vec![1],
    //     1
    // ));
}


#[test]
fn create_bc_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(CountryModule::create_bc(
            Origin::signed(ALICE),
            vec![1]
        ));
        assert_eq!(
            CountryModule::get_country(&COUNTRY_ID),
            Some(Country {
                ownership_id: OwnershipId::Standard(ALICE),
                metadata: vec![1],
                currency_id: SocialTokenCurrencyId::SocialToken(0),
            })
        );
        let event = Event::bitcountry(crate::Event::NewCountryCreated(COUNTRY_ID));
        assert_eq!(last_event(), event);
    });
}

#[test]
fn create_bc_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            CountryModule::create_bc(Origin::none(), vec![1],),
            BadOrigin
        );
    });
}

#[test]
fn transfer_country_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(CountryModule::create_bc(
            Origin::signed(ALICE),
            vec![1]
        ));
        assert_ok!(CountryModule::transfer_country(
            Origin::signed(ALICE),
            BOB,
            COUNTRY_ID
        ));
        let event = Event::bitcountry(crate::Event::TransferredCountry(COUNTRY_ID, ALICE, BOB));
        assert_eq!(last_event(), event);
        //Make sure 2 ways transfer works
        assert_ok!(CountryModule::transfer_country(
            Origin::signed(BOB),
            ALICE,
            COUNTRY_ID
        ));
        let event = Event::bitcountry(crate::Event::TransferredCountry(COUNTRY_ID, BOB, ALICE));
        assert_eq!(last_event(), event);
    })
}


#[test]
fn transfer_tokenized_country_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        init_nft_collection(Origin::signed(ALICE));        
        assert_ok!(CountryModule::create_bc(Origin::signed(ALICE), vec![1]));
        assert_ok!(CountryModule::tokenize_ownership(Origin::signed(ALICE), COUNTRY_ID));
        assert_ok!(CountryModule::transfer_country(Origin::signed(ALICE), BOB, COUNTRY_ID));        
        
        assert_eq!(
            CountryModule::get_country_owner(COUNTRY_ID, OwnershipId::Token(COUNTRY_ID)),
            Some(())
        );

        assert_eq!(CountryModule::check_ownership(&BOB, &COUNTRY_ID), true);

        let event = Event::bitcountry(crate::Event::TransferredCountry(COUNTRY_ID, ALICE, BOB));
        assert_eq!(last_event(), event);
        //Make sure 2 ways transfer works
        assert_ok!(CountryModule::transfer_country(
            Origin::signed(BOB),
            ALICE,
            COUNTRY_ID
        ));
        assert_eq!(
            CountryModule::get_country_owner(COUNTRY_ID, OwnershipId::Token(COUNTRY_ID)),
            Some(())
        );
        
        assert_eq!(CountryModule::check_ownership(&ALICE, &COUNTRY_ID), true);
        let event = Event::bitcountry(crate::Event::TransferredCountry(COUNTRY_ID, BOB, ALICE));
        assert_eq!(last_event(), event);
    })
}



#[test]
fn transfer_country_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(CountryModule::create_bc(
            Origin::signed(ALICE),
            vec![1]
        ));
        assert_noop!(
            CountryModule::transfer_country(Origin::signed(BOB), ALICE, COUNTRY_ID),
            Error::<Runtime>::NoPermission
        );
    })
}

#[test]
fn freeze_country_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(CountryModule::create_bc(
            Origin::signed(ALICE),
            vec![1]
        ));
        assert_ok!(CountryModule::freeze_country(Origin::root(), COUNTRY_ID));
        let event = Event::bitcountry(crate::Event::CountryFreezed(COUNTRY_ID));
        assert_eq!(last_event(), event);
    })
}

#[test]
fn freeze_country_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(CountryModule::create_bc(
            Origin::signed(ALICE),
            vec![1]
        ));
        //Country owner tries to freeze their own bitcountry
        assert_noop!(
            CountryModule::freeze_country(Origin::signed(ALICE), COUNTRY_ID),
            BadOrigin
        );
    })
}

#[test]
fn unfreeze_country_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(CountryModule::create_bc(
            Origin::signed(ALICE),
            vec![1]
        ));
        assert_ok!(CountryModule::freeze_country(Origin::root(), COUNTRY_ID));
        let event = Event::bitcountry(crate::Event::CountryFreezed(COUNTRY_ID));
        assert_eq!(last_event(), event);
        assert_ok!(CountryModule::unfreeze_country(Origin::root(), COUNTRY_ID));
        let event = Event::bitcountry(crate::Event::CountryUnFreezed(COUNTRY_ID));
        assert_eq!(last_event(), event);
    })
}

#[test]
fn destroy_country_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(CountryModule::create_bc(
            Origin::signed(ALICE),
            vec![1]
        ));
        assert_ok!(CountryModule::destroy_country(Origin::root(), COUNTRY_ID));
        let event = Event::bitcountry(crate::Event::CountryDestroyed(COUNTRY_ID));
        assert_eq!(last_event(), event);
    })
}

#[test]
fn destroy_country_without_root_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(CountryModule::create_bc(
            Origin::signed(ALICE),
            vec![1]
        ));
        assert_noop!(
            CountryModule::destroy_country(Origin::signed(ALICE), COUNTRY_ID),
            BadOrigin
        );
    })
}

#[test]
fn destroy_country_with_no_id_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(CountryModule::create_bc(
            Origin::signed(ALICE),
            vec![1]
        ));
        assert_noop!(
            CountryModule::destroy_country(Origin::root(), COUNTRY_ID_NOT_EXIST),
            Error::<Runtime>::CountryInfoNotFound
        );
    })
}


#[test]
fn tokenize_ownership_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        // Setup & verify standard ownership
        init_nft_collection(Origin::signed(ALICE));
        assert_ok!(CountryModule::create_bc(
            Origin::signed(ALICE),
            vec![1]
        ));
        assert_eq!(CountryModule::check_ownership(&ALICE, &COUNTRY_ID), true);
        assert_eq!(
            CountryModule::get_country_owner(COUNTRY_ID, OwnershipId::Standard(ALICE)),
            Some(())
        );
        assert_eq!(
            CountryModule::get_country_owner(COUNTRY_ID, OwnershipId::Token(COUNTRY_ID)),
            None
        );
        let country = CountryModule::get_country(COUNTRY_ID);
        assert_eq!(country.unwrap().ownership_id, OwnershipId::Standard(1));

        // Verify tokenized ownership
        assert_ok!(CountryModule::tokenize_ownership(Origin::signed(ALICE), COUNTRY_ID));
        assert_eq!(
            CountryModule::get_country_owner(COUNTRY_ID, OwnershipId::Token(COUNTRY_ID)),
            Some(())
        );
        assert_eq!(
            CountryModule::get_country_owner(COUNTRY_ID, OwnershipId::Standard(ALICE)),
            None
        );
        let country = CountryModule::get_country(COUNTRY_ID);
        assert_eq!(country.unwrap().ownership_id, OwnershipId::Token(TOKEN_ID));

        let event = Event::bitcountry(crate::Event::CountryOwnershipTokenized(COUNTRY_ID, OwnershipId::Token(TOKEN_ID)));        
        assert_eq!(last_event(), event);
        assert_eq!(CountryModule::check_ownership(&ALICE, &COUNTRY_ID), true);
    })
}

#[test]
fn detokenize_ownership_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        // Setup, tokenize, & verify ownership
        init_nft_collection(Origin::signed(ALICE));
        assert_ok!(CountryModule::create_bc(Origin::signed(ALICE), vec![1]));
        assert_ok!(CountryModule::tokenize_ownership(Origin::signed(ALICE), COUNTRY_ID));
        assert_eq!(
            CountryModule::get_country_owner(COUNTRY_ID, OwnershipId::Standard(ALICE)),
            None
        );
        assert_eq!(
            CountryModule::get_country_owner(COUNTRY_ID, OwnershipId::Token(COUNTRY_ID)),
            Some(())
        );
        let country = CountryModule::get_country(COUNTRY_ID);
        assert_eq!(country.unwrap().ownership_id, OwnershipId::Token(TOKEN_ID));

        // Detokenize & verify ownership
        assert_ok!(CountryModule::detokenize_ownership(Origin::signed(ALICE), COUNTRY_ID));
        assert_eq!(
            CountryModule::get_country_owner(COUNTRY_ID, OwnershipId::Standard(ALICE)),
            Some(())
        );
        assert_eq!(
            CountryModule::get_country_owner(COUNTRY_ID, OwnershipId::Token(COUNTRY_ID)),
            None
        );
        let country = CountryModule::get_country(COUNTRY_ID);
        assert_eq!(country.unwrap().ownership_id, OwnershipId::Standard(ALICE));

        let event = Event::bitcountry(crate::Event::CountryOwnershipDetokenized(COUNTRY_ID, OwnershipId::Standard(ALICE)));        
        assert_eq!(last_event(), event);
        assert_eq!(CountryModule::check_ownership(&ALICE, &COUNTRY_ID), true);
    })
}

#[test]
fn tokenize_ownership_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        init_nft_collection(Origin::signed(ALICE));
        // Country doesn't exist yet:
        assert_noop!(
            CountryModule::tokenize_ownership(Origin::signed(ALICE), COUNTRY_ID),
            Error::<Runtime>::CountryNotExists
        );

        assert_ok!(CountryModule::create_bc(Origin::signed(ALICE), vec![1]));
        
        // Bob not owner:
        assert_noop!(
            CountryModule::tokenize_ownership(Origin::signed(BOB), COUNTRY_ID),
            Error::<Runtime>::NoPermission
        );

        // Already tokenized:
        assert_ok!(CountryModule::tokenize_ownership(Origin::signed(ALICE), COUNTRY_ID));
        assert_noop!(
            CountryModule::tokenize_ownership(Origin::signed(ALICE), COUNTRY_ID),
            Error::<Runtime>::OwnershipAlreadyTokenized
        );
    })
}

#[test]
fn detokenize_ownership_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        init_nft_collection(Origin::signed(ALICE));
        // Country doesn't exist yet:
        assert_noop!(
            CountryModule::detokenize_ownership(Origin::signed(ALICE), COUNTRY_ID),
            Error::<Runtime>::CountryNotExists
        );

        assert_ok!(CountryModule::create_bc(Origin::signed(ALICE), vec![1]));
        
        // Bob not owner:
        assert_noop!(
            CountryModule::detokenize_ownership(Origin::signed(BOB), COUNTRY_ID),
            Error::<Runtime>::NoPermission
        );

        // Already detokenized /standard:        
        assert_noop!(
            CountryModule::detokenize_ownership(Origin::signed(ALICE), COUNTRY_ID),
            Error::<Runtime>::OwnershipAlreadyDeTokenized
        );
    })
}