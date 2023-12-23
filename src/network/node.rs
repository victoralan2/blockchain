use std::collections::HashSet;
use std::error::Error;
use std::hint::spin_loop;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use actix_web::{App, HttpServer};
use actix_web::dev::ServerHandle;
use actix_web::web::{Data};
use local_ip_address::local_ip;
use rand::prelude::IteratorRandom;
use rand::thread_rng;
use reqwest::{Client, Url};
use serde::{Serialize};
use spin_sleep::SpinStrategy;
use tokio::spawn;
use tokio::sync::RwLock;

use crate::core::address::P2PKHAddress;
use crate::core::blockchain::{BlockChain};
use crate::core::parameters::Parameters;
use crate::core::utxo::transaction::Transaction;
use crate::network::{config};
use crate::network::config::config_routes;
use crate::network::models::{HttpScheme, NewBlock, NewTransaction, PairUp};
use crate::network::sender::Sender;
use crate::network::standard::standard_serialize;

#[derive(Clone)]
pub struct NodeConfig {
	pub(crate) listing_port: u16,
	pub(crate) http_scheme: HttpScheme,
	pub(crate) max_peers: usize,
	pub(crate) peer_cycle_count: usize,
	pub(crate) trusted_peers: HashSet<Url>,
}

#[derive(Clone)]
pub struct Node {
	pub version: u32,
	pub current_slot: Arc<AtomicU64>,
	pub blockchain: Arc<RwLock<BlockChain>>,
	pub peers: Arc<RwLock<HashSet<Url>>>,
	pub shutdown: Arc<AtomicBool>,
	pub server_handle: Option<ServerHandle>,
	pub config: NodeConfig,
	pub parameters: Parameters,
}

impl Node {
	pub fn new(version: u32, config: NodeConfig, parameters: Parameters) -> Self {
		Self {
			version,
			current_slot: Arc::new(Default::default()),
			blockchain: Arc::new(RwLock::new(BlockChain::new_empty(parameters))),
			shutdown: Arc::new(AtomicBool::new(false)),
			server_handle: None,
			config,
			peers: Arc::new(Default::default()),
			parameters,
		}
	}
	pub fn start(mut self) {
		self.start_node();
		let mut self_clone = self.clone();
		spawn(async move {
			self_clone.main_loop().await;
		});
		let mut self_clone = self.clone();
		spawn(async move {
			self_clone.heart_beat_thread().await;
		});
	}
	pub fn start_node(&mut self)  {
		// STARTS THE NODE, THE ENTRY POINT.
		let app_state = Data::new(self.clone());
		let server = HttpServer::new(move || {
			App::new()
				.app_data(app_state.clone())
				.configure(config_routes)
		})
			.bind(format!("{}:{}", local_ip().unwrap(), self.config.listing_port)).unwrap().run();
		let handle = server.handle();
		tokio::spawn(server);

		self.server_handle = Some(handle);
	}

	pub async fn heart_beat_thread(&mut self) {
 		let interval: Duration = Duration::from_millis(self.parameters.technical_parameters.slot_duration as u64); // THE INTERVAL OF 2000 MILLISECONDS PER SLOT
		loop {
			let start = Instant::now();


			// Do something
			self.current_slot.fetch_add(1, Ordering::Release);
			spin_sleep::SpinSleeper::new(0).with_spin_strategy(SpinStrategy::SpinLoopHint).sleep(interval - start.elapsed());
		}
	}

	pub async fn main_loop(&mut self) {
		let mut counter = 0u32; // Counter to replace peers
		const REPLACE_PEER_TIME: u32 = 10u32; // In seconds
		while !self.shutdown.load(Ordering::Relaxed) {


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
				spawn(async move {self_copy.discover_peers().await;});
			}

			if counter > REPLACE_PEER_TIME {
				let mut self_copy = self.clone();
				counter = 0;
				spawn(async move {self_copy.cycle_peers().await;});
			}

			counter += 1;
			tokio::time::sleep(Duration::from_secs(1)).await;
		}
		// If shutdown
		self.shutdown().await;
	}
	pub async fn discover_peers(&mut self) {
		// TODO
	}
	async fn discover_n_peers(&self, n: u32) -> HashSet<Url> {
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
		let new_peers: HashSet<Url> = self.discover_n_peers(peer_cycle_count as u32).await;

		let mut peers = self.peers.write().await;
		let peers_to_remove: Vec<Url> = peers.iter()
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
		if let Some(handle) = &self.server_handle {
			handle.stop(true).await;
		}
		// TODO: Save to file or smth
	}

	// Returns weather it was added or not
	pub async fn new_transaction(&self, transaction: Transaction) -> bool{
		if self.blockchain.write().await.add_transaction_to_mempool(&transaction) {
			let msg = NewTransaction {
				version: self.version,
				transaction
			};
			let peers = self.peers.read().await.clone();
			Self::broadcast_transaction(peers, &msg).await;
			true
		} else {
			false
		}
	}

	pub async fn broadcast_transaction(peers: HashSet<Url>, tx: &NewTransaction) {
		let urls: HashSet<Url> = peers.iter().map(|url| {
			let mut new_url = url.clone();
			new_url.set_path(config::NEW_TRANSACTION_URL);
			new_url
		}).collect();

		Self::broadcast_bytes(urls, tx).await;
	}
	pub async fn broadcast_block(peers: HashSet<Url>, block: &NewBlock) {
		let urls: HashSet<Url> = peers.iter().map(|url| {
			let mut new_url = url.clone();
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
				let client = client.clone();
				handles.push(spawn( async move {
					Sender::send_bytes(&client, url, bytes).await
				}));
			}
		}
		for h in handles {
			h.await.ok();
		}
	}
}