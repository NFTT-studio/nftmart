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

/// NFT资产作品集定义
/// owner   作品集拥有者
/// name    作品集名称
/// icon    作品集 ICON 文件
/// description         作品集介绍
/// mode                作品集下的资产的类型
/// decimal_digits      小数位数（仅 mode = Separable(_,_) 时有效）
/// custom_data_size    作品集下的每个资产的元数据长度
/// 
/// 
#[derive(Encode, Decode, Debug, Default, Clone, PartialEq)]
pub struct CollectionType<AccountId> {
	pub owner: AccountId,
    pub name: Vec<u8>,
    pub icon: Vec<u8>,
    pub description: Vec<u8>,
    pub mode: NFTMode,
    pub decimal_digits: u32,
	pub custom_data_size: u32,
}

/// NFT 资产的类型
/// Inseparable 不可拆分的，每个人只能拥有完整的一个，不可拆分
/// Separable   可拆分的，每个人可以拥有一个资产的一部分
#[derive(Encode, Decode, Debug, Eq, Clone, PartialEq)]
pub enum NFTMode {
    Invalid,
    Inseparable(u32),
	Separable(u32, u32),
}
impl Into<u8> for NFTMode {
    fn into(self) -> u8{
        match self {
            NFTMode::Invalid => 0,
            NFTMode::Inseparable(_) => 1,
            NFTMode::Separable(_, _) => 2,
        }
    }
}
impl Default for NFTMode { fn default() -> Self { Self::Invalid } }

/// NFT 资产
/// creater     资产的创建者
/// name        资产的名称
/// data        资产的 metadata 信息
/// image       资产图片地址
/// ext_link    资产外部介绍链接
/// description 资产文字介绍
#[derive(Encode, Decode, Debug, Default, Clone, PartialEq)]
pub struct NTFItem<AccountId> {
    pub creater : AccountId,
    pub name:   Vec<u8>,
    pub data: Vec<u8>,
    pub image: Vec<u8>,
    pub ext_link: Vec<u8>,
    pub description: Vec<u8>,
}

pub trait Config: system::Config {
    type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;

    // 作品集相关的长度限定
    type MaxCollectionNameLength: Get<u32>;
    type MaxCollectionIconLength: Get<u32>;
    type MaxCollectionDescriptionLength: Get<u32>;
    type MaxCollectionCustomDataSize: Get<u32>;
    type MaxCollectionDecimalDigits: Get<u32>;
    // 作品相关的长度限定
    type MaxItemNameLength: Get<u32>;
    type MaxItemImageLength: Get<u32>;
    type MaxItemExtLinkLength: Get<u32>;
    type MaxItemDescriptionLength: Get<u32>;
}

decl_storage! {
    trait Store for Module<T: Config> as Nft {
        // 下一个作品集ID
        NextCollectionID: u64;
        // 各个作品集中，下一个作品的ID，key 为作品集ID，value 为下一个作品的ID
        NextItemID: map hasher(blake2_128_concat) u64 => u64;
        // 作品集数据，因为作品可以删除，部分作品集可能没有数据，所以需要用 Option
        pub CollectionList get(fn collection_list): map hasher(identity) u64 => Option<CollectionType<T::AccountId>>;
        // 作品集的管理员，管理员可以在该作品集下创建作品，管理员支持多个
        pub CollectionCreaterList get(fn collection_creater_list): map hasher(identity) u64 => Vec<T::AccountId>;
        // 作品数据，因为作品可以被删除，部分作品集可能没有数据，所以需要 Option
        // double_map ,key1 是 作品集ID，key2 是作品ID，值是作品信息
        pub NFTItemList get(fn nft_item_list): double_map hasher(blake2_128_concat) u64, hasher(blake2_128_concat) u64 => Option<NTFItem<T::AccountId>>;
        // 作品的拥有者列表 如果是可分割的 NFT 资产，一个资产可能由多人共同拥有
        // double_map ，key1 是集合ID，key2 是作品ID，值是 拥有者和拥有份额构成的 tuple 组成的 Vec
        pub NFTOwnerList get(fn nft_owner_list): double_map hasher(blake2_128_concat) u64, hasher(blake2_128_concat) u64 => Vec<(T::AccountId, u128)>;
        // 账号拥有的作品列表
        // douple_map key1 是拥有者账号，key2 是拥有的资产（由 作品集ID 和 作品ID 组成的 tuple )，值是拥有的数量，不可拆分固定是1，可拆分的则是一个大数字
        pub OwnedNFTList get(fn owned_nft_list): double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) (u64, u64) => u128;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Config>::AccountId,
    {
        // 作品集创建，参数：(1) 创建者, (2) 集合ID
        CollectionCreated(AccountId, u64),
        // 作品创建，参数：（1）创建者，（2）集合ID；（3）作品ID
        ItemCreated(AccountId, u64, u64), 
         // 作品销毁，参数为：(1) 作品集合ID；（2）作品ID
        ItemDestroyed(u64, u64),
    }
);

