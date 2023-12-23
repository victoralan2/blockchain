use crypto_hash::Algorithm;

pub mod merkle;

pub fn hash(data: &[u8]) -> [u8; 32] {
	sha256(data).as_slice().try_into().expect("Unable to convert hash to byte array")
}
// pub fn mine_hash(data: &[u8]) -> [u8; 32] {
// 	scrypt(data)
// }
// pub fn scrypt(data: &[u8]) -> [u8; 32] {
// 	let params = scrypt::Params::new(11, 8, 1, 32).unwrap();
// 	let mut out = [0u8; 32];
// 	scrypt::scrypt(data, b"", &params, &mut out).unwrap();
// 	out
// }
pub fn sha256(data: &[u8]) -> Vec<u8> {
	crypto_hash::digest(Algorithm::SHA256, data)
}
