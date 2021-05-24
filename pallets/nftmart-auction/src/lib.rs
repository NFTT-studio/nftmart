#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ReservableCurrency},
	transactional,
};
use sp_std::vec::Vec;
use frame_system::pallet_prelude::*;
pub use sp_core::constants_types::{GlobalId, Balance, ACCURACY, NATIVE_CURRENCY_ID};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, StaticLookup},
	RuntimeDebug, SaturatedConversion, PerU16,
};
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use nftmart_traits::{NftmartConfig, NftmartNft, OrderItem};

mod mock;
mod tests;

pub use module::*;

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct BritishAuction<AccountId, CurrencyId, BlockNumber, CategoryId, ClassId, TokenId> {
	/// currency ID for this auction
	#[codec(compact)]
	pub currency_id: CurrencyId,
	/// If encountered this price, the auction should be finished.
	pub fixed_price: Option<Balance>,
	/// The new price offered should meet `new_price>=old_price*(1+min_raise)`
	/// if Some(min_raise), min_raise > 0.
	pub min_raise: Option<PerU16>,
	/// The auction owner/creator should deposit some balances to create an auction.
	/// After this auction finishing or deleting, this balances
	/// will be returned to the auction owner.
	#[codec(compact)]
	pub deposit: Balance,
	/// The initialized price of `currency_id` for auction.
	#[codec(compact)]
	pub init_price: Balance,
	/// the newest price offered by
	#[codec(compact)]
	pub last_price: Balance,
	/// the last account offering.
	pub last_offer: Option<AccountId>,
	/// The auction should be forced to be ended if current block number higher than this value.
	pub deadline: Option<BlockNumber>,
	/// if false, `deadline` will be updated by new offering.
	/// if ture, `deadline` will be set by creating this auction.
	pub set_deadline: bool,
	/// Category of this auction.
	#[codec(compact)]
	pub category_id: CategoryId,
	/// nft list
	pub items: Vec<OrderItem<ClassId, TokenId>>,
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

		/// The class ID type
		type ClassId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + codec::FullCodec;

		/// The token ID type
		type TokenId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + codec::FullCodec;

		/// NFTMart nft
		type NFT: NftmartNft<Self::AccountId, Self::ClassId, Self::TokenId>;

		/// Extra Configurations
		type ExtraConfig: NftmartConfig<Self::AccountId, BlockNumberFor<Self>>;
	}

	#[pallet::error]
	pub enum Error<T> {

		TakeOwnOffer,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// CreatedOrder \[who, order_id\]
		CreatedOrder(T::AccountId, GlobalId),

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

	/// Index/store offers by account as primary key and order id as secondary key.
	#[pallet::storage]
	#[pallet::getter(fn offers)]
	pub type Offers<T: Config> = StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Twox64Concat, GlobalId, u32>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create an order.
		///
		/// - `currency_id`: currency id
		/// - `category_id`: category id
		/// - `deposit`: The balances to create an order
		/// - `price`: nfts' price.
		/// - `deadline`: deadline
		/// - `items`: a list of `(class_id, token_id, quantity, price)`
		#[pallet::weight(100_000)]
		#[transactional]
		pub fn submit_order(
			origin: OriginFor<T>,
			#[pallet::compact] currency_id: CurrencyIdOf<T>,
			#[pallet::compact] category_id: GlobalId,
			#[pallet::compact] deposit: Balance,
			#[pallet::compact] price: Balance,
			#[pallet::compact] deadline: BlockNumberOf<T>,
			items: Vec<(ClassIdOf<T>, TokenIdOf<T>, TokenIdOf<T>)>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;


			Ok(().into())
		}
	}
}
