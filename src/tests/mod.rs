use std::collections::HashSet;

use reqwest::{Client, Url};

use crate::core::parameters::Parameters;
use crate::core::utxo::transaction::Transaction;
use crate::network::models::HttpScheme;
use crate::network::node::{Node, NodeConfig};
use crate::network::sender::Sender;

use super::*;

#[tokio::test(flavor = "multi_thread")]
async fn test1() {
	// env::set_var("RUST_BACKTRACE", "4");
	let node_config = NodeConfig {
		listing_port: 8000,
		http_scheme: HttpScheme::HTTP,
		max_peers: 128,
		peer_cycle_count: 10,
		trusted_peers: HashSet::from([Url::parse("http://192.168.1.104:8000").unwrap()]),
	};

	let parameters = Parameters::default();

	let mut node = Node::new(0, node_config, parameters);
	node.start();

	let client = Client::new();
	if let Ok(inf) = Sender::get_blockchain_info(&client, Url::parse("http://192.168.1.104:8000").unwrap()).await {
		dbg!(&inf);
		assert_eq!(inf.mempool_size, 0);
	}
	let tx = Transaction::create_transaction(vec!(), vec!(), 3131);
	assert!(node.new_transaction(tx).await);
	tokio::time::sleep(Duration::from_secs_f32(0.5)).await;
	for i in 0..10 {
		if let Ok(inf) = Sender::get_blockchain_info(&client, Url::parse("http://192.168.1.104:8000").unwrap()).await {
			dbg!(&inf.best_block_header.time);
			assert_eq!(inf.mempool_size, 1);
		}
		tokio::time::sleep(Duration::from_secs_f32(1.0)).await;
	}
	node.shutdown().await;
}
