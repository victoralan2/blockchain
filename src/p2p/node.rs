use std::{io, usize};
use std::collections::HashSet;
use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::net::Shutdown::Both;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};
use futures::executor::block_on;
use tokio::task;

use crate::core::block::Block;
use crate::core::blockchain::{BlockChain, BlockChainConfig};
use crate::core::utxo::transaction::Transaction;
use crate::crypto::hash::merkle::calculate_merkle_root;
use crate::p2p::chain_sync::{sync_in, sync_out};
use crate::p2p::request_types::{RequestType};
use crate::p2p::peer::Peer;

pub struct Node {
	pub blockchain: Arc<Mutex<BlockChain>>,
	peers: HashSet<Peer>,
	pub open_connections: Vec<TcpStream>,
}

impl Node {
	pub fn new(blockchain: BlockChain, peers: HashSet<Peer>) -> Self {
		Node {
			blockchain: Arc::new(Mutex::new(blockchain)),
			peers,
			open_connections: vec![],
		}
	}
	pub fn add_peer(&mut self, peer: Peer) {
		// TODO: Do some checks
		if let Ok(mut stream) = TcpStream::connect(peer.address) {
			if Self::handshake_in(&mut stream).is_ok() {
				self.peers.insert(peer);
			}
		}
	}
	pub async fn start_server(&mut self, address: SocketAddr) -> Result<(), io::Error> {
		let listener = TcpListener::bind(address)?;
		// let clone = this.clone();
		// tokio::spawn(async move {
		// 	Self::event_loop(clone).await;
		// });

		while let Some(stream) = listener.incoming().next() {
			let mut stream = stream?;
			Self::handshake_in(&mut stream)?;
			let mut request_type = Vec::new();
			if let Ok(_) = stream.read_to_end(&mut request_type) {
				self.handle_message(stream, request_type).unwrap();
			} else {
				stream.shutdown(Both).ok();
			}
		}

		Ok(())
	}
	pub fn handle_message(&mut self, mut connection: TcpStream, request_type: Vec<u8>) -> Result<(), io::Error> {
		// Todo: Handle the given data
		if let Ok(request_type) = RequestType::try_from(request_type) {
			match request_type {
				RequestType::P2PDiscover => {
					let response: Vec<SocketAddr> = self.peers.iter().map(|p| p.address).collect();
					if let Ok(msg) = bincode::serialize(&response) {
						connection.write_all(&msg)?;
					} else {
						return Err(io::Error::new(ErrorKind::InvalidData, "Unable to serialize response"));
					}
					connection.shutdown(Both)?;
				}
				RequestType::NewTransaction => {
					let mut buffer = Vec::new();
					connection.read_to_end(&mut buffer)?;
					if let Ok(tx) = bincode::deserialize::<Transaction>(&buffer) {
						let is_valid = self.blockchain.lock().expect("Unable to lock blockchain").add_transaction_to_mempool(block_data);
						connection.write_all(&bincode::serialize(&is_valid).expect("Unexpected error serializing boolean"))?;
					}
					connection.shutdown(Both)?;
				}
				RequestType::NewBlock => {
					let mut buffer = Vec::new();
					connection.read_to_end(&mut buffer)?;
					if let Ok(block) = bincode::deserialize::<Block>(&buffer) {
						let is_valid = self.blockchain.lock().expect("Unable to lock blockchain").add_block(block);
						connection.write_all(&bincode::serialize(&is_valid).expect("Unexpected error serializing boolean"))?;
					}
					connection.shutdown(Both)?;
				}
				RequestType::SyncRequest => {
					let mut request = Vec::new();
					connection.read_to_end(&mut request)?;
					match request.as_slice() {
						b"GetLastBlock" => {
							let blockchain = self.blockchain.lock().expect("Unable to lock");
							let last_block = blockchain.get_last_block();
							if let Some(block) = last_block {
								let serialized_block = bincode::serialize(block);
								if let Ok(serialized_block) = serialized_block {
									connection.write_all(&serialized_block).expect("Unable to write");
								} else {
									connection.write_all(b"err").expect("Unable to write");
								}
							}
							connection.shutdown(Both)?;
						}
						b"GetChain" => {
							let blockchain = self.blockchain.clone();
							task::spawn(async move {
								if let Err(err) = sync_in(connection, blockchain) {
									println!("{}", err);
								}
							});
						}
						_ => {
							return Err(io::Error::new(ErrorKind::InvalidInput, "Invalid request"));
						}
					}
				}
			}
		}
		Ok(())
	}
	pub fn sync_peer(&mut self, peer: Peer) -> Result<(), io::Error>{
		let mut connection = TcpStream::connect(peer.address)?;
		Self::handshake_out(&mut connection)?;
		connection.write_all(b"SyncRequest")?;
		connection.write_all(b"GetLastBlock")?;
		let mut last_block_data = Vec::new();
		connection.read_to_end(&mut last_block_data)?;
		connection.shutdown(Both)?;



		let mut blockchain = self.blockchain.lock().expect("Unable to lock");
		if let Ok(last_block) = bincode::deserialize::<Block>(&last_block_data) {
			let my_last_block = blockchain.get_last_block();
			if let Some(my_last_block) = my_last_block {
				if last_block.header.previous_hash == my_last_block.hash {
					// Peer is one ahead
					blockchain.add_block(last_block);
				} else if last_block.hash == my_last_block.hash {
					// We are at the same level
				} else if last_block.index > my_last_block.index {
					// Peer has different chain
					let mut connection = TcpStream::connect(peer.address)?;
					Self::handshake_out(&mut connection)?;
					connection.write_all(b"SyncRequest")?;
					connection.write_all(b"GetChain")?;
					let new_chain = sync_out(connection, &blockchain)?;
					blockchain.replace(&new_chain);
				}
			}
		}


		Ok(())
	}
	/// Returns the acceptance count of that block and the total count of peers
	pub fn broadcast_block(&mut self, block: Block) -> Result<(u32, u32), io::Error> {
		let mut accepted = 0;
		for p in &self.peers {
			if let Ok(mut stream) = TcpStream::connect(p.address) {
				Self::handshake_out(&mut stream).ok();
				stream.write_all(b"NewBlock").ok();
				if let Ok(block_bytes) = bincode::serialize(&block) {
					stream.write_all(&block_bytes).ok();
					let mut is_valid = Vec::new();
					stream.read_to_end(&mut is_valid).ok();
					if let Ok(is_valid) = bincode::deserialize::<bool>(&is_valid) {
						if is_valid {
							accepted += 1;
						}
					}
				} else {
					stream.shutdown(Both).ok();
				}
			}
		}
		Ok((accepted, self.peers.len() as u32))
	}
	pub fn broadcast_transaction(&mut self, block_data: Transaction) -> Result<(u32, u32), io::Error> {
		let mut accepted = 0;
		for p in &self.peers {
			if let Ok(mut stream) = TcpStream::connect(p.address) {
				Self::handshake_out(&mut stream).ok();
				stream.write_all(b"NewBlock").ok();
				if let Ok(block_bytes) = bincode::serialize(&block_data) {
					stream.write_all(&block_bytes).ok();
					let mut is_valid = Vec::new();
					stream.read_to_end(&mut is_valid).ok();
					if let Ok(is_valid) = bincode::deserialize::<bool>(&is_valid) {
						if is_valid {
							accepted += 1;
						}
					}
				} else {
					stream.shutdown(Both).ok();
				}
			}
		}
		Ok((accepted, self.peers.len() as u32))
	}