// 定义错误信息
decl_error! {
	pub enum Error for Module<T: Config> {
        CollectionNameTooLong,
        CollectionIconTooLong,
        CollectionDescriptionTooLong,
        CollectionCustomDataSizeTooBig,
        CollectionDecimalDigitsTooBig,
	}
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;

        /// 创建集合
        #[weight = 1000]
        pub fn create_collection(origin, name: Vec<u8>, icon: Vec<u8>, description: Vec<u8>, mode: NFTMode) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!( name.len() as u32 <= T::MaxCollectionNameLength::get() , Error::<T>::CollectionNameTooLong);
            ensure!( icon.len() as u32 <= T::MaxCollectionIconLength::get() , Error::<T>::CollectionIconTooLong);
            ensure!( description.len() as u32 <= T::MaxCollectionDescriptionLength::get() , Error::<T>::CollectionDescriptionTooLong);
            
            let custom_data_size = match mode {
                NFTMode::Inseparable(size) => size,
                NFTMode::Separable(size,_) => size,
                _ => 0,
            };
            ensure!( custom_data_size <= T::MaxCollectionCustomDataSize::get() , Error::<T>::CollectionCustomDataSizeTooBig);
            let decimal_digits = match mode {
                NFTMode::Separable(_, decimal) => decimal,
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
        // 转让作品集拥有者
        #[weight=0]
        pub fn transfer_collection(origin, to: T::AccountId, collection_id: u64) -> DispatchResult {

            Ok(())
        }
        // 删除作品集
        #[weight=0]
        pub fn destroy_collection(origin, collection_id: u64) -> DispatchResult {
            
            Ok(())
        }
        // 设置管理员
        #[weight=0]
        pub fn set_collection_creater(origin, creater: T::AccountId, collection_id: u64) -> DispatchResult {
            
            Ok(())
        }
        // 移除管理员
        #[weight = 1000]
        pub fn remove_collection_creater(origin, creater: T::AccountId, collection_id: u64) -> DispatchResult {
            
            Ok(())
        }
        // 创建作品
        #[weight = 1000]
        pub fn create_item(origin, name: Vec<u8>, data: Vec<u8>, image: Vec<u8>, ext_link: Vec<u8>, description: Vec<u8>) -> DispatchResult {

            Ok(())
        }
        // 销毁作品
        #[weight = 1000]
        pub fn burn_item(origin, collection_id: u64, item_id: u64) -> DispatchResult{

            Ok(())
        }
        // 作品转让/转账
        #[weight = 1000]
        pub fn transfer(origin, to: T::AccountId, collection_id: u64, item_id: u64, amount: u128) -> DispatchResult{

            Ok(())
        }
        // 设置作品转让/转账授权
        #[weight = 1000]
        pub fn approve(origin, approved: T::AccountId, collection_id: u64, item_id: u64, amount: u128) -> DispatchResult{

            Ok(())
        }
        // 基于授权就行作品转让/转账
        #[weight = 1000]
        pub fn transfer_from(origin, from: T::AccountId, to: T::AccountId, collection_id: u64, item_id: u64, amount: u128) -> DispatchResult{

            Ok(())
        }
    }
}