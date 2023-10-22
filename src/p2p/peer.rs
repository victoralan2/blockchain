use std::net::SocketAddr;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Peer {
	pub address: SocketAddr
}