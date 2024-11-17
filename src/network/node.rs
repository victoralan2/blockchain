use std::cmp::min;
use std::collections::HashSet;
use std::mem::size_of_val;
use std::ops::Deref;
use std::process::{exit, ExitCode, ExitStatus};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use actix_web::{App, HttpServer};
use actix_web::dev::ServerHandle;
use actix_web::web::{Data, to};
use local_ip_address::local_ip;
use rand::prelude::IteratorRandom;
use rand::thread_rng;
use reqwest::{Client, Url};
use rsntp::{Config, SynchronizationError};
use serde::Serialize;
use tokio::sync::{Mutex, RwLock};
use tokio::task::block_in_place;
use crate::consensus::miner::Miner;

use crate::core::block::Block;
use crate::core::blockchain::BlockChain;
use crate::core::keys::NodeKeyChain;
use crate::core::parameters::Parameters;
use crate::core::utxo::transaction::Transaction;
use crate::data_storage::node_config_storage::node_config::NodeConfig;
use crate::data_storage::node_config_storage::url_serialize::PeerUrl;
use crate::network::{config};
use crate::network::config::config_routes;
use crate::network::models::{BlocksData, GetBlocks, GetData, GetHeaders, Headers, HttpScheme, InvDataType, NewBlock, NewTransaction};
use crate::network::sender::Sender;
use crate::network::standard::standard_serialize;

#[derive(Clone)]
pub struct Node {
	// TODO: Keys... and stuff
	pub version: u32,
	pub recently_seen_ids: Arc<RwLock<HashSet<[u8; 32]>>>,
	pub blockchain: Arc<RwLock<BlockChain>>,
	pub peers: Arc<RwLock<HashSet<PeerUrl>>>,
	// TODO: Implement gossip protocol instead of broadcasting everything to everyone
	shutdown: Arc<AtomicBool>,
	key_chain: NodeKeyChain,
	pub server_handle: Option<ServerHandle>,
	pub config: NodeConfig,
	pub parameters: Parameters,  // TODO: Keep in mind that if something changes that is not Arc<> it will not be updated in the main loop
	pub miner: Arc<Mutex<Miner>>,
	pub should_mine: Arc<AtomicBool>
}

impl Node {
	pub async fn default(version: u32) -> Self {
		// let ntp_client = AsyncSntpClient::new();
		// let slot_time = ntp_client.synchronize("time.google.com").await
		// 	.expect("Unable to sync with NTP server")
		// 	.datetime()
		// 	.unix_timestamp()
		// 	.expect("Time went backwards") - Duration::from_secs(STARTING_SLOT_SECOND);
		let parameters = Parameters::default();
		let key_chain = NodeKeyChain::random();
		let reward_address = key_chain.address;
		let config = NodeConfig::default();
		Self {
			version,
			recently_seen_ids: Arc::new(Default::default()),
			blockchain: Arc::new(RwLock::new(BlockChain::init(parameters, &config))),
			shutdown: Arc::new(AtomicBool::new(false)),
			key_chain,
			server_handle: None,
			config,
			peers: Arc::new(Default::default()), // TODO: Load from default file
			parameters,
			miner: Arc::new(Mutex::new(Miner::new(vec![], 0, [0u8; 32], reward_address, [255u8; 32]))),
			should_mine: Arc::new(AtomicBool::new(false)),
		}
	}
	pub async fn new(version: u32, config_file: Option<String>, parameters: Parameters) -> Self {
		// let ntp_client = AsyncSntpClient::new();
		// let slot_time = ntp_client.synchronize("time.google.com").await
		// 	.expect("Unable to sync with NTP server")
		// 	.datetime()
		// 	.unix_timestamp()
		// 	.expect("Time went backwards") - Duration::from_secs(STARTING_SLOT_SECOND);
		// TODO: Store in some way the keychain
		let config = NodeConfig::load_or_create(config_file);
		let peers = config.trusted_peers.clone();

		let key_chain = NodeKeyChain::random();
		let reward_address = key_chain.address;
		Self {
			version,
			recently_seen_ids: Arc::new(Default::default()),
			blockchain: Arc::new(RwLock::new(BlockChain::init(parameters, &config))),
			shutdown: Arc::new(AtomicBool::new(false)),
			key_chain,
			server_handle: None,
			config,
			peers: Arc::new(RwLock::new(peers)),
			parameters,
			miner: Arc::new(Mutex::new(Miner::new(vec![], 0, [0u8; 32], reward_address, [255u8; 32]))),
			should_mine: Arc::new(AtomicBool::new(false)),
		}
	}
	pub fn start(&mut self) {
		log::info!("Starting the node");
		self.start_node();
		log::info!("Node started successfully");
		let mut self_clone = self.clone();
		tokio::spawn(async move {
			// TODO: Give miner needed info
			loop {
				self_clone.update_miner().await;
				let mined_block = Miner::start_mining(Arc::clone(&self_clone.miner), self_clone.should_mine.clone()).await;
				log::info!("Block mined successfully!");
				let mut chain = self_clone.blockchain.write().await;
				if chain.add_block(&mined_block) {
					let msg = NewBlock {
						version: self_clone.version,
						block: mined_block,
					};
					let peers = self_clone.peers.read().await.clone();

					log::info!("Started broadcasting block {}", msg.block.header.height);
					self_clone.broadcast_block(&msg, &peers).await;
				} else {
					log::error!("New block created but could not add to blockchain")
				}
			}
		});

		log::info!("Started main loop thread");
	}
	fn start_node(&mut self) {
		// STARTS THE NODE
		let app_state = Data::new(self.clone());

		// Setup server
		let server = match HttpServer::new(move || {
			App::new()
				.app_data(app_state.clone())
				.configure(config_routes)
		})
			.bind(format!("{}:{}", local_ip().expect("Unable to get local IP"), self.config.listing_port)) {
			Ok(server) => { // Just run and return server
				server.run()
			}
			Err(err) => {
				// Log error and exit
				log::error!("Unable to bind to IP: \"{}\". Maybe already in use?. Error: {}", format!("{}:{}", local_ip().expect("Unable to get local IP"), self.config.listing_port), err);
				exit(0);
			}
		};
		log::info!("Started node at: {}", format!("{}:{}", local_ip().expect("Unable to get local IP"), self.config.listing_port));

		let handle = server.handle();
		tokio::spawn(server);

		self.server_handle = Some(handle);
	}

