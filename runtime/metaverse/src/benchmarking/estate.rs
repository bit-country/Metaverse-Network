#![cfg(feature = "runtime-benchmarks")]
use crate::{
    Balances, Call, Currencies, Event, Estate, Metaverse, Nft, Runtime, System,
};
use super::utils::{
    create_metaverse_for_account, dollar, set_balance, mint_NFT, 
    create_land_and_estate_groups, issue_new_undeployed_land_block,
};
use estate::Config;
use frame_benchmarking::{account, whitelisted_caller};
use orml_benchmarking::runtime_benchmarks;
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;
use primitives::estate::{EstateInfo, MintingRateConfig, MintingRateInfo, OwnerId, Round};
use primitives::staking::RoundInfo;
use primitives::{Balance, FungibleTokenId, TokenId};
use sp_runtime::traits::{AccountIdConversion, Lookup, StaticLookup, UniqueSaturatedInto};
use primitives::UndeployedLandBlockType;

pub type AccountId = u128;
pub type LandId = u64;
pub type EstateId = u64;

const SEED: u32 = 0;

const METAVERSE_ID: u64 = 1;
const ALICE: AccountId = 1;
const BENEFICIARY_ID: AccountId = 99;

const MAX_BOUND: (i32, i32) = (-100, 100);
const COORDINATE_IN_1: (i32, i32) = (-10, 10);
const COORDINATE_IN_2: (i32, i32) = (-5, 5);
const COORDINATE_OUT: (i32, i32) = (0, 101);
const COORDINATE_IN_AUCTION: (i32, i32) = (99, 99);
const ESTATE_IN_AUCTION: EstateId = 99;
const ESTATE_ID: EstateId = 0;

