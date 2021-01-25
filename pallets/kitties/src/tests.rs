use crate::{Error, Event, mock::*};
use frame_support::{assert_ok, assert_noop};

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
            System::events()[0].event,
            TestEvent::kitty_event(Event::<Test>::Created(1, 0))
        );
	})
}
#[test]
fn transfer_kitty_success() {
	new_test_ext().execute_with(|| {
		run_to_block(10);

		assert_eq!(Kitties::create(Origin::signed(1),), Ok(()));
		assert_eq!(
            System::events()[0].event,
            TestEvent::kitty_event(Event::<Test>::Created(1, 0))
        );
		assert_ok!(Kitties::transfer(Origin::signed(1), 2, 0));
		assert_eq!(
            System::events()[1].event,
            TestEvent::kitty_event(Event::<Test>::Transfered(1, 2, 0))
        );
	})
}

#[test]
fn breed_kitty_success() {
	new_test_ext().execute_with(|| {
		run_to_block(10);

		assert_eq!(Kitties::create(Origin::signed(1),), Ok(()));

		assert_eq!(
            System::events()[0].event,
            TestEvent::kitty_event(Event::<Test>::Created(1, 0))
		);

		assert_eq!(Kitties::create(Origin::signed(1),), Ok(()));
		assert_eq!(
            System::events()[1].event,
            TestEvent::kitty_event(Event::<Test>::Created(1, 1))
		);
		assert_ok!(Kitties::breed(Origin::signed(1), 0, 1));
		assert_eq!(
            System::events()[2].event,
            TestEvent::kitty_event(Event::<Test>::Created(1, 2))
        );
	})
}
