#![cfg_attr(not(feature = "std"), no_std)]

sp_api::decl_runtime_apis! {
	/// The API to query account nonce (aka transaction index).
	pub trait AccountNonceApi1<AccountId, Index> where
		AccountId: codec::Codec,
		Index: codec::Codec,
	{
		/// Get current account nonce of given `AccountId`.
		fn account_nonce1(account: AccountId) -> Index;
	}
}
