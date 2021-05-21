#![cfg(test)]
#![allow(unused_imports)]
#![allow(dead_code)]

use super::{NATIVE_CURRENCY_ID};
use crate::mock::{add_class, ExtBuilder, ALICE, BOB, free_balance,
				  Origin, add_token, all_tokens_by, add_category,
				  NftmartOrder, CLASS_ID0, TOKEN_ID1, TOKEN_ID0,
				  last_event, Event, current_gid, ensure_account,
				  CHARLIE,
};
use orml_nft::AccountToken;
use frame_support::{assert_ok};

#[test]
fn xxxx() {
	ExtBuilder::default().build().execute_with(|| {
		add_class(ALICE);
		add_token(BOB, 20, None);
		add_token(BOB, 40, Some(false));
		assert_eq!(vec![
			(CLASS_ID0, TOKEN_ID0, AccountToken { quantity: 20, reserved: 0 }),
			(CLASS_ID0, TOKEN_ID1, AccountToken { quantity: 40, reserved: 0 })
		], all_tokens_by(BOB));


	});
}
