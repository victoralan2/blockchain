use std::time::{Instant, SystemTime, UNIX_EPOCH};
use pqcrypto_traits::sign::SecretKey;
use crate::core::address::Address;
use crate::core::block::Block;
use crate::core::blockchain::{BlockChain, BlockChainConfig};
use crate::core::blockdata::{BlockData, Transaction};

mod crypto;
mod core;

fn main() {
	let config = BlockChainConfig {
		difficulty: 10,
		reward: 10,
		block_size: 1,
		trust_threshold: 0,
	};
	let bob = Address::random();
	let alice = Address::random();
	let charlie = Address::random();

	let mut blockchain = BlockChain::new(vec![Block::genesis()], vec![], config);

	let mut t1 = Transaction::new_unsigned(0, &alice.0, &bob.0, 10);
	let mut t2 = Transaction::new_unsigned(0, &bob.0, &charlie.0, 15);
	let mut t3 = Transaction::new_unsigned(0, &charlie.0, &alice.0, 30);

	t1.sign(&alice.1).unwrap();
	t2.sign(&bob.1).unwrap();
	t3.sign(&charlie.1).unwrap();

	let b1 = Block::new(vec![BlockData::TX(t1)], 0, 1);
	let b2 = Block::new(vec![BlockData::TX(t2)], 0, 2);
	let b3 = Block::new(vec![BlockData::TX(t3)], 0, 3);

	blockchain.add_block(b1);
	blockchain.add_block(b2);
	blockchain.add_block(b3);

}
