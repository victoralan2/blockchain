use std::collections::HashSet;
use crate::core::block::{Block, BlockHeader};
use crate::core::parameters::Parameters;
use crate::core::utxo::transaction::Transaction;
use crate::core::utxo::UTXO;
use crate::data_storage::blockchain_storage::chain_database::ChainDB;
use crate::data_storage::blockchain_storage::mempool_database::{Mempool};
use crate::data_storage::blockchain_storage::undo_items::{UndoBlock, UndoTransaction};
use crate::data_storage::blockchain_storage::utxo_database::UTXODB;
use crate::data_storage::node_config_storage::node_config::NodeConfig;

#[derive(Clone)]
pub struct BlockChain {
	chain: ChainDB,
	pub utxo_set: UTXODB,
	pub(crate) mempool: Mempool,
	pub(crate) parameters: Parameters,
}

impl BlockChain {
	pub fn init(parameters: Parameters, config: &NodeConfig) -> Self {
		let chain = ChainDB::default();
		BlockChain { chain, utxo_set: UTXODB::genesis(parameters), mempool: Mempool::new(config.max_mempool_size_mb, parameters.network_parameters.max_tx_size), parameters }
	}
	pub fn get_utxo_list(&self, txid: &[u8; 32]) -> Option<HashSet<UTXO>>{
		self.utxo_set.get(txid)
	}
	
	/// Validates and adds the transaction to the memory pool if valid.
	/// Returns whether the tx was added or not
	pub fn add_transaction_to_mempool(&mut self, tx: &Transaction) -> bool {
		let is_valid = tx.is_valid(self);
		if is_valid {
			let insert_result = self.mempool.insert(tx);
			insert_result.is_ok_and(|b| b)
		} else {
			false
		}
	}
	pub fn get_context(&self) -> String { // TODO: If this function is still usefull, make it return a struct not a string
		let last_block_hash = self.get_last_block().header.hash;
		hex::encode(last_block_hash) // TODO: Maybe add some more context
	}
	pub fn get_block_at(&self, height: usize) -> Option<Block> {
		self.chain.get_block_by_height(height)
	}
	pub fn get_block_hash_at(&self, height: usize) -> Option<[u8; 32]> {
		self.chain.get_block_hash_by_height(height)
	}
	pub fn get_block_by(&self, hash: [u8; 32]) -> Option<Block> {
		self.chain.get_block(hash)
	}
	pub fn flush(&mut self) {
		self.chain.flush().unwrap();
		self.utxo_set.flush().unwrap();
	}
	fn get_last_common_block(&self, others: &Vec<[u8; 32]>) -> Option<Block> {
		for &other in others.iter() {
			if let Some(block) = self.chain.get_block(other) {
				return Some(block);
			}
		}
		None
	}
	pub fn get_blocks(&self, others: &Vec<[u8; 32]>) -> Option<Vec<[u8; 32]>> {
		if let Some(last_common) = self.get_last_common_block(others) {
			let height = last_common.header.height;
			let mut result = vec![];
			const MAX_BLOCKS: usize = 512;
			for i in 0..MAX_BLOCKS {
				if let Some(block) = self.chain.get_block_by_height(height + i) {
					result.push(block.header.hash);
				} else {
					break
				}
			}
			return Some(result);
		}
		None
	}
	
