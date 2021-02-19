#![cfg_attr(not(feature = "std"), no_std)]

/// NFT Pallet
use frame_support::{
	dispatch::DispatchResult, decl_module, decl_storage, decl_event, decl_error,
	ensure,
    pallet_prelude::Get,
};
use frame_system::{self as system, ensure_signed };
use codec::{Encode, Decode};
use sp_runtime::sp_std::prelude::Vec;

/// Item define
/// owner   
/// name    
/// icon
/// description
/// mode      The mode of item,Type is ItemMode
/// decimal_digits      decimal digits ,only for mode = Separable(_,_)
/// custom_data_size    Data size of item
/// 
/// 
#[derive(Encode, Decode, Debug, Default, Clone, PartialEq)]
pub struct CollectionType<AccountId> {
	pub owner: AccountId,
    pub name: Vec<u8>,
    pub icon: Vec<u8>,
    pub description: Vec<u8>,
    pub mode: ItemMode,
    pub decimal_digits: u32,
	pub custom_data_size: u32,
}

/// Item mode
///     Inseparable There can only be one owner
///     Separable   Can have multiple owners, each with a portion
#[derive(Encode, Decode, Debug, Eq, Clone, PartialEq)]
pub enum ItemMode {
    Invalid,
    Inseparable(u32),
	Separable(u32, u32),
}
impl Into<u8> for ItemMode {
    fn into(self) -> u8{
        match self {
            ItemMode::Invalid => 0,
            ItemMode::Inseparable(_) => 1,
            ItemMode::Separable(_, _) => 2,
        }
    }
}
impl Default for ItemMode { fn default() -> Self { Self::Invalid } }

/// item define
/// creater
/// name
/// data    Custom data of the item
/// image
/// ext_link
/// description
#[derive(Encode, Decode, Debug, Default, Clone, PartialEq)]
pub struct ItemType<AccountId> {
    pub creater : AccountId,
    pub name:   Vec<u8>,
    pub data: Vec<u8>,
    pub image: Vec<u8>,
    pub ext_link: Vec<u8>,
    pub description: Vec<u8>,
}

pub trait Config: system::Config {
    type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;

    // Set limits for fields related to collection
    type MaxCollectionNameLength: Get<u32>;
    type MaxCollectionIconLength: Get<u32>;
    type MaxCollectionDescriptionLength: Get<u32>;
    type MaxCollectionCustomDataSize: Get<u32>;
    type MaxCollectionDecimalDigits: Get<u32>;
    // Set limits for fields related to item
    type MaxItemNameLength: Get<u32>;
    type MaxItemImageLength: Get<u32>;
    type MaxItemExtLinkLength: Get<u32>;
    type MaxItemDescriptionLength: Get<u32>;
}

decl_storage! {
    trait Store for Module<T: Config> as Nft {
        NextCollectionID: u64;
        NextItemID: map hasher(blake2_128_concat) u64 => u64;

        // collection_id => Option Collection
        pub CollectionList get(fn collection_list): map hasher(identity) u64 => Option<CollectionType<T::AccountId>>;
        // collection_id => vec of creater
        pub CollectionCreaterList get(fn collection_creater_list): map hasher(identity) u64 => Vec<T::AccountId>;
        // collection_id + item_id  => Item
        pub ItemList get(fn item_list): double_map hasher(blake2_128_concat) u64, hasher(blake2_128_concat) u64 => Option<ItemType<T::AccountId>>;
        // collection_id + item_id  => (owner , amount)
        pub OwnerList get(fn owner_list): double_map hasher(blake2_128_concat) u64, hasher(blake2_128_concat) u64 => Vec<(T::AccountId, u128)>;
        // owner + (collection_id , item_id ) => amount
        pub OwnedList get(fn owned_list): double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) (u64, u64) => u128;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Config>::AccountId,
    {
        // CollectionCreated(creater, collection_id)
        CollectionCreated(AccountId, u64),
        // ItemCreated(creater, collection_id, item_id)
        ItemCreated(AccountId, u64, u64), 
        // ItemDestroyed(collection_id, item_id)
        ItemDestroyed(u64, u64),
    }
);

