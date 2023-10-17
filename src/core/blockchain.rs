use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::channel;
use std::time::SystemTime;
use crate::core::address::Address;
use crate::core::block::Block;
use crate::core::blockdata::{BlockData, CoinBaseTransaction};
use crate::core::Hashable;

#[derive(Clone)]
pub struct BlockChainConfig {
	pub(crate) difficulty: u8,
	pub(crate) reward: u64,
	pub(crate) block_size: usize,
	pub(crate) trust_threshold: u32,
}

#[derive(Clone)]
pub struct BlockChain {
	chain: Vec<Block>,
	cache: HashMap<Address, u64>,
	mempool: Vec<BlockData>,
	configuration: BlockChainConfig,
}

impl BlockChain {
	pub fn new_empty(configuration: BlockChainConfig) -> Self {
		let chain = vec![Block::genesis()];
		BlockChain { chain, cache: Default::default(), mempool: Vec::new(), configuration }
	}
	pub fn new(chain: Vec<Block>, mempool: Vec<BlockData>, configuration: BlockChainConfig) -> Self {
		BlockChain { chain, cache: Default::default(), mempool, configuration }
	}
	fn mine_one_block(&mut self, miner: Address) -> Option<Block> {
		if self.mempool.len() >= self.configuration.block_size {
			let end = self.configuration.block_size;
			let mut transaction_slices: Vec<BlockData> = self.mempool
				.iter()
				.take(end).cloned()
				.collect::<Vec<_>>();

			let mut new_block = Block::new(
				transaction_slices.clone(),
				SystemTime::now()
					.duration_since(SystemTime::UNIX_EPOCH)
					.unwrap()
					.as_secs(),
				self.chain.len(),
			);

			let coinbase_transaction = CoinBaseTransaction::new(miner, self.configuration.reward);
			new_block.coinbase_transaction = coinbase_transaction;

			if let Some(mut last_block) = self.get_last_block().cloned() {
				last_block.update_hash();
				let hash_val = last_block.hash;
				new_block.header.previous_hash = hash_val;
				// TODO: node.current_block_mining_hash = new_block.hash;
				let mut keep_mining = Arc::new(AtomicBool::new(true));
				if new_block.mine(self.configuration.difficulty, keep_mining.clone()) {
					println!("Block mined, nonce to solve PoW: {}", new_block.header.nonce);
					return Some(new_block);
				}
			}
		}
		None
	}
	pub fn get_balance_at(&self, address: &Address, index: usize) -> u64 {
		let addr = address.address.clone();
		let trusted_chain = &self.chain[..(self.chain.len() - self.configuration.trust_threshold as usize)];
		let mut balance = 0u64;
		for block in &trusted_chain[..index] {
			if block.coinbase_transaction.receiver.address == addr {
				balance += block.coinbase_transaction.amount;
			}
			for t in block.data
				.iter()
				.filter_map(|d| if let BlockData::TX(tx) = d { Some(tx) } else { None }) // Filters out all non transactions
			{
				if t.recipient_address.address == addr {
					balance += t.amount;
				}
			}
		}
		balance
	}
	pub fn get_balance(&self, address: &Address) -> u64 {
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
		// TODO: CHECK IF THE TRANSACTIONS ARE THE ONES AT MEMPOOL AND THEN REMOVE
		let last_block = self.get_last_block().cloned();
		if let Some(last_block) = last_block {
			if last_block.header.previous_hash == new_block.hash {
				if last_block.is_valid(self) {
					// Todo: some more checks and add block to blockchain
					self.chain.push(new_block);
				}
			} else if new_block.index > last_block.index {
				// Todo: ask for blockchain
				return false;
			}
		}
		true
	}
}