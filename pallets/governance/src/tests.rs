#![cfg(test)]

use super::*;
use frame_support::sp_runtime::DispatchError::BadOrigin;
use frame_support::{assert_err, assert_noop, assert_ok};
use mock::{RuntimeEvent, *};

// Update country referendum parameters tests
#[test]
fn update_country_referendum_parameters_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(BOB);
		assert_ok!(GovernanceModule::update_referendum_parameters(
			origin.clone(),
			BOB_COUNTRY_ID,
			REFERENDUM_PARAMETERS
		));
		assert_eq!(
			last_event(),
			RuntimeEvent::Governance(crate::Event::ReferendumParametersUpdated(BOB_COUNTRY_ID))
		);
	});
}

#[test]
fn update_country_referendum_parameters_when_not_country_owner_does_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(ALICE);
		assert_noop!(
			GovernanceModule::update_referendum_parameters(origin.clone(), BOB_COUNTRY_ID, REFERENDUM_PARAMETERS),
			Error::<Runtime>::AccountIsNotMetaverseOwner
		);
	});
}

// Creating preimage tests
#[test]
fn create_new_preimage_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(ALICE);
		let encoded_proposal = set_freeze_metaverse_proposal(4);
		assert_ok!(GovernanceModule::note_preimage(
			origin.clone(),
			BOB_COUNTRY_ID,
			encoded_proposal
		));
		assert_eq!(Balances::free_balance(&ALICE), 99990);
		assert_eq!(Balances::reserved_balance(&ALICE), 10);
		let hash = set_freeze_metaverse_proposal_hash(4);
		assert_eq!(
			last_event(),
			RuntimeEvent::Governance(crate::Event::PreimageNoted(BOB_COUNTRY_ID, hash, ALICE, 10))
		);
	});
}

// Creating proposal tests
#[test]
fn create_new_proposal_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(ALICE);
		let hash = set_freeze_metaverse_proposal_hash(1);
		add_freeze_metaverse_preimage(hash);
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		assert_eq!(
			last_event(),
			RuntimeEvent::Governance(crate::Event::ReferendumStarted(
				BOB_COUNTRY_ID,
				0,
				0,
				VoteThreshold::RelativeMajority
			))
		);
	});
}

// Creating proposal tests
#[test]
fn create_local_metaverse_proposal_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(ALICE);
		let hash = set_freeze_metaverse_proposal_hash(0);
		add_metaverse_preimage(hash);
		println!("{:#x}", hash);
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		assert_eq!(
			last_event(),
			RuntimeEvent::Governance(crate::Event::ReferendumStarted(
				BOB_COUNTRY_ID,
				0,
				0,
				VoteThreshold::RelativeMajority
			))
		);
	});
}

#[test]
fn create_new_proposal_when_not_enough_funds_does_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(BOB);
		let hash = set_freeze_metaverse_proposal_hash(1);
		add_freeze_metaverse_preimage(hash);
		assert_noop!(
			GovernanceModule::propose(
				origin.clone(),
				BOB_COUNTRY_ID,
				600,
				hash.clone(),
				PROPOSAL_DESCRIPTION.to_vec()
			),
			Error::<Runtime>::InsufficientBalance
		);
	});
}

#[test]
fn create_new_proposal_with_out_of_scope_preimage_does_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(ALICE);
		let hash = set_balance_proposal_hash(1);
		add_preimage(hash);
		assert_err!(
			GovernanceModule::propose(
				origin.clone(),
				BOB_COUNTRY_ID,
				600,
				hash.clone(),
				PROPOSAL_DESCRIPTION.to_vec()
			),
			Error::<Runtime>::PreimageInvalid
		);
		//assert_eq!(
		//	last_event(),
		//	RuntimeEvent::Governance(crate::Event::ProposalRefused(BOB_COUNTRY_ID, hash))
		//);
	});
}

#[test]
fn create_new_proposal_when_too_small_deposit_does_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(BOB);
		let hash = set_freeze_metaverse_proposal_hash(1);
		add_freeze_metaverse_preimage(hash);
		assert_noop!(
			GovernanceModule::propose(
				origin.clone(),
				BOB_COUNTRY_ID,
				40,
				hash.clone(),
				PROPOSAL_DESCRIPTION.to_vec()
			),
			Error::<Runtime>::DepositTooLow
		);
	});
}

