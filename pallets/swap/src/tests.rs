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

#[test]
fn add_liquidity_should_fail_with_invalid_pair() {
    ExtBuilder::default().build().execute_with(|| {        
       /// As written code requires NUUM_SOC begins w/ native token (NUUM)
        assert_noop!(
            SwapModule::add_liquidity(ALICE.into(), SOC, SOC_2, 10, 10),
            Error::<Runtime>::InvalidFungileTokenIds
        );
       /// assert_noop!(
       ///     SwapModule::add_liquidity(ALICE.into(), SOC, NUUM, 10, 10),
       ///     Error::<Runtime>::InvalidFungileTokenIds
       /// );
       /// TODO - test NUUM_SOC not enabled
    });
}

#[test]
fn add_liquidity_should_fail_with_insufficient_balances() {
    ExtBuilder::default().build().execute_with(|| {      
       /// NUUM balance too low
        assert_noop!(
            SwapModule::add_liquidity(ALICE.into(), NUUM, SOC, 1000, 10),
            pallet_balances::Error::<Runtime>::InsufficientBalance
        );
       /// SOC balance too low
        assert_noop!(
            SwapModule::add_liquidity(ALICE.into(), NUUM, SOC, 10, 1000),
            orml_tokens::Error::<Runtime>::BalanceTooLow
        );
       /// Below existential deposit
        assert_noop!(
            SwapModule::add_liquidity(ALICE.into(), NUUM, SOC, 100, 10),
            pallet_balances::Error::<Runtime>::KeepAlive
        );
    });
}
        
#[test]
fn add_liquidity_should_not_work_with_zero_liquidity_increment() {
    ExtBuilder::default().build().execute_with(|| {      
       /// Invalid Liquidity Increment
        assert_noop!(
            SwapModule::add_liquidity(ALICE.into(), NUUM, SOC, 0, 10),
            Error::<Runtime>::InvalidLiquidityIncrement
        );
        assert_noop!(
            SwapModule::add_liquidity(ALICE.into(), NUUM, SOC, 10, 0),
            Error::<Runtime>::InvalidLiquidityIncrement
        );
        assert_noop!(
            SwapModule::add_liquidity(ALICE.into(), NUUM, SOC, 0, 0),
            Error::<Runtime>::InvalidLiquidityIncrement
        );
    });
}

#[test]
fn add_liquidity_should_work_with_greater_max_native_amount() {
    ExtBuilder::default().build().execute_with(|| {    
        assert_ok!(SwapModule::add_liquidity(ALICE.into(), NUUM_SOC.0, NUUM_SOC.1, 10, 5));
    
        assert_eq!(SocialCurrencies::total_balance(NUUM_SOC.0, &ALICE), 90); 
        assert_eq!(SocialCurrencies::total_balance(NUUM_SOC.1, &ALICE), 95); 
        assert_eq!(SocialCurrencies::total_balance(SHARE, &ALICE), 20); 
        assert_eq!(SocialCurrencies::total_balance(NUUM_SOC.0, &DEX), 10); 
        assert_eq!(SocialCurrencies::total_balance(NUUM_SOC.1, &DEX), 5); 
        
        assert_eq!(SwapModule::liquidity_pool(NUUM_SOC), (10, 5));
        let event = mock::Event::swap(
            crate::Event::AddLiquidity(ALICE, NUUM_SOC.0, 10, NUUM_SOC.1, 5, 20)
        );
        assert_eq!(last_event(), event);    
    });
}

#[test]
fn add_liquidity_should_work_with_greater_max_social_amount() {
    ExtBuilder::default().build().execute_with(|| {            
        assert_ok!(SwapModule::add_liquidity(ALICE.into(), NUUM_SOC.0, NUUM_SOC.1, 5, 10));
    
        assert_eq!(SocialCurrencies::total_balance(NUUM_SOC.0, &ALICE), 95); 
        assert_eq!(SocialCurrencies::total_balance(NUUM_SOC.1, &ALICE), 90); 
        assert_eq!(SocialCurrencies::total_balance(SHARE, &ALICE), 20); 
        assert_eq!(SocialCurrencies::total_balance(NUUM_SOC.0, &DEX), 5); 
        assert_eq!(SocialCurrencies::total_balance(NUUM_SOC.1, &DEX), 10); 
        
        assert_eq!(SwapModule::liquidity_pool(NUUM_SOC), (5, 10));
        let event = mock::Event::swap(
            crate::Event::AddLiquidity(ALICE, NUUM_SOC.0, 5, NUUM_SOC.1, 10, 20)
        );
        assert_eq!(last_event(), event);    
    });
}

