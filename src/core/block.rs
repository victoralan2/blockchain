use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::core::blockchain::BlockChain;
use crate::core::Hashable;
use crate::core::utxo::transaction::Transaction;
use crate::crypto::hash::merkle::calculate_merkle_root;
use crate::crypto::public_key::{PublicKeyAlgorithm, PublicKeyError};

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
pub struct BlockHeader {
	pub hash: [u8; 32],
	pub height: usize,
	pub previous_hash: [u8; 32],
	pub time: u64,
	pub merkle_root: [u8; 32],
	pub forger_signature: Vec<u8>,
	pub forger_key: Vec<u8>,
}
pub type BlockContent = Vec<Transaction>;
#[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
pub struct Block {
	pub header: BlockHeader,
	pub transactions: BlockContent,
}

impl Block {

	pub fn new(height: usize, transactions: Vec<Transaction>, time: u64, previous_hash: [u8; 32], public_key:&[u8], private_key: &[u8]) -> Result<Self, PublicKeyError> {
		let header = BlockHeader {
			hash: [0u8; 32],
			height,
			previous_hash,
			time,
			merkle_root: [0u8; 32],
			forger_signature: vec![],
			forger_key: public_key.to_vec(),
		};
		let mut block = Block { header, transactions };

		block.sign(private_key)?;
		block.update_hash();
		Ok(block)
	}
	pub fn genesis() -> Self {
		const EXTRA_ENTROPY:  [u8; 32] = [60, 92, 162, 110, 82, 120, 10, 250, 102, 233, 226, 182, 114, 155, 80, 178, 35, 57, 107, 9, 122, 187, 253, 38, 160, 225, 171, 15, 110, 230, 47, 21];
		let header = BlockHeader {
			hash: [0u8; 32],
			height: 0,
			previous_hash: EXTRA_ENTROPY,
			time: 0u64,
			merkle_root: [0u8; 32],
			forger_signature: vec![],
			forger_key: vec![],
		};
		let mut block = Block {
			transactions: vec![],
			header,
		};
		block.update_hash();
		block
	}
	pub fn sign(&mut self, private_key: &[u8]) -> Result<(), PublicKeyError>{
		let hash = self.calculate_hash();
		self.header.forger_signature = PublicKeyAlgorithm::sign(private_key, &hash)?;
		Ok(())
	}
	pub fn verify_signature(&self) -> bool {
		let hash = self.calculate_hash();
		let signature =  &self.header.forger_signature;
		if PublicKeyAlgorithm::verify(&self.header.forger_key, &hash, signature).is_ok() {
			return true;
		}
		false
	}
	pub fn calculate_merkle_tree(&self) -> [u8; 32]{

		let mut hashes: Vec<[u8; 32]> = Vec::new();
		for tx in &self.transactions {
			hashes.push(tx.calculate_hash());
		}
		calculate_merkle_root(hashes)
	}
	pub fn is_valid(&self, blockchain: &BlockChain, height: usize) -> bool {
		// TODO: CHECK IF TIMESTAMP IS ACCEPTABLE, CHECK: https://en.bitcoin.it/wiki/Block_timestamp
		if let Some(previous) = blockchain.get_block_at(height - 1) {
			let is_previous_hash_correct = self.header.previous_hash == previous.header.hash;
			if !is_previous_hash_correct {
				return false;
			}
		} else {
			return false;
		}
		let is_hash_correct = self.calculate_hash() == self.header.hash;
		let is_height_correct = self.header.height == blockchain.get_height();
		let is_merkle_tree_correct = self.calculate_merkle_tree() == self.header.merkle_root;
		// TODO: Check for leader validity
		if !(is_merkle_tree_correct && is_hash_correct && is_height_correct) {
			return false;
		}
		let mut input_tx_list = HashSet::new();
		for tx in &self.transactions {
			// CHECKS IF THERE ARE TWO INPUTS USING SAME OUTPUT
			for input in &tx.input_list {
				if !input_tx_list.insert(input.calculate_hash()) {
					return false;
				}
			}
			let is_transaction_valid = tx.is_valid(blockchain);

			if !(is_transaction_valid) {
				return false;
			}
		}
		true
	}
}

