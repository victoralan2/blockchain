use std::collections::HashSet;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use crate::data_storage::BaseDirectory;
use crate::data_storage::node_config_storage::{CONFIG_FILE_NAME, NODE_DIRECTORY_NAME};
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
	pub max_mempool_size_mb: usize,
}
impl NodeConfig {
	/// If path is None, the default path will be used
	/// If the path or default path does not exist, a new file is created with the default settings
	pub fn load_or_create(path: Option<String>) -> Self {
		let default_path = format!("{}/{}/{}", BaseDirectory::get_base_directory(), NODE_DIRECTORY_NAME, CONFIG_FILE_NAME);
		let path = path.unwrap_or(default_path);
		if !Path::new(&path).exists() {
			let config = NodeConfig::default();
			create_dir_all(Path::new(&path).parent().expect("Unable to get parent directory")).expect("Unable to create directories");
			let mut file = OpenOptions::new().create(true).write(true).open(&path).expect("Unable to open node config file");
			let data = serde_json::to_string_pretty(&config).expect("Unable to serialize");
			file.write_all(data.as_bytes()).expect("Unable to write to file");
			config
		} else {
			let data = std::fs::read_to_string(&path).expect(&format!("Unable to load data from given path: {}", path));
			serde_json::from_str(&data).expect("Unable to deserialize")
		}

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
			max_mempool_size_mb: 300, // 300 MB
		}
	}
}
