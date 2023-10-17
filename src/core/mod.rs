use crate::core::block::Block;
use crate::core::blockdata::Transaction;
use crate::crypto::hash::{hash, mine_hash};

pub mod blockchain;
pub mod block;
pub mod address;
pub mod blockdata;

pub trait Hashable {
	fn calculate_hash(&self) -> [u8; 32];
	fn update_hash(&mut self);
}
impl Hashable for Block {
	fn calculate_hash(&self) -> [u8; 32]{
		let header = self.header;
		let merkle_tree = self.calculate_merkle_root();
		let str = format!("{}.{}.{:?}.{:?}", header.previous_hash, header.time, merkle_tree, header.nonce);
		mine_hash(str.as_bytes()).as_slice().try_into().expect("Unable to convert hash to byte array")
	}
	fn update_hash(&mut self) {
		let merkle_tree = self.calculate_merkle_root();
		self.header.merkle_root =
		self.hash = self.calculate_hash();
	}
}
impl Hashable for Transaction {
	fn calculate_hash(&self) -> [u8; 32]{
		let str = format!("{}.{}.{}.{}", self.time, self.sender_address, self.recipient_address, self.amount);
		hash(str.as_bytes()).as_slice().try_into().expect("Unable to convert hash to byte array")
	}
	fn update_hash(&mut self) {
		self.hash = self.calculate_hash();
	}

}