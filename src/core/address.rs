use std::fmt::{Debug, Display, Formatter};
use pqcrypto_dilithium::dilithium5::{PublicKey, SecretKey};
use serde::de::Unexpected::Str;
use crate::crypto::hash::sha256;
use crate::crypto::public_key::Dilithium;

#[derive(Clone, PartialEq)]
pub struct Address {
	pub public_key: PublicKey,
	pub address: String
}
impl Debug for Address {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let public_key_str = hex::encode(Dilithium::serialize_pkey(self.public_key));
		let address_str = &self.address;

		// Format the struct fields as a JSON object
		write!(f, r#"{{ "public_key": "{}", "address": "{}" }}"#, public_key_str, address_str)
	}
}
impl Display for Address {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let address_str = &self.address;
		let pk = Dilithium::serialize_pkey(self.public_key);
		// Format the struct fields as a JSON object
		write!(f, r#"{{"public_key": "{}", "address": "{}"}}"#, hex::encode(pk), address_str)
	}
}
impl Address {
	pub fn random() -> (Self, SecretKey) {
		let keypair = Dilithium::gen_dilithium();
		let addr = Address {
			public_key: keypair.0,
			address: hex::encode(sha256(&Dilithium::serialize_pkey(keypair.0))),
		};
		(addr, keypair.1)
	}
	pub fn new() -> Self {
		// TODO
		unimplemented!()
	}
	pub fn null() -> Self {
		// TODO
		Address {
			public_key: Dilithium::pkey_from_bytes(&[0u8; 2592]).unwrap(),
			address: String::new(),
		}
	}
}