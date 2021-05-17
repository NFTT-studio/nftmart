#![cfg(test)]

use super::{NATIVE_CURRENCY_ID};
use crate::mock::{add_class, ExtBuilder, ALICE, BOB, free_balance,
				  Origin, add_token, all_tokens_by, add_category,
				  NftmartOrder, CLASS_ID0, TOKEN_ID1, TOKEN_ID0,
				  last_event, Event, current_gid, ensure_account};
use orml_nft::AccountToken;
use frame_support::{assert_ok};

#[test]
fn submit_order_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		add_class(ALICE);
		add_token(BOB, 20, None);
		add_token(BOB, 40, Some(false));

		let cate_id = current_gid();
		add_category();

		assert_eq!(vec![
			(CLASS_ID0, TOKEN_ID0, AccountToken { quantity: 20, reserved: 0 }),
			(CLASS_ID0, TOKEN_ID1, AccountToken { quantity: 40, reserved: 0 })
		], all_tokens_by(BOB));

		let order_id = current_gid();
		let deposit = 10;
		let price = 100;
		let deadline = 2;

		assert_ok!(NftmartOrder::submit_order(Origin::signed(BOB),
			NATIVE_CURRENCY_ID, cate_id, deposit, price, deadline,
			vec![(CLASS_ID0, TOKEN_ID0, 10), (CLASS_ID0, TOKEN_ID1, 20)]
		));

		assert_eq!(
			last_event(),
			Event::nftmart_order(crate::Event::CreatedOrder(BOB, order_id)),
		);

		// Some tokens should be reserved.
		ensure_account(&BOB, CLASS_ID0, TOKEN_ID0, 10, 10);
		ensure_account(&BOB, CLASS_ID0, TOKEN_ID1, 20, 20);
	});
}

#[test]
fn take_order_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		add_class(ALICE);
		assert_eq!(198, free_balance(&ALICE));

		add_token(BOB, 20, None);
		add_token(BOB, 40, Some(false));

		let cate_id = current_gid();
		add_category();

		assert_eq!(vec![
			(CLASS_ID0, TOKEN_ID0, AccountToken { quantity: 20, reserved: 0 }),
			(CLASS_ID0, TOKEN_ID1, AccountToken { quantity: 40, reserved: 0 })
		], all_tokens_by(BOB));

		let order_id = current_gid();
		let deposit = 10;
		let price = 100;
		let deadline = 2;

		assert_ok!(NftmartOrder::submit_order(Origin::signed(BOB),
			NATIVE_CURRENCY_ID, cate_id, deposit, price, deadline,
			vec![(CLASS_ID0, TOKEN_ID0, 10), (CLASS_ID0, TOKEN_ID1, 20)]
		));

		assert_eq!(
			last_event(),
			Event::nftmart_order(crate::Event::CreatedOrder(BOB, order_id)),
		);

		assert_ok!(NftmartOrder::take_order(Origin::signed(ALICE), order_id, BOB));

		assert_eq!(98, free_balance(&ALICE));
		assert_eq!(200, free_balance(&BOB));
		ensure_account(&BOB, CLASS_ID0, TOKEN_ID0, 0, 10);
		ensure_account(&BOB, CLASS_ID0, TOKEN_ID1, 0, 20);
		ensure_account(&ALICE, CLASS_ID0, TOKEN_ID0, 0, 10);
		ensure_account(&ALICE, CLASS_ID0, TOKEN_ID1, 0, 20);
	});
}