#[test]
fn create_new_proposal_when_not_country_member_does_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let hash = set_freeze_metaverse_proposal_hash(1);
		add_freeze_metaverse_preimage(hash);
		assert_noop!(
			GovernanceModule::propose(
				RuntimeOrigin::signed(5).clone(),
				ALICE_COUNTRY_ID,
				400,
				hash.clone(),
				PROPOSAL_DESCRIPTION.to_vec()
			),
			Error::<Runtime>::AccountIsNotMetaverseMember
		);
	});
}

#[test]
fn create_new_proposal_when_queue_full_does_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(BOB);
		let hash = set_freeze_metaverse_proposal_hash(1);
		add_freeze_metaverse_preimage(hash);
		let parameters = ReferendumParameters {
			voting_threshold: Some(VoteThreshold::RelativeMajority),
			min_proposal_launch_period: 12,
			voting_period: 5,
			enactment_period: 10,
			local_vote_locking_period: 30,
			max_proposals_per_metaverse: 0,
		};
		assert_ok!(GovernanceModule::update_referendum_parameters(
			origin.clone(),
			BOB_COUNTRY_ID,
			parameters
		));
		assert_noop!(
			GovernanceModule::propose(
				origin.clone(),
				BOB_COUNTRY_ID,
				400,
				hash.clone(),
				PROPOSAL_DESCRIPTION.to_vec()
			),
			Error::<Runtime>::ProposalQueueFull
		);
	});
}

// Cancel proposal tests
// #[test]
// fn cancel_proposal_work() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		let origin = RuntimeOrigin::signed(ALICE);
// 		let hash = set_freeze_metaverse_proposal_hash(1);
// 		let hash2 = set_freeze_metaverse_proposal_hash(2);
// 		add_freeze_metaverse_preimage(hash);
// 		add_freeze_metaverse_preimage(hash2);
// 		assert_ok!(GovernanceModule::propose(
// 			origin.clone(),
// 			BOB_COUNTRY_ID,
// 			600,
// 			hash.clone(),
// 			PROPOSAL_DESCRIPTION.to_vec()
// 		));
// 		assert_ok!(GovernanceModule::propose(
// 			origin.clone(),
// 			BOB_COUNTRY_ID,
// 			600,
// 			hash2.clone(),
// 			PROPOSAL_DESCRIPTION.to_vec()
// 		));
// 		assert_ok!(GovernanceModule::cancel_proposal(origin.clone(), 1, BOB_COUNTRY_ID));
// 		assert_eq!(Balances::free_balance(&ALICE), 100000);
// 		assert_eq!(
// 			last_event(),
// 			RuntimeEvent::Governance(crate::Event::ProposalCancelled(ALICE, 1))
// 		);
// 	});
// }

// #[test]
// fn cancel_non_existing_proposal_does_not_work() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		let origin = RuntimeOrigin::signed(ALICE);
// 		assert_noop!(
// 			GovernanceModule::cancel_proposal(origin.clone(), 0, BOB_COUNTRY_ID),
// 			Error::<Runtime>::ProposalDoesNotExist
// 		);
// 	});
// }

//#[test]
// fn cancel_proposal_that_you_have_not_submitted_does_not_work() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		let origin = RuntimeOrigin::signed(ALICE);
// 		let hash = set_freeze_metaverse_proposal_hash(1);
// 		let hash2 = set_freeze_metaverse_proposal_hash(2);
// 		add_freeze_metaverse_preimage(hash);
// 		add_freeze_metaverse_preimage(hash2);
// 		assert_ok!(GovernanceModule::propose(
// 			origin.clone(),
// 			BOB_COUNTRY_ID,
// 			600,
// 			hash.clone(),
// 			PROPOSAL_DESCRIPTION.to_vec()
// 		));
// 		assert_ok!(GovernanceModule::propose(
// 			origin.clone(),
// 			BOB_COUNTRY_ID,
// 			600,
// 			hash2.clone(),
// 			PROPOSAL_DESCRIPTION.to_vec()
// 		));
// 		assert_noop!(
// 			GovernanceModule::cancel_proposal(RuntimeOrigin::signed(BOB), 1, BOB_COUNTRY_ID),
// 			Error::<Runtime>::NotProposalCreator
// 		);
// 	});
// }

