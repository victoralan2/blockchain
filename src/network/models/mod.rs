pub mod http_errors;

use std::net::SocketAddr;
use serde::{Deserialize, Serialize};
use crate::core::block::{Block, BlockHeader};
use crate::core::utxo::transaction::Transaction;

#[derive(Clone, Deserialize, Serialize)]
pub enum InvDataType {
	Transaction,
	Block,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct Inv {
	pub(crate) data_type: InvDataType,
	pub(crate) hashes: Vec<[u8; 32]>
}
#[derive(Clone, Deserialize, Serialize)]
pub struct GetBlocks {
	pub(crate) version: u32,
	pub(crate) last_known_block: u32,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct GetData {
	pub(crate) version: u32,
	pub(crate) data_type: InvDataType,
	pub(crate) hashes: Vec<[u8; 32]>,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct GetHeaders {
	pub(crate) version: u32,
	pub(crate) last_known_block: u32,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Headers {
	pub(crate) headers: Vec<BlockHeader>,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct SendPeers {
	pub(crate) version: u32,
	pub(crate) peers: Vec<SocketAddr>,
}
#[derive(Copy, Clone, Deserialize, Serialize)]
pub enum HttpScheme {
	HTTP,
	HTTPS,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct Subscribe {
	pub(crate) version: u32,
	pub(crate) method: HttpScheme,
	pub(crate) port: u16,
	pub(crate) subdirectory: String,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct NewTransaction {
	pub(crate) version: u32,
	pub(crate) transaction: Transaction, // TODO: This an actual transaction
	// TODO: Some extra info from https://www.blockchain.com/explorer/es/explorer/api/blockchain_api
}

#[derive(Clone, Deserialize, Serialize)]
pub struct NewBlock {
	pub(crate) version: u32,
	pub(crate) block: Block, // TODO: This an actual block
	// TODO: Some extra info from https://www.blockchain.com/explorer/es/explorer/api/blockchain_api
}