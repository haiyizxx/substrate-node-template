#![cfg_attr(not(feature="std"), no_std)]

use codec::{Encode, Decode};
// use dispatch::DispatchError;
use frame_support::{
    decl_module, decl_storage, decl_event, decl_error, sp_runtime,
    dispatch, ensure, StorageValue, StorageMap, traits::Randomness, Parameter};
use sp_io::hashing::blake2_128;
use frame_system::ensure_signed;
use sp_runtime::DispatchError;
use sp_std::prelude::*;
use sp_runtime::traits::{Member, AtLeast32Bit, Bounded, One};


// Kitty dna data, array of u8, length 16
#[derive(Encode, Decode)]
pub struct Kitty(pub [u8; 16]);

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    //Random kitty dna
    type Randomness: Randomness<Self::Hash>;

	// type KittyIndex: Parameter + Member + Default;
	type KittyIndex: Parameter + Default + Copy + AtLeast32Bit;
}

decl_storage! {
	trait Store for Module<T: Trait> as Kitties {
        pub Kitties get(fn kitties): map hasher(blake2_128_concat) <T as Trait>::KittyIndex => Option<Kitty>;
        // Get number of kitties created
        pub KittiesCount get(fn kitties_count): <T as Trait>::KittyIndex;
        pub KittyOwners get(fn kitty_owner): map hasher(blake2_128_concat) <T as Trait>::KittyIndex => Option<T::AccountId>;
        // pub KittyOwners2 get(fn kitty_owner2): map hasher(blake2_128_concat) <T as Trait>::AccountId => Vec<T::AccountId>;

    }
}

decl_error! {
    pub enum Error for Module<T:Trait> {
        KittiesCountOverflow,
        InvalidKittyId,
        RequireDifferentParent
    }
}

decl_event!(
	pub enum Event<T> where
	AccountId = <T as frame_system::Trait>::AccountId,
	// KittyIndex = <T as Trait>::KittyIndex
	<T as Trait>::KittyIndex
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
            <KittyOwners<T>>::insert(kitty_id, to.clone());
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

    }

    fn next_kitty_id() -> sp_std::result::Result<T::KittyIndex, DispatchError> {
        let kitty_id = Self::kitties_count();
        if kitty_id == T::KittyIndex::max_value() {
            return Err(Error::<T>::KittiesCountOverflow.into());
        }
        Ok(kitty_id)
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
        Ok(kitty_id)

    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sp_core::H256;
    use frame_support::{impl_outer_origin, parameter_types, weights::Weight,
        traits::{OnFinalize, OnInitialize}
    };
    use sp_runtime::{
        traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill
    };
    use frame_system as system;

    // Mock check signature
    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    #[derive(Clone, Eq, PartialEq, Debug)]

    pub struct Test;
    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: Weight = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);

    }

    impl system::Trait for Test {
        type BaseCallFilter = ();
        type Origin = Origin;
        type Call = ();
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type DbWeight = ();
        type BlockExecutionWeight = ();
        type ExtrinsicBaseWeight = ();
        type MaximumExtrinsicWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
        type PalletInfo = ();
        type AccountData = ();
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type SystemWeightInfo = ();
    }

    type Randomness = pallet_randomness_collective_flip::Module<Test>;

    impl Trait for Test {
        type Event = ();
        type Randomness = Randomness;
    }

    pub type Kitties = Module<Test>;
    pub type System = frame_system::Module<Test>;

    fn run_to_block(n: u64) {
        while System::block_number() < n {
            Kitties::on_finalize(System::block_number());
            System::on_finalize(System::block_number());
            System::set_block_number(System::block_number()+1);
            System::on_initialize(System::block_number());
            Kitties::on_initialize(System::block_number());
        }
    }

    // Build genesis storage according to the mock runtime.
    pub fn new_test_ext() -> sp_io::TestExternalities {
        system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
    }

    #[test]
    fn owned_kitties_can_append_values() {
        new_test_ext().execute_with(|| {
            run_to_block(10);
            assert_eq!(Kitties::create(Origin::signed(1),), Ok(()));
        })
    }

}
