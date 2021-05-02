#![cfg(test)]

use crate::mock::{add_class, ExtBuilder, ALICE, add_token};

#[test]
fn submit_order_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		add_class(ALICE);
		add_token(ALICE, 20, None);
	});
}
