use std::io;
use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::net::Shutdown::Both;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};
use tokio::task;

use crate::core::block::Block;
use crate::core::blockchain::BlockChain;
use crate::core::blockdata::{BlockData, Transaction};
use crate::p2p::datatypes::{DataType};
use crate::p2p::peer::Peer;

pub struct Node {
	pub test: String,
	pub tests: Vec<u64>,
	pub blockchain: BlockChain,
	peers: Vec<Peer>,
	pub open_connections: Vec<TcpStream>,
}

impl Node {
	pub fn new(blockchain: BlockChain, peers: Vec<Peer>) -> Self {
		Node {
			test: "".to_string(),
			tests: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0],
			blockchain,
			peers,
			open_connections: vec![],
		}
	}
	pub fn add_peer(&mut self, peer: Peer) {
		// TODO: Do some checks
		self.peers.push(peer);
	}
	pub async fn start_server(this: Arc<Mutex<Self>>, address: SocketAddr) -> Result<(), io::Error> {
		let listener = TcpListener::bind(address)?;
		let clone = this.clone();
		tokio::spawn(async move {
			Self::event_loop(clone).await;
		});

		while let Some(stream) = listener.incoming().next() {
			let mut stream = stream?;
			Self::handshake_in(&mut stream)?;
			stream.set_nonblocking(true)?;

			this.lock().expect("Unable to lock").open_connections.push(stream);
		}

		Ok(())
	}
	async fn event_loop(this: Arc<Mutex<Node>>) {
		loop {
			for p in this.lock().expect("Unable to lock").open_connections.iter_mut() {
				let mut data_type = Vec::new();
				if let Err(e) = p.read_to_end(&mut data_type) {
					println!("{}", e);
					p.shutdown(Both).ok();
				}
				p.set_nonblocking(false).ok();
				this.lock().expect("Unable to lock").handle_message(p, data_type).ok();
				p.set_nonblocking(true).ok();
			}
		}
	}
	pub fn handle_message(&mut self, connection: &mut TcpStream, data_type: Vec<u8>) -> Result<(), io::Error> {
		// Todo: Handle the given data
		if let Ok(data_type) = DataType::try_from(data_type) {
			match data_type {
				DataType::Disconnect => {
					connection.write_all(b"ok")?;
					connection.shutdown(Both)?;
				}
				DataType::P2PDiscover => {
					let response: Vec<SocketAddr> = self.peers.iter().map(|p|p.address).collect();
					if let Ok(msg) = bincode::serialize(&response) {
						connection.write_all(&msg)?;
					} else {
						return Err(io::Error::new(ErrorKind::InvalidData, "Unable to serialize response"))
					}
				}
				DataType::NewBlockData => {
					let mut size = [0u8; 4];
					connection.read_exact(&mut size)?;
					let size = u32::from_be_bytes(size);
					let mut buffer = vec![0; size as usize];
					connection.read_exact(&mut buffer)?;
					if let Ok(block_data) = bincode::deserialize::<BlockData>(&buffer) {
						let is_valid = self.blockchain.add_data_to_mempool(block_data);
						connection.write_all(&bincode::serialize(&is_valid).expect("Unexpected error serializing boolean"))?;
					}
				}
				DataType::NewBlock => {
					let mut size = [0u8; 4];
					connection.read_exact(&mut size)?;
					let size = u32::from_be_bytes(size);
					let mut buffer = vec![0; size as usize];
					connection.read_exact(&mut buffer)?;
					if let Ok(block) = bincode::deserialize::<Block>(&buffer) {
						let is_valid = self.blockchain.add_block(block);
						connection.write_all(&bincode::serialize(&is_valid).expect("Unexpected error serializing boolean"))?;
					}
				}
				DataType::SyncRequest => {
					let mut request = Vec::new();
					connection.read_to_end(&mut request)?;
					match request.as_slice() {
						b"GetBlock" => {

						},
						b"GetChain" => {
							// let this = Arc::new(self);
							// let self_copy = this.clone();
							// task::spawn( async move {
							// 	connection.write_all(b"get_chain").unwrap();
							// 	let mut response = Vec::new();
							// 	connection.read_to_end(&mut response);
							// 	if response == b"get" {
							// 		connection.write_all(self_copy.test.as_bytes());
							// 	} else {
							// 		return;
							// 	}
							// });
						},
						_ => {
							return Err(io::Error::new(ErrorKind::InvalidInput, "Invalid request"))
						}
					}
				}
			}
		} else {
			connection.shutdown(Both)?;
		}
		Ok(())
	}
	pub fn connect_to_peers(&mut self, address: SocketAddr) -> Result<(), io::Error> {
		for p in &self.peers {
			let mut stream = TcpStream::connect(p.address)?;
			Self::handshake_out(&mut stream)?;
			stream.set_nonblocking(true)?;
			self.open_connections.push(stream);
		}
		Ok(())
	}
	pub fn broadcast_block(&mut self, block: Block) {

	}
	pub fn discover_peers(&mut self) {

	}
	fn handshake_out(connection: &mut TcpStream) -> Result<(), io::Error>{
		connection.write_all(b"node_hello")?;
		let mut buf = [0u8; 7];
		connection.read_exact(&mut buf)?;
		if &buf == b"node_ok" {
			Ok(())
		} else {
			Err(io::Error::new(ErrorKind::ConnectionAborted, "Handshake failed"))
		}
	}
	fn handshake_in(connection: &mut TcpStream) -> Result<(), io::Error>{
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