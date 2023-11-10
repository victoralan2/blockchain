use std::collections::HashSet;
use std::error::Error;
use std::io;
use std::net::{SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use bincode::serialize;
use local_ip_address::local_ip;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use crate::network::capsule::{BlockCapsule, Capsule, TransactionCapsule};
use crate::network::node::Node;
use crate::network::request_types::RequestType;

pub mod node;
pub mod request_types;
pub mod capsule;
pub mod errors;

#[async_trait]
pub trait Listener {
	const SUBSCRIBE_MODE: u8 = 0;
	const REQUEST_MODE: u8 = 1;
	const AUTO_SUBSCRIBE_MODE: u8 = 2;
	async fn listen(state: Arc<Mutex<Node>>, port: u16);
	async fn handle_handshake(connection: TcpStream, state: Arc<Mutex<Node>>) -> Result<(), Box<dyn Error>>;
	async fn handle_peer(connection: TcpStream, state: Arc<Mutex<Node>>) -> Result<(), Box<dyn Error>>;
}
#[async_trait]
pub trait Broadcast {
	async fn broadcast<T: Capsule>(capsule: &T, request_type: u8,state: Arc<Mutex<Node>>) ;
}
#[async_trait]
pub trait Subscribe {
	async fn subscribe(state: Arc<Mutex<Node>>, addr: SocketAddr, port: u16) -> Result<(), io::Error>;
}
#[async_trait]
impl Listener for Node {
	async fn listen(state: Arc<Mutex<Node>>, port: u16){
		state.lock().await.listen = true;
		let listener = TcpListener::bind(SocketAddr::new(local_ip().unwrap(), port)).await.unwrap();
		while let Ok((conn, _)) = listener.accept().await{
			if !state.lock().await.listen {
				return;
			}
			let state = state.clone();
			tokio::spawn(async move{
				Self::handle_handshake(conn, state.clone()).await.ok();
			});
		}

	}
	async fn handle_handshake(mut connection: TcpStream, state: Arc<Mutex<Node>>)  -> Result<(), Box<dyn Error>>{
		let timeout_duration = Duration::from_secs(1);
		if let Ok(Ok(mode)) = tokio::time::timeout(timeout_duration, connection.read_u8()).await {
			match mode {
				Self::SUBSCRIBE_MODE => {
					if let Ok(port) = connection.read_u16().await {
						let mut addr = connection.peer_addr().unwrap();
						addr.set_port(port);
						state.lock().await.subscriber_list.insert(addr);
					}
				},
				Self::REQUEST_MODE => {
					Self::handle_peer(connection, state).await?;
				},
				Self::AUTO_SUBSCRIBE_MODE => {
					// TODO: Maybe check for some things
					let state_clone = state.clone();
					let config = &state.lock().await.config;
					Self::subscribe(state_clone, connection.peer_addr()?, config.listing_port).await?;
				},
				_ => {
					connection.shutdown().await?;
				}
			}
		}
		Ok(())
	}
	async fn handle_peer(mut connection: TcpStream, state: Arc<Mutex<Node>>) -> Result<(), Box<dyn Error>>{
		let request_type = connection.read_u8().await?;
		if let Ok(request_type) = RequestType::try_from(request_type) {
			match request_type {
				RequestType::P2PDiscover => {
					let state = state.lock().await;
					let mut peers: HashSet<SocketAddr> = Default::default();
					for &p in state.subscribed_to.iter().chain(state.subscriber_list.iter()) {
						peers.insert(p);
					}
					let serialized_list = serialize(&peers)?;
					connection.write_all(&serialized_list).await?;
				}
				RequestType::NewTransaction => {
					let mut buf = Vec::new();
					connection.read_to_end(&mut buf).await?;
					let mut tx_capsule: TransactionCapsule = bincode::deserialize(&buf)?;
					if let Some(tx) = tx_capsule.consume() {
						if tx_capsule.is_alive() {
							let node = state.lock().await;
							let mut bc = node.blockchain.lock().await;
							if bc.add_transaction_to_mempool(&tx) {
								let state = state.clone();
								tokio::spawn(async move{
									Self::broadcast(&tx_capsule, RequestType::NewTransaction.into(), state).await;
								});
							}
						}
					}
					connection.shutdown().await.ok();
				}
				RequestType::NewBlock => {
					let mut buf = Vec::new();
					connection.read_to_end(&mut buf).await?;
					let mut block_capsule: BlockCapsule = bincode::deserialize(&buf)?;
					if let Some(block) = block_capsule.consume() {
						if block_capsule.is_alive() {
							let node = state.lock().await;
							let mut bc = node.blockchain.lock().await;
							if bc.add_block(&block) && block_capsule.is_alive() {
								let state = state.clone();
								tokio::spawn(async move{
									Self::broadcast(&block_capsule, RequestType::NewBlock.into(), state).await;
								});
							}
						}
					}
					connection.shutdown().await.ok();
				}
				RequestType::SyncRequest => {
					// First, send last block
					// If not valid, then determine the sync starting point
					// When starting point is selected, the other node must compute UTXO Set
					// When starting point is determined, start sending data block by block, while other node builds up the UTXO set
					// Compare UTXO sets
					// Todo: Snap-sync
				}
				RequestType::InitialBlockDownload => {
					let state = state.lock().await;
					let len = state.blockchain.lock().await.get_len();
					for i in 0..len {
						let blockchain = state.blockchain.lock().await;
						let next_block = blockchain.get_block_at(i).unwrap();
						let data = serialize(next_block).unwrap();
						drop(blockchain);
						connection.write_all(&data).await.ok();
						if connection.read(&mut []).await.unwrap_or(0) == 0 {
							break;
						}
					}
					connection.shutdown().await.ok();
				}
			}
		}
		Ok(())
	}
}
#[async_trait]
impl Broadcast for Node {
	async fn broadcast<T: Capsule>(capsule: &T, request_type: u8, state: Arc<Mutex<Node>>) {
		let timeout = Duration::from_secs(1);
		let sub_list = &state.lock().await.subscriber_list;
		for subscriber in sub_list {
			if let Ok(conn) = std::net::TcpStream::connect_timeout(subscriber, timeout) {
				if let Ok(mut conn) = TcpStream::from_std(conn) {
					if let Ok(data) = serialize(&capsule) {
						conn.write_u8(request_type).await.ok();
						conn.write_all(&data).await.ok();
						conn.shutdown().await.ok();
					}
				}
			}
		}
	}
}
#[async_trait]
impl Subscribe for Node {
	async fn subscribe(state: Arc<Mutex<Node>>, addr: SocketAddr, port: u16) -> Result<(), io::Error> {
		let mut stream = TcpStream::connect(addr).await?;
		stream.write_u8(Self::SUBSCRIBE_MODE).await?;
		stream.write_u16(port).await?;
		state.lock().await.subscribed_to.insert(addr);
		Ok(())
	}
}