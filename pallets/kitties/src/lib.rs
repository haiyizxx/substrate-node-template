#![cfg_attr(not(feature="std"), no_std)]

use codec::{Encode, Decode};
use frame_support::{
    decl_module, decl_storage, decl_event, decl_error, sp_runtime,
    dispatch, ensure, StorageValue, StorageMap, traits::Randomness, Parameter};
use sp_io::hashing::blake2_128;
use frame_system::ensure_signed;
use sp_runtime::DispatchError;
use sp_std::prelude::*;
use sp_runtime::traits::{AtLeast32Bit, Bounded, One, Zero};


#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    //Random kitty dna
    type Randomness: Randomness<Self::Hash>;

	type KittyIndex: Parameter + Default + Copy + AtLeast32Bit;

}

// Kitty dna data, array of u8, length 16
// TODO: u32 might overfolow if kittyindex is changed from runtime
#[derive(Encode, Decode)]
pub struct Kitty(pub [u8; 16]);


decl_storage! {

	trait Store for Module<T: Trait> as Kitties {
        pub Kitties get(fn kitties): map hasher(blake2_128_concat) T::KittyIndex => Option<Kitty>;
        // Get number of kitties created
        pub KittiesCount get(fn kitties_count): T::KittyIndex;
        pub KittyOwners get(fn kitty_owner): map hasher(blake2_128_concat) T::KittyIndex => Option<T::AccountId>;
		// Get all kitties belong to an account
		pub AccountKitties get(fn account_kitty): map hasher(blake2_128_concat) T::AccountId => Vec<T::KittyIndex>;

		// Track kitty's children and partner
		pub FamilyMap get(fn family_map):  double_map hasher(blake2_128_concat) T::KittyIndex, hasher(blake2_128_concat) &'static str  => Vec<T::KittyIndex>;
		// Track kitty and parents
		pub KittyParents get(fn kitty_parents):  map hasher(blake2_128_concat) T::KittyIndex => (T::KittyIndex, T::KittyIndex);
		// Track kitty's sibling
		pub ParentsChildren get(fn sibling):  map hasher(blake2_128_concat) (T::KittyIndex, T::KittyIndex) => Vec<T::KittyIndex>;

    }
}

decl_error! {
    pub enum Error for Module<T:Trait> {
        KittiesCountOverflow,
        InvalidKittyId,
		RequireDifferentParent,
		KittyNotExit,
		NotKittyOwner,
    }
}

decl_event!(
	pub enum Event<T> where
	AccountId = <T as frame_system::Trait>::AccountId,
	KittyIndex = <T as Trait>::KittyIndex
	{
        Created(AccountId, KittyIndex),
        Transfered(AccountId, AccountId, KittyIndex),
    }
);

decl_module! {
	pub struct Module<T:Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;

        #[weight=0]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let kitty_id = Self::next_kitty_id()?;
            let dna = Self::random_value(&sender);
            let kitty = Kitty(dna);

            Self::insert_kitty(&sender, kitty_id, kitty);

            Self::deposit_event(RawEvent::Created(sender, kitty_id));
        }

        #[weight=0]
        pub fn transfer(origin, to: T::AccountId, kitty_id: T::KittyIndex) {
			let sender = ensure_signed(origin)?;

			// Have to check if the sender own the kitty
			let kitty_owner = <KittyOwners<T>>::get(kitty_id)
								.ok_or(Error::<T>::KittyNotExit)
								.unwrap();
			ensure!(sender == kitty_owner, Error::<T>::NotKittyOwner);

			<KittyOwners<T>>::insert(kitty_id, to.clone());

			// Update owner
			AccountKitties::<T>::mutate(&sender, |val| val.retain(|&x| x != kitty_id));


			Self::add_kitty_to_owner(&to, kitty_id);

			Self::deposit_event(RawEvent::Transfered(sender, to, kitty_id));
        }

        #[weight=0]
        pub fn breed(origin, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) {
            let sender = ensure_signed(origin)?;
            let new_kitty_id = Self::do_breed(&sender, kitty_id_1, kitty_id_2)?;
            Self::deposit_event(RawEvent::Created(sender, new_kitty_id));
        }
    }
}

fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
    // if selector =1, use 1, if selector = 0, use 2
    (selector & dna1) | (!selector & dna2)
}


impl <T:Trait> Module<T> {

    fn insert_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex, kitty: Kitty) {
        Kitties::<T>::insert(kitty_id, kitty);
        KittiesCount::<T>::put(kitty_id + One::one());
		<KittyOwners<T>>::insert(kitty_id, owner);

		Self::add_kitty_to_owner(&owner, kitty_id);
    }

    fn next_kitty_id() -> sp_std::result::Result<T::KittyIndex, DispatchError> {
		let  kitty_id = Self::kitties_count();

        if kitty_id == T::KittyIndex::max_value() {
            return Err(Error::<T>::KittiesCountOverflow.into());
        }
        Ok(kitty_id)
    }

	fn add_kitty_to_owner(owner: &T::AccountId, kitty_id: T::KittyIndex,) {
		match AccountKitties::<T>::contains_key(&owner) {
			true => {
				AccountKitties::<T>::append(owner, kitty_id)
			},
			false => {
				AccountKitties::<T>::insert(owner, vec![kitty_id])
			}
		}
	}

	fn add_kitty_to_family_tree(parent_1: T::KittyIndex, parent_2: T::KittyIndex, child:T::KittyIndex) {


		for (idx, p) in [parent_1, parent_2].iter().enumerate() {
			// Parent add child

			match FamilyMap::<T>::contains_key(p, "Children") {
				true => {
					FamilyMap::<T>::mutate(p, "Children",  |children| children.push(child));

				},
				false => {
					FamilyMap::<T>::insert(p, "Children", vec![child]);
				}
			}
			// Parent add partner
			let mut p2 = parent_2;
			if idx == 1 {
				p2 = parent_1;
			}
			match FamilyMap::<T>::contains_key(p, "Partner") {
				true => {
					FamilyMap::<T>::mutate(p, "Partner",  |partner| if !partner.contains(&p2){partner.push(p2)});

				},
				false => {
					FamilyMap::<T>::insert(p, "Partner", vec![p2]);
				}
			}
		}

		// Add kitty -> parents map
		let mut parents_tuple = (parent_1, parent_2);
		if parent_1 > parent_2 {
			parents_tuple = (parent_2, parent_1);
		}
		<KittyParents<T>>::insert(child, parents_tuple.clone());

		// Add parents -> children map
		if <ParentsChildren<T>>::contains_key(parents_tuple) {
			<ParentsChildren<T>>::mutate(parents_tuple, |children| children.push(child));
        } else {
            <ParentsChildren<T>>::insert(parents_tuple, vec![child]);
        }

	}


    fn random_value(sender: &T::AccountId) -> [u8; 16] {
        let payload = (
            T::Randomness::random_seed(),
            &sender,
            <frame_system::Module<T>>::extrinsic_index(), //Transaction index
        );
        // Hash with blake_128, 128 bit
        payload.using_encoded(blake2_128)
    }



    fn do_breed(sender: &T::AccountId, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> sp_std::result::Result<T::KittyIndex, DispatchError>
    {
        let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
        let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;

        ensure!(kitty_id_1 != kitty_id_2, Error::<T>::RequireDifferentParent);

        let kitty_id = Self::next_kitty_id()?;

        let kitty1_dna = kitty1.0;
        let kitty2_dna = kitty2.0;
        let selector = Self::random_value(&sender);
        let mut new_dna = [0u8; 16];

        for i in 0..kitty1_dna.len() {
            new_dna[i] = combine_dna(kitty1_dna[i], kitty2_dna[i], selector[i]);
        }
		Self::insert_kitty(sender, kitty_id, Kitty(new_dna));

		// Add parent -> children
		Self::add_kitty_to_family_tree(kitty_id_1, kitty_id_2, kitty_id);


        Ok(kitty_id)

    }
}
