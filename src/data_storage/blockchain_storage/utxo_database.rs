use std::collections::HashMap;
use std::fs::File;

use sled::Db;

use crate::core::parameters::Parameters;
use crate::core::utxo::UTXO;
use crate::data_storage::BaseDirectory;
use crate::network::standard::{standard_deserialize, standard_serialize};

#[derive(Clone)]
pub struct UTXODB {
	utxo_set: Db,
}
impl UTXODB {
	pub fn genesis(parameters: Parameters) -> Self {
		let utxo_directory = format!("{}/blockchain/utxo-set/", BaseDirectory::get_base_directory());
		let mut utxo_set = sled::open(utxo_directory).expect("Unable to open / create utxo set");
		// TODO: Add genesis distribution in here
		Self {
			utxo_set,
		}
	}
	
	/// Adds to the UTxO set the given list of sorted UTxOs associated with the given transaction id
	pub fn insert(&self, txid: &[u8; 32], utxo_list: Vec<UTXO>) {
		let utxo_data = standard_serialize(&utxo_list).expect("Unable to serialize UTXO list");
		self.utxo_set.insert(txid, utxo_data).expect("Unable to insert to UTXO set");
		self.utxo_set.flush().expect("Unable to flush");
	}
	/// Returns all the UTxOs in order from the given transaction id
	pub fn get(&self, txid: &[u8; 32]) -> Option<Vec<UTXO>> {
		let data = self.utxo_set.get(txid).expect("Unable to get list from UTXO set")?;
		let utxo_list = standard_deserialize(&data).map_err(|err| log::error!("Unable to deserialize UTXO set content: {}", err)).unwrap();
		Some(utxo_list)
	}
	/// Removes all the UTXOs related with some TxID
	pub fn remove(&self, txid: &[u8; 32]) {
		self.utxo_set.remove(txid).expect("Unable to remove TxID");
		self.utxo_set.flush().expect("Unable to flush");
	}
	
	
	/// Removes an output of the given txid and with the given index.
	/// Indexes of all UTxOs will be checked instead of removing the nth one, this is because a previous index could have been removed before.
	pub fn remove_utxo(&self, txid: &[u8; 32], index: usize) {
		// TODO: Check that in one block there are not two transactions that use the same input (or the same input in the same transaction)
		// TODO: Max two outputs for one transaction
		if let Some(mut utxo_list) = self.get(txid) {
			for this_utxo in utxo_list.clone() {
				if this_utxo.output_index == index {
					let index = utxo_list
						.iter()
						.position(|utxo| *utxo == this_utxo)
						.unwrap();
					utxo_list.remove(index);
				}
			}
			self.remove(txid);
			if !utxo_list.is_empty() { // If the utxo_list is empty just don't bother putting it in again (we remove it)
				self.insert(txid, utxo_list);
			}
			self.utxo_set.flush().expect("Unable to flush");
		}
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