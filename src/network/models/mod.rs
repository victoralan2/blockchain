use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::core::block::{Block, BlockContent, BlockHeader};
use crate::core::utxo::transaction::Transaction;

pub mod http_errors;

#[derive(Clone, Deserialize, Serialize, Copy)]
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
	pub(crate) block_locator_object: Vec<[u8; 32]>,
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
	/// The block locator object contains element where index 0 is the last block of the blockchain and the subsequent elements descend in height
	pub(crate) block_locator_object: Vec<[u8; 32]>,
}
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct BlockchainInfo {
	pub(crate) version: u32,
	pub(crate) height: usize,
	pub(crate) best_block_header: BlockHeader,
	pub(crate) mempool_size: usize,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct Headers {
	/// The headers vector contains block headers from oldest to newest
	pub(crate) headers: Vec<BlockHeader>,

	/// This value is true if the sender was able to send all headers in a single response, if false, more requests are needed to get all headers
	pub(crate) sent_all_headers: bool,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct BlocksData {
	pub(crate) version: u32,
	pub(crate) blocks_data: Vec<Option<BlockContent>>,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct SendPeers {
	pub(crate) peers: HashSet<String>,
}
#[derive(Copy, Clone, Deserialize, Serialize)]
pub enum HttpScheme {
	HTTP,
	HTTPS,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct PairUp {
	pub(crate) version: u32,
	pub(crate) method: HttpScheme,
	pub(crate) port: u16,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct Unpair {
	pub(crate) version: u32,
	pub(crate) method: HttpScheme,
	pub(crate) port: u16,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct NewTransaction {
	pub(crate) version: u32,
	pub(crate) transaction: Transaction,
	// TODO: Some extra info from https://www.blockchain.com/explorer/es/explorer/api/blockchain_api
}
#[derive(Clone, Deserialize, Serialize)]
pub struct NewBlock {
	pub(crate) version: u32,
	pub(crate) block: Block,
	// TODO: Some extra info from https://www.blockchain.com/explorer/es/explorer/api/blockchain_api
}