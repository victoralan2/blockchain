use std::collections::HashSet;

use sled::Db;

use crate::core::utxo::transaction::Transaction;
use crate::network::standard::{standard_deserialize, standard_serialize};


#[derive(Clone)]
pub struct MempoolDB {
	mempool: HashSet<Transaction>,
	mempool_db: Db,
}
impl MempoolDB {
	pub fn insert(&mut self, tx: &Transaction) -> bool {
		let data = standard_serialize(&tx).expect("Unable to serialize tx");
		self.mempool.insert(tx.clone());
		let replaced = self.mempool_db.insert(tx.id, data).expect("TODO: panic message").is_some();
		self.mempool_db.flush().expect("Unable to flush mempool");
		replaced
	}
	pub fn remove(&mut self, tx: &Transaction) {
		self.mempool.remove(tx);
		self.mempool_db.remove(tx.id).unwrap();
		self.mempool_db.flush().unwrap();
	}
	pub fn get_map(&self) -> &HashSet<Transaction> {
		&self.mempool
	}
}
impl Default for MempoolDB {
	fn default() -> Self {
		let db = sled::open("./blockchain/mempool-db").unwrap(); // FIXME: Change the file for the actual Db location
		let txs: HashSet<Transaction> = db.iter().filter_map(|tx| {
			match tx {
				Ok((_, tx)) => {
					if let Ok(tx) = standard_deserialize(&tx) {
						Some(tx)
					} else {
						None
					}
				}
				Err(_) => {None}
			}
		}).collect();
		Self {
			mempool: txs,
			mempool_db: db,
		}
	}
}