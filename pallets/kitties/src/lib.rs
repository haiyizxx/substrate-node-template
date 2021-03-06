#![cfg_attr(not(feature="std"), no_std)]

use codec::{Encode, Decode};
use frame_support::{
	decl_module, decl_storage, decl_event, decl_error, sp_runtime, ensure,
	StorageValue, StorageMap, Parameter, weights::Weight,
	traits::{Randomness, Currency, ReservableCurrency, Get, ExistenceRequirement},

};
use sp_io::hashing::blake2_128;
use frame_system::ensure_signed;
use sp_runtime::DispatchError;
use sp_std::prelude::*;
use sp_runtime::traits::{AtLeast32Bit, Bounded, One, Member, AtLeast32BitUnsigned};


#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    //Random kitty dna
    type Randomness: Randomness<Self::Hash>;

	type KittyIndex: Member + Parameter + Default + Copy + AtLeast32Bit;

	/// The currency mechanism.
	type Currency: ReservableCurrency<Self::AccountId>;
	type StakeForKitty: Get<BalanceOf<Self>>;

}

// Kitty dna data, array of u8, length 16
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

		pub Test: u32;
		pub KittyPrices get(fn kitty_price): map hasher(blake2_128_concat) T::KittyIndex => Option<BalanceOf<T>>;
    }
}

decl_error! {
    pub enum Error for Module<T:Trait> {
        KittiesCountOverflow,
        InvalidKittyId,
		RequireDifferentParent,
		KittyNotExit,
		NotKittyOwner,
		NotEnoughBalance,
		CantTransferToSelf,
		NotForSale,
		PriceTooLow
    }
}

decl_event!(
	pub enum Event<T> where
	AccountId = <T as frame_system::Trait>::AccountId,
	KittyIndex = <T as Trait>::KittyIndex,
	Balance = BalanceOf<T>
	{
        Created(AccountId, KittyIndex),
		Transfered(AccountId, AccountId, KittyIndex),
		StakeForKitty(AccountId, Balance),
		UnstakeForKitty(AccountId, Balance),
		// StakeTransferred(AccountId, AccountId, Balance),
    }
);

decl_module! {
	pub struct Module<T:Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;

        #[weight=0]
        pub fn create(origin) {
			let sender = ensure_signed(origin)?;

			Self::reserve(sender.clone(), T::StakeForKitty::get());

            let kitty_id = Self::next_kitty_id()?;
            let dna = Self::random_value(&sender);
            let kitty = Kitty(dna);

            Self::insert_kitty(&sender, kitty_id, kitty);

            Self::deposit_event(RawEvent::Created(sender, kitty_id));
        }

        #[weight=0]
        pub fn transfer(origin, to: T::AccountId, kitty_id: T::KittyIndex) {
			let sender = ensure_signed(origin)?;
			Self::transfer_stake(sender.clone(), to.clone(), T::StakeForKitty::get());

			// Have to check if the sender own the kitty
			let kitty_owner = <KittyOwners<T>>::get(kitty_id)
								.ok_or(Error::<T>::KittyNotExit)
								.unwrap();
			ensure!(sender == kitty_owner, Error::<T>::NotKittyOwner);
			ensure!(sender != to, Error::<T>::CantTransferToSelf);

			<KittyOwners<T>>::insert(kitty_id, to.clone());

			// Update owner
			AccountKitties::<T>::mutate(&sender, |val| val.retain(|&x| x != kitty_id));


			Self::add_kitty_to_owner(&to, kitty_id);

			Self::deposit_event(RawEvent::Transfered(sender, to, kitty_id));
        }

        #[weight=0]
        pub fn breed(origin, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) {
			let sender = ensure_signed(origin)?;
			Self::reserve(sender.clone(), T::StakeForKitty::get());
            let new_kitty_id = Self::do_breed(&sender, kitty_id_1, kitty_id_2)?;
            Self::deposit_event(RawEvent::Created(sender, new_kitty_id));
		}

		fn on_runtime_upgrade() -> Weight {
			// Map from old u16 to u32
			let _ = Test::translate::<_, _>(|value: Option<u16>| value.map(|v| v as u32));
			0
		}

		#[weight = 0]
		pub fn ask(orign, kitty_id: T::KittyIndex, new_price: Option<BalanceOf<T>>) {
			let sender = ensure_signed(orign)?;
			ensure!(Self::kitty_owner(&kitty_id) == Some(sender.clone()), Error::<T>::NotKittyOwner);
			<KittyPrices<T>>::mutate_exists(kitty_id, |price| *price = new_price);
		}

		#[weight = 0]
		pub fn buy(orign, kitty_id: T::KittyIndex, price: Option<BalanceOf<T>>) {
			let sender = ensure_signed(orign)?;
			let owner = Self::kitty_owner(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
			let kitty_price = Self::kitty_price(kitty_id).ok_or(Error::<T>::NotForSale)?;
			ensure!(price.unwrap() >= kitty_price, Error::<T>::PriceTooLow);
			T::Currency::transfer(&sender, &owner, kitty_price, ExistenceRequirement::KeepAlive)?;
			<KittyPrices<T>>::remove(kitty_id);
			<KittyOwners<T>>::insert(kitty_id, sender.clone());
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

		// Add kitty relationship
		Self::add_kitty_to_family_tree(kitty_id_1, kitty_id_2, kitty_id);


        Ok(kitty_id)

	}

	fn reserve(account: T::AccountId, amount: BalanceOf<T>) {
		// Reserve
		// ensure!(T::Currency::can_reserve(&account, T::StakeForKitty::get()) == true, Error::<T>::NotEnoughBalance);
		let _ = T::Currency::reserve(&account, amount).map_err(|_| Error::<T>::NotEnoughBalance);

		Self::deposit_event(RawEvent::StakeForKitty(account, amount));

	}

	fn transfer_stake(from: T::AccountId, to: T::AccountId, amount: BalanceOf<T>) {
		T::Currency::unreserve(&from, amount);
		let _ = T::Currency::transfer(&from, &to, amount, ExistenceRequirement::KeepAlive);
		let _ =  T::Currency::reserve(&to, amount);
		// Self::deposit_event(RawEvent::StakeTransferred(from, to, amount));
	}
}
