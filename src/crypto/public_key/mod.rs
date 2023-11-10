extern crate rand;

use pqcrypto_dilithium::dilithium5;
use pqcrypto_dilithium::dilithium5::{keypair, open, PublicKey, SecretKey, sign};
use pqcrypto_traits::sign;
use pqcrypto_traits::sign::{PublicKey as _, SecretKey as _, SignedMessage as _};

pub struct Dilithium;

impl Dilithium {
	pub fn gen_dilithium() -> (PublicKey, SecretKey) {
		keypair()
	}
	pub fn sign_dilithium(key: &SecretKey, data: &[u8]) -> Vec<u8> {
		sign(data, key).as_bytes().to_vec()
	}
	pub fn open_dilithium(key: &PublicKey, data: &[u8]) -> Option<Vec<u8>> {
		let signature = dilithium5::SignedMessage::from_bytes(data).ok()?;
		open(&signature, key).ok()
	}
	pub fn serialize_pkey(key: &impl sign::PublicKey) -> Vec<u8> {
		key.as_bytes().to_vec()
	}
	pub fn pkey_from_bytes(bytes: &[u8]) -> pqcrypto_traits::Result<PublicKey> {
		PublicKey::from_bytes(&bytes)
	}
	pub fn skey_from_bytes(bytes: &[u8]) -> pqcrypto_traits::Result<SecretKey> {
		SecretKey::from_bytes(&bytes)
	}
	pub fn serialize_skey(key: &impl sign::SecretKey) -> Vec<u8> {
		key.as_bytes().to_vec()
	}
}
