#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ReservableCurrency, ExistenceRequirement::KeepAlive},
	transactional, dispatch::DispatchResult, PalletId,
};
use sp_std::vec::Vec;
use frame_system::pallet_prelude::*;
pub use sp_core::constants_types::{Balance, ACCURACY, NATIVE_CURRENCY_ID};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{
	traits::{CheckedAdd, Bounded, CheckedSub,
			 AccountIdConversion, StaticLookup, Zero, One, AtLeast32BitUnsigned},
	RuntimeDebug, SaturatedConversion,
};
use codec::FullCodec;
use orml_traits::{MultiCurrency, MultiReservableCurrency};

mod mock;
mod tests;

pub use module::*;

// #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, RuntimeDebug)]
// pub enum OrderKind {
// 	Normal,
// 	Offer,
// 	British,
// 	Dutch,
// }

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
	/// True, if the order was submitted by the token owner.
	pub by_token_owner: bool,
	/// The quantity of token.
	#[codec(compact)]
	pub quantity: TokenIdOf<T>,
}

#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug)]
enum Releases {
	V1_0_0,
}

impl Default for Releases {
	fn default() -> Self {
		Releases::V1_0_0
	}
}

pub type TokenIdOf<T> = <T as module::Config>::TokenId;
pub type ClassIdOf<T> = <T as module::Config>::ClassId;
pub type CategoryIdOf<T> = <T as module::Config>::CategoryId;
pub type BalanceOf<T> = <<T as module::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type CurrencyIdOf<T> = <<T as module::Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// MultiCurrency type for trading
		type MultiCurrency: MultiReservableCurrency<Self::AccountId, Balance = Balance>;

		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// The Category ID type
		type CategoryId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + Bounded + FullCodec;

		/// The class ID type
		type ClassId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + codec::FullCodec;

		/// The token ID type
		type TokenId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + codec::FullCodec;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// ClassId not found
		ClassIdNotFound,
		/// TokenId not found
		TokenIdNotFound,
		/// Order not found
		OrderNotFound,
		/// Category not found
		CategoryNotFound,
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
		/// Can not destroy class Total issuance is not 0
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
		/// NameTooLong
		NameTooLong,
		/// DescriptionTooLong
		DescriptionTooLong,
		/// Not supported for now
		NotSupportedForNow,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// OrderMinDeposit updated \[old, new\]
		UpdatedMinOrderDeposit(Balance, Balance),
	}

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_runtime_upgrade() -> Weight {
			0
		}

		fn integrity_test () {}
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		_phantom: PhantomData<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				_phantom: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			<StorageVersion<T>>::put(Releases::default());
		}
	}

	/// Storage version of the pallet.
	#[pallet::storage]
	pub(super) type StorageVersion<T: Config> = StorageValue<_, Releases, ValueQuery>;

	// /// An index mapping from token to order.
	// #[pallet::storage]
	// #[pallet::getter(fn orders)]
	// pub type Orders<T: Config> = StorageDoubleMap<_, Blake2_128Concat, (ClassIdOf<T>, TokenIdOf<T>), Blake2_128Concat, T::AccountId, OrderData<T>>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// /// Take an NFT order.
		// ///
		// /// - `class_id`: class id
		// /// - `token_id`: token id
		// /// - `price`: The max/min price to take an order. Usually it is set to the price of the target order.
		// #[pallet::weight(100_000)]
		// #[transactional]
		// pub fn take_order(
		// 	origin: OriginFor<T>,
		// 	#[pallet::compact] class_id: ClassIdOf<T>,
		// 	#[pallet::compact] token_id: TokenIdOf<T>,
		// 	#[pallet::compact] price: Balance,
		// 	order_owner: <T::Lookup as StaticLookup>::Source,
		// ) -> DispatchResultWithPostInfo {
		// 	let who = ensure_signed(origin)?;
		// 	let order_owner = T::Lookup::lookup(order_owner)?;
		// 	// Simplify the logic, to make life easier.
		// 	ensure!(order_owner != who, Error::<T>::TakeOwnOrder);
		// 	let token_owner = orml_nft::Pallet::<T>::tokens(class_id, token_id).ok_or(Error::<T>::TokenIdNotFound)?.owner;
		//
		// 	let order: OrderData<T> = {
		// 		let order = Self::orders((class_id, token_id), &order_owner);
		// 		ensure!(order.is_some(), Error::<T>::OrderNotFound);
		// 		let order = order.unwrap();
		// 		ensure!(<frame_system::Pallet<T>>::block_number() <= order.deadline, Error::<T>::OrderExpired);
		// 		order
		// 	};
		//
		// 	match (order_owner == token_owner, token_owner == who) {
		// 		(true, false) => {
		// 			ensure!(price >= order.price, Error::<T>::CanNotAfford);
		// 			// `who` will take the order submitting by `order_owner`/`token_owner`
		// 			Self::delete_order(class_id, token_id, &order_owner)?;
		// 			// Try to delete another order for safety.
		// 			// Because `who` may have already submitted an order to the same token.
		// 			Self::try_delete_order(class_id, token_id, &who);
		// 			// `order_owner` transfers this NFT to `who`
		// 			Self::do_transfer(&order_owner, &who, class_id, token_id)?;
		// 			T::MultiCurrency::transfer(order.currency_id, &who, &order_owner, order.price)?;
		// 			// TODO: T::MultiCurrency::transfer(order.currency_id, &order_owner, some_account,platform-fee)?;
		// 			Self::deposit_event(Event::TakenOrder(class_id, token_id, order_owner));
		// 		},
		// 		(false, true) => {
		// 			ensure!(price <= order.price, Error::<T>::PriceTooLow);
		// 			// `who`/`token_owner` will accept the order submitted by `order_owner`
		// 			Self::delete_order(class_id, token_id, &order_owner)?;
		// 			Self::try_delete_order(class_id, token_id, &who);
		// 			// `order_owner` transfers this NFT to `who`
		// 			Self::do_transfer(&who, &order_owner, class_id, token_id)?;
		// 			T::MultiCurrency::transfer(order.currency_id, &order_owner, &who, order.price)?;
		// 			// TODO: T::MultiCurrency::transfer(order.currency_id, &who, some_account,platform-fee)?;
		// 			Self::deposit_event(Event::TakenOrder(class_id, token_id, order_owner));
		// 		},
		// 		_ => {
		// 			return Err(Error::<T>::NoPermission.into());
		// 		},
		// 	}
		// 	Ok(().into())
		// }

		// /// Create an NFT order. Create only.
		// ///
		// /// - `currency_id`: currency id
		// /// - `price`: price
		// /// - `category_id`: category id
		// /// - `class_id`: class id
		// /// - `token_id`: token id
		// /// - `deposit`: The balances to create an order
		// /// - `deadline`: deadline
		// #[pallet::weight(100_000)]
		// #[transactional]
		// pub fn submit_order(
		// 	origin: OriginFor<T>,
		// 	#[pallet::compact] currency_id: CurrencyIdOf<T>,
		// 	#[pallet::compact] price: Balance,
		// 	#[pallet::compact] category_id: CategoryIdOf<T>,
		// 	#[pallet::compact] class_id: ClassIdOf<T>,
		// 	#[pallet::compact] token_id: TokenIdOf<T>,
		// 	#[pallet::compact] deposit: Balance,
		// 	#[pallet::compact] deadline: BlockNumberOf<T>,
		// ) -> DispatchResultWithPostInfo {
		// 	let who = ensure_signed(origin)?;
		//
		// 	// TODO: Get rid of this limitation.
		// 	ensure!(currency_id == NATIVE_CURRENCY_ID.saturated_into(), nftmart_nft::Error::<T>::NotSupportedForNow);
		//
		// 	let token_info: TokenInfoOf<T> = orml_nft::Pallet::<T>::tokens(class_id, token_id).ok_or(nftmart_nft::Error::<T>::TokenIdNotFound)?;
		// 	// TODO: Get rid of this limitation.
		// 	ensure!(token_info.quantity == One::one(), Error::<T>::NotSupportedForNow);
		//
		// 	ensure!(Self::orders((class_id, token_id), &who).is_none(), Error::<T>::DuplicatedOrder);
		// 	// ensure!(<frame_system::Pallet<T>>::block_number() < deadline, Error::<T>::InvalidDeadline);
		// 	// Categories::<T>::try_mutate(category_id, |maybe_category| -> DispatchResult {
		// 	// 	let category = maybe_category.as_mut().ok_or(Error::<T>::CategoryNotFound)?;
		// 	// 	category.nft_count = category.nft_count.saturating_add(One::one());
		// 	// 	Ok(())
		// 	// })?;
		// 	//
		// 	// ensure!(deposit >= Self::min_order_deposit(), Error::<T>::InvalidDeposit);
		// 	// // Reserve native currency.
		// 	// <T as Config>::Currency::reserve(&who, deposit.saturated_into())?;
		// 	//
		// 	// if token.owner != who {
		// 	// 	// Reserve specified currency.
		// 	// 	T::MultiCurrency::reserve(currency_id, &who, price.saturated_into())?;
		// 	// }
		// 	//
		// 	// let order: OrderData<T> = OrderData {
		// 	// 	currency_id,
		// 	// 	price,
		// 	// 	deposit,
		// 	// 	deadline,
		// 	// 	category_id,
		// 	// 	by_token_owner: token.owner == who,
		// 	// };
		// 	// Orders::<T>::insert((class_id, token_id), &who, order);
		// 	//
		// 	// Self::deposit_event(Event::CreatedOrder(class_id, token_id, who));
		// 	Ok(().into())
		// }

		// /// remove an order by order owner.
		// ///
		// /// - `class_id`: class id
		// /// - `token_id`: token id
		// #[pallet::weight(100_000)]
		// #[transactional]
		// pub fn remove_order(
		// 	origin: OriginFor<T>,
		// 	#[pallet::compact] class_id: ClassIdOf<T>,
		// 	#[pallet::compact] token_id: TokenIdOf<T>,
		// ) -> DispatchResultWithPostInfo {
		// 	let who = ensure_signed(origin)?;
		// 	Self::delete_order(class_id, token_id, &who)?;
		// 	Ok(().into())
		// }
	}
}

