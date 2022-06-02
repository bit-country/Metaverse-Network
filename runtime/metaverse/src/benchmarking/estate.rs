#![cfg(feature = "runtime-benchmarks")]
use super::utils::{
	create_land_and_estate_group, create_metaverse_for_account, dollar, get_estate_info,
	issue_new_undeployed_land_block, mint_NFT, set_balance,
};
use crate::{Call, Currencies, Estate, Event, Metaverse, MinimumStake, Runtime, System};
use estate::{MintingRateConfig, MintingRateInfo, Round};
use frame_benchmarking::{account, whitelisted_caller};
use frame_support::traits::{Currency, Get, OnInitialize};
use frame_system::RawOrigin;
use orml_benchmarking::runtime_benchmarks;
use primitives::estate::{EstateInfo, OwnerId};
use primitives::staking::RoundInfo;
use primitives::UndeployedLandBlockType;
use primitives::{AccountId, Balance, FungibleTokenId, TokenId};
use sp_runtime::traits::{AccountIdConversion, Lookup, StaticLookup, UniqueSaturatedInto};

//pub type AccountId = u128;
pub type LandId = u64;
pub type EstateId = u64;

const SEED: u32 = 0;
const METAVERSE_ID: u64 = 0;

const MAX_BOUND: (i32, i32) = (-100, 100);
const COORDINATE_IN_1: (i32, i32) = (-10, 10);
const COORDINATE_IN_2: (i32, i32) = (-5, 5);
const COORDINATE_OUT: (i32, i32) = (0, 101);
const COORDINATE_IN_AUCTION: (i32, i32) = (99, 99);
const ESTATE_IN_AUCTION: EstateId = 99;
const ESTATE_ID: EstateId = 0;

