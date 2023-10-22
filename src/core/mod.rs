use crate::core::block::Block;
use crate::core::utxo::transaction::Transaction;
use crate::crypto::hash::{hash, mine_hash};

pub mod blockchain;
pub mod block;
pub mod address;
pub mod utxo;

pub trait Hashable {
	fn calculate_hash(&self) -> [u8; 32];
	fn update_hash(&mut self);
}
impl Hashable for Block {
	/// IMPORTANT
	/// CHECK VALIDITY OF DATA BEFORE CALCULATING HASH. HASH DOES NOT CHECK FOR ERRORS IN COHERENCE
	fn calculate_hash(&self) -> [u8; 32]{
		let header = &self.header;
		let merkle_tree = self.calculate_merkle_tree();
		let str = format!("{}.{}.{}.{}.{}", hex::encode(header.previous_hash), header.nonce, hex::encode(merkle_tree), header.miners_address.to_string(), header.time);
		println!("{}", str);
		mine_hash(str.as_bytes()).as_slice().try_into().expect("Unable to convert hash to byte array")
	}
	fn update_hash(&mut self) {
		let merkle_tree = self.calculate_merkle_tree();
		self.header.merkle_root = merkle_tree;
		self.hash = self.calculate_hash();
	}
}
impl Hashable for Transaction {
	/// IMPORTANT
	/// CHECK VALIDITY OF DATA BEFORE CALCULATING HASH. HASH DOES NOT CHECK FOR ERRORS IN COHERENCE
	fn calculate_hash(&self) -> [u8; 32]{
		let str = format!("{}.{}.{}.{}.{}", self.time, self.nonce, self.sender_address.to_string(), self.recipient_address.to_string(), self.amount);
		hash(str.as_bytes())
	}
	fn update_hash(&mut self) {
		self.id = self.calculate_hash();
	}

}