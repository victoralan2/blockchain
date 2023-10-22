use std::fmt::{Debug, Display};

use base58::{FromBase58, FromBase58Error, ToBase58};
use pqcrypto_dilithium::dilithium5::{PublicKey, SecretKey};
use serde::{Deserialize, Serialize};

use crate::crypto::hash::hash;
use crate::crypto::public_key::Dilithium;

const SIGNATURE_BYTES: [u8; 2] = [0b0001000, 0b0001000];

#[derive(Clone, Debug, Ord, PartialOrd, Eq, Serialize, Deserialize)]
pub struct P2PKHAddress {
	pub address: [u8; 32]
}
impl P2PKHAddress {
	pub fn random() -> (Self, PublicKey, SecretKey) {
		let keypair = Dilithium::gen_dilithium();
		let addr = P2PKHAddress {
			address: hash(&Dilithium::serialize_pkey(&keypair.0)),
		};
		(addr, keypair.0, keypair.1)
	}
	pub fn null() -> Self {
		P2PKHAddress {
			address: [0u8; 32],
		}
	}
	pub fn from_string(string: String) -> Result<Self, FromBase58Error>{
		let bytes = string.from_base58()?;
		let bytes = &bytes[2..];
		let mut result = [0u8; 32];
		result.copy_from_slice(bytes);
		Ok(P2PKHAddress {
			address: result,
		})
	}
	pub fn to_string(&self) -> String {
		Self::hash_to_address(self.address).to_base58()
	}
	pub fn from(pk: PublicKey) -> Self {
		P2PKHAddress {
			address: hash(&Dilithium::serialize_pkey(&pk)),
		}
	}
	pub fn hash_to_address(hash: [u8; 32]) -> [u8; 34]{
		let mut signature_bytes = SIGNATURE_BYTES.to_vec();
		signature_bytes.append(&mut hash.to_vec());
		let mut result = [0u8; 34];
		result.copy_from_slice(&signature_bytes);
		result
	}
}
// #[derive(Clone, PartialEq)]
// pub struct Address {
// 	pub public_key: PublicKey,
// 	pub address: String
// }
// impl Debug for Address {
// 	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
// 		let public_key_str = hex::encode(Dilithium::serialize_pkey(self.public_key));
// 		let address_str = &self.address;
//
// 		// Format the struct fields as a JSON object
// 		write!(f, r#"{{ "public_key": "{}", "address": "{}" }}"#, public_key_str, address_str)
// 	}
// }
// impl Address {
// 	pub fn random() -> (Self, SecretKey) {
// 		let keypair = Dilithium::gen_dilithium();
// 		let addr = Address {
// 			public_key: keypair.0,
// 			address: hash(&Dilithium::serialize_pkey(keypair.0)).to_base58(),
// 		};
// 		(addr, keypair.1)
// 	}
// 	pub fn from(pk: PublicKey) -> Self {
// 		Address {
// 			public_key: pk,
// 			address: hash(&Dilithium::serialize_pkey(pk)).to_base58(),
// 		}
// 	}
// 	pub fn null() -> Self {
// 		Address {
// 			public_key: Dilithium::pkey_from_bytes(&[0u8; 2592]).unwrap(),
// 			address: String::new(),
// 		}
// 	}
// }