impl<T: Config> Pallet<T> {
	// fn delete_order(class_id: ClassIdOf<T>, token_id: TokenIdOf<T>, who: &T::AccountId) -> DispatchResult {
	// 	Orders::<T>::try_mutate_exists((class_id, token_id), who, |maybe_order| {
	// 		let order = maybe_order.as_mut().ok_or(Error::<T>::OrderNotFound)?;
	//
	// 		let mut deposit: Balance = Zero::zero();
	// 		if !order.by_token_owner {
	// 			// todo: emit an event for `order.currency_id`.
	// 			let d = T::MultiCurrency::unreserve(order.currency_id, &who, order.price.saturated_into());
	// 			deposit = deposit.saturating_add(order.price).saturating_sub(d);
	// 		}
	//
	// 		Categories::<T>::try_mutate(order.category_id, |category| -> DispatchResult {
	// 			category.as_mut().map(|cate| cate.nft_count = cate.nft_count.saturating_sub(One::one()) );
	// 			Ok(())
	// 		})?;
	//
	// 		let deposit = {
	// 			let d = <T as Config>::Currency::unreserve(&who, order.deposit.saturated_into());
	// 			deposit.saturating_add(order.deposit).saturating_sub(d.saturated_into())
	// 		};
	// 		Self::deposit_event(Event::RemovedOrder(class_id, token_id, who.clone(), deposit.saturated_into()));
	// 		*maybe_order = None;
	// 		Ok(())
	// 	})
	// }

	// fn try_delete_order(class_id: ClassIdOf<T>, token_id: TokenIdOf<T>, who: &T::AccountId) {
	// 	let _ = Self::delete_order(class_id, token_id, who);
	// }
}
