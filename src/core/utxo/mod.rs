use pqcrypto_dilithium::dilithium5::PublicKey;
use serde::{Deserialize, Serialize};
use crate::core::address::P2PKHAddress;
use crate::core::blockchain::BlockChain;
use crate::core::Hashable;
use crate::core::utxo::transaction::Transaction;
use crate::crypto::public_key::Dilithium;
use crate::crypto::hash::hash;
pub mod transaction;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Input {
	pub prev_txid: [u8; 32],
	pub output_index: usize,
	pub signature: Vec<u8>,
	pub public_key: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Output {
	pub amount: u64,
	pub address: P2PKHAddress,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UTXO {
	pub txid: [u8; 32],
	pub output_index: usize,
	pub amount: u64,
	pub recipient_address: P2PKHAddress,
}
impl Output {
	pub fn calculate_hash(&self) -> [u8; 32] {
		let str = format!("{}.{}", hex::encode(&self.address.address), hex::encode(self.calculate_hash()));
		hash(str.as_bytes())
	}
}
impl Input {
	pub fn calculate_hash(&self) -> [u8; 32] {
		let str = format!("{}.{}.{}", hex::encode(&self.prev_txid), self.output_index, hex::encode(&self.public_key));
		hash(str.as_bytes())
	}
	pub fn verify_signature(&self, tx: &Transaction) -> bool {
		let hash = tx.calculate_hash();
		let signature =  &self.signature;
		if let Ok(pk) = Dilithium::pkey_from_bytes(&self.public_key) {
			if let Some(data) = Dilithium::open_dilithium(&pk, signature) {
				if data == hash {
					return true;
				}
			}
		}
		false
	}
	pub fn sign(&mut self, tx: Transaction, sk: &Vec<u8>) {
		let hash = tx.calculate_hash();
		if let Ok(sk) = Dilithium::skey_from_bytes(sk) {
			self.signature = Dilithium::sign_dilithium(&sk, &hash);

		}
	}

	pub fn validate(&self, tx: &Transaction, blockchain: &BlockChain) -> bool {
		if let Some(utxos) = blockchain.get_utxo_list(&self.prev_txid) {
			if let Some(utxo) = utxos.get(self.output_index) {
				if utxo.txid == self.prev_txid && utxo.output_index == self.output_index {
					let derived_key = hash(&self.public_key);
					let is_address_correct = derived_key == utxo.recipient_address.address;
					let is_signature_valid = self.verify_signature(tx);
					return is_address_correct && is_signature_valid;
				}
			}
		}

		false
	}
}