use std::collections::HashSet;

use sled::Db;

use crate::core::utxo::transaction::Transaction;
use crate::network::standard::standard_serialize;

pub struct MempoolDB {
	mempool: HashSet<Transaction>,
	mempool_db: Db,
}
impl MempoolDB {
	pub fn insert(&mut self, tx: Transaction) {
		let data = standard_serialize(&tx).expect("Unable to serialize tx");
		self.mempool_db.insert(&tx.id, data).expect("TODO: panic message");
		self.mempool_db.flush().expect("Unable to flush mempool");
		self.mempool.insert(tx);
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
					if let Ok(tx) = serde_json::from_slice(&tx) {
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