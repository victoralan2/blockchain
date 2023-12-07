use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::ADDRESS;
use crate::core::{Hashable, is_smaller};
use crate::core::address::P2PKHAddress;
use crate::core::blockchain::{BlockChain, BlockChainConfig};
use crate::core::utxo::coinbase::CoinbaseTransaction;
use crate::core::utxo::transaction::Transaction;
use crate::crypto::hash::merkle::calculate_merkle_root;

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
pub struct BlockHeader {
	pub hash: [u8; 32],
	pub height: usize,
	pub previous_hash: [u8; 32],
	pub nonce: u64,
	pub time: u64,
	pub merkle_root: [u8; 32],
	pub coinbase_transaction: CoinbaseTransaction,
}
pub type BlockContent = Vec<Transaction>;
#[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
pub struct Block {
	pub header: BlockHeader,
	pub transactions: BlockContent,
}

impl Block {
	pub fn new(height: usize, transactions: Vec<Transaction>, time: u64, previous_hash: [u8; 32], miner_address: P2PKHAddress, blockchain_config: BlockChainConfig) -> Self {
		let header = BlockHeader {
			hash: [0u8; 32],
			height,
			previous_hash,
			time,
			nonce: 0,
			merkle_root: [0u8; 32],
			coinbase_transaction: CoinbaseTransaction::genesis(),
		};
		let mut block = Block { header, transactions };
		let coinbase = CoinbaseTransaction::create(miner_address, block.calculate_reward(blockchain_config));
		block.header.coinbase_transaction = coinbase;

		block.update_hash();
		block
	}
	pub fn genesis() -> Self {
		let addr = P2PKHAddress::random();
		unsafe { ADDRESS = Some(addr); }
		let header = BlockHeader{
			hash: [0u8; 32],
			height: 0,
			previous_hash:  [0u8; 32],
			time: 0u64,
			nonce: 0,
			merkle_root: [0u8; 32],
			coinbase_transaction: CoinbaseTransaction::genesis(),
		};
		let mut block = Block {
			transactions: vec![],
			header,
		};
		block.update_hash();
		block
	}
	pub fn calculate_reward(&self, config: BlockChainConfig) -> u64 {
		// TODO
		0
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
		let is_pow_valid = is_smaller(&self.header.hash, &blockchain.configuration.target_value); // TODO: Check that value is valid for its time and not for the current target value
		let is_merkle_tree_correct = self.calculate_merkle_tree() == self.header.merkle_root;
		let is_coinbase_tx_valid = self.header.coinbase_transaction.is_valid(blockchain);

		if !(is_merkle_tree_correct && is_hash_correct && is_pow_valid && is_height_correct && is_coinbase_tx_valid) {
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

