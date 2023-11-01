use serde::{Deserialize, Serialize};
use crate::core::block::Block;
use crate::core::utxo::transaction::Transaction;


pub trait Capsule: Send + Sync + Deserialize<'static> + Serialize{
	type T: Serialize + Deserialize<'static>;


	fn is_alive(&self) -> bool;
	fn consume(&mut self) -> Option<Self::T>;
}
#[derive(Clone, Serialize, Deserialize)]
pub struct TransactionCapsule {
	pub(crate) transaction: Transaction,
	pub(crate) time_to_live: u32,
}
impl Capsule for TransactionCapsule {
	type T = Transaction;
	fn is_alive(&self) -> bool {
		return self.time_to_live != 0;
	}
	fn consume(&mut self) -> Option<Self::T> {
		if self.time_to_live != 0 {
			self.time_to_live -= 1;
			Some(self.transaction.clone())
		} else {
			None
		}
	}
}
#[derive(Clone, Serialize, Deserialize)]
pub struct BlockCapsule {
	pub(crate) block: Block,
	pub(crate) time_to_live: u32,
}
impl Capsule for BlockCapsule {
	type T = Block;
	fn is_alive(&self) -> bool {
		return self.time_to_live != 0;
	}
	fn consume(&mut self) -> Option<Self::T> {
		if self.time_to_live > 0 {
			self.time_to_live -= 1;
			Some(self.block.clone())
		} else {
			None
		}
	}
}