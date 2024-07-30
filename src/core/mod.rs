use std::io::Read;

use crate::core::block::Block;
use crate::core::utxo::transaction::Transaction;
use crate::crypto::hash::hash;
use crate::crypto::hash::merkle::calculate_merkle_root;

pub mod blockchain;
pub mod block;
pub mod address;
pub mod utxo;
pub mod parameters;
pub mod keys;

pub trait Hashable {
	fn calculate_hash(&self) -> [u8; 32];
	fn update_hash(&mut self);
}
impl Hashable for Block {
	/// IMPORTANT
	/// CHECK VALIDITY OF DATA BEFORE CALCULATING HASH. HASH DOES NOT CHECK FOR ERRORS IN COHERENCE
	fn calculate_hash(&self) -> [u8; 32]{
		let header = &self.header;
		let merkle_tree = self.calculate_merkle_tree();
		let str = format!("{}.{}.{}.{}.{}", hex::encode(header.previous_hash), hex::encode(merkle_tree), header.nonce, header.height, header.miner_address);
		hash(str.as_bytes()).as_slice().try_into().expect("Unable to convert hash to byte array")
	}
	fn update_hash(&mut self) {
		let merkle_tree = self.calculate_merkle_tree();
		self.header.merkle_root = merkle_tree;
		self.header.hash = self.calculate_hash();
	}
}
impl Hashable for Transaction {
	/// IMPORTANT
	/// CHECK VALIDITY OF DATA BEFORE CALCULATING HASH. HASH DOES NOT CHECK FOR ERRORS IN COHERENCE
	fn calculate_hash(&self) -> [u8; 32] {
		let input_hash_list = self.input_list.iter().map(|x|x.calculate_hash()).collect();
		let inputs = hex::encode(calculate_merkle_root(input_hash_list));

		let output_hash_list = self.output_list.iter().map(|x|x.calculate_hash()).collect();
		let outputs = hex::encode(calculate_merkle_root(output_hash_list));

		let str = format!("{}.{}", inputs, outputs);
		hash(str.as_bytes())
	}
	fn update_hash(&mut self) {
		self.id = self.calculate_hash();
	}
}