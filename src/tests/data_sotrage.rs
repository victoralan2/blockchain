use std::str::FromStr;
use reqwest::Url;
use crate::core::parameters::Parameters;
use crate::core::utxo::transaction::Transaction;
use crate::data_storage::node_config_storage::node_config::NodeConfig;
use crate::network::node::Node;

#[tokio::test(flavor = "multi_thread")]
async fn data_storage_test() {
	let mut node = Node::new(1, None, Parameters::default()).await;
	// Url::from_str("https://www.youtube.com").unwrap();
}