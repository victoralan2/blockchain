use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use rand::thread_rng;
use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use crate::address;

use crate::core::address::P2PKHAddress;
use crate::core::blockchain::{BlockChain, BlockChainConfig};
use crate::core::{Hashable, is_smaller};
use crate::core::utxo::coinbase::CoinbaseTransaction;
use crate::core::utxo::transaction::Transaction;
use crate::crypto::hash::merkle::calculate_merkle_root;

#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub struct BlockHeader {
	pub previous_hash: [u8; 32],
	pub nonce: u64,
	pub time: u64,
	pub merkle_root: [u8; 32],
	pub coinbase_transaction: CoinbaseTransaction,
}

#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub struct Block {
	pub header: BlockHeader,
	pub hash: [u8; 32],
	pub transactions: Vec<Transaction>,
}

impl Block {
	pub fn new(transactions: Vec<Transaction>, time: u64, previous_hash: [u8; 32], coinbase_transaction: CoinbaseTransaction) -> Self {
		let header = BlockHeader{
			previous_hash,
			time,
			nonce: 0,
			merkle_root: [0u8; 32],
			coinbase_transaction,
		};
		let mut block = Block {hash: [0u8; 32], transactions, header };
		block.update_hash();
		block
	}
	pub fn genesis() -> Self {
		let addr = P2PKHAddress::random();
		unsafe { address = Some(addr); }
		let header = BlockHeader{
			previous_hash:  [0u8; 32],
			time: 0u64,
			nonce: 0,
			merkle_root: [0u8; 32],
			coinbase_transaction: CoinbaseTransaction::genesis(), // TODO: Make the first miner myself
		};
		let mut block = Block {
			hash: [0u8; 32],
			transactions: vec![],
			header,
		};
		block.update_hash();
		block
	}
	pub fn calculate_reward(&self, config: BlockChainConfig) -> u64 {
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
	pub fn is_valid(&self, blockchain: &BlockChain, index: usize) -> bool {
		if let Some(previous) = blockchain.get_block_at(index - 1) {
			let is_previous_hash_correct = self.header.previous_hash == previous.hash;
			if !is_previous_hash_correct {
				return false;
			}
		} else {
			return false;
		}
		let is_hash_correct = self.calculate_hash() == self.hash;
		// let is_index_correct = self.index == blockchain.get_len();
		let is_pow_valid = is_smaller(&self.hash, &blockchain.configuration.target_value); // TODO: Check that value is valid for its time and not for the current target value
		let is_merkle_tree_correct = self.calculate_merkle_tree() == self.header.merkle_root;

		// CHECKS IF THERE ARE TWO INPUTS USING SAME OUTPUT
		let mut are_tx_unique = true;
		let mut input_tx_list = HashSet::new();

		for tx in &self.transactions {
			for input in &tx.input_list {
				if !input_tx_list.insert(input.calculate_hash()) {
					are_tx_unique = false;
				}
			}
		}
		if !(is_merkle_tree_correct && is_hash_correct && is_pow_valid && are_tx_unique) {
			return false;
		}

		for tx in &self.transactions {
			// TODO: CHECK IF TIMESTAMP IS ACCEPTABLE
			let is_transaction_valid = tx.is_valid(blockchain);
			if !(is_transaction_valid) {
				return false;
			}
		}

		true
	}
}

