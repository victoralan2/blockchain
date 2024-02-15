use std::fmt::{Debug, Display, Formatter};

use base58::{FromBase58, FromBase58Error, ToBase58};
use serde::{Deserialize, Serialize};

use crate::core::parameters::COIN_NAME_ABBREVIATION;
use crate::crypto::hash::blake;
use crate::crypto::public_key::PublicKeyAlgorithm;

const ADDRESS_SIZE: usize = 16;

#[derive(Clone, Copy, Debug, Hash, Eq, Serialize, Deserialize, PartialEq)]
pub struct P2PKHAddress {
	pub address: [u8; ADDRESS_SIZE]
}
impl P2PKHAddress {
	///
	/// Returns an address, a public and a private key: (P2PKHAddress, private_key, public_key)
	///
	pub fn random() -> (Self, Vec<u8>, Vec<u8>) {
		let (private_key, public_key) = PublicKeyAlgorithm::gen_keypair();
		let address: &[u8; ADDRESS_SIZE] = &blake(&public_key)[0..ADDRESS_SIZE].try_into().expect("Unable to shorten address");

		let addr = P2PKHAddress {
			address: *address,
		};
		(addr, private_key, public_key)
	}
	pub fn null() -> Self {
		P2PKHAddress {
			address: [0u8; ADDRESS_SIZE],
		}
	}
	pub fn from_string(mut string: String) -> Result<Self, FromBase58Error> {
		if string.starts_with(&format!("{}:", COIN_NAME_ABBREVIATION)) {
			string = string[COIN_NAME_ABBREVIATION.len() + 1..].to_string();
		}
		let bytes = string.from_base58()?;
		let mut result = [0u8; ADDRESS_SIZE];
		result.copy_from_slice(&bytes);
		Ok(P2PKHAddress {
			address: result,
		})
	}
	pub fn from(pk: &[u8]) -> Self {
		let address: &[u8; ADDRESS_SIZE] = &blake(pk)[0..ADDRESS_SIZE].try_into().expect("Unable to shorten key");
		P2PKHAddress {
			address: *address,
		}
	}
}
impl Display for P2PKHAddress {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let str = self.address.to_base58();
		write!(f, "{}:{}", COIN_NAME_ABBREVIATION, str)
	}
}