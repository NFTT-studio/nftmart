#![cfg_attr(not(feature = "std"), no_std)]

use enumflags2::BitFlags;
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ReservableCurrency, ExistenceRequirement::KeepAlive},
	transactional, dispatch::DispatchResult
};
use sp_std::vec::Vec;
use frame_system::pallet_prelude::*;
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use sp_core::constants_types::{Balance, CurrencyId};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{
	traits::{CheckedAdd, Bounded,
			 AccountIdConversion, StaticLookup, Zero, One, AtLeast32BitUnsigned},
	ModuleId, RuntimeDebug, SaturatedConversion,
};
use codec::FullCodec;

mod mock;
mod tests;

pub use module::*;
use orml_nft::TokenInfoOf;

#[repr(u8)]
#[derive(Encode, Decode, Clone, Copy, BitFlags, RuntimeDebug, PartialEq, Eq)]
pub enum ClassProperty {
	/// Token can be transferred
	Transferable = 0b00000001,
	/// Token can be burned
	Burnable = 0b00000010,
}

#[derive(Clone, Copy, PartialEq, Default, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Properties(pub BitFlags<ClassProperty>);

impl Eq for Properties {}
impl Encode for Properties {
	fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		self.0.bits().using_encoded(f)
	}
}
impl Decode for Properties {
	fn decode<I: codec::Input>(input: &mut I) -> sp_std::result::Result<Self, codec::Error> {
		let field = u8::decode(input)?;
		Ok(Self(
			<BitFlags<ClassProperty>>::from_bits(field as u8).map_err(|_| "invalid value")?,
		))
	}
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ClassData {
	/// The minimum balance to create class
	#[codec(compact)]
	pub deposit: Balance,
	/// Property of all tokens in this class.
	pub properties: Properties,
	/// Name of class.
	pub name: Vec<u8>,
	/// Description of class.
	pub description: Vec<u8>,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct TokenData {
	/// The minimum balance to create token
	#[codec(compact)]
	pub deposit: Balance,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct CategoryData {
	/// The category metadata.
	pub metadata: NFTMetadata,
	/// The number of NFTs in this category.
	#[codec(compact)]
	pub nft_count: Balance,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct OrderData<T: Config> {
	/// currency ID.
	#[codec(compact)]
	pub currency_id: CurrencyId,
	/// Price of this order.
	#[codec(compact)]
	pub price: Balance,
	/// Category of this order.
	#[codec(compact)]
	pub category_id: CategoryIdOf<T>,
	/// Class ID of the NFT.
	#[codec(compact)]
	pub class_id: ClassIdOf<T>,
	/// Token ID of the NFT.
	#[codec(compact)]
	pub token_id: TokenIdOf<T>,
}

pub type NFTMetadata = Vec<u8>;
pub type TokenIdOf<T> = <T as orml_nft::Config>::TokenId;
pub type ClassIdOf<T> = <T as orml_nft::Config>::ClassId;
pub type CategoryIdOf<T> = <T as Config>::CategoryId;
pub type BalanceOf<T> = <<T as module::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type CurrencyIdOf<T> = <<T as module::Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + orml_nft::Config<ClassData = ClassData, TokenData = TokenData> + pallet_proxy::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The minimum balance to create class
		#[pallet::constant]
		type CreateClassDeposit: Get<Balance>;

		/// The amount of balance that must be deposited per byte of metadata.
		#[pallet::constant]
		type MetaDataByteDeposit: Get<Balance>;

		/// The minimum balance to create token
		#[pallet::constant]
		type CreateTokenDeposit: Get<Balance>;

		/// The NFT's module id
		#[pallet::constant]
		type ModuleId: Get<ModuleId>;

		/// MultiCurrency type for trading
		type MultiCurrency: MultiReservableCurrency<Self::AccountId, Balance = Balance>;

		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// The Category ID type
		type CategoryId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + Bounded + FullCodec;

		/// The Order ID type
		type OrderId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + Bounded + FullCodec;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// OrderId not found
		OrderIdNotFound,
		/// ClassId not found
		ClassIdNotFound,
		/// TokenId not found
		TokenIdNotFound,
		/// The operator is not the owner of the token and has no permission
		NoPermission,
		/// Quantity is invalid. need >= 1
		InvalidQuantity,
		/// Price is invalid. need > 0
		InvalidPrice,
		/// Property of class don't support transfer
		NonTransferable,
		/// Property of class don't support burn
		NonBurnable,
		/// Can not destroy class
		/// Total issuance is not 0
		CannotDestroyClass,
		/// No available category ID
		NoAvailableCategoryId,
		/// No available order ID
		NoAvailableOrderId,
		/// Order price too high.
		CanNotAfford,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Created NFT class. \[owner, class_id\]
		CreatedClass(T::AccountId, ClassIdOf<T>),
		/// Minted NFT token. \[from, to, class_id, quantity\]
		MintedToken(T::AccountId, T::AccountId, ClassIdOf<T>, u32),
		/// Transferred NFT token. \[from, to, class_id, token_id\]
		TransferredToken(T::AccountId, T::AccountId, ClassIdOf<T>, TokenIdOf<T>),
		/// Burned NFT token. \[owner, class_id, token_id\]
		BurnedToken(T::AccountId, ClassIdOf<T>, TokenIdOf<T>),
		/// Destroyed NFT class. \[owner, class_id, dest\]
		DestroyedClass(T::AccountId, ClassIdOf<T>, T::AccountId),
		/// Created NFT common category. \[category_id\]
		CreatedCategory(CategoryIdOf<T>),
		/// Created a NFT Order. \[order_id, category_id\]
		CreatedOrder(T::OrderId, CategoryIdOf<T>),
		/// Updated a NFT Order. \[order_id, category_id\]
		UpdatedOrder(T::OrderId, CategoryIdOf<T>),
		/// Removed a NFT Order. \[order_id\]
		RemovedOrder(T::OrderId),
		/// An order had been taken. \[order_id, price\]
		TakenOrder(T::OrderId, Balance),
	}

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	/// Next available common category ID.
	#[pallet::storage]
	#[pallet::getter(fn next_category_id)]
	pub type NextCategoryId<T: Config> = StorageValue<_, T::CategoryId, ValueQuery>;

	/// The storage of categories.
	#[pallet::storage]
	#[pallet::getter(fn category)]
	pub type Categories<T: Config> = StorageMap<_, Identity, T::CategoryId, CategoryData>;

	/// Next available order ID.
	#[pallet::storage]
	#[pallet::getter(fn next_order_id)]
	pub type NextOrderId<T: Config> = StorageValue<_, T::OrderId, ValueQuery>;

	/// The storage of orders.
	#[pallet::storage]
	#[pallet::getter(fn order)]
	pub type Orders<T: Config> = StorageMap<_, Identity, T::OrderId, OrderData<T>>;

	/// Create an index mapping from token to orderId.
	#[pallet::storage]
	#[pallet::getter(fn token2order)]
	pub type Token2Order<T: Config> = StorageMap<_, Blake2_128Concat, (ClassIdOf<T>,TokenIdOf<T>), T::OrderId>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Take an NFT order.
		///
		/// - `order_id`: order ID.
		/// - `max_price`: The max price to take an order. Usually it is set to the price of the target order.
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn take_order(
			origin: OriginFor<T>,
			#[pallet::compact] order_id: T::OrderId,
			#[pallet::compact] max_price: Balance,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let order = Orders::<T>::get(order_id).ok_or(Error::<T>::OrderIdNotFound)?;
			let token_info = orml_nft::Module::<T>::tokens(order.class_id, order.token_id).ok_or(Error::<T>::TokenIdNotFound)?;
			if who == token_info.owner {
				return Ok(().into());
			}
			ensure!(max_price >= order.price, Error::<T>::CanNotAfford);
			Self::do_transfer(&token_info.owner, &who, order.class_id, order.token_id, Some(&token_info))?;

			T::MultiCurrency::transfer(order.currency_id.saturated_into(), &who, &token_info.owner, order.price)?;
			Self::deposit_event(Event::TakenOrder(order_id, order.price));
			Ok(().into())
		}

		/// Create an NFT order. If the order already exists, it will update the order.
		/// Set the price to zero to delete the order.
		///
		/// - `currency_id`: currency id
		/// - `price`: price
		/// - `category_id`: category id
		/// - `class_id`: class id
		/// - `token_id`: token id
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn submit_order(
			origin: OriginFor<T>,
			#[pallet::compact] currency_id: CurrencyId,
			#[pallet::compact] price: Balance,
			#[pallet::compact] category_id: CategoryIdOf<T>,
			#[pallet::compact] class_id: ClassIdOf<T>,
			#[pallet::compact] token_id: TokenIdOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let token = (class_id, token_id);
			// Check ownership.
			ensure!(orml_nft::Module::<T>::is_owner(&who, token), Error::<T>::NoPermission);

			let (is_new, order_id) = match Token2Order::<T>::get(token) {
				Some(order_id) => {
					(false, order_id)
				},
				None => {
					ensure!(!price.is_zero(), Error::<T>::InvalidPrice);
					// Get and increase order ID.
					let id = NextOrderId::<T>::try_mutate(|id| -> Result<T::OrderId, DispatchError> {
						let current_id = *id;
						*id = id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailableOrderId)?;
						Ok(current_id)
					})?;
					(true, id)
				}
			};

			match (is_new, price.is_zero()) {
				(false, true) => {
					Self::do_remove_order(order_id, class_id, token_id);
				},
				(true, true) => {
					ensure!(false, Error::<T>::InvalidPrice);
				},
				(false, false) => {
					// update the order
					let order = OrderData {
						currency_id,
						price,
						category_id,
						class_id,
						token_id,
					};
					Orders::<T>::insert(order_id, order);
					Self::deposit_event(Event::UpdatedOrder(order_id, category_id));
				},
				(true, false) => {
					let order = OrderData {
						currency_id,
						price,
						category_id,
						class_id,
						token_id,
					};
					Orders::<T>::insert(order_id, order);
					Token2Order::<T>::insert(token, order_id);
					Self::deposit_event(Event::CreatedOrder(order_id, category_id));
				},
			}
			Ok(().into())
		}

		/// Create a common category for trading NFT.
		/// A Selling NFT should belong to a category.
		///
		/// - `metadata`: metadata
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn create_category(origin: OriginFor<T>, metadata: NFTMetadata) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			let category_id = NextCategoryId::<T>::try_mutate(|id| -> Result<T::CategoryId, DispatchError> {
				let current_id = *id;
				*id = id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailableCategoryId)?;
				Ok(current_id)
			})?;

