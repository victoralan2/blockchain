use std::collections::HashSet;
use std::str::FromStr;

use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::data_storage::BaseDirectory;
use crate::data_storage::node_config_storage::url_serialize::PeerUrl;
use crate::network::models::HttpScheme;

const DEFAULT_PORT: u16 = 1379;


#[derive(Clone, Serialize, Deserialize)]
pub struct NodeConfig {
	pub listing_port: u16,
	pub http_scheme: HttpScheme,
	pub max_peers: usize,
	/// The amount of peers that will cycle each time
	pub peer_cycle_count: usize,
	pub trusted_peers: HashSet<PeerUrl>,
}

impl NodeConfig {
	/// If path is None, the default path will be used
	pub fn load(path: Option<String>) -> anyhow::Result<Self> {
		let default_path = format!("{}/node/config.json", BaseDirectory::get_base_directory());
		let path = path.unwrap_or(default_path);
		let data = std::fs::read_to_string(&path).expect(&format!("Unable to load data from given path: {}", path));
		let _self = serde_json::from_str(&data)?;
		Ok(_self)
	}
}
impl Default for NodeConfig {
	fn default() -> Self {
		Self {
			listing_port: DEFAULT_PORT,
			http_scheme: HttpScheme::HTTP,
			max_peers: 128,
			peer_cycle_count: 8,
			trusted_peers: Default::default(),
		}
	}
}
