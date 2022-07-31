use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn create_proposal_rejects_unsigned_origin() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_noop!(
			Quadravote::create_proposal(Origin::none(), 42),
			Error::<Test>::ExtrinsicNotSigned
		);
	});
}

#[test]
fn create_proposal_rejects_when_not_in_proposal_period() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_noop!(
			Quadravote::create_proposal(Origin::signed(1), 42),
			Error::<Test>::NotInProposalPeriod
		);
	});
}

//#[test]
//fn correct_error_for_none_value() {
//	new_test_ext().execute_with(|| {
//		// Ensure the expected error is thrown when no value is present.
//		assert_noop!(Quadravote::cause_error(Origin::signed(1)), Error::<Test>::NoneValue);
//	});
//}
