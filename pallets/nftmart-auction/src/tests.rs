#![cfg(test)]
#![allow(unused_imports)]
#![allow(dead_code)]

use super::{NATIVE_CURRENCY_ID};
use crate::mock::*;
use sp_runtime::PerU16;
use orml_nft::AccountToken;
use frame_support::{assert_ok};

#[test]
fn submit_british_auction_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		add_class(ALICE);
		add_token(BOB, 20, None);
		add_token(BOB, 40, Some(false));
		assert_eq!(vec![
			(CLASS_ID0, TOKEN_ID0, AccountToken { quantity: 20, reserved: 0 }),
			(CLASS_ID0, TOKEN_ID1, AccountToken { quantity: 40, reserved: 0 })
		], all_tokens_by(BOB));

		let cate_id = current_gid();
		add_category();

		assert_ok!(NftmartAuction::submit_british_auction(
			Origin::signed(BOB),
			NATIVE_CURRENCY_ID,
			500, // hammer_price
			PerU16::from_percent(50), // min_raise
			50, // deposit
			200, // init_price
			10, // deadline
			true, // allow_delay
			cate_id, // category_id
			vec![(CLASS_ID0, TOKEN_ID0, 10), (CLASS_ID0, TOKEN_ID1, 20)],
		));
	});
}
