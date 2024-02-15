use num_bigint::BigUint;

pub mod lottery;


const SIZE: usize = 32;
pub fn gen_difficulty(probability: f64) -> [u8; SIZE] {
	let max_num = BigUint::from_bytes_be(&[255u8; SIZE]);
	let result = (max_num.clone() / ((1.0 / probability) as u32)).to_bytes_be(); // Important big endian here
	let mut bytes = [0u8; SIZE];
	let difference = SIZE-result.len();
	// for i in difference..SIZE {
	// 	bytes[i-difference] = result[difference];
	// }
	bytes[difference..SIZE].copy_from_slice(&result[..(SIZE - difference)]);
	bytes
}