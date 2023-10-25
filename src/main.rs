use std::collections::{BinaryHeap, VecDeque};
use std::hint::black_box;
use std::io::Read;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use async_std::task;
use crate::crypto::hash::hash;
use pqcrypto_traits::sign::SecretKey;

use crate::core::address::P2PKHAddress;
use crate::core::block::Block;
use crate::core::blockchain::{BlockChain, BlockChainConfig};
use crate::core::utxo::UTXO;

mod crypto;
mod core;
mod p2p;

fn main() {
	let config = BlockChainConfig {
		difficulty: 10,
		reward: 10,
		block_size: 1,
		trust_threshold: 0,
		transaction_fee_multiplier: 0.5,
		max_transaction_fee: 10,
	};
	let main = P2PKHAddress::random();
	let other = P2PKHAddress::random();
	let mut blockchain = BlockChain::new(vec![Block::genesis()], vec![], config);
	let txid = hash(b"test");
	let main_utxo = UTXO {
		txid,
		output_index: 0,
		amount: 126,
		recipient_address: main.0.clone(),
	};
	blockchain.utxo_set.insert(txid, vec![main_utxo]);
	let txid2 = hash(b"test2");

	let mut utxos = Vec::new();
	for _ in 0..10000000 {
		let other_utxo = UTXO {
			txid,
			output_index: 0,
			amount: 3613,
			recipient_address: other.0.clone(),
		};
		utxos.push(other_utxo);
	}
	blockchain.utxo_set.insert(txid2, utxos);
	println!("START");
	let start = Instant::now();
	let balance = blockchain.get_balance(&main.0);
	let time = start.elapsed();
	println!("{:?}", time);
	println!("Balance: {}", balance);
}
fn test_blockchain() {
	let config = BlockChainConfig {
		difficulty: 10,
		reward: 10,
		block_size: 1,
		trust_threshold: 0,
		transaction_fee_multiplier: 0.5,
		max_transaction_fee: 10,
	};
	let bob = P2PKHAddress::random();
	let alice = P2PKHAddress::random();
	let charlie = P2PKHAddress::random();

	let mut blockchain = BlockChain::new(vec![Block::genesis()], vec![], config);

	// let mut t1 = Transaction::new_unsigned(0, 13, &alice.0, &bob.1, &bob.0, 10);
	// let mut t2 = Transaction::new_unsigned(0, 316, &bob.0, &charlie.1, &charlie.0, 15);
	// let mut t3 = Transaction::new_unsigned(0, 6317, &charlie.0, &alice.1, &alice.0, 30);
	//
	// t1.sign(&alice.2).unwrap();
	// t2.sign(&bob.2).unwrap();
	// t3.sign(&charlie.2).unwrap();
	//
	// let b1 = Block::new(vec![t1], 0, 1);
	// let b2 = Block::new(vec![t2], 0, 2);
	// let b3 = Block::new(vec![t3], 0, 3);
	//
	// blockchain.add_block(b1);
	// blockchain.add_block(b2);
	// blockchain.add_block(b3);
}