// Fast track proposal tests
#[test]
fn fast_track_proposal_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(ALICE);
		let hash = set_freeze_metaverse_proposal_hash(1);
		let hash2 = set_freeze_metaverse_proposal_hash(2);
		add_freeze_metaverse_preimage(hash);
		add_freeze_metaverse_preimage(hash2);
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			600,
			hash2.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		assert_ok!(GovernanceModule::fast_track_proposal(
			RuntimeOrigin::signed(ALICE),
			1,
			BOB_COUNTRY_ID
		));
		assert_eq!(
			last_event(),
			RuntimeEvent::Governance(crate::Event::ProposalFastTracked(BOB_COUNTRY_ID, 1))
		);
	});
}

#[test]
fn fast_track_proposal_when_not_country_owner_does_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(ALICE);
		let hash = set_freeze_metaverse_proposal_hash(1);
		let hash2 = set_freeze_metaverse_proposal_hash(2);
		add_freeze_metaverse_preimage(hash);
		add_freeze_metaverse_preimage(hash2);
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			600,
			hash2.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		assert_noop!(
			GovernanceModule::fast_track_proposal(RuntimeOrigin::signed(BOB), 1, BOB_COUNTRY_ID),
			BadOrigin
		);
	});
}

/// Voting tests
#[test]
fn vote_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(ALICE);
		let hash = set_freeze_metaverse_proposal_hash(1);
		add_freeze_metaverse_preimage(hash);
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		run_to_block(2);
		assert_ok!(GovernanceModule::try_vote(
			RuntimeOrigin::signed(BOB),
			BOB_COUNTRY_ID,
			0,
			VOTE_FOR
		));
		assert_eq!(Balances::usable_balance(&BOB), 490);
		assert_eq!(
			last_event(),
			RuntimeEvent::Governance(crate::Event::VoteRecorded(BOB, 0, true))
		);
	});
}

#[test]
fn vote_when_not_country_member_does_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(BOB);
		let hash = set_freeze_metaverse_proposal_hash(1);
		add_freeze_metaverse_preimage_alice(hash);
		assert_ok!(GovernanceModule::propose(
			RuntimeOrigin::signed(ALICE),
			ALICE_COUNTRY_ID,
			100,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		run_to_block(16);
		assert_noop!(
			GovernanceModule::try_vote(RuntimeOrigin::signed(BOB), ALICE_COUNTRY_ID, 0, VOTE_FOR),
			Error::<Runtime>::AccountIsNotMetaverseMember
		);
	});
}

#[test]
fn vote_more_than_once_does_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(ALICE);
		let hash = set_freeze_metaverse_proposal_hash(1);
		add_freeze_metaverse_preimage(hash);
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		run_to_block(16);
		assert_ok!(GovernanceModule::try_vote(
			RuntimeOrigin::signed(BOB),
			BOB_COUNTRY_ID,
			0,
			VOTE_FOR
		));
		assert_noop!(
			GovernanceModule::try_vote(RuntimeOrigin::signed(BOB), BOB_COUNTRY_ID, 0, VOTE_FOR),
			Error::<Runtime>::AccountAlreadyVoted
		);
	});
}

// Remove vote tests
#[test]
fn remove_vote_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(ALICE);
		let hash = set_freeze_metaverse_proposal_hash(1);
		add_freeze_metaverse_preimage(hash);
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		run_to_block(16);
		assert_ok!(GovernanceModule::try_vote(
			RuntimeOrigin::signed(BOB),
			BOB_COUNTRY_ID,
			0,
			VOTE_FOR
		));
		assert_ok!(GovernanceModule::try_remove_vote(
			RuntimeOrigin::signed(BOB),
			0,
			BOB_COUNTRY_ID
		));
		assert_eq!(
			last_event(),
			RuntimeEvent::Governance(crate::Event::VoteRemoved(BOB, 0))
		);
	});
}

#[test]
fn remove_vote_when_you_have_not_voted_does_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(ALICE);
		let hash = set_freeze_metaverse_proposal_hash(1);
		add_freeze_metaverse_preimage(hash);
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		run_to_block(16);
		assert_noop!(
			GovernanceModule::try_remove_vote(RuntimeOrigin::signed(BOB), 0, BOB_COUNTRY_ID),
			Error::<Runtime>::AccountHasNotVoted
		);
	});
}
// Emergency canceling of referendum tests
#[test]
fn emergency_cancel_referendum_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(ALICE);
		let hash = set_freeze_metaverse_proposal_hash(1);
		add_freeze_metaverse_preimage(hash);
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		run_to_block(18);
		assert_ok!(GovernanceModule::emergency_cancel_referendum(
			origin.clone(),
			BOB_COUNTRY_ID,
			0
		));
		assert_eq!(Balances::free_balance(&ALICE), 100000);
		assert_eq!(
			last_event(),
			RuntimeEvent::Governance(crate::Event::ReferendumCancelled(0))
		);
	});
}

