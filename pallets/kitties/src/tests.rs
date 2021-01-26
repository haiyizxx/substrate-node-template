use crate::{Error, Event, mock::*};
use frame_support::{assert_ok, assert_noop, assert_err};

#[test]
fn owned_kitties_can_append_values() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_eq!(Kitties::create(Origin::signed(1),), Ok(()));
	})
}

#[test]
fn create_kitty_success() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_eq!(Kitties::create(Origin::signed(1),), Ok(()));
		assert_eq!(
            System::events()[1].event,
            TestEvent::kitty_event(Event::<Test>::StakeForKitty(1, 1000000))
		);
		assert_eq!(
            System::events()[2].event,
            TestEvent::kitty_event(Event::<Test>::Created(1, 0))
        );
	})
}
#[test]
fn transfer_kitty_success() {
	new_test_ext().execute_with(|| {
		run_to_block(10);

		assert_eq!(Kitties::create(Origin::signed(1),), Ok(()));
		assert_ok!(Kitties::transfer(Origin::signed(1), 2, 0));
		assert_eq!(
            System::events()[6].event,
            TestEvent::kitty_event(Event::<Test>::Transfered(1, 2, 0))
        );
	})
}
#[test]
fn transfer_kitty_failed_not_owner() {
	new_test_ext().execute_with(|| {
		run_to_block(10);

		assert_eq!(Kitties::create(Origin::signed(1),), Ok(()));

		assert_eq!(
            System::events()[1].event,
            TestEvent::kitty_event(Event::<Test>::StakeForKitty(1, 1000000))
		);
		assert_eq!(
            System::events()[2].event,
            TestEvent::kitty_event(Event::<Test>::Created(1, 0))
        );
		assert_err!(
			Kitties::transfer(Origin::signed(2), 3, 0),
			Error::<Test>::NotKittyOwner
		);

	})
}

#[test]
fn transfer_kitty_failed_to_self() {
	new_test_ext().execute_with(|| {
		run_to_block(10);

		assert_eq!(Kitties::create(Origin::signed(1),), Ok(()));
		assert_eq!(
            System::events()[1].event,
            TestEvent::kitty_event(Event::<Test>::StakeForKitty(1, 1000000))
		);
		assert_eq!(
            System::events()[2].event,
            TestEvent::kitty_event(Event::<Test>::Created(1, 0))
        );

		assert_err!(
			Kitties::transfer(Origin::signed(1), 1, 0),
			Error::<Test>::CantTransferToSelf
		);

	})
}


#[test]
fn breed_kitty_success() {
	new_test_ext().execute_with(|| {
		run_to_block(10);

		assert_eq!(Kitties::create(Origin::signed(1),), Ok(()));

		assert_eq!(Kitties::create(Origin::signed(1),), Ok(()));

		assert_ok!(Kitties::breed(Origin::signed(1), 0, 1));
		assert_eq!(
            System::events()[8].event,
            TestEvent::kitty_event(Event::<Test>::Created(1, 2))
        );
	})
}

#[test]
fn breed_kitty_failed_invalid_id() {
	new_test_ext().execute_with(|| {
		run_to_block(10);

		assert_eq!(Kitties::create(Origin::signed(1),), Ok(()));

		assert_eq!(Kitties::create(Origin::signed(1),), Ok(()));

		assert_err!(
			Kitties::breed(Origin::signed(1), 0, 2),
			Error::<Test>::InvalidKittyId
		);

	})
}
#[test]
fn breed_kitty_failed_same_parent() {
	new_test_ext().execute_with(|| {
		run_to_block(10);

		assert_eq!(Kitties::create(Origin::signed(1),), Ok(()));

		assert_eq!(Kitties::create(Origin::signed(1),), Ok(()));

		assert_err!(
			Kitties::breed(Origin::signed(1), 0, 0),
			Error::<Test>::RequireDifferentParent
		);

	})
}
