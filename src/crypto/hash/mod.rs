use crypto_hash::Algorithm;

pub mod merkle;

pub fn hash(data: &[u8]) -> [u8; 32] {
	sha256(data).as_slice().try_into().expect("Unable to convert hash to byte array")
}

pub fn sha256(data: &[u8]) -> Vec<u8> {
	crypto_hash::digest(Algorithm::SHA256, data)
}
