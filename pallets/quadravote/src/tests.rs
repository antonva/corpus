use crate::{
	mock::{run_to_block, *},
	CountedProposals, Error,
};
use frame_support::traits::Currency;
use frame_support::{assert_noop, assert_ok, error::BadOrigin};

#[test]
fn call_create_proposal() {
	new_test_ext().execute_with(|| {
		let period_length = <Test as crate::Config>::PeriodLength::get();
		let alice = 0u64;
		// Set some balance
		assert_ok!(Balances::set_balance(Origin::root(), alice, 10_000_000, 0));

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
		assert_noop!(
			Quadravote::create_proposal(Origin::signed(alice), [0u8; 32]),
			Error::<Test>::NotIdentified
		);

		// Register alice with the votingregistry. Unwraps should be covered
		// by the pallet-votingregistry unit tests.
		VotingRegistry::register(Origin::signed(alice)).unwrap();

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

		// Set some balance for alice
		assert_ok!(Balances::set_balance(Origin::root(), alice, 10_000_000, 0));

		// Register alice with the votingregistry. Unwraps should be covered
		// by the pallet-votingregistry unit tests
		VotingRegistry::register(Origin::signed(alice)).unwrap();

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

		// Set some balance for alice
		assert_ok!(Balances::set_balance(Origin::root(), alice, 10_000_000, 0));

		// Set some balance for evelyn
		assert_ok!(Balances::set_balance(Origin::root(), evelyn, 10_000_000, 0));

		// Register alice with the votingregistry. Unwraps should be covered
		// by the pallet-votingregistry unit tests
		VotingRegistry::register(Origin::signed(alice)).unwrap();

		// Unsigned fails
		assert_noop!(Quadravote::withdraw_proposal(Origin::none(), [0u8; 32]), BadOrigin);

		// Signed fails while in the voting period
		assert_noop!(
			Quadravote::withdraw_proposal(Origin::signed(alice), [0u8; 32]),
			Error::<Test>::NotInProposalPeriod
		);

		// Advance the chain by the period length
		run_to_block(period_length.into());

		// Set up an existing proposal
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

#[test]
fn call_cast_vote() {
	new_test_ext().execute_with(|| {
		let period_length = <Test as crate::Config>::PeriodLength::get();
		let alice = 0u64;
		let evelyn = 1u64;

		// Set some balance for alice
		assert_ok!(Balances::set_balance(Origin::root(), alice, 10_000_000, 0));

		// Set some balance for evelyn
		assert_ok!(Balances::set_balance(Origin::root(), evelyn, 10_000_000, 0));

		// Unsigned call fails
		assert_noop!(Quadravote::cast_vote(Origin::none(), 0, 0, 0), BadOrigin);
		// Unidentified call fails
		assert_noop!(
			Quadravote::cast_vote(Origin::signed(alice), 0, 0, 0),
			Error::<Test>::NotIdentified
		);
		// Register alice & evelyn with the votingregistry.
		VotingRegistry::register(Origin::signed(alice)).unwrap();
		VotingRegistry::register(Origin::signed(evelyn)).unwrap();

		// Proposal period
		// Advance the chain by the period length
		let mut current_height = period_length;
		run_to_block(current_height.into());
		// Submit 2 proposals
		assert_ok!(Quadravote::create_proposal(Origin::signed(alice), [0u8; 32]));
		assert_ok!(Quadravote::create_proposal(Origin::signed(evelyn), [1u8; 32]));

		// Voting period
		// Advance the chain by the period length
		current_height += period_length;
		run_to_block(current_height.into());

		// Submit 5 votes for a proposal
		// At this point alice is missing 50 from registering with the
		// identity provider.
		assert_ok!(Quadravote::cast_vote(Origin::signed(alice), 0, 5, 0));
		assert_eq!(Balances::free_balance(&alice), 9_999_925);

		// Submit another 5 votes
		assert_ok!(Quadravote::cast_vote(Origin::signed(alice), 0, 5, 0));
		assert_eq!(Balances::free_balance(&alice), 9_999_850);

		// Submit another 5 votes
		assert_noop!(
			Quadravote::cast_vote(Origin::signed(alice), 0, 5, 0),
			Error::<Test>::AllVotesCastForAccount
		);

		// Submit another 5 votes, this time exceeding the account voting limit
		assert_noop!(
			Quadravote::cast_vote(Origin::signed(alice), 0, 5, 0),
			Error::<Test>::AllVotesCastForAccount
		);

		// Submit 5 votes to another proposal, should also affect the voting limit
		assert_noop!(
			Quadravote::cast_vote(Origin::signed(alice), 1, 5, 0),
			Error::<Test>::AllVotesCastForAccount
		);

		// Proposal period
		// Advance the chain by the period length
		current_height += period_length;
		run_to_block(current_height.into());

		// Refunded the voter balance
		assert_eq!(Balances::free_balance(&alice), 9_999_950);
	});
}
