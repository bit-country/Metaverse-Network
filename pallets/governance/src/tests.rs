#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};


// Update country referendum parameters tests
#[test]
fn update_country_referendum_parameters_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(BOB);
        assert_ok!(GovernanceModule::update_referendum_parameters(origin.clone()
            , BOB_COUNTRY_ID, REFERENDUM_PARAMETERS));
        assert_eq!(last_event(), Event::governance(crate::Event::ReferendumParametersUpdated(BOB_COUNTRY_ID)));
    });
}

#[test]
fn update_country_referendum_parameters_when_not_country_owner_does_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);
        assert_noop!(GovernanceModule::update_referendum_parameters(origin.clone(), BOB_COUNTRY_ID
            , REFERENDUM_PARAMETERS), Error::<Runtime>::AccountNotCountryOwner);
    });
}


// Creating proposal tests
#[test]
fn create_new_proposal_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);
        assert_ok!(GovernanceModule::propose(origin.clone(), BOB_COUNTRY_ID, 600,PROPOSAL_PARAMETERS.to_vec(), PROPOSAL_DESCRIPTION.to_vec()));
        assert_eq!(last_event(), Event::governance(crate::Event::ProposalSubmitted(ALICE, BOB_COUNTRY_ID, 0)));
    });
}

#[test]
fn create_new_proposal_when_not_enough_funds_does_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(BOB);
        assert_noop!(GovernanceModule::propose(origin.clone(), BOB_COUNTRY_ID, 600
            ,PROPOSAL_PARAMETERS.to_vec(), PROPOSAL_DESCRIPTION.to_vec()), Error::<Runtime>::InsufficientBalance);
    });
}

#[test]
fn create_new_proposal_when_not_country_member_does_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(GovernanceModule::propose(Origin::signed(5).clone(), ALICE_COUNTRY_ID, 400
            ,PROPOSAL_PARAMETERS.to_vec(), PROPOSAL_DESCRIPTION.to_vec()), Error::<Runtime>::AccountNotCountryMember);
    });
}

#[test]
fn create_new_proposal_when_queue_full_does_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(BOB);
        let parameters = ReferendumParameters {
            voting_threshold:  Some(VoteThreshold::RelativeMajority),
            min_proposal_launch_period: 12,
            voting_period:5, 
            enactment_period: 10, 
            max_params_per_proposal: 2,
            max_proposals_per_country: 0,
        };  
        assert_ok!(GovernanceModule::update_referendum_parameters(origin.clone(), BOB_COUNTRY_ID, parameters));
        assert_noop!(GovernanceModule::propose(origin.clone(), BOB_COUNTRY_ID, 400
            ,PROPOSAL_PARAMETERS.to_vec(), PROPOSAL_DESCRIPTION.to_vec()), Error::<Runtime>::ProposalQueueFull);
        
    });
}

#[test]
fn create_new_proposal_with_too_many_parameters_does_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(BOB);
        let parameters = ReferendumParameters {
            voting_threshold:  Some(VoteThreshold::RelativeMajority),
            min_proposal_launch_period: 12,
            voting_period:5, 
            enactment_period: 10, 
            max_params_per_proposal: 1,
            max_proposals_per_country: 1,
        };  
        assert_ok!(GovernanceModule::update_referendum_parameters(origin.clone(), BOB_COUNTRY_ID, parameters));
        assert_noop!(GovernanceModule::propose(origin.clone(), BOB_COUNTRY_ID, 400
            ,PROPOSAL_PARAMETERS.to_vec(), PROPOSAL_DESCRIPTION.to_vec()), Error::<Runtime>::TooManyProposalParameters);
        
    });
}

// Cancel proposal tests
#[test]
fn cancel_proposal_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);
        assert_ok!(GovernanceModule::propose(origin.clone(), BOB_COUNTRY_ID, 600,PROPOSAL_PARAMETERS.to_vec(), PROPOSAL_DESCRIPTION.to_vec()));
        assert_ok!(GovernanceModule::cancel_proposal(origin.clone(), 0, BOB_COUNTRY_ID));
        assert_eq!(last_event(), Event::governance(crate::Event::ProposalCancelled(ALICE, BOB_COUNTRY_ID, 0)));
    });
}

#[test]
fn cancel_non_existing_proposal_does_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);
        assert_noop!(GovernanceModule::cancel_proposal(origin.clone(), 0, BOB_COUNTRY_ID), Error::<Runtime>::ProposalDoesNotExist);
    });
}

#[test]
fn cancel_proposal_that_you_have_not_submitted_does_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);
        assert_ok!(GovernanceModule::propose(origin.clone(), BOB_COUNTRY_ID, 600,PROPOSAL_PARAMETERS.to_vec(), PROPOSAL_DESCRIPTION.to_vec()));
        assert_noop!(GovernanceModule::cancel_proposal(Origin::signed(BOB), 0, BOB_COUNTRY_ID), Error::<Runtime>::NotProposalCreator);
    });
}

