use std::collections::HashMap;
use std::time::SystemTime;
use crate::core::address::Address;
use crate::core::block::Block;
use crate::core::blockdata::{BlockData, CoinBaseTransaction};

#[derive(Clone)]
pub struct BlockChainConfig {
	difficulty: u8,
	reward: u64,
	block_size: usize,
	trust_threshold: u32,
}

#[derive(Clone)]
pub struct BlockChain {
	chain: Vec<Block>,
	cache: HashMap<Address, u64>,
	mempool: Vec<BlockData>,
	forks: Vec<(Vec<Block>, usize)>, // Store forks and the index of the diverging block
	configuration: BlockChainConfig,
}
impl BlockChain {
	pub fn new_empty(configuration: BlockChainConfig) -> Self {
		let chain = vec![Block::genesis()];
		BlockChain { chain, cache: Default::default(), mempool: Vec::new(), forks: vec![], configuration }
	}
	pub fn new(chain: Vec<Block>, mempool: Vec<BlockData>, configuration: BlockChainConfig) -> Self {
		BlockChain { chain, cache: Default::default(), mempool, forks: vec![], configuration }
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
				self.chain.len() as u32,
			);

			let coinbase_transaction = CoinBaseTransaction::new(miner, self.configuration.reward);
			new_block.coinbase_transaction = coinbase_transaction;

			if let Some(mut last_block) = self.get_last_block().cloned() {
				last_block.update_hash();
				let hash_val = last_block.hash;
				new_block.previous_hash = hash_val;
				// TODO: node.current_block_mining_hash = new_block.hash;
				if new_block.mine(self.configuration.difficulty) {
					println!("Block mined, nonce to solve PoW: {}", new_block.nonce);
					return Some(new_block);
				}
			}
		}
		None
	}
	pub fn get_block_at(&self, index: usize) -> &Block {
		&self.chain[index]
	}
	pub fn get_len(&self) -> usize {
		self.chain.len()
	}
	pub fn get_last_block(&self) -> Option<&Block>{
		self.chain.last()
	}
	pub fn add_block(&mut self, new_block: Block) -> bool {
		if new_block.is_valid(self) {
			let last_block = self.get_last_block().cloned();

			match last_block {
				Some(last) => {
					if new_block.previous_hash == last.hash {
						// Add the block to the main chain
						self.chain.push(new_block);
					} else {
						// Check if it's a fork
						let mut is_in_a_fork = false;
						for (fork, diverging_index) in &mut self.forks {
							if new_block.previous_hash == fork.last().map(|b| b.hash).unwrap_or([0; 32]) {
								fork.push(new_block);
								is_in_a_fork = true;
								break;
							}
						}

						if !is_in_a_fork {
							// Create a new fork
							let mut fork = vec![new_block];
							let diverging_index = self.chain.iter().position(|block| block.hash == new_block.previous_hash).unwrap_or(0);
							self.forks.push((fork, diverging_index));
						}
					}
				},
				None => {
					// If there are no blocks yet, this is the genesis block
					self.chain.push(new_block);
				}
			}
			self.resolve_forks();
			true
		} else {
			false
		}
	}
	pub fn resolve_forks(&mut self) { // TODO: CALL THIS
		let mut longest_chain_len = self.chain.len();

		// Iterate through the forks to find the longest chain
		let mut new_forks = Vec::new();

		for (fork, diverging_index) in &self.forks {
			let fork_len = fork.len() + diverging_index;
			let difference1 = fork_len as isize - longest_chain_len as isize;
			let difference2 = longest_chain_len as isize - fork_len as isize;

			if difference1 >= self.configuration.trust_threshold as isize { // If the current fork has at least *configuration.trust_threshold* more blocks than the longest
				// This fork is longer, consider it as the new main chain
				longest_chain_len = fork.len() + diverging_index;
				// Update the main chain
				self.chain.remove(*diverging_index);
				self.chain.append(&mut fork.clone());
				// TODO: Update the cache and other necessary state
				// ...
			} else if difference2 <= self.configuration.trust_threshold as isize { // If the longest chain has least than *configuration.trust_threshold* than the current chain
				// This means that fork can still be valid
				new_forks.push((fork.clone(), *diverging_index));
			}
		}

		self.forks = new_forks;
	}
}