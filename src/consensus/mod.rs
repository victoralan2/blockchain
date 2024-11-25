pub(crate) mod miner;

use num_bigint::BigUint;

const SIZE: usize = 32;
pub fn gen_difficulty(k: u32) -> [u8; SIZE] {
	let max_num = BigUint::from_bytes_be(&[255u8; SIZE]);
	let result = (max_num.clone() / k).to_bytes_be(); // Important big endian here
	let mut bytes = [0u8; SIZE];
	let difference = SIZE-result.len();
	bytes[difference..SIZE].copy_from_slice(&result[..(SIZE - difference)]);
	bytes
}

