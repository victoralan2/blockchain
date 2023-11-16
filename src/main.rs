use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};
use pqcrypto_dilithium::dilithium5::{PublicKey, SecretKey};
use rand::{Rng};
use crate::application::gen_difficulty;
use crate::core::address::P2PKHAddress;
use crate::core::blockchain::{BlockChain, BlockChainConfig};
use crate::network::models::HttpScheme;
use crate::network::node::{Node, NodeConfig};

pub mod crypto;
pub mod core;
pub mod network;
pub mod application;

pub static mut ADDRESS: Option<(P2PKHAddress, PublicKey, SecretKey)> = None;
#[tokio::main]
async fn main() {
	unsafe {
		const DIFFICULTY: u128 = 5000;
		let target_value = gen_difficulty(DIFFICULTY);

		let node_config = NodeConfig {
			listing_port: 8000,
			http_scheme: HttpScheme::HTTP,
		};

		let blockchain_config = BlockChainConfig {
			target_value,
			reward: 10,
			block_size: 1024,
			trust_threshold: 6,
			transaction_fee_multiplier: 1.0,
			max_transaction_fee: 10,
		};

		let node = Node::new(0, node_config, blockchain_config);
		let handle = node.start_node();


	}
}


// fn test_blockchain() {
//
// 	let config = BlockChainConfig {
// 		difficulty: 10,
// 		reward: 10,
// 		block_size: 1,
// 		trust_threshold: 0,
// 		transaction_fee_multiplier: 0.5,
// 		max_transaction_fee: 10,
// 	};
// 	let bob = P2PKHAddress::random();
// 	let alice = P2PKHAddress::random();
// 	let charlie = P2PKHAddress::random();
//
// 	let mut blockchain = BlockChain::new(vec![Block::genesis()], vec![], config);
//
// 	// let mut t1 = Transaction::new_unsigned(0, 13, &alice.0, &bob.1, &bob.0, 10);
// 	// let mut t2 = Transaction::new_unsigned(0, 316, &bob.0, &charlie.1, &charlie.0, 15);
// 	// let mut t3 = Transaction::new_unsigned(0, 6317, &charlie.0, &alice.1, &alice.0, 30);
// 	//
// 	// t1.sign(&alice.2).unwrap();
// 	// t2.sign(&bob.2).unwrap();
// 	// t3.sign(&charlie.2).unwrap();
// 	//
// 	// let b1 = Block::new(vec![t1], 0, 1);
// 	// let b2 = Block::new(vec![t2], 0, 2);
// 	// let b3 = Block::new(vec![t3], 0, 3);
// 	//
// 	// blockchain.add_block(b1);
// 	// blockchain.add_block(b2);
// 	// blockchain.add_block(b3);
// }