use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use rand::thread_rng;
use rand_core::RngCore;
use crate::core::blockchain::BlockChain;
use crate::core::blockdata::{BlockData, CoinBaseTransaction};
use crate::core::Hashable;

#[derive(Clone, PartialEq)]
pub struct BlockHeader {
	pub previous_hash: [u8; 32],
	pub time: u64,
	pub nonce: u64,
	pub merkle_root: [u8; 32],
}

#[derive(Clone, PartialEq)]
pub struct Block {
	pub index: usize,
	pub header: BlockHeader,
	pub hash: [u8; 32],
	pub coinbase_transaction: CoinBaseTransaction,
	pub data: Vec<BlockData>,
}

impl Block {
	pub fn new(data: Vec<BlockData>, time: u64, index: usize) -> Self {
		let block_header = BlockHeader{
			previous_hash:  [0u8; 32],
			time,
			nonce: 0,
			merkle_root: [0u8; 32],
		};
		let mut block = Block {hash: [0u8; 32], coinbase_transaction: CoinBaseTransaction::null(), data, index, header: block_header };
		block.update_hash();
		block
	}
	pub fn genesis() -> Self {
		let block_header = BlockHeader{
			previous_hash:  [0u8; 32],
			time: 0u64,
			nonce: 0,
			merkle_root: [0u8; 32],
		};
		Block {
			hash: [0u8; 32],
			coinbase_transaction: CoinBaseTransaction::null(),
			data: vec![],
			index: 0,
			header: block_header,
		}
	}
	pub fn mine(&mut self, difficulty: u8, keep_mining: Arc<AtomicBool>) -> bool {
		self.header.nonce = thread_rng().next_u64();
		while get_leading_zeros(&self.hash) < difficulty as u32 && keep_mining.load(Ordering::Relaxed) {
			self.header.nonce += 1;
			self.update_hash();
		}
		keep_mining.load(Ordering::Relaxed) // Return true if the block was mined, false if the opperation was cancelled
	}
	pub fn is_valid(&self, blockchain: &BlockChain) -> bool {
		// TODO: CHECK IF COINBASE TRANSACTION IS VALID
		// TODO: CHECK IF THE TRANSACTIONS ARE THE ONES AT MEMPOOL AND THEN REMOVE
		if let Some(previous) = blockchain.get_block_at((self.index - 1)) {
			let is_previous_hash_correct = self.header.previous_hash == previous.hash;
			if !is_previous_hash_correct {
				return false;
			}
		} else {
			return false;
		}
		for d in &self.data {
			match d {
				BlockData::TX(tx) => {
					// TODO: CHECK IF TIMESTAMP IS ACCEPTABLE
					// TODO: CHECK IF TWO TRANSACTIONS ARE THE SAME
					// TODO: CHECK IF TRANSACTIONS ARE VALID
					// TODO: CHECK IF SENDER HAS THE MONEY TO SEND TRANSACTIONS
					let actual_hash = tx.calculate_hash();
					let does_sender_have_money = blockchain.get_balance_at(&tx.sender_address, self.index) >= tx.amount;
					let is_transaction_valid = tx.is_valid();
					let is_hash_valid = tx.hash == actual_hash;
					let is_unique = 1 == self.data.iter()
						.filter(|&d| if let BlockData::TX(tx) = d { tx.calculate_hash() == actual_hash } else { false }).count();
				}
				BlockData::Data(data) => {}
			}
		}

		true
	}
}

pub fn get_leading_zeros(vec: &[u8]) -> u32 {
	vec.iter().try_fold(0, |acc, n| {
		if n == &0 {
			Ok(acc + 8)
		} else {
			Err(acc + n.leading_zeros())
		}
	}).unwrap_or_else(|e| e)
}
