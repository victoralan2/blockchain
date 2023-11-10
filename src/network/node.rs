use std::collections::HashSet;
use std::error::Error;
use std::io;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use bincode::deserialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::timeout;
use crate::core::address::P2PKHAddress;
use crate::core::block::Block;
use crate::core::blockchain::BlockChain;
use crate::core::utxo::transaction::Transaction;
use crate::network::{Broadcast, Listener, Subscribe};
use crate::network::capsule::{BlockCapsule, TransactionCapsule};
use crate::network::request_types::RequestType;

pub struct NodeConfig {
	pub(crate) default_transaction_ttl: u32,
	pub(crate) default_block_ttl: u32,
	pub(crate) listing_port: u16,
}
impl Default for NodeConfig {
	fn default() -> Self {
		NodeConfig {
			default_transaction_ttl: 10,
			default_block_ttl: 10,
			listing_port: 25565,
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
	pub async fn start_node(self, seed_peers: Vec<SocketAddr>) -> Result<Arc<Mutex<Self>>, io::Error> {
		let port = self.config.listing_port;
		let state = Arc::new(Mutex::new(self));
		let s1 = state.clone();
		tokio::spawn(async move {
			Self::listen(s1, port).await;
		});
		for peer in seed_peers {
			timeout(Duration::from_millis(500), Self::subscribe(state.clone(),peer, port)).await.ok();
		}
		Ok(state)
	}
	pub async fn mine_block(keep_mining: Arc<AtomicBool>, state: Arc<Mutex<Self>>, miner_address: P2PKHAddress) {
		let blocked_state = state.lock().await;
		let mut blockchain = blocked_state.blockchain.lock().await;
		while keep_mining.load(Ordering::Relaxed) {

			if let Some(block) = blockchain.mine_one_block(miner_address, keep_mining.clone()) {
				if blockchain.add_block(&block) {
					let capsule = BlockCapsule {
						block,
						time_to_live: blocked_state.config.default_transaction_ttl,
					};
					Self::broadcast(&capsule, RequestType::NewBlock.into(), state.clone()).await;
				}
			}
		}
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

	pub async fn initial_block_download(&mut self, peer: SocketAddr) -> Result<(), Box<dyn Error>>{
		let mut conn = TcpStream::connect(peer).await?;
		conn.write_u8(RequestType::InitialBlockDownload.into()).await?;
		let mut blockchain = self.blockchain.lock().await;
		blockchain.clear();
		let mut block_data = Vec::new();
		while conn.read_to_end(&mut block_data).await? != 0 {
			let block = deserialize::<Block>(&block_data)?;
			if blockchain.add_block(&block) {
				conn.write_all(b"ok").await?;
			} else {
				conn.shutdown().await?;
				return Err(Box::new(io::Error::new(ErrorKind::InvalidInput, "Received block was not valid")))
			}
		}
		Ok(())
	}

}