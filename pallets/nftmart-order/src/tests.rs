#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use crate::mock::{Event, *};

#[test]
fn submit_order_should_work() {
	ExtBuilder::default().build().execute_with(|| {

	});
}
