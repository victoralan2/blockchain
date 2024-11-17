use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::core::address::{P2PKHAddress, SigningKey, VerifyingKey};
use crate::data_storage::BaseDirectory;
use crate::data_storage::node_config_storage::{CONFIG_FILE_NAME, KEYCHAIN_FILE_NAME, NODE_DIRECTORY_NAME};
use crate::data_storage::node_config_storage::node_config::NodeConfig;

#[derive(Clone, Deserialize, Serialize)]
pub struct NodeKeyChain {
	pub signing_key: SigningKey,
	pub verifying_key: VerifyingKey,
	pub address: P2PKHAddress,
}
impl NodeKeyChain {
	pub fn random() -> Self {
		let a  = P2PKHAddress::random();
		Self {
			signing_key: a.1,
			verifying_key: a.2,
			address: a.0,
		}
	}
	pub fn load() -> Self {
		let path = format!("{}/{}/{}", BaseDirectory::get_base_directory(), NODE_DIRECTORY_NAME, KEYCHAIN_FILE_NAME);
		if !Path::new(&path).exists() {
			let keychain = Self::random();
			create_dir_all(Path::new(&path).parent().expect("Unable to get parent directory")).expect("Unable to create directories");
			let mut file = OpenOptions::new().create(true).write(true).open(&path).expect("Unable to open node config file");
			let data = serde_json::to_string_pretty(&keychain).expect("Unable to serialize");
			file.write_all(data.as_bytes()).expect("Unable to write to file");
			keychain
		} else {
			let data = std::fs::read_to_string(&path).expect(&format!("Unable to load data from given path: {}", path));
			serde_json::from_str(&data).expect("Unable to deserialize")
		}
	}
}