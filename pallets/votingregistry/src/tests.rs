use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, error::BadOrigin};

#[test]
fn test_balance_assumptions() {
	new_test_ext().execute_with(|| {
		let alice = 1u64;
		assert_ok!(<Test as crate::Config>::Currency::set_balance(
			Origin::root(),
			alice,
			10_000_000,
			6969
		));
		let alice_balance = <Test as crate::Config>::Currency::free_balance(&alice);
		assert_eq!(10_000_000, alice_balance);
	})
}

#[test]
fn call_register() {
	new_test_ext().execute_with(|| {
		let alice = 1u64;
		// Set some balance
		assert_ok!(Balances::set_balance(Origin::root(), alice, 10_000_000, 0));
		// Does not work unsigned
		assert_noop!(VotingRegistry::register(Origin::none()), BadOrigin);
		// Does work signed
		assert_ok!(VotingRegistry::register(Origin::signed(alice)));
		// Does not work if already registered
		assert_noop!(
			VotingRegistry::register(Origin::signed(alice)),
			Error::<Test>::AlreadyRegistered
		);
	})
}
#[test]
fn call_deregister() {
	new_test_ext().execute_with(|| {
		let alice = 1u64;
		// Set some balance
		assert_ok!(Balances::set_balance(Origin::root(), alice, 10_000_000, 0));
		// Tested above, assumed working.
		VotingRegistry::register(Origin::signed(alice)).unwrap();
		// Does not work unsigned
		assert_noop!(VotingRegistry::deregister(Origin::none()), BadOrigin);
		// Does work signed and registered
		assert_ok!(VotingRegistry::deregister(Origin::signed(alice)));
		// Does not work if already deregistered
		assert_noop!(
			VotingRegistry::deregister(Origin::signed(alice)),
			Error::<Test>::NotRegistered
		);
	})
}

//#[test]
//fn it_works_for_default_value() {
//	new_test_ext().execute_with(|| {
//		// Dispatch a signed extrinsic.
//		assert_ok!(VotingRegistry::do_something(Origin::signed(1), 42));
//		// Read pallet storage and assert an expected result.
//		assert_eq!(VotingRegistry::something(), Some(42));
//	});
//}
//
//#[test]
//fn correct_error_for_none_value() {
//	new_test_ext().execute_with(|| {
//		// Ensure the expected error is thrown when no value is present.
//		assert_noop!(VotingRegistry::cause_error(Origin::signed(1)), Error::<Test>::NoneValue);
//	});
//}
