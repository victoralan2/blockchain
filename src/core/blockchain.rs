use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::SystemTime;
use rand::thread_rng;
use rand_core::RngCore;
use serde::{Deserialize, Serialize};

use crate::core::address::P2PKHAddress;
use crate::core::block::{Block, get_leading_zeros};
use crate::core::Hashable;
use crate::core::utxo::transaction::Transaction;
use crate::core::utxo::UTXO;

#[derive(Clone, Serialize, Deserialize)]
pub struct BlockChainConfig {
	pub(crate) difficulty: u8,
	pub(crate) reward: u64,
	pub(crate) block_size: usize,
	pub(crate) trust_threshold: u32,
	pub(crate) transaction_fee_multiplier: f64,
	pub(crate) max_transaction_fee: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BlockChain {
	chain: Vec<Block>,
	pub utxo_set: HashMap<[u8; 32], Vec<UTXO>>, //TODO: Not this
	mempool: HashSet<Transaction>,
	pub configuration: BlockChainConfig,
}

impl BlockChain {
	pub fn new_empty(configuration: BlockChainConfig) -> Self {
		let chain = vec![Block::genesis()];
		BlockChain { chain, utxo_set: HashMap::new(), mempool: Default::default(), configuration }
	}
	pub fn new(chain: Vec<Block>, mempool: HashSet<Transaction>, configuration: BlockChainConfig) -> Self {
		BlockChain { chain, utxo_set: HashMap::new(),mempool, configuration }
	}
	pub fn clone_empty(&self) -> Self {
		Self {
			chain: vec![],
			utxo_set: Default::default(),
			mempool: Default::default(),
			configuration: self.configuration.clone(),
		}
	}
	pub fn get_utxo_list(&self, txid: &[u8; 32]) -> Option<&Vec<UTXO>>{
		self.utxo_set.get(txid)
	}
	pub fn get_utxo_list_by_address(&self, address: &P2PKHAddress) -> Vec<UTXO> {
		let mut utxos = Vec::new();
		for (_, utxo_list) in &self.utxo_set {
			for utxo in utxo_list {
				if utxo.recipient_address.eq(address) {
					utxos.push(utxo.clone());
				}
			}
		}
		utxos
	}
	pub fn replace(&mut self, new: BlockChain) {
		self.chain = new.chain;
	}
	pub fn truncate(&mut self, index: usize) {
		self.chain.truncate(index);
	}
	/// Validates and adds the transaction to the memory pool if valid.
	/// Returns whether the it was added or not
	pub fn add_transaction_to_mempool(&mut self, tx: &Transaction) -> bool{
		let is_valid = tx.is_valid(self);
		if is_valid {
			self.mempool.insert(tx.clone()) && is_valid
		} else {
			false
		}

	}
	pub fn mine_one_block(&mut self, miner: P2PKHAddress) -> Option<Block> {
		if self.mempool.len() >= self.configuration.block_size {
			let block_size = self.configuration.block_size;
			let transaction_slices: Vec<Transaction> = self.mempool
				.iter()
				.take(block_size)
				.cloned()
				.collect::<Vec<_>>();

			let mut new_block = Block::new(
				transaction_slices.clone(),
				SystemTime::now()
					.duration_since(SystemTime::UNIX_EPOCH)
					.unwrap()
					.as_secs(),
				self.chain.len(),
			);

			new_block.header.miners_address = miner;

			if let Some(mut last_block) = self.get_last_block() {

				last_block.update_hash();
				let hash_val = last_block.hash;
				new_block.header.previous_hash = hash_val;

				// TODO: MAKE MINING CANCELLABLE
				new_block.header.nonce = thread_rng().next_u64();
				new_block.update_hash();
				while true {
					new_block.header.nonce += 1;
					if get_leading_zeros(&new_block.calculate_hash()) < self.configuration.difficulty as u32 {
						new_block.update_hash();
					} // TODO
				}
				println!("Block mined, nonce to solve PoW: {}", new_block.header.nonce);
				return Some(new_block);
			}
		}
		None
	}
	pub fn get_balance(&self, address: &P2PKHAddress) -> u64 {
		let mut balance = 0;
		for utxo_list in self.utxo_set.values() {
			for utxo in utxo_list.iter().filter(|x|x.recipient_address.eq(address)) {
				balance += utxo.amount;
			}
		}
		balance
	}
	pub fn get_block_at(&self, index: usize) -> Option<&Block> {
		if index < self.chain.len(){
			Some(&self.chain[index])
		} else {
			None
		}
	}
	pub fn get_len(&self) -> usize {
		self.chain.len()
	}
	pub fn get_trusted_len(&self) -> usize {
		let len = self.chain.len();
		if len > self.configuration.trust_threshold as usize {
			self.get_len() - self.configuration.trust_threshold as usize
		} else {
			0
		}
	}
	pub fn get_last_trusted_block(&self) -> &Block {
		&self.chain[self.get_trusted_len() - 1]
	}
	pub fn get_last_block(&self) -> Option<&Block> {
		self.chain.last()
	}

	pub fn add_block(&mut self, new_block: &Block) -> bool {
		if new_block.is_valid(self) {
			// Todo: some more checks and add block to blockchain
			for d in &new_block.transactions {
				self.mempool.retain(|d2| d.eq(d2))
			}
			self.chain.push(new_block.clone());
			return true;
		}
		false
	}
	pub fn is_block_next(&self, block: &Block) -> bool {
		block.index == self.get_len() && block.header.previous_hash == self.get_last_block().unwrap().hash
	}
}
fn get_leading_zeros(bytes: &[u8; 32]) -> u32 {
	let mut result = 0;
	for &byte in bytes.iter() {
		if byte == 0 {
			result += 8;
		} else {
			result += byte.leading_zeros();
			break;
		}
	}
	result
}
