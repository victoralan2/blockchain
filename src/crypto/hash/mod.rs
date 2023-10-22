use argon2::{Config, Variant};
use argon2::Version::Version13;
use crypto_hash::Algorithm;

pub mod merkle;

pub fn hash(data: &[u8]) -> [u8; 32] {
	sha256(data).as_slice().try_into().expect("Unable to convert hash to byte array")
}
pub fn mine_hash(data: &[u8]) -> Vec<u8> {
	// TODO:  Change function
	argon2(data)
}
pub fn argon2(data: &[u8]) -> Vec<u8> {
	let mut config = Config::default();
	config.lanes = 4;
	config.variant = Variant::Argon2id;
	config.version = Version13;
	config.hash_length = 32;
	config.mem_cost = 1024;
	config.time_cost = 1;
	argon2::hash_raw(data, 0u64.to_be_bytes().as_slice(), &config).unwrap()
}
pub fn bcrypt(data: &[u8]) -> String {
	bcrypt::hash(data, 4).unwrap()
}
pub fn sha256(data: &[u8]) -> Vec<u8> {
	crypto_hash::digest(Algorithm::SHA256, data)
}
