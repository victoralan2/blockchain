use std::collections::HashSet;
use std::time::{Duration, Instant};
use log::info;

use reqwest::{Client, Url};
use spin_sleep::sleep;

use crate::core::parameters::Parameters;
use crate::core::utxo::transaction::Transaction;
use crate::network::models::HttpScheme;
use crate::network::node::{Node, NodeConfig};
use crate::network::sender::Sender;

mod address;
mod lottery;
pub(crate) mod timing;

#[tokio::test(flavor = "multi_thread")]
async fn blockinfo_test() {
	// env::set_var("RUST_BACKTRACE", "4");
	let node_config = NodeConfig {
		listing_port: 8000,
		http_scheme: HttpScheme::HTTP,
		max_peers: 128,
		peer_cycle_count: 10,
		trusted_peers: HashSet::from([]), // Url::parse("http://192.168.1.104:8000").expect("Unable to parse url")
	};

	let parameters = Parameters::default();
	let start = Instant::now();
	let mut node = Node::new(0, node_config, parameters).await;
	node.start();
	let client = Client::new();
	if let Ok(inf) = Sender::get_blockchain_info(&client, Url::parse("http://192.168.1.104:8000").expect("Unable to parse url")).await {
		log::debug!("Info {:?}", inf);
		assert_eq!(inf.mempool_size, 0);
	}
	let tx = Transaction::create_transaction(vec!(), vec!(), 31263);
	assert!(node.new_transaction(tx).await);
	sleep(Duration::from_secs_f32(0.5));
	for _ in 0..3 {
		if let Ok(inf) = Sender::get_blockchain_info(&client, Url::parse("http://192.168.1.104:8000").expect("Unable to parse url")).await {
			log::debug!("Info {:?}", inf);
			assert_eq!(inf.mempool_size, 1);
		}
		sleep(Duration::from_secs_f32(0.5));
	}
	node.shutdown().await;
}
