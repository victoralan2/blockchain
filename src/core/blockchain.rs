use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::core::address::P2PKHAddress;
use crate::core::block::{Block, BlockHeader};
use crate::core::parameters::Parameters;
use crate::core::utxo::{UTXO, UTXOSet};
use crate::core::utxo::transaction::Transaction;



#[derive(Clone)]
pub struct BlockChain {
	chain: Vec<Block>,
	pub utxo_set: HashMap<[u8; 32], Vec<UTXO>>,
	pub(crate) mempool: HashSet<Transaction>,
	pub(crate) parameters: Parameters,
}

impl BlockChain {
	pub fn new_empty(parameters: Parameters) -> Self {
		let chain = vec![];
		BlockChain { chain, utxo_set: UTXOSet::genesis(parameters), mempool: Default::default(), parameters }
	}
	pub fn new(chain: Vec<Block>, mempool: HashSet<Transaction>, parameters: Parameters) -> Self {
		BlockChain { chain, utxo_set: HashMap::new(),mempool, parameters }
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
	pub fn get_context(&self) -> String {
		let last_block_hash = self.get_last_block().header.hash;
		hex::encode(last_block_hash) // TODO: Maybe add some more context
	}
	pub fn get_block_at(&self, height: usize) -> Option<&Block> {
		if height < self.chain.len(){
			Some(&self.chain[height])
		} else {
			None
		}
	}
	pub fn get_block_by(&self, hash: [u8; 32]) -> Option<&Block> {
		todo!()
	}

	fn get_last_common_block(&self, others: &Vec<[u8; 32]>) -> Option<&Block> {
		for &other in others.iter() {
			if let Some(block) = self.get_block_by(other) {
				return Some(block);
			}
		}
		None
	}
	pub fn get_blocks(&self, others: &Vec<[u8; 32]>) -> Vec<[u8; 32]> {
		if let Some(last_common) = self.get_last_common_block(others){
			let height = last_common.header.height;
			let mut result = vec![];
			const MAX_BLOCKS: usize = 512;
			for b in self.chain.iter().skip(height).take(MAX_BLOCKS) {
				result.push(b.header.hash);
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
			for b in self.chain.iter().skip(height).take(MAX_HEADERS) {
				result.push(b.header.clone());
			}
			return result;
		}
		vec![]
	}
	pub fn get_height(&self) -> usize {
		self.chain.len()
	}
	pub fn get_last_block(&self) -> &Block {
		self.chain.last().unwrap()
	}

	pub fn add_block(&mut self, new_block: &Block) -> bool {
		if new_block.is_valid(self, self.get_height()) {
			// Todo: some more checks and add block to blockchain
			// Todo: build up the utxo set. PROBABLY DONE
			for tx in &new_block.transactions {
				self.mempool.retain(|t2| tx.eq(t2));
				for input in &tx.input_list {
					if let Some(utxo_list) = self.utxo_set.get_mut(&input.prev_txid) {
						utxo_list.remove(input.output_index);
					}
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
				self.utxo_set.insert(tx.id, utxo_list);
			}
			// TODO: Add fees to the fee pool
			self.chain.push(new_block.clone());
			return true;
		}
		false
	}
}
