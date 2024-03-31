use std::collections::HashSet;
use std::mem::size_of_val;
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
use tokio::runtime::Handle;
use tokio::sync::RwLock;
use tokio::task::block_in_place;

use crate::consensus::lottery::Lottery;
use crate::core::block::Block;
use crate::core::blockchain::BlockChain;
use crate::core::keys::NodeKeyChain;
use crate::core::parameters::Parameters;
use crate::core::utxo::transaction::Transaction;
use crate::crypto::vrf::{prove, VrfPk, VrfProof, VrfSk};
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
	pub current_slot: Arc<AtomicU64>,
	pub blockchain: Arc<RwLock<BlockChain>>,
	pub peers: Arc<RwLock<HashSet<PeerUrl>>>,
	// TODO: Implement gossip protocol instead of broadcasting everything to everyone
	shutdown: Arc<AtomicBool>,
	key_chain: NodeKeyChain,
	pub server_handle: Option<ServerHandle>,
	pub config: NodeConfig,
	pub parameters: Parameters,
}

pub(crate) const STARTING_SLOT_SECOND: u64 = 0;

// TODO: AT THE END CHANGE THIS NUMBER FOR THE EPOCH SECOND OF THE TIME THE CRYPTO IS RELEASED
impl Node {
	pub async fn default(version: u32) -> Self {
		let ntp_client = AsyncSntpClient::new();
		let slot_time = ntp_client.synchronize("time.google.com").await
			.expect("Unable to sync with NTP server")
			.datetime()
			.unix_timestamp()
			.expect("Time went backwards") - Duration::from_secs(STARTING_SLOT_SECOND);
		let parameters = Parameters::default();

		Self {
			version,
			current_slot: Arc::new(AtomicU64::new(slot_time.as_millis() as u64 / parameters.technical_parameters.slot_duration as u64)),
			blockchain: Arc::new(RwLock::new(BlockChain::init(parameters))),
			shutdown: Arc::new(AtomicBool::new(false)),
			key_chain: NodeKeyChain::random(),
			server_handle: None,
			config: NodeConfig::default(),
			peers: Arc::new(Default::default()), // TODO: Load from default file
			parameters,
		}
	}
	pub async fn new(version: u32, config: NodeConfig, parameters: Parameters) -> Self {
		let ntp_client = AsyncSntpClient::new();
		let slot_time = ntp_client.synchronize("time.google.com").await
			.expect("Unable to sync with NTP server")
			.datetime()
			.unix_timestamp()
			.expect("Time went backwards") - Duration::from_secs(STARTING_SLOT_SECOND);
		let peers = config.trusted_peers.clone();
		Self {
			version,
			current_slot: Arc::new(AtomicU64::new(slot_time.as_millis() as u64 / parameters.technical_parameters.slot_duration as u64)),
			blockchain: Arc::new(RwLock::new(BlockChain::init(parameters))),
			shutdown: Arc::new(AtomicBool::new(false)),
			key_chain: NodeKeyChain::random(),
			server_handle: None,
			config,
			peers: Arc::new(RwLock::new(peers)),
			parameters,
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
		log::info!("Started main loop thread");

		let mut self_clone = self.clone();
		tokio::spawn(async move {
			self_clone.heart_beat_thread().await;
		});
		log::info!("Started heart beat thread");
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

	pub async fn heart_beat_thread(&mut self) {
		const SLOTS_PER_RE_SYNC: u32 = 128; // Every 128 slots the client will re-sync FIXME: Maybe change this value or choose a more appropriated one?

		let ntp_client = AsyncSntpClient::new();

		// Sync to slot
		timing::sync_to_slot(&ntp_client, self.parameters.technical_parameters.slot_duration as u64).await;

		let interval: Duration = Duration::from_millis(self.parameters.technical_parameters.slot_duration as u64); // THE INTERVAL
		let mut counter = 0;
		loop {
			let start = Instant::now();
			// Do something
			let current_slot = self.current_slot.fetch_add(1, Ordering::Relaxed) + 1; // Update and get the current block

			// FIXME: ADD THE HASH OF THE PREVIOUS EPOCH AS ENTRY IN THE VRF
			let vrf_proving_key = VrfSk::from_bytes(&self.key_chain.vrf_key_pair.0).unwrap();
			let active_slot_coeff = self.parameters.technical_parameters.active_slot_coefficient;

			let node_stake = 1; // FIXME: Actually get the node stake
			let total_staked = 2; // FIXME: Actually get the total stake

			let lottery = Lottery::run_lottery(current_slot, active_slot_coeff, &[0u8; 32], &vrf_proving_key, node_stake, total_staked); // FIXME: Replace last_epoch_hash with actual last epoch hash

			if let Some((random_number, proof)) = lottery {
				// LOTERY WON!!!
				self.forge_new_block(current_slot, random_number, proof).await;
			}

			if counter % 10 == 0 {
				let chain = self.blockchain.read().await;
				log::info!("Info: Height: {}", chain.get_height());
			}

			counter += 1;
			if counter & 1 == 0 && self.is_shutdown() {
				break;
			}
			if counter % SLOTS_PER_RE_SYNC == 0 {
				timing::sync_to_slot(&ntp_client, self.parameters.technical_parameters.slot_duration as u64).await; // Re-sync sleep
				match timing::get_accurate_slot(&ntp_client, self.parameters.technical_parameters.slot_duration as u64).await { // Re-set the slot
					Ok(actual_current_slot) => {
						self.current_slot.store(actual_current_slot, Ordering::Relaxed);
					}
					Err(err) => {
						log::error!("Unable to synchronize with NTP server. Error: {}", err);
					}
				}
			} else {
				spin_sleep::sleep(interval - start.elapsed()); // Regular sleep
			}
		}
	}
	/// Forges a new block when the lottery is won
	pub async fn forge_new_block(&mut self, current_slot: u64, random_number: [u8; 32], proof: VrfProof) {
		log::info!("Lottery won!");
		
		let start = Instant::now();
		let mut chain = self.blockchain.write().await;

		let prev_hash = chain.get_last_block().header.hash;

		let mut transactions = Vec::new();

		for tx in chain.mempool.iter() {
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
			current_slot,
			prev_hash,
			self.key_chain.wallet_key_pair.0,
			self.key_chain.vrf_key_pair.1,
			random_number,
			&proof);
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
	pub fn get_current_slot(&self) -> u64 {
		self.current_slot.load(Ordering::Relaxed)
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