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
use rsntp::{AsyncSntpClient, Config, SynchronizationError};
use serde::Serialize;
use tokio::sync::{Mutex, RwLock};
use tokio::task::block_in_place;
use crate::consensus::miner::Miner;

use crate::core::block::Block;
use crate::core::blockchain::BlockChain;
use crate::core::Hashable;
use crate::core::keys::NodeKeyChain;
use crate::core::parameters::Parameters;
use crate::core::utxo::transaction::Transaction;
use crate::data_storage::node_config_storage::node_config::NodeConfig;
use crate::data_storage::node_config_storage::url_serialize::PeerUrl;
use crate::network::{config, timing};
use crate::network::config::config_routes;
use crate::network::models::{HttpScheme, NewBlock, NewTransaction};
use crate::network::sender::Sender;
use crate::network::standard::standard_serialize;

#[derive(Clone)]
pub struct Node {
	// TODO: Keys... and stuff
	pub version: u32,
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

pub(crate) const STARTING_SLOT_SECOND: u64 = 0;

// TODO: AT THE END CHANGE THIS NUMBER FOR THE EPOCH SECOND OF THE TIME THE CRYPTO IS RELEASED
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
		let reward_address = key_chain.wallet_key_pair.0;
		let config = NodeConfig::default();
		Self {
			version,
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
		let reward_address = key_chain.wallet_key_pair.0;
		Self {
			version,
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
			self_clone.main_loop().await;
		});
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
					tokio::spawn(async move {
						Self::broadcast_block(&msg, &peers).await;
						log::info!("Finished broadcasting block {}.", msg.block.header.height);
					}).await.ok();
				} else {
					log::error!("New block created but could not add to blockchain")
				}
			}
		});

		log::info!("Started main loop thread");
	}
	fn start_node(&mut self) {
		// STARTS THE NODE, THE ENTRY POINT.
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
		miner.reward_address = self.key_chain.wallet_key_pair.0;
		let blockchain = self.blockchain.read().await;
		miner.height = blockchain.get_height();
		miner.last_hash = blockchain.get_last_block().header.hash;
		
		let max_tx_size = self.parameters.network_parameters.max_tx_size;
		let max_block_body_size = self.parameters.network_parameters.max_block_body_size;
		miner.transactions = blockchain.mempool.get_map().iter().take(max_block_body_size / max_tx_size).cloned().collect();
	}
	/// Forges a new block when the lottery is won
	pub async fn mine_new_block(&mut self) {
		// todo!();
		log::info!("Lottery won!");
		
		let start = Instant::now();
		let mut chain = self.blockchain.write().await;

		let prev_hash = chain.get_last_block().header.hash;

		let mut transactions = Vec::new();

		for tx in chain.mempool.get_map() {
			if size_of_val(&transactions) > self.parameters.network_parameters.max_block_body_size {
				transactions.remove(transactions.len() - 1);
				break;
			} else {
				transactions.push(tx.clone());
			}
		}

		
		let new_block = Block::new(
			chain.get_height() + 1,
			transactions,
			prev_hash,
			self.key_chain.wallet_key_pair.0,
			0
		);
		if chain.add_block(&new_block) {
			let msg = NewBlock {
				version: self.version,
				block: new_block,
			};
			let peers = self.peers.read().await.clone();
			log::info!("Took about {:?} to add to chain", start.elapsed());

			log::info!("Started broadcasting block {}", msg.block.header.height);
			tokio::spawn(async move {
				Self::broadcast_block(&msg, &peers).await;
				log::info!("Finished broadcasting block {}.", msg.block.header.height);
			}).await.ok();
		} else {
			log::error!("New block created but could not add to blockchain")
		}
	}

	pub fn start_mining(&mut self) {
		self.should_mine.store(true, Ordering::Relaxed);
	}
	pub fn stop_mining(&mut self) {
		self.should_mine.store(false, Ordering::Relaxed);
	}
	pub async fn main_loop(&mut self) {
		let mut counter = 0u32; // Counter to replace peers
		const REPLACE_PEER_TIME: u32 = 10u32; // In seconds
		while !self.is_shutdown() {
			// // Check if pairs height is bigger
			// let self_copy = self.clone();
			// spawn(async move {
			// 	let peers = self_copy.peers.read().await.clone();
			//
			// 	let client = Client::new();
			// 	let current_height = self_copy.blockchain.read().await.get_height();
			// 	for peer in peers {
			// 		let info = tokio::time::timeout(Duration::from_millis(500), Sender::get_blockchain_info(&client, peer)).await; // Timeout because it may take a long time
			// 		if let Ok(Ok(info)) = info {
			// 			let height = info.height;
			// 			if height > current_height {
			// 				let mut self_copy_copy = self_copy.clone();
			// 				spawn(async move {self_copy_copy.sync_chain().await});
			// 				break;
			// 			}
			// 		}
			// 	}
			// });
			// Check if peer list is full
			if self.peers.read().await.len() < self.config.max_peers {
				let mut self_copy = self.clone();
				// spawn(async move {self_copy.discover_peers().await;}); // TODO: Enable this
			}

			if counter > REPLACE_PEER_TIME {
				let mut self_copy = self.clone();
				counter = 0;
				// spawn(async move { self_copy.cycle_peers().await; }); //TODO: Enable this
			}

			counter += 1;
			tokio::time::sleep(Duration::from_secs(1)).await;
		}
	}
	
	pub async fn discover_peers(&mut self) {
		// TODO
	}
	
	async fn discover_n_peers(&self, n: u32) -> HashSet<PeerUrl> {
		// TODO: Check that the peer discovered isn't already in peer list
		// TODO: Check that if the peer list is empty, use seed peers
		// TODO: Check that the peer version is valid and the peer is online

		const N: u32 = 2;
		const M: u32 = 10;

		let mut current_peers = self.peers.read().await.clone();
		let mut rng = thread_rng();
		for _ in 0..N {
			for p in current_peers.clone() {
				// Sender::get_peers();
			}
		}
		todo!()
	}
	pub async fn cycle_peers(&mut self) {
		let peer_cycle_count = self.config.peer_cycle_count;
		let trusted_peers = &self.config.trusted_peers;
		let new_peers: HashSet<PeerUrl> = self.discover_n_peers(peer_cycle_count as u32).await;

		let mut peers = self.peers.write().await;
		let peers_to_remove: Vec<PeerUrl> = peers.iter()
			.filter(|&url| { !trusted_peers.contains(url) }) // Check that it does not remove a trusted peer
			.cloned()
			.choose_multiple(&mut thread_rng(), peer_cycle_count);

		for peer in peers_to_remove {
			peers.remove(&peer);
		}
		for new_peer in new_peers {
			peers.insert(new_peer);
		}
	}
	pub async fn sync_chain(&mut self) {
		// TODO
	}
	pub async fn shutdown(&mut self) {
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
			Self::broadcast_transaction(peers, &msg).await;
			true
		} else {
			false
		}
	}

	pub async fn broadcast_transaction(peers: HashSet<PeerUrl>, tx: &NewTransaction) {
		let urls: HashSet<Url> = peers.iter().map(|url| {
			let mut new_url = url.to_url().clone();
			new_url.set_path(config::NEW_TRANSACTION_URL);
			new_url
		}).collect();

		Self::broadcast_bytes(urls, tx).await;
	}
	pub async fn broadcast_block(block: &NewBlock, peers: &HashSet<PeerUrl>) {
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
}