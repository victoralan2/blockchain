extern crate rand;

use std::fmt::{Display, Formatter};
use pqcrypto_dilithium::dilithium5;
use pqcrypto_dilithium::dilithium5::{keypair, open, PublicKey, SecretKey, sign};
use pqcrypto_traits::sign;
use pqcrypto_traits::sign::{PublicKey as _, SecretKey as _, SignedMessage as _};

pub struct PublicKeyAlgorithm;



impl PublicKeyAlgorithm {
	/*
	Returns a randomly generated public and private key: (public_key, private_key)
	 */
	pub fn gen_dilithium() -> (Vec<u8>, Vec<u8>) {
		let kp = keypair();
		let pk = kp.0;
		let sk = kp.1;
		(Self::serialize_pkey(&pk), Self::serialize_skey(&sk))
	}
	pub fn sign(key: &Vec<u8>, data: &[u8]) -> Result<Vec<u8>, PublicKeyError> {
		let sk = Self::skey_from_bytes(key.as_slice())?;
		sign(data, key).as_bytes().to_vec()
	}
	pub fn open(key: &Vec<u8>, data: &[u8]) -> Result<Vec<u8>, PublicKeyError> {
		let signature = dilithium5::SignedMessage::from_bytes(data).ok()?;
		open(&signature, key).ok()
	}
	fn serialize_pkey(key: &impl sign::PublicKey) -> Vec<u8> {
		key.as_bytes().to_vec()
	}
 	fn pkey_from_bytes(bytes: &[u8]) -> Result<PublicKey, PublicKeyError> {
		match PublicKey::from_bytes(&bytes) {
			Ok(pk) => {
				Ok(pk)
			}
			Err(err) => {
				PublicKeyError {
					description: "Invalid public key".to_string(),
				}
			}
		}
	}
	fn skey_from_bytes(bytes: &[u8]) -> Result<SecretKey, PublicKeyError> {
		SecretKey::from_bytes(&bytes)
	}
	fn serialize_skey(key: &impl sign::SecretKey) -> Vec<u8> {
		key.as_bytes().to_vec()
	}
}
#[derive(Debug)]
pub struct PublicKeyError {
	description: String
}
impl Display for PublicKeyError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}