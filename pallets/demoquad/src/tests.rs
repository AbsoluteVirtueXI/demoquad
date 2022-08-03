#![allow(unused_must_use)]
//use super::*;
use crate::mock;
use crate::types::Choice;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, pallet_prelude::*};
use sp_core::Hasher;
use sp_runtime::traits::BlakeTwo256;
fn hash(b: &[u8]) -> <Test as frame_system::Config>::Hash {
	BlakeTwo256::hash(b)
}

use crate::Event as DemoQuadEvent;

// found it here: https://stackoverflow.com/questions/60666012/getting-payload-from-a-substrate-event-back-in-rust-tests
/*
fn last_event() -> DemoQuadEvent<Test> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| {
			if let mock::Event::DemoQuad(DemoQuadEvent::ProposalSubmited(inner, b)) = e {
				Some(inner)
			} else {
				None
			}
		})
		.last()
		.unwrap()
}
*/

#[test]
fn only_identified_account_can_make_a_proposal() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			DemoQuad::submit_proposal(Origin::signed(1), "vote for me?".encode()),
			Error::<Test>::NoIdentityFound
		);
		mock::Identity::set_hash(Origin::signed(1), hash(b"alice"));
		assert_ok!(DemoQuad::submit_proposal(Origin::signed(1), "Vote for me".encode()),);
	});
}

#[test]
fn error_catching_proposal_length_should_work() {
	new_test_ext().execute_with(|| {
		mock::Identity::set_hash(Origin::signed(1), hash(b"alice"));
		assert_noop!(
			DemoQuad::submit_proposal(Origin::signed(1), "?".encode()),
			Error::<Test>::ProposalTooShort
		);
		assert_noop!(
			DemoQuad::submit_proposal(
				Origin::signed(1),
				"Polkadot Blockchain Academy was a great adventure, isn't it?".encode()
			),
			Error::<Test>::ProposalTooLong
		);
	});
}

#[test]
fn error_catching_max_proposals_per_block_should_work() {
	new_test_ext().execute_with(|| {
		mock::Identity::set_hash(Origin::signed(1), hash(b"alice"));
		DemoQuad::submit_proposal(Origin::signed(1), "vote for me 1?".encode());
		DemoQuad::submit_proposal(Origin::signed(1), "vote for me 2?".encode());
		DemoQuad::submit_proposal(Origin::signed(1), "vote for me 3?".encode());
		assert_noop!(
			DemoQuad::submit_proposal(Origin::signed(1), "vote for me 4?".encode()),
			Error::<Test>::TooManyProposalsInBloc
		);
	});
}

#[test]
fn should_increment_proposal_id() {
	new_test_ext().execute_with(|| {
		mock::Identity::set_hash(Origin::signed(1), hash(b"alice"));
		assert_eq!(DemoQuad::next_proposals_id(), 0);
		DemoQuad::submit_proposal(Origin::signed(1), "vote for me 1?".encode());
		assert_eq!(DemoQuad::next_proposals_id(), 1);
		DemoQuad::submit_proposal(Origin::signed(1), "vote for me 1?".encode());
		assert_eq!(DemoQuad::next_proposals_id(), 2);
	});
}

fn should_emit_event_on_proposal_submission() {
	mock::Identity::set_hash(Origin::signed(1), hash(b"alice"));
	DemoQuad::submit_proposal(Origin::signed(1), "vote for me 1?".encode());
	//assert_eq!(last_event(), DemoQuadEvent::ProposalSubmited(0, 1));
}

#[test]
fn only_identified_account_can_vote() {
	new_test_ext().execute_with(|| {
		mock::Identity::set_hash(Origin::signed(1), hash(b"alice"));
		mock::Identity::set_hash(Origin::signed(2), hash(b"alice"));
		DemoQuad::submit_proposal(Origin::signed(1), "Vote for me".encode());
		assert_ok!(DemoQuad::vote_proposal(Origin::signed(2), 0, Choice::Yes));
		assert_noop!(
			DemoQuad::vote_proposal(Origin::signed(3), 0, Choice::Yes),
			Error::<Test>::NoIdentityFound
		);
	});
}

#[test]
fn can_only_vote_on_existensial_proposal() {
	new_test_ext().execute_with(|| {
		mock::Identity::set_hash(Origin::signed(1), hash(b"alice"));
		DemoQuad::submit_proposal(Origin::signed(1), "Vote for me".encode());
		assert_ok!(DemoQuad::vote_proposal(Origin::signed(1), 0, Choice::Yes));
		assert_noop!(
			DemoQuad::vote_proposal(Origin::signed(1), 1, Choice::Yes),
			Error::<Test>::ProposalNotExists
		);
	});
}

/*
#[test]
fn only_identified_account_can_vote() {
	new_test_ext().execute_with(|| {
		assert_eq!(1, 1);
		//assert_eq!(Balances::total_balance(&2), 10);
		//assert_ok!(SimpleIdentity::kill_hash(Origin::signed(1), 2));
		//assert_eq!(Balances::total_balance(&2), 8);
		//assert_eq!(<Identities<Test>>::get(2), None);
	});
}*/
