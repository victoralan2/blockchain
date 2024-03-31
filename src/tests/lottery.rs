use std::hint::black_box;

use crate::consensus::lottery::Lottery;
use crate::crypto::vrf::keygen;

#[test]
fn lottery_test() {
	let (sk, pk) = keygen();

	const NODE_STAKE: u64 = 2u64.pow(5);
	const TOTAL_STAKE: u64 = 2u64.pow(10);
	const ACTIVE_SLOT_COEFFICIENT: f32 = 0.5;
	const ITERATIONS: u64 = 1000;
	// let expected_wins = ((NODE_STAKE as f64 / TOTAL_STAKE as f64) * F_FACTOR * ITERATIONS as f64) as u64;

	for slot in 0..ITERATIONS {
		let lottery = Lottery::run_lottery(slot, ACTIVE_SLOT_COEFFICIENT, &[0u8; 32], &sk, NODE_STAKE, TOTAL_STAKE);
		black_box(lottery);
	}
}