#[test]
fn add_liquidity_should_work_when_share_issuance_greater_than_zero() {
    ExtBuilder::default().build().execute_with(|| {                            
        assert_ok!(SwapModule::add_liquidity(ALICE.into(), NUUM_SOC.0, NUUM_SOC.1, 5, 5));            
        let event = mock::Event::swap(
            crate::Event::AddLiquidity(ALICE, NUUM_SOC.0, 5, NUUM_SOC.1, 5, 10)
        );
        assert_eq!(last_event(), event);    

        assert_ok!(SwapModule::add_liquidity(ALICE.into(), NUUM_SOC.0, NUUM_SOC.1, 5, 5));            
        let event = mock::Event::swap(
            crate::Event::AddLiquidity(ALICE, NUUM_SOC.0, 5, NUUM_SOC.1, 5, 10)
        );
        assert_eq!(SwapModule::liquidity_pool(NUUM_SOC), (10, 10));        
        assert_eq!(SocialCurrencies::total_balance(NUUM_SOC.0, &ALICE), 90); 
        assert_eq!(SocialCurrencies::total_balance(NUUM_SOC.1, &ALICE), 90); 
        assert_eq!(SocialCurrencies::total_balance(SHARE, &ALICE), 20); 
        assert_eq!(SocialCurrencies::total_balance(NUUM_SOC.0, &DEX), 10); 
        assert_eq!(SocialCurrencies::total_balance(NUUM_SOC.1, &DEX), 10); 
        assert_eq!(last_event(), event);    
    });
}

#[test]
fn remove_liquidity_should_fail_with_invalid_pair() {
    ExtBuilder::default().build().execute_with(|| {        
       /// As written code requires NUUM_SOC begins w/ native token (NUUM)
        assert_noop!(
            SwapModule::remove_liquidity(ALICE.into(), SOC, SOC_2, 10),
            Error::<Runtime>::InvalidFungileTokenIds
        );
       /// assert_noop!(
       ///     SwapModule::remove_liquidity(ALICE.into(), SOC, NUUM, 10),
       ///     Error::<Runtime>::InvalidFungileTokenIds
       /// );
       /// TODO - Test NUUM_SOC not enabled
    });
}

#[test]
fn remove_liquidity_should_fail_with_insufficient_balances () {
    ExtBuilder::default().build().execute_with(|| {                
        assert_ok!(
            SwapModule::add_liquidity(ALICE.into(), NUUM_SOC.0, NUUM_SOC.1, 1, 1)
        );            

       /// ALICE attempts to take more shares than she owns:
        assert_noop!(
            SwapModule::remove_liquidity(ALICE.into(), NUUM, SOC, 10),
            orml_tokens::Error::<Runtime>::BalanceTooLow
        );        
       /// ALICE's drops dex account below existential deposit
        assert_noop!(
            SwapModule::remove_liquidity(ALICE.into(), NUUM, SOC, 2),
            pallet_balances::Error::<Runtime>::KeepAlive
        );
       /// Make sure state unchanged
        assert_eq!(SwapModule::liquidity_pool(NUUM_SOC), (1, 1));        
        assert_eq!(SocialCurrencies::total_balance(NUUM_SOC.0, &ALICE), 99); 
        assert_eq!(SocialCurrencies::total_balance(NUUM_SOC.1, &ALICE), 99); 
        assert_eq!(SocialCurrencies::total_balance(SHARE, &ALICE), 2); 
        assert_eq!(SocialCurrencies::total_balance(NUUM_SOC.0, &DEX), 1); 
        assert_eq!(SocialCurrencies::total_balance(NUUM_SOC.1, &DEX), 1);        
    });
}

