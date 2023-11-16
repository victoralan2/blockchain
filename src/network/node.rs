use std::error::Error;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use actix_web::{App, HttpServer};
use actix_web::dev::ServerHandle;
use actix_web::web::Data;
use reqwest::{Client, RequestBuilder, Url};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use crate::core::address::P2PKHAddress;
use crate::core::blockchain::{BlockChain, BlockChainConfig};
use crate::core::utxo::transaction::Transaction;
use crate::network::{config, standard};
use crate::network::config::config_routes;
use crate::network::models::{HttpScheme, NewTransaction, Subscribe};

#[derive(Clone)]
pub struct State {
	pub version: u32,
	pub(crate) blockchain: Arc<Mutex<BlockChain>>,
	pub(crate) subscribed_to: Arc<Mutex<Vec<SocketAddr>>>,
	pub(crate) subscribers: Arc<Mutex<Vec<Url>>>,
	pub(crate) config: NodeConfig,
}
#[derive(Copy, Clone)]
pub struct NodeConfig {
	pub(crate) listing_port: u16,
	pub(crate) http_scheme: HttpScheme,
}

pub struct Node {
	pub version: u32,
	pub state: Arc<Mutex<State>>,
	pub config: NodeConfig,
}

impl Node {
	pub fn new(version: u32, config: NodeConfig, blockchain_config: BlockChainConfig) -> Self {
		Self {
			version,
			state: Arc::new(Mutex::new(State {
				version,
				blockchain: Arc::new(Mutex::new(BlockChain::new_empty(blockchain_config))),
				subscribed_to: Arc::new(Default::default()),
				subscribers: Arc::new(Default::default()),
				config,
			})),
			config,
		}
	}

	pub fn start_node(&self) -> ServerHandle {
		let app_state = Data::new(self.state.clone());
		let server = HttpServer::new(move || {
			App::new()
				.app_data(app_state.clone())
				.configure(config_routes)
		})
			.bind("192.168.1.104:8000").unwrap().run();
		let handle = server.handle();
		tokio::spawn(server);
		handle
	}

	pub async fn subscribe_to(&self, endpoint: Url) -> Result<(), Box<dyn Error>> {
		let msg = Subscribe {
			version: self.state.lock().await.version,
			method: self.config.http_scheme,
			port: self.config.listing_port,
			subdirectory: "/subscribe".to_string(),
		};
		let client = reqwest::Client::new();
		client.post(endpoint)
			.body(standard::serialize(&msg)?)
			.send()
			.await?;

		Ok(())
	}
	pub async fn ibd() {
		// TODO
	}
	pub async fn mine_block(&mut self, miner_address: P2PKHAddress, keep_mining: Arc<AtomicBool>) {
		// TODO
	}
	pub async fn new_transaction(&self, transaction: Transaction) {
		let msg = NewTransaction {
			version: self.version,
			transaction
		};
		let subs = self.state.lock().await.subscribers.lock().await.clone();
		for s in subs {
			let client = Client::new();
			client.post(s);
			// TODO
		}
	}
	async fn broadcast_bytes(&self, msg: &[u8]) {

	}
}