#[test]
fn emergency_cancel_non_existing_referendum_does_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(ALICE);
		assert_noop!(
			GovernanceModule::emergency_cancel_referendum(origin.clone(), 0, 3),
			Error::<Runtime>::ReferendumDoesNotExist
		);
	});
}

#[test]
fn emergency_cancel_referendum_when_not_having_privileges_does_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(ALICE);
		let hash = set_freeze_metaverse_proposal_hash(1);
		add_freeze_metaverse_preimage(hash);

		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		run_to_block(17);
		assert_noop!(
			GovernanceModule::emergency_cancel_referendum(RuntimeOrigin::signed(BOB), 0, 0),
			BadOrigin
		);
	});
}

// Referendum Finalization Tests
#[test]
fn referendum_proposal_passes() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(ALICE);
		let hash = set_freeze_metaverse_proposal_hash(1);
		add_freeze_metaverse_preimage(hash);
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		run_to_block(16);
		assert_ok!(GovernanceModule::try_vote(
			RuntimeOrigin::signed(BOB),
			BOB_COUNTRY_ID,
			0,
			VOTE_FOR
		));
		run_to_block(117);
		assert_eq!(Balances::free_balance(&ALICE), 100000);
		assert_eq!(
			GovernanceModule::referendum_info(BOB_COUNTRY_ID, 0),
			Some(ReferendumInfo::Finished {
				passed: true,
				end: 116,
				title: PROPOSAL_DESCRIPTION.to_vec()
			})
		);
		//assert_eq!(
		//	last_event(),
		//	RuntimeEvent::Governance(crate::Event::ReferendumPassed(0))
		//);
	});
}

#[test]
fn referendum_proposal_is_rejected() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(ALICE);
		let hash = set_freeze_metaverse_proposal_hash(1);
		add_freeze_metaverse_preimage(hash);
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		run_to_block(16);
		assert_ok!(GovernanceModule::try_vote(
			RuntimeOrigin::signed(BOB),
			BOB_COUNTRY_ID,
			0,
			VOTE_AGAINST
		));
		run_to_block(117);
		assert_eq!(Balances::free_balance(&ALICE), 100000);
		assert_eq!(
			GovernanceModule::referendum_info(BOB_COUNTRY_ID, 0),
			Some(ReferendumInfo::Finished {
				passed: false,
				end: 116,
				title: PROPOSAL_DESCRIPTION.to_vec()
			})
		);
		assert_eq!(
			last_event(),
			RuntimeEvent::Governance(crate::Event::ReferendumNotPassed(0))
		);
	});
}

#[test]
fn referendum_proposal_is_enacted() {
	ExtBuilder::default().build().execute_with(|| {
		let root = RuntimeOrigin::root();
		let proposer = RuntimeOrigin::signed(ALICE);
		let hash = set_freeze_metaverse_proposal_hash(1);
		add_freeze_metaverse_preimage(hash);
		assert_ok!(GovernanceModule::propose(
			proposer.clone(),
			BOB_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		assert_ok!(GovernanceModule::enact_proposal(
			root.clone(),
			0,
			BOB_COUNTRY_ID,
			0,
			hash.clone()
		));
		assert_eq!(
			last_event(),
			RuntimeEvent::Governance(crate::Event::ProposalEnacted(BOB_COUNTRY_ID, 0))
		);
	});
}

#[test]
fn referendum_proposal_rejected_as_out_of_scope() {
	ExtBuilder::default().build().execute_with(|| {
		let root = RuntimeOrigin::root();
		let preimage_hash = set_balance_proposal_hash(1);
		add_preimage(preimage_hash);
		add_out_of_scope_proposal(preimage_hash);

		assert_ok!(GovernanceModule::enact_proposal(
			root.clone(),
			0,
			BOB_COUNTRY_ID,
			0,
			preimage_hash.clone()
		));
		assert_eq!(
			last_event(),
			RuntimeEvent::Governance(crate::Event::PreimageInvalid(BOB_COUNTRY_ID, preimage_hash.clone(), 0))
		);
	});
}

#[test]
fn unlocking_balance_after_removing_vote_works() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(ALICE);
		let hash = set_freeze_metaverse_proposal_hash(1);
		add_freeze_metaverse_preimage(hash);
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		run_to_block(16);
		assert_ok!(GovernanceModule::try_vote(
			RuntimeOrigin::signed(BOB),
			BOB_COUNTRY_ID,
			0,
			VOTE_FOR
		));
		assert_eq!(Balances::usable_balance(&BOB), 490);
		run_to_block(26);
		assert_ok!(GovernanceModule::try_remove_vote(
			RuntimeOrigin::signed(BOB),
			0,
			BOB_COUNTRY_ID
		));
		assert_ok!(GovernanceModule::unlock_balance(RuntimeOrigin::signed(BOB), BOB));
		assert_eq!(Balances::usable_balance(&BOB), 500);
	});
}