runtime_benchmarks! {
	{ Runtime, estate }
	// mint_land as tokens
	mint_land {
		let caller: AccountId = whitelisted_caller();
		let caller_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(caller.clone());

		create_land_and_estate_group();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1u8]);
	}: _(RawOrigin::Root, caller.clone(), METAVERSE_ID, COORDINATE_IN_1)
	verify {
		assert_eq!(Estate::get_land_units(METAVERSE_ID, COORDINATE_IN_1), Some(OwnerId::Token(10, 0)));
	}

	// mint_lands as tokens
	mint_lands {
		let caller: AccountId = whitelisted_caller();
		let caller_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(caller.clone());

		create_land_and_estate_group();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1u8]);
	}: _(RawOrigin::Root, caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1, COORDINATE_IN_2])
	verify {
		assert_eq!(Estate::get_land_units(METAVERSE_ID, COORDINATE_IN_1), Some(OwnerId::Token(0, 0)));
		assert_eq!(Estate::get_land_units(METAVERSE_ID, COORDINATE_IN_2), Some(OwnerId::Token(0, 1)))
	}

	// transfer_land
	transfer_land {
		let caller: AccountId = whitelisted_caller();
		let caller_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(caller.clone());

		let target: AccountId = account("target", 0, SEED);
		let target_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(target.clone());

		let initial_balance = dollar(1000);

		// <T as pallet::Config>::Currency::make_free_balance_be(&caller, initial_balance.unique_saturated_into());

		create_land_and_estate_group();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1u8]);
		Estate::mint_land(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, COORDINATE_IN_1);
	}: _(RawOrigin::Signed(caller.clone()), target.clone(), METAVERSE_ID, COORDINATE_IN_1)
	verify {
		// TODO: issue with blow line
		// assert_eq!(Estate::get_land_units(METAVERSE_ID, COORDINATE_IN_1), target.clone())
	}

	// mint_estate as tokens
	mint_estate {
		let caller: AccountId = whitelisted_caller();
		let caller_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(caller.clone());

		create_land_and_estate_group();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1u8]);
	}: _(RawOrigin::Root, caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1])
	verify {
		assert_eq!(Estate::get_estates(0), Some(get_estate_info(vec![COORDINATE_IN_1])));
		assert_eq!(Estate::get_estate_owner(0), Some(OwnerId::Token(1,0)));
		assert_eq!(Estate::get_land_units(METAVERSE_ID, COORDINATE_IN_1), Some(OwnerId::Token(0,1)));
	}

	// dissolve_estate
	dissolve_estate {
		let caller: AccountId = whitelisted_caller();
		let caller_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(caller.clone());

				create_land_and_estate_group();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1u8]);
		Estate::mint_estate(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1]);
	}: _(RawOrigin::Signed(caller.clone()), 0)
	verify {
		assert_eq!(Estate::get_estates(0), None)
	}

	// add_land_unit_to_estate
	add_land_unit_to_estate {
		let caller: AccountId = whitelisted_caller();
		let caller_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(caller.clone());

		create_land_and_estate_group();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1u8]);
		Estate::mint_estate(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1]);
		Estate::mint_land(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, COORDINATE_IN_2);
	}: _(RawOrigin::Signed(caller.clone()), 0, vec![COORDINATE_IN_2])
	verify {
		assert_eq!(Estate::get_estates(0), Some(get_estate_info(vec![COORDINATE_IN_1, COORDINATE_IN_2])))
	}

	// remove_land_unit_from_estate
	remove_land_unit_from_estate {
		let caller: AccountId = whitelisted_caller();
		let caller_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(caller.clone());

		create_land_and_estate_group();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1u8]);
		Estate::mint_estate(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1, COORDINATE_IN_2]);
	}: _(RawOrigin::Signed(caller.clone()), 0, vec![COORDINATE_IN_2])
	verify {
		assert_eq!(Estate::get_estates(0), Some(get_estate_info(vec![COORDINATE_IN_1])))
	}

	// create_estate as token
	create_estate {
		let caller: AccountId = whitelisted_caller();
		let caller_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(caller.clone());

		create_land_and_estate_group();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1u8]);
		Estate::mint_lands(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1, COORDINATE_IN_2]);
	}: _(RawOrigin::Signed(caller.clone()), METAVERSE_ID, vec![COORDINATE_IN_1, COORDINATE_IN_2])
	verify {
		assert_eq!(Estate::get_estates(0), Some(get_estate_info(vec![COORDINATE_IN_1, COORDINATE_IN_2])));
		assert_eq!(Estate::get_estate_owner(0), Some(OwnerId::Token(1,0)));
		//assert_eq!(Estate::get_land_units(METAVERSE_ID, COORDINATE_IN_1), Some(OwnerId::Account(caller.clone())));
		//assert_eq!(Estate::get_land_units(METAVERSE_ID, COORDINATE_IN_2), Some(OwnerId::Account(caller)))
	}

	// transfer_estate
	transfer_estate {
		let caller: AccountId = whitelisted_caller();
		let caller_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(caller.clone());

		let target: AccountId = account("target", 0, SEED);
		let target_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(target.clone());

		create_land_and_estate_group();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1u8]);
		Estate::mint_estate(RawOrigin::Root.into(), caller.clone(), METAVERSE_ID, vec![COORDINATE_IN_1, COORDINATE_IN_2]);
	}: _(RawOrigin::Signed(caller.clone()), target.clone(), 0)
	verify {
		assert_eq!(Estate::get_estate_owner(0), Some(OwnerId::Token(1,0)))
	}

	// issue_undeployed_land_blocks
	issue_undeployed_land_blocks {
		let caller: AccountId = whitelisted_caller();
		let caller_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(caller.clone());
		create_land_and_estate_group();
	}: _(RawOrigin::Root, caller.clone(), 20, 100, UndeployedLandBlockType::BoundToAddress)
	verify {
		let issued_undeployed_land_block = Estate::get_undeployed_land_block(0);
		match issued_undeployed_land_block {
			Some(a) => {
				// Verify details of UndeployedLandBlock
				assert_eq!(a.owner, caller.clone());
				assert_eq!(a.number_land_units, 100);
				assert_eq!(a.undeployed_land_block_type, UndeployedLandBlockType::BoundToAddress);
				assert_eq!(a.is_locked, false);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	}

	// freeze_undeployed_land_blocks
	freeze_undeployed_land_blocks {
		let caller: AccountId = whitelisted_caller();
		let caller_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(caller.clone());
		create_land_and_estate_group();
		issue_new_undeployed_land_block(5)?;
	}: _(RawOrigin::Root, 0)
	verify {
		let issued_undeployed_land_block = Estate::get_undeployed_land_block(0);
		match issued_undeployed_land_block {
			Some(a) => {
				// Verify details of UndeployedLandBlock
				assert_eq!(a.is_locked, true);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	}

	// unfreeze_undeployed_land_blocks
	unfreeze_undeployed_land_blocks {
	let caller: AccountId = whitelisted_caller();
		let caller_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(caller.clone());
		create_land_and_estate_group();
		issue_new_undeployed_land_block(5)?;
		Estate::freeze_undeployed_land_blocks(RawOrigin::Root.into(), Default::default());
	}: _(RawOrigin::Root, 0)
	verify {
		let issued_undeployed_land_block = Estate::get_undeployed_land_block(0);
		match issued_undeployed_land_block {
			Some(a) => {
				// Verify details of UndeployedLandBlock
				assert_eq!(a.is_locked, false);
			}
			_ => {
				// Should fail test
				assert_eq!(0, 1);
			}
		}
	}

	// burn_undeployed_land_blocks
	burn_undeployed_land_blocks {
		let caller: AccountId = whitelisted_caller();
		let caller_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(caller.clone());
		create_land_and_estate_group();
		issue_new_undeployed_land_block(5)?;
		Estate::freeze_undeployed_land_blocks(RawOrigin::Root.into(), Default::default());
	}: _(RawOrigin::Root, 0)
	verify {
		assert_eq!(Estate::get_undeployed_land_block(0), None)
	}

	// approve_undeployed_land_blocks
	approve_undeployed_land_blocks {
		let caller: AccountId = whitelisted_caller();
		let caller_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(caller.clone());

		let target: AccountId = account("target", 0, SEED);
		let target_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(target.clone());
		create_land_and_estate_group();
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
		let caller: AccountId = whitelisted_caller();
		let caller_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(caller.clone());
		create_land_and_estate_group();
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
		let caller: AccountId = whitelisted_caller();
		let caller_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(caller.clone());

		let target: AccountId = account("target", 0, SEED);
		let target_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(target.clone());
		create_land_and_estate_group();
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
		let caller: AccountId = whitelisted_caller();
		let caller_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(caller.clone());
		create_land_and_estate_group();
		Metaverse::create_metaverse(RawOrigin::Signed(caller.clone()).into(), vec![1u8]);
		Estate::issue_undeployed_land_blocks(RawOrigin::Root.into(), caller.clone(), 5, 100, UndeployedLandBlockType::Transferable);
	}: _(RawOrigin::Signed(caller.clone()), Default::default(), METAVERSE_ID, (0,0), vec![COORDINATE_IN_1, COORDINATE_IN_2])
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

	on_initialize {
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

		Round::<Runtime>::put(new_round);
		let high_inflation_rate = MintingRateInfo {
			expect: Default::default(),
			annual: 20,
			// Max 100 millions land unit
			max: 100_000_000,
		};
		MintingRateConfig::<Runtime>::put(high_inflation_rate);

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
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::benchmarking::utils::tests::new_test_ext;
	use orml_benchmarking::impl_benchmark_test_suite;

	impl_benchmark_test_suite!(new_test_ext(),);
}