decl_error! {
	pub enum Error for Module<T: Config> {
        CollectionNameTooLong,
        CollectionIconTooLong,
        CollectionDescriptionTooLong,
        CollectionCustomDataSizeTooBig,
        CollectionDecimalDigitsTooBig,

        CollectionNotExists,
        NotOwnerOfCollection,
	}
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;

        #[weight = 1000]
        pub fn create_collection(origin, name: Vec<u8>, icon: Vec<u8>, description: Vec<u8>, mode: ItemMode) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!( name.len() as u32 <= T::MaxCollectionNameLength::get() , Error::<T>::CollectionNameTooLong);
            ensure!( icon.len() as u32 <= T::MaxCollectionIconLength::get() , Error::<T>::CollectionIconTooLong);
            ensure!( description.len() as u32 <= T::MaxCollectionDescriptionLength::get() , Error::<T>::CollectionDescriptionTooLong);
            
            let custom_data_size = match mode {
                ItemMode::Inseparable(size) => size,
                ItemMode::Separable(size,_) => size,
                _ => 0,
            };
            ensure!( custom_data_size <= T::MaxCollectionCustomDataSize::get() , Error::<T>::CollectionCustomDataSizeTooBig);

            let decimal_digits = match mode {
                ItemMode::Separable(_, decimal) => decimal,
                _ => 0,
            };
            ensure!( decimal_digits <= T::MaxCollectionDecimalDigits::get() , Error::<T>::CollectionDecimalDigitsTooBig);
            
            let next_id = NextCollectionID::get().checked_add(1).expect( "CollectionIdError" );
            
            NextCollectionID::put(next_id);

            let new_collection = CollectionType {
                owner : sender.clone(),
                name : name,
                icon : icon,
                description : description,
                mode : mode.clone(),
                decimal_digits : decimal_digits,
                custom_data_size : custom_data_size,
            };

            <CollectionList<T>>::insert(next_id, new_collection);

            Self::deposit_event(RawEvent::CollectionCreated(sender, next_id));

            Ok(())
        }
        #[weight=0]
        pub fn transfer_collection(origin, to: T::AccountId, collection_id: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let mut collection = Self::collection_list(collection_id).ok_or( Error::<T>::CollectionNotExists )?;
            
            ensure!( sender == collection.owner, Error::<T>::NotOwnerOfCollection );

            collection.owner = sender;
            <CollectionList<T>>::insert(collection_id, collection);

            Ok(())
        }
        #[weight=0]
        pub fn destroy_collection(origin, collection_id: u64) -> DispatchResult {
            
            Ok(())
        }
        #[weight=0]
        pub fn set_collection_creater(origin, creater: T::AccountId, collection_id: u64) -> DispatchResult {
            
            Ok(())
        }
        #[weight = 1000]
        pub fn remove_collection_creater(origin, creater: T::AccountId, collection_id: u64) -> DispatchResult {
            
            Ok(())
        }
        #[weight = 1000]
        pub fn create_item(origin, name: Vec<u8>, data: Vec<u8>, image: Vec<u8>, ext_link: Vec<u8>, description: Vec<u8>) -> DispatchResult {

            Ok(())
        }
        #[weight = 1000]
        pub fn burn_item(origin, collection_id: u64, item_id: u64) -> DispatchResult{

            Ok(())
        }
        #[weight = 1000]
        pub fn transfer(origin, to: T::AccountId, collection_id: u64, item_id: u64, amount: u128) -> DispatchResult{

            Ok(())
        }
        #[weight = 1000]
        pub fn approve(origin, approved: T::AccountId, collection_id: u64, item_id: u64, amount: u128) -> DispatchResult{

            Ok(())
        }
        #[weight = 1000]
        pub fn transfer_from(origin, from: T::AccountId, to: T::AccountId, collection_id: u64, item_id: u64, amount: u128) -> DispatchResult{

            Ok(())
        }
    }
}