#[test]
fn unlocking_balance_after_referendum_is_over_works() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(ALICE);
		let hash = set_freeze_metaverse_proposal_hash(1);
		add_freeze_metaverse_preimage(hash);
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		run_to_block(16);
		assert_ok!(GovernanceModule::try_vote(
			RuntimeOrigin::signed(BOB),
			BOB_COUNTRY_ID,
			0,
			VOTE_FOR
		));
		assert_eq!(Balances::usable_balance(&BOB), 490);
		run_to_block(30);
		assert_ok!(GovernanceModule::try_remove_vote(
			RuntimeOrigin::signed(BOB),
			0,
			BOB_COUNTRY_ID
		));
		assert_ok!(GovernanceModule::unlock_balance(RuntimeOrigin::signed(BOB), BOB));
		assert_eq!(Balances::usable_balance(&BOB), 500);
	});
}

#[test]
fn second_proposal_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(ALICE);
		let seconder = RuntimeOrigin::signed(BOB);
		let hash = set_freeze_metaverse_proposal_hash(1);
		let hash2 = set_freeze_metaverse_proposal_hash(2);
		add_freeze_metaverse_preimage(hash);
		add_freeze_metaverse_preimage(hash2);
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			500,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			500,
			hash2.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		assert_ok!(GovernanceModule::second(seconder.clone(), 1, 1));
		assert_eq!(last_event(), RuntimeEvent::Governance(crate::Event::Seconded(BOB, 1)));
	});
}

#[test]
fn second_proposal_does_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(ALICE);
		let seconder = RuntimeOrigin::signed(BOB);
		assert_noop!(
			GovernanceModule::second(seconder.clone(), 1, 1),
			Error::<Runtime>::ProposalMissing
		);
		let hash = set_freeze_metaverse_proposal_hash(1);
		let hash2 = set_freeze_metaverse_proposal_hash(2);
		add_freeze_metaverse_preimage(hash);
		add_freeze_metaverse_preimage(hash2);
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			500,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			500,
			hash2.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		assert_noop!(
			GovernanceModule::second(seconder.clone(), 1, 0),
			Error::<Runtime>::WrongUpperBound
		);
	});
}

#[test]
fn get_next_proposal_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = RuntimeOrigin::signed(ALICE);
		let seconder = RuntimeOrigin::signed(BOB);
		let hash = set_freeze_metaverse_proposal_hash(1);
		let hash2 = set_freeze_metaverse_proposal_hash(2);
		let hash3 = set_freeze_metaverse_proposal_hash(3);
		add_freeze_metaverse_preimage(hash);
		add_freeze_metaverse_preimage(hash2);
		add_freeze_metaverse_preimage(hash3);
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			500,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			300,
			hash2.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		assert_eq!(GovernanceModule::deposit_of(1), Some((vec![1], 300)));
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			400,
			hash3.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		assert_eq!(GovernanceModule::deposit_of(2), Some((vec![1], 400)));
		assert_ok!(GovernanceModule::second(seconder.clone(), 1, 1));
		assert_eq!(GovernanceModule::proposals(BOB_COUNTRY_ID, 0), None);
		run_to_block(117);
		assert_eq!(GovernanceModule::proposals(BOB_COUNTRY_ID, 1), None);
		run_to_block(232);
		assert_eq!(GovernanceModule::proposals(BOB_COUNTRY_ID, 2), None);
	})
}
