use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::core::address::P2PKHAddress;
use crate::core::blockchain::BlockChain;
use crate::core::parameters::Parameters;
use crate::crypto::hash::hash;
use crate::crypto::public_key::PublicKeyAlgorithm;

pub mod transaction;

pub struct UTXOSet;
impl UTXOSet {
	pub fn genesis(parameters: Parameters) -> HashMap<[u8; 32], Vec<UTXO>> {
		let mut map = HashMap::new();
		// TODO: Add genesis distribution in here
		map
	}
}

#[derive(Clone, Debug, Eq, Hash, Serialize, Deserialize, PartialEq)]
pub struct Input {
	pub prev_txid: [u8; 32],
	pub output_index: usize,
	pub signature: Vec<u8>,
	pub public_key: Vec<u8>,
}

#[derive(Clone, Debug, Eq, Hash, Serialize, Deserialize, PartialEq)]
pub struct Output {
	pub amount: u64,
	pub address: P2PKHAddress,
}
#[derive(Clone, Debug, Eq, Hash, Serialize, Deserialize, PartialEq)]
pub struct UTXO {
	pub txid: [u8; 32],
	pub output_index: usize,
	pub amount: u64,
	pub recipient_address: P2PKHAddress,
}
impl Output {
	pub fn calculate_hash(&self) -> [u8; 32] {
		let str = format!("{}.{}", hex::encode(self.address.address), self.amount);
		hash(str.as_bytes())
	}
}
impl Input {
	pub fn calculate_hash(&self) -> [u8; 32] {
		let str = format!("{}.{}.{}", hex::encode(self.prev_txid), self.output_index, hex::encode(&self.public_key));
		hash(str.as_bytes())
	}
	pub fn verify_signature(&self) -> bool {
		// TODO: Check code
		let hash = self.calculate_hash();
		let signature =  &self.signature;
		if PublicKeyAlgorithm::verify(&self.public_key, &hash, signature).is_ok() {
			return true;
		}
		false
	}
	pub fn validate(&self, blockchain: &BlockChain) -> bool {
		if let Some(utxos) = blockchain.get_utxo_list(&self.prev_txid) {
			if let Some(utxo) = utxos.get(self.output_index) {
				if utxo.txid == self.prev_txid && utxo.output_index == self.output_index {
					let derived_key = P2PKHAddress::from(&self.public_key).address;
					let is_address_correct = derived_key == utxo.recipient_address.address;
					let is_signature_valid = self.verify_signature();
					return is_address_correct && is_signature_valid;
				}
			}
		}
		false
	}
}