	pub fn discover_peers(&mut self) {
		for p in self.peers {
			if let Ok(mut stream) = TcpStream::connect(p.address) {
				Self::handshake_out(&mut stream).ok();
				stream.write_all(b"P2PDiscover").ok();
				let mut response = Vec::new();
				stream.read_to_end(&mut response).ok();
				if let Ok(peer_list) = bincode::deserialize::<Vec<SocketAddr>>(&response) {
					for p in peer_list {
						self.add_peer(Peer { address: p });
					}
				}
			}
		}
	}
	fn handshake_out(connection: &mut TcpStream) -> Result<(), io::Error> {
		connection.write_all(b"node_hello")?;
		let mut buf = [0u8; 7];
		connection.read_exact(&mut buf)?;
		if &buf == b"node_ok" {
			Ok(())
		} else {
			Err(io::Error::new(ErrorKind::ConnectionAborted, "Handshake failed"))
		}
	}
	fn handshake_in(connection: &mut TcpStream) -> Result<(), io::Error> {
		let mut buf = [0u8; 10];
		connection.read_exact(&mut buf)?;
		if &buf == b"node_hello" {
			connection.write_all(b"node_ok")?;
			Ok(())
		} else {
			Err(io::Error::new(ErrorKind::ConnectionAborted, "Handshake failed"))
		}
	}
}