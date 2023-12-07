use serde::{Deserialize, Serialize};

use crate::core::address::P2PKHAddress;
use crate::core::blockchain::BlockChain;
use crate::core::Hashable;
use crate::core::utxo::Output;

#[derive(Clone, Debug, Eq, Hash, Serialize, Deserialize, PartialEq)]
pub struct CoinbaseTransaction {
	pub id: [u8; 32],
	pub output: Output,
	pub time: u64,
}

impl CoinbaseTransaction {
	pub fn create(address: P2PKHAddress, reward: u64) -> Self {
		let output = Output {
			amount: reward,
			address,
		};
		let mut this = Self {
			id: [0u8; 32],
			output,
			time: 0,
		};
		this.update_hash();
		this
	}
	pub fn genesis() -> Self {
		Self {
			id: [0u8; 32],
			output: Output { amount: 10, address: P2PKHAddress::null() },
			time: 0,
		}
	}
	pub fn is_valid(&self, blockchain: &BlockChain) -> bool {
		todo!()
	}
}