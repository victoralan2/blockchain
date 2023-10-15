use crate::core::blockchain::BlockChain;
use crate::core::blockdata::{BlockData, CoinBaseTransaction, Transaction};

#[derive(Clone, PartialEq)]
pub struct Block {
	pub index: u32,
	pub previous_hash: [u8; 32],
	pub hash: [u8; 32],
	pub coinbase_transaction: CoinBaseTransaction,
	pub data: Vec<BlockData>,
	pub time: u64,
	pub blockchain_confirmations: u32,
	pub nonce: u64,
}

impl Block {
	pub fn is_valid(&self, blockchain: &BlockChain) -> bool {
		// TODO: CHECK IF COINBASE TRANSACTION IS VALID
		let is_previous_hash_correct = self.previous_hash == blockchain.get_block_at((self.index-1) as usize).hash;
		if !is_previous_hash_correct {
			return false;
		}
		for d in self.data {
			match d {
				BlockData::TX(tx) => {

					// TODO: CHECK IF TIMESTAMP IS ACCEPTABLE
					// TODO: CHECK IF TWO TRANSACTIONS ARE THE SAME
					// TODO: CHECK IF TRANSACTIONS ARE VALID
					// TODO: CHECK IF SENDER HAS THE MONEY TO SEND TRANSACTIONS
					let actual_hash = tx.calculate_hash();
					let does_sender_have_money = blockchain.get_balance_at(tx.sender_address, self.index) >= tx.amount;
					let is_transaction_valid = tx.is_valid();
					let is_hash_valid = tx.hash == actual_hash;
					let is_unique = 1 == self.data.iter()
						.filter(|&d| if let BlockData::TX(tx) = d { tx.calculate_hash() == actual_hash } else { false }).count();

				}
				BlockData::Data(data) => {

				}
			}
		}

		true
	}
	pub fn new(data: Vec<BlockData>, time: u64, index: u32) -> Self {
		let mut block = Block { previous_hash: [0u8; 32], hash: [0u8; 32], coinbase_transaction: CoinBaseTransaction::null(), data, time, blockchain_confirmations: 0, index, nonce: 0 };
		block.update_hash();
		block
	}
	pub fn update_hash(&mut self) {
		// TODO
		todo!()
	}
	pub fn genesis() -> Self {
		// TODO
		todo!()
	}

	pub fn mine(&mut self, difficulty: u8) -> bool {
		while get_leading_zeros(&self.hash) < difficulty as u32 {
			self.nonce+=1;
			self.update_hash();
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