	async fn update_miner(&mut self) {
		let mut miner = self.miner.lock().await;
		miner.reward_address = self.key_chain.address;
		let blockchain = self.blockchain.read().await;
		miner.height = blockchain.get_height();
		miner.last_hash = blockchain.get_last_block().header.hash;
		
		let max_tx_size = self.parameters.network_parameters.max_tx_size;
		let max_block_body_size = self.parameters.network_parameters.max_block_body_size;
		miner.transactions = blockchain.mempool.get_map().iter().take(max_block_body_size / max_tx_size).cloned().collect();
	}
	pub fn start_mining(&mut self) {
		self.should_mine.store(true, Ordering::Relaxed);
	}
	pub fn stop_mining(&mut self) {
		self.should_mine.store(false, Ordering::Relaxed);
	}
	pub async fn shutdown(&mut self) {
		self.blockchain.write().await.flush();
		self.shutdown.store(true, Ordering::Relaxed);
		if let Some(handle) = &self.server_handle {
			handle.stop(true).await;
		}
		// TODO: Save to file or smth
	}
	pub fn is_shutdown(&self) -> bool {
		block_in_place(|| self.shutdown.load(Ordering::Relaxed))
	}
	// Returns weather it was added or not
	pub async fn new_transaction(&self, transaction: Transaction) -> bool {
		if self.blockchain.write().await.add_transaction_to_mempool(&transaction) {
			let msg = NewTransaction {
				version: self.version,
				transaction,
			};
			let peers = self.peers.read().await.clone();
			self.broadcast_transaction(peers, &msg).await;
			true
		} else {
			false
		}
	}

	pub async fn broadcast_transaction(&self, peers: HashSet<PeerUrl>, tx: &NewTransaction) {
		let mut set = self.recently_seen_ids.write().await;
		if !set.insert(tx.transaction.id) {
			return;
		}
		let urls: HashSet<Url> = peers.iter().map(|url| {
			let mut new_url = url.to_url().clone();
			new_url.set_path(config::NEW_TRANSACTION_URL);
			new_url
		}).collect();

		Self::broadcast_bytes(urls, tx).await;
	}
	pub async fn broadcast_block(&self, block: &NewBlock, peers: &HashSet<PeerUrl>) {
		let mut set = self.recently_seen_ids.write().await;
		if !set.insert(block.block.header.hash) {
			return;
		}
		// TODO: Implement gossip protocol instead of broadcasting everything to everyone
		let urls: HashSet<Url> = peers.iter().map(|url| {
			let mut new_url = url.to_url().clone();
			new_url.set_path(config::NEW_BLOCK_URL);
			new_url
		}).collect();

		Self::broadcast_bytes(urls, block).await;
	}
	async fn broadcast_bytes<T>(urls: HashSet<Url>, msg: &T)
		where T: Serialize + Send {
		let mut handles = vec![];
		let client = Client::new();

		if let Ok(bytes) = standard_serialize(msg) {
			for url in urls {
				let bytes = bytes.clone();

				let url = url.clone();
				let client = client.clone();
				handles.push(tokio::spawn(async move {
					Sender::send_bytes(&client, url, bytes.clone()).await.ok();
				}));
			}
		}
		for h in handles {
			h.await.ok();
		}
	}

	pub async fn get_headers(&self, peer_url: &PeerUrl, client: &Client, block_locator_object: Vec<[u8; 32]>) -> anyhow::Result<Headers> {
		Sender::get_headers(client, peer_url.to_url(), &GetHeaders {
			version: self.version,
			block_locator_object,
		}).await
	}
	pub async fn get_blocks_data(&self, peer_url: &PeerUrl, client: &Client, hashes: Vec<[u8; 32]>) -> anyhow::Result<BlocksData> {
		Sender::get_block_data(client, peer_url.to_url(), &GetData {
			version: self.version,
			data_type: InvDataType::Block,
			hashes,
		}).await
	}
}