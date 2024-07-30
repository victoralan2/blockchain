use std::fmt::Error;
use std::fs::{File, OpenOptions};
use std::path::Path;

use log::Log;
use p256::pkcs8::der::Writer;
use serde::{Deserialize, Serialize};
use sled::{Db};

use crate::core::block::Block;
use crate::core::Hashable;
use crate::data_storage::BaseDirectory;
use crate::data_storage::blockchain_storage::{BLOCKCHAIN_DIRECTORY_NAME, CHAIN_DIRECTORY_NAME, INDEX_DIRECTORY_NAME, METADATA_FILE_NAME, UNDO_DIRECTORY_NAME, UNDO_INDEX_DIRECTORY_NAME};
use crate::data_storage::blockchain_storage::undo_items::{UndoBlock};
use crate::network::models::InvDataType::Block;
use crate::network::standard::{standard_deserialize, standard_serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ChainMetadata { // TODO: Make sure everything here is updated each and every write
	/// This is equivalent to the height of the best block **minus one**
	length: usize,
	best_block: [u8; 32],
}
impl ChainMetadata {
	pub fn load() -> Self {
		let file_location = format!("{}/{}/{}", 
									BaseDirectory::get_base_directory(), 
									BLOCKCHAIN_DIRECTORY_NAME, 
									METADATA_FILE_NAME);
		if Path::exists(file_location.as_str().as_ref()) {
			let data = std::fs::read_to_string(&file_location).unwrap();
			serde_json::from_str(&data).expect("Unable to deserialize metadata")
		} else {
			File::create(&file_location).unwrap();
			let metadata = Self {
				length: 0,
				best_block: [0u8; 32],
			};
			metadata.save();
			metadata
		}
	}
	pub fn get_len(&mut self) -> usize{
		self.length
	}
	pub fn set_best_block(&mut self, best_block: [u8; 32]) {
		self.best_block = best_block;
		self.save();
	}
	pub fn get_best_block(&mut self) -> [u8; 32] {
		self.best_block
	}
	pub fn save(&self) {
		let mut metadata_file = OpenOptions::new().write(true).create(true).truncate(true).open(format!("{}/{}/{}", 
																										BaseDirectory::get_base_directory(), 
																										BLOCKCHAIN_DIRECTORY_NAME, 
																										METADATA_FILE_NAME)).expect("Unable to open chain metadata file");
		let new_data = serde_json::to_string_pretty(self).expect("Unable to serialize metadata to json");
		metadata_file.write(new_data.as_bytes()).expect("Unable to write metadata to file");
	}
}
#[derive(Clone)]
pub struct ChainDB {
	chain_db: Db,
	index_to_hash_db: Db,
	undo_block_db: Db,
	index_undo_block_db: Db,
	chain_metadata: ChainMetadata,
}
impl ChainDB {
	pub fn print_debug(&self) {
		for i in 0..self.get_length() {
			dbg!(self.get_block_by_height(i).unwrap().header.height, self.get_block_by_height(i).unwrap().header.nonce);
		}
	}
	/// Does not check if there is already a block in that index or hash. Must be checked before calling this function.
	pub fn push_block_to_end(&mut self, block: &Block, undo_block: &UndoBlock) -> anyhow::Result<()> {
		let serialized_block = standard_serialize(&block)?;
		let hash = block.calculate_hash();
		self.chain_db.insert(hash, serialized_block)?;
		self.index_to_hash_db.insert(block.header.height.to_be_bytes(), &hash)?;
		
		self.chain_db.flush()?;
		self.index_to_hash_db.flush()?;
		
		self.index_undo_block_db.insert(undo_block.height.to_be_bytes(), &undo_block.original_hash)?;
		let serialized_undo = standard_serialize(&undo_block)?;
		self.undo_block_db.insert(undo_block.original_hash, serialized_undo)?;
		
		self.undo_block_db.flush()?;
		self.index_undo_block_db.flush()?;
		
		self.chain_metadata.length += 1;
		self.chain_metadata.best_block = block.calculate_hash();
		self.chain_metadata.save();
		Ok(())
	}
	
	pub fn get_undo_block(&self, block_hash: &[u8; 32]) -> anyhow::Result<Option<UndoBlock>> {
		if let Some(undo_block) = self.undo_block_db.get(block_hash)? {
			let undo_block: UndoBlock = standard_deserialize(&undo_block.to_vec())?;
			Ok(Some(undo_block))
		} else {
			Ok(None)
		}
	}
	
	pub fn get_best_block(&self) -> Option<Block> {
		let best_block_height = self.chain_metadata.length - 1;
		let best_block_hash = self.index_to_hash_db.get(best_block_height.to_be_bytes()).ok()??.to_vec();
		let best_block = self.chain_db.get(best_block_hash).ok()??.to_vec();
		standard_deserialize(&best_block).ok()
	}

	/// Undoes the block effect and returns the block itself if undid
	pub fn undo_block(&mut self, block_hash: &[u8; 32]) -> anyhow::Result<Option<Block>>{
		if let Some(undo_block) = self.get_undo_block(block_hash)? {
			if undo_block.height != self.get_length() - 1 {
				return Ok(None);
			}
			if let Some(block) = self.chain_db.get(undo_block.original_hash)? {
				let block: Block = standard_deserialize(&block.to_vec())?;
				self.chain_db.remove(undo_block.original_hash)?;
				self.index_to_hash_db.remove(undo_block.height.to_be_bytes())?;
				self.undo_block_db.remove(undo_block.original_hash)?;
				self.index_undo_block_db.remove(undo_block.height.to_be_bytes())?;

				// Update metadata
				self.chain_metadata.length -= 1;
				self.chain_metadata.best_block = self.get_best_block().expect("Unable to update metadata best block").header.hash;

				return Ok(Some(block));
			}
		}
		Ok(None)
	}

	pub fn get_block(&self, hash: [u8; 32]) -> Option<Block> {
		self.chain_db.get(hash)
			.ok()
			.flatten()
			.and_then(|block| {
				standard_deserialize(&block)
					.ok()
			})
	}
	pub fn get_block_by_height(&self, height: usize) -> Option<Block> {
		let hash = self.index_to_hash_db.get(height.to_be_bytes()).ok()??;
		let block = standard_deserialize(&self.chain_db.get(hash).ok()??).ok()?; // This gets the block based on the key (the hash) and serializes it (yeah, there is a lot of "?")
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
		self.index_to_hash_db.flush()?;
		self.undo_block_db.flush()?;
		self.index_undo_block_db.flush()?;
		self.chain_metadata.save();
		
		Ok(())
	}
}
impl Default for ChainDB {
	fn default() -> Self {
		let base_directory = BaseDirectory::get_base_directory();
		let chain_db = sled::open(format!("{}/{}/{}", 
										  base_directory, 
										  BLOCKCHAIN_DIRECTORY_NAME, 
										  CHAIN_DIRECTORY_NAME)).expect("failed to write to database"); // FIXME: Change the file for the actual Db location
		let index_to_hash_db = sled::open(format!("{}/{}/{}", 
												  base_directory, 
												  BLOCKCHAIN_DIRECTORY_NAME, 
												  INDEX_DIRECTORY_NAME)).expect("failed to write to database"); // FIXME: Change the file for the actual Db location
		let undo_block_db = sled::open(format!("{}/{}/{}", 
											   base_directory, 
											   BLOCKCHAIN_DIRECTORY_NAME, 
											   UNDO_DIRECTORY_NAME)).expect("failed to write to database"); // FIXME: Change the file for the actual Db location
		let index_undo_block_db = sled::open(format!("{}/{}/{}", 
													 base_directory, 
													 BLOCKCHAIN_DIRECTORY_NAME, 
													 UNDO_INDEX_DIRECTORY_NAME)).expect("failed to write to database"); // FIXME: Change the file for the actual Db location

		let chain_metadata = ChainMetadata::load();
		let mut this = Self {
			chain_db,
			index_to_hash_db,
			undo_block_db,
			index_undo_block_db,
			chain_metadata,
		};
		if this.is_empty() {
			this.push_block_to_end(&Block::genesis(), &UndoBlock::genesis()).expect("Unable to insert genesis block");
		}
		
		this
	}
}