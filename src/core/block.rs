use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::core::address::P2PKHAddress;
use crate::core::blockchain::BlockChain;
use crate::core::Hashable;
use crate::core::utxo::transaction::Transaction;
use crate::crypto::hash::merkle::calculate_merkle_root;

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Debug)]
pub struct BlockHeader {
	pub hash: [u8; 32],
	pub nonce: u64,
	pub height: usize,
	pub previous_hash: [u8; 32],
	pub merkle_root: [u8; 32],
	pub miner_address: P2PKHAddress,
}
pub type BlockContent = Vec<Transaction>;
#[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
pub struct Block {
	pub header: BlockHeader,
	pub transactions: BlockContent,
}

impl Block {
	pub fn new(height: usize, transactions: Vec<Transaction>, previous_hash: [u8; 32], reward_address: P2PKHAddress, nonce: u64) -> Self {
		let header = BlockHeader {
			hash: [0u8; 32],
			nonce,
			height,
			previous_hash,
			merkle_root: [0u8; 32],
			miner_address: reward_address,
		};
		let mut block = Block { header, transactions };
		block.update_hash();
		block
	}
	pub fn genesis() -> Self {
		
		// TODO: Choose extra entropy better
		const EXTRA_ENTROPY:  [u8; 32] = [60, 92, 162, 110, 82, 120, 10, 250, 102, 233, 226, 182, 114, 155, 80, 178, 35, 57, 107, 9, 122, 187, 253, 38, 160, 225, 171, 15, 110, 230, 47, 21];
		
		let header = BlockHeader {
			hash: [0u8; 32],
			nonce: 0,
			height: 0,
			previous_hash: EXTRA_ENTROPY,
			merkle_root: [0u8; 32],
			miner_address: P2PKHAddress::null(),
		};
		let mut block = Block {
			transactions: vec![],
			header,
		};
		block.update_hash();
		block
	}
	pub fn verify_proof_of_work(&self) -> bool {
		todo!(); // TODO: USE
	}
	pub fn calculate_merkle_tree(&self) -> [u8; 32]{
		let mut hashes: Vec<[u8; 32]> = Vec::new();
		for tx in &self.transactions {
			hashes.push(tx.calculate_hash());
		}
		calculate_merkle_root(hashes)
	}
	
	/// Returns whether the block is correct and has no inconsistencies
	pub fn is_correct(&self) -> bool {
		let is_hash_correct = self.calculate_hash() == self.header.hash;
		let is_merkle_tree_correct = self.calculate_merkle_tree() == self.header.merkle_root;
		if !(is_merkle_tree_correct && is_hash_correct) {
			return false;
		}
		let mut input_tx_list = HashSet::new();
		for tx in &self.transactions {
			// CHECKS IF THERE ARE TWO INPUTS USING SAME OUTPUT
			for input in &tx.input_list {
				if !input_tx_list.insert(input.calculate_hash()) {
					return false;
				}
			}
			let is_transaction_valid = tx.is_valid_heuristic();

			if !is_transaction_valid {
				return false;
			}
		}
		true
	}
}
pub enum BlockValidity {
	/// Meaning is not valid
	NotValid, 
	/// /// Meaning that it is better than the current last block of the blockchain and should REPLACE it
	BetterThanLastBlock, 
	Valid,
}
pub enum InvalidityReason { // TODO
	/// Last hash is different
	WrongContext, 
	InvalidHash,
	/// The VRF was not valid
	InvalidVRF,
	/// There is some transaction that is not valid in the block
	InvalidTransaction, 
	TooLarge,
}