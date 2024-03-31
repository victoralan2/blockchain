use std::fs::File;
use std::path::Path;

use log::Log;
use p256::pkcs8::der::Writer;
use serde::{Deserialize, Serialize};
use sled::Db;

use crate::core::block::Block;
use crate::core::Hashable;
use crate::data_storage::BaseDirectory;
use crate::network::standard::{standard_deserialize, standard_serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ChainMetadata { // TODO: Make sure everything here is updated each and every write
	/// This is equivalent to the height of the best block **minus one**
	length: usize,
	best_block: [u8; 32],
}
impl ChainMetadata {
	pub fn load() -> Self {
		let file_location = format!("{}/blockchain/metadata.json", BaseDirectory::get_base_directory());
		if !Path::exists(file_location.as_str().as_ref()) {
			std::fs::File::create(&file_location).unwrap();
			let data = std::fs::read_to_string(&file_location).unwrap();
			let metadata = serde_json::from_str(&data).expect("Unable to deserialize metadata");
			return metadata;
		}
		Self {
			length: 0,
			best_block: [0; 32],
		}

	}
	pub fn set_len(&mut self, length: usize) {
		self.length = length;
		self.flush();
	}
	pub fn get_len(&mut self) -> usize{
		self.length
	}
	pub fn set_best_block(&mut self, best_block: [u8; 32]) {
		self.best_block = best_block;
		self.flush();
	}
	pub fn get_best_block(&mut self) -> [u8; 32] {
		self.best_block
	}
	pub fn flush(&self) {
		let mut metadata_file = File::open(format!("{}/blockchain/metadata.json", BaseDirectory::get_base_directory())).expect("Unable to open chain metadata file");
		let new_data = serde_json::to_string_pretty(self).expect("Unable to serialize metadata to json");
		metadata_file.write(new_data.as_bytes()).expect("Unable to write metadata to file");
	}
}
#[derive(Clone)]
pub struct ChainDB {
	chain_db: Db,
	index_to_hash_db: Db,
	chain_metadata: ChainMetadata,

}
impl ChainDB {
	/// Does not check if there is already a block in that index or hash. Must be checked before calling this function.
	pub fn push_block_to_end(&mut self, block: &Block) -> anyhow::Result<()> {
		let serialized_block = standard_serialize(&block)?;
		let hash = block.calculate_hash();
		self.chain_db.insert(hash, serialized_block)?;
		self.index_to_hash_db.insert(block.header.height.to_be_bytes(), &hash)?;
		self.chain_db.flush()?;
		self.index_to_hash_db.flush()?;
		self.chain_metadata.length = block.header.height + 1;
		self.chain_metadata.best_block = block.calculate_hash();
		self.chain_metadata.flush();
		Ok(())
	}
	
	pub fn get_best_block(&self) -> Option<Block> {
		let best_block_height = self.chain_metadata.length - 1;
		let best_block_hash = self.index_to_hash_db.get(&best_block_height.to_be_bytes()).ok()??;
		let best_block = self.chain_db.get(best_block_hash).ok()??;
		Some(standard_deserialize(&best_block.to_vec()).ok()?)
	}

	pub fn get_block(&self, hash: [u8; 32]) -> Option<Block> {
		self.chain_db.get(hash)
			.ok()
			.flatten()
			.map(|block| {
				standard_deserialize(&block.to_vec())
					.ok()
			})
			.flatten()
	}
	pub fn get_block_by_height(&self, height: usize) -> Option<Block> {
		let hash = self.index_to_hash_db.get(height.to_be_bytes()).ok()??;
		let block = serde_json::from_slice(&self.chain_db.get(hash).ok()??).ok()?; // This gets the block based on the key (the hash) and serializes it (yeah, there is a lot of "?")
		block
	}
	fn is_empty(&self) -> bool {
		self.chain_db.is_empty() || self.index_to_hash_db.is_empty()
	}
	
	pub fn get_length(&self) -> usize {
		self.chain_metadata.length
	}
	pub fn flush(&mut self) -> sled::Result<()> {
		self.chain_db.flush()?;
		Ok(())
	}
}
impl Default for ChainDB {
	fn default() -> Self {
		let base_directory = BaseDirectory::get_base_directory();
		let chain_db = sled::open(format!("{}/blockchain/chain-db", base_directory)).expect("failed to write to database"); // FIXME: Change the file for the actual Db location
		let index_to_hash_db = sled::open(format!("{}/blockchain/index-db", base_directory)).expect("failed to write to database"); // FIXME: Change the file for the actual Db location
		
		let chain_metadata = ChainMetadata::load();
		let mut this = Self {
			chain_db,
			index_to_hash_db,
			chain_metadata,
		};
		if this.is_empty() {
			this.push_block_to_end(&Block::genesis()).expect("Unable to insert genesis block");
		}
		
		this
	}
}