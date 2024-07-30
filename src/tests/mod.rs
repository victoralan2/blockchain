use std::collections::HashSet;
use std::time::{Duration, Instant};

use log::info;
use reqwest::{Client, Url};
use spin_sleep::sleep;

use crate::core::parameters::Parameters;
use crate::core::utxo::transaction::Transaction;
use crate::data_storage::node_config_storage::node_config::NodeConfig;
use crate::network::models::HttpScheme;
use crate::network::node::{Node};
use crate::network::sender::Sender;

mod address;
mod lottery;
pub(crate) mod timing;
mod data_sotrage;
mod miner;

#[tokio::test(flavor = "multi_thread")]
async fn blockinfo_test() {
	// env::set_var("RUST_BACKTRACE", "4");

	let parameters = Parameters::default();
	let mut node = Node::new(0, None, parameters).await;
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
