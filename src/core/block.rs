use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use rand::thread_rng;
use rand_core::RngCore;
use serde::{Deserialize, Serialize};

use crate::core::address::P2PKHAddress;
use crate::core::blockchain::{BlockChain, BlockChainConfig};
use crate::core::Hashable;
use crate::core::utxo::transaction::Transaction;
use crate::crypto::hash::merkle::calculate_merkle_root;

#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub struct BlockHeader {
	pub previous_hash: [u8; 32],
	pub nonce: u64,
	pub time: u64,
	pub merkle_root: [u8; 32],
	pub miners_address: P2PKHAddress,
}

#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub struct Block {
	pub index: usize,
	pub header: BlockHeader,
	pub hash: [u8; 32],
	pub transactions: Vec<Transaction>,
}

impl Block {
	pub fn new(transactions: Vec<Transaction>, time: u64, index: usize) -> Self {
		let block_header = BlockHeader{
			previous_hash:  [0u8; 32],
			time,
			nonce: 0,
			merkle_root: [0u8; 32],
			miners_address: P2PKHAddress::null(),
		};
		let mut block = Block {hash: [0u8; 32], transactions, index, header: block_header };
		block.update_hash();
		block
	}
	pub fn genesis() -> Self {
		let block_header = BlockHeader{
			previous_hash:  [0u8; 32],
			time: 0u64,
			nonce: 0,
			merkle_root: [0u8; 32],
			miners_address: P2PKHAddress::null(),
		};
		Block {
			hash: [0u8; 32],
			transactions: vec![],
			index: 0,
			header: block_header,
		}
	}
	pub fn mine(&mut self, difficulty: u8, keep_mining: Arc<AtomicBool>) -> bool {
		self.header.nonce = thread_rng().next_u64();
		while get_leading_zeros(&self.hash) < difficulty as u32 && keep_mining.load(Ordering::Relaxed) {
			self.header.nonce += 1;
			self.update_hash();
		}
		keep_mining.load(Ordering::Relaxed) // Return true if the block was mined, false if the opperation was cancelled
	}
	pub fn calculate_reward(&self, config: &BlockChainConfig) -> u64 {
		todo!()
		// TODO
	}
	pub fn calculate_merkle_tree(&self) -> [u8; 32]{
		let mut hashes: Vec<[u8; 32]> = Vec::new();
		for tx in &self.transactions {
			hashes.push(tx.calculate_hash());
		}
		calculate_merkle_root(hashes)
	}
	pub fn is_valid(&self, blockchain: &BlockChain) -> bool {
		if let Some(previous) = blockchain.get_block_at(self.index - 1) {
			let is_previous_hash_correct = self.header.previous_hash == previous.hash;
			if !is_previous_hash_correct {
				return false;
			}
		} else {
			return false;
		}
		let is_hash_correct = self.calculate_hash() == self.hash;
		let is_index_correct = self.index == blockchain.get_len();
		let is_merkle_tree_correct = self.calculate_merkle_tree() == self.header.merkle_root;
		if !(is_merkle_tree_correct && is_hash_correct && is_index_correct){
			return false;
		}
		for tx in &self.transactions {

			// TODO: CHECK IF TIMESTAMP IS ACCEPTABLE
			let is_transaction_valid = tx.is_valid(blockchain);
			let is_unique = 1 == self.transactions.iter()
				.filter(|&tx2| tx.calculate_hash() == tx2.calculate_hash()).count();
			if !(is_transaction_valid && is_unique) {
				return false;
			}
		}

		true
	}
}

pub fn get_leading_zeros(vec: &[u8]) -> u32 {
	vec.iter().try_fold(0, |acc, n| {
		if n == &0 {
			Ok(acc + 8)
		} else {
			Err(acc + n.leading_zeros())
		}
	}).unwrap_or_else(|e| e)
}
