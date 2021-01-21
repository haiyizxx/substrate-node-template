use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};
use super::*;
#[test]
fn create_claim_works(){
	new_test_ext().execute_with(|| {
		let claim = vec![0,1];
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));
		assert_eq!(Proofs::<Test>::get(&claim),(1, frame_system::Module::<Test>::block_number()));
	})
}

#[test]
fn create_claim_failed_exceed_length_limit(){
	new_test_ext().execute_with(|| {
		let claim = vec![0,1,3,4,5,6];
		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ClaimLengthExceedLimit
		);
	})
}

#[test]
fn create_claim_failed_when_claim_already_exit() {
	new_test_ext().execute_with(|| {
		let claim = vec![0,1];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ProofAlreadyExist
		);
	})
}

#[test]
fn revoke_claim_works(){
	new_test_ext().execute_with(||{
		let claim = vec![0,1];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
		assert_ok!(
			PoeModule::revoke_claim(Origin::signed(1), claim.clone())
		);
	})
}

#[test]
fn revoke_claim_failed_when_claim_is_not_exist() {
	new_test_ext().execute_with(||{
		let claim = vec![0,1];
		assert_noop!(
			PoeModule::revoke_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ClaimNotExist
		);
	})
}


#[test]
fn transfer_claim_failed_when_not_owner() {
	new_test_ext().execute_with(||{
		let claim = vec![0,1];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
		
		assert_noop!(
			PoeModule::revoke_claim(Origin::signed(2), claim.clone()),
			Error::<Test>::NotClaimOwner
		);
	})
}


#[test]
fn transfer_claim_success() {
	new_test_ext().execute_with(||{
		let claim = vec![0,1];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
		
		assert_ok!(PoeModule::transfer_claim(Origin::signed(1), claim.clone(), 2));
		assert_eq!(Proofs::<Test>::get(&claim),(2, frame_system::Module::<Test>::block_number()));

	})
}
#[test]
fn transfer_claim_failed_claim_not_exit() {
	new_test_ext().execute_with(||{
		let claim = vec![0,1];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
		let claim2 = vec![1,2];
		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(1), claim2.clone(), 2),
			Error::<Test>::ClaimNotExist
		);


	})
}

#[test]
fn transfer_claim_failed_not_owner() {
	new_test_ext().execute_with(||{
		let claim = vec![0,1];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(2), claim.clone(),3),
			Error::<Test>::NotClaimOwner
		);
	})
}