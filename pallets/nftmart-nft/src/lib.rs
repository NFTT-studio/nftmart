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
use sp_core::constants_types::Balance;
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
	pub currency_id: CurrencyIdOf<T>,
	/// Price of this order.
	#[codec(compact)]
	pub price: Balance,
	/// The balances to create an order
	#[codec(compact)]
	pub deposit: Balance,
	/// This order will be invalidated after `deadline` block number.
	#[codec(compact)]
	pub deadline: BlockNumberOf<T>,
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
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;

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
	}

	#[pallet::error]
	pub enum Error<T> {
		/// ClassId not found
		ClassIdNotFound,
		/// TokenId not found
		TokenIdNotFound,
		/// Order not found
		OrderNotFound,
		/// The operator is not the owner of the token and has no permission
		NoPermission,
		/// Quantity is invalid. need >= 1
		InvalidQuantity,
		/// Invalid deadline
		InvalidDeadline,
		/// Invalid deposit
		InvalidDeposit,
		/// Property of class don't support transfer
		NonTransferable,
		/// Property of class don't support burn
		NonBurnable,
		/// Can not destroy class
		/// Total issuance is not 0
		CannotDestroyClass,
		/// No available category ID
		NoAvailableCategoryId,
		/// Order price too high.
		CanNotAfford,
		/// Price too low to accept.
		PriceTooLow,
		/// Duplicated order.
		DuplicatedOrder,
		/// Not allow to take own order.
		TakeOwnOrder,
		/// Cannot transfer NFT while order existing.
		OrderExists,
		/// Order expired
		OrderExpired,
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
		/// Created a NFT Order. \[class_id, token_id, order_owner\]
		CreatedOrder(ClassIdOf<T>, TokenIdOf<T>, T::AccountId),
		/// Removed a NFT Order. \[class_id, token_id, order_owner, unreserved\]
		RemovedOrder(ClassIdOf<T>, TokenIdOf<T>, T::AccountId, Balance),
		/// An order had been taken. \[class_id, token_id, order_owner\]
		TakenOrder(ClassIdOf<T>, TokenIdOf<T>, T::AccountId),
		/// Price updated \[class_id, token_id, order_owner, price\]
		UpdatedOrderPrice(ClassIdOf<T>, TokenIdOf<T>, T::AccountId, Balance),
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

	/// An index mapping from token to order.
	#[pallet::storage]
	#[pallet::getter(fn order)]
	pub type Order<T: Config> = StorageDoubleMap<_, Blake2_128Concat, (ClassIdOf<T>,TokenIdOf<T>), Blake2_128Concat, T::AccountId, OrderData<T>>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Take an NFT order.
		///
		/// - `class_id`: class id
		/// - `token_id`: token id
		/// - `price`: The max/min price to take an order. Usually it is set to the price of the target order.
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn take_order(
			origin: OriginFor<T>,
			#[pallet::compact] class_id: ClassIdOf<T>,
			#[pallet::compact] token_id: TokenIdOf<T>,
			#[pallet::compact] price: Balance,
			order_owner: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(order_owner != who, Error::<T>::TakeOwnOrder);

			let order: OrderData<T> = {
				let order = Self::order((class_id, token_id), &order_owner);
				ensure!(order.is_some(), Error::<T>::OrderNotFound);
				order.unwrap()
			};

			ensure!(<frame_system::Pallet<T>>::block_number() <= order.deadline, Error::<T>::OrderExpired);

			let token_info = orml_nft::Module::<T>::tokens(class_id, token_id).ok_or(Error::<T>::TokenIdNotFound)?;
			match (order_owner == token_info.owner, token_info.owner == who) {
				(true, false) => {
					ensure!(price >= order.price, Error::<T>::CanNotAfford);
					// `who` will take the order submitting by `order_owner`/`token_info.owner`
					Self::delete_order(class_id, token_id, &order_owner);
					Self::delete_order(class_id, token_id, &who);
					// `order_owner` transfers this NFT to `who`
					Self::do_transfer(&order_owner, &who, order.class_id, order.token_id)?;
					T::MultiCurrency::transfer(order.currency_id, &who, &order_owner, order.price)?;
					// TODO: T::MultiCurrency::transfer(order.currency_id, &order_owner, some_account,platform-fee)?;
					Self::deposit_event(Event::TakenOrder(class_id, token_id, order_owner));
				},
				(false, true) => {
					ensure!(price <= order.price, Error::<T>::PriceTooLow);
					// `who`/`token_info.owner` will accept the order submitting by `order_owner`
					Self::delete_order(class_id, token_id, &order_owner);
					Self::delete_order(class_id, token_id, &who);
					// `order_owner` transfers this NFT to `who`
					Self::do_transfer(&who, &order_owner, order.class_id, order.token_id)?;
					T::MultiCurrency::transfer(order.currency_id, &order_owner, &who, order.price)?;
					// TODO: T::MultiCurrency::transfer(order.currency_id, &who, some_account,platform-fee)?;
					Self::deposit_event(Event::TakenOrder(class_id, token_id, order_owner));
				},
				_ => {
					return Err(Error::<T>::NoPermission.into());
				},
			}
			Ok(().into())
		}

		/// Create an NFT order. Create only.
		///
		/// - `currency_id`: currency id
		/// - `price`: price
		/// - `category_id`: category id
		/// - `class_id`: class id
		/// - `token_id`: token id
		/// - `deposit`: The balances to create an order
		/// - `deadline`: deadline
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn submit_order(
			origin: OriginFor<T>,
			#[pallet::compact] currency_id: CurrencyIdOf<T>,
			#[pallet::compact] price: Balance,
			#[pallet::compact] category_id: CategoryIdOf<T>,
			#[pallet::compact] class_id: ClassIdOf<T>,
			#[pallet::compact] token_id: TokenIdOf<T>,
			#[pallet::compact] deposit: Balance,
			#[pallet::compact] deadline: BlockNumberOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let _ = orml_nft::Module::<T>::tokens(class_id, token_id).ok_or(Error::<T>::TokenIdNotFound)?;
			// ensure!(orml_nft::Module::<T>::is_owner(&who, (class_id, token_id)), Error::<T>::NoPermission);

			ensure!(Self::order((class_id, token_id), &who).is_none(), Error::<T>::DuplicatedOrder);
			ensure!(<frame_system::Pallet<T>>::block_number() < deadline, Error::<T>::InvalidDeadline);

			// TODO: make it configurable.
			ensure!(deposit > 10000u128, Error::<T>::InvalidDeposit);
			<T as Config>::Currency::reserve(&who, deposit.saturated_into())?;

			let order: OrderData<T> = OrderData {
				currency_id,
				price,
				deposit,
				deadline,
				category_id,
				class_id,
				token_id,
			};
			Order::<T>::insert((class_id, token_id), &who, order);

			// TODO: update category.
			// Categories::<T>::try_mu

			Self::deposit_event(Event::CreatedOrder(class_id, token_id, who));
			Ok(().into())
		}

		/// remove an order by order owner.
		///
		/// - `class_id`: class id
		/// - `token_id`: token id
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn remove_order(
			origin: OriginFor<T>,
			#[pallet::compact] class_id: ClassIdOf<T>,
			#[pallet::compact] token_id: TokenIdOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			Self::delete_order(class_id, token_id, &who);
			Ok(().into())
		}

		/// Update order price
		///
		/// - `class_id`: class id
		/// - `token_id`: token id
		/// - `price`: price
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn update_order_price(
			origin: OriginFor<T>,
			#[pallet::compact] class_id: ClassIdOf<T>,
			#[pallet::compact] token_id: TokenIdOf<T>,
			#[pallet::compact] price: Balance,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			if let Some(mut order) = Self::order((class_id, token_id), &who) {
				order.price = price;
				Order::<T>::insert((class_id, token_id), &who, order);
				Self::deposit_event(Event::UpdatedOrderPrice(class_id, token_id, who, price));
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
			Self::do_transfer(&who, &to, class_id, token_id)?;
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

			ensure!(Self::order((class_id, token_id), &who).is_none(), Error::<T>::OrderExists);
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

	fn delete_order(class_id: ClassIdOf<T>, token_id: TokenIdOf<T>, who: &T::AccountId) {
		if let Some(order) = Self::order((class_id, token_id), who) {
			let deposit = order.deposit;
			let deposit = <T as Config>::Currency::unreserve(&who, deposit.saturated_into());
			Order::<T>::remove((class_id, token_id), who);
			Self::deposit_event(Event::RemovedOrder(class_id, token_id, who.clone(), deposit.saturated_into()));

			// TODO: update category.
		}
	}

	/// Ensured atomic.
	#[transactional]
	fn do_transfer(from: &T::AccountId, to: &T::AccountId, class_id: ClassIdOf<T>, token_id: TokenIdOf<T>) -> DispatchResult {
		let class_info = orml_nft::Module::<T>::classes(class_id).ok_or(Error::<T>::ClassIdNotFound)?;
		let data = class_info.data;
		ensure!(
			data.properties.0.contains(ClassProperty::Transferable),
			Error::<T>::NonTransferable
		);

		let token_info = orml_nft::Module::<T>::tokens(class_id, token_id).ok_or(Error::<T>::TokenIdNotFound)?;
		ensure!(*from == token_info.owner, Error::<T>::NoPermission);

		ensure!(Self::order((class_id, token_id), from).is_none(), Error::<T>::OrderExists);

		orml_nft::Module::<T>::transfer(from, to, (class_id, token_id))?;

		Self::deposit_event(Event::TransferredToken(from.clone(), to.clone(), class_id, token_id));
		Ok(())
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
