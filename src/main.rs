use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};
use pqcrypto_dilithium::dilithium5::{PublicKey, SecretKey};
use rand::{Rng};
use crate::application::gen_difficulty;
use crate::core::address::P2PKHAddress;
use crate::core::blockchain::{BlockChain, BlockChainConfig};
use crate::network::node::{Node, NodeConfig};

pub mod crypto;
pub mod core;
pub mod network;
pub mod application;

pub static mut address: Option<(P2PKHAddress, PublicKey, SecretKey)> = None;
#[tokio::main]
async fn main() {
	unsafe {
		let hash: [u8; 32] = [
			0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
			0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF,
		];

		let target: [u8; 32] = [
			0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEE, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
			0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF,
		];
		const DIFFICULTY: u128 = 50000;
		let target_value = gen_difficulty(DIFFICULTY);

		let blockchain = BlockChain::new_empty(BlockChainConfig {
			target_value,
			reward: 10,
			block_size: 1024,
			trust_threshold: 6,
			transaction_fee_multiplier: 1.0,
			max_transaction_fee: 10,
		});
		let node = Node::new(blockchain, NodeConfig {
			default_transaction_ttl: 10,
			default_block_ttl: 10,
			listing_port: 25565,
		});
		let seed_peers = vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)), 25565)];
		let node = node.start_node(seed_peers).await.expect("Unable to start node");
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