use crypto_hash::Algorithm;

pub mod merkle;

pub fn hash(data: &[u8]) -> [u8; 32] {
	blake(data)
}
fn blake(data: &[u8]) -> [u8; 32] {
	*blake3::hash(data).as_bytes()
}
fn sha256(data: &[u8]) -> Vec<u8> {
	crypto_hash::digest(Algorithm::SHA256, data)
}