			let info = CategoryData {
				metadata,
				nft_count: Default::default(),
			};
			Categories::<T>::insert(category_id, info);

			Self::deposit_event(Event::CreatedCategory(category_id));
			Ok(().into())
		}

		/// Create NFT class, tokens belong to the class.
		///
		/// - `metadata`: external metadata
		/// - `properties`: class property, include `Transferable` `Burnable`
		/// - `name`: class name, with len limitation.
		/// - `description`: class description, with len limitation.
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn create_class(origin: OriginFor<T>, metadata: NFTMetadata, name: Vec<u8>, description: Vec<u8>, properties: Properties) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// TODO: pass constants from runtime configuration.
			ensure!(name.len() <= 20, "Name too long.");
			ensure!(description.len() <= 256, "Description too long.");

			let next_id = orml_nft::Module::<T>::next_class_id();
			let owner: T::AccountId = T::ModuleId::get().into_sub_account(next_id);
			let (deposit, all_deposit) = Self::create_class_deposit(
				metadata.len().saturated_into(),
				name.len().saturated_into(),
				description.len().saturated_into(),
			);

			<T as Config>::Currency::transfer(&who, &owner, all_deposit.saturated_into(), KeepAlive)?;
			<T as Config>::Currency::reserve(&owner, deposit.saturated_into())?;
			// owner add proxy delegate to origin
			<pallet_proxy::Module<T>>::add_proxy_delegate(&owner, who, Default::default(), Zero::zero())?;

			let data = ClassData {
				deposit,
				properties,
				name,
				description,
			};
			orml_nft::Module::<T>::create_class(&owner, metadata, data)?;

			Self::deposit_event(Event::CreatedClass(owner, next_id));
			Ok(().into())
		}

		/// Mint NFT token
		///
		/// - `to`: the token owner's account
		/// - `class_id`: token belong to the class id
		/// - `metadata`: external metadata
		/// - `quantity`: token quantity
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn mint(
			origin: OriginFor<T>,
			to: <T::Lookup as StaticLookup>::Source,
			#[pallet::compact] class_id: ClassIdOf<T>,
			metadata: NFTMetadata,
			#[pallet::compact] quantity: u32,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let to = T::Lookup::lookup(to)?;
			ensure!(quantity >= 1, Error::<T>::InvalidQuantity);
			let class_info = orml_nft::Module::<T>::classes(class_id).ok_or(Error::<T>::ClassIdNotFound)?;
			ensure!(who == class_info.owner, Error::<T>::NoPermission);
			let (deposit, total_deposit) = Self::mint_token_deposit(metadata.len().saturated_into(), quantity);

			<T as Config>::Currency::reserve(&class_info.owner, total_deposit.saturated_into())?;
			let data = TokenData { deposit };
			for _ in 0..quantity {
				orml_nft::Module::<T>::mint(&to, class_id, metadata.clone(), data.clone())?;
			}

			Self::deposit_event(Event::MintedToken(who, to, class_id, quantity));
			Ok(().into())
		}

		/// Transfer NFT token to another account
		///
		/// - `to`: the token owner's account
		/// - `class_id`: class id
		/// - `token_id`: token id
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn transfer(
			origin: OriginFor<T>,
			to: <T::Lookup as StaticLookup>::Source,
			#[pallet::compact] class_id: ClassIdOf<T>,
			#[pallet::compact] token_id: TokenIdOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let to = T::Lookup::lookup(to)?;
			Self::do_transfer(&who, &to, class_id, token_id, None)?;
			Ok(().into())
		}

		/// Burn NFT token
		///
		/// - `class_id`: class id
		/// - `token_id`: token id
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn burn(
			origin: OriginFor<T>,
			#[pallet::compact] class_id: ClassIdOf<T>,
			#[pallet::compact] token_id: TokenIdOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let class_info = orml_nft::Module::<T>::classes(class_id).ok_or(Error::<T>::ClassIdNotFound)?;
			let data = class_info.data;
			ensure!(
				data.properties.0.contains(ClassProperty::Burnable),
				Error::<T>::NonBurnable
			);

			let token_info = orml_nft::Module::<T>::tokens(class_id, token_id).ok_or(Error::<T>::TokenIdNotFound)?;
			ensure!(who == token_info.owner, Error::<T>::NoPermission);

			orml_nft::Module::<T>::burn(&who, (class_id, token_id))?;
			let owner: T::AccountId = T::ModuleId::get().into_sub_account(class_id);
			let data = token_info.data;
			// `repatriate_reserved` will check `to` account exist and return `DeadAccount`.
			// `transfer` not do this check.
			<T as Config>::Currency::unreserve(&owner, data.deposit.saturated_into());
			<T as Config>::Currency::transfer(&owner, &who, data.deposit.saturated_into(), KeepAlive)?;

			Self::deposit_event(Event::BurnedToken(who, class_id, token_id));
			Ok(().into())
		}

		/// Destroy NFT class
		///
		/// - `class_id`: destroy class id
		/// - `dest`: transfer reserve balance from sub_account to dest
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn destroy_class(
			origin: OriginFor<T>,
			#[pallet::compact] class_id: ClassIdOf<T>,
			dest: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let dest = T::Lookup::lookup(dest)?;
			let class_info = orml_nft::Module::<T>::classes(class_id).ok_or(Error::<T>::ClassIdNotFound)?;
			ensure!(who == class_info.owner, Error::<T>::NoPermission);
			ensure!(
				class_info.total_issuance == Zero::zero(),
				Error::<T>::CannotDestroyClass
			);

			let owner: T::AccountId = T::ModuleId::get().into_sub_account(class_id);
			let data = class_info.data;
			// `repatriate_reserved` will check `to` account exist and return `DeadAccount`.
			// `transfer` not do this check.
			<T as Config>::Currency::unreserve(&owner, data.deposit.saturated_into());
			// At least there is one admin at this point.
			<T as Config>::Currency::transfer(&owner, &dest, data.deposit.saturated_into(), KeepAlive)?;

			// transfer all free from origin to dest
			orml_nft::Module::<T>::destroy_class(&who, class_id)?;

			Self::deposit_event(Event::DestroyedClass(who, class_id, dest));
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Ensured atomic.
	#[transactional]
	fn do_transfer(from: &T::AccountId, to: &T::AccountId, class_id: ClassIdOf<T>, token_id: TokenIdOf<T>, token_info: Option<&TokenInfoOf<T>>) -> DispatchResult {
		let class_info = orml_nft::Module::<T>::classes(class_id).ok_or(Error::<T>::ClassIdNotFound)?;
		let data = class_info.data;
		ensure!(
			data.properties.0.contains(ClassProperty::Transferable),
			Error::<T>::NonTransferable
		);

		let token_owner = match token_info {
			Some(token_info)=> token_info.owner.clone(),
			None => orml_nft::Module::<T>::tokens(class_id, token_id).ok_or(Error::<T>::TokenIdNotFound)?.owner
		};

		ensure!(*from == token_owner, Error::<T>::NoPermission);

		if from == to {
			return Ok(());
		}

		orml_nft::Module::<T>::transfer(from, to, (class_id, token_id))?;

		Self::do_remove_order_by_token(class_id, token_id);

		Self::deposit_event(Event::TransferredToken(from.clone(), to.clone(), class_id, token_id));
		Ok(())
	}

	/// Delete the nft order by class_id & token_id.
	fn do_remove_order_by_token(class_id: ClassIdOf<T>, token_id: TokenIdOf<T>) {
		if let Some(order_id) = Token2Order::<T>::get((class_id, token_id)) {
			Self::do_remove_order(order_id, class_id, token_id);
		}
	}

	/// remove an order
	fn do_remove_order(order_id: T::OrderId, class_id: ClassIdOf<T>, token_id: TokenIdOf<T>) {
		Orders::<T>::remove(order_id);
		Token2Order::<T>::remove((class_id, token_id));
		Self::deposit_event(Event::RemovedOrder(order_id));
	}

	pub fn add_class_admin_deposit(admin_count: u32) -> Balance {
		let proxy_deposit_before: Balance = <pallet_proxy::Module<T>>::deposit(1).saturated_into();
		let proxy_deposit_after: Balance = <pallet_proxy::Module<T>>::deposit(admin_count.saturating_add(1)).saturated_into();
		proxy_deposit_after.saturating_sub(proxy_deposit_before)
	}

	pub fn mint_token_deposit(metadata_len: u32, quantity: u32) -> (Balance, Balance) {
		let deposit: Balance = {
			let total_bytes = metadata_len;
			T::CreateTokenDeposit::get().saturating_add(
				(total_bytes as Balance).saturating_mul(T::MetaDataByteDeposit::get())
			)
		};
		let total_deposit: Balance = deposit.saturating_mul(quantity as Balance);
		(deposit, total_deposit)
	}

	pub fn create_class_deposit(metadata_len: u32, name_len: u32, description_len: u32) -> (Balance, Balance) {
		let deposit: Balance = {
			let total_bytes = metadata_len.saturating_add(name_len).saturating_add(description_len);
			T::CreateClassDeposit::get().saturating_add(
				(total_bytes as Balance).saturating_mul(T::MetaDataByteDeposit::get())
			)
		};
		let proxy_deposit: Balance = <pallet_proxy::Module<T>>::deposit(1).saturated_into();
		(deposit, deposit.saturating_add(proxy_deposit))
	}
}
