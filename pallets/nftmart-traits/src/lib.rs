#![cfg_attr(not(feature = "std"), no_std)]

// use frame_support::Parameter;
// use frame_support::pallet_prelude::*;

pub trait NftmartConfig<AccountId> {
	fn is_in_whitelist(_who: &AccountId) -> bool;
}
