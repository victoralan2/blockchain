use std::io::Read;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use async_std::task;

use pqcrypto_traits::sign::SecretKey;

use crate::core::address::P2PKHAddress;
use crate::core::block::Block;
use crate::core::blockchain::{BlockChain, BlockChainConfig};
use crate::core::blockdata::{BlockData, Transaction};
use crate::p2p::node::Node;

mod crypto;
mod core;
mod p2p;

pub struct Test {
	pub value: u64,
}
impl Test {
	pub fn test(&mut self) {
		let this = Arc::new(self);
		let self_copy = this.clone();
		task::spawn( async move {
			for _ in 0..100000000 {
				println!("{}", self_copy.value);
			}
		});
		for i in 0..1000000000 {
			self.value = i;
		}
	}
}
#[tokio::main]
async fn main() {


}
fn test_blockchain() {
	let config = BlockChainConfig {
		difficulty: 10,
		reward: 10,
		block_size: 1,
		trust_threshold: 0,
		max_data_size: 512,
		data_fee_multiplier: 0.5,
		transaction_fee_multiplier: 0.5,
		max_transaction_fee: 10,
	};
	let bob = P2PKHAddress::random();
	let alice = P2PKHAddress::random();
	let charlie = P2PKHAddress::random();

	let mut blockchain = BlockChain::new(vec![Block::genesis()], Default::default(), config);

	let mut t1 = Transaction::new_unsigned(0, 13, &alice.0, &bob.1, &bob.0, 10);
	let mut t2 = Transaction::new_unsigned(0, 316, &bob.0, &charlie.1, &charlie.0, 15);
	let mut t3 = Transaction::new_unsigned(0, 6317, &charlie.0, &alice.1, &alice.0, 30);

	t1.sign(&alice.2).unwrap();
	t2.sign(&bob.2).unwrap();
	t3.sign(&charlie.2).unwrap();

	let b1 = Block::new(vec![BlockData::TX(t1)], 0, 1);
	let b2 = Block::new(vec![BlockData::TX(t2)], 0, 2);
	let b3 = Block::new(vec![BlockData::TX(t3)], 0, 3);

	blockchain.add_block(b1);
	blockchain.add_block(b2);
	blockchain.add_block(b3);
}