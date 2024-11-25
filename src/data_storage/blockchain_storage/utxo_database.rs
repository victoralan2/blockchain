use std::collections::{HashMap, HashSet};
use std::fs::File;

use sled::Db;
use crate::core::address::P2PKHAddress;
use crate::core::parameters::Parameters;
use crate::core::utxo::UTXO;
use crate::data_storage::BaseDirectory;
use crate::data_storage::blockchain_storage::undo_items::UndoTransaction;
use crate::network::standard::{standard_deserialize, standard_serialize};

#[derive(Clone)]
pub struct UTXODB {
	utxo_set: Db,
}
impl UTXODB {
	// TEST
	pub fn genesis(parameters: Parameters) -> Self {
		let utxo_directory = format!("{}/blockchain/utxo-set/", BaseDirectory::get_base_directory());
		let utxo_set = sled::open(utxo_directory).expect("Unable to open / create utxo set");
		Self {
			utxo_set,
		}
	}
	
	/// Adds to the UTxO set the given list of sorted UTxOs associated with the given transaction id
	pub fn insert(&self, txid: &[u8; 32], utxo_list: HashSet<UTXO>) {
		let utxo_data = standard_serialize(&utxo_list).expect("Unable to serialize UTXO list");
		self.utxo_set.insert(txid, utxo_data).expect("Unable to insert to UTXO set");
		self.utxo_set.flush().expect("Unable to flush");
	}
	/// Returns all the UTxOs from the given transaction id
	pub fn get(&self, txid: &[u8; 32]) -> Option<HashSet<UTXO>> {
		let data = self.utxo_set.get(txid).expect("Unable to get list from UTXO set")?;
		let utxo_list: Vec<UTXO> = standard_deserialize(&data).map_err(|err| log::error!("Unable to deserialize UTXO set content: {}", err)).unwrap();
		Some(utxo_list.iter().copied().collect())
	}
	/// Removes all the UTXOs related with some TxID
	pub fn remove(&self, txid: &[u8; 32]) {
		self.utxo_set.remove(txid).expect("Unable to remove TxID");
		self.utxo_set.flush().expect("Unable to flush");
	}
	pub fn undo_transaction(&self, undo_transaction: &UndoTransaction) {
		self.remove(&undo_transaction.original_tx_id);
		for (txid, utxo) in undo_transaction.removed_utxos.clone() {
			if let Some(mut tx_data) = self.get(&txid) {
				tx_data.insert(utxo);
				self.insert(&txid, tx_data);
			} else {
				self.insert(&txid, HashSet::from([utxo]))
			}
		}
	}
	/// Returns all the UTxOs from the given transaction id
	pub fn get_utxos_from_address(&self, address: P2PKHAddress) -> Vec<UTXO> {
		let mut final_utxos = vec![];
		for tx in self.utxo_set.iter() {
			if let Ok((_, utxos)) = tx {
				let utxos: Vec<UTXO> = standard_deserialize(&utxos).map_err(|err| log::error!("Unable to deserialize UTXO set content: {}", err)).unwrap();
				for u in utxos {
					if u.recipient_address == address {
						final_utxos.push(u);
					}
				}
			}
		}
		final_utxos
	}
	/// Removes an output of the given txid and with the given index.
	/// Indexes of all UTxOs will be checked instead of removing the nth one, this is because a previous index could have been removed before.
	pub fn remove_utxo(&self, txid: &[u8; 32], index: usize) {
		// TODO: Check that in one block there are not two transactions that use the same input (or the same input in the same transaction)
		// TODO: Max two outputs for one transaction
		if let Some(mut utxo_list) = self.get(txid) {
			for this_utxo in utxo_list.clone() {
				if this_utxo.output_index == index {
					utxo_list.remove(&this_utxo);
				}
			}
			
			if !utxo_list.is_empty() { // If the utxo_list is empty just don't bother putting it in again (we remove it)
				self.insert(txid, utxo_list);
			} else {
				self.remove(txid);
			}
			self.utxo_set.flush().expect("Unable to flush");
		}
	}
	pub fn flush(&self) -> anyhow::Result<()> {
		self.utxo_set.flush()?;
		Ok(())
	}
}
impl Default for UTXODB {
	fn default() -> Self {
		let db = sled::open(format!("{}/blockchain/utxo-set", BaseDirectory::get_base_directory())).unwrap(); // FIXME: Change the file for the actual Db location
		Self {
			utxo_set: db,
		}
	}
}