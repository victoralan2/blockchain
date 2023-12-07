use std::time::Duration;

use pqcrypto_dilithium::dilithium5::{PublicKey, SecretKey};
use reqwest::{Client, Url};

use crate::application::gen_difficulty;
use crate::core::address::P2PKHAddress;
use crate::core::blockchain::BlockChainConfig;
use crate::network::models::HttpScheme;
use crate::network::node::{Node, NodeConfig};
use crate::network::sender::Sender;

pub mod crypto;
pub mod core;
pub mod network;
pub mod application;

pub static mut ADDRESS: Option<(P2PKHAddress, PublicKey, SecretKey)> = None;
#[tokio::main]
async fn main() {
	unsafe { ADDRESS = Some(P2PKHAddress::random()) }
	const DIFFICULTY: u128 = 5000;
	let target_value = gen_difficulty(DIFFICULTY);

	let node_config = NodeConfig {
		listing_port: 8000,
		http_scheme: HttpScheme::HTTP,
		max_peers: 128,
		peer_cycle_count: 10,
		trusted_peers: Default::default(),
	};

	let blockchain_config = BlockChainConfig {
		target_value,
		reward: 10,
		block_size: 1024,
		trust_threshold: 6,
		transaction_fee_multiplier: 1.0,
		max_transaction_fee: 10,
	};

	let mut node = Node::new(0, node_config, blockchain_config);
	node.start_node();
	tokio::time::sleep(Duration::from_millis(1000)).await;

	let client = Client::new();
	if let Ok(inf) = Sender::get_blockchain_info(&client, Url::parse("http://192.168.1.104:8000").unwrap()).await {
		println!("{:?}", inf);
	}
	tokio::signal::ctrl_c().await.unwrap();
	//
	// let msg = TestBody {
	// 	test: "asdasd".to_string(),
	// };
	// let data = standard_serialize(&msg).unwrap();
	//
	// let c = reqwest::Client::new();
	// let response = c.("http://192.168.1.104:8000/test")
	// 	.body(data)
	// 	.header(reqwest::header::CONTENT_TYPE, "application/octet-stream") // Set the content type
	// 	.send().await.unwrap();
	// println!("{:?}", response.text().await.unwrap());
	// tokio::signal::ctrl_c().await.unwrap();
	// handle.stop(false).await;
}
