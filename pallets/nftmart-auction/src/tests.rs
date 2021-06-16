#![cfg(test)]
#![allow(unused_imports)]
#![allow(dead_code)]

use super::{NATIVE_CURRENCY_ID};
use crate::mock::*;
use sp_runtime::PerU16;
use orml_nft::AccountToken;
use frame_support::{assert_ok};
use paste::paste;

macro_rules! submit_british_auction_should_work {
    ( $(#[$attr: meta])* $test_name: ident, $hammer_price: expr) => {
		paste! {
			#[test]
			$(#[$attr])*
			fn [<submit_british_auction_should_work $test_name>] () {
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

					let bob_free = 100;
					assert_eq!(free_balance(&BOB), bob_free);

					let deposit = 50;

					let auction_id = current_gid();
					assert_ok!(NftmartAuction::submit_british_auction(
						Origin::signed(BOB),
						NATIVE_CURRENCY_ID,
						$hammer_price, // hammer_price
						PerU16::from_percent(50), // min_raise
						deposit, // deposit
						200, // init_price
						10, // deadline
						true, // allow_delay
						cate_id, // category_id
						vec![(CLASS_ID0, TOKEN_ID0, 10), (CLASS_ID0, TOKEN_ID1, 20)],
					));
					let event = Event::NftmartAuction(crate::Event::CreatedBritishAuction(BOB, auction_id));
					assert_eq!(last_event(), event);

					assert_eq!(vec![
						(CLASS_ID0, TOKEN_ID0, AccountToken { quantity: 10, reserved: 10 }),
						(CLASS_ID0, TOKEN_ID1, AccountToken { quantity: 20, reserved: 20 })
					], all_tokens_by(BOB));
					assert_eq!(free_balance(&BOB), bob_free - deposit);
					assert_eq!(1, categories(cate_id).count);
					assert!(get_bid(auction_id).is_some());
					assert!(get_auction(&BOB, auction_id).is_some());
				});
			}
		}
	};
}

submit_british_auction_should_work!(_1, 500);
submit_british_auction_should_work!(_2, 0);
submit_british_auction_should_work!(_3, 201);

#[test]
fn bid_british_auction_should_work_hammer_price() {
	ExtBuilder::default().build().execute_with(|| {
		add_class(ALICE);
		add_token(BOB, 20, None);
		add_token(BOB, 40, Some(false));
		let cate_id = current_gid();
		add_category();
		let auction_id = current_gid();

		let bob_free = free_balance(&BOB);
		let hammer = 500;
		assert_ok!(NftmartAuction::submit_british_auction(
			Origin::signed(BOB),
			NATIVE_CURRENCY_ID,
			hammer, // hammer_price
			PerU16::from_percent(50), // min_raise
			50, // deposit
			200, // init_price
			10, // deadline
			true, // allow_delay
			cate_id, // category_id
			vec![(CLASS_ID0, TOKEN_ID0, 10), (CLASS_ID0, TOKEN_ID1, 20)],
		));

		let price = 600;
		assert_ok!(NftmartAuction::bid_british_auction(Origin::signed(CHARLIE), price, BOB, auction_id));
		let event = Event::NftmartAuction(crate::Event::HammerBritishAuction(CHARLIE, auction_id));
		assert_eq!(last_event(), event);

		assert_eq!(vec![
			(CLASS_ID0, TOKEN_ID0, AccountToken { quantity: 10, reserved: 0 }),
			(CLASS_ID0, TOKEN_ID1, AccountToken { quantity: 20, reserved: 0 })
		], all_tokens_by(CHARLIE));
		assert_eq!(free_balance(&CHARLIE), price - hammer);
		assert_eq!(free_balance(&BOB), bob_free + hammer);
	});
}

#[test]
fn bid_british_auction_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		add_class(ALICE);
		add_token(BOB, 20, None);
		add_token(BOB, 40, Some(false));
		let cate_id = current_gid();
		add_category();
		let auction_id = current_gid();

		let hammer = 500;
		assert_ok!(NftmartAuction::submit_british_auction(
			Origin::signed(BOB),
			NATIVE_CURRENCY_ID,
			hammer, // hammer_price
			PerU16::from_percent(50), // min_raise
			50, // deposit
			200, // init_price
			10, // deadline
			true, // allow_delay
			cate_id, // category_id
			vec![(CLASS_ID0, TOKEN_ID0, 10), (CLASS_ID0, TOKEN_ID1, 20)],
		));

		let price = 200;
		assert_eq!(free_balance(&CHARLIE), CHARLIE_INIT);
		assert_ok!(NftmartAuction::bid_british_auction(Origin::signed(CHARLIE), price, BOB, auction_id));
		let event = Event::NftmartAuction(crate::Event::BidBritishAuction(CHARLIE, auction_id));
		assert_eq!(last_event(), event);
		assert_eq!(reserved_balance(&CHARLIE), price);
		assert_eq!(free_balance(&CHARLIE), CHARLIE_INIT - price);
	});
}
