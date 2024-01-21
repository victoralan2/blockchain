extern crate rand;

use std::fmt::{Display, Formatter};

use p256::ecdsa::{Signature, SigningKey, VerifyingKey};
use p256::ecdsa::signature::{SignerMut, Verifier};
use rand_core::OsRng;

pub struct PublicKeyAlgorithm;



impl PublicKeyAlgorithm {
	/*
	Returns a randomly generated public and private key: (public_key, private_key)
	 */
	pub fn gen_keypair() -> (Vec<u8>, Vec<u8>) {
		let sk = SigningKey::random(&mut OsRng);
		let vk = VerifyingKey::from(&sk);
		(Self::serialize_vkey(&vk), Self::serialize_skey(&sk))
	}
	pub fn sign(key: &[u8], data: &[u8]) -> Result<Vec<u8>, PublicKeyError> {
		let mut sk = Self::skey_from_bytes(&key)?;
		let signature: Signature = sk.sign(&[]);
		Ok(signature.to_bytes().to_vec())
	}
	/**
	Returns true if the signature matches, false otherwise. If the signature is not valid a error is returned
	**/
	pub fn verify(key: &[u8], data: &[u8], signature: &[u8]) -> Result<(), PublicKeyError> {
		let vk = Self::pkey_from_bytes(&key)?;
		if let Ok(signature) = Signature::from_slice(signature) {
			match vk.verify(data, &signature) {
				Ok(_) => {
					Ok(())
				}
				Err(_) => {
					Err(PublicKeyError{
						description: "The signature didn't match".to_string()
					})
				}
			}
		} else {
			Err(PublicKeyError{
				description: "Invalid signature".to_string()
			})
		}
	}
	fn serialize_vkey(key: &VerifyingKey) -> Vec<u8> {
		key.to_sec1_bytes().to_vec()
	}
 	fn pkey_from_bytes(bytes: &[u8]) -> Result<VerifyingKey, PublicKeyError> {
		match VerifyingKey::from_sec1_bytes(&bytes) {
			Ok(vk) => {
				Ok(vk)
			}
			Err(err) => {
				Err(PublicKeyError {
					description: "Invalid public key".to_string(),
				})
			}
		}
	}
	fn skey_from_bytes(bytes: &[u8]) -> Result<SigningKey, PublicKeyError> {
		match p256::ecdsa::SigningKey::from_slice(bytes) {
			Ok(sk) => {
				Ok(sk)
			}
			Err(err) => {
				Err(PublicKeyError {
					description: "Invalid secret key".to_string(),
				})
			}
		}
	}
	fn serialize_skey(key: &SigningKey) -> Vec<u8> {
		key.to_bytes().to_vec()
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