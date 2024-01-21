use num_bigint::BigUint;

pub fn gen_difficulty(difficulty: u128) -> [u8; 32] {
	let a = BigUint::from_bytes_be(&[255u8; 32]);
	let result = (a.clone() - (a / (difficulty + 1))).to_bytes_be();
	let mut bytes = [0u8; 32];
	bytes.copy_from_slice(&result);
	bytes
}