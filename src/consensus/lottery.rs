use std::arch::is_aarch64_feature_detected;
use std::cmp::max;
use num_bigint::BigUint;
use crate::consensus::gen_difficulty;
use crate::crypto::vrf::{prove, verify, VrfPk, VrfProof, VrfSk};

pub struct Lottery;

impl Lottery {

	/// Runs a lottery for a given slot.
	/// current_slot is the current slot
	/// active_slot_coefficient is the percentage slots that will be active / have a lieder
	/// last_epoch_hash is the hash of the random oracle of the last epoch
	/// proving_key is the private key of the node running the lottery
	/// node_stake is the stake of the node that runs the lottery
	/// total_staked is the total amount of coins staked
	pub fn run_lottery(current_slot: u64, active_slot_coefficient: f32, last_epoch_hash: &[u8; 32], proving_key: &VrfSk, node_stake: u64, total_staked: u64) -> Option<([u8; 32], VrfProof)> {
		let mut input = last_epoch_hash.to_vec();
		input.append(&mut current_slot.to_be_bytes().to_vec());
		let (lottery_number, proof) = prove(&input, proving_key);

		if Self::is_win(node_stake as f64 / total_staked as f64, active_slot_coefficient, lottery_number) {
			Some((lottery_number, proof))
		} else {
			None
		}
	}

	pub fn is_win(stake_percentage: f64, mut active_slot_coefficient: f32, lottery_number: [u8; 32]) -> bool {
		active_slot_coefficient = active_slot_coefficient.clamp(0.0001, 0.9999);

		let lottery_probability = 1.0-(1.0 - active_slot_coefficient as f64).powf(stake_percentage);

		let threshold = gen_difficulty(lottery_probability);

		lottery_number.cmp(&threshold).is_lt() // Returns true if the lottery is smaller than the threshold
	}

	pub fn verify_vrf_lottery(current_slot: u64, last_epoch_hash: &[u8; 32], vrf: [u8; 32], proof: [u8; 96], public_key: &VrfPk) -> bool {
		let mut input = last_epoch_hash.to_vec();
		input.append(&mut current_slot.to_be_bytes().to_vec());
		if let Ok(proof) = VrfProof::from_bytes(&proof) {
			let is_valid = verify(&input, public_key, &vrf, &proof);
			return is_valid
		}
		false
	}
}