#[test]
fn remove_liquidity_should_work() {
    ExtBuilder::default().build().execute_with(|| {                        
        assert_ok!(SwapModule::add_liquidity(ALICE.into(), NUUM_SOC.0, NUUM_SOC.1, 10, 5));
        assert_ok!(SwapModule::add_liquidity(BOB.into(), NUUM_SOC.0, NUUM_SOC.1, 50, 20));            

        let event = mock::Event::swap(
            crate::Event::RemoveLiquidity(ALICE, NUUM_SOC.0, 2, NUUM_SOC.1, 1, 5)
        );
        
        assert_ok!(SwapModule::remove_liquidity(ALICE.into(), NUUM, SOC, 5));
        assert_eq!(last_event(), event);                ;                  
    });
}

#[test]
fn swap_native_token_with_exact_supply_should_fail() {
    ExtBuilder::default().build().execute_with(|| {                        
        assert_noop!(
            SwapModule::swap_native_token_with_exact_supply(ALICE.into(), SOC, NUUM, 1, 1),
            Error::<Runtime>::InvalidTradingCurrency
        );

        assert_noop!(
            SwapModule::swap_native_token_with_exact_supply(ALICE.into(), NUUM, SOC, 1, 1),
            Error::<Runtime>::InsufficientLiquidity
        );

        assert_ok!(SwapModule::add_liquidity(ALICE.into(), NUUM, SOC, 50, 50));        

        assert_noop!(
            SwapModule::swap_native_token_with_exact_supply(ALICE.into(), NUUM, SOC, 5, 10),
            Error::<Runtime>::InsufficientTargetAmount
        );        
       /// TODO - Test NUUM_SOC not enabled
    });
}

#[test]
fn swap_native_token_with_exact_supply_should_work() {
    ExtBuilder::default().build().execute_with(|| {                        
        assert_ok!(SwapModule::add_liquidity(ALICE.into(), NUUM, SOC, 50, 50));

        assert_ok!(SwapModule::swap_native_token_with_exact_supply(BOB.into(), NUUM, SOC, 10, 7));
        let event = mock::Event::swap(
            crate::Event::Swap(BOB, vec![NUUM, SOC], 10, 7)
        );

        assert_eq!(last_event(), event);
        assert_eq!(SwapModule::liquidity_pool(NUUM_SOC), (60, 43));        
        assert_eq!(SocialCurrencies::total_balance(NUUM, &BOB), 90); 
        assert_eq!(SocialCurrencies::total_balance(SOC, &BOB), 107); 
    });
}

#[test]
fn swap_social_token_with_exact_native_token_should_fail() {
    ExtBuilder::default().build().execute_with(|| {                        
        assert_noop!(
            SwapModule::swap_social_token_with_exact_native_token(ALICE.into(), NUUM, SOC, 1, 1),
            Error::<Runtime>::InvalidTradingCurrency
        );

        assert_noop!(
            SwapModule::swap_social_token_with_exact_native_token(ALICE.into(), SOC, NUUM, 1, 1),
            Error::<Runtime>::InsufficientLiquidity
        );

        assert_ok!(SwapModule::add_liquidity(ALICE.into(), NUUM, SOC, 50, 50));        
            
        assert_noop!(
            SwapModule::swap_social_token_with_exact_native_token(ALICE.into(), SOC, NUUM, 10, 5),
            Error::<Runtime>::TooMuchSupplyAmount
        );        
    });
}

#[test]
fn swap_social_token_with_exact_native_token_should_work() {
    ExtBuilder::default().build().execute_with(|| {                        
        assert_ok!(SwapModule::add_liquidity(ALICE.into(), NUUM, SOC, 50, 50));

        assert_ok!(
            SwapModule::swap_social_token_with_exact_native_token(BOB.into(), SOC, NUUM, 7, 10)
        );
        
        let event = mock::Event::swap(
            crate::Event::Swap(BOB, vec![SOC, NUUM], 9, 7)
        );

        assert_eq!(last_event(), event);
        assert_eq!(SwapModule::liquidity_pool(NUUM_SOC), (43, 59));        
        assert_eq!(SocialCurrencies::total_balance(NUUM, &BOB), 107);         
        assert_eq!(SocialCurrencies::total_balance(SOC, &BOB), 91);         
    });
}