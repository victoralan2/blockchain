use std::collections::HashSet;

use crate::core::block::{Block, BlockHeader};
use crate::core::parameters::Parameters;
use crate::core::utxo::transaction::Transaction;
use crate::core::utxo::UTXO;
use crate::data_storage::blockchain_storage::chain_database::ChainDB;
use crate::data_storage::blockchain_storage::utxo_database::UTXODB;

#[derive(Clone)]
pub struct BlockChain {
	chain: ChainDB,
	pub utxo_set: UTXODB,
	pub(crate) mempool: HashSet<Transaction>,
	pub(crate) parameters: Parameters,
}

impl BlockChain {
	pub fn init(parameters: Parameters) -> Self {
		let chain = ChainDB::default();
		BlockChain { chain, utxo_set: UTXODB::genesis(parameters), mempool: Default::default(), parameters }
	}
	pub fn get_utxo_list(&self, txid: &[u8; 32]) -> Option<Vec<UTXO>>{
		self.utxo_set.get(txid)
	}
	
	/// Validates and adds the transaction to the memory pool if valid.
	/// Returns whether the tx was added or not
	pub fn add_transaction_to_mempool(&mut self, tx: &Transaction) -> bool{
		let is_valid = tx.is_valid(self);
		if is_valid {
			self.mempool.insert(tx.clone()) && is_valid
		} else {
			false
		}
	}
	pub fn get_context(&self) -> String {
		let last_block_hash = self.get_last_block().header.hash;
		hex::encode(last_block_hash) // TODO: Maybe add some more context
	}
	pub fn get_block_at(&self, height: usize) -> Option<Block> {
		self.chain.get_block_by_height(height)
	}
	pub fn get_block_by(&self, hash: [u8; 32]) -> Option<&Block> {
		todo!()
	}

	fn get_last_common_block(&self, others: &Vec<[u8; 32]>) -> Option<Block> {
		for &other in others.iter() {
			if let Some(block) = self.chain.get_block(other) {
				return Some(block);
			}
		}
		None
	}
	pub fn get_blocks(&self, others: &Vec<[u8; 32]>) -> Vec<[u8; 32]> {
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
			return result;
		}
		vec![]
	}
	pub fn get_headers(&self, others: &Vec<[u8; 32]>) -> Vec<BlockHeader> {
		if let Some(last_common) = self.get_last_common_block(others){
			let height = last_common.header.height;
			let mut result = vec![];
			const MAX_HEADERS: usize = 2048;
			for i in 0..MAX_HEADERS {
				if let Some(block) = self.chain.get_block_by_height(height + i) {
					result.push(block.header);
				} else {
					break
				}
			}
			return result;
		}
		vec![]
	}
	pub fn get_height(&self) -> usize {
		self.chain.get_length() - 1
	}
	pub fn get_last_block(&self) -> Block {
		self.chain.get_best_block().expect("Chain was empty")
	}

	pub fn add_block(&mut self, new_block: &Block) -> bool {
		// if new_block.is_valid(self) { // TODO: In this line maybe test for the other cases too
		if true { // FIXME
			// Todo: some more checks and add block to blockchain
			// Todo: Check if block has higher VRF and it does not diverge more than 3k/f
			// Todo: build up the utxo set. PROBABLY DONE
			for tx in &new_block.transactions {
				self.mempool.remove(tx);
				for input in &tx.input_list {
					self.utxo_set.remove_utxo(&input.prev_txid, input.output_index);
				}
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
				self.utxo_set.insert(&tx.id, utxo_list);
			}
			// TODO: Add fees to the fee pool
			self.chain.push_block_to_end(&new_block.clone()).map_err(|e| log::error!("Unable to write block to database: {}", e)).unwrap();
			return true;
		}
		false
	}
}
