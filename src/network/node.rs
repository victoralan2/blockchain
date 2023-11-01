use std::collections::HashSet;
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Mutex;
use crate::core::address::P2PKHAddress;
use crate::core::blockchain::BlockChain;
use crate::core::utxo::transaction::Transaction;
use crate::network::{Broadcast, Listener, Subscribe};
use crate::network::capsule::TransactionCapsule;
use crate::network::request_types::RequestType;

pub struct NodeConfig {
	pub(crate) default_transaction_ttl: u32,
	pub(crate) default_block_ttl: u32,
}
impl Default for NodeConfig {
	fn default() -> Self {
		NodeConfig {
			default_transaction_ttl: 10,
			default_block_ttl: 10,
		}
	}
}
pub struct Node {
	pub blockchain: Arc<Mutex<BlockChain>>,
	pub subscriber_list: HashSet<SocketAddr>,
	pub subscribed_to: HashSet<SocketAddr>,
	pub listen: bool,
	pub config: NodeConfig,
}

impl Node {
	pub fn new(blockchain: BlockChain, config: NodeConfig, ) -> Self {
		Node {
			blockchain: Arc::new(Mutex::new(blockchain)),
			subscriber_list: Default::default(),
			subscribed_to: Default::default(),
			listen: false,
			config,
		}
	}
	pub async fn start_node(self, seed_peers: Vec<SocketAddr>, listening_port: u16, request_port: u16) -> Result<Arc<Mutex<Self>>, io::Error> {
		let state = Arc::new(Mutex::new(self));
		let s1 = state.clone();
		tokio::spawn(async move {
			Self::listen(s1, listening_port).await;
		});
		for peer in seed_peers {
			Self::subscribe(state.clone(),peer, request_port).await.ok();
		}
		Ok(state)
	}
	pub async fn new_transaction(state: Arc<Mutex<Self>>, transaction: Transaction) -> bool {
		let blocked_state = state.lock().await;
		let mut blockchain = blocked_state.blockchain.lock().await;
		if blockchain.add_transaction_to_mempool(&transaction) {
			let capsule = TransactionCapsule {
				transaction,
				time_to_live: blocked_state.config.default_transaction_ttl,
			};
			Self::broadcast(&capsule, RequestType::NewTransaction.into(), state.clone()).await;
		}
		true
	}
	pub async fn mine_block(keep_mining: Arc<AtomicBool>, state: Arc<Mutex<Self>>, miner_address: P2PKHAddress) {
		let blocked_state = state.lock().await;
		let mut blockchain = blocked_state.blockchain.lock().await;
		while keep_mining.load(Ordering::Release) {
			let block = blockchain.mine_one_block(miner_address);
		}
	}
}