#[test]
fn cancel_proposal_that_is_a_referendum_does_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);
        assert_ok!(GovernanceModule::propose(origin.clone(), BOB_COUNTRY_ID, 600,PROPOSAL_PARAMETERS.to_vec(), PROPOSAL_DESCRIPTION.to_vec()));
        run_to_block(16);
        assert_noop!(GovernanceModule::cancel_proposal(origin.clone(), 0, BOB_COUNTRY_ID), Error::<Runtime>::ProposalIsReferendum);
    });
}


// Fast track proposal tests
#[test]
fn fast_track_proposal_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);
        assert_ok!(GovernanceModule::propose(origin.clone(), BOB_COUNTRY_ID, 600,PROPOSAL_PARAMETERS.to_vec(), PROPOSAL_DESCRIPTION.to_vec()));
        assert_ok!(GovernanceModule::fast_track_proposal(Origin::signed(BOB), 0, BOB_COUNTRY_ID));
        assert_eq!(last_event(), Event::governance(crate::Event::ProposalFastTracked(BOB, BOB_COUNTRY_ID, 0)));
    });
}

#[test]
fn fast_track_proposal_when_not_country_owner_does_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);
        assert_ok!(GovernanceModule::propose(origin.clone(), BOB_COUNTRY_ID, 600,PROPOSAL_PARAMETERS.to_vec(), PROPOSAL_DESCRIPTION.to_vec()));
        assert_noop!(GovernanceModule::fast_track_proposal(origin.clone(), 0, BOB_COUNTRY_ID), Error::<Runtime>::AccountNotCountryOwner);
    });
}

#[test]
fn fast_track_proposal_that_is_a_referendum_does_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);
        assert_ok!(GovernanceModule::propose(origin.clone(), BOB_COUNTRY_ID, 600,PROPOSAL_PARAMETERS.to_vec(), PROPOSAL_DESCRIPTION.to_vec()));
        run_to_block(16);
        assert_noop!(GovernanceModule::fast_track_proposal(Origin::signed(BOB), 0, BOB_COUNTRY_ID), Error::<Runtime>::ProposalIsReferendum);
    });
}


// Voting tests
#[test]
#[ignore]
fn vote_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);
        assert_ok!(GovernanceModule::propose(origin.clone(), BOB_COUNTRY_ID, 600,PROPOSAL_PARAMETERS.to_vec(), PROPOSAL_DESCRIPTION.to_vec()));
        run_to_block(16);
    });
}

#[test]
#[ignore]
fn vote_when_not_country_member_does_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);
        assert_ok!(GovernanceModule::propose(origin.clone(), BOB_COUNTRY_ID, 600,PROPOSAL_PARAMETERS.to_vec(), PROPOSAL_DESCRIPTION.to_vec()));
        run_to_block(16);
    });
}

#[test]
#[ignore]
fn vote_more_than_once_does_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);
        assert_ok!(GovernanceModule::propose(origin.clone(), BOB_COUNTRY_ID, 600,PROPOSAL_PARAMETERS.to_vec(), PROPOSAL_DESCRIPTION.to_vec()));
        run_to_block(16);
    });
}

// Remove vote tests
#[test]
#[ignore]
fn remove_vote_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);
        assert_ok!(GovernanceModule::propose(origin.clone(), BOB_COUNTRY_ID, 600,PROPOSAL_PARAMETERS.to_vec(), PROPOSAL_DESCRIPTION.to_vec()));
        run_to_block(16);
    });
}

#[test]
#[ignore]
fn remove_vote_when_you_have_not_voted_does_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);
        assert_ok!(GovernanceModule::propose(origin.clone(), BOB_COUNTRY_ID, 600,PROPOSAL_PARAMETERS.to_vec(), PROPOSAL_DESCRIPTION.to_vec()));
        run_to_block(16);
    });
}


// Emergency canceling of referendum tests
#[test]
#[ignore]
fn emergency_cancel_referendum_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);
        assert_ok!(GovernanceModule::propose(origin.clone(), BOB_COUNTRY_ID, 600,PROPOSAL_PARAMETERS.to_vec(), PROPOSAL_DESCRIPTION.to_vec()));
        run_to_block(16);
    });
}

#[test]
#[ignore]
fn emergency_cancel_non_existing_referendum_does_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);
        assert_ok!(GovernanceModule::propose(origin.clone(), BOB_COUNTRY_ID, 600,PROPOSAL_PARAMETERS.to_vec(), PROPOSAL_DESCRIPTION.to_vec()));
        
    });
}

#[test]
#[ignore]
fn emergency_cancel_referendum_when_not_having_privileges_does_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);
        assert_ok!(GovernanceModule::propose(origin.clone(), BOB_COUNTRY_ID, 600,PROPOSAL_PARAMETERS.to_vec(), PROPOSAL_DESCRIPTION.to_vec()));
        run_to_block(16);
    });
}

#[test]
#[ignore]
fn emergency_cancel_referendum_which_removes_privileges_does_not_work() {
    ExtBuilder::default().build().execute_with(|| {
        let origin = Origin::signed(ALICE);
        assert_ok!(GovernanceModule::propose(origin.clone(), BOB_COUNTRY_ID, 600,PROPOSAL_PARAMETERS.to_vec(), PROPOSAL_DESCRIPTION.to_vec()));
        run_to_block(16);
    });
}


