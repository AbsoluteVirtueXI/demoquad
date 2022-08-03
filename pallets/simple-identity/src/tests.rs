use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use sp_core::Hasher;
use sp_runtime::traits::{BadOrigin, BlakeTwo256};

fn hash(b: &[u8]) -> <Test as frame_system::Config>::Hash {
	BlakeTwo256::hash(b)
}

#[test]
fn kill_hash_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(SimpleIdentity::set_hash(Origin::signed(2), hash(b"Dave")));
		assert_eq!(Balances::total_balance(&2), 10);
		assert_ok!(SimpleIdentity::kill_hash(Origin::signed(1), 2));
		assert_eq!(Balances::total_balance(&2), 8);
		assert_eq!(<Identities<Test>>::get(2), None);
	});
}

#[test]
fn force_hash_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(SimpleIdentity::set_hash(Origin::signed(2), hash(b"Dave")));
		assert_eq!(Balances::reserved_balance(2), 2);
		assert_ok!(SimpleIdentity::force_hash(Origin::signed(1), 2, hash(b"Dr. Brubeck, III")));
		assert_eq!(Balances::reserved_balance(2), 2);
		let (name, amount) = <Identities<Test>>::get(2).unwrap();
		assert_eq!(name, hash(b"Dr. Brubeck, III"));
		assert_eq!(amount, 2);
	});
}

#[test]
fn normal_operation_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(SimpleIdentity::set_hash(Origin::signed(1), hash(b"Gav")));
		assert_eq!(Balances::reserved_balance(1), 2);
		assert_eq!(Balances::free_balance(1), 8);
		assert_eq!(<Identities<Test>>::get(1).unwrap().0, hash(b"Gav"));

		assert_ok!(SimpleIdentity::set_hash(Origin::signed(1), hash(b"Gavin")));
		assert_eq!(Balances::reserved_balance(1), 2);
		assert_eq!(Balances::free_balance(1), 8);
		assert_eq!(<Identities<Test>>::get(1).unwrap().0, hash(b"Gavin"));

		assert_ok!(SimpleIdentity::clear_hash(Origin::signed(1)));
		assert_eq!(Balances::reserved_balance(1), 0);
		assert_eq!(Balances::free_balance(1), 10);
	});
}

#[test]
fn error_catching_should_work() {
	new_test_ext().execute_with(|| {
		assert_noop!(SimpleIdentity::clear_hash(Origin::signed(1)), Error::<Test>::NotExists);

		assert_noop!(
			SimpleIdentity::set_hash(Origin::signed(3), hash(b"Dave")),
			pallet_balances::Error::<Test, _>::InsufficientBalance
		);

		assert_ok!(SimpleIdentity::set_hash(Origin::signed(1), hash(b"Dave")));
		assert_noop!(SimpleIdentity::kill_hash(Origin::signed(2), 1), BadOrigin);
		assert_noop!(
			SimpleIdentity::force_hash(Origin::signed(2), 1, hash(b"Whatever")),
			BadOrigin
		);
	});
}
