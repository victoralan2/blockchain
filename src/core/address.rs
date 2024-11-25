use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use base58::{FromBase58, FromBase58Error, ToBase58};
use serde::{Deserialize, Serialize};
use crate::core::parameters::COIN_NAME_ABBREVIATION;
use crate::crypto::hash::{hash};
use crate::crypto::public_key::PublicKeyAlgorithm;

const ADDRESS_SIZE: usize = 16;

#[derive(Clone, Copy, Debug, Hash, Eq, Serialize, Deserialize, PartialEq)]
pub struct P2PKHAddress {
	pub address: [u8; ADDRESS_SIZE],
}
pub type SigningKey = Vec<u8>;
pub type VerifyingKey = Vec<u8>;

impl P2PKHAddress {
	/// Returns an address, a public and a private key: (P2PKHAddress, private_key, public_key)
	pub fn random() -> (Self, SigningKey, VerifyingKey) {
		let (private_key, public_key) = PublicKeyAlgorithm::gen_keypair();
		let address: &[u8; ADDRESS_SIZE] = &hash(&public_key)[0..ADDRESS_SIZE].try_into().expect("Unable to shorten address");
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
	pub fn from_string(mut string: String) -> anyhow::Result<Self, Box<dyn Error>> {
		if string.starts_with(&format!("{}:", COIN_NAME_ABBREVIATION)) {
			string = string[COIN_NAME_ABBREVIATION.len() + 1..].to_string();
		} else {
			return Err(format!("Address has to have the prefix '{}:'", COIN_NAME_ABBREVIATION).into());
		}
		if let Ok(bytes) = string.from_base58() {
			let mut result = [0u8; ADDRESS_SIZE];
			if bytes.len() != ADDRESS_SIZE {
				return Err(format!("Address must be {} characters long", bytes_to_base58_chars(ADDRESS_SIZE)).into());
			}
			result.copy_from_slice(&bytes);
			Ok(P2PKHAddress {
				address: result,
			})
		} else {
			Err("Unable to decode base58".into())
		}
	}
	pub fn from(pk: &[u8]) -> Self {
		let address: &[u8; ADDRESS_SIZE] = &hash(pk)[0..ADDRESS_SIZE].try_into().expect("Unable to shorten key");
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
fn bytes_to_base58_chars(n: usize) -> i32 {
	// 1 byte = 8 bits
	let bits = n * 8;
	// Each Base58 character represents approximately 5.858 bits
	let bits_per_char = 58f64.log2();

	// Calculate the number of Base58 characters
	let chars = (bits as f64) / bits_per_char;

	chars.ceil() as i32
}
