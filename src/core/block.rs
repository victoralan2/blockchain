use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::consensus::lottery::Lottery;
use crate::core::address::P2PKHAddress;
use crate::core::blockchain::BlockChain;
use crate::core::Hashable;
use crate::core::utxo::transaction::Transaction;
use crate::crypto::hash::merkle::calculate_merkle_root;
use crate::crypto::vrf::{VrfPk, VrfProof};

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
pub struct BlockHeader {
	pub hash: [u8; 32],
	pub height: usize,
	pub previous_hash: [u8; 32],
	pub slot: u64,
	pub merkle_root: [u8; 32],
	pub vrf: [u8; 32],
	#[serde(with = "BigArray")]
	pub vrf_proof: [u8; 96],
	pub forger_vrf_public_key: [u8; 32],
	pub forger_address: P2PKHAddress,
}
pub type BlockContent = Vec<Transaction>;
#[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
pub struct Block {
	pub header: BlockHeader,
	pub transactions: BlockContent,
}

impl Block {
	pub fn new(height: usize, transactions: Vec<Transaction>, slot: u64, previous_hash: [u8; 32], reward_address: P2PKHAddress, forger_vrf_public_key: [u8; 32], vrf: [u8; 32], vrf_proof: &VrfProof) -> Self {
		let header = BlockHeader {
			hash: [0u8; 32],
			height,
			previous_hash,
			slot,
			merkle_root: [0u8; 32],
			vrf,
			vrf_proof: vrf_proof.to_bytes(),
			forger_vrf_public_key,
			forger_address: reward_address,
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
			height: 0,
			previous_hash: EXTRA_ENTROPY,
			slot: 0u64,
			merkle_root: [0u8; 32],
			vrf: [0u8; 32],
			vrf_proof: [0u8; 96],
			forger_vrf_public_key: [0u8; 32],
			forger_address: P2PKHAddress::null(),
		};
		let mut block = Block {
			transactions: vec![],
			header,
		};
		block.update_hash();
		block
	}
	pub fn verify_vrf(&self, current_slot: u64, active_slot_coefficient: f32, last_epoch_hash: [u8; 32], node_stake: u64, total_staked: u64) -> bool {
		let vrf =  self.header.vrf;
		let vrf_proof =  self.header.vrf_proof;
		if let Ok(vrf_pk) = VrfPk::from_bytes(&self.header.forger_vrf_public_key) {
			Lottery::verify_vrf_lottery(current_slot, &last_epoch_hash, vrf, vrf_proof, &vrf_pk)
		} else {
			false
		}
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

			if !(is_transaction_valid) {
				return false;
			}
		}
		true
	}
	pub fn is_valid(&self, blockchain: &BlockChain) -> bool {
		// TODO
		
		let height = blockchain.get_height();
		let is_block_correct = self.is_correct();
		if !is_block_correct {
			return false
		}
		
		// TODO: VERIFY THE VRF
		
		// TODO: Check for leader validity

		// TODO: DOING: I was trying to make so that when the block can replace the last one is valid. Problem: Transactions are bitches bc last block interfeers with that and SHIT FUCK
		for tx in &self.transactions {
			if !tx.is_valid(blockchain) {
				return false
			}
		}
		
		if let Some(previous) = blockchain.get_block_at(height - 1) {
			let is_previous_hash_correct = self.header.previous_hash == previous.header.hash;
			if !is_previous_hash_correct {
				return false;
			}
		} 
		
		let is_height_correct = self.header.height == blockchain.get_height();
		if !is_height_correct {
			return false
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