	/// Returns the headers continuing from the last common headers (Up to **MAX_HEADERS**).
	/// It also returns true if all headers have benn returned or false if **MAX_HEADERS** has been reached 
	/// The first block in the vector should be the last common block
	pub fn get_headers(&self, others: &Vec<[u8; 32]>) -> Option<(Vec<BlockHeader>, bool)> {
		if let Some(last_common) = self.get_last_common_block(others){
			let height = last_common.header.height;
			let mut result = vec![];
			const MAX_HEADERS: usize = 1024;
			for i in 0..MAX_HEADERS {
				if let Some(block) = self.chain.get_block_by_height(height + i) {
					result.push(block.header);
				} else {
					break
				}
			}

			return Some((result, self.get_height() < height + MAX_HEADERS));
		}
		None
	}
	/// The height of the last block
	pub fn get_height(&self) -> usize {
		self.chain.get_length() - 1
	}
	pub fn get_last_block(&self) -> Block {
		self.chain.get_best_block().expect("Chain was empty")
	}
	pub fn print_debug(&self) {
		self.chain.print_debug();
	}
	pub fn add_block(&mut self, new_block: &Block) -> bool {
		// TODO: Add a reward to the miner (add a UTXO with miner's address)
		if self.is_block_valid(new_block) { // TODO: In this line maybe test for the other cases too
			// Todo: some more checks and add block to blockchain
			// Todo: Check if block has higher VRF and it does not diverge more than 3k/f
			// Todo: build up the utxo set. PROBABLY DONE
		
			let mut undo_block = UndoBlock {
				height: new_block.header.height,
				original_hash: new_block.header.hash,
				undo_transactions: vec![],
			};
			for tx in &new_block.transactions {
				self.mempool.remove(tx);
		
				let mut undo_transaction = UndoTransaction {
					original_tx_id: tx.id,
					removed_utxos: vec![],
				};
		
				for input in &tx.input_list {
					if let Some(utxos) = self.utxo_set.get(&input.prev_txid) {
						// This finds the utxo that the input was referring to
						if let Some(&utxo) = utxos.iter().find(|utxo| utxo.output_index == input.output_index) {
							undo_transaction.removed_utxos.push((input.prev_txid, utxo)); // Add it to the undo transaction
						}
					}
					// Remove the utxo from the UTXOset
					self.utxo_set.remove_utxo(&input.prev_txid, input.output_index);
				}
				// Add the undo transaction to the undo block
				undo_block.undo_transactions.push(undo_transaction);
		
				let mut utxo_list = Vec::new();
				for (i, output) in tx.output_list.iter().enumerate() {
					let utxo = UTXO{
						txid: tx.id,
						output_index: i,
						amount: output.amount,
						recipient_address: output.address,
					};
					utxo_list.push(utxo);
				}
			}
			// TODO: Add the
		
		
			self.chain.push_block_to_end(&new_block.clone(), &undo_block).expect("Unable to write block to database");
			return true;
		}
		false
	}
	pub fn is_block_valid(&self, block: &Block) -> bool {
		// TODO

		let height = self.get_height();
		let is_block_correct = block.is_correct();
		if !is_block_correct {
			return false
		}

		// TODO: Verify PoW
		

		// TODO: DOING: I was trying to make so that when the block can replace the last one is valid. Problem: Transactions are bitches bc last block interfeers with that and SHIT FUCK
		for tx in &block.transactions {
			if !tx.is_valid(self) {
				return false
			}
		}

		if let Some(previous) = self.get_block_at(height - 1) {
			let is_previous_hash_correct = block.header.previous_hash == previous.header.hash;
			if !is_previous_hash_correct {
				return false;
			}
		}

		let is_height_correct = block.header.height == self.get_height();
		if !is_height_correct {
			return false
		}
		true
		// TODO: NOW
	}
	pub fn undo_block(&mut self, block_hash: &[u8; 32]) -> anyhow::Result<bool> {
		if block_hash == &self.get_last_block().header.hash {
			if let Some(undo_block) = self.chain.get_undo_block(block_hash)? {
				self.chain.undo_block(block_hash)?;
				for tx in undo_block.undo_transactions {
					self.utxo_set.undo_transaction(&tx);
				}
				return Ok(true)
			}
		}
		Ok(false)
	}
	
	/// Undoes the blockchain until block_hash (exclusive) is the best block, returns all the blocks that have been undone
	/// Returns None if block_hash does not exist
	pub fn undo_until(&mut self, block_hash: [u8; 32]) -> Option<Vec<Block>> {
		if self.get_block_by(block_hash).is_none() {
			return None;
		}

		// TODO: Maybe add a limit to this function as it is very very memory expensive, (it holds all blocks in memory) (maybe optimize in some way??)
		let mut undone_blocks = vec![];
		let mut last_block = self.get_last_block();
		while last_block.header.hash != block_hash {
			let _ = self.undo_block(&self.get_last_block().header.hash).expect("Unable to undo block");
			undone_blocks.push(last_block);
			last_block = self.get_last_block();
		}
		Some(undone_blocks)
	}
}