runtime_benchmarks! {
	{ Runtime, estate }
    // set_max_bounds
	set_max_bounds{
	}: _(RawOrigin::Root, METAVERSE_ID, MAX_BOUND)
	verify {
		assert_eq!(Estate::get_max_bounds(METAVERSE_ID), MAX_BOUND)
	}

	// mint_land as tokens
	mint_land {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		Estate::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
		create_land_and_estate_groups();
	}: _(RawOrigin::Root, caller.clone(), METAVERSE_ID, COORDINATE_IN_1)
	verify {
		assert_eq!(Estate::get_land_units(METAVERSE_ID, COORDINATE_IN_1), Some(OwnerId::Token(0)));
	}

	// mint_lands as tokens
	mint_lands {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		Estate::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
		create_land_and_estate_groups();
	}: _(RawOrigin::Root, caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1, COORDINATE_IN_2])
	verify {
		assert_eq!(Estate::get_land_units(METAVERSE_ID, COORDINATE_IN_1), Some(OwnerId::Token(0)));
		assert_eq!(Estate::get_land_units(METAVERSE_ID, COORDINATE_IN_2), Some(OwnerId::Token(1)))
	}

	// transfer_land
	transfer_land {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		let target: T::AccountId = account("target", 0, SEED);
		let target_lookup = T::Lookup::unlookup(target.clone());

		let initial_balance = dollar(1000);

		// <T as pallet::Config>::Currency::make_free_balance_be(&caller, initial_balance.unique_saturated_into());
		Estate::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
		create_land_and_estate_groups();
        Estate::mint_land(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, COORDINATE_IN);

	}: _(RawOrigin::Signed(caller.clone()), target.clone(), METAVERSE_ID, COORDINATE_IN_1)
	verify {
		// TODO: issue with blow line
		// assert_eq!(Estate::get_land_units(METAVERSE_ID, COORDINATE_IN_1), target.clone())
	}

	// mint_estate as tokens
	mint_estate {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		Estate::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
		create_land_and_estate_groups();
	}: _(RawOrigin::Root, caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1])
	verify {
		assert_eq!(Estate::get_estates(0), Some(get_estate_info(vec![COORDINATE_IN_1])));
		assert_eq!(Estate::get_estate_owner(0), Some(OwnerId::Token(0)));
		assert_eq!(Estate::get_land_units(METAVERSE_ID, COORDINATE_IN_1), Some(OwnerId::Token(1)));
	}

	// dissolve_estate
	dissolve_estate {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		Estate::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
        create_land_and_estate_groups();
		Estate::mint_estate(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1]);
	}: _(RawOrigin::Signed(caller.clone()), 0)
	verify {
		assert_eq!(Estate::get_estates(0), None)
	}

	// add_land_unit_to_estate
	add_land_unit_to_estate {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		Estate::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
        create_land_and_estate_groups();
		Estate::mint_estate(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1]);
		Estate::mint_land(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, COORDINATE_IN_2);
	}: _(RawOrigin::Signed(caller.clone()), 0, vec![COORDINATE_IN_2])
	verify {
		assert_eq!(Estate::get_estates(0), Some(get_estate_info(vec![COORDINATE_IN_1, COORDINATE_IN_2])))
	}

	// remove_land_unit_from_estate
	remove_land_unit_from_estate {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		Estate::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
        create_land_and_estate_groups();
		Estate::mint_estate(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1, COORDINATE_IN_2]);
	}: _(RawOrigin::Signed(caller.clone()), 0, vec![COORDINATE_IN_2])
	verify {
		assert_eq!(Estate::get_estates(0), Some(get_estate_info(vec![COORDINATE_IN_1])))
	}

	// create_estate as token
	create_estate {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		Estate::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
        create_land_and_estate_groups();
		Estate::mint_lands(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1, COORDINATE_IN_2]);

	}: _(RawOrigin::Root, caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1, COORDINATE_IN_2])
	verify {
		assert_eq!(Estate::get_estates(0), Some(get_estate_info(vec![COORDINATE_IN_1, COORDINATE_IN_2])));
		assert_eq!(Estate::get_estate_owner(0), Some(OwnerId::Token(0)));
		//assert_eq!(Estate::get_land_units(METAVERSE_ID, COORDINATE_IN_1), Some(OwnerId::Account(caller.clone())));
		//assert_eq!(Estate::get_land_units(METAVERSE_ID, COORDINATE_IN_2), Some(OwnerId::Account(caller)))
	}

	// transfer_estate
	transfer_estate {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		let target: T::AccountId = account("target", 0, SEED);
		let target_lookup = T::Lookup::unlookup(target.clone());

		Estate::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
        create_land_and_estate_groups();
		Estate::mint_estate(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1, COORDINATE_IN_2]);

	}: _(RawOrigin::Signed(caller.clone()), target.clone(), 0)
	verify {
		assert_eq!(Estate::get_estate_owner(0), Some(OwnerId::Account(target.clone())))
	}

	// issue_undeployed_land_blocks
	issue_undeployed_land_blocks {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());
        create_land_and_estate_groups();
	}: _(RawOrigin::Root, caller.clone(), 20, 100, UndeployedLandBlockType::BoundToAddress)
	verify {
		let issued_undeployed_land_block = Estate::get_undeployed_land_block(0);
		match issued_undeployed_land_block {
			Some(a) => {
				// Verify details of UndeployedLandBlock
				assert_eq!(a.owner, caller.clone());
				assert_eq!(a.number_land_units, 100);
				assert_eq!(a.undeployed_land_block_type, UndeployedLandBlockType::BoundToAddress);
				assert_eq!(a.is_frozen, false);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	}

	// freeze_undeployed_land_blocks
	freeze_undeployed_land_blocks {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());
        create_land_and_estate_groups();
		issue_new_undeployed_land_block(5)?;
	}: _(RawOrigin::Root, 0)
	verify {
		let issued_undeployed_land_block = Estate::get_undeployed_land_block(0);
		match issued_undeployed_land_block {
			Some(a) => {
				// Verify details of UndeployedLandBlock
				assert_eq!(a.is_frozen, true);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	}

	// unfreeze_undeployed_land_blocks
	unfreeze_undeployed_land_blocks {
	let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());
        create_land_and_estate_groups();
		issue_new_undeployed_land_block(5)?;
		Estate::freeze_undeployed_land_blocks(RawOrigin::Root.into(), Default::default());
	}: _(RawOrigin::Root, 0)
	verify {
		let issued_undeployed_land_block = Estate::get_undeployed_land_block(0);
		match issued_undeployed_land_block {
			Some(a) => {
				// Verify details of UndeployedLandBlock
				assert_eq!(a.is_frozen, false);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	}

	// burn_undeployed_land_blocks
	burn_undeployed_land_blocks {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());
        create_land_and_estate_groups();
		issue_new_undeployed_land_block(5)?;
		Estate::freeze_undeployed_land_blocks(RawOrigin::Root.into(), Default::default());
	}: _(RawOrigin::Root, 0)
	verify {
		assert_eq!(Estate::get_undeployed_land_block(0), None)
	}

	// approve_undeployed_land_blocks
	approve_undeployed_land_blocks {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		let target: T::AccountId = account("target", 0, SEED);
		let target_lookup = T::Lookup::unlookup(target.clone());
        create_land_and_estate_groups();
		Estate::issue_undeployed_land_blocks(RawOrigin::Root.into(), caller.clone(), 5, 100, UndeployedLandBlockType::BoundToAddress);
	}: _(RawOrigin::Signed(caller.clone()), target.clone(), Default::default())
	verify {
		let issued_undeployed_land_block = Estate::get_undeployed_land_block(0);
		match issued_undeployed_land_block {
			Some(a) => {
				// Verify details of UndeployedLandBlock
				assert_eq!(a.approved, Some(target.clone()));
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	}

	// unapprove_undeployed_land_blocks
	unapprove_undeployed_land_blocks {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());
        create_land_and_estate_groups();
		Estate::issue_undeployed_land_blocks(RawOrigin::Root.into(), caller.clone(), 5, 100, UndeployedLandBlockType::BoundToAddress);
	}: _(RawOrigin::Signed(caller.clone()), Default::default())
	verify {
		let issued_undeployed_land_block = Estate::get_undeployed_land_block(0);
		match issued_undeployed_land_block {
			Some(a) => {
				// Verify details of UndeployedLandBlock
				assert_eq!(a.approved, None);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	}

	// transfer_undeployed_land_blocks
	transfer_undeployed_land_blocks {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		let target: T::AccountId = account("target", 0, SEED);
		let target_lookup = T::Lookup::unlookup(target.clone());
        create_land_and_estate_groups();
		Estate::issue_undeployed_land_blocks(RawOrigin::Root.into(), caller.clone(), 5, 100, UndeployedLandBlockType::Transferable);
	}: _(RawOrigin::Signed(caller.clone()), target.clone(), Default::default())
	verify {
		let issued_undeployed_land_block = Estate::get_undeployed_land_block(0);
		match issued_undeployed_land_block {
			Some(a) => {
				// Verify details of UndeployedLandBlock
				assert_eq!(a.owner, target.clone());
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	}

	// deploy_land_block
	deploy_land_block {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());
        create_land_and_estate_groups();
		Estate::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
		Estate::issue_undeployed_land_blocks(RawOrigin::Root.into(), caller.clone(), 5, 100, UndeployedLandBlockType::Transferable);
	}: _(RawOrigin::Signed(caller.clone()), Default::default(), METAVERSE_ID, vec![COORDINATE_IN_1, COORDINATE_IN_2])
	verify {
		let issued_undeployed_land_block = Estate::get_undeployed_land_block(0);
		match issued_undeployed_land_block {
			Some(a) => {
				// Verify details of UndeployedLandBlock
				assert_eq!(a.number_land_units, 98);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	}
	active_issue_undeploy_land_block{
		// INITIALIZE RUNTIME STATE
		let minting_info = 	MintingRateInfo {
			expect: Default::default(),
			// 10% minting rate per annual
			annual: 10,
			// Max 100 millions land unit
			max: 100_000_000,
		};
		// Pre issue 5 land blocks x 100 land units
		issue_new_undeployed_land_block(5)?;
		let min_block_per_round = 5u32;

		let new_round = RoundInfo::new(1u32, 0u32.into(), min_block_per_round.into());

		Round::<T>::put(new_round);
		let high_inflation_rate = MintingRateInfo {
			expect: Default::default(),
			annual: 20,
			// Max 100 millions land unit
			max: 100_000_000,
		};
		MintingRateConfig::<T>::put(high_inflation_rate);

//
//		// PREPARE RUN_TO_BLOCK LOOP
//		let before_running_round_index = EstateModule::<T>::round().current;
//		let round_length: T::BlockNumber = EstateModule::<T>::round().length.into();
//
//
//		let mut now = <frame_system::Pallet<T>>::block_number() + 1u32.into();
//		let mut counter = 0usize;
//		let end = EstateModule::<T>::round().first + (round_length * min_block_per_round.into());

	}: {
		Estate::on_initialize(6u32.into());
	}

	// bond_more
	bond_more {
		let min_stake = Estate::MinimumStake::get();

		let caller: T::AccountId = whitelisted_caller();
		set_balance(FungibleTokenId::NativeToken(0), caller, 10000);
        create_land_and_estate_groups();s
		Estate::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
		Estate::mint_estate(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1]);
	}: _(RawOrigin::Signed(caller.clone()), 0, Estate::MinimumStake::get())
	verify {
		assert_eq!(Estate::estate_stake(0, caller.clone()), Estate::MinimumStake::get())
	}

	// bond_less
	bond_less {
		let caller: T::AccountId = whitelisted_caller();
		set_balance(FungibleTokenId::NativeToken(0), caller, 10000);

		let min_stake = Estate::MinimumStake::get();
		let bond_amount = min_stake + 1u32.into();
        create_land_and_estate_groups();
		Estate::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
		Estate::mint_estate(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1]);
		Estate::bond_more(RawOrigin::Signed(caller.clone()).into(), 0, bond_amount);
	}: _(RawOrigin::Signed(caller.clone()), 0, 1u32.into())
	verify {
		assert_eq!(Estate::estate_stake(0, caller.clone()),  Estate::MinimumStake::get())
	}

	// leave_staking
	leave_staking {
		let caller: T::AccountId = whitelisted_caller();
		set_balance(FungibleTokenId::NativeToken(0), caller, 10000);

		let min_stake = Estate::MinimumStake::get();
		let bond_amount = min_stake + 1u32.into();
        create_land_and_estate_groups();
		Estate::set_max_bounds(RawOrigin::Root.into(), METAVERSE_ID, MAX_BOUND);
		Estate::mint_estate(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1]);
		Estate::bond_more(RawOrigin::Signed(caller.clone()).into(), 0, bond_amount);
	}: _(RawOrigin::Signed(caller.clone()), 0)
	verify {
		assert_eq!(Estate::exit_queue(caller.clone(), 0), Some(()))
	}

}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::benchmarking::utils::tests::new_test_ext;
	use orml_benchmarking::impl_benchmark_test_suite;

	impl_benchmark_test_suite!(new_test_ext(),);
}