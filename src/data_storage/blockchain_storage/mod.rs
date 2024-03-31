use crate::data_storage::blockchain_storage::chain_database::ChainDB;
use crate::data_storage::blockchain_storage::mempool_database::MempoolDB;
use crate::data_storage::blockchain_storage::utxo_database::UTXODB;

pub mod chain_database;
pub mod mempool_database;
pub mod utxo_database;

#[derive(Default)]
pub struct BlockchainDB {
	blockchain: ChainDB,
	mempool: MempoolDB,
	utxo_set: UTXODB
}


