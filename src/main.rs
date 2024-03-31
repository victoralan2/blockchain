use std::collections::HashSet;
use std::fs::{File, read_to_string};
use std::time::{Duration, Instant};

use clap::Parser;
use reqwest::Url;
use rsntp::{AsyncSntpClient, Config, SntpClient};

use crate::args::{Cli, Commands};
use crate::core::parameters::Parameters;
use crate::data_storage::node_config_storage::node_config::NodeConfig;
use crate::data_storage::node_config_storage::url_serialize::PeerUrl;
use crate::network::models::HttpScheme;
use crate::network::node::{Node};
use crate::network::timing;
use crate::network::timing::sync_to_slot;

// TODO: Check that this is cool https://github.com/advisories/GHSA-r8w9-5wcg-vfj7
pub mod crypto;
pub mod core;
pub mod network;
mod consensus;
mod tests;
mod logger;
mod init;
mod args;
mod data_storage;


// TODO: Use logger to log everything
#[tokio::main(flavor = "multi_thread", worker_threads = 30)]
async fn main() {
	let cli = Cli::parse();
	match cli.commands {
		Commands::StartNode(start_node) => {
			let mut trusted_peers = HashSet::new();
			if let Some(path) = start_node.trusted_peers_file {
				if let Ok(str) = read_to_string(path) {

					let lines = str.lines();
					for line in lines {
						if let Ok(url) = Url::parse(line) {
							trusted_peers.insert(PeerUrl::new(url));
						} else {
							log::error!("Unable to parse to URL string {:?} from trusted peers file", line);
						}
					}
				} else {
					log::error!("Unable to load trusted peer file");
				}
			}
			let config = NodeConfig {
				listing_port: start_node.port,
				http_scheme: HttpScheme::HTTP, // TODO: May change with command line arguments
				max_peers: 0, // TODO: May change with command line arguments
				peer_cycle_count: 0, // TODO: May change with command line arguments
				trusted_peers
			};
			let mut node = Node::new(0, config, Parameters::default()).await;
			node.start();

			tokio::signal::ctrl_c().await.unwrap();
			log::info!("Shutting down program...");
			node.shutdown().await;
			log::info!("Program exited successfully");
		}
	}
}
