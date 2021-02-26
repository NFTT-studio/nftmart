#![cfg_attr(not(feature = "std"), no_std)]

use enumflags2::BitFlags;
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ReservableCurrency, ExistenceRequirement::KeepAlive},
	transactional,
};
use sp_std::vec::Vec;
use frame_system::pallet_prelude::*;
use orml_traits::{MultiReservableCurrency, NFT};
use sp_core::constants_types::Balance;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{
	traits::{AccountIdConversion, StaticLookup, Zero},
	DispatchResult, ModuleId, RuntimeDebug,
};

mod mock;

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
pub type Metadata = sp_std::vec::Vec<u8>;

#[frame_support::pallet]
pub mod module {
	use super::*;
	use sp_runtime::SaturatedConversion;
	use sp_runtime::traits::Saturating;

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
		#[pallet::weight(1000)]
		#[transactional]
		pub fn create_class(origin: OriginFor<T>, metadata: Metadata, name: Vec<u8>, description: Vec<u8>, properties: Properties) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// TODO: pass constants from runtime configuration.
			// ensure!(metadata.len() < ?, "xx");
			ensure!(name.len() < 20, "xx");
			ensure!(description.len() < 256, "xx");

			let next_id = orml_nft::Module::<T>::next_class_id();
			let owner: T::AccountId = T::ModuleId::get().into_sub_account(next_id);
			let total_bytes = metadata.len().saturating_add(name.len()).saturating_add(description.len());
			let deposit: Balance = T::CreateClassDeposit::get().saturating_add((total_bytes as Balance).saturating_mul(T::MetaDataByteDeposit::get()));

			<T as Config>::Currency::transfer(&who, &owner, deposit.saturated_into(), KeepAlive)?;
			<T as Config>::Currency::reserve(&owner, deposit.saturated_into())?;

			// owner add proxy delegate to origin
			let proxy_deposit = <pallet_proxy::Module<T>>::deposit(1);
			<T as pallet_proxy::Config>::Currency::transfer(&who, &owner, proxy_deposit, KeepAlive)?;
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
	}
}

impl<T: Config> NFT<T::AccountId> for Pallet<T> {
	type ClassId = ClassIdOf<T>;
	type TokenId = TokenIdOf<T>;
	type Balance = Balance;

	fn balance(who: &T::AccountId) -> Self::Balance {
		// orml_nft::TokensByOwner::<T>::iter_prefix(who).count() as u128
		todo!()
	}

	fn owner(token: (Self::ClassId, Self::TokenId)) -> Option<T::AccountId> {
		// orml_nft::Module::<T>::tokens(token.0, token.1).map(|t| t.owner)
		todo!()
	}

	fn transfer(from: &T::AccountId, to: &T::AccountId, token: (Self::ClassId, Self::TokenId)) -> DispatchResult {
		// Self::do_transfer(from, to, token)
		todo!()
	}
}


