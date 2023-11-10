use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool};
use std::sync::{Arc};
use std::time::SystemTime;
use rand::thread_rng;
use rand_core::RngCore;
use serde::{Deserialize, Serialize};

use crate::core::address::P2PKHAddress;
use crate::core::block::{Block};
use crate::core::{Hashable, is_smaller};
use crate::core::utxo::transaction::Transaction;
use crate::core::utxo::{UTXO, UTXOSet};
use crate::core::utxo::coinbase::CoinbaseTransaction;

#[derive(Clone, Serialize, Deserialize, Copy)]
pub struct BlockChainConfig {
	pub(crate) target_value: [u8; 32], // Todo: Make this value automatically update like in btc
	pub(crate) reward: u64,
	pub(crate) block_size: usize,
	pub(crate) trust_threshold: u32,
	pub(crate) transaction_fee_multiplier: f64,
	pub(crate) max_transaction_fee: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BlockChain {
	chain: Vec<Block>,
	pub utxo_set: HashMap<[u8; 32], Vec<UTXO>>,
	mempool: HashSet<Transaction>,
	pub configuration: BlockChainConfig,
}

impl BlockChain {
	pub fn new_empty(configuration: BlockChainConfig) -> Self {
		let chain = vec![Block::genesis()];
		BlockChain { chain, utxo_set: UTXOSet::genesis(configuration), mempool: Default::default(), configuration }


	}
	pub fn new(chain: Vec<Block>, mempool: HashSet<Transaction>, configuration: BlockChainConfig) -> Self {
		BlockChain { chain, utxo_set: HashMap::new(),mempool, configuration }
	}
	pub fn clear(&mut self) {
		self.chain = vec![Block::genesis()];
		self.utxo_set.clear();
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
	pub fn mine_one_block(&mut self, miner: P2PKHAddress, keep_mining: Arc<AtomicBool>) -> Option<Block> {
		if self.mempool.len() >= self.configuration.block_size {
			let block_size = self.configuration.block_size;
			let transaction_slices: Vec<Transaction> = self.mempool
				.iter()
				.take(block_size)
				.cloned()
				.collect::<Vec<_>>();




			if let Some(mut last_block) = self.get_last_block().cloned() {
				let utxo_transaction = CoinbaseTransaction::genesis();

				let mut new_block = Block::new(
					transaction_slices.clone(),
					SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64,
					self.chain.len(),
					last_block.header.previous_hash,
					utxo_transaction);
				new_block.header.coinbase_transaction = CoinbaseTransaction::create(miner, new_block.calculate_reward(self.configuration));
				last_block.update_hash();

				// TODO: MAKE MINING CANCELLABLE
				new_block.header.nonce = thread_rng().next_u64();
				new_block.update_hash();
				let mut i = 0;
				const CHECK_RATE: u32 = 1000;
				loop {
					new_block.header.nonce += 1;
					if is_smaller(&new_block.calculate_hash(), &self.configuration.target_value) {
						new_block.update_hash();
					} else {
						println!("Block mined, nonce to solve PoW: {}", new_block.header.nonce);
						return Some(new_block);
					}

					if i > CHECK_RATE {
						i=0;
						if keep_mining.load(std::sync::atomic::Ordering::Relaxed) || !new_block.is_valid(&self) {
							break
						}
					}
					i+=1;
				}

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
						time: tx.time,
					};
					utxo_list.push(utxo);
				}
				self.utxo_set.insert(tx.id, utxo_list);
			}
			let coinbase_utxo = UTXO {
				txid: new_block.header.coinbase_transaction.id,
				output_index: 0,
				amount: new_block.header.coinbase_transaction.output.amount,
				recipient_address: new_block.header.coinbase_transaction.output.address,
				time: new_block.header.coinbase_transaction.time,
			};
			if let Some(coinbase_list) = self.utxo_set.get_mut(&[0u8; 32]) {
				coinbase_list.push(coinbase_utxo);
			} else {
				self.utxo_set.insert([0u8; 32], vec![coinbase_utxo]);
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
