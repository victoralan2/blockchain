use std::collections::{HashSet};
use crate::core::utxo::transaction::Transaction;


#[derive(Clone, Default)]
pub struct Mempool {
	pool: HashSet<Transaction>,
	max_length: usize,
}

impl Mempool {
	/// Creates a new mempool with a maximum size in Megabytes of max_size knowing that the max transaction size is max_transaction_size in bytes
	pub fn new(max_size: usize, max_transaction_size: usize) -> Self {
		const BYTES_IN_A_MEGABYTE: usize = 2516582400;
		Self {
			pool: Default::default(),
			max_length: max_size * BYTES_IN_A_MEGABYTE / max_transaction_size,
		}
	}
	pub fn insert(&mut self, tx: &Transaction) -> anyhow::Result<bool, &str> {
		let length = self.pool.len();
		if length >= self.max_length {
			return Err("The mempool is already full");
		}
		Ok(self.pool.insert(tx.clone()))
	}
	pub fn remove(&mut self, tx: &Transaction) {
		self.pool.remove(tx);
	}
	pub fn get_map(&self) -> &HashSet<Transaction> {
		&self.pool
	}
}