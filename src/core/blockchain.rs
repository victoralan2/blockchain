use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::SystemTime;

use crate::core::address::P2PKHAddress;
use crate::core::block::Block;
use crate::core::blockdata::BlockData;
use crate::core::Hashable;

#[derive(Clone)]
pub struct BlockChainConfig {
	pub(crate) difficulty: u8,
	pub(crate) reward: u64,
	pub(crate) block_size: usize,
	pub(crate) trust_threshold: u32,
	pub(crate) max_data_size: usize,
	pub(crate) data_fee_multiplier: f64,
	pub(crate) transaction_fee_multiplier: f64,
	pub(crate) max_transaction_fee: u64,
}

#[derive(Clone)]
pub struct BlockChain {
	chain: Vec<Block>,
	cache: HashMap<P2PKHAddress, u64>,
	mempool: Vec<BlockData>,
	pub configuration: BlockChainConfig,
}

impl BlockChain {
	pub fn new_empty(configuration: BlockChainConfig) -> Self {
		let chain = vec![Block::genesis()];
		BlockChain { chain, cache: Default::default(), mempool: vec![], configuration }
	}
	pub fn new(chain: Vec<Block>, mempool: Vec<BlockData>, configuration: BlockChainConfig) -> Self {
		BlockChain { chain, cache: Default::default(), mempool, configuration }
	}
	pub fn replace(&mut self, new: BlockChain) {
		self.cache = new.cache;
		self.chain = new.chain;
	}
	pub fn truncate(&mut self, index: usize) {
		self.chain.truncate(index);
	}
	/// Validates and adds the blockdata to the memory pool if valid.
	/// Returns whether the it was added or not
	pub fn add_data_to_mempool(&mut self, data: BlockData) -> bool{
		let is_valid = data.is_valid(self);
		if is_valid {
			self.mempool.push(data);
		}
		is_valid
	}
	pub fn mine_one_block(&mut self, miner: P2PKHAddress) -> Option<Block> {
		if self.mempool.len() >= self.configuration.block_size {
			let block_size = self.configuration.block_size;
			let mut transaction_slices: Vec<BlockData> = self.mempool
				.iter()
				.take(block_size).cloned()
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

			if let Some(mut last_block) = self.get_last_block().cloned() {

				last_block.update_hash();
				let hash_val = last_block.hash;
				new_block.header.previous_hash = hash_val;

				let mut keep_mining = Arc::new(AtomicBool::new(true)); // TODO: MAKE MINING CANCELLABLE

				if new_block.mine(self.configuration.difficulty, keep_mining.clone()) && new_block.is_valid(self) {
					println!("Block mined, nonce to solve PoW: {}", new_block.header.nonce);
					return Some(new_block);
				}
			}
		}
		None
	}
	pub fn get_balance_at(&self, address: &P2PKHAddress, index: usize) -> u64 {
		let addr = address.address;
		let trusted_chain = &self.chain[..(self.chain.len() - self.configuration.trust_threshold as usize)];
		let mut balance = 0u64;
		for block in &trusted_chain[..index] {
			if block.header.miners_address.address == addr {
				balance += block.calculate_reward(&self.configuration);
			}
			for t in block.data
				.iter()
				.filter_map(|d| if let BlockData::TX(tx) = d { Some(tx) } else { None }) // Filters out all non transactions
			{
				if t.recipient_address.address == addr {
					balance += t.amount;
				}
				if t.sender_address.address == addr {
					if balance < t.amount {
						panic!("BALANCE IS NEGATIVE")
					}
					balance -= t.amount;
				}
			}
		}
		balance
	}
	pub fn get_balance(&self, address: &P2PKHAddress) -> u64 {
		self.get_balance_at(address, self.chain.len())
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

	/// Returns false if the blockchain is needed
	pub fn add_block(&mut self, new_block: Block) -> bool {
		let last_block = self.get_last_block().cloned();
		if let Some(last_block) = last_block {
			if last_block.header.previous_hash == new_block.hash && new_block.is_valid(self) {
   					// Todo: some more checks and add block to blockchain
   					for d in &new_block.data {
   						self.mempool.retain(|d2| d.eq(d2))
   					}
   					self.chain.push(new_block);
   					return true;
   				}
		}
		false
	}
}