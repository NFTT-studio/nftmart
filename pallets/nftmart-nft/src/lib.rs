#![cfg_attr(not(feature = "std"), no_std)]

use enumflags2::BitFlags;
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ReservableCurrency, ExistenceRequirement::KeepAlive},
	transactional, dispatch::DispatchResult
};
use sp_std::vec::Vec;
use frame_system::pallet_prelude::*;
use orml_traits::{MultiReservableCurrency};
use sp_core::constants_types::Balance;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{
	traits::{AccountIdConversion, StaticLookup, Zero},
	ModuleId, RuntimeDebug,
};
use sp_runtime::SaturatedConversion;

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
	pub deposit: Balance,
}

pub type TokenIdOf<T> = <T as orml_nft::Config>::TokenId;
pub type ClassIdOf<T> = <T as orml_nft::Config>::ClassId;
pub type BalanceOf<T> = <<T as module::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type NFTMetadata = Vec<u8>;

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
	}

	#[pallet::error]
	pub enum Error<T> {
		/// ClassId not found
		ClassIdNotFound,
		/// TokenId not found
		TokenIdNotFound,
		/// The operator is not the owner of the token and has no permission
		NoPermission,
		/// Quantity is invalid. need >= 1
		InvalidQuantity,
		/// Property of class don't support transfer
		NonTransferable,
		/// Property of class don't support burn
		NonBurnable,
		/// Can not destroy class
		/// Total issuance is not 0
		CannotDestroyClass,
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
	}

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
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
			let (deposit, all_deposit) = Self::create_class_deposit(&metadata, &name, &description);

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
			class_id: ClassIdOf<T>,
			metadata: NFTMetadata,
			quantity: u32,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let to = T::Lookup::lookup(to)?;
			ensure!(quantity >= 1, Error::<T>::InvalidQuantity);
			let class_info = orml_nft::Module::<T>::classes(class_id).ok_or(Error::<T>::ClassIdNotFound)?;
			ensure!(who == class_info.owner, Error::<T>::NoPermission);
			let (deposit, total_deposit) = Self::create_token_deposit(&metadata, quantity);

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
		/// - `token`: (class_id, token_id)
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn transfer(
			origin: OriginFor<T>,
			to: <T::Lookup as StaticLookup>::Source,
			token: (ClassIdOf<T>, TokenIdOf<T>),
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let to = T::Lookup::lookup(to)?;
			Self::do_transfer(&who, &to, token)?;
			Ok(().into())
		}

		/// Burn NFT token
		///
		/// - `token`: (class_id, token_id)
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn burn(origin: OriginFor<T>, token: (ClassIdOf<T>, TokenIdOf<T>)) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let class_info = orml_nft::Module::<T>::classes(token.0).ok_or(Error::<T>::ClassIdNotFound)?;
			let data = class_info.data;
			ensure!(
				data.properties.0.contains(ClassProperty::Burnable),
				Error::<T>::NonBurnable
			);

			let token_info = orml_nft::Module::<T>::tokens(token.0, token.1).ok_or(Error::<T>::TokenIdNotFound)?;
			ensure!(who == token_info.owner, Error::<T>::NoPermission);

			orml_nft::Module::<T>::burn(&who, token)?;
			let owner: T::AccountId = T::ModuleId::get().into_sub_account(token.0);
			let data = token_info.data;
			// `repatriate_reserved` will check `to` account exist and return `DeadAccount`.
			// `transfer` not do this check.
			<T as Config>::Currency::unreserve(&owner, data.deposit.saturated_into());
			<T as Config>::Currency::transfer(&owner, &who, data.deposit.saturated_into(), KeepAlive)?;

			Self::deposit_event(Event::BurnedToken(who, token.0, token.1));
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
			class_id: ClassIdOf<T>,
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
	fn do_transfer(from: &T::AccountId, to: &T::AccountId, token: (ClassIdOf<T>, TokenIdOf<T>)) -> DispatchResult {
		let class_info = orml_nft::Module::<T>::classes(token.0).ok_or(Error::<T>::ClassIdNotFound)?;
		let data = class_info.data;
		ensure!(
			data.properties.0.contains(ClassProperty::Transferable),
			Error::<T>::NonTransferable
		);

		let token_info = orml_nft::Module::<T>::tokens(token.0, token.1).ok_or(Error::<T>::TokenIdNotFound)?;
		ensure!(*from == token_info.owner, Error::<T>::NoPermission);

		orml_nft::Module::<T>::transfer(from, to, token)?;

		Self::deposit_event(Event::TransferredToken(from.clone(), to.clone(), token.0, token.1));
		Ok(())
	}

	pub fn add_class_admin_deposit(admin_count: u32) -> Balance {
		let proxy_deposit_before: Balance = <pallet_proxy::Module<T>>::deposit(1).saturated_into();
		let proxy_deposit_after: Balance = <pallet_proxy::Module<T>>::deposit(admin_count.saturating_add(1)).saturated_into();
		proxy_deposit_after.saturating_sub(proxy_deposit_before)
	}

	pub fn create_token_deposit(metadata: &[u8], quantity: u32) -> (Balance, Balance) {
		let deposit: Balance = {
			let total_bytes = metadata.len();
			T::CreateTokenDeposit::get().saturating_add(
				(total_bytes as Balance).saturating_mul(T::MetaDataByteDeposit::get())
			)
		};
		let total_deposit: Balance = deposit.saturating_mul(quantity as Balance);
		(deposit, total_deposit)
	}

	pub fn create_class_deposit(metadata: &[u8], name: &[u8], description: &[u8]) -> (Balance, Balance) {
		let deposit: Balance = {
			let total_bytes = metadata.len().saturating_add(name.len()).saturating_add(description.len());
			T::CreateClassDeposit::get().saturating_add(
				(total_bytes as Balance).saturating_mul(T::MetaDataByteDeposit::get())
			)
		};
		let proxy_deposit: Balance = <pallet_proxy::Module<T>>::deposit(1).saturated_into();
		(deposit, deposit.saturating_add(proxy_deposit))
	}
}
