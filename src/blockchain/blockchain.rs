use std::time::SystemTime;
use crate::blockchain::address::Address;
use crate::blockchain::block::Block;
use crate::blockchain::transaction::{CoinBaseTransaction, Transaction};

#[derive(Clone)]
pub struct BlockChainConfig {
	difficulty: u8,
	reward: f64,
	block_size: usize,
}

#[derive(Clone)]
pub struct BlockChain {
	chain: Vec<Block>,
	pending_transactions: Vec<Transaction>,
	pub configuration: BlockChainConfig
}
impl BlockChain {
	pub fn new_empty(configuration: BlockChainConfig) -> Self {
		let chain = vec![Block::genesis()];
		BlockChain { chain, pending_transactions: Vec::new(), configuration }
	}
	pub fn new(chain: Vec<Block>, pending_transactions: Vec<Transaction>, configuration: BlockChainConfig) -> Self {
		BlockChain { chain, pending_transactions, configuration }
	}
	fn mine_one_block(&mut self, miner: Address) -> Option<Block> {
		if self.pending_transactions.len() >= self.configuration.block_size {
			let end = self.configuration.block_size;
			let mut transaction_slices: Vec<Transaction> = self.pending_transactions
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

	pub fn get_last_block(&self) -> Option<&Block>{
		self.chain.last()
	}
	pub fn add_block(&mut self, new_block: Block) -> bool {
		let mut new_block = new_block;
		if !self.chain.is_empty() {
			new_block.previous_hash = self.get_last_block().unwrap().hash.clone();
		}
		self.chain.push(new_block);
		true
	}
}