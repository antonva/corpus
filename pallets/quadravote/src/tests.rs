use crate::{
	mock::{run_to_block, *},
	Error,
};
use frame_support::{assert_noop, assert_ok, error::BadOrigin};

#[test]
fn call_create_proposal() {
	new_test_ext().execute_with(|| {
		let period_length = <Test as crate::Config>::PeriodLength::get();
		let alice = 0u64;
		// Unsigned fails
		assert_noop!(Quadravote::create_proposal(Origin::none(), [0u8; 32]), BadOrigin);
		// Signed fails while in the voting period
		assert_noop!(
			Quadravote::create_proposal(Origin::signed(alice), [0u8; 32]),
			Error::<Test>::NotInProposalPeriod
		);
		// Advance the chain by the period length to get to proposal period
		run_to_block(period_length.into());
		// Signed does not work if not identified by the voting registry
		// Signed works in the proposal period
		assert_ok!(Quadravote::create_proposal(Origin::signed(alice), [0u8; 32]));
		// Submitting the same proposal fails
		assert_noop!(
			Quadravote::create_proposal(Origin::signed(alice), [0u8; 32]),
			Error::<Test>::ProposalAlreadyExists
		);
	});
}

#[test]
fn call_create_proposal_maximum() {
	new_test_ext().execute_with(|| {
		let period_length = <Test as crate::Config>::PeriodLength::get();
		let max_proposals = <Test as crate::Config>::MaxProposals::get();
		let alice = 0u64;
		// Advance the chain by the period length to get to proposal period
		run_to_block(period_length.into());
		for i in 0..max_proposals {
			assert_ok!(Quadravote::create_proposal(Origin::signed(alice), [i as u8; 32]));
		}
		assert_noop!(
			Quadravote::create_proposal(Origin::signed(alice), [10 as u8; 32]),
			Error::<Test>::TooManyProposals
		);
	});
}
#[test]
fn call_withdraw_proposal() {
	new_test_ext().execute_with(|| {
		let period_length = <Test as crate::Config>::PeriodLength::get();
		let alice = 0u64;
		let evelyn = 1u64;
		// Unsigned fails
		assert_noop!(Quadravote::withdraw_proposal(Origin::none(), [0u8; 32]), BadOrigin);
		// Signed fails while in the voting period
		assert_noop!(
			Quadravote::withdraw_proposal(Origin::signed(alice), [0u8; 32]),
			Error::<Test>::NotInProposalPeriod
		);
		// Advance the chain by the period length
		run_to_block(period_length.into());
		// Signed does not work if not identified by the voting registry

		// Test above demonstrates that this works so unwrap is less icky
		Quadravote::create_proposal(Origin::signed(alice), [0u8; 32]).unwrap();
		// Withdrawing as another account id fails
		assert_noop!(
			Quadravote::withdraw_proposal(Origin::signed(evelyn), [0u8; 32]),
			Error::<Test>::NotYourProposal
		);
		// Signed works in the proposal period
		assert_ok!(Quadravote::withdraw_proposal(Origin::signed(alice), [0u8; 32]));
		// Withdrawing a withdrawn proposal fails
		assert_noop!(
			Quadravote::withdraw_proposal(Origin::signed(alice), [0u8; 32]),
			Error::<Test>::ProposalDoesNotExist
		);
	});
}
