use std::time::Duration;

use pqcrypto_dilithium::dilithium5::{PublicKey, SecretKey};
use reqwest::{Client, Url};

use crate::core::address::P2PKHAddress;
use crate::core::parameters::Parameters;
use crate::network::models::HttpScheme;
use crate::network::node::{Node, NodeConfig};
use crate::network::sender::Sender;

pub mod crypto;
pub mod core;
pub mod network;
pub mod application;
mod consensus;

pub static mut ADDRESS: Option<(P2PKHAddress, Vec<u8>, Vec<u8>)> = None;
#[tokio::main]
async fn main() {
	unsafe { ADDRESS = Some(P2PKHAddress::random()) }
	let node_config = NodeConfig {
		listing_port: 8000,
		http_scheme: HttpScheme::HTTP,
		max_peers: 128,
		peer_cycle_count: 10,
		trusted_peers: Default::default(),
	};

	let parameters = Parameters::default();

	let mut node = Node::new(0, node_config, parameters);
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
