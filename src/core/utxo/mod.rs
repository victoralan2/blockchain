use serde::{Deserialize, Serialize};
use crate::core::address::P2PKHAddress;
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
	pub public_key: Vec<u8>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UTXO {
	pub txid: [u8; 32],
	pub output_index: usize,
	pub amount: u64,
}

impl Input {
	/// This function signs the transaction with the given key.
	/// It also __updates_the_hash__ of the transaction as a **side effect**

	pub fn calculate_hash(&self) -> [u8; 32] {
		let str = format!("{}.{}.{}", hex::encode(self.prev_txid), self.output_index, hex::encode(&self.public_key));
		hash(str.as_bytes())
	}
}
