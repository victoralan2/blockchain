use num_traits::PrimInt;
use crate::blockchain::transaction::{CoinBaseTransaction, Transaction};

#[derive(Clone)]
pub struct Block {
	pub previous_hash: Vec<u8>,
	pub hash: Vec<u8>,
	pub coinbase_transaction: CoinBaseTransaction,
	pub transactions: Vec<Transaction>,
	pub time: u64,
	pub blockchain_confirmations: u32,
	pub index: u32,
	pub nonce: u64,
}

impl Block {
	pub fn new(transactions: Vec<Transaction>, time: u64, index: u32) -> Self {
		let mut block = Block { previous_hash: vec![], hash: vec![], coinbase_transaction: CoinBaseTransaction::null(), transactions, time, blockchain_confirmations: 0, index, nonce: 0 };
		block.update_hash();
		block
	}
	pub fn update_hash(&mut self) {
		// TODO
		unimplemented!()
	}
	pub fn genesis() -> Self {
		// TODO
		unimplemented!("")
	}

	pub fn mine(&mut self, difficulty: u8) -> bool {
		while leading_zeros_vec_u8(&self.hash) < difficulty as u32 {
			self.nonce+=1;
			self.update_hash();
		}
		true
	}
}

pub fn leading_zeros_vec_u8(vec: &[u8]) -> u32 {
	vec.iter().try_fold(0, |acc, n| {
		if n == &0 {
			Ok(acc + 8)
		} else {
			Err(acc + n.leading_zeros())
		}
	}).unwrap_or_else(|e| e)
}
