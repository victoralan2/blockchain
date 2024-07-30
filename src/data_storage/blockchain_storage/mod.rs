pub mod chain_database;
pub mod mempool_database;
pub mod utxo_database;
pub mod undo_items;

pub const BLOCKCHAIN_DIRECTORY_NAME: &str = "blockchain";
pub const CHAIN_DIRECTORY_NAME: &str = "chain-db";
pub const INDEX_DIRECTORY_NAME: &str = "index-db";
pub const UNDO_DIRECTORY_NAME: &str = "undo-db";
pub const UNDO_INDEX_DIRECTORY_NAME: &str = "undo-index-db";
pub const METADATA_FILE_NAME: &str = "metadata.json";


